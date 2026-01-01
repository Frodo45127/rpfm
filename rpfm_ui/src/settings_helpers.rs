//---------------------------------------------------------------------------//
// Copyright (c) 2017-2024 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Helper functions for settings access via IPC.
//!
//! These functions provide a clean interface for the UI to access settings
//! from the server.

use std::path::{Path, PathBuf};

use anyhow::{Result, anyhow};
use rpfm_extensions::optimizer::OptimizerOptions;
use rpfm_ipc::messages::{Command, Response};

use crate::{CENTRAL_COMMAND, communications::CentralCommand};

/// Get a boolean setting from the server.
pub fn settings_bool(key: &str) -> bool {
    let receiver = CENTRAL_COMMAND.read().unwrap().send(Command::SettingsGetBool(key.to_string()));
    match CentralCommand::recv(&receiver) {
        Response::SettingsBool(value) => value,
        _ => {
            eprintln!("Error getting bool setting '{}', returning default false", key);
            false
        }
    }
}

/// Get an i32 setting from the server.
pub fn settings_i32(key: &str) -> i32 {
    let receiver = CENTRAL_COMMAND.read().unwrap().send(Command::SettingsGetI32(key.to_string()));
    match CentralCommand::recv(&receiver) {
        Response::SettingsI32(value) => value,
        _ => {
            eprintln!("Error getting i32 setting '{}', returning default 0", key);
            0
        }
    }
}

/// Get an f32 setting from the server.
pub fn settings_f32(key: &str) -> f32 {
    let receiver = CENTRAL_COMMAND.read().unwrap().send(Command::SettingsGetF32(key.to_string()));
    match CentralCommand::recv(&receiver) {
        Response::SettingsF32(value) => value,
        _ => {
            eprintln!("Error getting f32 setting '{}', returning default 0.0", key);
            0.0
        }
    }
}

/// Get a string setting from the server.
pub fn settings_string(key: &str) -> String {
    let receiver = CENTRAL_COMMAND.read().unwrap().send(Command::SettingsGetString(key.to_string()));
    match CentralCommand::recv(&receiver) {
        Response::SettingsString(value) => value,
        _ => {
            eprintln!("Error getting string setting '{}', returning default empty string", key);
            String::new()
        }
    }
}

/// Get a PathBuf setting from the server.
pub fn settings_path_buf(key: &str) -> PathBuf {
    let receiver = CENTRAL_COMMAND.read().unwrap().send(Command::SettingsGetPathBuf(key.to_string()));
    match CentralCommand::recv(&receiver) {
        Response::SettingsPathBuf(value) => value,
        _ => {
            eprintln!("Error getting PathBuf setting '{}', returning default empty PathBuf", key);
            PathBuf::new()
        }
    }
}

/// Get a Vec<String> setting from the server.
pub fn settings_vec_string(key: &str) -> Vec<String> {
    let receiver = CENTRAL_COMMAND.read().unwrap().send(Command::SettingsGetVecString(key.to_string()));
    match CentralCommand::recv(&receiver) {
        Response::SettingsVecString(value) => value,
        _ => {
            eprintln!("Error getting Vec<String> setting '{}', returning default empty vec", key);
            Vec::new()
        }
    }
}

/// Get a Vec<u8> setting from the server.
pub fn settings_raw_data(key: &str) -> Vec<u8> {
    let receiver = CENTRAL_COMMAND.read().unwrap().send(Command::SettingsGetVecRaw(key.to_string()));
    match CentralCommand::recv(&receiver) {
        Response::SettingsVecRaw(value) => value,
        _ => {
            eprintln!("Error getting Vec<u8> setting '{}', returning default empty vec", key);
            Vec::new()
        }
    }
}

/// Set a boolean setting on the server.
pub fn settings_set_bool(key: &str, value: bool) -> bool {
    let receiver = CENTRAL_COMMAND.read().unwrap().send(Command::SettingsSetBool(key.to_string(), value));
    matches!(CentralCommand::recv(&receiver), Response::Success)
}

/// Set an i32 setting on the server.
pub fn settings_set_i32(key: &str, value: i32) -> bool {
    let receiver = CENTRAL_COMMAND.read().unwrap().send(Command::SettingsSetI32(key.to_string(), value));
    matches!(CentralCommand::recv(&receiver), Response::Success)
}

/// Set an f32 setting on the server.
pub fn settings_set_f32(key: &str, value: f32) -> bool {
    let receiver = CENTRAL_COMMAND.read().unwrap().send(Command::SettingsSetF32(key.to_string(), value));
    matches!(CentralCommand::recv(&receiver), Response::Success)
}

