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

use std::fs::{DirBuilder, File, read_dir};
use std::io::{BufReader, Read};
use std::path::{Path, PathBuf};

use crate::template;
use crate::schema;
use crate::config::get_config_path;
use crate::games::{InstallType, KEY_TROY};
use crate::GAME_SELECTED;
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

/// This function gets the last modified date from a file and return it, as a decoded u32.
#[allow(dead_code)]
pub fn get_last_modified_time_from_file(file: &File) -> i64 {
    let last_modified_time: DateTime<Utc> = DateTime::from(file.metadata().unwrap().modified().unwrap());
    last_modified_time.naive_utc().timestamp()
}

/// This function gets the oldest modified file in a folder and return it.
#[allow(dead_code)]
pub fn get_oldest_file_in_folder(current_path: &Path) -> Result<Option<PathBuf>> {
    let mut files = get_files_from_subdir(current_path)?;
    files.sort();
    files.sort_by(|a, b| {
        let a = File::open(a).unwrap();
        let b = File::open(b).unwrap();
        get_last_modified_time_from_file(&a).cmp(&get_last_modified_time_from_file(&b))
    });

    Ok(files.get(0).cloned())
}

/// This function gets the files in a folder sorted from newest to oldest.
#[allow(dead_code)]
pub fn get_files_in_folder_from_newest_to_oldest(current_path: &Path) -> Result<Vec<PathBuf>> {
    let mut files = get_files_from_subdir(current_path)?;
    files.sort();
    files.sort_by(|a, b| {
        let a = File::open(a).unwrap();
        let b = File::open(b).unwrap();
        get_last_modified_time_from_file(&b).cmp(&get_last_modified_time_from_file(&a))
    });

    Ok(files)
}

/// This function gets the `/data` path of the game selected, straighoutta settings, if it's configured.
#[allow(dead_code)]
pub fn get_game_selected_data_path() -> Option<PathBuf> {
    let game_selected: &str = &*GAME_SELECTED.read().unwrap();
    if let Some(Some(path)) = SETTINGS.read().unwrap().paths.get(game_selected) {
        Some(path.join(PathBuf::from("data")))
    } else { None }
}

/// This function gets the `/assembly_kit` path of the game selected, if supported and it's configured.
#[allow(dead_code)]
pub fn get_game_selected_assembly_kit_path() -> Option<PathBuf> {
    let game_selected: &str = &*GAME_SELECTED.read().unwrap();
    if let Some(Some(path)) = SETTINGS.read().unwrap().paths.get(game_selected) {
        Some(path.join(PathBuf::from("assembly_kit")))
    } else { None }
}

