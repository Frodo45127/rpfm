//---------------------------------------------------------------------------//
// Copyright (c) 2017-2024 Ismael Gutiérrez González. All rights reserved.
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

use qt_widgets::QApplication;
use qt_widgets::QMainWindow;

use qt_core::QPtr;

use anyhow::{anyhow, Result};
use ron::ser::{PrettyConfig, to_string_pretty};

use std::collections::HashMap;
use std::fs::{DirBuilder, File};
use std::io::{BufWriter, Write};
use std::path::PathBuf;

use rpfm_lib::error::RLibError;
use rpfm_lib::games::{*, supported_games::*};
use rpfm_lib::schema::{SCHEMA_FOLDER, DefinitionPatch};

pub use rpfm_ui_common::settings::*;

use crate::GAME_SELECTED;
use crate::SUPPORTED_GAMES;
use crate::updater_ui::STABLE;

/// Key of the MyMod path in the settings";
pub const MYMOD_BASE_PATH: &str = "mymods_base_path";
pub const SECONDARY_PATH: &str = "secondary_path";

const DEPENDENCIES_FOLDER: &str = "dependencies";
const TABLE_PATCHES_FOLDER: &str = "table_patches";
const TABLE_PROFILES_FOLDER: &str = "table_profiles";
#[cfg(feature = "enable_tools")] const TRANSLATIONS_LOCAL_FOLDER: &str = "translations_local";
#[cfg(feature = "enable_tools")] const TRANSLATIONS_REMOTE_FOLDER: &str = "translations_remote";

//-------------------------------------------------------------------------------//
//                         Setting-related functions
//-------------------------------------------------------------------------------//

