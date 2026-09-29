//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Table operations: merge, update, TSV import/export, key deletes, and lookups of table data
//! and references across the open packs and the dependencies.

use anyhow::{anyhow, Result};
use rayon::prelude::*;

use serde_json::{Number, Value};

use std::borrow::Cow;
use std::collections::{HashMap, HashSet};
use std::path::Path;

use rpfm_extensions::dependencies::{Dependencies, KEY_DELETES_TABLE_NAME, TableReferences};
use rpfm_extensions::merge::{db_baseline, delta_merge_db, delta_merge_loc, loc_baseline, MergeConflict, MergeOptions, MergeResolution};

use rpfm_ipc::api::ApiError;
use rpfm_ipc::api::files::{ASSEMBLY_KIT_TABLE_FILE_NAME, FileRef, FileSource};
use rpfm_ipc::api::tables::{ColumnInfo, ColumnReference, DEFAULT_ROWS_LIMIT, FilterOp, GetTableRows, RowFilter, TableInfo, TableRow, TableRows};
use rpfm_ipc::helpers::{DataSource, RFileInfo};

use rpfm_lib::files::{Container, ContainerPath, db::DB, DecodeableExtraData, FileType, RFile, RFileDecoded, table::{DecodedData, local::TableInMemory, Table}};
use rpfm_lib::schema::{Definition, DefinitionPatch, Field};
use rpfm_lib::utils::current_time;

use super::{SessionState, loaded_schema, pack, pack_mut};

/// A [`RowFilter`] ready to check rows: its column index resolved, and its value lowercased if it ignores case.
struct PreparedFilter {
    column_index: usize,
    op: FilterOp,
    value: String,
    ignore_case: bool,
}

/// Result of merging tables.
#[derive(Debug)]
pub enum MergeOutcome {

    /// The tables were merged into a new file at this path.
    Merged(String),

    /// Rows that couldn't be reconciled automatically. Nothing was written.
    Conflicts(Vec<MergeConflict>),
}

/// Result of a delta merge attempt: either a finished file ready to insert, or the conflicts blocking it.
enum DeltaMergeOutcome {
    Merged(RFile),
    Conflicts(Vec<MergeConflict>),
}

/// Location of a row in a table: its source, the path of the table, and the column and row indexes.
pub type RowLocation = (DataSource, String, usize, usize);

/// A row referencing a value: its source, the key of its pack (empty for dependencies), the path
/// of its table, the name and index of the referencing column, and the row index.
pub type ReferenceLocation = (DataSource, String, String, String, usize, usize);

impl SessionState {

    /// Merges compatible tables of a pack into a new one.
    ///
    /// # Arguments
    ///
    /// * `pack_key` - Key of the pack with the tables.
    /// * `paths` - Paths of the tables to merge.
    /// * `merged_path` - Path of the merged table.
    /// * `delete_source_files` - If the merged tables are deleted afterwards.
    /// * `options` - How to merge the rows.
    ///
    /// # Returns
    ///
    /// The path of the merged table, or the conflicts that prevented the merge.
    pub fn merge_files(&mut self, pack_key: &str, paths: &[ContainerPath], merged_path: &str, delete_source_files: bool, options: &MergeOptions) -> Result<MergeOutcome> {
        let pack = pack_mut(&mut self.packs, pack_key)?;
        let files_to_merge = pack.files_by_paths(paths, false);

        let merge_result = if *options.delta_merge() {
            delta_merge_files(&files_to_merge, merged_path, &self.dependencies, options.resolutions())?
        } else {
            DeltaMergeOutcome::Merged(RFile::merge(&files_to_merge, merged_path)?)
        };

        match merge_result {
            DeltaMergeOutcome::Merged(file) => {
                let _ = pack.insert(file);

                // Make sure to only delete the files if they're not the destination file.
                if delete_source_files {
                    paths.iter()
                        .filter(|path| merged_path != path.path_raw())
                        .for_each(|path| { pack.remove(path); });
                }

                Ok(MergeOutcome::Merged(merged_path.to_owned()))
            },
            DeltaMergeOutcome::Conflicts(conflicts) => Ok(MergeOutcome::Conflicts(conflicts)),
        }
    }

