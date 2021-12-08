//---------------------------------------------------------------------------//
// Copyright (c) 2017-2020 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

/*!
Module with all the code to deal with the settings used to configure this lib.

This module contains all the code related with the settings used by the lib to work. These
settings are saved in the config folder, in a file called `settings.ron`, in case you want
to change them manually.
!*/

use directories::ProjectDirs;
use ron::de::from_reader;
use ron::ser::{to_string_pretty, PrettyConfig};
use serde_derive::{Serialize, Deserialize};

use std::collections::BTreeMap;
use std::fs::{DirBuilder, File};
use std::io::{BufReader, BufWriter, Write};
use std::path::PathBuf;

use rpfm_error::{ErrorKind, Result};

use crate::games::*;
use crate::games::supported_games::*;
use crate::SETTINGS;
use crate::SUPPORTED_GAMES;
use crate::settings::supported_games::KEY_THREE_KINGDOMS;
use crate::updater::STABLE;

/// Qualifier for the config folder. Only affects MacOS.
const QUALIFIER: &str = "";

/// Organisation for the config folder. Only affects Windows and MacOS.
const ORGANISATION: &str = "";

/// Name of the config folder.
const PROGRAM_NAME: &str = "rpfm";

/// Name of the settings file.
const SETTINGS_FILE: &str = "settings.ron";

/// Key of the 7Zip path in the settings";
pub const ZIP_PATH: &str = "7zip_path";

/// Key of the MyMod path in the settings";
pub const MYMOD_BASE_PATH: &str = "mymods_base_path";

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//
//
/// This struct hold every setting of the lib and of RPFM_UI/CLI.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Settings {
    pub paths: BTreeMap<String, Option<PathBuf>>,
    pub settings_string: BTreeMap<String, String>,
    pub settings_bool: BTreeMap<String, bool>,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

/// Implementation of `Settings`.
impl Settings {

    /// This function creates a new default `Settings`.
    ///
    /// Should be run if no settings file has been found at the start of any program using this lib.
    pub fn new() -> Self {
        let mut paths = BTreeMap::new();
        let mut settings_string = BTreeMap::new();
        let mut settings_bool = BTreeMap::new();
        paths.insert(MYMOD_BASE_PATH.to_owned(), None);
        paths.insert(ZIP_PATH.to_owned(), None);
        for game in &SUPPORTED_GAMES.get_games() {
            let game_key = game.get_game_key_name();
            paths.insert(game_key.to_owned(), None);

            if game_key != KEY_EMPIRE &&
                game_key != KEY_NAPOLEON &&
                game_key != KEY_ARENA {

                paths.insert(game.get_game_key_name() + "_assembly_kit", None);
            }
        }

        // General Settings.
        settings_string.insert("default_game".to_owned(), KEY_THREE_KINGDOMS.to_owned());
        settings_string.insert("language".to_owned(), "English_en".to_owned());
        settings_string.insert("update_channel".to_owned(), STABLE.to_owned());
        settings_string.insert("autosave_amount".to_owned(), "10".to_owned());
        settings_string.insert("autosave_interval".to_owned(), "5".to_owned());
        settings_string.insert("font_name".to_owned(), "".to_owned());
        settings_string.insert("font_size".to_owned(), "".to_owned());

        // UI Settings.
        settings_bool.insert("start_maximized".to_owned(), false);
        settings_bool.insert("use_dark_theme".to_owned(), false);
        settings_bool.insert("hide_background_icon".to_owned(), true);
        settings_bool.insert("allow_editing_of_ca_packfiles".to_owned(), false);
        settings_bool.insert("check_updates_on_start".to_owned(), true);
        settings_bool.insert("check_schema_updates_on_start".to_owned(), true);
        settings_bool.insert("use_lazy_loading".to_owned(), true);
        settings_bool.insert("optimize_not_renamed_packedfiles".to_owned(), false);
        settings_bool.insert("disable_uuid_regeneration_on_db_tables".to_owned(), true);
        settings_bool.insert("packfile_treeview_resize_to_fit".to_owned(), false);
        settings_bool.insert("expand_treeview_when_adding_items".to_owned(), true);
        settings_bool.insert("use_right_size_markers".to_owned(), false);
        settings_bool.insert("disable_file_previews".to_owned(), false);

        // Table Settings.
        settings_bool.insert("adjust_columns_to_content".to_owned(), true);
        settings_bool.insert("extend_last_column_on_tables".to_owned(), true);
        settings_bool.insert("disable_combos_on_tables".to_owned(), false);
        settings_bool.insert("tight_table_mode".to_owned(), false);
        settings_bool.insert("table_resize_on_edit".to_owned(), false);
        settings_bool.insert("tables_use_old_column_order".to_owned(), true);

        // Debug Settings.
        settings_bool.insert("check_for_missing_table_definitions".to_owned(), false);
        settings_bool.insert("enable_debug_menu".to_owned(), false);
        settings_bool.insert("spoof_ca_authoring_tool".to_owned(), false);
        settings_bool.insert("enable_rigidmodel_editor".to_owned(), true);
        settings_bool.insert("enable_esf_editor".to_owned(), false);
        settings_bool.insert("enable_unit_editor".to_owned(), false);

        // Diagnostics Settings
        settings_bool.insert("diagnostics_trigger_on_open".to_owned(), true);
        settings_bool.insert("diagnostics_trigger_on_table_edit".to_owned(), true);

        Self {
            paths,
            settings_string,
            settings_bool,
        }
    }