/// Set a string setting on the server.
pub fn settings_set_string(key: &str, value: &str) -> bool {
    let receiver = CENTRAL_COMMAND.read().unwrap().send(Command::SettingsSetString(key.to_string(), value.to_string()));
    matches!(CentralCommand::recv(&receiver), Response::Success)
}

/// Set a PathBuf setting on the server.
pub fn settings_set_path_buf(key: &str, value: &PathBuf) -> bool {
    let receiver = CENTRAL_COMMAND.read().unwrap().send(Command::SettingsSetPathBuf(key.to_string(), value.clone()));
    matches!(CentralCommand::recv(&receiver), Response::Success)
}

/// Set a Vec<String> setting on the server.
pub fn settings_set_vec_string(key: &str, value: &[String]) -> bool {
    let receiver = CENTRAL_COMMAND.read().unwrap().send(Command::SettingsSetVecString(key.to_string(), value.to_vec()));
    matches!(CentralCommand::recv(&receiver), Response::Success)
}

/// Set a Vec<u8> setting on the server.
pub fn settings_set_raw_data(key: &str, value: &[u8]) -> bool {
    let receiver = CENTRAL_COMMAND.read().unwrap().send(Command::SettingsSetVecRaw(key.to_string(), value.to_vec()));
    matches!(CentralCommand::recv(&receiver), Response::Success)
}

pub fn config_path() -> Result<PathBuf> {
    let receiver = CENTRAL_COMMAND.read().unwrap().send(Command::ConfigPath);
    match CentralCommand::recv(&receiver) {
        Response::PathBuf(value) => Ok(value),
        _ => Err(anyhow!("Error getting config path")),
    }
}

pub fn assembly_kit_path() -> Result<PathBuf> {
    let receiver = CENTRAL_COMMAND.read().unwrap().send(Command::AssemblyKitPath);
    match CentralCommand::recv(&receiver) {
        Response::PathBuf(value) => Ok(value),
        _ => Err(anyhow!("Error getting assembly kit path")),
    }
}

pub fn backup_autosave_path() -> Result<PathBuf> {
    let receiver = CENTRAL_COMMAND.read().unwrap().send(Command::BackupAutosavePath);
    match CentralCommand::recv(&receiver) {
        Response::PathBuf(value) => Ok(value),
        _ => Err(anyhow!("Error getting backup autosave path")),
    }
}

pub fn old_ak_data_path() -> Result<PathBuf> {
    let receiver = CENTRAL_COMMAND.read().unwrap().send(Command::OldAkDataPath);
    match CentralCommand::recv(&receiver) {
        Response::PathBuf(value) => Ok(value),
        _ => Err(anyhow!("Error getting old ak data path")),
    }
}

pub fn schemas_path() -> Result<PathBuf> {
    let receiver = CENTRAL_COMMAND.read().unwrap().send(Command::SchemasPath);
    match CentralCommand::recv(&receiver) {
        Response::PathBuf(value) => Ok(value),
        _ => Err(anyhow!("Error getting schemas path")),
    }
}

pub fn table_profiles_path() -> Result<PathBuf> {
    let receiver = CENTRAL_COMMAND.read().unwrap().send(Command::TableProfilesPath);
    match CentralCommand::recv(&receiver) {
        Response::PathBuf(value) => Ok(value),
        _ => Err(anyhow!("Error getting table profiles path")),
    }
}

pub fn translations_local_path() -> Result<PathBuf> {
    let receiver = CENTRAL_COMMAND.read().unwrap().send(Command::TranslationsLocalPath);
    match CentralCommand::recv(&receiver) {
        Response::PathBuf(value) => Ok(value),
        _ => Err(anyhow!("Error getting translations local path")),
    }
}

pub fn dependencies_cache_path() -> Result<PathBuf> {
    let receiver = CENTRAL_COMMAND.read().unwrap().send(Command::DependenciesCachePath);
    match CentralCommand::recv(&receiver) {
        Response::PathBuf(value) => Ok(value),
        _ => Err(anyhow!("Error getting dependencies cache path")),
    }
}

pub fn settings_clear_path(path: &Path) -> Result<()> {
    let receiver = CENTRAL_COMMAND.read().unwrap().send(Command::SettingsClearPath(path.to_path_buf()));
    match CentralCommand::recv(&receiver) {
        Response::Success => Ok(()),
        _ => Err(anyhow!("Error clearing settings path")),
    }
}

pub fn optimizer_options() -> OptimizerOptions {
    let receiver = CENTRAL_COMMAND.read().unwrap().send(Command::OptimizerOptions);
    match CentralCommand::recv(&receiver) {
        Response::OptimizerOptions(value) => value,
        _ => panic!("Error getting optimizer options"),
    }
}
