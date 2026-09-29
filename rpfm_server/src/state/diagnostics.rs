//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Diagnostics and Lua script operations.

use anyhow::{anyhow, Result};

use rpfm_extensions::diagnostics::Diagnostics;
use rpfm_extensions::lua::check::{check_script, LuaDefinitions};
use rpfm_extensions::lua::harness::{run_tests, LuaScripts, LuaTestOptions, LuaTestReport};

use rpfm_lib::files::ContainerPath;

use rpfm_telemetry::info;

use crate::settings::Settings;

use super::{SessionState, cached_lua_api};

/// A hover of a Lua script: its start line and column, end line and column (all 0-based), and its docs as rich text.
pub type LuaHover = (u64, u64, u64, u64, String);

impl SessionState {

    /// Checks the open packs for problems.
    ///
    /// Does nothing if there is no schema.
    ///
    /// # Arguments
    ///
    /// * `diagnostics` - Results of a previous check, or a new one with the diagnostics to ignore.
    /// * `paths_to_check` - Paths to recheck, keeping the previous results for the rest. If empty, everything is checked.
    /// * `check_ak_only_refs` - If references to tables only in the Assembly Kit are checked.
    /// * `settings` - Settings, to find the game's install folder and Assembly Kit.
    ///
    /// # Returns
    ///
    /// The updated diagnostics.
    pub fn check_diagnostics(&mut self, mut diagnostics: Diagnostics, paths_to_check: &[ContainerPath], check_ak_only_refs: bool, settings: &Settings) -> Diagnostics {
        if let Some(ref schema) = self.schema {
            let game_path = settings.path_buf(self.game.key());
            let lua_api = cached_lua_api(&mut self.lua_api_cache, &self.game, settings, &self.dependencies);
            diagnostics.check(&mut self.packs, &mut self.dependencies, schema, &self.game, &game_path, paths_to_check, check_ak_only_refs, lua_api);
        }

        info!("Checking diagnostics: done.");
        diagnostics
    }

    /// Returns the hovers of a Lua script, with the docs of the game's Lua API. Empty if the API is not available.
    pub fn lua_hovers(&mut self, source: &str, settings: &Settings) -> Vec<LuaHover> {
        let Some(lua_api) = cached_lua_api(&mut self.lua_api_cache, &self.game, settings, &self.dependencies) else { return vec![] };

        check_script(source, Some(lua_api), &LuaDefinitions::default()).hovers().iter()
            .filter_map(|hover| {
                let ((start_line, start_column), (end_line, end_column)) = *hover.range();
                lua_api.hover_html(hover.target()).map(|html| (start_line, start_column, end_line, end_column, html))
            })
            .collect()
    }

    /// Runs Lua tests against the scripts of the game and the open packs, outside of the game.
    ///
    /// # Arguments
    ///
    /// * `test_source` - Code of the Lua test file.
    /// * `campaign` - Campaign whose vanilla scripts to load. If `None`, only the script libraries and the mods are loaded.
    /// * `settings` - Settings, to find the game's Assembly Kit.
    ///
    /// # Returns
    ///
    /// The results of the tests.
    ///
    /// # Errors
    ///
    /// Fails if the game's Lua API is not available, or if the tests can't be loaded.
    pub fn lua_run_tests(&mut self, test_source: &str, campaign: Option<String>, settings: &Settings) -> Result<LuaTestReport> {
        let lua_api = cached_lua_api(&mut self.lua_api_cache, &self.game, settings, &self.dependencies)
            .ok_or_else(|| anyhow!("The Lua API of the game is not available. Lua tests need the game's Assembly Kit to be installed."))?;

        let scripts = LuaScripts::from_game_and_packs(&self.dependencies, &self.packs);
        let key_values = |table_name| self.dependencies.db_key_values(Some(&self.packs), table_name)
            .map(|(_, keys)| keys.into_iter().collect::<Vec<_>>())
            .unwrap_or_default();

        let mut options = LuaTestOptions::default();
        options.set_campaign(campaign);
        options.set_faction_keys(key_values("factions_tables"));
        options.set_region_keys(key_values("regions_tables"));

        Ok(run_tests(lua_api, &scripts, test_source, &options)?)
    }
}
