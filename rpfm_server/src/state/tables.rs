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

use std::collections::BTreeMap;

use std::borrow::Cow;
use std::collections::{HashMap, HashSet};
use std::path::Path;

use rpfm_extensions::dependencies::{Dependencies, KEY_DELETES_TABLE_NAME, TableReferences};
use rpfm_extensions::merge::{db_baseline, delta_merge_db, delta_merge_loc, loc_baseline, MergeConflict, MergeOptions, MergeResolution};

use rpfm_ipc::api::ApiError;
use rpfm_ipc::api::files::{ASSEMBLY_KIT_TABLE_FILE_NAME, FileRef, FileSource};
use rpfm_ipc::api::session::{DependencyTableData, DependencyTables};
use rpfm_ipc::api::references::{DEFAULT_REFERENCE_VALUES_LIMIT, DEFAULT_USAGES_LIMIT, FindUsages, GetReferenceValues, GetTableReferenceData, TableReferenceData, LocSource, ReferenceValue, ReferenceValues, RowLocation, Usage, Usages};
use rpfm_ipc::api::tables::{AddKeyDeletes, ExportTsv, FilesEdited, ImportTsv, MergeTables, RenameKey, TableUpgraded, TablesMerged, UpgradeTable};
use rpfm_ipc::api::tables::{ColumnInfo, ColumnValues, DEFAULT_VALUES_LIMIT, GetColumnValues, GetTableDefinition, ColumnReference, DEFAULT_ROWS_LIMIT, EditTable, FilterOp, GetTableRows, RowEdit, RowFilter, TableEdited, TableInfo, TableRow, TableRows};
use rpfm_ipc::helpers::{DataSource, RFileInfo};

use rpfm_lib::files::{Container, ContainerPath, db::DB, DecodeableExtraData, FileType, RFile, RFileDecoded, table::{DecodedData, local::TableInMemory, Table}};
use rpfm_lib::schema::{Definition, DefinitionPatch, Field, FieldType};
use rpfm_lib::utils::current_time;

use super::{SessionState, loaded_schema, pack, pack_mut};

/// A [`RowEdit`] with its values converted to the types of their columns, and its indexes checked.
enum PreparedEdit {
    Insert(usize, Vec<DecodedData>),
    Update(usize, Vec<(usize, DecodedData)>),

