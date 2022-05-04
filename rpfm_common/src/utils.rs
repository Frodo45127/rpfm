//---------------------------------------------------------------------------//
// Copyright (c) 2017-2022 Ismael Gutiérrez González. All rights reserved.
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

use anyhow::{anyhow, Result};
use chrono::{Utc, DateTime};
use pelite::pe64;
use pelite::resources::{FindError, Resources, version_info::VersionInfo};

use std::cmp::Ordering;
use std::fs::{File, read_dir};
use std::io::BufReader;
use std::path::{Path, PathBuf};

/// These consts are used for dealing with Time-related operations.
pub const WINDOWS_TICK: i64 = 10_000_000;
pub const SEC_TO_UNIX_EPOCH: i64 = 11_644_473_600;

/// This function retuns a `Vec<PathBuf>` containing all the files in the provided folder.
pub fn files_from_subdir(current_path: &Path, scan_subdirs: bool) -> Result<Vec<PathBuf>> {
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
                            let mut subfolder_files_path = files_from_subdir(&file_path, scan_subdirs)?;
                            file_list.append(&mut subfolder_files_path);
                        }
                    }
                    Err(_) => return Err(anyhow!("Error while trying to read the following file: {}
                        This means that path may not be readable by RPFM (permissions? other programs locking access to it?) or may not exists at all.",
                        current_path.to_string_lossy())),
                }
            }
        }

        // In case of reading error, report it.
        Err(_) => return Err(anyhow!("Error while trying to read the following file: {}
            This means that path may not be readable by RPFM (permissions? other programs locking access to it?) or may not exists at all.",
            current_path.to_string_lossy())),
    }

    // Return the list of paths.
    Ok(file_list)
}

/// This function gets the current date and return it, as a decoded u32.
pub fn current_time() -> i64 {
    Utc::now().naive_utc().timestamp()
}

/// This function gets the last modified date from a file and return it, as an i64.
pub fn last_modified_time_from_file(file: &File) -> Result<i64> {
    let last_modified_time: DateTime<Utc> = DateTime::from(file.metadata()?.modified()?);
    Ok(last_modified_time.naive_utc().timestamp())
}

/// This function gets the last modified date from a file and return it, as an i64.
pub fn last_modified_time_from_buffered_file(file: &BufReader<File>) -> Result<i64> {
    let last_modified_time: DateTime<Utc> = DateTime::from(file.get_ref().metadata()?.modified()?);
    Ok(last_modified_time.naive_utc().timestamp())
}

/// This function gets the newer last modified time from the provided list.
pub fn last_modified_time_from_files(paths: &[PathBuf]) -> Result<i64> {
    let mut last_time = 0;
    for path in paths {
        if path.is_file() {
            let file = File::open(path)?;
            let time = last_modified_time_from_file(&file)?;
            if time > last_time {
                last_time = time
            }
        }
    }

    Ok(last_time)
}

/// This function gets the oldest modified file in a folder and return it.
pub fn oldest_file_in_folder(current_path: &Path) -> Result<Option<PathBuf>> {
    let files = files_in_folder_from_newest_to_oldest(current_path)?;
    Ok(files.last().cloned())
}

/// This function gets the files in a folder sorted from newest to oldest.
pub fn files_in_folder_from_newest_to_oldest(current_path: &Path) -> Result<Vec<PathBuf>> {
    let mut files = files_from_subdir(current_path, false)?;
    files.sort();
    files.sort_by(|a, b| {
        if let Ok(a) = File::open(a) {
            if let Ok(b) = File::open(b) {
                if let Ok(a) = last_modified_time_from_file(&a) {
                    if let Ok(b) = last_modified_time_from_file(&b) {
                        a.cmp(&b)
                    } else { Ordering::Equal}
                } else { Ordering::Equal}
            } else { Ordering::Equal}
        } else { Ordering::Equal}
    });

    Ok(files)
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
        Err(anyhow!("Error while trying to parse the following value as a bool: {}", string))
    }
}

/// Function to get the version info of a file, courtesy of TES Loot team.
pub fn pe_version_info(bytes: &[u8]) -> std::result::Result<VersionInfo, FindError> {
    pe_resources(bytes)?.version_info()
}

/// Function to get the resources of a file, courtesy of TES Loot team.
pub fn pe_resources(bytes: &[u8]) -> std::result::Result<Resources, pelite::Error> {
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
