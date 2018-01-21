// In this file are all the helper functions used by the code (no GTK here)
// As we may or may not use them, all functions here should have the "#[allow(dead_code)]"
// var set, so the compiler doesn't spam us every time we try to compile.

use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::io::Error;

use packfile::packfile::PackFile;

pub mod coding_helpers;

/// This enum has the different types of selected items in a TreeView.
#[derive(Clone, Debug)]
pub enum TreePathType {
    File((Vec<String>, usize)),
    Folder(Vec<String>),
    PackFile,
    None,
}

/// This function checks if the PackedFile at the given TreePath is a file or a folder. Please note
/// that the tree_path NEEDS TO BE COMPLETE (including PackFile's name) for the function to work
/// properly.
#[allow(dead_code)]
pub fn get_type_of_selected_tree_path(
    tree_path: &Vec<String>,
    pack_file_decoded: &PackFile
) -> TreePathType {

    let mut tree_path = tree_path.clone();

    // First, we check if we even have a TreePath to work with.
    if tree_path.is_empty() {
        return TreePathType::None;
    }
    // Then we check if the path is just the PackFile.
    else if tree_path.len() == 1 && tree_path[0] == pack_file_decoded.pack_file_extra_data.file_name {
        return TreePathType::PackFile
    }

    // If is not a PackFile, we remove his first field, as our PackedFiles's paths don't have it.
    else {
        tree_path.reverse();
        tree_path.pop();
        tree_path.reverse();

        // Now we check if it's a file or a folder.
        let mut is_a_file = false;
        let mut index = 0;
        for i in &pack_file_decoded.pack_file_data.packed_files {
            if i.packed_file_path == tree_path {
                is_a_file = true;
                break;
            }
            index += 1;
        }

        // If is a file, we return it.
        if is_a_file {
            return TreePathType::File((tree_path, index))

        }

        // If it isn't a file, we check if it's a folder.
        else {
            for i in &pack_file_decoded.pack_file_data.packed_files  {
                if i.packed_file_path.starts_with(&tree_path) && i.packed_file_path.len() > tree_path.len() {
                    return TreePathType::Folder(tree_path)
                }
            }
        }
    }

    // If we reach this, the tree_path we provided does not exist in the tree_view.
    return TreePathType::None
}

/// This function takes a &Path and returns a Vec<PathBuf> with the paths of every file under the
/// original &Path.
#[allow(dead_code)]
pub fn get_files_from_subdir(current_path: &Path) -> Result<Vec<PathBuf>, Error> {

    let mut file_list: Vec<PathBuf> = vec![];

    // For every file in this folder
    match fs::read_dir(current_path) {
        Ok(files_in_current_path) => {
            for file in files_in_current_path {

                // Get his path
                let file_path = file.unwrap().path().clone();

                // If it's a file, to the file_list it goes
                if file_path.is_file() {
                    file_list.push(file_path);
                }

                // If it's a folder, get all the files from it and his subfolders recursively
                else if file_path.is_dir() {
                    let mut subfolder_files_path = get_files_from_subdir(&file_path).unwrap();
                    file_list.append(&mut subfolder_files_path);
                }
            }
        }
        Err(error) => return Err(error),
    }

    // Return the list of paths
    Ok(file_list)
}

/// This function takes a &Path and returns a Vec<PathBuf> with the paths of every file under the
/// original &Path. This is a modification of the normal "get_files_from_subdir" where we only get
/// the files in the current folder and with a special beginning.
#[allow(dead_code)]
pub fn get_assembly_kit_schemas(current_path: &Path) -> Result<Vec<PathBuf>, Error> {

    let mut file_list: Vec<PathBuf> = vec![];

    // For every file in this folder
    match fs::read_dir(current_path) {
        Ok(files_in_current_path) => {
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
        Err(error) => return Err(error),
    }
    // Return the list of paths ordered alphabetically
    file_list.sort();
    Ok(file_list)
}