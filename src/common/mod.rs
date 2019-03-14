//---------------------------------------------------------------------------//
// Copyright (c) 2017-2019 Ismael Gutiérrez González. All rights reserved.
// 
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
// 
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

// In this file are all the "Generic" helper functions used by RPFM (no UI code here).
// As we may or may not use them, all functions here should have the "#[allow(dead_code)]"
// var set, so the compiler doesn't spam us every time we try to compile.

use chrono::{Utc, DateTime};
use serde_derive::{Serialize, Deserialize};

use std::fs::{File, read_dir};
use std::path::{Path, PathBuf};

use crate::SUPPORTED_GAMES;
use crate::GAME_SELECTED;
use crate::RPFM_PATH;
use crate::SETTINGS;
use crate::error::{ErrorKind, Result};
use crate::packfile::PackFile;

pub mod coding_helpers;
pub mod communications;

// This tells the compiler to only compile this mod when testing. It's just to make sure the "coders" don't break.
#[cfg(test)]
pub mod tests;

/// This const is the standard message in case of message deserializing error. If this happens, crash the program and send a report to Sentry.
pub const THREADS_MESSAGE_ERROR: &str = "Error in thread messages system.";

/// This const is the standard message in case of message communication error. If this happens, crash the program and send a report to Sentry.
pub const THREADS_COMMUNICATION_ERROR: &str = "Error in thread communication system.";

/// This enum has the different types of selected items in a TreeView. File and Folder have their tree_path without
/// the mod's name.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum TreePathType {
    File(Vec<String>),
    Folder(Vec<String>),
    PackFile,
    None,
}

