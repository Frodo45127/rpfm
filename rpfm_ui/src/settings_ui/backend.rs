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

use anyhow::{anyhow, Result};

use std::{collections::HashMap, path::{Path, PathBuf}};
use std::sync::{LazyLock, RwLock};
use std::sync::atomic::{AtomicBool, Ordering};

use rpfm_extensions::optimizer::OptimizerOptions;

use rpfm_ipc::api::schema::{DeleteDefinition, GetRawDefinitions, GetReferencingColumns, GetTablePatches};
use rpfm_ipc::api::session::{GetSessionStatus, ListDependencyTables};
use rpfm_ipc::settings::{self as settings_store, Settings};
use rpfm_ipc::settings_keys::*;

use rpfm_lib::schema::{Definition, DefinitionPatch};

use rpfm_telemetry::warn;

use crate::app_ui::AppUI;
use crate::communications::call_api;
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

/// If a schema is loaded for the selected game.
pub fn is_schema_loaded() -> bool {
    call_api(&GetSessionStatus {}).is_ok_and(|status| status.schema_loaded)
}

/// If the dependencies of the selected game are loaded, optionally requiring the Assembly Kit tables too.
pub fn is_dependency_database_loaded(include_asskit: bool) -> bool {
    call_api(&GetSessionStatus {}).is_ok_and(|status| status.dependencies.vanilla_loaded && (!include_asskit || status.dependencies.assembly_kit_loaded))
}

/// Version new tables of a type should use: the one in the game files, or the newest one for startpos and twad tables.
pub fn dependency_table_version(table_name: &str) -> Result<i32> {
    call_api(&ListDependencyTables {})?.tables.get(table_name).copied()
        .ok_or_else(|| anyhow!("Table {table_name} not found in the game files, or the dependencies are not loaded."))
}

/// All the definitions of a table. Empty if the table is not in the schema.
pub fn definitions_by_table_name(name: &str) -> Result<Vec<Definition>> {
    raw_definitions(name, None)
}

/// The definitions of a table, optionally only the one with a version.
fn raw_definitions(name: &str, version: Option<i32>) -> Result<Vec<Definition>> {
    call_api(&GetRawDefinitions { table_name: name.to_owned(), version })?.definitions
        .into_iter()
        .map(|definition| serde_json::from_value(definition).map_err(From::from))
        .collect()
}

/// The columns of other tables referencing each column of a table definition.
pub fn referencing_columns_for_table(name: &str, definition: &Definition) -> Result<HashMap<String, HashMap<String, Vec<String>>>> {
    let columns = call_api(&GetReferencingColumns { table_name: name.to_owned(), version: Some(*definition.version()) })?.columns;
    Ok(columns.into_iter().map(|(column, tables)| (column, tables.into_iter().collect())).collect())
}

/// The definition of a table with a version.
pub fn definition_by_table_name_and_version(name: &str, version: i32) -> Result<Definition> {
    raw_definitions(name, Some(version))?.into_iter().next()
        .ok_or_else(|| anyhow!("No definition found for table '{name}' with version {version}."))
}

/// The patches of a definition of a table.
pub fn definition_patches(name: &str, version: i32) -> Result<DefinitionPatch> {
    let patches = call_api(&GetTablePatches { table_name: name.to_owned(), version })?.patches;
    Ok(patches.into_iter().map(|(column, patch)| (column, patch.into_iter().collect())).collect())
}

/// Removes a definition of a table from the schema, and saves it.
pub fn delete_definition(name: &str, version: i32) -> Result<()> {
    call_api(&DeleteDefinition { table_name: name.to_owned(), version }).map(|_| ())
}
