//---------------------------------------------------------------------------//
// Copyright (c) 2017-2024 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Module with generic functions used by the crate.
//!
//! If a function doesn't fit anywhere, it goes here.

use pelite::pe64;
use pelite::resources::{FindError, Resources, version_info::VersionInfo};
use rayon::prelude::*;

use std::cmp::Ordering;
use std::fs::{canonicalize, read_dir, File};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::error::{RLibError, Result};

//--------------------------------------------------------//
// Generic utils.
//--------------------------------------------------------//

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
        Err(RLibError::ParseBoolError(string.to_owned()))
    }
}

/// This function checks if a String starts with another String in a case-insensitive way.
pub fn starts_with_case_insensitive(full_str: &str, partial_str: &str) -> bool {
    let full_str_chars = full_str.chars().count();
    let partial_str_chars = partial_str.chars().count();
    if full_str_chars > partial_str_chars {
        let partial_str_len_in_bytes = partial_str.len();

        let full_str_max_index = full_str.char_indices().map(|(index, _)| index).find(|index| index >= &partial_str_len_in_bytes).unwrap_or(full_str.len());
        let full_str_base = &full_str[..full_str_max_index];
        caseless::default_caseless_match_str(full_str_base, partial_str)
    } else {
        false
    }
}

/// This function returns the first byte where a character starts, starting on the one provided.
pub fn closest_valid_char_byte(string: &str, start_byte: usize) -> usize {
    if start_byte < string.len() && string.get(start_byte..).is_some() { start_byte }
    else if start_byte + 1 < string.len() && string.get(start_byte + 1..).is_some() { start_byte + 1 }
    else if start_byte + 2 < string.len() && string.get(start_byte + 2..).is_some() { start_byte + 2 }
    else if start_byte + 3 < string.len() && string.get(start_byte + 3..).is_some() { start_byte + 3 }

    // Characters are max 4 bytes. This can never happen unless you provide an invalid start_byte.
    else { unimplemented!() }
}

/// This function returns the line and column of a position in a string. Only works for \r\n-terminated lines.
pub fn line_column_from_string_pos(string: &str, pos: u64) -> (u64, u64) {
    let mut row = 0;
    let mut col = 0;
    let mut pos_processed = 0;
    let end_skip = if string.contains("\r\n") { 2 } else { 1 };

    for (index, line) in string.lines().enumerate() {

        // If we're not yet in the line, continue.
        if pos > pos_processed + line.len() as u64 {
            pos_processed += line.len() as u64 + end_skip;
            continue;
        }

        // If we're in the line, find the column.
        else {
            row = index as u64;
            col = pos.checked_sub(pos_processed).unwrap_or_default();
            break;
        }
    }

    (row, col)
}

//--------------------------------------------------------//
// Path utils.
//--------------------------------------------------------//

/// This function retuns a `Vec<PathBuf>` containing all the files in the provided folder.
pub fn files_from_subdir(current_path: &Path, scan_subdirs: bool) -> Result<Vec<PathBuf>> {

    // Fast path. Takes a few ms less than the other one.
    if !scan_subdirs {
        return Ok(read_dir(current_path)?
            .flatten()
            .filter(|file| {
                if let Ok(metadata) = file.metadata() {
                    metadata.is_file()
                } else { false }
            })
            .map(|file| file.path()).collect());
    }

    // Slow path. Can scan subdirs.
    let mut file_list: Vec<PathBuf> = vec![];
    match read_dir(current_path) {
        Ok(files_in_current_path) => {
            for file in files_in_current_path {

                // Get his path and continue, or return an error if it can't be read.
                match file {
                    Ok(file) => {
                        let file_path = file.path();

                        // If it's a file, add it to the list.
                        if file_path.is_file() {
                            file_list.push(file_path);
                        }

                        // If it's a folder, add his files to the list.
                        else if file_path.is_dir() && scan_subdirs {
                            let mut subfolder_files_path = files_from_subdir(&file_path, scan_subdirs)?;
                            file_list.append(&mut subfolder_files_path);
                        }
                    }
                    Err(_) => return Err(RLibError::ReadFileFolderError(current_path.to_string_lossy().to_string())),
                }
            }
        }

        // In case of reading error, report it.
        Err(_) => return Err(RLibError::ReadFileFolderError(current_path.to_string_lossy().to_string())),
    }

    // Return the list of paths.
    Ok(file_list)
}