    /// Updates a table of a pack to the version of the table in the game files.
    ///
    /// # Returns
    ///
    /// The old and new version, and the names of the fields deleted and added.
    pub fn update_table(&mut self, pack_key: &str, path: &ContainerPath) -> Result<(i32, i32, Vec<String>, Vec<String>)> {
        let path = path.path_raw();
        let file = pack_mut(&mut self.packs, pack_key)?.file_mut(path, false)
            .ok_or_else(|| anyhow!("File not found in the open Pack: {}", path))?;

        let decoded = file.decoded_mut().map_err(|_| anyhow!("File with the following path undecoded: {}", path))?;
        Ok(self.dependencies.update_db(decoded)?)
    }

    /// Exports a table to a TSV file.
    ///
    /// # Arguments
    ///
    /// * `pack_key` - Key of the pack with the table. Only used if `data_source` is a pack.
    /// * `internal_path` - Path of the table in its source.
    /// * `external_path` - Path of the TSV file to write.
    /// * `data_source` - Where the table is.
    /// * `tsv_keys_first` - If the TSV uses the old column order, with keys first.
    pub fn export_tsv(&mut self, pack_key: &str, internal_path: &str, external_path: &Path, data_source: DataSource, tsv_keys_first: bool) -> Result<()> {
        let schema = loaded_schema(&self.schema)?;
        let file = match data_source {
            DataSource::PackFile => self.packs.get_mut(pack_key).and_then(|pack| pack.file_mut(internal_path, false)),
            DataSource::ParentFiles => self.dependencies.file_mut(internal_path, false, true).ok(),
            DataSource::GameFiles => self.dependencies.file_mut(internal_path, true, false).ok(),
            DataSource::AssKitFiles => return Err(anyhow!("Exporting a TSV from the Assembly Kit is not yet supported.")),
            DataSource::ExternalFile => return Err(anyhow!("Exporting a TSV from a external file is not yet supported.")),
        }.ok_or_else(|| anyhow!("File with the following path not found in the Pack: {}", internal_path))?;

        file.tsv_export_to_path(external_path, schema, tsv_keys_first)?;
        Ok(())
    }

    /// Replaces a table of a pack with the contents of a TSV file, keeping the table's GUID.
    ///
    /// # Returns
    ///
    /// The imported table.
    pub fn import_tsv(&mut self, pack_key: &str, internal_path: &str, external_path: &Path) -> Result<RFileDecoded> {
        let file = pack_mut(&mut self.packs, pack_key)?.file_mut(internal_path, false)
            .ok_or_else(|| anyhow!("File with the following path not found in the Pack: {}", internal_path))?;

        // Preserve the original table GUID, as set_decoded would replace it with a fresh
        // one from the imported table, making the import non-idempotent for DB tables.
        let original_guid = match file.decoded() {
            Ok(RFileDecoded::DB(table)) => Some(table.guid().to_owned()),
            _ => None,
        };

        let imported = RFile::tsv_import_from_path(external_path, &self.schema)?;
        let mut decoded = imported.decoded()?.clone();
        if let (RFileDecoded::DB(table), Some(guid)) = (&mut decoded, original_guid) {
            table.set_guid(guid);
        }

        file.set_decoded(decoded.clone())?;
        Ok(decoded)
    }

    /// Changes values referencing an edited key in every table of a pack.
    ///
    /// # Arguments
    ///
    /// * `pack_key` - Key of the pack to edit.
    /// * `table_name` - Name of the table whose key was edited.
    /// * `definition` - Definition of the table whose key was edited.
    /// * `changes` - Edited field, and its value before and after the edit, of each change.
    ///
    /// # Returns
    ///
    /// The paths of the edited files, and their info.
    pub fn cascade_edition(&mut self, pack_key: &str, table_name: &str, definition: &Definition, changes: &[(Field, String, String)]) -> Result<(Vec<ContainerPath>, Vec<RFileInfo>)> {
        let pack = pack_mut(&mut self.packs, pack_key)?;
        let edited_paths = match self.schema {
            Some(ref schema) => changes.iter()
                .flat_map(|(field, value_before, value_after)| DB::cascade_edition(pack, schema, table_name, field, definition, value_before, value_after))
                .collect::<Vec<_>>(),
            None => vec![],
        };

        let files_info = pack.files_by_paths(&edited_paths, false).into_par_iter().map(From::from).collect();
        Ok((edited_paths, files_info))
    }

