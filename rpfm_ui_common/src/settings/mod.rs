//---------------------------------------------------------------------------//
// Copyright (c) 2017-2023 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

use qt_core::QBox;
use qt_core::QByteArray;
use qt_core::QSettings;
use qt_core::QString;
use qt_core::QVariant;

use cpp_core::CppBox;
use cpp_core::Ref;

use anyhow::{anyhow, Result};
use directories::ProjectDirs;

use std::path::{Path, PathBuf};

use crate::QUALIFIER;
use crate::ORGANISATION;
use crate::PROGRAM_NAME;

//-------------------------------------------------------------------------------//
//                         Setting-related functions
//-------------------------------------------------------------------------------//

pub fn settings() -> QBox<QSettings> {
    unsafe { QSettings::from_2_q_string(&QString::from_std_str(&*ORGANISATION.read().unwrap()), &QString::from_std_str(&*PROGRAM_NAME.read().unwrap())) }
}

pub fn setting_path(setting: &str) -> PathBuf {
    unsafe {
        let q_settings = QSettings::from_2_q_string(&QString::from_std_str(&*ORGANISATION.read().unwrap()), &QString::from_std_str(&*PROGRAM_NAME.read().unwrap()));
        PathBuf::from(q_settings.value_1a(&QString::from_std_str(setting)).to_string().to_std_string())
    }
}

pub fn setting_string(setting: &str) -> String {
    unsafe {
        let q_settings = QSettings::from_2_q_string(&QString::from_std_str(&*ORGANISATION.read().unwrap()), &QString::from_std_str(&*PROGRAM_NAME.read().unwrap()));
        q_settings.value_1a(&QString::from_std_str(setting)).to_string().to_std_string()
    }
}

pub fn setting_int(setting: &str) -> i32 {
    unsafe {
        let q_settings = QSettings::from_2_q_string(&QString::from_std_str(&*ORGANISATION.read().unwrap()), &QString::from_std_str(&*PROGRAM_NAME.read().unwrap()));
        q_settings.value_1a(&QString::from_std_str(setting)).to_int_0a()
    }
}

pub fn setting_f32(setting: &str) -> f32 {
    unsafe {
        let q_settings = QSettings::from_2_q_string(&QString::from_std_str(&*ORGANISATION.read().unwrap()), &QString::from_std_str(&*PROGRAM_NAME.read().unwrap()));
        q_settings.value_1a(&QString::from_std_str(setting)).to_float_0a()
    }
}

pub fn setting_bool(setting: &str) -> bool {
    unsafe {
        let q_settings = QSettings::from_2_q_string(&QString::from_std_str(&*ORGANISATION.read().unwrap()), &QString::from_std_str(&*PROGRAM_NAME.read().unwrap()));
        q_settings.value_1a(&QString::from_std_str(setting)).to_bool()
    }
}

pub fn setting_byte_array(setting: &str) -> CppBox<QByteArray> {
    unsafe {
        let q_settings = QSettings::from_2_q_string(&QString::from_std_str(&*ORGANISATION.read().unwrap()), &QString::from_std_str(&*PROGRAM_NAME.read().unwrap()));
        q_settings.value_1a(&QString::from_std_str(setting)).to_byte_array()
    }
}

pub fn setting_variant_from_q_setting(q_settings: &QBox<QSettings>, setting: &str) -> CppBox<QVariant> {
    unsafe {
        q_settings.value_1a(&QString::from_std_str(setting))
    }
}

pub fn setting_path_from_q_setting(q_settings: &QBox<QSettings>, setting: &str) -> PathBuf {
    unsafe {
        PathBuf::from(q_settings.value_1a(&QString::from_std_str(setting)).to_string().to_std_string())
    }
}

pub fn setting_string_from_q_setting(q_settings: &QBox<QSettings>, setting: &str) -> String {
    unsafe {
        q_settings.value_1a(&QString::from_std_str(setting)).to_string().to_std_string()
    }
}

pub fn setting_int_from_q_setting(q_settings: &QBox<QSettings>, setting: &str) -> i32 {
    unsafe {
        q_settings.value_1a(&QString::from_std_str(setting)).to_int_0a()
    }
}

pub fn setting_f32_from_q_setting(q_settings: &QBox<QSettings>, setting: &str) -> f32 {
    unsafe {
        q_settings.value_1a(&QString::from_std_str(setting)).to_float_0a()
    }
}

pub fn setting_bool_from_q_setting(q_settings: &QBox<QSettings>, setting: &str) -> bool {
    unsafe {
        q_settings.value_1a(&QString::from_std_str(setting)).to_bool()
    }
}

pub fn setting_byte_array_from_q_setting(q_settings: &QBox<QSettings>, setting: &str) -> CppBox<QByteArray> {
    unsafe {
        q_settings.value_1a(&QString::from_std_str(setting)).to_byte_array()
    }
}

pub fn set_setting_path(setting: &str, value: &Path) {
    set_setting_string(setting, &value.to_string_lossy())
}

pub fn set_setting_string(setting: &str, value: &str) {
    unsafe {
        let q_settings = QSettings::from_2_q_string(&QString::from_std_str(&*ORGANISATION.read().unwrap()), &QString::from_std_str(&*PROGRAM_NAME.read().unwrap()));
        q_settings.set_value(&QString::from_std_str(setting), &QVariant::from_q_string(&QString::from_std_str(value)));
        q_settings.sync();
    }
}

pub fn set_setting_int(setting: &str, value: i32) {
    unsafe {
        let q_settings = QSettings::from_2_q_string(&QString::from_std_str(&*ORGANISATION.read().unwrap()), &QString::from_std_str(&*PROGRAM_NAME.read().unwrap()));
        q_settings.set_value(&QString::from_std_str(setting), &QVariant::from_int(value));
        q_settings.sync();
    }
}

