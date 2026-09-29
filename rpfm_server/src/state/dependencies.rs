//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Game selection and dependencies operations.

use anyhow::{anyhow, Result};

use rpfm_extensions::dependencies::Dependencies;

use rpfm_ipc::helpers::DependenciesInfo;
use rpfm_ipc::settings_keys::SECONDARY_PATH;

use rpfm_lib::compression::CompressionFormat;

use rpfm_telemetry::info;

use crate::settings::{dependencies_cache_path, Settings};

use super::{SessionState, load_schema, loaded_schema, parent_pack_names};

impl SessionState {

    /// Changes the selected game, loading its schema and adapting the open packs to it.
    ///
    /// If the dependencies are rebuilt, their tables are left undecoded: call [`Self::decode_dependency_tables`] after this.
    ///
    /// # Arguments
    ///
    /// * `game_key` - Key of the game to select.
    /// * `rebuild_dependencies` - If the dependencies are rebuilt for the new game.
    /// * `settings` - Settings, to find the game's install folder.
    /// * `disable_uuid_regeneration` - If tables keep their GUID when re-encoded for the new schema.
    ///
    /// # Returns
    ///
    /// The compression format of the first open pack, and the info of the dependencies if they were rebuilt.
    pub fn set_game_selected(&mut self, game_key: &str, rebuild_dependencies: bool, settings: &Settings, disable_uuid_regeneration: bool) -> Result<(CompressionFormat, Option<DependenciesInfo>)> {
        info!("Setting game selected.");
        let game_changed = self.game.key() != game_key || !self.first_game_change_done;
        self.game = self.supported_games.game(game_key)
            .ok_or_else(|| anyhow!("The selected game is not supported!"))?
            .clone();

        // We need to make sure the compression format is valid for our game for all open packs.
        for pack in self.packs.values_mut() {
            let current_cf = pack.compression_format();
            if current_cf != CompressionFormat::None && !self.game.compression_formats_supported().contains(&current_cf) {
                let new_cf = self.game.compression_formats_supported().first().copied().unwrap_or(CompressionFormat::None);
                pack.set_compression_format(new_cf, &self.game);
            }
        }

        load_schema(&mut self.schema, &mut self.packs, &self.game, disable_uuid_regeneration);

        let dependencies_info = if rebuild_dependencies {
            let cache_path = dependencies_cache_path()?.join(self.game.dependencies_cache_file_name());
            let cache_path = if game_changed { Some(&*cache_path) } else { None };
            let _ = self.dependencies.rebuild(&None, &parent_pack_names(&self.packs), cache_path, &self.game, &settings.path_buf(self.game.key()), &settings.path_buf(SECONDARY_PATH));

            Some(DependenciesInfo::new(&self.dependencies, self.game.vanilla_db_table_name_logic()))
        } else {
            None
        };

        let compression_format = self.packs.values().next()
            .map(|pack| pack.compression_format())
            .unwrap_or(CompressionFormat::None);

        // For all open packs, change their id to match the one of the new game.
        let game_version_number = self.game.game_version_number(&settings.path_buf(self.game.key()));
        for pack in self.packs.values_mut() {
            if !pack.disk_file_path().is_empty() {
                let pfh_file_type = *pack.header().pfh_file_type();
                pack.header_mut().set_pfh_version(self.game.pfh_version_by_file_type(pfh_file_type));

                if let Some(version_number) = game_version_number {
                    pack.set_game_version(version_number);
                }
            }
        }

        self.first_game_change_done = true;
        info!("Switching game selected done.");

        Ok((compression_format, dependencies_info))
    }

    /// Decodes the tables of the dependencies with the loaded schema.
    pub fn decode_dependency_tables(&mut self) {
        self.dependencies.decode_tables(&self.schema);
    }

    /// Generates the dependencies cache of the selected game from its files and Assembly Kit, then loads it.
    ///
    /// # Arguments
    ///
    /// * `settings` - Settings, to find the game's install folder and Assembly Kit.
    /// * `ignore_game_files_in_ak` - If tables in the game files are skipped when reading the Assembly Kit.
    ///
    /// # Returns
    ///
    /// The info of the new dependencies.
    pub fn generate_dependencies_cache(&mut self, settings: &Settings, ignore_game_files_in_ak: bool) -> Result<DependenciesInfo> {
        let game_path = settings.path_buf(self.game.key());
        if !game_path.is_dir() {
            return Err(anyhow!("Game Path not configured. Go to <i>'PackFile/Settings'</i> and configure it."));
        }

        let asskit_path = settings.assembly_kit_path(&self.game).ok();
        let mut cache = Dependencies::generate_dependencies_cache(&self.schema, &self.game, &game_path, &asskit_path, ignore_game_files_in_ak)?;

        let cache_path = dependencies_cache_path()?.join(self.game.dependencies_cache_file_name());
        cache.save(&cache_path)?;

        let _ = self.dependencies.rebuild(&self.schema, &parent_pack_names(&self.packs), Some(&cache_path), &self.game, &game_path, &settings.path_buf(SECONDARY_PATH));
        Ok(DependenciesInfo::new(&self.dependencies, self.game.vanilla_db_table_name_logic()))
    }

    /// Rebuilds the dependencies of the selected game.
    ///
    /// # Arguments
    ///
    /// * `only_parent_packs` - If only the parent packs are reloaded, instead of the whole dependencies.
    /// * `settings` - Settings, to find the game's install folder.
    ///
    /// # Returns
    ///
    /// The info of the rebuilt dependencies.
    pub fn rebuild_dependencies(&mut self, only_parent_packs: bool, settings: &Settings) -> Result<DependenciesInfo> {
        loaded_schema(&self.schema)?;

        let cache_path = dependencies_cache_path()?.join(self.game.dependencies_cache_file_name());
        let cache_path = if only_parent_packs { None } else { Some(&*cache_path) };
        let _ = self.dependencies.rebuild(&self.schema, &parent_pack_names(&self.packs), cache_path, &self.game, &settings.path_buf(self.game.key()), &settings.path_buf(SECONDARY_PATH));

        Ok(DependenciesInfo::new(&self.dependencies, self.game.vanilla_db_table_name_logic()))
    }

    /// Rebuilds the dependencies after the schema changed, if the vanilla data is loaded.
    ///
    /// # Arguments
    ///
    /// * `settings` - Settings, to find the game's install folder.
    pub fn rebuild_dependencies_after_schema_update(&mut self, settings: &Settings) -> Result<()> {
        if !self.dependencies.is_vanilla_data_loaded(false) {
            return Ok(());
        }

        let cache_path = dependencies_cache_path()?.join(self.game.dependencies_cache_file_name());
        self.dependencies.rebuild(&self.schema, &parent_pack_names(&self.packs), Some(&cache_path), &self.game, &settings.path_buf(self.game.key()), &settings.path_buf(SECONDARY_PATH))
            .map_err(|_| anyhow!("Schema updated, but dependencies cache rebuilding failed. You may need to regenerate it."))
    }

    /// Returns if the vanilla data of the dependencies is loaded.
    ///
    /// # Arguments
    ///
    /// * `include_asskit` - If the Assembly Kit data must be loaded too.
    pub fn is_dependency_database_loaded(&self, include_asskit: bool) -> bool {
        self.dependencies.is_vanilla_data_loaded(include_asskit)
    }
}