    /// Returns the paths of the tables of a pack with the provided table name.
    pub fn table_paths_by_name(&self, pack_key: &str, table_name: &str) -> Result<Vec<String>> {
        let path = ContainerPath::Folder(format!("db/{table_name}/"));
        Ok(pack(&self.packs, pack_key)?.files_by_type_and_paths(&[FileType::DB], &[path], true)
            .iter()
            .map(|file| file.path_in_container_raw().to_owned())
            .collect())
    }

    /// Adds rows to a key deletes table of a pack, one per key.
    ///
    /// # Arguments
    ///
    /// * `pack_key` - Key of the pack with the key deletes table.
    /// * `table_file_name` - File name of the key deletes table.
    /// * `key_table_name` - Name of the table the keys belong to.
    /// * `keys` - Keys to delete.
    ///
    /// # Returns
    ///
    /// The path of the edited table, or `None` if the table wasn't found.
    pub fn add_keys_to_key_deletes(&mut self, pack_key: &str, table_file_name: &str, key_table_name: &str, keys: &HashSet<String>) -> Result<Option<ContainerPath>> {
        let path = ContainerPath::File(format!("db/{KEY_DELETES_TABLE_NAME}/{table_file_name}"));
        let mut files = pack_mut(&mut self.packs, pack_key)?.files_by_type_and_paths_mut(&[FileType::DB], &[path], true);

        let Some(file) = files.first_mut() else { return Ok(None) };
        let Ok(RFileDecoded::DB(db)) = file.decoded_mut() else { return Ok(None) };

        for key in keys {
            db.data_mut().push(vec![
                DecodedData::StringU8(key.to_owned()),
                DecodedData::StringU8(key_table_name.to_owned()),
            ]);
        }

        Ok(Some(file.path_in_container()))
    }

    /// Returns the valid values of the reference columns of a table, from the open packs and the dependencies.
    ///
    /// # Arguments
    ///
    /// * `table_name` - Name of the table.
    /// * `definition` - Definition of the table.
    /// * `force_local_generation` - If the references to the open packs are regenerated even if they're cached.
    ///
    /// # Returns
    ///
    /// The references of each reference column, by column index. Empty if there is no schema.
    pub fn reference_data(&mut self, table_name: &str, definition: &Definition, force_local_generation: bool) -> HashMap<i32, TableReferences> {
        let Some(ref schema) = self.schema else { return HashMap::new() };

        if force_local_generation || !self.dependencies.local_tables_references().contains_key(table_name) {
            self.dependencies.generate_local_definition_references(schema, table_name, definition);
        }

        self.dependencies.db_reference_data(schema, &self.packs, table_name, definition, &None)
    }

