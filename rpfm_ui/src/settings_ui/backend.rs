//---------------------------------------------------------------------------//
// Copyright (c) 2017-2022 Ismael Gutiérrez González. All rights reserved.
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

use qt_core::QBox;
use qt_core::QSettings;
use qt_core::QString;
use qt_core::QVariant;

use anyhow::{anyhow, Result};
use directories::ProjectDirs;

use std::fs::{DirBuilder, File};
use std::path::{Path, PathBuf};

use rpfm_lib::error::RLibError;
use rpfm_lib::games::{*, supported_games::*};
use rpfm_lib::schema::SCHEMA_FOLDER;

use crate::GAME_SELECTED;
use crate::SUPPORTED_GAMES;
use crate::updater::STABLE;

/// Qualifier for the config folder. Only affects MacOS.
const QUALIFIER: &str = "com";

/// Organisation for the config folder. Only affects Windows and MacOS.
const ORGANISATION: &str = "FrodoWazEre";

/// Name of the config folder.
const PROGRAM_NAME: &str = "rpfm";

/// Key of the 7Zip path in the settings";
pub const ZIP_PATH: &str = "7zip_path";

/// Key of the MyMod path in the settings";
pub const MYMOD_BASE_PATH: &str = "mymods_base_path";

const DEPENDENCIES_FOLDER: &str = "dependencies";

//-------------------------------------------------------------------------------//
//                         Setting-related functions
//-------------------------------------------------------------------------------//

pub unsafe fn init_settings() {
    let q_settings = QSettings::from_2_q_string(&QString::from_std_str(ORGANISATION), &QString::from_std_str(PROGRAM_NAME));

    set_setting_if_new_string(&q_settings, MYMOD_BASE_PATH, "");
    set_setting_if_new_string(&q_settings, ZIP_PATH, "");

    for game in &SUPPORTED_GAMES.games() {
        let game_key = game.game_key_name();
        set_setting_if_new_string(&q_settings, &game_key, "");

        if game_key != KEY_EMPIRE &&
            game_key != KEY_NAPOLEON &&
            game_key != KEY_ARENA {

            set_setting_if_new_string(&q_settings, &(game_key + "_assembly_kit"), "");
        }
    }

    // General Settings.
    set_setting_if_new_string(&q_settings, "default_game", KEY_WARHAMMER_3);
    set_setting_if_new_string(&q_settings, "language", "English_en");
    set_setting_if_new_string(&q_settings, "update_channel", STABLE);
    set_setting_if_new_int(&q_settings, "autosave_amount", 10);
    set_setting_if_new_int(&q_settings, "autosave_interval", 5);
    set_setting_if_new_string(&q_settings, "font_name", "");
    set_setting_if_new_string(&q_settings, "font_size", "");

    // UI Settings.
    set_setting_if_new_bool(&q_settings, "start_maximized", false);
    set_setting_if_new_bool(&q_settings, "use_dark_theme", false);
    set_setting_if_new_bool(&q_settings, "hide_background_icon", true);
    set_setting_if_new_bool(&q_settings, "allow_editing_of_ca_packfiles", false);
    set_setting_if_new_bool(&q_settings, "check_updates_on_start", true);
    set_setting_if_new_bool(&q_settings, "check_schema_updates_on_start", true);
    set_setting_if_new_bool(&q_settings, "check_message_updates_on_start", false);
    set_setting_if_new_bool(&q_settings, "check_lua_autogen_updates_on_start", true);
    set_setting_if_new_bool(&q_settings, "use_lazy_loading", true);
    set_setting_if_new_bool(&q_settings, "optimize_not_renamed_packedfiles", false);
    set_setting_if_new_bool(&q_settings, "disable_uuid_regeneration_on_db_tables", true);
    set_setting_if_new_bool(&q_settings, "packfile_treeview_resize_to_fit", false);
    set_setting_if_new_bool(&q_settings, "expand_treeview_when_adding_items", true);
    set_setting_if_new_bool(&q_settings, "use_right_size_markers", false);
    set_setting_if_new_bool(&q_settings, "disable_file_previews", false);

    // Table Settings.
    set_setting_if_new_bool(&q_settings, "adjust_columns_to_content", true);
    set_setting_if_new_bool(&q_settings, "extend_last_column_on_tables", true);
    set_setting_if_new_bool(&q_settings, "disable_combos_on_tables", false);
    set_setting_if_new_bool(&q_settings, "tight_table_mode", false);
    set_setting_if_new_bool(&q_settings, "table_resize_on_edit", false);
    set_setting_if_new_bool(&q_settings, "tables_use_old_column_order", true);

    // Debug Settings.
    set_setting_if_new_bool(&q_settings, "check_for_missing_table_definitions", false);
    set_setting_if_new_bool(&q_settings, "enable_debug_menu", false);
    set_setting_if_new_bool(&q_settings, "spoof_ca_authoring_tool", false);
    set_setting_if_new_bool(&q_settings, "enable_rigidmodel_editor", true);
    set_setting_if_new_bool(&q_settings, "enable_esf_editor", false);
    set_setting_if_new_bool(&q_settings, "enable_unit_editor", false);

    // Diagnostics Settings
    set_setting_if_new_bool(&q_settings, "diagnostics_trigger_on_open", true);
    set_setting_if_new_bool(&q_settings, "diagnostics_trigger_on_table_edit", true);

    // Colours.
    set_setting_if_new_string(&q_settings, "colour_light_table_added", "#87ca00");
    set_setting_if_new_string(&q_settings, "colour_light_table_modified", "#e67e22");
    set_setting_if_new_string(&q_settings, "colour_light_diagnostic_error", "#ff0000");
    set_setting_if_new_string(&q_settings, "colour_light_diagnostic_warning", "#bebe00");
    set_setting_if_new_string(&q_settings, "colour_light_diagnostic_info", "#55aaff");
    set_setting_if_new_string(&q_settings, "colour_dark_table_added", "#00ff00");
    set_setting_if_new_string(&q_settings, "colour_dark_table_modified", "#e67e22");
    set_setting_if_new_string(&q_settings, "colour_dark_diagnostic_error", "#ff0000");
    set_setting_if_new_string(&q_settings, "colour_dark_diagnostic_warning", "#cece67");
    set_setting_if_new_string(&q_settings, "colour_dark_diagnostic_info", "#55aaff");

    set_setting_if_new_string(&q_settings, "colour_light_local_tip", "#363636");
    set_setting_if_new_string(&q_settings, "colour_light_remote_tip", "#7e7e7e");
    set_setting_if_new_string(&q_settings, "colour_dark_local_tip", "#363636");
    set_setting_if_new_string(&q_settings, "colour_dark_remote_tip", "#7e7e7e");

    q_settings.sync();
}

