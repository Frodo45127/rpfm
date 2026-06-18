//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Module with all the code to deal with the settings used to configure this program.

use qt_core::QBox;
use qt_core::QSettings;
use qt_core::QString;
use qt_core::QVariant;

use anyhow::Result;
use directories::ProjectDirs;

use std::{collections::HashMap, path::{Path, PathBuf}};
use std::sync::{LazyLock, RwLock};

use rpfm_extensions::optimizer::OptimizerOptions;

use rpfm_ipc::messages::{Command, Response};
use rpfm_ipc::settings_keys::*;

use rpfm_lib::schema::{Definition, Schema};

use crate::app_ui::AppUI;
use crate::communications::{send_ipc_command, send_ipc_command_async, send_ipc_command_result, send_ipc_command_result_async};

//-------------------------------------------------------------------------------//
//                          Settings cache
//-------------------------------------------------------------------------------//

/// Filename of the on-disk settings JSON file the server writes.
/// Mirrors `rpfm_server::settings::SETTINGS_FILE_NAME`.
const SETTINGS_FILE_NAME: &str = "settings.json";

/// `ProjectDirs` triple shared with the server so both processes resolve to
/// the same config directory.
const ORG_DOMAIN: &str = "com";
const ORG_NAME: &str = "FrodoWazEre";
const APP_NAME: &str = "rpfm";

/// Name of the custom-config-folder redirect file. Mirrors
/// `rpfm_server::settings::CONFIG_REDIRECT_FILE_NAME`.
const CONFIG_REDIRECT_FILE_NAME: &str = "config_folder.txt";

/// Settings cache for the UI.
static SETTINGS_CACHE: LazyLock<RwLock<Option<SettingsSnapshot>>> = LazyLock::new(|| RwLock::new(None));

/// Returns the snapshot from the cache, or [`SettingsSnapshot::default`] when
/// the cache hasn't been seeded yet.
fn with_cache<T>(f: impl FnOnce(&SettingsSnapshot) -> T) -> T {

    // Fast path: the cache is already seeded, so a shared read lock is enough.
    if let Some(snapshot) = SETTINGS_CACHE.read().unwrap().as_ref() {
        return f(snapshot);
    }

    // Seed an empty default once, then read it back.
    let mut write = SETTINGS_CACHE.write().unwrap();
    if write.is_none() {
        *write = Some(SettingsSnapshot::default());
    }
    f(write.as_ref().unwrap())
}

/// Resolve RPFM's default config directory, ignoring any custom-folder redirect.
fn default_config_dir() -> Option<PathBuf> {
    if cfg!(debug_assertions) {
        std::env::current_dir().ok()
    } else {
        Some(ProjectDirs::from(ORG_DOMAIN, ORG_NAME, APP_NAME)?.config_dir().to_path_buf())
    }
}

/// Resolve the active config directory, mirroring `rpfm_server::settings::config_path`
/// so both processes hit the same folder before the server is even up.
fn config_dir() -> Option<PathBuf> {
    let default = default_config_dir()?;
    let redirect_file = default.join(CONFIG_REDIRECT_FILE_NAME);
    if let Ok(raw) = std::fs::read_to_string(&redirect_file) {
        let trimmed = raw.trim();
        if !trimmed.is_empty() {
            return Some(PathBuf::from(trimmed));
        }
    }
    Some(default)
}

/// Resolve the on-disk settings file path so both processes hit the same JSON.
fn settings_file_path() -> Option<PathBuf> {
    Some(config_dir()?.join(SETTINGS_FILE_NAME))
}

/// Populate the settings cache from the on-disk JSON file the server persists.
pub fn load_settings_cache_from_disk() {
    let Some(path) = settings_file_path() else { return; };
    let Ok(data) = std::fs::read(&path) else { return; };
    let Ok(snapshot) = serde_json::from_slice::<SettingsSnapshot>(&data) else { return; };
    *SETTINGS_CACHE.write().unwrap() = Some(snapshot);
}

/// Replace the cached snapshot with the one from the server.
pub fn load_settings_cache_from_server() {
    let snapshot = send_ipc_command_async(Command::SettingsGetAll, response_extractor!(Response::SettingsAll));
    *SETTINGS_CACHE.write().unwrap() = Some(snapshot);
}

//-------------------------------------------------------------------------------//
//                         Setting-related functions
//-------------------------------------------------------------------------------//

pub unsafe fn set_setting_if_new_string(q_settings: &QBox<QSettings>, setting: &str, value: &str) {
    if !q_settings.value_1a(&QString::from_std_str(setting)).is_valid() {
        q_settings.set_value(&QString::from_std_str(setting), &QVariant::from_q_string(&QString::from_std_str(value)));
    }
}

