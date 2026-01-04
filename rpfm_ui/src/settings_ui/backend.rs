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

use rpfm_extensions::optimizer::OptimizerOptions;

use rpfm_ipc::messages::{Command, Response};

use rpfm_lib::schema::{Definition, Schema};

use crate::app_ui::AppUI;
use crate::communications::{send_ipc_command, send_ipc_command_result};

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

    // These settings need to use QSettings because they're read in the C++ side.
    settings_set_raw_data("originalGeometry", &app_ui.main_window().save_geometry().as_slice().iter().map(|x| *x as u8).collect::<Vec<_>>());
    settings_set_raw_data("originalWindowState", &app_ui.main_window().save_state_0a().as_slice().iter().map(|x| *x as u8).collect::<Vec<_>>());

    // This one needs to be checked here, due to how the ui works.
    app_ui.menu_bar_debug().menu_action().set_visible(settings_bool("enable_debug_menu"));
}

/// Get a boolean setting from the server.
pub fn settings_bool(key: &str) -> bool {
    send_ipc_command(Command::SettingsGetBool(key.to_string()), response_extractor!(Response::Bool))
}

/// Get an i32 setting from the server.
pub fn settings_i32(key: &str) -> i32 {
    send_ipc_command(Command::SettingsGetI32(key.to_string()), response_extractor!(Response::I32))
}

/// Get an f32 setting from the server.
pub fn settings_f32(key: &str) -> f32 {
    send_ipc_command(Command::SettingsGetF32(key.to_string()), response_extractor!(Response::F32))
}

/// Get a string setting from the server.
pub fn settings_string(key: &str) -> String {
    send_ipc_command(Command::SettingsGetString(key.to_string()), response_extractor!(Response::String))
}

/// Get a PathBuf setting from the server.
pub fn settings_path_buf(key: &str) -> PathBuf {
    send_ipc_command(Command::SettingsGetPathBuf(key.to_string()), response_extractor!(Response::PathBuf))
}

/// Get a Vec<String> setting from the server.
pub fn settings_vec_string(key: &str) -> Vec<String> {
    send_ipc_command(Command::SettingsGetVecString(key.to_string()), response_extractor!(Response::VecString))
}

/// Get a Vec<u8> setting from the server.
pub fn settings_raw_data(key: &str) -> Vec<u8> {
    send_ipc_command(Command::SettingsGetVecRaw(key.to_string()), response_extractor!(Response::VecU8))
}

/// Set a boolean setting on the server.
pub fn settings_set_bool(key: &str, value: bool) {
    send_ipc_command(Command::SettingsSetBool(key.to_string(), value), response_extractor!())
}

/// Set an i32 setting on the server.
pub fn settings_set_i32(key: &str, value: i32) {
    send_ipc_command(Command::SettingsSetI32(key.to_string(), value), response_extractor!())
}

/// Set an f32 setting on the server.
pub fn settings_set_f32(key: &str, value: f32) {
    send_ipc_command(Command::SettingsSetF32(key.to_string(), value), response_extractor!())
}

/// Set a string setting on the server.
pub fn settings_set_string(key: &str, value: &str) {
    send_ipc_command(Command::SettingsSetString(key.to_string(), value.to_string()), response_extractor!())
}

/// Set a PathBuf setting on the server.
pub fn settings_set_path_buf(key: &str, value: &PathBuf) {
    send_ipc_command(Command::SettingsSetPathBuf(key.to_string(), value.clone()), response_extractor!())
}

/// Set a Vec<String> setting on the server.
pub fn settings_set_vec_string(key: &str, value: &[String]) {
    send_ipc_command(Command::SettingsSetVecString(key.to_string(), value.to_vec()), response_extractor!())
}

/// Set a Vec<u8> setting on the server.
pub fn settings_set_raw_data(key: &str, value: &[u8]) {
    send_ipc_command(Command::SettingsSetVecRaw(key.to_string(), value.to_vec()), response_extractor!())
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