pub fn settings() -> QBox<QSettings> {
    unsafe { QSettings::from_2_q_string(&QString::from_std_str(ORGANISATION), &QString::from_std_str(PROGRAM_NAME)) }
}

#[allow(dead_code)]
pub fn setting_path(setting: &str) -> PathBuf {
    unsafe {
        let q_settings = QSettings::from_2_q_string(&QString::from_std_str(ORGANISATION), &QString::from_std_str(PROGRAM_NAME));
        PathBuf::from(q_settings.value_1a(&QString::from_std_str(setting)).to_string().to_std_string())
    }
}

#[allow(dead_code)]
pub fn setting_string(setting: &str) -> String {
    unsafe {
        let q_settings = QSettings::from_2_q_string(&QString::from_std_str(ORGANISATION), &QString::from_std_str(PROGRAM_NAME));
        q_settings.value_1a(&QString::from_std_str(setting)).to_string().to_std_string()
    }
}

#[allow(dead_code)]
pub fn setting_int(setting: &str) -> i32 {
    unsafe {
        let q_settings = QSettings::from_2_q_string(&QString::from_std_str(ORGANISATION), &QString::from_std_str(PROGRAM_NAME));
        q_settings.value_1a(&QString::from_std_str(setting)).to_int_0a()
    }
}

#[allow(dead_code)]
pub fn setting_bool(setting: &str) -> bool {
    unsafe {
        let q_settings = QSettings::from_2_q_string(&QString::from_std_str(ORGANISATION), &QString::from_std_str(PROGRAM_NAME));
        q_settings.value_1a(&QString::from_std_str(setting)).to_bool()
    }
}

#[allow(dead_code)]
pub fn set_setting_path(setting: &str, value: &Path) {
    set_setting_string(setting, &value.to_string_lossy())
}

#[allow(dead_code)]
pub fn set_setting_string(setting: &str, value: &str) {
    unsafe {
        let q_settings = QSettings::from_2_q_string(&QString::from_std_str(ORGANISATION), &QString::from_std_str(PROGRAM_NAME));
        q_settings.set_value(&QString::from_std_str(setting), &QVariant::from_q_string(&QString::from_std_str(value)));
        q_settings.sync();
    }
}

#[allow(dead_code)]
pub fn set_setting_int(setting: &str, value: i32) {
    unsafe {
        let q_settings = QSettings::from_2_q_string(&QString::from_std_str(ORGANISATION), &QString::from_std_str(PROGRAM_NAME));
        q_settings.set_value(&QString::from_std_str(setting), &QVariant::from_int(value));
        q_settings.sync();
    }
}

#[allow(dead_code)]
pub fn set_setting_bool(setting: &str, value: bool) {
    unsafe {
        let q_settings = QSettings::from_2_q_string(&QString::from_std_str(ORGANISATION), &QString::from_std_str(PROGRAM_NAME));
        q_settings.set_value(&QString::from_std_str(setting), &QVariant::from_bool(value));
        q_settings.sync();
    }
}

#[allow(dead_code)]
pub fn set_setting_path_to_q_setting(q_settings: &QBox<QSettings>, setting: &str, value: &Path) {
    set_setting_string_to_q_setting(q_settings, setting, &value.to_string_lossy())
}

#[allow(dead_code)]
pub fn set_setting_string_to_q_setting(q_settings: &QBox<QSettings>, setting: &str, value: &str) {
    unsafe {
        q_settings.set_value(&QString::from_std_str(setting), &QVariant::from_q_string(&QString::from_std_str(value)));
    }
}