    /// Finds the first row with a value in a column of a table.
    ///
    /// Searches the open packs (starting with `pack_key`), then the parent packs, the game files and the Assembly Kit tables.
    ///
    /// # Arguments
    ///
    /// * `pack_key` - Key of the pack to search first.
    /// * `ref_table` - Name of the table, without the `_tables` suffix.
    /// * `ref_column` - Name of the column. If it's localised, the first key column is searched instead.
    /// * `ref_data` - Values to search. Only the first one is used.
    ///
    /// # Returns
    ///
    /// Where the row is.
    pub fn go_to_definition(&self, pack_key: &str, ref_table: &str, ref_column: &str, ref_data: &[String]) -> Result<RowLocation> {
        let value = ref_data.first().ok_or_else(|| anyhow!("No value to search for."))?;
        let table_name = format!("{ref_table}_tables");
        let table_folders = ContainerPath::db_table_folders(&table_name);

        // Search first in the pack that sent the request (if still open), then in the rest of the open packs.
        let packs_to_search = self.packs.get(pack_key).into_iter()
            .chain(self.packs.iter().filter(|(key, _)| *key != pack_key).map(|(_, pack)| pack));

        for pack in packs_to_search {
            if let Some(location) = find_in_db_files(&pack.files_by_paths(&table_folders, true), ref_column, value, DataSource::PackFile) {
                return Ok(location);
            }
        }

        for (data_source, include_vanilla, include_parent) in [(DataSource::ParentFiles, false, true), (DataSource::GameFiles, true, false)] {
            if let Ok(files) = self.dependencies.db_data(&table_name, include_vanilla, include_parent) {
                if let Some(location) = find_in_db_files(&files, ref_column, value, data_source) {
                    return Ok(location);
                }
            }
        }

        if let Some(table) = self.dependencies.asskit_only_db_tables().get(&table_name) {
            if let Some((column_index, row_index)) = find_in_db(table, ref_column, value) {
                return Ok((DataSource::AssKitFiles, format!("db/{table_name}/{ASSEMBLY_KIT_TABLE_FILE_NAME}"), column_index, row_index));
            }
        }

        Err(anyhow!("source_data_for_field_not_found"))
    }

    /// Finds the first row of a Loc file with a key, searching a pack, then the parent packs and then the game files.
    ///
    /// # Returns
    ///
    /// Where the row is.
    pub fn go_to_loc(&self, pack_key: &str, loc_key: &str) -> Result<RowLocation> {
        let pack_files = pack(&self.packs, pack_key)?.files_by_type(&[FileType::Loc]);
        if let Some(location) = find_in_loc_files(&pack_files, loc_key, DataSource::PackFile) {
            return Ok(location);
        }

        for (data_source, include_vanilla, include_parent) in [(DataSource::ParentFiles, false, true), (DataSource::GameFiles, true, false)] {
            if let Ok(files) = self.dependencies.loc_data(include_vanilla, include_parent) {
                if let Some(location) = find_in_loc_files(&files, loc_key, data_source) {
                    return Ok(location);
                }
            }
        }

        Err(anyhow!("loc_key_not_found"))
    }

    /// Finds every row referencing a value, in a pack, the parent packs and the game files.
    ///
    /// # Arguments
    ///
    /// * `pack_key` - Key of the pack to search.
    /// * `reference_map` - Columns to search, by table name.
    /// * `value` - Value to search.
    ///
    /// # Returns
    ///
    /// The rows referencing the value.
    pub fn search_references(&self, pack_key: &str, reference_map: &HashMap<String, Vec<String>>, value: &str) -> Result<Vec<ReferenceLocation>> {
        let paths = reference_map.keys().flat_map(|table_name| ContainerPath::db_table_folders(table_name)).collect::<Vec<ContainerPath>>();
        let files = pack(&self.packs, pack_key)?.files_by_paths(&paths, true);

        let mut references = vec![];

        // Tag each hit in the pack with its key so the UI can open the right tab when several
        // packs are open with files at the same path.
        for (table_name, columns) in reference_map {
            for file in &files {
                if file.db_table_name_from_path() == Some(table_name.as_str()) {
                    references.extend(references_in_file(file, columns, value, DataSource::PackFile, pack_key));
                }
            }
        }

        // Dependencies results are navigated through the dependencies tree, which has a single root
        // per source, so their pack key is empty.
        for (data_source, include_vanilla, include_parent) in [(DataSource::ParentFiles, false, true), (DataSource::GameFiles, true, false)] {
            for (table_name, columns) in reference_map {
                if let Ok(tables) = self.dependencies.db_data(table_name, include_vanilla, include_parent) {
                    references.par_extend(tables.par_iter().flat_map(|table| references_in_file(table, columns, value, data_source, "")));
                }
            }
        }

        Ok(references)
    }

    /// Returns the table, column and key values a loc key was generated from, if it can be found in the dependencies.
    pub fn loc_key_source(&self, loc_key: &str) -> Option<(String, String, Vec<String>)> {
        self.dependencies.loc_key_source(loc_key)
    }

