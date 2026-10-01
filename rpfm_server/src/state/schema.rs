//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Schema operations: definitions, local patches, and updating the schema from the Assembly Kit.

use anyhow::Result;
use itertools::Itertools;
use rayon::prelude::*;

use std::collections::HashMap;

use rpfm_ipc::api::ApiError;
use rpfm_ipc::api::schema::{DeleteDefinition, GetReferencingColumns, GetTablePatches, ImportPatches, PATCH_KEYS, PatchColumn, RawDefinitions, ReferencingColumns, RemovePatches, SchemaTables, SetDefinition, TablePatches};
use rpfm_ipc::api::tables::{GetTableDefinition, TableDefinition};

use rpfm_lib::files::{Container, db::DB, FileType, RFileDecoded};
use rpfm_lib::integrations::assembly_kit::update_schema_from_raw_files;
use rpfm_lib::schema::{Definition, DefinitionPatch, Schema};

use rpfm_ipc::settings::{schemas_path, table_patches_path, Settings};

use super::{SessionState, load_schema, loaded_schema, loaded_schema_mut};
use super::tables::columns_info;

impl SessionState {

    /// Updates the schema of the selected game with the table definitions of its Assembly Kit, and saves it.
    ///
    /// # Arguments
    ///
    /// * `settings` - Settings, to find the game's Assembly Kit.
    /// * `ignore_game_files_in_ak` - If tables in the game files are skipped.
    /// * `disable_uuid_regeneration` - If tables keep their GUID when re-encoded to reload the schema.
    pub fn update_schema_from_asskit(&mut self, settings: &Settings, ignore_game_files_in_ak: bool, disable_uuid_regeneration: bool) -> Result<()> {
        let schema = loaded_schema_mut(&mut self.schema)?;
        let asskit_path = settings.assembly_kit_path(&self.game)?;
        let schema_path = schemas_path()?.join(self.game.schema_file_name());

        // If there are packs open, also add the packs' tables to it. That way we can treat some special tables, like starpos tables.
        let mut tables_to_check = self.dependencies.db_and_loc_data(true, false, true, false)?;
        for pack in self.packs.values() {
            if !pack.disk_file_path().is_empty() {
                tables_to_check.append(&mut pack.files_by_type(&[FileType::DB]));
            }
        }

        // Split the tables by name, merging tables of the same name and version, so we got more chances of loc data being found.
        let mut tables_to_check_split: HashMap<String, Vec<DB>> = HashMap::new();
        for table_to_check in tables_to_check {
            if let Ok(RFileDecoded::DB(table)) = table_to_check.decoded() {
                let tables = tables_to_check_split.entry(table.table_name().to_owned()).or_default();
                match tables.iter_mut().find(|source| source.definition().version() == table.definition().version()) {
                    Some(db_source) => *db_source = DB::merge(&[db_source, table])?,
                    None => tables.push(table.clone()),
                }
            }
        }

        let tables_to_skip = if ignore_game_files_in_ak {
            self.dependencies.vanilla_loose_tables().keys().chain(self.dependencies.vanilla_tables().keys()).map(|name| &**name).collect::<Vec<_>>()
        } else {
            vec![]
        };

        let possible_loc_fields = update_schema_from_raw_files(schema, &self.game, &asskit_path, &schema_path, &tables_to_skip, &tables_to_check_split)?;

        // The update deletes all loc fields, so we need to get them again from the TExc_LocalisableFields.xml, if said file exists.
        // That's why it does the update again, to re-populate the loc fields list with the ones not bruteforced.
        let local_packs = if self.packs.is_empty() { None } else { Some(&self.packs) };
        if self.dependencies.bruteforce_loc_key_order(schema, possible_loc_fields, local_packs, None).is_err() {
            return Ok(());
        }

        let _ = update_schema_from_raw_files(schema, &self.game, &asskit_path, &schema_path, &tables_to_skip, &tables_to_check_split);

        // This generates the automatic patches in the schema (like ".png are files" kinda patches).
        if self.dependencies.generate_automatic_patches(schema, &self.packs).is_err() {
            return Ok(());
        }

        // Fix for old file relative paths using incorrect separators.
        schema.definitions_mut().par_iter_mut().for_each(|(_, definitions)| {
            definitions.iter_mut().for_each(|definition| {
                definition.fields_mut().iter_mut().for_each(|field| {
                    if let Some(path) = field.filename_relative_path(None) {
                        if path.len() == 1 && path[0].contains(',') {
                            let new_paths = path[0].split(',').map(|path| path.trim()).join(";");
                            field.set_filename_relative_path(Some(new_paths));
                        }
                    }
                });
            });
        });

        schema.save(&schema_path)?;

        // The update clears the definitions' patches, so reload the saved schema to apply them again.
        load_schema(&mut self.schema, &mut self.packs, &self.game, disable_uuid_regeneration);
        Ok(())
    }