pub unsafe fn init_app_exclusive_settings(app_ui: &AppUI) {

    // Colours.
    let q_settings = qt_core::QSettings::new();
    set_setting_if_new_string(&q_settings, COLOUR_LIGHT_TABLE_ADDED, "#87ca00");
    set_setting_if_new_string(&q_settings, COLOUR_LIGHT_TABLE_MODIFIED, "#e67e22");
    set_setting_if_new_string(&q_settings, COLOUR_LIGHT_DIAGNOSTIC_ERROR, "#ff0000");
    set_setting_if_new_string(&q_settings, COLOUR_LIGHT_DIAGNOSTIC_WARNING, "#bebe00");
    set_setting_if_new_string(&q_settings, COLOUR_LIGHT_DIAGNOSTIC_INFO, "#55aaff");
    set_setting_if_new_string(&q_settings, COLOUR_DARK_TABLE_ADDED, "#00ff00");
    set_setting_if_new_string(&q_settings, COLOUR_DARK_TABLE_MODIFIED, "#e67e22");
    set_setting_if_new_string(&q_settings, COLOUR_DARK_DIAGNOSTIC_ERROR, "#ff0000");
    set_setting_if_new_string(&q_settings, COLOUR_DARK_DIAGNOSTIC_WARNING, "#cece67");
    set_setting_if_new_string(&q_settings, COLOUR_DARK_DIAGNOSTIC_INFO, "#55aaff");
    q_settings.sync();

    // These settings need to use QSettings because they're read in the C++ side.
    let _ = settings_set_raw_data(ORIGINAL_GEOMETRY, &app_ui.main_window().save_geometry().as_slice().iter().map(|x| *x as u8).collect::<Vec<_>>());
    let _ = settings_set_raw_data(ORIGINAL_WINDOW_STATE, &app_ui.main_window().save_state_0a().as_slice().iter().map(|x| *x as u8).collect::<Vec<_>>());

    // This one needs to be checked here, due to how the ui works.
    app_ui.menu_bar_debug().menu_action().set_visible(settings_bool(ENABLE_DEBUG_MENU));
}

/// Get all settings from the cache (fetching from server on first call).
pub fn settings_get_all() -> SettingsSnapshot {
    with_cache(|s| s.clone())
}

/// Get a boolean setting from the cache.
pub fn settings_bool(key: &str) -> bool {
    with_cache(|s| s.bool.get(key).copied().unwrap_or_default())
}

/// Get an i32 setting from the cache.
pub fn settings_i32(key: &str) -> i32 {
    with_cache(|s| s.i32.get(key).copied().unwrap_or_default())
}

/// Get an f32 setting from the cache.
#[allow(dead_code)]
pub fn settings_f32(key: &str) -> f32 {
    with_cache(|s| s.f32.get(key).copied().unwrap_or_default())
}

/// Get a string setting from the cache.
pub fn settings_string(key: &str) -> String {
    with_cache(|s| s.string.get(key).cloned().unwrap_or_default())
}

/// Get a PathBuf setting from the cache.
pub fn settings_path_buf(key: &str) -> PathBuf {
    with_cache(|s| PathBuf::from(s.string.get(key).cloned().unwrap_or_default()))
}

/// Get a Vec<String> setting from the cache.
pub fn settings_vec_string(key: &str) -> Vec<String> {
    with_cache(|s| s.vec_string.get(key).cloned().unwrap_or_default())
}

/// Get a Vec<u8> setting from the cache.
pub fn settings_raw_data(key: &str) -> Vec<u8> {
    with_cache(|s| s.raw_data.get(key).cloned().unwrap_or_default())
}

/// Set a boolean setting on the server and update the local cache.
pub fn settings_set_bool(key: &str, value: bool) -> Result<()> {
    send_ipc_command_result(Command::SettingsSetBool(key.to_string(), value), response_extractor!())?;
    if let Some(ref mut s) = *SETTINGS_CACHE.write().unwrap() {
        s.bool.insert(key.to_owned(), value);
    }
    Ok(())
}

/// Set an i32 setting on the server and update the local cache.
pub fn settings_set_i32(key: &str, value: i32) -> Result<()> {
    send_ipc_command_result(Command::SettingsSetI32(key.to_string(), value), response_extractor!())?;
    if let Some(ref mut s) = *SETTINGS_CACHE.write().unwrap() {
        s.i32.insert(key.to_owned(), value);
    }
    Ok(())
}

/// Set an f32 setting on the server and update the local cache.
#[allow(dead_code)]
pub fn settings_set_f32(key: &str, value: f32) -> Result<()> {
    send_ipc_command_result(Command::SettingsSetF32(key.to_string(), value), response_extractor!())?;
    if let Some(ref mut s) = *SETTINGS_CACHE.write().unwrap() {
        s.f32.insert(key.to_owned(), value);
    }
    Ok(())
}

/// Set a string setting on the server and update the local cache.
pub fn settings_set_string(key: &str, value: &str) -> Result<()> {
    send_ipc_command_result(Command::SettingsSetString(key.to_string(), value.to_string()), response_extractor!())?;
    if let Some(ref mut s) = *SETTINGS_CACHE.write().unwrap() {
        s.string.insert(key.to_owned(), value.to_owned());
    }
    Ok(())
}