    /// Returns the distinct values of a column of a table.
    ///
    /// # Arguments
    ///
    /// * `table_name` - Name of the table, like `factions_tables`.
    /// * `column_name` - Name of the column.
    /// * `include_packs` - If the open packs are searched.
    /// * `include_dependencies` - If the game files and parent packs are searched.
    pub fn column_values(&self, table_name: &str, column_name: &str, include_packs: bool, include_dependencies: bool) -> HashSet<String> {
        let packs = if include_packs { Some(&self.packs) } else { None };
        self.dependencies.db_values_from_table_name_and_column_name(packs, table_name, column_name, include_dependencies, include_dependencies)
    }

    /// Returns the names of the tables in the game files.
    pub fn dependency_table_names(&self) -> Vec<String> {
        self.dependencies.vanilla_loose_tables().keys()
            .chain(self.dependencies.vanilla_tables().keys())
            .cloned()
            .collect()
    }

    /// Returns the version of a table in the game files.
    ///
    /// Startpos, twad and CEO tables not in the game files return the latest version in the schema.
    pub fn dependency_table_version(&self, table_name: &str) -> Result<i32> {
        if !self.dependencies.is_vanilla_data_loaded(false) {
            return Err(ApiError::DependenciesNotLoaded.into());
        }

        if let Some(version) = self.dependencies.db_version(table_name) {
            return Ok(version);
        }

        if !(table_name.starts_with("start_pos_") || table_name.starts_with("twad_") || table_name.starts_with("ceo")) {
            return Err(anyhow!("Table not found in the game files."));
        }

        loaded_schema(&self.schema)?.definitions_by_table_name(table_name)
            .and_then(|definitions| definitions.first())
            .map(|definition| *definition.version())
            .ok_or_else(|| anyhow!("There are no definitions for this specific table."))
    }

    /// Returns the definition of a table with the version it has in the game files.
    pub fn dependency_table_definition(&self, table_name: &str) -> Result<Definition> {
        if !self.dependencies.is_vanilla_data_loaded(false) {
            return Err(ApiError::DependenciesNotLoaded.into());
        }

        let schema = loaded_schema(&self.schema)?;
        let version = self.dependencies.db_version(table_name)
            .ok_or_else(|| anyhow!("Table version not found in dependencies for table {}.", table_name))?;

        schema.definition_by_name_and_version(table_name, version)
            .cloned()
            .ok_or_else(|| anyhow!("No definition found for table {}.", table_name))
    }

    /// Returns the tables with the provided name from the game files and the parent packs.
    pub fn dependency_tables(&self, table_name: &str) -> Result<Vec<RFile>> {
        Ok(self.dependencies.db_data(table_name, true, true)?.into_iter().cloned().collect())
    }
}

impl SessionState {

    /// Returns the definition and row count of a table.
    pub fn table_info(&mut self, file: &FileRef) -> Result<TableInfo> {
        let table = self.table(file)?;
        Ok(TableInfo {
            table_name: table.name().to_owned(),
            version: *table.definition().version(),
            columns: columns_info(table.definition(), table.patches()),
            row_count: table.len(),
        })
    }

    /// Returns a page of the rows of a table matching the request's filters, with only the requested columns.
    ///
    /// # Errors
    ///
    /// Fails if the table can't be read, or if the request names a column the table doesn't have.
    pub fn table_rows(&mut self, request: &GetTableRows) -> Result<TableRows> {
        let table = self.table(&request.file)?;
        let fields = table.definition().fields_processed();
        let column_index = |name: &str| fields.iter()
            .position(|field| field.name() == name)
            .ok_or_else(|| ApiError::InvalidParams(format!("The table has no column named {name}.")));

        let columns = match request.columns {
            Some(ref names) => names.iter()
                .map(|name| column_index(name).map(|index| (index, name.to_owned())))
                .collect::<Result<Vec<_>, _>>()?,
            None => fields.iter().enumerate().map(|(index, field)| (index, field.name().to_owned())).collect(),
        };

        let filters = request.filters.iter()
            .map(|filter| column_index(&filter.column).map(|column_index| PreparedFilter::new(filter, column_index)))
            .collect::<Result<Vec<_>, _>>()?;

        let limit = request.limit.unwrap_or(DEFAULT_ROWS_LIMIT);
        let mut rows = vec![];
        let mut total = 0;

        for (index, row) in table.data().iter().enumerate() {
            if !filters.iter().all(|filter| filter.matches(row)) {
                continue;
            }

            if total >= request.offset && rows.len() < limit {
                let values = columns.iter()
                    .map(|(column_index, _)| row.get(*column_index).map(decoded_to_json).unwrap_or(Value::Null))
                    .collect();

                rows.push(TableRow { index, values });
            }

            total += 1;
        }

        Ok(TableRows {
            columns: columns.into_iter().map(|(_, name)| name).collect(),
            rows,
            total,
        })
    }

