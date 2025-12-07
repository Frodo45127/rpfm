//---------------------------------------------------------------------------//
// Copyright (c) 2017-2024 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Module with all the code to deal with the settings used to configure this program.

use qt_widgets::QApplication;

use qt_core::QBox;
use qt_core::QByteArray;
use qt_core::QSettings;
use qt_core::QString;
use qt_core::QStringList;
use qt_core::QVariant;

use anyhow::{anyhow, Result};
use ron::ser::{PrettyConfig, to_string_pretty};

use std::collections::HashMap;
use std::fs::{DirBuilder, File};
use std::io::{BufWriter, Write};
use std::path::PathBuf;

use rpfm_extensions::optimizer::OptimizerOptions;

use rpfm_lib::error::RLibError;
use rpfm_lib::games::{*, supported_games::*};
use rpfm_lib::schema::{SCHEMA_FOLDER, DefinitionPatch};

use rpfm_ui_common::{SETTINGS, settings::{Settings, config_path, error_path}};

use crate::app_ui::AppUI;
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

pub unsafe fn init_settings(app_ui: &AppUI) -> Result<()> {
    let mut settings = match Settings::read() {
        Ok(settings) => settings,
        Err(_) => Settings::default(),
    };

    settings.set_block_write(true);

    settings.initialize_string(MYMOD_BASE_PATH, "");
    settings.initialize_string(SECONDARY_PATH, "");

    for game in &SUPPORTED_GAMES.games() {
        let game_key = game.key();

        // Fix unsanitized paths.
        let current_path = settings.string(game_key);
        if current_path.contains("\\") {
            let _ = settings.set_string(game_key, &current_path.replace("\\", "/"));
        }

        let game_path = if let Ok(Some(game_path)) = game.find_game_install_location() {
            game_path.to_string_lossy().replace("\\", "/")
        } else {
            String::new()
        };

        // If we got a path and we don't have it saved yet, save it automatically.
        if current_path.is_empty() && !game_path.is_empty() {
            let _ = settings.set_string(game_key, &game_path);
        } else {
            settings.initialize_string(game_key, &game_path);
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
            let current_path = settings.string(&ak_key);

            // Fix unsanitized paths.
            if current_path.contains("\\") {
                let _ = settings.set_string(&ak_key, &current_path.replace("\\", "/"));
            }

            // Ignore shogun 2, as that one is a zip.
            if current_path.is_empty() && !ak_path.is_empty() && game_key != KEY_SHOGUN_2 {
                let _ = settings.set_string(&ak_key, &ak_path);
            } else {
                settings.initialize_string(&ak_key, &ak_path);
            }
        }
    }

    // Hidden setting.
    settings.initialize_bool("import_from_qt", false);

    // General Settings.
    settings.initialize_string("default_game", KEY_WARHAMMER_3);
    settings.initialize_string("language", "English_en");
    settings.initialize_string("update_channel", STABLE);
    settings.initialize_i32("autosave_amount", 10);
    settings.initialize_i32("autosave_interval", 5);

    let font = QApplication::font();
    let font_name = font.family().to_std_string();
    let font_size = font.point_size();
    settings.initialize_string("font_name", &font_name);
    settings.initialize_i32("font_size", font_size);
    settings.initialize_string("original_font_name", &font_name);
    settings.initialize_i32("original_font_size", font_size);

    // UI Settings.
    settings.initialize_bool("start_maximized", false);
    settings.initialize_bool("use_dark_theme", false);
    settings.initialize_bool("hide_background_icon", true);
    settings.initialize_bool("allow_editing_of_ca_packfiles", false);
    settings.initialize_bool("check_updates_on_start", true);
    settings.initialize_bool("check_schema_updates_on_start", true);
    settings.initialize_bool("check_lua_autogen_updates_on_start", true);
    settings.initialize_bool("check_old_ak_updates_on_start", true);
    settings.initialize_bool("use_lazy_loading", true);
    settings.initialize_bool("optimize_not_renamed_packedfiles", false);
    settings.initialize_bool("disable_uuid_regeneration_on_db_tables", true);
    settings.initialize_bool("packfile_treeview_resize_to_fit", false);
    settings.initialize_bool("expand_treeview_when_adding_items", true);
    settings.initialize_bool("use_right_size_markers", false);
    settings.initialize_bool("disable_file_previews", false);
    settings.initialize_bool("include_base_folder_on_add_from_folder", true);
    settings.initialize_bool("delete_empty_folders_on_delete", true);
    settings.initialize_bool("autosave_folder_size_warning_triggered", false);
    settings.initialize_bool("ignore_game_files_in_ak", false);
    settings.initialize_bool("enable_multifolder_filepicker", false);
    settings.initialize_bool("enable_pack_contents_drag_and_drop", true);

    // Table Settings.
    settings.initialize_bool("adjust_columns_to_content", true);
    settings.initialize_bool("extend_last_column_on_tables", true);
    settings.initialize_bool("disable_combos_on_tables", false);
    settings.initialize_bool("tight_table_mode", false);
    settings.initialize_bool("table_resize_on_edit", false);
    settings.initialize_bool("tables_use_old_column_order", true);
    settings.initialize_bool("tables_use_old_column_order_for_tsv", true);
    settings.initialize_bool("enable_lookups", true);
    settings.initialize_bool("enable_icons", true);
    settings.initialize_bool("enable_diff_markers", true);
    settings.initialize_bool("hide_unused_columns", true);

    // Debug Settings.
    settings.initialize_bool("check_for_missing_table_definitions", false);
    settings.initialize_bool("enable_debug_menu", false);
    settings.initialize_bool("spoof_ca_authoring_tool", false);
    settings.initialize_bool("enable_rigidmodel_editor", true);
    settings.initialize_bool("enable_unit_editor", false);
    settings.initialize_bool("enable_esf_editor", false);
    settings.initialize_bool("use_debug_view_unit_variant", false);
    #[cfg(feature = "support_model_renderer")] settings.initialize_bool("enable_renderer", true);

    // Diagnostics Settings
    settings.initialize_bool("diagnostics_trigger_on_open", true);
    settings.initialize_bool("diagnostics_trigger_on_table_edit", true);

    settings.initialize_vec_string("recentFileList", &[]);

    // These settings need to use QSettings because they're read in the C++ side.
    settings.initialize_raw_data("originalGeometry", &app_ui.main_window().save_geometry().as_slice().iter().map(|x| *x as u8).collect::<Vec<_>>());
    settings.initialize_raw_data("originalWindowState", &app_ui.main_window().save_state_0a().as_slice().iter().map(|x| *x as u8).collect::<Vec<_>>());

    // This one needs to be checked here, due to how the ui works.
    app_ui.menu_bar_debug().menu_action().set_visible(settings.bool("enable_debug_menu"));

    // Colours.
    let q_settings = qt_core::QSettings::new();
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

    // Optimizer settings.
    let opt = OptimizerOptions::default();
    settings.initialize_bool("pack_remove_itm_files", *opt.pack_remove_itm_files());
    settings.initialize_bool("db_import_datacores_into_twad_key_deletes", *opt.db_import_datacores_into_twad_key_deletes());
    settings.initialize_bool("db_optimize_datacored_tables", *opt.db_optimize_datacored_tables());
    settings.initialize_bool("table_remove_duplicated_entries", *opt.table_remove_duplicated_entries());
    settings.initialize_bool("table_remove_itm_entries", *opt.table_remove_itm_entries());
    settings.initialize_bool("table_remove_itnr_entries", *opt.table_remove_itnr_entries());
    settings.initialize_bool("table_remove_empty_file", *opt.table_remove_empty_file());
    settings.initialize_bool("text_remove_unused_xml_map_folders", *opt.text_remove_unused_xml_map_folders());
    settings.initialize_bool("text_remove_unused_xml_prefab_folder", *opt.text_remove_unused_xml_prefab_folder());
    settings.initialize_bool("text_remove_agf_files", *opt.text_remove_agf_files());
    settings.initialize_bool("text_remove_model_statistics_files", *opt.text_remove_model_statistics_files());
    settings.initialize_bool("pts_remove_unused_art_sets", *opt.pts_remove_unused_art_sets());
    settings.initialize_bool("pts_remove_unused_variants", *opt.pts_remove_unused_variants());
    settings.initialize_bool("pts_remove_empty_masks", *opt.pts_remove_empty_masks());
    settings.initialize_bool("pts_remove_empty_file", *opt.pts_remove_empty_file());

    settings.set_block_write(false);

    if !settings.bool("import_from_qt") {
        import_from_q_settings(&mut settings);
    }

    settings.write()?;
    *SETTINGS.write().unwrap() = settings;

    Ok(())
}