pub unsafe fn init_settings(main_window: &QPtr<QMainWindow>) {
    let q_settings = settings();

    set_setting_if_new_q_byte_array(&q_settings, "originalGeometry", main_window.save_geometry().as_ref());
    set_setting_if_new_q_byte_array(&q_settings, "originalWindowState", main_window.save_state_0a().as_ref());

    set_setting_if_new_string(&q_settings, MYMOD_BASE_PATH, "");
    set_setting_if_new_string(&q_settings, SECONDARY_PATH, "");

    for game in &SUPPORTED_GAMES.games() {
        let game_key = game.key();

        // Fix unsanitized paths.
        let current_path = setting_string_from_q_setting(&q_settings, game_key);
        if current_path.contains("\\") {
            set_setting_string_to_q_setting(&q_settings, game_key, &current_path.replace("\\", "/"));
        }

        let game_path = if let Ok(Some(game_path)) = game.find_game_install_location() {
            game_path.to_string_lossy().replace("\\", "/")
        } else {
            String::new()
        };

        // If we got a path and we don't have it saved yet, save it automatically.
        if current_path.is_empty() && !game_path.is_empty() {
            set_setting_string_to_q_setting(&q_settings, game_key, &game_path);
        } else {
            set_setting_if_new_string(&q_settings, game_key, &game_path);
        }

        if game_key != KEY_EMPIRE &&
            game_key != KEY_NAPOLEON &&
            game_key != KEY_ARENA {

            let ak_path = if let Ok(Some(ak_path)) = game.find_assembly_kit_install_location() {
                ak_path.join("assembly_kit").to_string_lossy().replace("\\", "/")
            } else {
                String::new()
            };

            // If we got a path and we don't have it saved yet, save it automatically.
            let ak_key = game_key.to_owned() + "_assembly_kit";
            let current_path = setting_string_from_q_setting(&q_settings, &ak_key);

            // Fix unsanitized paths.
            if current_path.contains("\\") {
                set_setting_string_to_q_setting(&q_settings, &ak_key, &current_path.replace("\\", "/"));
            }

            // Ignore shogun 2, as that one is a zip.
            if current_path.is_empty() && !ak_path.is_empty() && game_key != KEY_SHOGUN_2 {
                set_setting_string_to_q_setting(&q_settings, &ak_key, &ak_path);
            } else {
                set_setting_if_new_string(&q_settings, &ak_key, &ak_path);
            }
        }
    }

    // General Settings.
    set_setting_if_new_string(&q_settings, "default_game", KEY_WARHAMMER_3);
    set_setting_if_new_string(&q_settings, "language", "English_en");
    set_setting_if_new_string(&q_settings, "update_channel", STABLE);
    set_setting_if_new_int(&q_settings, "autosave_amount", 10);
    set_setting_if_new_int(&q_settings, "autosave_interval", 5);

    let font = QApplication::font();
    let font_name = font.family().to_std_string();
    let font_size = font.point_size();
    set_setting_if_new_string(&q_settings, "font_name", &font_name);
    set_setting_if_new_int(&q_settings, "font_size", font_size);
    set_setting_if_new_string(&q_settings, "original_font_name", &font_name);
    set_setting_if_new_int(&q_settings, "original_font_size", font_size);

    // UI Settings.
    set_setting_if_new_bool(&q_settings, "start_maximized", false);
    set_setting_if_new_bool(&q_settings, "use_dark_theme", false);
    set_setting_if_new_bool(&q_settings, "hide_background_icon", true);
    set_setting_if_new_bool(&q_settings, "allow_editing_of_ca_packfiles", false);
    set_setting_if_new_bool(&q_settings, "check_updates_on_start", true);
    set_setting_if_new_bool(&q_settings, "check_schema_updates_on_start", true);
    set_setting_if_new_bool(&q_settings, "check_lua_autogen_updates_on_start", true);
    set_setting_if_new_bool(&q_settings, "check_old_ak_updates_on_start", true);
    set_setting_if_new_bool(&q_settings, "use_lazy_loading", true);
    set_setting_if_new_bool(&q_settings, "optimize_not_renamed_packedfiles", false);
    set_setting_if_new_bool(&q_settings, "disable_uuid_regeneration_on_db_tables", true);
    set_setting_if_new_bool(&q_settings, "packfile_treeview_resize_to_fit", false);
    set_setting_if_new_bool(&q_settings, "expand_treeview_when_adding_items", true);
    set_setting_if_new_bool(&q_settings, "use_right_size_markers", false);
    set_setting_if_new_bool(&q_settings, "disable_file_previews", false);
    set_setting_if_new_bool(&q_settings, "include_base_folder_on_add_from_folder", true);
    set_setting_if_new_bool(&q_settings, "delete_empty_folders_on_delete", true);
    set_setting_if_new_bool(&q_settings, "autosave_folder_size_warning_triggered", false);
    set_setting_if_new_bool(&q_settings, "ignore_game_files_in_ak", false);
    set_setting_if_new_bool(&q_settings, "enable_multifolder_filepicker", false);
    set_setting_if_new_bool(&q_settings, "enable_pack_contents_drag_and_drop", true);

    // Table Settings.
    set_setting_if_new_bool(&q_settings, "adjust_columns_to_content", true);
    set_setting_if_new_bool(&q_settings, "extend_last_column_on_tables", true);
    set_setting_if_new_bool(&q_settings, "disable_combos_on_tables", false);
    set_setting_if_new_bool(&q_settings, "tight_table_mode", false);
    set_setting_if_new_bool(&q_settings, "table_resize_on_edit", false);
    set_setting_if_new_bool(&q_settings, "tables_use_old_column_order", true);
    set_setting_if_new_bool(&q_settings, "tables_use_old_column_order_for_tsv", true);
    set_setting_if_new_bool(&q_settings, "enable_lookups", true);
    set_setting_if_new_bool(&q_settings, "enable_icons", true);
    set_setting_if_new_bool(&q_settings, "enable_diff_markers", true);

    // Debug Settings.
    set_setting_if_new_bool(&q_settings, "check_for_missing_table_definitions", false);
    set_setting_if_new_bool(&q_settings, "enable_debug_menu", false);
    set_setting_if_new_bool(&q_settings, "spoof_ca_authoring_tool", false);
    set_setting_if_new_bool(&q_settings, "enable_rigidmodel_editor", true);
    set_setting_if_new_bool(&q_settings, "enable_unit_editor", false);
    set_setting_if_new_bool(&q_settings, "enable_esf_editor", false);
    #[cfg(feature = "support_model_renderer")] set_setting_if_new_bool(&q_settings, "enable_renderer", true);

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

    q_settings.sync();
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
    DirBuilder::new().recursive(true).create(&config_path)?;
    DirBuilder::new().recursive(true).create(backup_autosave_path()?)?;
    DirBuilder::new().recursive(true).create(error_path()?)?;
    DirBuilder::new().recursive(true).create(schemas_path()?)?;
    DirBuilder::new().recursive(true).create(table_patches_path()?)?;
    DirBuilder::new().recursive(true).create(table_profiles_path()?)?;
    DirBuilder::new().recursive(true).create(old_ak_files_path()?)?;

    // Schema patches need their file existing to even save.
    let games = SupportedGames::default();
    for game in games.games_sorted() {
        let path = table_patches_path().unwrap().join(game.schema_file_name());
        if !path.is_file() {
            let base: HashMap<String, DefinitionPatch> = HashMap::new();
            let mut file = BufWriter::new(File::create(path)?);
            let config = PrettyConfig::default();
            file.write_all(to_string_pretty(&base, config)?.as_bytes())?;
        }
    }

    #[cfg(feature = "support_model_renderer")] {
        let assets_path = format!("{}/assets/", rpfm_ui_common::ASSETS_PATH.to_string_lossy());
        if !PathBuf::from(&assets_path).is_dir() {
            DirBuilder::new().recursive(true).create(&assets_path)?;
        }

        unsafe {crate::ffi::set_asset_folder(&assets_path); }

        let log_path = config_path.to_string_lossy();
        unsafe {crate::ffi::set_log_folder(&log_path); }
    }

    Ok(())
}