    /// Returns a DB or Loc table from any source, decoding it first if needed.
    fn table(&mut self, file: &FileRef) -> Result<&TableInMemory> {
        let not_found = || ApiError::FileNotFound(file.path.clone());
        let rfile = match file.source {
            FileSource::Pack(ref pack_key) => pack_mut(&mut self.packs, pack_key)?.files_mut().get_mut(&file.path),
            FileSource::GameFiles => self.dependencies.file_mut(&file.path, true, false).ok(),
            FileSource::ParentFiles => self.dependencies.file_mut(&file.path, false, true).ok(),
            FileSource::AssemblyKit => {
                let table_name = file.path.split('/').nth(1).ok_or_else(not_found)?;
                let table = self.dependencies.asskit_only_db_tables().get(table_name).ok_or_else(not_found)?;
                return Ok(table.table());
            }
        }.ok_or_else(not_found)?;

        let file_type = rfile.file_type();
        if file_type != FileType::DB && file_type != FileType::Loc {
            return Err(ApiError::NotATable(file.path.clone()).into());
        }

        if file_type == FileType::DB && self.schema.is_none() {
            return Err(ApiError::SchemaNotLoaded.into());
        }

        let mut extra_data = DecodeableExtraData::default();
        extra_data.set_schema(self.schema.as_ref());
        rfile.decode(&Some(extra_data), true, false)?;

        match rfile.decoded()? {
            RFileDecoded::DB(table) => Ok(table.table()),
            RFileDecoded::Loc(table) => Ok(table.table()),
            _ => Err(ApiError::NotATable(file.path.clone()).into()),
        }
    }
}

impl PreparedFilter {

    /// Prepares a filter to check rows, with the index of its column.
    fn new(filter: &RowFilter, column_index: usize) -> Self {
        Self {
            column_index,
            op: filter.op,
            value: if filter.ignore_case { filter.value.to_lowercase() } else { filter.value.clone() },
            ignore_case: filter.ignore_case,
        }
    }

    /// Returns if the value of the filter's column in a row matches the filter.
    fn matches(&self, row: &[DecodedData]) -> bool {
        let Some(data) = row.get(self.column_index) else { return false };
        let value = data.data_to_string();
        let value = if self.ignore_case { Cow::Owned(value.to_lowercase()) } else { value };

        match self.op {
            FilterOp::Equals => *value == self.value,
            FilterOp::NotEquals => *value != self.value,
            FilterOp::Contains => value.contains(&self.value),
            FilterOp::StartsWith => value.starts_with(&self.value),
            FilterOp::EndsWith => value.ends_with(&self.value),
        }
    }
}

/// Returns the columns of a definition as rows see them, with the patches applied.
pub(super) fn columns_info(definition: &Definition, patches: &DefinitionPatch) -> Vec<ColumnInfo> {
    definition.fields_processed().iter()
        .map(|field| ColumnInfo {
            name: field.name().to_owned(),
            field_type: field.field_type().clone(),
            is_key: field.is_key(Some(patches)),
            reference: field.is_reference(Some(patches)).map(|(table, column)| ColumnReference { table, column }),
            default_value: field.default_value(Some(patches)),
            description: field.description(Some(patches)),
        })
        .collect()
}

