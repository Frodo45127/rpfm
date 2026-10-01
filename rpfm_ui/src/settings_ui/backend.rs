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

use std::{collections::HashMap, path::{Path, PathBuf}};
use std::sync::{LazyLock, RwLock};
use std::sync::atomic::{AtomicBool, Ordering};

use rpfm_extensions::optimizer::OptimizerOptions;

use rpfm_ipc::messages::{Command, Response};
use rpfm_ipc::settings::{self as settings_store, Settings};
use rpfm_ipc::settings_keys::*;

use rpfm_lib::schema::{Definition, DefinitionPatch, Schema};

use rpfm_telemetry::warn;

use crate::app_ui::AppUI;
use crate::communications::{send_ipc_command, send_ipc_command_result};
use crate::GAME_SELECTED;

pub use rpfm_ipc::settings::{backup_autosave_path, clear_config_path, config_path, dependencies_cache_path, old_ak_files_path, schemas_path, table_profiles_path, translations_local_path};

//-------------------------------------------------------------------------------//
//                          Settings store
//-------------------------------------------------------------------------------//

/// Settings of the UI. The UI owns the settings file: changes are saved to it, and sent to the server.
static SETTINGS: LazyLock<RwLock<Settings>> = LazyLock::new(|| RwLock::new(Settings::default()));

/// If the settings changed since they were last sent to the server.
static SETTINGS_CHANGED: AtomicBool = AtomicBool::new(true);

/// Loads the settings from disk, initializing the missing ones to their defaults, and saves them back.
pub fn init_settings() {
    if let Err(error) = settings_store::init_config_path() {
        warn!("Failed to initialize the config folder. Error: {error}");
    }

    let settings = Settings::init(false);
    if let Err(error) = settings.write() {
        warn!("Failed to save the settings file, continuing with in-memory settings. Error: {error}");
    }

    replace_settings(settings);
}

/// Replaces all the settings, without saving them to disk.
pub fn replace_settings(settings: Settings) {
    *SETTINGS.write().unwrap() = settings;
    mark_settings_changed();
}

/// Marks the settings as changed, so they're sent to the server before the next message.
pub fn mark_settings_changed() {
    SETTINGS_CHANGED.store(true, Ordering::SeqCst);
}

/// Returns the settings if they changed since the last call, so they can be sent to the server.
pub fn take_changed_settings() -> Option<Settings> {
    SETTINGS_CHANGED.swap(false, Ordering::SeqCst).then(settings_get_all)
}

/// Applies a change to the settings, saving them to disk.
fn set_setting(change: impl FnOnce(&mut Settings) -> settings_store::Result<()>) -> Result<()> {
    let result = change(&mut SETTINGS.write().unwrap());
    mark_settings_changed();
    result.map_err(From::from)
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

/// Get a copy of all the settings.
pub fn settings_get_all() -> Settings {
    SETTINGS.read().unwrap().clone()
}

/// Get a boolean setting.
pub fn settings_bool(key: &str) -> bool {
    SETTINGS.read().unwrap().bool(key)
}

/// Get an i32 setting.
pub fn settings_i32(key: &str) -> i32 {
    SETTINGS.read().unwrap().i32(key)
}

/// Get an f32 setting.
#[allow(dead_code)]
pub fn settings_f32(key: &str) -> f32 {
    SETTINGS.read().unwrap().f32(key)
}

/// Get a string setting.
pub fn settings_string(key: &str) -> String {
    SETTINGS.read().unwrap().string(key)
}

/// Get a PathBuf setting.
pub fn settings_path_buf(key: &str) -> PathBuf {
    SETTINGS.read().unwrap().path_buf(key)
}

/// Get a Vec<String> setting.
pub fn settings_vec_string(key: &str) -> Vec<String> {
    SETTINGS.read().unwrap().vec_string(key)
}

/// Get a Vec<u8> setting.
pub fn settings_raw_data(key: &str) -> Vec<u8> {
    SETTINGS.read().unwrap().raw_data(key)
}

/// Set a boolean setting.
pub fn settings_set_bool(key: &str, value: bool) -> Result<()> {
    set_setting(|settings| settings.set_bool(key, value))
}

/// Set an i32 setting.
pub fn settings_set_i32(key: &str, value: i32) -> Result<()> {
    set_setting(|settings| settings.set_i32(key, value))
}

/// Set an f32 setting.
#[allow(dead_code)]
pub fn settings_set_f32(key: &str, value: f32) -> Result<()> {
    set_setting(|settings| settings.set_f32(key, value))
}

/// Set a string setting.
pub fn settings_set_string(key: &str, value: &str) -> Result<()> {
    set_setting(|settings| settings.set_string(key, value))
}

/// Set a PathBuf setting.
#[allow(dead_code)]
pub fn settings_set_path_buf(key: &str, value: &Path) -> Result<()> {
    set_setting(|settings| settings.set_path_buf(key, value))
}

/// Set a Vec<String> setting.
pub fn settings_set_vec_string(key: &str, value: &[String]) -> Result<()> {
    set_setting(|settings| settings.set_vec_string(key, value))
}

/// Set a Vec<u8> setting.
pub fn settings_set_raw_data(key: &str, value: &[u8]) -> Result<()> {
    set_setting(|settings| settings.set_raw_data(key, value))
}

/// Path of the Assembly Kit db files of the selected game.
pub fn assembly_kit_path() -> Result<PathBuf> {
    SETTINGS.read().unwrap().assembly_kit_path(&GAME_SELECTED.read().unwrap()).map_err(From::from)
}

/// The optimizer options in the settings.
pub fn optimizer_options() -> OptimizerOptions {
    SETTINGS.read().unwrap().optimizer_options()
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

pub fn definition_patches(name: &str, version: i32) -> Result<DefinitionPatch> {
    send_ipc_command_result(Command::DefinitionPatches(name.to_owned(), version), response_extractor!(Response::DefinitionPatch))
}

pub fn delete_definition(name: &str, version: i32) {
    send_ipc_command(Command::DeleteDefinition(name.to_owned(), version), response_extractor!())
}
