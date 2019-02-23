//---------------------------------------------------------------------------//
// Copyright (c) 2017-2019 Ismael Gutiérrez González. All rights reserved.
// 
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
// 
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

// In this module should be everything related to the settings stuff.

use serde_derive::{Serialize, Deserialize};

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::fs::File;
use std::io::Write;
use std::io::{BufReader, BufWriter};

use crate::RPFM_PATH;
use crate::SUPPORTED_GAMES;
use crate::error::Result;
use crate::packfile::PFHVersion;

pub mod shortcuts;

const SETTINGS_FILE: &str = "settings.json";

/// `GameInfo`: This struct holds all the info needed for a game to be "supported" by RPFM features.
/// It's stores the following data:
/// - `display_name`: This is the name it'll show up in the UI. For example, in a dropdown (Warhammer 2).
/// - `id`: This is the ID used at the start of every PackFile for that game. (PFH5)
/// - `schema`: This is the name of the schema file used for the game. (wh2.json)
/// - `db_packs`: These are the PackFiles from where we load the data for db references. Since 1.0, we use data.pack or equivalent for this.
/// - `loc_packs`: These are the PackFiles from where we load the data for loc special stuff. This should be the one for english. For other languages, we'll have to search it.
/// - `steam_id`: This is the "SteamID" used by the game, if it's on steam. If not, it's just None.
/// - `raw_files_version`: This is the **type** of raw files the game uses. -1 is "Don't have Assembly Kit". 0 is Empire/Nappy. 1 is Shogun 2. 2 is anything newer than Shogun 2.
/// - `pak_file`: This is the file containing the processed data from the raw db files from the Assembly Kit. If no Asskit is released for the game, set this to none.
/// - `ca_types_file`: This is the file used for checking scripts with Kailua. If there is no file, set it as None.
/// - `supports_editing`: True if we can save PackFiles for this game. False if we cannot (Arena). This also affect if we can use this game for "MyMod" stuff.
#[derive(Clone, Debug)]
pub struct GameInfo {
    pub display_name: String,
    pub id: PFHVersion,
    pub schema: String,
    pub db_packs: Vec<String>,
    pub loc_packs: Vec<String>,
    pub steam_id: Option<u64>,
    pub raw_db_version: i16,
    pub pak_file: Option<String>,
    pub ca_types_file: Option<String>,
    pub supports_editing: bool,
}

/// This struct hold every setting of the program, and it's the one that we are going to serialize.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Settings {
    pub paths: BTreeMap<String, Option<PathBuf>>,
    pub settings_string: BTreeMap<String, String>,
    pub settings_bool: BTreeMap<String, bool>,
}

/// Implementation of `Settings`.
impl Settings {

    /// This function creates a new settings file with default values and loads it into memory.
    /// Should be run if no settings file has been found at the start of the program. It requires
    /// the list of supported games, so it can store the game paths properly.
    pub fn new() -> Self {

        // Create the maps to hold the settings.
        let mut paths = BTreeMap::new();
        let mut settings_string = BTreeMap::new();
        let mut settings_bool = BTreeMap::new();

        // Populate the maps with the default shortcuts. New settings MUST BE ADDED HERE.
        paths.insert("mymods_base_path".to_owned(), None);
        
        for (folder_name, _) in SUPPORTED_GAMES.iter() {
            paths.insert(folder_name.to_string(), None);
        }

        // Default Game.
        settings_string.insert("default_game".to_owned(), "warhammer_2".to_owned());

        // UI Settings.
        settings_bool.insert("adjust_columns_to_content".to_owned(), true);
        settings_bool.insert("extend_last_column_on_tables".to_owned(), true);
        settings_bool.insert("disable_combos_on_tables".to_owned(), false);
        settings_bool.insert("start_maximized".to_owned(), false);
        settings_bool.insert("remember_table_state_permanently".to_owned(), false);
        settings_bool.insert("use_dark_theme".to_owned(), false);

        // Behavioral Settings.
        settings_bool.insert("allow_editing_of_ca_packfiles".to_owned(), false);
        settings_bool.insert("check_updates_on_start".to_owned(), true);
        settings_bool.insert("check_schema_updates_on_start".to_owned(), true);
        settings_bool.insert("use_pfm_extracting_behavior".to_owned(), false);
        settings_bool.insert("use_dependency_checker".to_owned(), false);
        settings_bool.insert("use_lazy_loading".to_owned(), true);

        // Debug Settings.
        settings_bool.insert("check_for_missing_table_definitions".to_owned(), false);

        // TableView Specific Settings.
        settings_bool.insert("remember_column_sorting".to_owned(), true);
        settings_bool.insert("remember_column_visual_order".to_owned(), true);
        settings_bool.insert("remember_column_hidden_state".to_owned(), true);

        // Return it.
        Self {
            paths,
            settings_string,
            settings_bool,
        }
    }

    /// This function takes a settings.json file and reads it into a "Settings" object.
    pub fn load() -> Result<Self> {

        let path = RPFM_PATH.to_path_buf().join(PathBuf::from(SETTINGS_FILE));
        let file = BufReader::new(File::open(path)?);

        let mut settings: Self = serde_json::from_reader(file)?;

        // Add/Remove settings missing/no-longer-needed for keeping it update friendly. First, remove the outdated ones, then add the new ones.
        let defaults = Self::new();

        {          
            let mut keys_to_delete = vec![];
            for (key, _) in settings.paths.clone() { if defaults.paths.get(&*key).is_none() { keys_to_delete.push(key); } }
            for key in &keys_to_delete { settings.paths.remove(key); }

            let mut keys_to_delete = vec![];
            for (key, _) in settings.settings_string.clone() { if defaults.settings_string.get(&*key).is_none() { keys_to_delete.push(key); } }
            for key in &keys_to_delete { settings.settings_string.remove(key); }

            let mut keys_to_delete = vec![];
            for (key, _) in settings.settings_bool.clone() { if defaults.settings_bool.get(&*key).is_none() { keys_to_delete.push(key); } }
            for key in &keys_to_delete { settings.settings_bool.remove(key); }
        }

        {          
            for (key, value) in defaults.paths { if settings.paths.get(&*key).is_none() { settings.paths.insert(key, value);  } }
            for (key, value) in defaults.settings_string { if settings.settings_string.get(&*key).is_none() { settings.settings_string.insert(key, value);  } }
            for (key, value) in defaults.settings_bool { if settings.settings_bool.get(&*key).is_none() { settings.settings_bool.insert(key, value);  } }
        }

        Ok(settings)
    }

    /// This function takes the Settings object and saves it into a settings.json file.
    pub fn save(&self) -> Result<()> {

        // Try to open the settings file.
        let path = RPFM_PATH.to_path_buf().join(PathBuf::from(SETTINGS_FILE));
        let mut file = BufWriter::new(File::create(path)?);

        // Try to save the file, and return the result.
        let settings = serde_json::to_string_pretty(self);
        file.write_all(settings.unwrap().as_bytes())?;

        // Return success.
        Ok(())
    }
}
