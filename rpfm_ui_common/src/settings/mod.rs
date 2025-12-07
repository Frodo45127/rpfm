//---------------------------------------------------------------------------//
// Copyright (c) 2017-2024 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

use anyhow::{anyhow, Result};
use directories::ProjectDirs;
use serde_derive::{Serialize, Deserialize};

use std::collections::HashMap;
use std::io::{BufReader, BufWriter, Read, Write};
use std::fs::File;
use std::path::{Path, PathBuf};

use crate::APP_NAME;
use crate::ORG_DOMAIN;
use crate::ORG_NAME;

const SETTINGS_FILE_NAME: &str = "settings.json";

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
//                         Setting-related functions
//-------------------------------------------------------------------------------//

impl Settings {

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
        self.bool.get(setting).map(|x| *x).unwrap_or_default()
    }

    pub fn i32(&self, setting: &str) -> i32 {
        self.i32.get(setting).map(|x| *x).unwrap_or_default()
    }

    pub fn f32(&self, setting: &str) -> f32 {
        self.f32.get(setting).map(|x| *x).unwrap_or_default()
    }

    pub fn string(&self, setting: &str) -> String {
        self.string.get(setting).map(|x| x.to_owned()).unwrap_or_default()
    }

    pub fn path_buf(&self, setting: &str) -> PathBuf {
        self.string.get(setting).map(|x| PathBuf::from(x)).unwrap_or_default()
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
        if self.bool.get(setting).is_none() {
            self.bool.insert(setting.to_owned(), value);
        }
    }

    pub fn initialize_i32(&mut self, setting: &str, value: i32) {
        if self.i32.get(setting).is_none() {
            self.i32.insert(setting.to_owned(), value);
        }
    }

    pub fn initialize_f32(&mut self, setting: &str, value: f32) {
        if self.f32.get(setting).is_none() {
            self.f32.insert(setting.to_owned(), value);
        }
    }

    pub fn initialize_string(&mut self, setting: &str, value: &str) {
        if self.string.get(setting).is_none() {
            self.string.insert(setting.to_owned(), value.to_owned());
        }
    }

    pub fn initialize_path_buf(&mut self, setting: &str, value: &Path) {
        if self.string.get(setting).is_none() {
            self.string.insert(setting.to_owned(), value.to_string_lossy().to_string());
        }
    }

    pub fn initialize_raw_data(&mut self, setting: &str, value: &[u8]) {
        if self.raw_data.get(setting).is_none() {
            self.raw_data.insert(setting.to_owned(), value.to_vec());
        }
    }

    pub fn initialize_vec_string(&mut self, setting: &str, value: &[String]) {
        if self.vec_string.get(setting).is_none() {
            self.vec_string.insert(setting.to_owned(), value.to_vec());
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
        match ProjectDirs::from(&ORG_DOMAIN.read().unwrap(), &ORG_NAME.read().unwrap(), &APP_NAME.read().unwrap()) {
            Some(proj_dirs) => Ok(proj_dirs.config_dir().to_path_buf()),
            None => Err(anyhow!("Failed to get the config path."))
        }
    }
}

/// This function returns the path where crash logs are stored.
pub fn error_path() -> Result<PathBuf> {
    Ok(config_path()?.join("error"))
}