/// This function retuns a `Vec<PathBuf>` containing all the folders which do not have any more folders under them.
pub fn final_folders_from_subdir(current_path: &Path, ignore_empty_folders: bool) -> Result<Vec<PathBuf>> {
    let mut folder_list: Vec<PathBuf> = vec![];
    match read_dir(current_path) {
        Ok(dir_entry_in_current_path) => {
            let mut has_subfolders = false;
            let mut has_files = false;
            for dir_entry in dir_entry_in_current_path {

                // Get his path and continue, or return an error if it can't be read.
                match dir_entry {
                    Ok(dir_entry) => {
                        let path = dir_entry.path();

                        // If it's a file, skip it.
                        if path.is_file() {
                            has_files = true;
                            continue;
                        }

                        if path.is_dir() {
                        // If it's a folder, check it..
                            let mut subfolder_files_path = final_folders_from_subdir(&path, ignore_empty_folders)?;
                            folder_list.append(&mut subfolder_files_path);
                            has_subfolders = true;
                        }
                    }
                    Err(_) => return Err(RLibError::ReadFileFolderError(current_path.to_string_lossy().to_string())),
                }
            }

            if !has_subfolders && (!ignore_empty_folders || has_files) {
                folder_list.push(current_path.to_path_buf());
            }
        }

        // In case of reading error, report it.
        Err(_) => return Err(RLibError::ReadFileFolderError(current_path.to_string_lossy().to_string())),
    }

    // Return the list of paths.
    Ok(folder_list)
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
                        b.cmp(&a)
                    } else { Ordering::Equal}
                } else { Ordering::Equal}
            } else { Ordering::Equal}
        } else { Ordering::Equal}
    });

    Ok(files)
}

/// This function generates an absolute path string from a path.
///
/// This does nothing if the path doesn't exists.
pub fn path_to_absolute_string(path: &Path) -> String {
    let mut path_str = path.to_string_lossy().to_string();

    match canonicalize(path) {
        Ok(cannon_path) => {
            let cannon_path_str = cannon_path.to_string_lossy();
            if let Some(strip) = cannon_path_str.strip_prefix("\\\\?\\") {
                path_str = strip.to_owned();
            } else {
                path_str = cannon_path_str.to_string();
            }
        },

        // These errors are usually for trying to cannonicalize an already cannon path, or because the file doesn't exist.
        Err(_) => {
            if path_str.starts_with("\\\\?\\") {
                path_str = path_str[4..].to_owned();
            }
        }
    }

    path_str
}

/// This function generates an absolute path from a path.
///
/// This does nothing if the path doesn't exists.
pub fn path_to_absolute_path(path: &Path, strip_prefix: bool) -> PathBuf {
    let mut path = path.to_owned();

    match canonicalize(&path) {
        Ok(cannon_path) => {
            let cannon_path_str = cannon_path.to_string_lossy();

            if strip_prefix {
                if let Some(strip) = cannon_path_str.strip_prefix("\\\\?\\") {
                    path = PathBuf::from(strip);
                } else {
                    path = cannon_path;
                }
            } else {
                path = cannon_path;
            }
        },

        // These errors are usually for trying to cannonicalize an already cannon path, or because the file doesn't exist.
        Err(_) => {
            let path_str = path.to_string_lossy();
            if strip_prefix {
                if let Some(strip) = path_str.strip_prefix("\\\\?\\") {
                    path = PathBuf::from(strip);
                }
            }
        }
    }

    path
}


//--------------------------------------------------------//
// Time utils.
//--------------------------------------------------------//

/// This function gets the current date and return it, as an u64.
pub fn current_time() -> Result<u64> {
    Ok(SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs())
}

/// This function gets the last modified date from a file and return it, as an u64.
pub fn last_modified_time_from_file(file: &File) -> Result<u64> {
    Ok(file.metadata()?.modified()?.duration_since(UNIX_EPOCH)?.as_secs())
}

/// This function gets the newer last modified time from the provided list.
pub fn last_modified_time_from_files(paths: &[PathBuf]) -> Result<u64> {
    Ok(paths
        .par_iter()
        .filter_map(|path| File::open(path).ok())
        .filter_map(|file| last_modified_time_from_file(&file).ok())
        .max().unwrap_or(0)
    )
}

//--------------------------------------------------------//
// Pelite utils.
//--------------------------------------------------------//

/// Function to get the version info of a file, courtesy of TES Loot team.
pub(crate) fn pe_version_info(bytes: &[u8]) -> std::result::Result<VersionInfo, FindError> {
    pe_resources(bytes)?.version_info()
}

/// Function to get the resources of a file, courtesy of TES Loot team.
pub(crate) fn pe_resources(bytes: &[u8]) -> std::result::Result<Resources, pelite::Error> {
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

//--------------------------------------------------------//
// VWise utils.
//--------------------------------------------------------//

const VWISE_HASH_VALUE: u32 = 0x811C9DC5;
const VWISE_MULT_VALUE: u32 = 0x01000193;
const VWISE_AND_VALUE: u32 = 0xFFFFFFFF;

/// Function to generate a vwise hash from a file name.
///
/// Copy/pasted from Asset Editor.
pub fn hash_vwise(name: &str) -> u32 {
    let name = name.trim().to_lowercase();
    let mut hash_value = VWISE_HASH_VALUE;
    for byte in name.as_bytes() {
        hash_value *= VWISE_MULT_VALUE;
        hash_value ^= *byte as u32;
        hash_value &= VWISE_AND_VALUE;
    }

    hash_value
}

//--------------------------------------------------------//
// Decoder utils.
//--------------------------------------------------------//

/// Function to check for a size mismatch error (we expected the cursor to be at `expected_pos`,
/// but instead we're at `curr_pos`).
pub(crate) fn check_size_mismatch(curr_pos: usize, expected_pos: usize) -> Result<()> {
    if curr_pos != expected_pos {
        return Err(RLibError::DecodingMismatchSizeError(expected_pos, curr_pos));
    }

    Ok(())
}
