//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Generic utility functions for the crate.
//!
//! This module contains miscellaneous utility functions that don't fit into more specific modules.
//! Functions are organized into categories:
//!
//! - **Generic utils**: String parsing, case-insensitive operations, line/column calculations
//! - **Path utils**: File and folder enumeration, path sanitization, absolute path conversion
//! - **Time utils**: File modification time queries, current time helpers
//! - **Pelite utils**: PE (Windows executable) file inspection
//! - **VWise utils**: WWise audio hash generation
//! - **Filename sanitization**: Windows-compatible filename cleaning
//! - **Decoder utils**: Size mismatch validation for binary decoders

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

/// Parses a string to a boolean value.
///
/// Accepts common boolean representations (case-insensitive):
/// - `true` or `"1"` → `true`
/// - `false` or `"0"` → `false`
///
/// # Arguments
///
/// * `string` - The string to parse
///
/// # Returns
///
/// Returns `Ok(bool)` if the string is a valid boolean representation, or
/// `Err` if the string cannot be parsed.
///
/// # Examples
///
/// ```
/// # use rpfm_lib::utils::parse_str_as_bool;
/// assert_eq!(parse_str_as_bool("true").unwrap(), true);
/// assert_eq!(parse_str_as_bool("1").unwrap(), true);
/// assert_eq!(parse_str_as_bool("FALSE").unwrap(), false);
/// assert_eq!(parse_str_as_bool("0").unwrap(), false);
/// assert!(parse_str_as_bool("maybe").is_err());
/// ```
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

/// Checks if a string starts with another string (case-insensitive).
///
/// This function performs a case-insensitive prefix check, handling UTF-8 strings correctly
/// by working with character boundaries rather than byte boundaries.
///
/// # Arguments
///
/// * `full_str` - The string to check
/// * `partial_str` - The prefix to look for
///
/// # Returns
///
/// Returns `true` if `full_str` starts with `partial_str` (ignoring case), `false` otherwise.
///
/// # Examples
///
/// ```
/// # use rpfm_lib::utils::starts_with_case_insensitive;
/// assert!(starts_with_case_insensitive("Hello World", "hello"));
/// assert!(starts_with_case_insensitive("RPFM", "rpf"));
/// assert!(!starts_with_case_insensitive("Short", "ThisIsLonger"));
/// ```
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

/// Finds the closest valid UTF-8 character boundary at or after the given byte position.
///
/// When working with byte indices in UTF-8 strings, you may land in the middle of a
/// multi-byte character. This function finds the next valid character boundary.
///
/// # Arguments
///
/// * `string` - The string to search in
/// * `start_byte` - The byte index to start from
///
/// # Returns
///
/// Returns the byte index of the next valid character boundary (may be `start_byte` itself
/// if it's already at a boundary).
///
/// # Panics
///
/// Panics if `start_byte` is more than 3 bytes away from the next valid boundary, which
/// should never happen with valid UTF-8 (max character size is 4 bytes).
pub fn closest_valid_char_byte(string: &str, start_byte: usize) -> usize {
    if start_byte < string.len() && string.get(start_byte..).is_some() { start_byte }
    else if start_byte + 1 < string.len() && string.get(start_byte + 1..).is_some() { start_byte + 1 }
    else if start_byte + 2 < string.len() && string.get(start_byte + 2..).is_some() { start_byte + 2 }
    else if start_byte + 3 < string.len() && string.get(start_byte + 3..).is_some() { start_byte + 3 }

    // Characters are max 4 bytes. This can never happen unless you provide an invalid start_byte.
    else { unimplemented!() }
}

/// Converts a byte position in a string to a line and column number.
///
/// This function is useful for error reporting and text editor-like functionality,
/// where you need to display human-readable positions.
///
/// # Arguments
///
/// * `string` - The string to analyze
/// * `pos` - The byte position in the string
///
/// # Returns
///
/// Returns a tuple of `(line, column)` where both are 0-indexed.
///
/// # Note
///
/// Works with both `\r\n` (Windows) and `\n` (Unix) line endings.
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

/// Returns all files in a directory, optionally scanning subdirectories recursively.
///
/// # Arguments
///
/// * `current_path` - The directory to scan
/// * `scan_subdirs` - If `true`, recursively scans subdirectories; if `false`, only scans the top level
///
/// # Returns
///
/// Returns a vector of paths to all files found, or an error if the directory cannot be read.
///
/// # Examples
///
/// ```no_run
/// # use std::path::Path;
/// # use rpfm_lib::utils::files_from_subdir;
/// // Get only files in the current directory
/// let files = files_from_subdir(Path::new("./data"), false)?;
///
/// // Get all files recursively
/// let all_files = files_from_subdir(Path::new("./data"), true)?;
/// # Ok::<(), rpfm_lib::error::RLibError>(())
/// ```
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