pub fn import_from_q_settings(settings: &mut Settings) {

    unsafe {
        let q_settings = QSettings::new();

        settings.bool.iter_mut().for_each(|(key, set)| {
            let default = QVariant::from_bool(*set);
            let key = QString::from_std_str(key);
            *set = q_settings.value_2a(&key, &default).to_bool();
        });

        settings.i32.iter_mut().for_each(|(key, set)| {
            let default = QVariant::from_int(*set);
            let key = QString::from_std_str(key);
            *set = q_settings.value_2a(&key, &default).to_int_0a();
        });

        settings.f32.iter_mut().for_each(|(key, set)| {
            let default = QVariant::from_float(*set);
            let key = QString::from_std_str(key);
            *set = q_settings.value_2a(&key, &default).to_float_0a();
        });

        settings.string.iter_mut().for_each(|(key, set)| {
            let default = QVariant::from_q_string(&QString::from_std_str(&*set));
            let key = QString::from_std_str(key);
            *set = q_settings.value_2a(&key, &default).to_string().to_std_string();
        });

        settings.raw_data.iter_mut().for_each(|(key, set)| {
            let default = QVariant::from_q_byte_array(&QByteArray::from_slice(set));
            let key = QString::from_std_str(key);
            *set = q_settings.value_2a(&key, &default).to_byte_array().as_slice().iter().map(|x| *x as u8).collect::<Vec<_>>();
        });

        settings.vec_string.iter_mut().for_each(|(key, set)| {
            let q_list = QStringList::new();
            for s in set.iter() {
                q_list.append_q_string(&QString::from_std_str(s));
            }

            let default = QVariant::from_q_string_list(&q_list);
            let key = QString::from_std_str(key);
            let new_q_list = q_settings.value_2a(&key, &default).to_string_list();

            let mut new_set = vec![];
            for i in 0..new_q_list.count_0a() {
                new_set.push(new_q_list.at(i).to_std_string());
            }

            *set = new_set;
        });

        let _ = settings.set_bool("import_from_qt", true);
    }
}