    /// Indexes to delete, sorted from last to first so removing one doesn't move the rest.
    Delete(Vec<usize>),
}

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

    /// Returns the values each column of a table can reference, with their lookups.
    ///
    /// # Errors
    ///
    /// Fails if there is no schema, or it has no definition of the table with the requested version.
    pub fn table_reference_data(&mut self, request: &GetTableReferenceData) -> Result<TableReferenceData> {
        let definition = loaded_schema(&self.schema)?.definition_by_name_and_version(&request.table_name, request.version)
            .cloned()
            .ok_or_else(|| ApiError::DefinitionNotFound(request.table_name.clone()))?;

        Ok(TableReferenceData { columns: self.reference_data(&request.table_name, &definition, request.regenerate) })
    }

    /// Finds the first row with a value in a column of a table.
    ///
    /// Searches the open packs (starting with `pack_key`), then the parent packs, the game files and the Assembly Kit tables.
    ///
    /// # Arguments
    ///
    /// * `pack_key` - Key of the pack to search first, if any.
    /// * `table` - Name of the table, with or without the `_tables` suffix.
    /// * `column` - Name of the column. If it's localised, the first key column is searched instead.
    /// * `value` - Value to search.
    ///
    /// # Returns
    ///
    /// Where the row is.
    pub fn find_definition(&self, pack_key: Option<&str>, table: &str, column: &str, value: &str) -> Result<RowLocation> {
        let table_name = format!("{}_tables", table.strip_suffix("_tables").unwrap_or(table));
        let table_folders = ContainerPath::db_table_folders(&table_name);

        // Search first in the pack that sent the request (if still open), then in the rest of the open packs.
        let first = pack_key.and_then(|pack_key| self.packs.get_key_value(pack_key));
        let packs_to_search = first.into_iter()
            .chain(self.packs.iter().filter(|(key, _)| Some(key.as_str()) != pack_key));

        for (key, pack) in packs_to_search {
            if let Some(location) = find_in_db_files(&pack.files_by_paths(&table_folders, true), column, value, &FileSource::Pack(key.clone())) {
                return Ok(location);
            }
        }

        for (source, include_vanilla, include_parent) in [(FileSource::ParentFiles, false, true), (FileSource::GameFiles, true, false)] {
            if let Ok(files) = self.dependencies.db_data(&table_name, include_vanilla, include_parent) {
                if let Some(location) = find_in_db_files(&files, column, value, &source) {
                    return Ok(location);
                }
            }
        }

        if let Some(table) = self.dependencies.asskit_only_db_tables().get(&table_name) {
            if let Some((column_index, row_index)) = find_in_db(table, column, value) {
                let path = format!("db/{table_name}/{ASSEMBLY_KIT_TABLE_FILE_NAME}");
                return Ok(RowLocation { source: FileSource::AssemblyKit, path, column_index, row_index });
            }
        }

        Err(ApiError::NotFound(format!("The value {value} of the column {column} of {table_name}")).into())
    }

    /// Finds the first row of a Loc file with a key.
    ///
    /// Searches the open packs (starting with `pack_key`), then the parent packs and then the game files.
    ///
    /// # Returns
    ///
    /// Where the row is.
    pub fn find_loc(&self, pack_key: Option<&str>, loc_key: &str) -> Result<RowLocation> {
        let first = pack_key.and_then(|pack_key| self.packs.get_key_value(pack_key));
        let packs_to_search = first.into_iter()
            .chain(self.packs.iter().filter(|(key, _)| Some(key.as_str()) != pack_key));

        for (key, pack) in packs_to_search {
            if let Some(location) = find_in_loc_files(&pack.files_by_type(&[FileType::Loc]), loc_key, &FileSource::Pack(key.clone())) {
                return Ok(location);
            }
        }

        for (source, include_vanilla, include_parent) in [(FileSource::ParentFiles, false, true), (FileSource::GameFiles, true, false)] {
            if let Ok(files) = self.dependencies.loc_data(include_vanilla, include_parent) {
                if let Some(location) = find_in_loc_files(&files, loc_key, &source) {
                    return Ok(location);
                }
            }
        }

        Err(ApiError::NotFound(format!("The loc key {loc_key}")).into())
    }

    /// Finds every row referencing a value, in the open packs, the parent packs and the game files.
    ///
    /// # Arguments
    ///
    /// * `pack_key` - If set, only this open pack is searched.
    /// * `reference_map` - Columns to search, by table name.
    /// * `value` - Value to search.
    ///
    /// # Returns
    ///
    /// The rows referencing the value.
    pub fn search_references(&self, pack_key: Option<&str>, reference_map: &HashMap<String, Vec<String>>, value: &str) -> Result<Vec<Usage>> {
        let paths = reference_map.keys().flat_map(|table_name| ContainerPath::db_table_folders(table_name)).collect::<Vec<ContainerPath>>();
        let packs = match pack_key {
            Some(pack_key) => vec![(pack_key, pack(&self.packs, pack_key)?)],
            None => self.packs.iter().map(|(key, pack)| (key.as_str(), pack)).collect(),
        };

        let mut references = vec![];
        for (key, pack) in packs {
            let files = pack.files_by_paths(&paths, true);
            let source = FileSource::Pack(key.to_owned());
            for (table_name, columns) in reference_map {
                for file in &files {
                    if file.db_table_name_from_path() == Some(table_name.as_str()) {
                        references.extend(references_in_file(file, columns, value, &source));
                    }
                }
            }
        }

        for (source, include_vanilla, include_parent) in [(FileSource::ParentFiles, false, true), (FileSource::GameFiles, true, false)] {
            for (table_name, columns) in reference_map {
                if let Ok(tables) = self.dependencies.db_data(table_name, include_vanilla, include_parent) {
                    references.par_extend(tables.par_iter().flat_map(|table| references_in_file(table, columns, value, &source)));
                }
            }
        }

        Ok(references)
    }

    /// Finds the rows of other tables referencing a value of a table, taking the referencing columns from the schema.
    pub fn find_usages(&self, request: &FindUsages) -> Result<Usages> {
        let schema = loaded_schema(&self.schema)?;
        let version = match request.version {
            Some(version) => version,
            None => self.table_definition(&GetTableDefinition { table_name: request.table_name.clone(), version: None })?.version,
        };

        let definition = schema.definition_by_name_and_version(&request.table_name, version)
            .ok_or_else(|| ApiError::DefinitionNotFound(request.table_name.clone()))?;

        let reference_map = schema.referencing_columns_for_table(&request.table_name, definition)
            .remove(&request.column)
            .unwrap_or_default();

        let usages = self.search_references(request.pack.as_deref(), &reference_map, &request.value)?;
        let total = usages.len();
        let usages = usages.into_iter()
            .skip(request.offset)
            .take(request.limit.unwrap_or(DEFAULT_USAGES_LIMIT))
            .collect();

        Ok(Usages { usages, total })
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

    /// Returns a page of the distinct values of a column of a table, sorted.
    ///
    /// # Errors
    ///
    /// Fails if the schema has no definition of the table, or if the table has no column with the provided name.
    pub fn column_values_page(&self, request: &GetColumnValues) -> Result<ColumnValues> {
        let definition = self.table_definition(&GetTableDefinition { table_name: request.table_name.clone(), version: None })?;
        if !definition.columns.iter().any(|column| column.name == request.column) {
            return Err(ApiError::InvalidParams(format!("The table has no column named {}.", request.column)).into());
        }

        let mut values = self.column_values(&request.table_name, &request.column, request.include_packs, request.include_dependencies)
            .into_iter()
            .filter(|value| value.starts_with(&request.prefix))
            .collect::<Vec<_>>();
        values.sort_unstable();

        let total = values.len();
        let values = values.into_iter()
            .skip(request.offset)
            .take(request.limit.unwrap_or(DEFAULT_VALUES_LIMIT))
            .collect();

        Ok(ColumnValues { values, total })
    }

    /// Returns a page of the values a reference column of a table can have, with their display text.
    ///
    /// # Errors
    ///
    /// Fails if the table has no column with the provided name, or if the column is not a reference.
    pub fn reference_values(&mut self, request: &GetReferenceValues) -> Result<ReferenceValues> {
        let version = self.table_definition(&GetTableDefinition { table_name: request.table_name.clone(), version: None })?.version;
        let definition = loaded_schema(&self.schema)?.definition_by_name_and_version(&request.table_name, version)
            .ok_or_else(|| ApiError::DefinitionNotFound(request.table_name.clone()))?
            .clone();

        let column_index = definition.fields_processed().iter()
            .position(|field| field.name() == request.column)
            .ok_or_else(|| ApiError::InvalidParams(format!("The table has no column named {}.", request.column)))?;

        let references = self.reference_data(&request.table_name, &definition, false).remove(&(column_index as i32))
            .ok_or_else(|| ApiError::InvalidParams(format!("The column {} is not a reference column.", request.column)))?;

        let mut values = references.data().iter()
            .filter(|(value, _)| value.starts_with(&request.prefix))
            .map(|(value, lookup)| ReferenceValue { value: value.clone(), lookup: lookup.clone() })
            .collect::<Vec<_>>();
        values.sort_unstable_by(|a, b| a.value.cmp(&b.value));

        let total = values.len();
        let values = values.into_iter()
            .skip(request.offset)
            .take(request.limit.unwrap_or(DEFAULT_REFERENCE_VALUES_LIMIT))
            .collect();

        Ok(ReferenceValues { values, total })
    }

    /// Returns the table, column and key values a loc key was generated from, if it can be found in the dependencies.
    pub fn loc_source(&self, loc_key: &str) -> Option<LocSource> {
        self.loc_key_source(loc_key).map(|(table, column, key_values)| LocSource { table, column, key_values })
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

    /// Returns the tables with the provided name from the game files and the parent packs.
    pub fn dependency_tables(&self, table_name: &str) -> Result<Vec<RFile>> {
        Ok(self.dependencies.db_data(table_name, true, true)?.into_iter().cloned().collect())
    }

    /// Returns the tables of the game files, and the custom tables of the schema, with their version.
    pub fn dependency_table_versions(&self) -> DependencyTables {
        let names = self.dependency_table_names().into_iter().chain(self.custom_table_names().unwrap_or_default());
        let tables = names.filter_map(|name| self.dependency_table_version(&name).ok().map(|version| (name, version))).collect();
        DependencyTables { tables }
    }

    /// Returns every decoded table of a type in the game files and the parent packs.
    pub fn dependency_table_data(&self, table_name: &str) -> Result<DependencyTableData> {
        let tables = self.dependency_tables(table_name)?.iter()
            .filter_map(|file| match file.decoded() {
                Ok(RFileDecoded::DB(table)) => Some(table.clone()),
                _ => None,
            })
            .collect();

        Ok(DependencyTableData { tables })
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

    /// Edits rows of a table of an open pack. If any edit fails, none is applied.
    ///
    /// # Errors
    ///
    /// Fails if the pack doesn't have the table, or if an edit names a column the table doesn't have,
    /// a row that doesn't exist, or a value that doesn't fit its column.
    pub fn edit_table(&mut self, request: &EditTable) -> Result<TableEdited> {
        let file = FileRef { source: FileSource::Pack(request.pack.clone()), path: request.path.clone() };
        let row_count = match self.decoded_table_file(&file)?.decoded_mut()? {
            RFileDecoded::DB(table) => {
                let edits = prepare_row_edits(&request.edits, &table.definition().fields_processed(), table.new_row(), table.data().len())?;
                apply_row_edits(table.data_mut(), edits)
            }
            RFileDecoded::Loc(table) => {
                let edits = prepare_row_edits(&request.edits, &table.definition().fields_processed(), table.new_row(), table.data().len())?;
                apply_row_edits(table.data_mut(), edits)
            }
            _ => return Err(ApiError::NotATable(request.path.clone()).into()),
        };

        Ok(TableEdited { row_count })
    }

    /// Merges tables of the same type of a pack into a new one.
    ///
    /// # Returns
    ///
    /// The path of the merged table, or the conflicts that prevented the merge.
    pub fn merge_tables(&mut self, request: &MergeTables) -> Result<TablesMerged> {
        let paths = self.pack_container_paths(&request.pack, &request.paths)?;
        let mut options = MergeOptions::default();
        options.set_delta_merge(request.delta);
        options.set_resolutions(request.resolutions.clone());

        Ok(match self.merge_files(&request.pack, &paths, &request.merged_path, request.delete_sources, &options)? {
            MergeOutcome::Merged(path) => TablesMerged { merged: Some(path), conflicts: vec![] },
            MergeOutcome::Conflicts(conflicts) => TablesMerged { merged: None, conflicts },
        })
    }

    /// Updates a table of a pack to the version it has in the game files.
    pub fn upgrade_table(&mut self, request: &UpgradeTable) -> Result<TableUpgraded> {
        let (old_version, new_version, deleted_columns, added_columns) = self.update_table(&request.pack, &ContainerPath::File(request.path.clone()))?;
        Ok(TableUpgraded { old_version, new_version, deleted_columns, added_columns })
    }

    /// Changes a value of a key column in every table of a pack, including the columns referencing it.
    ///
    /// # Errors
    ///
    /// Fails if the table has no definition, or no column with the provided name.
    pub fn rename_key(&mut self, request: &RenameKey) -> Result<FilesEdited> {
        let version = match request.version {
            Some(version) => version,
            None => self.table_definition(&GetTableDefinition { table_name: request.table_name.clone(), version: None })?.version,
        };

        let definition = loaded_schema(&self.schema)?.definition_by_name_and_version(&request.table_name, version)
            .ok_or_else(|| ApiError::DefinitionNotFound(request.table_name.clone()))?
            .clone();

        let field = definition.fields_processed().into_iter()
            .find(|field| field.name() == request.column)
            .ok_or_else(|| ApiError::InvalidParams(format!("The table has no column named {}.", request.column)))?;

        let changes = [(field, request.old_value.clone(), request.new_value.clone())];
        let (edited, _) = self.cascade_edition(&request.pack, &request.table_name, &definition, &changes)?;
        Ok(FilesEdited { edited: edited.iter().map(|path| path.path_raw().to_owned()).collect() })
    }

    /// Adds rows to a key deletes table of a pack, one per key.
    ///
    /// # Errors
    ///
    /// Fails if the key deletes table doesn't exist in the pack.
    pub fn add_key_deletes(&mut self, request: &AddKeyDeletes) -> Result<FilesEdited> {
        let keys = request.keys.iter().cloned().collect::<HashSet<_>>();
        let path = self.add_keys_to_key_deletes(&request.pack, &request.file_name, &request.table_name, &keys)?
            .ok_or_else(|| ApiError::FileNotFound(format!("db/{KEY_DELETES_TABLE_NAME}/{}", request.file_name)))?;

        Ok(FilesEdited { edited: vec![path.path_raw().to_owned()] })
    }

    /// Writes a table from any source, except the Assembly Kit, to a TSV file.
    ///
    /// # Arguments
    ///
    /// * `request` - Table to export, and where.
    /// * `keys_first` - If the TSV uses the old column order, with keys first.
    pub fn export_table_tsv(&mut self, request: &ExportTsv, keys_first: bool) -> Result<()> {
        let (pack_key, data_source) = match request.file.source {
            FileSource::Pack(ref pack_key) => (pack_key.as_str(), DataSource::PackFile),
            FileSource::GameFiles => ("", DataSource::GameFiles),
            FileSource::ParentFiles => ("", DataSource::ParentFiles),
            FileSource::AssemblyKit => return Err(ApiError::InvalidParams("Assembly Kit tables can't be exported to TSV.".to_owned()).into()),
        };

        self.export_tsv(pack_key, &request.file.path, &request.destination, data_source, keys_first)
    }

    /// Replaces a table of a pack with the contents of a TSV file, keeping its GUID.
    pub fn import_table_tsv(&mut self, request: &ImportTsv) -> Result<TableEdited> {
        let row_count = match self.import_tsv(&request.pack, &request.path, &request.tsv_path)? {
            RFileDecoded::DB(table) => table.len(),
            RFileDecoded::Loc(table) => table.len(),
            _ => return Err(ApiError::NotATable(request.path.clone()).into()),
        };

        Ok(TableEdited { row_count })
    }

    /// Returns a DB or Loc table from any source, decoding it first if needed.
    fn table(&mut self, file: &FileRef) -> Result<&TableInMemory> {
        if file.source == FileSource::AssemblyKit {
            let not_found = || ApiError::FileNotFound(file.path.clone());
            let table_name = file.path.split('/').nth(1).ok_or_else(not_found)?;
            let table = self.dependencies.asskit_only_db_tables().get(table_name).ok_or_else(not_found)?;
            return Ok(table.table());
        }

        match self.decoded_table_file(file)?.decoded()? {
            RFileDecoded::DB(table) => Ok(table.table()),
            RFileDecoded::Loc(table) => Ok(table.table()),
            _ => Err(ApiError::NotATable(file.path.clone()).into()),
        }
    }

    /// Returns a DB or Loc file from an open pack or the dependencies, decoded.
    fn decoded_table_file(&mut self, file: &FileRef) -> Result<&mut RFile> {
        let rfile = match file.source {
            FileSource::Pack(ref pack_key) => pack_mut(&mut self.packs, pack_key)?.files_mut().get_mut(&file.path),
            FileSource::GameFiles => self.dependencies.file_mut(&file.path, true, false).ok(),
            FileSource::ParentFiles => self.dependencies.file_mut(&file.path, false, true).ok(),

            // Assembly Kit tables are kept decoded, outside of any file.
            FileSource::AssemblyKit => return Err(ApiError::ReadOnly(file.path.clone()).into()),
        }.ok_or_else(|| ApiError::FileNotFound(file.path.clone()))?;

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
        Ok(rfile)
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

/// Converts row edits to the types of their columns, and checks their indexes against the rows the table will have when each one is applied.
///
/// # Arguments
///
/// * `edits` - The edits to prepare.
/// * `fields` - Columns of the table, as rows see them.
/// * `new_row` - A row with the default value of each column.
/// * `row_count` - Amount of rows of the table before the edits.
fn prepare_row_edits(edits: &[RowEdit], fields: &[Field], new_row: Vec<DecodedData>, mut row_count: usize) -> Result<Vec<PreparedEdit>, ApiError> {
    let mut prepared = Vec::with_capacity(edits.len());
    for (edit_index, edit) in edits.iter().enumerate() {
        let invalid = |message: String| ApiError::InvalidParams(format!("Edit {edit_index}: {message}"));
        let missing_row = |index: usize, row_count: usize| invalid(format!("row {index} doesn't exist, the table has {row_count} rows at this point."));

        match edit {
            RowEdit::Insert { index, values } => {
                let index = index.unwrap_or(row_count);
                if index > row_count {
                    return Err(missing_row(index, row_count));
                }

                let mut row = new_row.clone();
                for (column_index, value) in row_values(fields, values).map_err(invalid)? {
                    row[column_index] = value;
                }

                prepared.push(PreparedEdit::Insert(index, row));
                row_count += 1;
            }
            RowEdit::Update { index, values } => {
                if *index >= row_count {
                    return Err(missing_row(*index, row_count));
                }

                prepared.push(PreparedEdit::Update(*index, row_values(fields, values).map_err(invalid)?));
            }
            RowEdit::Delete { indexes } => {
                let mut indexes = indexes.clone();
                indexes.sort_unstable_by(|a, b| b.cmp(a));
                indexes.dedup();

                if let Some(index) = indexes.first().filter(|index| **index >= row_count) {
                    return Err(missing_row(*index, row_count));
                }

                row_count -= indexes.len();
                prepared.push(PreparedEdit::Delete(indexes));
            }
        }
    }

    Ok(prepared)
}

/// Applies prepared row edits to the rows of a table.
///
/// # Returns
///
/// The amount of rows after the edits.
fn apply_row_edits(rows: &mut Vec<Vec<DecodedData>>, edits: Vec<PreparedEdit>) -> usize {
    for edit in edits {
        match edit {
            PreparedEdit::Insert(index, row) => rows.insert(index, row),
            PreparedEdit::Update(index, values) => {
                for (column_index, value) in values {
                    rows[index][column_index] = value;
                }
            }
            PreparedEdit::Delete(indexes) => {
                for index in indexes {
                    rows.remove(index);
                }
            }
        }
    }

    rows.len()
}

/// Converts values given by column name to the types of their columns.
///
/// # Returns
///
/// The index of the column of each value, and the converted value.
fn row_values(fields: &[Field], values: &BTreeMap<String, Value>) -> Result<Vec<(usize, DecodedData)>, String> {
    values.iter()
        .map(|(column, value)| {
            let column_index = fields.iter()
                .position(|field| field.name() == column)
                .ok_or_else(|| format!("the table has no column named {column}."))?;

            let value = json_to_decoded(fields[column_index].field_type(), value)
                .map_err(|error| format!("invalid value for column {column}: {error}"))?;

            Ok((column_index, value))
        })
        .collect()
}

/// Converts a JSON value to a table value of the provided type.
///
/// Booleans, numbers and strings are accepted, as long as their text parses as the type.
fn json_to_decoded(field_type: &FieldType, value: &Value) -> Result<DecodedData> {
    let text = match value {
        Value::String(text) => Cow::Borrowed(text.as_str()),
        Value::Bool(_) | Value::Number(_) => Cow::Owned(value.to_string()),
        Value::Null | Value::Array(_) | Value::Object(_) => return Err(anyhow!("expected a boolean, number or string, found {value}.")),
    };

    DecodedData::new_from_type_and_string(field_type, &text)
        .map_err(|_| anyhow!("expected {}, found {value}.", expected_value(field_type)))
}

/// Returns how the values a column of the provided type accepts are described in errors.
fn expected_value(field_type: &FieldType) -> &'static str {
    match field_type {
        FieldType::Boolean => "a boolean",
        FieldType::F32 | FieldType::F64 => "a number",
        FieldType::I16 | FieldType::OptionalI16 => "a 16-bit integer",
        FieldType::I32 | FieldType::OptionalI32 => "a 32-bit integer",
        FieldType::I64 | FieldType::OptionalI64 => "a 64-bit integer",
        FieldType::ColourRGB => "a hexadecimal colour, like FF0000",
        FieldType::StringU8 |
        FieldType::StringU16 |
        FieldType::OptionalStringU8 |
        FieldType::OptionalStringU16 |
        FieldType::SequenceU16(_) |
        FieldType::SequenceU32(_) => "a string",
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
fn find_in_db_files(files: &[&RFile], column_name: &str, value: &str, source: &FileSource) -> Option<RowLocation> {
    files.iter().find_map(|file| match file.decoded() {
        Ok(RFileDecoded::DB(table)) => find_in_db(table, column_name, value)
            .map(|(column_index, row_index)| RowLocation { source: source.clone(), path: file.path_in_container_raw().to_owned(), column_index, row_index }),
        _ => None,
    })
}

/// Finds the first row with a key, among Loc files.
fn find_in_loc_files(files: &[&RFile], loc_key: &str, source: &FileSource) -> Option<RowLocation> {
    files.iter().find_map(|file| match file.decoded() {
        Ok(RFileDecoded::Loc(table)) => {
            let (column_index, row_indexes) = table.table().rows_containing_data("key", loc_key)?;
            row_indexes.first().map(|row_index| RowLocation { source: source.clone(), path: file.path_in_container_raw().to_owned(), column_index, row_index: *row_index })
        }
        _ => None,
    })
}

/// Returns every row of a DB file with a value in any of the provided columns.
fn references_in_file(file: &RFile, columns: &[String], value: &str, source: &FileSource) -> Vec<Usage> {
    let Ok(RFileDecoded::DB(table)) = file.decoded() else { return vec![] };

    columns.iter()
        .filter_map(|column_name| table.table().rows_containing_data(column_name, value).map(|found| (column_name, found)))
        .flat_map(|(column_name, (column_index, row_indexes))| row_indexes.into_iter()
            .map(|row_index| Usage {
                location: RowLocation { source: source.clone(), path: file.path_in_container_raw().to_owned(), column_index, row_index },
                column: column_name.to_owned(),
            })
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

    fn fields() -> Vec<Field> {
        let mut key = Field::default();
        key.set_name("key".to_owned());
        key.set_field_type(FieldType::StringU8);

        let mut value = Field::default();
        value.set_name("value".to_owned());
        value.set_field_type(FieldType::F32);

        vec![key, value]
    }

    fn row(key: &str, value: f32) -> Vec<DecodedData> {
        vec![DecodedData::StringU8(key.to_owned()), DecodedData::F32(value)]
    }

    fn values(pairs: &[(&str, Value)]) -> BTreeMap<String, Value> {
        pairs.iter().map(|(column, value)| (column.to_string(), value.clone())).collect()
    }

    fn edit(rows: &mut Vec<Vec<DecodedData>>, edits: &[RowEdit]) -> Result<usize, ApiError> {
        let prepared = prepare_row_edits(edits, &fields(), row("", 0.0), rows.len())?;
        Ok(apply_row_edits(rows, prepared))
    }

    #[test]
    fn edits_apply_in_order() {
        let mut rows = vec![row("a", 1.0), row("b", 2.0), row("c", 3.0)];
        let edits = vec![
            RowEdit::Insert { index: Some(0), values: values(&[("key", json!("new")), ("value", json!(0.5))]) },
            RowEdit::Update { index: 2, values: values(&[("value", json!("2.5"))]) },
            RowEdit::Delete { indexes: vec![3, 1, 3] },
            RowEdit::Insert { index: None, values: values(&[("key", json!("last"))]) },
        ];

        assert_eq!(edit(&mut rows, &edits), Ok(3));
        assert_eq!(rows, vec![row("new", 0.5), row("b", 2.5), row("last", 0.0)]);
    }

    #[test]
    fn failed_edits_change_nothing() {
        let original = vec![row("a", 1.0), row("b", 2.0)];

        let failing = [
            vec![RowEdit::Update { index: 0, values: values(&[("value", json!(9.0))]) }, RowEdit::Update { index: 2, values: values(&[("value", json!(1.0))]) }],
            vec![RowEdit::Delete { indexes: vec![0] }, RowEdit::Update { index: 1, values: values(&[("value", json!(1.0))]) }],
            vec![RowEdit::Insert { index: Some(3), values: BTreeMap::new() }],
            vec![RowEdit::Update { index: 0, values: values(&[("nope", json!(1))]) }],
            vec![RowEdit::Update { index: 0, values: values(&[("value", json!("not a number"))]) }],
            vec![RowEdit::Update { index: 0, values: values(&[("value", json!(null))]) }],
        ];

        for edits in failing {
            let mut rows = original.clone();
            assert!(matches!(edit(&mut rows, &edits), Err(ApiError::InvalidParams(_))), "{edits:?}");
            assert_eq!(rows, original);
        }
    }

    #[test]
    fn values_are_converted_to_the_column_type() {
        assert_eq!(json_to_decoded(&FieldType::Boolean, &json!(true)).unwrap(), DecodedData::Boolean(true));
        assert_eq!(json_to_decoded(&FieldType::I32, &json!(42)).unwrap(), DecodedData::I32(42));
        assert_eq!(json_to_decoded(&FieldType::I32, &json!("42")).unwrap(), DecodedData::I32(42));
        assert_eq!(json_to_decoded(&FieldType::F32, &json!(3)).unwrap(), DecodedData::F32(3.0));
        assert_eq!(json_to_decoded(&FieldType::StringU8, &json!("text")).unwrap(), DecodedData::StringU8("text".to_owned()));
        assert!(json_to_decoded(&FieldType::StringU8, &json!(["a"])).is_err());
    }

    #[test]
    fn invalid_values_report_the_type_the_column_expects() {
        let error = json_to_decoded(&FieldType::I32, &json!(4.5)).unwrap_err();

        assert_eq!(error.to_string(), "expected a 32-bit integer, found 4.5.");
    }

    #[test]
    fn filters_on_missing_columns_never_match() {
        let row = vec![DecodedData::I32(5)];

        assert!(!filter(FilterOp::NotEquals, "anything", false).matches(&row));
    }
}