pub fn set_setting_f32(setting: &str, value: f32) {
    unsafe {
        let q_settings = QSettings::from_2_q_string(&QString::from_std_str(&*ORGANISATION.read().unwrap()), &QString::from_std_str(&*PROGRAM_NAME.read().unwrap()));
        q_settings.set_value(&QString::from_std_str(setting), &QVariant::from_float(value));
        q_settings.sync();
    }
}

pub fn set_setting_bool(setting: &str, value: bool) {
    unsafe {
        let q_settings = QSettings::from_2_q_string(&QString::from_std_str(&*ORGANISATION.read().unwrap()), &QString::from_std_str(&*PROGRAM_NAME.read().unwrap()));
        q_settings.set_value(&QString::from_std_str(setting), &QVariant::from_bool(value));
        q_settings.sync();
    }
}

pub fn set_setting_q_byte_array(setting: &str, value: Ref<QByteArray>) {
    unsafe {
        let q_settings = QSettings::from_2_q_string(&QString::from_std_str(&*ORGANISATION.read().unwrap()), &QString::from_std_str(&*PROGRAM_NAME.read().unwrap()));
        q_settings.set_value(&QString::from_std_str(setting), &QVariant::from_q_byte_array(value));
        q_settings.sync();
    }
}

pub fn set_setting_variant_to_q_setting(q_settings: &QBox<QSettings>, setting: &str, value: Ref<QVariant>) {
    unsafe {
        q_settings.set_value(&QString::from_std_str(setting), value);
    }
}

pub fn set_setting_path_to_q_setting(q_settings: &QBox<QSettings>, setting: &str, value: &Path) {
    set_setting_string_to_q_setting(q_settings, setting, &value.to_string_lossy())
}

pub fn set_setting_string_to_q_setting(q_settings: &QBox<QSettings>, setting: &str, value: &str) {
    unsafe {
        q_settings.set_value(&QString::from_std_str(setting), &QVariant::from_q_string(&QString::from_std_str(value)));
    }
}

pub fn set_setting_int_to_q_setting(q_settings: &QBox<QSettings>, setting: &str, value: i32) {
    unsafe {
        q_settings.set_value(&QString::from_std_str(setting), &QVariant::from_int(value));
    }
}

pub fn set_setting_f32_to_q_setting(q_settings: &QBox<QSettings>, setting: &str, value: f32) {
    unsafe {
        q_settings.set_value(&QString::from_std_str(setting), &QVariant::from_float(value));
    }
}

pub fn set_setting_bool_to_q_setting(q_settings: &QBox<QSettings>, setting: &str, value: bool) {
    unsafe {
        q_settings.set_value(&QString::from_std_str(setting), &QVariant::from_bool(value));
    }
}

pub fn set_setting_q_byte_array_to_q_setting(q_settings: &QBox<QSettings>, setting: &str, value: Ref<QByteArray>) {
    unsafe {
        q_settings.set_value(&QString::from_std_str(setting), &QVariant::from_q_byte_array(value));
    }
}

pub fn set_setting_if_new_path(q_settings: &QBox<QSettings>, setting: &str, value: &Path) {
    set_setting_if_new_string(q_settings, setting, &value.to_string_lossy())
}

pub fn set_setting_if_new_string(q_settings: &QBox<QSettings>, setting: &str, value: &str) {
    unsafe {
        if !q_settings.value_1a(&QString::from_std_str(setting)).is_valid() {
            q_settings.set_value(&QString::from_std_str(setting), &QVariant::from_q_string(&QString::from_std_str(value)));
        }
    }
}

pub fn set_setting_if_new_int(q_settings: &QBox<QSettings>, setting: &str, value: i32) {
    unsafe {
        if !q_settings.value_1a(&QString::from_std_str(setting)).is_valid() {
            q_settings.set_value(&QString::from_std_str(setting), &QVariant::from_int(value));
        }
    }
}

pub fn set_setting_if_new_f32(q_settings: &QBox<QSettings>, setting: &str, value: f32) {
    unsafe {
        if !q_settings.value_1a(&QString::from_std_str(setting)).is_valid() {
            q_settings.set_value(&QString::from_std_str(setting), &QVariant::from_float(value));
        }
    }
}

pub fn set_setting_if_new_bool(q_settings: &QBox<QSettings>, setting: &str, value: bool) {
    unsafe {
        if !q_settings.value_1a(&QString::from_std_str(setting)).is_valid() {
            q_settings.set_value(&QString::from_std_str(setting), &QVariant::from_bool(value));
        }
    }
}

pub fn set_setting_if_new_q_byte_array(q_settings: &QBox<QSettings>, setting: &str, value: Ref<QByteArray>) {
    unsafe {
        if !q_settings.value_1a(&QString::from_std_str(setting)).is_valid() {
            q_settings.set_value(&QString::from_std_str(setting), &QVariant::from_q_byte_array(value));
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
    if cfg!(debug_assertions) { std::env::current_dir().map_err(From::from) } else {
        match ProjectDirs::from(&QUALIFIER.read().unwrap(), &ORGANISATION.read().unwrap(), &PROGRAM_NAME.read().unwrap()) {
            Some(proj_dirs) => Ok(proj_dirs.config_dir().to_path_buf()),
            None => Err(anyhow!("Failed to get the config path."))
        }
    }
}

/// This function returns the path where crash logs are stored.
pub fn error_path() -> Result<PathBuf> {
    Ok(config_path()?.join("error"))
}