#[allow(dead_code)]
pub fn set_setting_int_to_q_setting(q_settings: &QBox<QSettings>, setting: &str, value: i32) {
    unsafe {
        q_settings.set_value(&QString::from_std_str(setting), &QVariant::from_int(value));
    }
}

#[allow(dead_code)]
pub fn set_setting_bool_to_q_setting(q_settings: &QBox<QSettings>, setting: &str, value: bool) {
    unsafe {
        q_settings.set_value(&QString::from_std_str(setting), &QVariant::from_bool(value));
    }
}

#[allow(dead_code)]
pub fn set_setting_if_new_path(q_settings: &QBox<QSettings>, setting: &str, value: &Path) {
    set_setting_if_new_string(q_settings, setting, &value.to_string_lossy())
}

#[allow(dead_code)]
pub fn set_setting_if_new_string(q_settings: &QBox<QSettings>, setting: &str, value: &str) {
    unsafe {
        if !q_settings.value_1a(&QString::from_std_str(setting)).is_valid() {
            q_settings.set_value(&QString::from_std_str(setting), &QVariant::from_q_string(&QString::from_std_str(value)));
        }
    }
}

#[allow(dead_code)]
pub fn set_setting_if_new_int(q_settings: &QBox<QSettings>, setting: &str, value: i32) {
    unsafe {
        if !q_settings.value_1a(&QString::from_std_str(setting)).is_valid() {
            q_settings.set_value(&QString::from_std_str(setting), &QVariant::from_int(value));
        }
    }
}

#[allow(dead_code)]
pub fn set_setting_if_new_bool(q_settings: &QBox<QSettings>, setting: &str, value: bool) {
    unsafe {
        if !q_settings.value_1a(&QString::from_std_str(setting)).is_valid() {
            q_settings.set_value(&QString::from_std_str(setting), &QVariant::from_bool(value));
        }
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

    let config_path = config_path()?;
    let autosaves_path = config_path.join("autosaves");
    let error_path = config_path.join("error");
    let schemas_path = config_path.join("schemas");
    let tips_local_path = config_path.join("tips/local");
    let tips_remote_path = config_path.join("tips/remote");

    DirBuilder::new().recursive(true).create(&autosaves_path)?;
    DirBuilder::new().recursive(true).create(&config_path)?;
    DirBuilder::new().recursive(true).create(&error_path)?;
    DirBuilder::new().recursive(true).create(&schemas_path)?;
    DirBuilder::new().recursive(true).create(&tips_local_path)?;
    DirBuilder::new().recursive(true).create(&tips_remote_path)?;

    // Init autosave files if they're not yet initialized. Minimum 1.
    let mut max_autosaves = setting_int("autosave_amount");
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
pub fn config_path() -> Result<PathBuf> {
    if cfg!(debug_assertions) { std::env::current_dir().map_err(From::from) } else {
        match ProjectDirs::from(QUALIFIER, ORGANISATION, PROGRAM_NAME) {
            Some(proj_dirs) => Ok(proj_dirs.config_dir().to_path_buf()),
            None => Err(anyhow!("Failed to get the config path."))
        }
    }
}

/// This function returns the path where crash logs are stored.
pub fn error_path() -> Result<PathBuf> {
    Ok(config_path()?.join("error"))
}

/// This function returns the schema path.
pub fn schemas_path() -> Result<PathBuf> {
    Ok(config_path()?.join(SCHEMA_FOLDER))
}

/// This function returns the lua autogen path.
pub fn lua_autogen_base_path() -> Result<PathBuf> {
    Ok(config_path()?.join(LUA_AUTOGEN_FOLDER))
}

/// This function returns the lua autogen path for a specific game.
pub fn lua_autogen_game_path(game: &GameInfo) -> Result<PathBuf> {
    match game.lua_autogen_folder() {
        Some(folder) => Ok(config_path()?.join(LUA_AUTOGEN_FOLDER).join(folder)),
        None => Err(anyhow!("Lua Autogen not available for this game."))
    }
}

/// This function returns the remote tips path.
//pub fn remote_tips_path() -> Result<PathBuf> {
//    Ok(config_path()?.join(TIPS_REMOTE_FOLDER))
//}

/// This function returns the autosave path.
pub fn backup_autosave_path() -> Result<PathBuf> {
    Ok(config_path()?.join("autosaves"))
}

/// This function returns the dependencies path.
pub fn dependencies_cache_path() -> Result<PathBuf> {
    Ok(config_path()?.join(DEPENDENCIES_FOLDER))
}

/// This function returns the dependencies path.
pub fn assembly_kit_path() -> Result<PathBuf> {
    let game_selected = GAME_SELECTED.read().unwrap();
    let mut base_path = setting_path(&format!("{}_assembly_kit", game_selected.game_key_name()));
    let version = game_selected.raw_db_version();
    match version {

        // Post-Shogun 2 games.
        2 | 1 => {
            base_path.push("raw_data/db");
            Ok(base_path)
        }

        // Shogun 2/Older games
        _ => Err(RLibError::AssemblyKitUnsupportedVersion(version).into())
    }
}