/// This function returns the schema path.
pub fn schemas_path() -> Result<PathBuf> {
    Ok(config_path()?.join(SCHEMA_FOLDER))
}

pub fn table_patches_path() -> Result<PathBuf> {
    Ok(config_path()?.join(TABLE_PATCHES_FOLDER))
}

pub fn table_profiles_path() -> Result<PathBuf> {
    Ok(config_path()?.join(TABLE_PROFILES_FOLDER))
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

/// This function returns the autosave path.
pub fn backup_autosave_path() -> Result<PathBuf> {
    Ok(config_path()?.join("autosaves"))
}

/// This function returns the dependencies path.
pub fn dependencies_cache_path() -> Result<PathBuf> {
    Ok(config_path()?.join(DEPENDENCIES_FOLDER))
}

pub fn old_ak_files_path() -> Result<PathBuf> {
    Ok(config_path()?.join("old_ak_files"))
}

#[cfg(feature = "enable_tools")]
pub fn translations_local_path() -> Result<PathBuf> {
    Ok(config_path()?.join(TRANSLATIONS_LOCAL_FOLDER))
}

#[cfg(feature = "enable_tools")]
pub fn translations_remote_path() -> Result<PathBuf> {
    Ok(config_path()?.join(TRANSLATIONS_REMOTE_FOLDER))
}

/// This function returns the dependencies path.
pub fn assembly_kit_path() -> Result<PathBuf> {
    let game_selected = GAME_SELECTED.read().unwrap();
    let version = *game_selected.raw_db_version();
    match version {

        // Post-Shogun 2 games.
        2 | 1 => {
            let mut base_path = setting_path(&format!("{}_assembly_kit", game_selected.key()));
            base_path.push("raw_data/db");
            Ok(base_path)
        }

        0 => {
            let base_path = old_ak_files_path()?.join(game_selected.key());
            Ok(base_path)
        },

        // Shogun 2/Older games
        _ => Err(RLibError::AssemblyKitUnsupportedVersion(version).into())
    }
}
