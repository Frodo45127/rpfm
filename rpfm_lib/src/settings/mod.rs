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
use std::io::{BufReader, BufWriter, Write};

use rpfm_error::Result;

use crate::SUPPORTED_GAMES;
use crate::config::get_config_path;

const SETTINGS_FILE: &str = "settings.json";

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
        settings_string.insert("default_game".to_owned(), "three_kingdoms".to_owned());

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
        settings_bool.insert("use_dependency_checker".to_owned(), false);
        settings_bool.insert("use_lazy_loading".to_owned(), true);
        settings_bool.insert("optimize_not_renamed_packedfiles".to_owned(), false);

        // Debug Settings.
        settings_bool.insert("check_for_missing_table_definitions".to_owned(), false);

        // TableView Specific Settings.
        settings_bool.insert("remember_column_sorting".to_owned(), true);
        settings_bool.insert("remember_column_visual_order".to_owned(), true);

        // Return it.
        Self {
            paths,
            settings_string,
            settings_bool,
        }
    }

    /// This function takes a settings.json file and reads it into a "Settings" object.
    pub fn load() -> Result<Self> {

        let file_path = get_config_path()?.join(SETTINGS_FILE);
        let file = BufReader::new(File::open(file_path)?);

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

    /// This function takes a custom json file and tries to read it into a `Settings` object.
    pub fn load_from_file(file_path: &str) -> Result<Self> {

        let file_path = PathBuf::from(file_path);
        let file = BufReader::new(File::open(file_path)?);

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
        let file_path = get_config_path()?.join(SETTINGS_FILE);
        let mut file = BufWriter::new(File::create(file_path)?);

        // Try to save the file, and return the result.
        let settings = serde_json::to_string_pretty(self);
        file.write_all(settings.unwrap().as_bytes())?;

        // Return success.
        Ok(())
    }
}