/// Returns all leaf directories (folders with no subfolders) in a directory tree.
///
/// This function recursively scans a directory and returns all directories that don't
/// contain any subdirectories (leaf nodes in the directory tree).
///
/// # Arguments
///
/// * `current_path` - The directory to start scanning from
/// * `ignore_empty_folders` - If `true`, only includes folders that contain files; if `false`, includes all leaf folders
///
/// # Returns
///
/// Returns a vector of paths to all leaf directories, or an error if any directory cannot be read.
///
/// # Examples
///
/// ```no_run
/// # use std::path::Path;
/// # use rpfm_lib::utils::final_folders_from_subdir;
/// // Get all leaf folders, including empty ones
/// let leaves = final_folders_from_subdir(Path::new("./project"), false)?;
///
/// // Get only leaf folders that contain files
/// let non_empty_leaves = final_folders_from_subdir(Path::new("./project"), true)?;
/// # Ok::<(), rpfm_lib::error::RLibError>(())
/// ```
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

/// Returns the oldest file in a directory based on modification time.
///
/// # Arguments
///
/// * `current_path` - The directory to search
///
/// # Returns
///
/// Returns `Some(PathBuf)` pointing to the oldest file, or `None` if the directory is empty.
pub fn oldest_file_in_folder(current_path: &Path) -> Result<Option<PathBuf>> {
    let files = files_in_folder_from_newest_to_oldest(current_path)?;
    Ok(files.last().cloned())
}

/// Returns all files in a directory sorted by modification time (newest first).
///
/// # Arguments
///
/// * `current_path` - The directory to search (non-recursive)
///
/// # Returns
///
/// Returns a vector of file paths sorted from newest to oldest by modification time.
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

/// Converts a path to an absolute path string, stripping Windows UNC prefix if present.
///
/// This function canonicalizes the path and removes the Windows `\\?\` prefix if it exists.
/// If canonicalization fails (e.g., path doesn't exist), it returns the path as-is.
///
/// # Arguments
///
/// * `path` - The path to convert
///
/// # Returns
///
/// Returns the absolute path as a string, with Windows UNC prefix removed.
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

/// Converts a path to an absolute [`PathBuf`], optionally stripping Windows UNC prefix.
///
/// This function canonicalizes the path and optionally removes the Windows `\\?\` prefix.
/// If canonicalization fails (e.g., path doesn't exist), it returns the path as-is.
///
/// # Arguments
///
/// * `path` - The path to convert
/// * `strip_prefix` - If `true`, removes the Windows `\\?\` prefix
///
/// # Returns
///
/// Returns the absolute path, with Windows UNC prefix removed if `strip_prefix` is `true`.
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

/// Returns the current Unix timestamp in seconds.
///
/// # Returns
///
/// Returns the number of seconds since the Unix epoch (January 1, 1970 UTC).
pub fn current_time() -> Result<u64> {
    Ok(SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs())
}

/// Returns the last modification time of a file as a Unix timestamp.
///
/// # Arguments
///
/// * `file` - The file handle to query
///
/// # Returns
///
/// Returns the number of seconds since the Unix epoch when the file was last modified.
pub fn last_modified_time_from_file(file: &File) -> Result<u64> {
    Ok(file.metadata()?.modified()?.duration_since(UNIX_EPOCH)?.as_secs())
}

/// Returns the most recent modification time from a list of file paths.
///
/// This function checks all provided paths in parallel and returns the newest
/// modification timestamp. Files that cannot be opened are silently ignored.
///
/// # Arguments
///
/// * `paths` - Slice of file paths to check
///
/// # Returns
///
/// Returns the newest modification time in seconds since Unix epoch, or `0` if no files could be read.
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