/// Custom implementation of "PartialEq" for "TreePathType", so we don't need to match each time while
/// want to compare two TreePathType.
impl PartialEq for TreePathType {
    fn eq(&self, other: &TreePathType) -> bool {
        match (self, other) {
            (&TreePathType::File(_), &TreePathType::File(_)) |
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
    if tree_path.is_empty() { TreePathType::None }

    // If the path is just the PackFile's name, it's the PackFile.
    else if tree_path.len() == 1 { TreePathType::PackFile }

    // If is not a PackFile...
    else {

        // We remove his first field, as our PackedFiles's paths don't have it.
        tree_path.remove(0);

        // Now we check if it's a file or a folder.
        let mut is_a_file = false;

        for i in &pack_file_decoded.packed_files {
            if i.path == tree_path {
                is_a_file = true;
                break;
            }
        }

        // If is a file, we return it.
        if is_a_file { TreePathType::File(tree_path) }

        // Otherwise, we assume it's a folder. This is not bulletproof so FIXME: find a way to make this more solid.
        // FIXME: This is confirmed to be broken for the situation where there is a folder and a file with the same name
        // in the same folder.
        else {

            // We check if the folder actually exists in our PackFile.
            let is_a_folder = pack_file_decoded.folder_exists(&tree_path);

            // If it exists, we return it as a folder.
            if is_a_folder { TreePathType::Folder(tree_path) }

            // Otherwise, it's a None.
            else { TreePathType::None }
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
/// Version 0 means Empire/Nappy format. Version 1 or 2 is everything after them.
#[allow(dead_code)]
pub fn get_raw_definitions(current_path: &Path, version: i16) -> Result<Vec<PathBuf>> {

    let mut file_list: Vec<PathBuf> = vec![];
    match read_dir(current_path) {

        // If we don't have any problems reading it...
        Ok(files_in_current_path) => {
            for file in files_in_current_path {
                let file_path = file.unwrap().path().clone();

                // If it's a file and starts with "TWaD_", to the file_list it goes (except if it's one of those special files).
                if version == 1 || version == 2 {
                    if file_path.is_file() &&
                        file_path.file_stem().unwrap().to_str().unwrap().to_string().starts_with("TWaD_") &&
                        !file_path.file_stem().unwrap().to_str().unwrap().to_string().starts_with("TWaD_TExc") &&
                        file_path.file_stem().unwrap().to_str().unwrap() != "TWaD_schema_validation" &&
                        file_path.file_stem().unwrap().to_str().unwrap() != "TWaD_relationships" &&
                        file_path.file_stem().unwrap().to_str().unwrap() != "TWaD_validation" &&
                        file_path.file_stem().unwrap().to_str().unwrap() != "TWaD_tables" &&
                        file_path.file_stem().unwrap().to_str().unwrap() != "TWaD_queries" {
                        file_list.push(file_path);
                    }
                }

                // In this case, we just catch all the xsd files on the folder.
                else if version == 0 {
                    if file_path.is_file() &&
                        file_path.file_stem().unwrap().to_str().unwrap().to_string().ends_with(".xsd") {
                        file_list.push(file_path);
                    }   
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

/// This is a modification of the normal "get_files_from_subdir" used to get a list with the path of
/// every raw table data from the assembly kit. Well, from the folder you tell it to search.
/// Version 0 means Empire/Nappy format. Version 1 or 2 is everything after them.
#[allow(dead_code)]
pub fn get_raw_data(current_path: &Path, version: i16) -> Result<Vec<PathBuf>> {

    let mut file_list: Vec<PathBuf> = vec![];
    match read_dir(current_path) {

        // If we don't have any problems reading it...
        Ok(files_in_current_path) => {
            for file in files_in_current_path {
                let file_path = file.unwrap().path().clone();

                // If it's a file and it doesn't start with "TWaD_", to the file_list it goes.
                if version == 1 || version == 2 {
                    if file_path.is_file() && !file_path.file_stem().unwrap().to_str().unwrap().to_string().starts_with("TWaD_") {
                        file_list.push(file_path);
                    }
                }

                // In this case, if it's an xml, to the file_list it goes.
                else if version == 0 {
                    if file_path.is_file() && !file_path.file_stem().unwrap().to_str().unwrap().to_string().ends_with(".xml") {
                        file_list.push(file_path);
                    }
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
pub fn get_current_time() -> i64 {
    Utc::now().naive_utc().timestamp()
}

/// Get the last modified date from a file and return it, as a decoded u32.
#[allow(dead_code)]
pub fn get_last_modified_time_from_file(file: &File) -> i64 {
    let last_modified_time: DateTime<Utc> = DateTime::from(file.metadata().unwrap().modified().unwrap());
    last_modified_time.naive_utc().timestamp()
}

/// Get the `/data` path of the game selected, straighoutta settings, if it's configured.
#[allow(dead_code)]
pub fn get_game_selected_data_path() -> Option<PathBuf> {
    let mut path = SETTINGS.lock().unwrap().paths[&**GAME_SELECTED.lock().unwrap()].clone()?;
    path.push("data");
    Some(path)
}

/// Get the `/data/xxx.pack` path of the PackFile with db tables of the game selected, straighoutta settings, if it's configured.
#[allow(dead_code)]
pub fn get_game_selected_db_pack_path() -> Option<Vec<PathBuf>> {

    let base_path = SETTINGS.lock().unwrap().paths[&**GAME_SELECTED.lock().unwrap()].clone()?;
    let db_packs = &SUPPORTED_GAMES[&**GAME_SELECTED.lock().unwrap()].db_packs;
    let mut db_paths = vec![];
    for pack in db_packs {
        let mut path = base_path.to_path_buf();
        path.push("data");
        path.push(pack);
        db_paths.push(path);
    } 
    Some(db_paths)
}

/// Get the `/data/xxx.pack` path of the PackFile with the english loc files of the game selected, straighoutta settings, if it's configured.
#[allow(dead_code)]
pub fn get_game_selected_loc_pack_path() -> Option<Vec<PathBuf>> {

    let base_path = SETTINGS.lock().unwrap().paths[&**GAME_SELECTED.lock().unwrap()].clone()?;
    let loc_packs = &SUPPORTED_GAMES[&**GAME_SELECTED.lock().unwrap()].loc_packs;
    let mut loc_paths = vec![];
    for pack in loc_packs {
        let mut path = base_path.to_path_buf();
        path.push("data");
        path.push(pack);
        loc_paths.push(path);
    } 
    Some(loc_paths)
}

/// Get a list of all the PackFiles in the `/data` folder of the game straighoutta settings, if it's configured.
#[allow(dead_code)]
pub fn get_game_selected_data_packfiles_paths() -> Option<Vec<PathBuf>> {

    let mut paths = vec![];
    let data_path = get_game_selected_data_path()?;

    for path in get_files_from_subdir(&data_path).ok()?.iter() {
        match path.extension() {
            Some(extension) => if extension == "pack" { paths.push(path.to_path_buf()); }
            None => continue,
        }
    }

    paths.sort();
    Some(paths)
}

/// Get a list of all the PackFiles in the `content` folder of the game straighoutta settings, if it's configured.
#[allow(dead_code)]
pub fn get_game_selected_content_packfiles_paths() -> Option<Vec<PathBuf>> {

    let mut path = SETTINGS.lock().unwrap().paths[&**GAME_SELECTED.lock().unwrap()].clone()?;
    let id = SUPPORTED_GAMES[&**GAME_SELECTED.lock().unwrap()].steam_id?.to_string();

    path.pop();
    path.pop();
    path.push("workshop");
    path.push("content");
    path.push(id);

    let mut paths = vec![];

    for path in get_files_from_subdir(&path).ok()?.iter() {
        match path.extension() {
            Some(extension) => if extension == "pack" { paths.push(path.to_path_buf()); }
            None => continue,
        }
    }

    paths.sort();
    Some(paths)
}

/// Get the `/rpfm_path/pak_files/xxx.pak` path of the Game Selected, if it has one.
#[allow(dead_code)]
pub fn get_game_selected_pak_file() -> Option<PathBuf> {

    if let Some(pak_file) = &SUPPORTED_GAMES[&**GAME_SELECTED.lock().unwrap()].pak_file {
        let mut base_path = RPFM_PATH.to_path_buf();
        base_path.push("pak_files");
        base_path.push(pak_file);

        if base_path.is_file() {
            return Some(base_path)
        }
    }
    None
}
