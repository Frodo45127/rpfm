// In this file are all the "Generic" helper functions used by RPFM (no UI code here).
// As we may or may not use them, all functions here should have the "#[allow(dead_code)]"
// var set, so the compiler doesn't spam us every time we try to compile.
extern crate chrono;
extern crate serde;
extern crate serde_json;

use chrono::{Utc, DateTime};

use std::fs::{File, read_dir};
use std::path::{Path, PathBuf};

use error::{ErrorKind, Result};
use packfile::packfile::PackFile;

pub mod coding_helpers;
pub mod communications;

// This tells the compiler to only compile this mod when testing. It's just to make sure the "coders" don't break.
#[cfg(test)]
pub mod tests;

/// This const is the standard message in case of message deserializing error. If this happens, crash the program and send a report to Sentry.
pub const THREADS_MESSAGE_ERROR: &str = "Error in thread messages system.";

/// This const is the standard message in case of message communication error. If this happens, crash the program and send a report to Sentry.
pub const THREADS_COMMUNICATION_ERROR: &str = "Error in thread communication system.";

/// This enum has the different types of selected items in a TreeView. File has (tree_path without
/// the mod's name, index in PackFile). Folder has the tree_path without the mod's name.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum TreePathType {
    File((Vec<String>, usize)),
    Folder(Vec<String>),
    PackFile,
    None,
}

/// Custom implementation of "PartialEq" for "TreePathType", so we don't need to match each time while
/// want to compare two TreePathType.
impl PartialEq for TreePathType {
    fn eq(&self, other: &TreePathType) -> bool {
        match (self, other) {
            (&TreePathType::File((_,_)), &TreePathType::File((_,_))) |
            (&TreePathType::Folder(_), &TreePathType::Folder(_)) |
            (&TreePathType::PackFile, &TreePathType::PackFile) |
            (&TreePathType::None, &TreePathType::None) => true,
            _ => false,
        }
    }
}

/// This function checks if the PackedFile at the given TreePath is a file or a folder. Please note
/// that the tree_path NEEDS TO BE COMPLETE (including PackFile's name) for the function to work
/// properly.
#[allow(dead_code)]
pub fn get_type_of_selected_path(
    tree_path: &[String],
    pack_file_decoded: &PackFile
) -> TreePathType {

    // Get a local copy of the Path.
    let mut tree_path = tree_path.to_owned();

    // If we don't have anything, it's an invalid path.
    if tree_path.is_empty() { return TreePathType::None }

    // If the path is just the PackFile's name, it's the PackFile.
    else if tree_path.len() == 1 && tree_path[0] == pack_file_decoded.extra_data.file_name {
        return TreePathType::PackFile
    }

    // If is not a PackFile...
    else {

        // We remove his first field, as our PackedFiles's paths don't have it.
        tree_path.reverse();
        tree_path.pop();
        tree_path.reverse();

        // Now we check if it's a file or a folder.
        let mut is_a_file = false;
        let mut index = 0;
        for i in &pack_file_decoded.data.packed_files {
            if i.path == tree_path {
                is_a_file = true;
                break;
            }
            index += 1;
        }

        // If is a file, we return it.
        if is_a_file { return TreePathType::File((tree_path, index)) }

        // Otherwise, we assume it's a folder. This is not bulletproof so FIXME: find a way to make this more solid.
        else {

            // We check if the folder actually exists in our PackFile.
            let is_a_folder = pack_file_decoded.data.folder_exists(&tree_path);

            // If it exists, we return it as a folder.
            if is_a_folder { return TreePathType::Folder(tree_path) }

            // Otherwise, it's a None.
            else { return TreePathType::None }
        }
    }
}

/// This function takes a &Path and returns a Vec<PathBuf> with the paths of every file under the &Path.
#[allow(dead_code)]
pub fn get_files_from_subdir(current_path: &Path) -> Result<Vec<PathBuf>> {

    // Create the list of files.
    let mut file_list: Vec<PathBuf> = vec![];

    // Get everything from the path we have.
    match read_dir(current_path) {

        // If we don't have any problems reading it...
        Ok(files_in_current_path) => {

            // For each thing in the current path...
            for file in files_in_current_path {

                // Get his path
                let file_path = file.unwrap().path().clone();

                // If it's a file, to the file_list it goes
                if file_path.is_file() { file_list.push(file_path); }

                // If it's a folder...
                else if file_path.is_dir() {

                    // Get the list of files inside of the folder...
                    let mut subfolder_files_path = get_files_from_subdir(&file_path).unwrap();

                    // ... and append it to the file list.
                    file_list.append(&mut subfolder_files_path);
                }
            }
        }

        // In case of reading error, report it.
        Err(_) => return Err(ErrorKind::IOReadFolder(current_path.to_path_buf()))?,
    }

    // Return the list of paths.
    Ok(file_list)
}

/// This is a modification of the normal "get_files_from_subdir" used to get a list with the path of
/// every table definition from the assembly kit. Well, from the folder you tell it to search.
#[allow(dead_code)]
pub fn get_assembly_kit_schemas(current_path: &Path) -> Result<Vec<PathBuf>> {

    // Create the list of files.
    let mut file_list: Vec<PathBuf> = vec![];

    // Get everything from the path we have.
    match read_dir(current_path) {

        // If we don't have any problems reading it...
        Ok(files_in_current_path) => {

            // For each thing in the current path...
            for file in files_in_current_path {

                // Get his path
                let file_path = file.unwrap().path().clone();

                // If it's a file and starts with "TWaD_", to the file_list it goes (except if it's one of those special files).
                if file_path.is_file() &&
                    file_path.file_stem().unwrap().to_str().unwrap().to_string().starts_with("TWaD_") &&
                    file_path.file_stem().unwrap().to_str().unwrap() != "TWaD_schema_validation" &&
                    file_path.file_stem().unwrap().to_str().unwrap() != "TWaD_relationships" &&
                    file_path.file_stem().unwrap().to_str().unwrap() != "TWaD_validation" &&
                    file_path.file_stem().unwrap().to_str().unwrap() != "TWaD_tables" &&
                    file_path.file_stem().unwrap().to_str().unwrap() != "TWaD_queries" {
                    file_list.push(file_path);
                }
            }
        }

        // In case of reading error, report it.
        Err(_) => return Err(ErrorKind::IOReadFolder(current_path.to_path_buf()))?,
    }

    // Sort the files alphabetically.
    file_list.sort();

    // Return the list of paths.
    Ok(file_list)
}

/// Get the current date and return it, as a decoded u32.
#[allow(dead_code)]
pub fn get_current_time() -> u32 {

    // Get the current time as an encoded i64.
    let mut creation_time = coding_helpers::encode_integer_i64(Utc::now().naive_utc().timestamp());

    // Truncate it, so we got just the four bytes we need.
    creation_time.truncate(4);

    // Decode it as an u32.
    coding_helpers::decode_integer_u32(&creation_time).unwrap()
}

/// Get the last modified date from a file and return it, as a decoded u32.
#[allow(dead_code)]
pub fn get_last_modified_time_from_file(file: &File) -> u32 {

    // Translate the SystemTime to DateTime<Utc>, so we can use it.
    let last_modified_time: DateTime<Utc> = DateTime::from(file.metadata().unwrap().modified().unwrap());

    // Get the current time as an encoded i64.
    let mut last_modified_time = coding_helpers::encode_integer_i64(last_modified_time.naive_utc().timestamp());

    // Truncate it, so we got just the four bytes we need.
    last_modified_time.truncate(4);

    // Decode it as an u32.
    coding_helpers::decode_integer_u32(&last_modified_time).unwrap()
}