/// This function gets the `/data/xxx.pack` paths of the PackFile with db tables of the game selected, straighoutta settings, if it's configured.
#[allow(dead_code)]
pub fn get_game_selected_db_pack_path() -> Option<Vec<PathBuf>> {
    let game_selected: &str = &*GAME_SELECTED.read().unwrap();
    let base_path = SETTINGS.read().unwrap().paths[game_selected].clone()?;
    let db_packs = &SUPPORTED_GAMES.get(game_selected)?.db_packs;
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
pub fn get_game_selected_loc_pack_path() -> Option<Vec<PathBuf>> {
    let game_selected: &str = &*GAME_SELECTED.read().unwrap();
    let base_path = SETTINGS.read().unwrap().paths[game_selected].clone()?;
    let loc_packs = &SUPPORTED_GAMES.get(game_selected)?.loc_packs;
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

/// This function gets a list of all the PackFiles in the `content` folder of the game straighoutta settings, if it's configured.
#[allow(dead_code)]
pub fn get_game_selected_content_packfiles_paths() -> Option<Vec<PathBuf>> {
    let game_selected: &str = &*GAME_SELECTED.read().unwrap();
    let path = match get_game_selected_install_type().ok()? {
        InstallType::Steam(steam_id) => {
            let mut path = SETTINGS.read().unwrap().paths[game_selected].clone()?;
            path.pop();
            path.pop();
            path.push("workshop");
            path.push("content");
            path.push(steam_id.to_string());
            path
        }
        InstallType::Epic => {
            let mut path = SETTINGS.read().unwrap().paths[game_selected].clone()?;
            path.push("mods");
            path
        }
        InstallType::Wargaming => return None,
    };

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
pub fn get_game_selected_pak_file() -> Result<PathBuf> {
    let game_selected: &str = &*GAME_SELECTED.read().unwrap();
    if let Some(pak_file) = &SUPPORTED_GAMES.get(game_selected).ok_or_else(|| Error::from(ErrorKind::GameNotSupported) )?.pak_file {
        let mut base_path = get_config_path()?;
        base_path.push("pak_files");
        base_path.push(pak_file);

        if base_path.is_file() { Ok(base_path) }
        else { Err(ErrorKind::IOFileNotFound.into()) }
    }
    else { Err(ErrorKind::PAKFileNotSupportedForThisGame.into()) }
}

/// This function gets the `/templates/definitions` path of the game selected, if they exists, and if it's custom or not.
#[allow(dead_code)]
pub fn get_game_selected_template_definitions_paths() -> Option<Vec<(bool, PathBuf)>> {
    let definitions_path = get_template_definitions_path().ok()?;

    let mut paths = vec![];
    for path in get_files_from_subdir(&definitions_path).ok()?.iter() {
        match path.extension() {
            Some(extension) => if extension == "json" { paths.push((false, path.to_path_buf())); }
            None => continue,
        }
    }

    if let Ok(definitions_path) = get_custom_template_definitions_path() {
        if let Ok(json_paths) = get_files_from_subdir(&definitions_path) {
            for path in &json_paths {
                match path.extension() {
                    Some(extension) => if extension == "json" { paths.push((true, path.to_path_buf())); }
                    None => continue,
                }
            }
        }
    }
    Some(paths)
}

/// This function gets the `/templates/assets` path of the game selected, if they exists.
#[allow(dead_code)]
pub fn get_game_selected_template_assets_paths() -> Option<Vec<PathBuf>> {
    let assets_path = get_template_assets_path().ok()?;

    let mut paths = vec![];
    for path in get_files_from_subdir(&assets_path).ok()?.iter() {
        paths.push(path.to_path_buf());
    }
    Some(paths)
}

/// This function returns the template assets path.
#[allow(dead_code)]
pub fn get_template_base_path() -> Result<PathBuf> {
    Ok(get_config_path()?.join(template::TEMPLATE_FOLDER))
}

/// This function returns the template definition path.
#[allow(dead_code)]
pub fn get_template_definitions_path() -> Result<PathBuf> {
    let game_selected: &str = &*GAME_SELECTED.read().unwrap();
    Ok(get_config_path()?.join(template::TEMPLATE_FOLDER.to_owned() + "/" + game_selected + "/" + template::DEFINITIONS_FOLDER))
}

/// This function returns the template assets path.
#[allow(dead_code)]
pub fn get_template_assets_path() -> Result<PathBuf> {
    let game_selected: &str = &*GAME_SELECTED.read().unwrap();
    Ok(get_config_path()?.join(template::TEMPLATE_FOLDER.to_owned() + "/" + game_selected + "/" + template::ASSETS_FOLDER))
}

/// This function returns the custom template definition path.
#[allow(dead_code)]
pub fn get_custom_template_definitions_path() -> Result<PathBuf> {
    let game_selected: &str = &*GAME_SELECTED.read().unwrap();
    Ok(get_config_path()?.join(template::CUSTOM_TEMPLATE_FOLDER.to_owned() + "/" + game_selected + "/" + template::DEFINITIONS_FOLDER))
}

/// This function returns the custom template assets path.
#[allow(dead_code)]
pub fn get_custom_template_assets_path() -> Result<PathBuf> {
    let game_selected: &str = &*GAME_SELECTED.read().unwrap();
    Ok(get_config_path()?.join(template::CUSTOM_TEMPLATE_FOLDER.to_owned() + "/" + game_selected + "/" + template::ASSETS_FOLDER))
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

/// This function returns the assembly kit raw data path, or an error if the game selected doesn't have a known path.
pub fn get_assembly_kit_db_tables_path() -> Result<PathBuf> {
    let version = SUPPORTED_GAMES.get(&**GAME_SELECTED.read().unwrap()).unwrap().raw_db_version;
    match version {

        // Post-Shogun 2 games.
        2 => {
            let mut path = SETTINGS.read().unwrap().paths[&**GAME_SELECTED.read().unwrap()].clone().unwrap();
            path.push("assembly_kit");
            path.push("raw_data");
            path.push("db");
            Ok(path)
        }

        // Shogun 2/Older games
        _ => Err(ErrorKind::AssemblyKitUnsupportedVersion(version).into())
    }
}

/// This function returns the install type corresponding to the current game selected.
#[allow(dead_code)]
pub fn get_game_selected_install_type() -> Result<InstallType> {
    let game_selected: &str = &*GAME_SELECTED.read().unwrap();
    let supported_install_types = &SUPPORTED_GAMES.get(game_selected).unwrap().install_type;
    if supported_install_types.len() == 1 {
        Ok(supported_install_types[0].clone())
    }
    else {
        Err(ErrorKind::NoInstallTypeForGame.into())
    }
}

/// This function gets the destination folder for MyMod packs.
#[allow(dead_code)]
pub fn get_mymod_install_path() -> Option<PathBuf> {
    let game_selected: &str = &*GAME_SELECTED.read().unwrap();
    match get_game_selected_install_type().ok()? {
        InstallType::Steam(_) => {
            let mut path = SETTINGS.read().unwrap().paths[game_selected].clone()?;
            path.push("data");
            Some(path)
        }
        InstallType::Epic => {
            let mut path = SETTINGS.read().unwrap().paths[game_selected].clone()?;
            path.push("mods");
            path.push("mymods");

            // Make sure the folder exists.
            DirBuilder::new().recursive(true).create(&path).ok()?;
            Some(path)
        }
        InstallType::Wargaming => None,
    }
}

/// This function gets the version number of the exe for the current GameSelected, if it exists.
#[allow(dead_code)]
pub fn get_game_selected_exe_version_number() -> Result<u32> {
    let game_selected: &str = &*GAME_SELECTED.read().unwrap();
    match game_selected {
        KEY_TROY => {
            let mut path = SETTINGS.read().unwrap().paths[game_selected].clone().ok_or_else(|| Error::from(ErrorKind::GameNotSupported))?;
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
