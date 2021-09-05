//---------------------------------------------------------------------------//
// Copyright (c) 2017-2020 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

/*!
Module with utility functions that don't fit anywhere else.

Basically, if you need a function, but it's kinda a generic function, it goes here.
!*/

use pelite::pe64;
use pelite::resources::{FindError, Resources};
use pelite::resources::version_info::VersionInfo;

use chrono::{Utc, DateTime};

use rpfm_error::{Error, ErrorKind, Result};

use std::cmp::Ordering;
use std::fs::{File, read_dir};
use std::io::{BufReader, Read};
use std::path::{Path, PathBuf};

use crate::schema;
use crate::settings::get_config_path;
use crate::games::supported_games::KEY_TROY;
use crate::GAME_SELECTED;

use crate::SETTINGS;

pub mod decoder;
pub mod encoder;

// This tells the compiler to only compile these mods when testing. It's just to make sure
// the encoders and decoders don't break between updates.
#[cfg(test)]
mod decoder_test;

#[cfg(test)]
mod encoder_test;

/// These consts are used for dealing with Time-related operations.
pub const WINDOWS_TICK: i64 = 10_000_000;
pub const SEC_TO_UNIX_EPOCH: i64 = 11_644_473_600;

/// This function retuns a `Vec<PathBuf>` containing all the files in the provided folder.
#[allow(dead_code)]
pub fn get_files_from_subdir(current_path: &Path, scan_subdirs: bool) -> Result<Vec<PathBuf>> {
    let mut file_list: Vec<PathBuf> = vec![];
    match read_dir(current_path) {
        Ok(files_in_current_path) => {
            for file in files_in_current_path {

                // Get his path and continue, or return an error if it can't be read.
                match file {
                    Ok(file) => {
                        let file_path = file.path();

                        // If it's a file, add it to the list. If it's a folder, add his files to the list.
                        if file_path.is_file() { file_list.push(file_path); }
                        else if file_path.is_dir() && scan_subdirs {
                            let mut subfolder_files_path = get_files_from_subdir(&file_path, scan_subdirs)?;
                            file_list.append(&mut subfolder_files_path);
                        }
                    }
                    Err(_) => return Err(ErrorKind::IOReadFile(current_path.to_path_buf()).into()),
                }
            }
        }

        // In case of reading error, report it.
        Err(_) => return Err(ErrorKind::IOReadFolder(current_path.to_path_buf()).into()),
    }

    // Return the list of paths.
    Ok(file_list)
}

/// This function gets the current date and return it, as a decoded u32.
#[allow(dead_code)]
pub fn get_current_time() -> i64 {
    Utc::now().naive_utc().timestamp()
}

/// This function gets the last modified date from a file and return it, as an i64.
#[allow(dead_code)]
pub fn get_last_modified_time_from_file(file: &File) -> Result<i64> {
    let last_modified_time: DateTime<Utc> = DateTime::from(file.metadata()?.modified()?);
    Ok(last_modified_time.naive_utc().timestamp())
}

/// This function gets the last modified date from a file and return it, as an i64.
#[allow(dead_code)]
pub fn get_last_modified_time_from_buffered_file(file: &BufReader<File>) -> Result<i64> {
    let last_modified_time: DateTime<Utc> = DateTime::from(file.get_ref().metadata()?.modified()?);
    Ok(last_modified_time.naive_utc().timestamp())
}

/// This function gets the newer last modified time from the provided list.
#[allow(dead_code)]
pub fn get_last_modified_time_from_files(paths: &[PathBuf]) -> Result<i64> {
    let mut last_time = 0;
    for path in paths {
        if path.is_file() {
            let file = File::open(path)?;
            let time = get_last_modified_time_from_file(&file)?;
            if time > last_time {
                last_time = time
            }
        }
    }

    Ok(last_time)
}

/// This function gets the oldest modified file in a folder and return it.
#[allow(dead_code)]
pub fn get_oldest_file_in_folder(current_path: &Path) -> Result<Option<PathBuf>> {
    let files = get_files_in_folder_from_newest_to_oldest(current_path)?;
    Ok(files.last().cloned())
}