/// Extracts version information from a Windows PE (Portable Executable) file.
///
/// This function parses the PE resources to extract version information embedded
/// in the executable. Courtesy of the TES Loot team.
///
/// # Arguments
///
/// * `bytes` - The raw PE file bytes
///
/// # Returns
///
/// Returns the version information structure, or an error if parsing fails.
pub(crate) fn pe_version_info(bytes: &'_ [u8]) -> std::result::Result<VersionInfo<'_>, FindError> {
    pe_resources(bytes)?.version_info()
}

/// Extracts the resource section from a Windows PE file.
///
/// This function parses a PE file (32-bit or 64-bit) and extracts its resource section.
/// Courtesy of the TES Loot team.
///
/// # Arguments
///
/// * `bytes` - The raw PE file bytes
///
/// # Returns
///
/// Returns the resources structure, or an error if parsing fails.
pub(crate) fn pe_resources(bytes: &'_ [u8]) -> std::result::Result<Resources<'_>, pelite::Error> {
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

/// Generates a WWise audio hash from a filename.
///
/// This function implements the WWise audio engine's hash algorithm for identifying
/// audio files and events. The algorithm performs FNV-1a hashing on the lowercase,
/// trimmed filename.
///
/// # Arguments
///
/// * `name` - The filename to hash (will be trimmed and lowercased)
///
/// # Returns
///
/// Returns the 32-bit WWise hash value.
///
/// # Note
///
/// Implementation courtesy of Asset Editor.
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
// Filename sanitization utils.
//--------------------------------------------------------//

/// Windows-invalid filename characters.
///
/// These characters cannot be used in Windows filenames: `< > : " / \ | ? *`
pub const INVALID_CHARACTERS_WINDOWS: [char; 9] = [
    '<',
    '>',
    ':',
    '"',
    '/',
    '\\',
    '|',
    '?',
    '*',
];

/// Default filename used when sanitization results in an empty name.
pub const DEFAULT_FILENAME: &str = "unnamed_file";

/// Sanitizes a file path by cleaning the filename component.
///
/// This function applies filename sanitization to the filename part of a path while
/// preserving the directory structure. Invalid Windows characters are replaced with underscores.
///
/// # Arguments
///
/// * `path` - The path to sanitize
///
/// # Returns
///
/// Returns a new path with a sanitized filename.
///
/// # Examples
///
/// ```
/// # use std::path::Path;
/// # use rpfm_lib::utils::sanitize_path;
/// let bad_path = Path::new("data/my:file?.txt");
/// let clean_path = sanitize_path(bad_path);
/// assert_eq!(clean_path, Path::new("data/my_file_.txt"));
/// ```
pub fn sanitize_path(path: &Path) -> PathBuf {
    if let Some(file_name) = path.file_name() {
        let sanitized_name = sanitize_filename(file_name.to_string_lossy().as_ref());
        let mut sanitized_path = path.to_path_buf();
        sanitized_path.set_file_name(sanitized_name);
        sanitized_path
    } else {
        path.to_path_buf()
    }
}

/// Sanitizes a filename by replacing invalid Windows characters.
///
/// This function ensures filenames are valid on Windows by:
/// - Replacing invalid characters (`< > : " / \ | ? *`) with underscores
/// - Removing leading/trailing whitespace and dots
/// - Using a default name if the result is empty
///
/// # Arguments
///
/// * `filename` - The filename to sanitize
///
/// # Returns
///
/// Returns a Windows-compatible filename.
///
/// # Examples
///
/// ```
/// # use rpfm_lib::utils::sanitize_filename;
/// assert_eq!(sanitize_filename("my:file?.txt"), "my_file_.txt");
/// assert_eq!(sanitize_filename("   .hidden   "), "hidden");
/// assert_eq!(sanitize_filename("<<<"), "___");
/// assert_eq!(sanitize_filename("..."), "unnamed_file");
/// ```
pub fn sanitize_filename(filename: &str) -> String {
    let mut sanitized = filename.to_string();

    // Replace invalid characters with underscores.
    for &ch in &INVALID_CHARACTERS_WINDOWS {
        sanitized = sanitized.replace(ch, "_");
    }

    // Remove leading/trailing spaces and dots.
    sanitized = sanitized.trim().trim_matches('.').to_string();

    // If the filename becomes empty after sanitization, use a default name.
    if sanitized.is_empty() {
        sanitized = DEFAULT_FILENAME.to_string();
    }

    sanitized
}

//--------------------------------------------------------//
// Decoder utils.
//--------------------------------------------------------//

/// Validates that a decoder cursor is at the expected position.
///
/// This function is used internally by binary decoders to verify that parsing ended
/// at the expected byte position, helping detect format mismatches or decoding errors.
///
/// # Arguments
///
/// * `curr_pos` - The current cursor position
/// * `expected_pos` - The expected cursor position
///
/// # Returns
///
/// Returns [`Ok`] if positions match, or an error if there's a size mismatch.
///
/// # Errors
///
/// Returns [`RLibError::DecodingMismatchSizeError`] if the positions don't match.
pub(crate) fn check_size_mismatch(curr_pos: usize, expected_pos: usize) -> Result<()> {
    if curr_pos != expected_pos {
        return Err(RLibError::DecodingMismatchSizeError(expected_pos, curr_pos));
    }

    Ok(())
}