    /// Reloads the schema of the selected game from disk, re-decoding the tables of the open packs.
    ///
    /// # Arguments
    ///
    /// * `disable_uuid_regeneration` - If tables keep their GUID when re-encoded.
    pub fn reload_schema(&mut self, disable_uuid_regeneration: bool) {
        load_schema(&mut self.schema, &mut self.packs, &self.game, disable_uuid_regeneration);
    }

    /// Saves a schema to disk as the schema of the selected game, and loads it.
    pub fn save_schema(&mut self, mut schema: Schema) -> Result<()> {
        schema.save(&schemas_path()?.join(self.game.schema_file_name()))?;
        self.schema = Some(schema);
        Ok(())
    }

    /// Saves local patches for the schema of the selected game.
    pub fn save_local_schema_patches(&self, patches: &HashMap<String, DefinitionPatch>) -> Result<()> {
        Schema::save_patches(patches, &table_patches_path()?.join(self.game.schema_file_name()))?;
        Ok(())
    }

    /// Removes the local schema patches of a table.
    pub fn remove_local_schema_patches_for_table(&self, table_name: &str) -> Result<()> {
        Schema::remove_patches_for_table(table_name, &table_patches_path()?.join(self.game.schema_file_name()))?;
        Ok(())
    }

    /// Removes the local schema patches of a field of a table.
    pub fn remove_local_schema_patches_for_table_and_field(&self, table_name: &str, field_name: &str) -> Result<()> {
        Schema::remove_patches_for_table_and_field(table_name, field_name, &table_patches_path()?.join(self.game.schema_file_name()))?;
        Ok(())
    }

    /// Adds patches to the schema of the selected game, and saves it.
    pub fn import_schema_patches(&mut self, patches: &HashMap<String, DefinitionPatch>) -> Result<()> {
        let schema = loaded_schema_mut(&mut self.schema)?;
        Schema::add_patches_to_patch_set(schema.patches_mut(), patches);
        schema.save(&schemas_path()?.join(self.game.schema_file_name()))?;
        Ok(())
    }

    /// Returns the names of the startpos and twad tables in the schema.
    pub fn custom_table_names(&self) -> Result<Vec<String>> {
        Ok(loaded_schema(&self.schema)?.definitions().par_iter()
            .filter(|(key, definitions)| !definitions.is_empty() && (key.starts_with("start_pos_") || key.starts_with("twad_")))
            .map(|(key, _)| key.to_owned())
            .collect())
    }

    /// Returns the columns of a table as defined in the schema.
    ///
    /// If the request has no version, the version of the table in the game files is used, or the newest one if there is none.
    pub fn table_definition(&self, request: &GetTableDefinition) -> Result<TableDefinition> {
        let schema = loaded_schema(&self.schema)?;
        let table_name = &request.table_name;
        let definition = match request.version {
            Some(version) => schema.definition_by_name_and_version(table_name, version),
            None => self.dependencies.db_version(table_name)
                .and_then(|version| schema.definition_by_name_and_version(table_name, version))
                .or_else(|| schema.definitions_by_table_name(table_name).and_then(|definitions| definitions.first())),
        }.ok_or_else(|| ApiError::DefinitionNotFound(table_name.to_owned()))?;

        Ok(TableDefinition {
            table_name: table_name.to_owned(),
            version: *definition.version(),
            columns: columns_info(definition, definition.patches()),
        })
    }