/// This function gets the files in a folder sorted from newest to oldest.
#[allow(dead_code)]
pub fn get_files_in_folder_from_newest_to_oldest(current_path: &Path) -> Result<Vec<PathBuf>> {
    let mut files = get_files_from_subdir(current_path, false)?;
    files.sort();
    files.sort_by(|a, b| {
        if let Ok(a) = File::open(a) {
            if let Ok(b) = File::open(b) {
                if let Ok(a) = get_last_modified_time_from_file(&a) {
                    if let Ok(b) = get_last_modified_time_from_file(&b) {
                        a.cmp(&b)
                    } else { Ordering::Equal}
                } else { Ordering::Equal}
            } else { Ordering::Equal}
        } else { Ordering::Equal}
    });

    Ok(files)
}

/// This function returns the schema path.
#[allow(dead_code)]
pub fn get_schemas_path() -> Result<PathBuf> {
    Ok(get_config_path()?.join(schema::SCHEMA_FOLDER))
}

/// This function returns the autosave path.
#[allow(dead_code)]
pub fn get_backup_autosave_path() -> Result<PathBuf> {
    Ok(get_config_path()?.join("autosaves"))
}

/// This function parses strings to booleans, properly.
pub fn parse_str_as_bool(string: &str) -> Result<bool> {
    let str_lower_case = string.to_lowercase();
    if str_lower_case == "true" || str_lower_case == "1" {
        Ok(true)
    }
    else if str_lower_case == "false" || str_lower_case == "0" {
        Ok(false)
    }
    else {
        Err(ErrorKind::NotABooleanValue.into())
    }
}

/// This function gets the version number of the exe for the current GameSelected, if it exists.
#[allow(dead_code)]
pub fn get_game_selected_exe_version_number() -> Result<u32> {
    let game_selected  = GAME_SELECTED.read().unwrap().get_game_key_name();
    match &*game_selected {
        KEY_TROY => {
            let mut path = SETTINGS.read().unwrap().paths[&game_selected].clone().ok_or_else(|| Error::from(ErrorKind::GameNotSupported))?;
            path.push("Troy.exe");
            if path.is_file() {
                let mut data = vec![];
                let mut file = BufReader::new(File::open(path)?);
                file.read_to_end(&mut data)?;

                let version_info = get_pe_version_info(&data).map_err(|_| Error::from(ErrorKind::IOGeneric))?;

                match version_info.fixed() {
                    Some(version_info) => {
                        let mut version: u32 = 0;

                        // The CA format is limited so these can only be u8 when encoded, so we can safetly convert them.
                        let major = version_info.dwFileVersion.Major as u32;
                        let minor = version_info.dwFileVersion.Minor as u32;
                        let patch = version_info.dwFileVersion.Patch as u32;
                        let build = version_info.dwFileVersion.Build as u32;

                        version += major << 24;
                        version += minor << 16;
                        version += patch << 8;
                        version += build;
                        Ok(version)
                    }

                    None => Err(ErrorKind::GamePathNotConfigured.into()),
                }
            }

            // If we have no exe, return a default value.
            else {
                Err(ErrorKind::GamePathNotConfigured.into())
            }

        }

        _ => Err(ErrorKind::GamePathNotConfigured.into()),
    }
}

/// Function to get the version info of a file, courtesy of TES Loot team.
fn get_pe_version_info(bytes: &[u8]) -> std::result::Result<VersionInfo, FindError> {
    get_pe_resources(bytes)?.version_info()
}

/// Function to get the resources of a file, courtesy of TES Loot team.
fn get_pe_resources(bytes: &[u8]) -> std::result::Result<Resources, pelite::Error> {
    match pe64::PeFile::from_bytes(bytes) {
        Ok(file) => {
            use pelite::pe64::Pe;

            file.resources()
        }
        Err(pelite::Error::PeMagic) => {
            use pelite::pe32::{Pe, PeFile};

            PeFile::from_bytes(bytes)?.resources()
        }
        Err(e) => Err(e),
    }
}