/// Returns a table value as JSON: booleans and numbers as themselves, everything else as text.
///
/// Floats not representable in JSON (NaN, infinite) become `null`.
fn decoded_to_json(data: &DecodedData) -> Value {
    match data {
        DecodedData::Boolean(value) => Value::Bool(*value),

        // Parsing the f32's shortest text form keeps values like 0.1 exact, instead of 0.10000000149011612.
        DecodedData::F32(value) => value.to_string().parse::<f64>().ok().and_then(Number::from_f64).map_or(Value::Null, Value::Number),
        DecodedData::F64(value) => Number::from_f64(*value).map_or(Value::Null, Value::Number),
        DecodedData::I16(value) | DecodedData::OptionalI16(value) => Value::from(*value),
        DecodedData::I32(value) | DecodedData::OptionalI32(value) => Value::from(*value),
        DecodedData::I64(value) | DecodedData::OptionalI64(value) => Value::from(*value),
        DecodedData::ColourRGB(value) |
        DecodedData::StringU8(value) |
        DecodedData::StringU16(value) |
        DecodedData::OptionalStringU8(value) |
        DecodedData::OptionalStringU16(value) => Value::String(value.clone()),
        DecodedData::SequenceU16(_) |
        DecodedData::SequenceU32(_) => Value::String(data.data_to_string().into_owned()),
    }
}

/// Finds the first row with a value in a column of a table.
///
/// If the column is localised, it's not in the table's data, so the first key column is searched instead.
///
/// # Returns
///
/// The column and row indexes of the row.
fn find_in_db(table: &DB, column_name: &str, value: &str) -> Option<(usize, usize)> {
    let definition = table.definition();
    let key_column_name = if definition.localised_fields().iter().any(|field| field.name() == column_name) {
        definition.localised_key_order().first()
            .and_then(|index| definition.fields_processed().get(*index as usize).map(|field| field.name().to_owned()))
    } else {
        None
    };

    let column_name = key_column_name.as_deref().unwrap_or(column_name);
    let (column_index, row_indexes) = table.table().rows_containing_data(column_name, value)?;
    row_indexes.first().map(|row_index| (column_index, *row_index))
}

/// Finds the first row with a value in a column, among DB files.
fn find_in_db_files(files: &[&RFile], column_name: &str, value: &str, data_source: DataSource) -> Option<RowLocation> {
    files.iter().find_map(|file| match file.decoded() {
        Ok(RFileDecoded::DB(table)) => find_in_db(table, column_name, value)
            .map(|(column_index, row_index)| (data_source, file.path_in_container_raw().to_owned(), column_index, row_index)),
        _ => None,
    })
}

/// Finds the first row with a key, among Loc files.
fn find_in_loc_files(files: &[&RFile], loc_key: &str, data_source: DataSource) -> Option<RowLocation> {
    files.iter().find_map(|file| match file.decoded() {
        Ok(RFileDecoded::Loc(table)) => {
            let (column_index, row_indexes) = table.table().rows_containing_data("key", loc_key)?;
            row_indexes.first().map(|row_index| (data_source, file.path_in_container_raw().to_owned(), column_index, *row_index))
        }
        _ => None,
    })
}

/// Returns every row of a DB file with a value in any of the provided columns.
fn references_in_file(file: &RFile, columns: &[String], value: &str, data_source: DataSource, pack_key: &str) -> Vec<ReferenceLocation> {
    let Ok(RFileDecoded::DB(table)) = file.decoded() else { return vec![] };

    columns.iter()
        .filter_map(|column_name| table.table().rows_containing_data(column_name, value).map(|found| (column_name, found)))
        .flat_map(|(column_name, (column_index, row_indexes))| row_indexes.into_iter()
            .map(|row_index| (data_source, pack_key.to_owned(), file.path_in_container_raw().to_owned(), column_name.to_owned(), column_index, row_index))
            .collect::<Vec<_>>())
        .collect()
}