    /// Returns the tables of the schema whose name starts with a prefix, with the versions they have definitions for.
    pub fn schema_tables(&self, prefix: &str) -> Result<SchemaTables> {
        let tables = loaded_schema(&self.schema)?.definitions().iter()
            .filter(|(table_name, definitions)| table_name.starts_with(prefix) && !definitions.is_empty())
            .map(|(table_name, definitions)| (table_name.to_owned(), definitions.iter().map(|definition| *definition.version()).collect()))
            .collect();

        Ok(SchemaTables { tables })
    }

    /// Saves a local patch for a column of a table, and reloads the schema so it applies.
    ///
    /// # Arguments
    ///
    /// * `request` - The patch.
    /// * `disable_uuid_regeneration` - If tables keep their GUID when re-encoded to reload the schema.
    ///
    /// # Errors
    ///
    /// Fails if a key of the patch is not a valid one, or if the patch can't be saved.
    pub fn patch_column(&mut self, request: &PatchColumn, disable_uuid_regeneration: bool) -> Result<()> {
        if let Some(key) = request.patch.keys().find(|key| !PATCH_KEYS.iter().any(|(valid_key, _)| valid_key == key)) {
            let valid_keys = PATCH_KEYS.iter().map(|(key, _)| *key).collect::<Vec<_>>().join(", ");
            return Err(ApiError::InvalidParams(format!("Unknown patch key: {key}. Valid ones: {valid_keys}.")).into());
        }

        let column_patch = request.patch.iter().map(|(key, value)| (key.clone(), value.clone())).collect();
        let patches = HashMap::from([(request.table_name.clone(), HashMap::from([(request.column.clone(), column_patch)]))]);
        self.save_local_schema_patches(&patches)?;
        self.reload_schema(disable_uuid_regeneration);
        Ok(())
    }

    /// Removes the local patches of a table, or of one of its columns, and reloads the schema.
    ///
    /// # Arguments
    ///
    /// * `request` - Patches to remove.
    /// * `disable_uuid_regeneration` - If tables keep their GUID when re-encoded to reload the schema.
    pub fn remove_patches(&mut self, request: &RemovePatches, disable_uuid_regeneration: bool) -> Result<()> {
        match request.column {
            Some(ref column) => self.remove_local_schema_patches_for_table_and_field(&request.table_name, column)?,
            None => self.remove_local_schema_patches_for_table(&request.table_name)?,
        }

        self.reload_schema(disable_uuid_regeneration);
        Ok(())
    }

    /// Returns the definitions of a table as the schema stores them, newest first.
    pub fn raw_definitions(&self, table_name: &str, version: Option<i32>) -> Result<RawDefinitions> {
        let definitions = self.definitions_by_table_name(table_name)?.iter()
            .filter(|definition| version.is_none_or(|version| *definition.version() == version))
            .map(serde_json::to_value)
            .collect::<Result<Vec<_>, _>>()?;

        Ok(RawDefinitions { definitions })
    }

    /// Adds a definition to the schema, or replaces the one with its version, then saves and reloads the schema.
    ///
    /// # Arguments
    ///
    /// * `request` - The definition.
    /// * `disable_uuid_regeneration` - If tables keep their GUID when re-encoded to reload the schema.
    pub fn set_definition(&mut self, request: &SetDefinition, disable_uuid_regeneration: bool) -> Result<()> {
        let definition = serde_json::from_value::<Definition>(request.definition.clone())
            .map_err(|error| ApiError::InvalidParams(format!("Invalid definition: {error}")))?;

        let mut schema = loaded_schema(&self.schema)?.clone();
        schema.add_definition(&request.table_name, &definition);
        self.save_schema(schema)?;
        self.reload_schema(disable_uuid_regeneration);
        Ok(())
    }

