//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

use anyhow::{anyhow, Result};
use directories::ProjectDirs;
use ron::ser::{PrettyConfig, to_string_pretty};
use serde_derive::{Serialize, Deserialize};

use std::collections::HashMap;
use std::io::{BufReader, BufWriter, Read, Write};
use std::fs::{DirBuilder, File};
use std::path::{Path, PathBuf};

use rpfm_extensions::optimizer::OptimizerOptions;

use rpfm_ipc::{MYMOD_BASE_PATH, SECONDARY_PATH};

use rpfm_lib::error::RLibError;
use rpfm_lib::games::{LUA_AUTOGEN_FOLDER, supported_games::*};
use rpfm_lib::schema::{DefinitionPatch, SCHEMA_FOLDER};

use crate::*;

const SETTINGS_FILE_NAME: &str = "settings.json";

const DEPENDENCIES_FOLDER: &str = "dependencies";
const TABLE_PATCHES_FOLDER: &str = "table_patches";
const TABLE_PROFILES_FOLDER: &str = "table_profiles";
const TRANSLATIONS_LOCAL_FOLDER: &str = "translations_local";
const TRANSLATIONS_REMOTE_FOLDER: &str = "translations_remote";

//-------------------------------------------------------------------------------//
//                                  Macros
//-------------------------------------------------------------------------------//

/// Macro to set a batch of settings in one go in an efficient way.
///
/// It expects a list of the following:
///
/// - $rtype: The setting's setter (set_bool, set_i32, etc.)
/// - $id: The ID of the setting as a string literal.
/// - $source: The expression to get the value.
///
/// You can add more settings by adding another 3 arguments to the macro.
#[macro_export]
macro_rules! set_batch {
    ($( $rtype:ident, $id:literal, $source:expr), *) => {
        {
            let mut set = SETTINGS.write().unwrap();
            set.set_block_write(true);
            $(
                let _ = set.$rtype($id, $source);
            )*
            set.set_block_write(false);
            let _ = set.write();
        }
    };
}

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Settings {

    #[serde(skip_serializing, skip_deserializing)]
    pub block_write: bool,

    pub bool: HashMap<String, bool>,
    pub i32: HashMap<String, i32>,
    pub f32: HashMap<String, f32>,
    pub string: HashMap<String, String>,
    pub raw_data: HashMap<String, Vec<u8>>,
    pub vec_string: HashMap<String, Vec<String>>
}

//-------------------------------------------------------------------------------//
//                         Settings implementation
//-------------------------------------------------------------------------------//

impl Settings {