/// Delta-merges the decoded DB or Loc `sources` (same type/table) against the vanilla/parent baseline,
/// applying any already-known `resolutions`. See [`rpfm_extensions::merge`] for the merge rules.
fn delta_merge_files(sources: &[&RFile], merged_path: &str, dependencies: &Dependencies, resolutions: &[MergeResolution]) -> Result<DeltaMergeOutcome> {
    if sources.len() < 2 {
        return Err(anyhow!("Not enough tables provided to merge."));
    }

    let (merged, conflicts) = match sources[0].decoded()? {
        RFileDecoded::DB(_) => {
            let tables = sources.iter()
                .filter_map(|file| if let Ok(RFileDecoded::DB(table)) = file.decoded() { Some((file.path_in_container_raw(), table)) } else { None })
                .collect::<Vec<_>>();

            let baseline = db_baseline(dependencies, tables[0].1.table_name());
            let (merged, conflicts) = delta_merge_db(&tables, baseline.as_ref(), resolutions)?;
            (RFileDecoded::DB(merged), conflicts)
        },
        RFileDecoded::Loc(_) => {
            let tables = sources.iter()
                .filter_map(|file| if let Ok(RFileDecoded::Loc(table)) = file.decoded() { Some((file.path_in_container_raw(), table)) } else { None })
                .collect::<Vec<_>>();

            let baseline = loc_baseline(dependencies);
            let (merged, conflicts) = delta_merge_loc(&tables, baseline.as_ref(), resolutions)?;
            (RFileDecoded::Loc(merged), conflicts)
        },
        _ => return Err(anyhow!("Delta merge is only supported for DB and Loc tables.")),
    };

    if conflicts.is_empty() {
        Ok(DeltaMergeOutcome::Merged(RFile::new_from_decoded(&merged, current_time()?, merged_path)))
    } else {
        Ok(DeltaMergeOutcome::Conflicts(conflicts))
    }
}

//-------------------------------------------------------------------------------//
//                                   Tests
//-------------------------------------------------------------------------------//

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    fn filter(op: FilterOp, value: &str, ignore_case: bool) -> PreparedFilter {
        PreparedFilter::new(&RowFilter { column: "key".to_owned(), op, value: value.to_owned(), ignore_case }, 1)
    }

    #[test]
    fn values_become_plain_json() {
        assert_eq!(decoded_to_json(&DecodedData::Boolean(true)), json!(true));
        assert_eq!(decoded_to_json(&DecodedData::F32(0.1)), json!(0.1));
        assert_eq!(decoded_to_json(&DecodedData::F32(f32::NAN)), Value::Null);
        assert_eq!(decoded_to_json(&DecodedData::F64(2.5)), json!(2.5));
        assert_eq!(decoded_to_json(&DecodedData::I16(-3)), json!(-3));
        assert_eq!(decoded_to_json(&DecodedData::OptionalI32(7)), json!(7));
        assert_eq!(decoded_to_json(&DecodedData::I64(1 << 40)), json!(1_i64 << 40));
        assert_eq!(decoded_to_json(&DecodedData::StringU8("wh_main_emp".to_owned())), json!("wh_main_emp"));
        assert_eq!(decoded_to_json(&DecodedData::ColourRGB("FF0000".to_owned())), json!("FF0000"));
    }

    #[test]
    fn filters_compare_the_column_value_as_text() {
        let row = vec![DecodedData::I32(5), DecodedData::StringU8("wh_main_Empire".to_owned())];

        assert!(filter(FilterOp::Equals, "wh_main_Empire", false).matches(&row));
        assert!(!filter(FilterOp::Equals, "wh_main_empire", false).matches(&row));
        assert!(filter(FilterOp::Equals, "WH_MAIN_EMPIRE", true).matches(&row));
        assert!(filter(FilterOp::NotEquals, "other", false).matches(&row));
        assert!(filter(FilterOp::Contains, "main", false).matches(&row));
        assert!(filter(FilterOp::StartsWith, "wh_", false).matches(&row));
        assert!(filter(FilterOp::EndsWith, "empire", true).matches(&row));
        assert!(!filter(FilterOp::EndsWith, "empire", false).matches(&row));
    }

    #[test]
    fn filters_on_missing_columns_never_match() {
        let row = vec![DecodedData::I32(5)];

        assert!(!filter(FilterOp::NotEquals, "anything", false).matches(&row));
    }
}
