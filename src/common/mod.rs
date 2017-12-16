// In this file are all the helper functions used by the code (no GTK here)
// As we may or may not use them, all functions here should have the "#[allow(dead_code)]"
// var set, so the compiler doesn't spam us every time we try to compile.

use std::string::String;
use std::path::PathBuf;

use std::fs;
use std::path::Path;

pub mod coding_helpers;

/// This function takes a PathBuf and turn it into a Vec<String>.
/// Useful for managing paths more easily.
#[allow(dead_code)]
pub fn path_to_vec_strings(path: PathBuf) -> Vec<String> {
    let mut path_processed: Vec<String> = vec![];

    for i in path.iter() {
        path_processed.push(i.to_str().unwrap().to_string());
    }

    path_processed
}

/// This function takes a Vec<String> and concatenate them into one String.
/// Useful for managing paths.
#[allow(dead_code)]
pub fn vec_strings_to_path_string(vec_strings: Vec<String>) -> String {
    let mut path_processed: String = String::new();

    for (i, j) in vec_strings.iter().enumerate() {
        path_processed.push_str(j);
        if (i + 1) < (vec_strings.len()) {
            path_processed.push_str("\\");
        }
    }
    path_processed
}
/// This function takes a &Path and returns a Vec<PathBuf> with the paths of every file under the
/// original &Path.
#[allow(dead_code)]
pub fn get_files_from_subdir(current_path: &Path) -> Vec<PathBuf> {

    let mut file_list: Vec<PathBuf> = vec![];

    // For every file in this folder
    for file in fs::read_dir(current_path).unwrap() {

        // Get his path
        let file_path = file.unwrap().path().clone();

        // If it's a file, to the file_list it goes
        if file_path.is_file() {
            file_list.push(file_path);
        }

        // If it's a folder, get all the files from it and his subfolders recursively
        else if file_path.is_dir() {
            let mut subfolder_files_path = get_files_from_subdir(&file_path);
            file_list.append(&mut subfolder_files_path);
        }
    }

    // Return the list of paths
    file_list
}