    pub fn init(as_new: bool) -> Result<Self> {
        let mut settings = if !as_new {
            Settings::read().unwrap_or_default()
        } else {
            Settings::default()
        };

        settings.set_block_write(true);

        settings.initialize_string(MYMOD_BASE_PATH, "");
        settings.initialize_string(SECONDARY_PATH, "");

        let supported_games = SupportedGames::default();
        for game in &supported_games.games() {
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
        //settings.initialize_string("update_channel", STABLE);
        settings.initialize_i32("autosave_amount", 10);
        settings.initialize_i32("autosave_interval", 5);

        /*
        let font = QApplication::font();
        let font_name = font.family().to_std_string();
        let font_size = font.point_size();
        settings.initialize_string("font_name", &font_name);
        settings.initialize_i32("font_size", font_size);
        settings.initialize_string("original_font_name", &font_name);
        settings.initialize_i32("original_font_size", font_size);
    */
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
        settings.initialize_bool("enable_renderer", true);

        // Diagnostics Settings
        settings.initialize_bool("diagnostics_trigger_on_open", true);
        settings.initialize_bool("diagnostics_trigger_on_table_edit", true);

        settings.initialize_string("ai_openai_api_key", "");
        settings.initialize_string("deepl_api_key", "");

        settings.initialize_vec_string("recentFileList", &[]);

        // Colours.
    /*    let q_settings = qt_core::QSettings::new();
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
        q_settings.sync();*/

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

        settings.write()?;

        Ok(settings)
    }

    pub fn read() -> Result<Self> {
        let mut data = vec![];
        let mut file = BufReader::new(File::open(config_path()?.join(SETTINGS_FILE_NAME))?);
        file.read_to_end(&mut data)?;

        serde_json::from_slice(&data).map_err(From::from)
    }

    /// Writes the settings to disk. Does nothing if the block write flag is set.
    pub fn write(&self) -> Result<()> {
        if self.block_write {
            return Ok(());
        }

        let mut file = BufWriter::new(File::create(config_path()?.join(SETTINGS_FILE_NAME))?);
        file.write_all(serde_json::to_string_pretty(self)?.as_bytes()).map_err(From::from)
    }

    /// Disables save to disk when storing a setting. For batch operations.
    pub fn set_block_write(&mut self, status: bool) {
        self.block_write = status;
    }

    pub fn bool(&self, setting: &str) -> bool {
        self.bool.get(setting).copied().unwrap_or_default()
    }

    pub fn i32(&self, setting: &str) -> i32 {
        self.i32.get(setting).copied().unwrap_or_default()
    }

    pub fn f32(&self, setting: &str) -> f32 {
        self.f32.get(setting).copied().unwrap_or_default()
    }

    pub fn string(&self, setting: &str) -> String {
        self.string.get(setting).map(|x| x.to_owned()).unwrap_or_default()
    }

    pub fn path_buf(&self, setting: &str) -> PathBuf {
        self.string.get(setting).map(PathBuf::from).unwrap_or_default()
    }

    pub fn raw_data(&self, setting: &str) -> Vec<u8> {
        self.raw_data.get(setting).map(|x| x.to_vec()).unwrap_or_default()
    }

    pub fn vec_string(&self, setting: &str) -> Vec<String> {
        self.vec_string.get(setting).map(|x| x.to_vec()).unwrap_or_default()
    }

    pub fn set_bool(&mut self, setting: &str, value: bool) -> Result<()> {
        self.bool.insert(setting.to_owned(), value);
        self.write()
    }

    pub fn set_i32(&mut self, setting: &str, value: i32) -> Result<()> {
        self.i32.insert(setting.to_owned(), value);
        self.write()
    }

    pub fn set_f32(&mut self, setting: &str, value: f32) -> Result<()> {
        self.f32.insert(setting.to_owned(), value);
        self.write()
    }

    pub fn set_string(&mut self, setting: &str, value: &str) -> Result<()> {
        self.string.insert(setting.to_owned(), value.to_owned());
        self.write()
    }

    pub fn set_path_buf(&mut self, setting: &str, value: &Path) -> Result<()> {
        self.string.insert(setting.to_owned(), value.to_string_lossy().to_string());
        self.write()
    }

    pub fn set_raw_data(&mut self, setting: &str, value: &[u8]) -> Result<()> {
        self.raw_data.insert(setting.to_owned(), value.to_vec());
        self.write()
    }

    pub fn set_vec_string(&mut self, setting: &str, value: &[String]) -> Result<()> {
        self.vec_string.insert(setting.to_owned(), value.to_vec());
        self.write()
    }

    pub fn initialize_bool(&mut self, setting: &str, value: bool) {
        if !self.bool.contains_key(setting) {
            self.bool.insert(setting.to_owned(), value);
        }
    }

    pub fn initialize_i32(&mut self, setting: &str, value: i32) {
        if !self.i32.contains_key(setting) {
            self.i32.insert(setting.to_owned(), value);
        }
    }

    pub fn initialize_f32(&mut self, setting: &str, value: f32) {
        if !self.f32.contains_key(setting) {
            self.f32.insert(setting.to_owned(), value);
        }
    }

    pub fn initialize_string(&mut self, setting: &str, value: &str) {
        if !self.string.contains_key(setting) {
            self.string.insert(setting.to_owned(), value.to_owned());
        }
    }

    pub fn initialize_path_buf(&mut self, setting: &str, value: &Path) {
        if !self.string.contains_key(setting) {
            self.string.insert(setting.to_owned(), value.to_string_lossy().to_string());
        }
    }

    pub fn initialize_raw_data(&mut self, setting: &str, value: &[u8]) {
        if !self.raw_data.contains_key(setting) {
            self.raw_data.insert(setting.to_owned(), value.to_vec());
        }
    }

    pub fn initialize_vec_string(&mut self, setting: &str, value: &[String]) {
        if !self.vec_string.contains_key(setting) {
            self.vec_string.insert(setting.to_owned(), value.to_vec());
        }
    }

    pub fn optimizer_options(&self) -> OptimizerOptions {
        let mut options = OptimizerOptions::default();

        options.set_pack_remove_itm_files(self.bool("pack_remove_itm_files"));
        options.set_db_import_datacores_into_twad_key_deletes(self.bool("db_import_datacores_into_twad_key_deletes"));
        options.set_db_optimize_datacored_tables(self.bool("db_optimize_datacored_tables"));
        options.set_table_remove_duplicated_entries(self.bool("table_remove_duplicated_entries"));
        options.set_table_remove_itm_entries(self.bool("table_remove_itm_entries"));
        options.set_table_remove_itnr_entries(self.bool("table_remove_itnr_entries"));
        options.set_table_remove_empty_file(self.bool("table_remove_empty_file"));
        options.set_text_remove_unused_xml_map_folders(self.bool("text_remove_unused_xml_map_folders"));
        options.set_text_remove_unused_xml_prefab_folder(self.bool("text_remove_unused_xml_prefab_folder"));
        options.set_text_remove_agf_files(self.bool("text_remove_agf_files"));
        options.set_text_remove_model_statistics_files(self.bool("text_remove_model_statistics_files"));
        options.set_pts_remove_unused_art_sets(self.bool("pts_remove_unused_art_sets"));
        options.set_pts_remove_unused_variants(self.bool("pts_remove_unused_variants"));
        options.set_pts_remove_empty_masks(self.bool("pts_remove_empty_masks"));
        options.set_pts_remove_empty_file(self.bool("pts_remove_empty_file"));

        options
    }

    /// This function returns the path where the db files from the assembly kit are stored.
    pub fn assembly_kit_path(&self, game: &GameInfo) -> Result<PathBuf> {
        let version = *game.raw_db_version();
        match version {

            // Post-Shogun 2 games.
            2 | 1 => {
                let mut base_path = self.path_buf(&format!("{}_assembly_kit", game.key()));
                base_path.push("raw_data/db");
                Ok(base_path)
            }

            0 => {
                let base_path = old_ak_files_path()?.join(game.key());
                Ok(base_path)
            },

            // Shogun 2/Older games
            _ => Err(RLibError::AssemblyKitUnsupportedVersion(version).into())
        }
    }
}

//-------------------------------------------------------------------------------//
//                             Extra Helpers
//-------------------------------------------------------------------------------//

/// This function returns the current config path, or an error if said path is not available.
///
/// Note: On `Debug´ mode this project is the project from where you execute one of RPFM's programs, which should be the root of the repo.
pub fn config_path() -> Result<PathBuf> {

    // On debug builds we use the local folder as the config folder.
    if cfg!(debug_assertions) {
        std::env::current_dir().map_err(From::from)
    } else {
        match ProjectDirs::from(ORG_DOMAIN, ORG_NAME, APP_NAME) {
            Some(proj_dirs) => Ok(proj_dirs.config_dir().to_path_buf()),
            None => Err(anyhow!("Failed to get the config path."))
        }
    }
}

/// This function returns the path where crash logs are stored.
pub fn error_path() -> Result<PathBuf> {
    Ok(config_path()?.join("error"))
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

    /*
    #[cfg(feature = "support_model_renderer")] {
        let assets_path = format!("{}/assets/", rpfm_ui_common::ASSETS_PATH.to_string_lossy());
        if !PathBuf::from(&assets_path).is_dir() {
            DirBuilder::new().recursive(true).create(&assets_path)?;
        }

        unsafe {crate::ffi::set_asset_folder(&assets_path); }

        let log_path = config_path.to_string_lossy();
        unsafe {crate::ffi::set_log_folder(&log_path); }
    }*/

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

pub fn translations_local_path() -> Result<PathBuf> {
    Ok(config_path()?.join(TRANSLATIONS_LOCAL_FOLDER))
}

pub fn translations_remote_path() -> Result<PathBuf> {
    Ok(config_path()?.join(TRANSLATIONS_REMOTE_FOLDER))
}

pub fn clear_config_path(path: &Path) -> Result<()> {
    if path.exists() && path.is_dir() && path.starts_with(config_path()?) {
        std::fs::remove_dir_all(path)?;
        init_config_path()
    } else {
        Err(anyhow!("Path is not a valid directory to clear or does not exist"))
    }
}