/// Set a PathBuf setting on the server and update the local cache.
#[allow(dead_code)]
pub fn settings_set_path_buf(key: &str, value: &Path) -> Result<()> {
    send_ipc_command_result(Command::SettingsSetPathBuf(key.to_string(), value.to_path_buf()), response_extractor!())?;
    if let Some(ref mut s) = *SETTINGS_CACHE.write().unwrap() {
        s.string.insert(key.to_owned(), value.to_string_lossy().to_string());
    }
    Ok(())
}

/// Set a Vec<String> setting on the server and update the local cache.
pub fn settings_set_vec_string(key: &str, value: &[String]) -> Result<()> {
    send_ipc_command_result(Command::SettingsSetVecString(key.to_string(), value.to_vec()), response_extractor!())?;
    if let Some(ref mut s) = *SETTINGS_CACHE.write().unwrap() {
        s.vec_string.insert(key.to_owned(), value.to_vec());
    }
    Ok(())
}

/// Set a Vec<u8> setting on the server and update the local cache.
pub fn settings_set_raw_data(key: &str, value: &[u8]) -> Result<()> {
    send_ipc_command_result_async(Command::SettingsSetVecRaw(key.to_string(), value.to_vec()), response_extractor!())?;
    if let Some(ref mut s) = *SETTINGS_CACHE.write().unwrap() {
        s.raw_data.insert(key.to_owned(), value.to_vec());
    }
    Ok(())
}

pub fn config_path() -> Result<PathBuf> {
    send_ipc_command_result(Command::ConfigPath, response_extractor!(Response::PathBuf))
}

pub fn assembly_kit_path() -> Result<PathBuf> {
    send_ipc_command_result(Command::AssemblyKitPath, response_extractor!(Response::PathBuf))
}

pub fn backup_autosave_path() -> Result<PathBuf> {
    send_ipc_command_result(Command::BackupAutosavePath, response_extractor!(Response::PathBuf))
}

pub fn old_ak_data_path() -> Result<PathBuf> {
    send_ipc_command_result(Command::OldAkDataPath, response_extractor!(Response::PathBuf))
}

pub fn schemas_path() -> Result<PathBuf> {
    send_ipc_command_result(Command::SchemasPath, response_extractor!(Response::PathBuf))
}

pub fn table_profiles_path() -> Result<PathBuf> {
    send_ipc_command_result(Command::TableProfilesPath, response_extractor!(Response::PathBuf))
}

pub fn translations_local_path() -> Result<PathBuf> {
    send_ipc_command_result(Command::TranslationsLocalPath, response_extractor!(Response::PathBuf))
}

pub fn dependencies_cache_path() -> Result<PathBuf> {
    send_ipc_command_result(Command::DependenciesCachePath, response_extractor!(Response::PathBuf))
}

pub fn settings_clear_path(path: &Path) -> Result<()> {
    send_ipc_command_result(Command::SettingsClearPath(path.to_path_buf()), response_extractor!())
}

/// Fetch the user-configured custom config folder. An empty path means RPFM uses the default one.
pub fn custom_config_path() -> Result<PathBuf> {
    send_ipc_command_result(Command::CustomConfigPath, response_extractor!(Response::PathBuf))
}

/// Set the custom config folder, or clear it when given an empty path. Takes effect on restart.
pub fn set_custom_config_path(path: &Path) -> Result<()> {
    send_ipc_command_result(Command::SetCustomConfigPath(path.to_path_buf()), response_extractor!())
}

pub fn optimizer_options() -> OptimizerOptions {
    send_ipc_command_result(Command::OptimizerOptions, response_extractor!(Response::OptimizerOptions)).unwrap()
}

pub fn is_schema_loaded() -> bool {
    send_ipc_command_result(Command::IsSchemaLoaded, response_extractor!(Response::Bool)).unwrap()
}

pub fn definitions_by_table_name(name: &str) -> Result<Vec<Definition>> {
    send_ipc_command_result(Command::DefinitionsByTableName(name.to_owned()), response_extractor!(Response::VecDefinition))
}

pub fn referencing_columns_for_table(name: &str, definition: &Definition) -> Result<HashMap<String, HashMap<String, Vec<String>>>> {
    send_ipc_command_result(Command::ReferencingColumnsForDefinition(name.to_owned(), definition.clone()), response_extractor!(Response::HashMapStringHashMapStringVecString))
}

pub fn schema() -> Result<Schema> {
    send_ipc_command_result(Command::Schema, response_extractor!(Response::Schema))
}

pub fn definition_by_table_name_and_version(name: &str, version: i32) -> Result<Definition> {
    send_ipc_command_result(Command::DefinitionByTableNameAndVersion(name.to_owned(), version), response_extractor!(Response::Definition))
}

pub fn delete_definition(name: &str, version: i32) {
    send_ipc_command(Command::DeleteDefinition(name.to_owned(), version), response_extractor!())
}
