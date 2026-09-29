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

use anyhow::{anyhow, Result};
use itertools::Itertools;
use rayon::prelude::*;

use std::collections::HashMap;

use rpfm_lib::files::{Container, db::DB, FileType, RFileDecoded};
use rpfm_lib::integrations::assembly_kit::update_schema_from_raw_files;
use rpfm_lib::schema::{Definition, DefinitionPatch, Schema};

use crate::settings::{schemas_path, table_patches_path, Settings};

use super::{NO_SCHEMA_ERROR, SessionState, load_schema, loaded_schema};

impl SessionState {

    /// Updates the schema of the selected game with the table definitions of its Assembly Kit, and saves it.
    ///
    /// # Arguments
    ///
    /// * `settings` - Settings, to find the game's Assembly Kit.
    /// * `ignore_game_files_in_ak` - If tables in the game files are skipped.
    /// * `disable_uuid_regeneration` - If tables keep their GUID when re-encoded to reload the schema.
    pub fn update_schema_from_asskit(&mut self, settings: &Settings, ignore_game_files_in_ak: bool, disable_uuid_regeneration: bool) -> Result<()> {
        let schema = self.schema.as_mut().ok_or_else(|| anyhow!(NO_SCHEMA_ERROR))?;
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
        let schema = self.schema.as_mut().ok_or_else(|| anyhow!(NO_SCHEMA_ERROR))?;
        Schema::add_patches_to_patch_set(schema.patches_mut(), patches);
        schema.save(&schemas_path()?.join(self.game.schema_file_name()))?;
        Ok(())
    }

    /// Returns a copy of the loaded schema.
    pub fn schema(&self) -> Result<Schema> {
        Ok(loaded_schema(&self.schema)?.clone())
    }

    /// Returns the names of the startpos and twad tables in the schema.
    pub fn custom_table_names(&self) -> Result<Vec<String>> {
        Ok(loaded_schema(&self.schema)?.definitions().par_iter()
            .filter(|(key, definitions)| !definitions.is_empty() && (key.starts_with("start_pos_") || key.starts_with("twad_")))
            .map(|(key, _)| key.to_owned())
            .collect())
    }

    /// Returns all the definitions of a table. Empty if the table is not in the schema.
    pub fn definitions_by_table_name(&self, table_name: &str) -> Result<Vec<Definition>> {
        Ok(loaded_schema(&self.schema)?.definitions_by_table_name(table_name)
            .map(|definitions| definitions.to_vec())
            .unwrap_or_default())
    }

    /// Returns a definition of a table.
    pub fn definition(&self, table_name: &str, version: i32) -> Result<Definition> {
        loaded_schema(&self.schema)?.definition_by_name_and_version(table_name, version)
            .cloned()
            .ok_or_else(|| anyhow!("No definition found for table '{}' with version {}.", table_name, version))
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

    /// Returns the columns of other tables referencing a table, by table name and column name.
    pub fn referencing_columns(&self, table_name: &str, definition: &Definition) -> Result<HashMap<String, HashMap<String, Vec<String>>>> {
        Ok(loaded_schema(&self.schema)?.referencing_columns_for_table(table_name, definition))
    }

    /// Removes a definition of a table from the loaded schema, without saving it.
    pub fn delete_definition(&mut self, table_name: &str, version: i32) {
        if let Some(ref mut schema) = self.schema {
            schema.remove_definition(table_name, version);
        }
    }
}
