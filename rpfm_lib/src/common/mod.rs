//---------------------------------------------------------------------------//
// Copyright (c) 2017-2019 Ismael Gutiérrez González. All rights reserved.
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

use chrono::{Utc, DateTime};

use rpfm_error::{ErrorKind, Result};

use std::fs::{File, read_dir};
use std::path::{Path, PathBuf};

use crate::config::get_config_path;
use crate::{SETTINGS, SUPPORTED_GAMES};

pub mod decoder;
pub mod encoder;

// This tells the compiler to only compile these mods when testing. It's just to make sure 
// the encoders and decoders don't break between updates.
#[cfg(test)]
mod decoder_test;

#[cfg(test)]
mod encoder_test;

/// This function retuns a `Vec<PathBuf>` containing all the files in the provided folder.
#[allow(dead_code)]
pub fn get_files_from_subdir(current_path: &Path) -> Result<Vec<PathBuf>> {
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
                        else if file_path.is_dir() {
                            let mut subfolder_files_path = get_files_from_subdir(&file_path)?;
                            file_list.append(&mut subfolder_files_path);
                        }
                    }
                    Err(_) => return Err(ErrorKind::IOReadFile(current_path.to_path_buf()))?,
                }
            }
        }

        // In case of reading error, report it.
        Err(_) => return Err(ErrorKind::IOReadFolder(current_path.to_path_buf()))?,
    }

    // Return the list of paths.
    Ok(file_list)
}

/// This function gets the current date and return it, as a decoded u32.
#[allow(dead_code)]
pub fn get_current_time() -> i64 {
    Utc::now().naive_utc().timestamp()
}

/// This function gets the last modified date from a file and return it, as a decoded u32.
#[allow(dead_code)]
pub fn get_last_modified_time_from_file(file: &File) -> i64 {
    let last_modified_time: DateTime<Utc> = DateTime::from(file.metadata().unwrap().modified().unwrap());
    last_modified_time.naive_utc().timestamp()
}

/// This function gets the `/data` path of the game selected, straighoutta settings, if it's configured.
#[allow(dead_code)]
pub fn get_game_selected_data_path(game_selected: &str) -> Option<PathBuf> {
    if let Some(path) = SETTINGS.lock().unwrap().paths.get(game_selected) {
        if let Some(path) = path {
            Some(path.join(PathBuf::from("data")))
        }
        else { None }
    } else { None }
}


/// This function gets the `/assembly_kit` path of the game selected, if supported and it's configured.
#[allow(dead_code)]
pub fn get_game_selected_assembly_kit_path(game_selected: &str) -> Option<PathBuf> {
    if let Some(path) = SETTINGS.lock().unwrap().paths.get(game_selected) {
        if let Some(path) = path {
            Some(path.join(PathBuf::from("assembly_kit")))
        }
        else { None }
    } else { None }
}

/// This function gets the `/data/xxx.pack` paths of the PackFile with db tables of the game selected, straighoutta settings, if it's configured.
#[allow(dead_code)]
pub fn get_game_selected_db_pack_path(game_selected: &str) -> Option<Vec<PathBuf>> {

    let base_path = SETTINGS.lock().unwrap().paths[game_selected].clone()?;
    let db_packs = &SUPPORTED_GAMES[game_selected].db_packs;
    let mut db_paths = vec![];
    for pack in db_packs {
        let mut path = base_path.to_path_buf();
        path.push("data");
        path.push(pack);
        db_paths.push(path);
    } 
    Some(db_paths)
}

/// This function gets the `/data/xxx.pack` paths of the PackFile with the loc files of the game selected, straighoutta settings, if it's configured.
#[allow(dead_code)]
pub fn get_game_selected_loc_pack_path(game_selected: &str) -> Option<Vec<PathBuf>> {

    let base_path = SETTINGS.lock().unwrap().paths[game_selected].clone()?;
    let loc_packs = &SUPPORTED_GAMES[game_selected].loc_packs;
    let mut loc_paths = vec![];
    for pack in loc_packs {
        let mut path = base_path.to_path_buf();
        path.push("data");
        path.push(pack);
        loc_paths.push(path);
    } 
    Some(loc_paths)
}

/// This function gets a list of all the PackFiles in the `/data` folder of the game straighoutta settings, if it's configured.
#[allow(dead_code)]
pub fn get_game_selected_data_packfiles_paths(game_selected: &str) -> Option<Vec<PathBuf>> {

    let mut paths = vec![];
    let data_path = get_game_selected_data_path(game_selected)?;

    for path in get_files_from_subdir(&data_path).ok()?.iter() {
        match path.extension() {
            Some(extension) => if extension == "pack" { paths.push(path.to_path_buf()); }
            None => continue,
        }
    }

    paths.sort();
    Some(paths)
}

/// This function gets a list of all the PackFiles in the `content` folder of the game straighoutta settings, if it's configured.
#[allow(dead_code)]
pub fn get_game_selected_content_packfiles_paths(game_selected: &str) -> Option<Vec<PathBuf>> {

    let mut path = SETTINGS.lock().unwrap().paths[game_selected].clone()?;
    let id = SUPPORTED_GAMES[game_selected].steam_id?.to_string();

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

/// This function gets the `/rpfm_path/pak_files/xxx.pak` path of the Game Selected, if it has one.
#[allow(dead_code)]
pub fn get_game_selected_pak_file(game_selected: &str) -> Result<PathBuf> {

    if let Some(pak_file) = &SUPPORTED_GAMES[game_selected].pak_file {
        let mut base_path = get_config_path()?;
        base_path.push("pak_files");
        base_path.push(pak_file);

        if base_path.is_file() { Ok(base_path) }
        else { Err(ErrorKind::IOFileNotFound)? }
    }
    else { Err(ErrorKind::PAKFileNotSupportedForThisGame)? }
}