    /// This function tries to load the `settings.ron` from disk, if exist, and return it.
    pub fn load(file_path: Option<&str>) -> Result<Self> {
        let file_path = if let Some(file_path) = file_path { PathBuf::from(file_path) } else { get_config_path()?.join(SETTINGS_FILE) };
        let file = BufReader::new(File::open(file_path)?);
        let mut settings: Self = from_reader(file)?;

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

    /// This function tries to save the provided `Settings` to disk.
    pub fn save(&self) -> Result<()> {
        let file_path = get_config_path()?.join(SETTINGS_FILE);
        let mut file = BufWriter::new(File::create(file_path)?);
        let config = PrettyConfig::default();
        // Append a newline `\n` to the file
        let mut data = to_string_pretty(&self, config)?;
        data.push_str("\n");
        file.write_all(data.as_bytes())?;
        Ok(())
    }
}

//-------------------------------------------------------------------------------//
//                             Extra Helpers
//-------------------------------------------------------------------------------//

/// Function to initialize the config folder, so RPFM can use it to store his stuff.
///
/// This can fail, so if this fails, better stop the program and check why it failed.
#[must_use = "Many things depend on this folder existing. So better check this worked."]
pub fn init_config_path() -> Result<()> {

    let config_path = get_config_path()?;
    let autosaves_path = config_path.join("autosaves");
    let error_path = config_path.join("error");
    let schemas_path = config_path.join("schemas");

    DirBuilder::new().recursive(true).create(&autosaves_path)?;
    DirBuilder::new().recursive(true).create(&config_path)?;
    DirBuilder::new().recursive(true).create(&error_path)?;
    DirBuilder::new().recursive(true).create(&schemas_path)?;

    // Init autosave files if they're not yet initialized. Minimum 1.
    let mut max_autosaves = SETTINGS.read().unwrap().settings_string["autosave_amount"].parse::<i32>().unwrap_or(10);
    if max_autosaves < 1 { max_autosaves = 1; }
    (1..=max_autosaves).for_each(|x| {
        let path = autosaves_path.join(format!("autosave_{:02?}.pack", x));
        if !path.is_file() {
            let _ = File::create(path);
        }
    });

    Ok(())
}

/// This function returns the current config path, or an error if said path is not available.
///
/// Note: On `Debug´ mode this project is the project from where you execute one of RPFM's programs, which should be the root of the repo.
pub fn get_config_path() -> Result<PathBuf> {
    if cfg!(debug_assertions) { std::env::current_dir().map_err(From::from) } else {
        match ProjectDirs::from(QUALIFIER, ORGANISATION, PROGRAM_NAME) {
            Some(proj_dirs) => Ok(proj_dirs.config_dir().to_path_buf()),
            None => Err(ErrorKind::IOFolderCannotBeOpened.into())
        }
    }
}