//-------------------------------------------------------------------------------//
//                             Extra Helpers
//-------------------------------------------------------------------------------//

pub unsafe fn set_setting_if_new_string(q_settings: &QBox<QSettings>, setting: &str, value: &str) {
    if !q_settings.value_1a(&QString::from_std_str(setting)).is_valid() {
        q_settings.set_value(&QString::from_std_str(setting), &QVariant::from_q_string(&QString::from_std_str(value)));
    }
}

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

pub fn init_optimizer_options() -> OptimizerOptions {
    let mut options = OptimizerOptions::default();

    let settings = SETTINGS.read().unwrap();
    options.set_pack_remove_itm_files(settings.bool("pack_remove_itm_files"));
    options.set_db_import_datacores_into_twad_key_deletes(settings.bool("db_import_datacores_into_twad_key_deletes"));
    options.set_db_optimize_datacored_tables(settings.bool("db_optimize_datacored_tables"));
    options.set_table_remove_duplicated_entries(settings.bool("table_remove_duplicated_entries"));
    options.set_table_remove_itm_entries(settings.bool("table_remove_itm_entries"));
    options.set_table_remove_itnr_entries(settings.bool("table_remove_itnr_entries"));
    options.set_table_remove_empty_file(settings.bool("table_remove_empty_file"));
    options.set_text_remove_unused_xml_map_folders(settings.bool("text_remove_unused_xml_map_folders"));
    options.set_text_remove_unused_xml_prefab_folder(settings.bool("text_remove_unused_xml_prefab_folder"));
    options.set_text_remove_agf_files(settings.bool("text_remove_agf_files"));
    options.set_text_remove_model_statistics_files(settings.bool("text_remove_model_statistics_files"));
    options.set_pts_remove_unused_art_sets(settings.bool("pts_remove_unused_art_sets"));
    options.set_pts_remove_unused_variants(settings.bool("pts_remove_unused_variants"));
    options.set_pts_remove_empty_masks(settings.bool("pts_remove_empty_masks"));
    options.set_pts_remove_empty_file(settings.bool("pts_remove_empty_file"));

    options
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
            let mut base_path = SETTINGS.read().unwrap().path_buf(&format!("{}_assembly_kit", game_selected.key()));
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