    /// Removes a definition from the schema, then saves and reloads the schema.
    ///
    /// # Arguments
    ///
    /// * `request` - The definition to remove.
    /// * `disable_uuid_regeneration` - If tables keep their GUID when re-encoded to reload the schema.
    pub fn delete_definition_and_save(&mut self, request: &DeleteDefinition, disable_uuid_regeneration: bool) -> Result<()> {
        let mut schema = loaded_schema(&self.schema)?.clone();
        schema.remove_definition(&request.table_name, request.version);
        self.save_schema(schema)?;
        self.reload_schema(disable_uuid_regeneration);
        Ok(())
    }

    /// Returns the columns of other tables referencing each column of a table.
    pub fn referencing_columns_of(&self, request: &GetReferencingColumns) -> Result<ReferencingColumns> {
        let table_name = &request.table_name;
        let version = match request.version {
            Some(version) => version,
            None => self.table_definition(&GetTableDefinition { table_name: table_name.to_owned(), version: None })?.version,
        };

        let definition = loaded_schema(&self.schema)?.definition_by_name_and_version(table_name, version)
            .ok_or_else(|| ApiError::DefinitionNotFound(table_name.to_owned()))?;

        let columns = self.referencing_columns(table_name, definition)?.into_iter()
            .map(|(column, tables)| (column, tables.into_iter().collect()))
            .collect();

        Ok(ReferencingColumns { columns })
    }

    /// Adds patches to the schema itself, saves it, and reloads it.
    ///
    /// # Arguments
    ///
    /// * `request` - The patches.
    /// * `disable_uuid_regeneration` - If tables keep their GUID when re-encoded to reload the schema.
    pub fn import_patches(&mut self, request: &ImportPatches, disable_uuid_regeneration: bool) -> Result<()> {
        let patches = request.patches.iter()
            .map(|(table, columns)| (table.clone(), columns.iter()
                .map(|(column, patch)| (column.clone(), patch.iter().map(|(key, value)| (key.clone(), value.clone())).collect()))
                .collect()))
            .collect::<HashMap<String, DefinitionPatch>>();

        self.import_schema_patches(&patches)?;
        self.reload_schema(disable_uuid_regeneration);
        Ok(())
    }

    /// Returns all the definitions of a table. Empty if the table is not in the schema.
    pub fn definitions_by_table_name(&self, table_name: &str) -> Result<Vec<Definition>> {
        Ok(loaded_schema(&self.schema)?.definitions_by_table_name(table_name)
            .map(|definitions| definitions.to_vec())
            .unwrap_or_default())
    }

    /// Returns the patches of a definition of a table, or of the table if the definition doesn't exist.
    ///
    /// Definitions lose their patches when serialized, so clients need to request them separately.
    pub fn definition_patches(&self, table_name: &str, version: i32) -> Result<DefinitionPatch> {
        let schema = loaded_schema(&self.schema)?;
        Ok(match schema.definition_by_name_and_version(table_name, version) {
            Some(definition) => definition.patches().clone(),
            None => schema.patches().get(table_name).cloned().unwrap_or_default(),
        })
    }

    /// Returns the patches of a definition of a table, by column name.
    pub fn table_patches(&self, request: &GetTablePatches) -> Result<TablePatches> {
        let patches = self.definition_patches(&request.table_name, request.version)?.into_iter()
            .map(|(column, patch)| (column, patch.into_iter().collect()))
            .collect();

        Ok(TablePatches { patches })
    }

    /// Returns the columns of other tables referencing a table, by table name and column name.
    pub fn referencing_columns(&self, table_name: &str, definition: &Definition) -> Result<HashMap<String, HashMap<String, Vec<String>>>> {
        Ok(loaded_schema(&self.schema)?.referencing_columns_for_table(table_name, definition))
    }
}
