//---------------------------------------------------------------------------//
// Copyright (c) 2017-2019 Ismael Gutiérrez González. All rights reserved.
// 
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
// 
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

// In this file are all the functions that the UI needs to interact with the PackFile logic.
// As a rule, there should be no UI-related stuff in this module or his childrens.

use bincode::deserialize;

use std::fs::{File, DirBuilder};
use std::io::{Read, Write};
use std::path::PathBuf;
use std::io::BufReader;
use std::io::BufWriter;

use crate::SUPPORTED_GAMES;
use crate::GAME_SELECTED;
use crate::DEPENDENCY_DATABASE;
use crate::SCHEMA;
use crate::common::*;
use crate::error::{Error, ErrorKind, Result};
use crate::packfile::{PackFile, PFHVersion, PFHFileType, PathType};
use crate::packfile::packedfile::PackedFile;
use crate::packedfile::DecodedData;
use crate::packedfile::loc::Loc;
use crate::packedfile::db::DB;
use crate::packedfile::rigidmodel::RigidModel;

/*
--------------------------------------------------------
                PackFile-Related Functions
--------------------------------------------------------
*/

/// This function creates a new PackFile with the name received.
pub fn new_packfile(file_name: String, pfh_version: PFHVersion) -> PackFile {
    PackFile::new_with_name(file_name, pfh_version)
}

/// This function is used to open the PackFiles. It requires the path of the PackFile to open, and
/// it returns the PackFile decoded (if success) or an error message (if error).
pub fn open_packfile(pack_file_path: PathBuf, use_lazy_loading: bool) -> Result<PackFile> {

    // If the name doesn't end in ".pack", we don't open it. It works, but it'll break some things.
    if pack_file_path.file_name().unwrap().to_str().unwrap().ends_with(".pack") {
        PackFile::read(pack_file_path, use_lazy_loading)
    }

    // Otherwise, return an error.
    else { Err(ErrorKind::OpenPackFileInvalidExtension)? }
}

/// This function is a special open function, to get all the DB and LOC PackedFiles for a game, and a mod if that mode requires another mod.
/// It returns all the PackedFiles in a big Vec<PackedFile>.
pub fn load_dependency_packfiles(dependencies: &[String]) -> Vec<PackedFile> {

    // Create the empty list.
    let mut packed_files = vec![];

    // Get all the paths we need.
    let main_db_pack_paths = get_game_selected_db_pack_path();
    let main_loc_pack_paths = get_game_selected_loc_pack_path();

    let data_packs_paths = get_game_selected_data_packfiles_paths();
    let content_packs_paths = get_game_selected_content_packfiles_paths();

    // Get all the DB Tables from the main DB PackFiles, if it's configured.
    if let Some(paths) = main_db_pack_paths {
        for path in &paths {
            if let Ok(pack_file) = open_packfile(path.to_path_buf(), true) {

                // For each PackFile in the data.pack...
                for packed_file in pack_file.packed_files.iter() {

                    // If it's a DB file...
                    if !packed_file.path.is_empty() && packed_file.path.starts_with(&["db".to_owned()]) {

                        // Clone the PackedFile, and add it to the list.
                        let mut packed_file = packed_file.clone();
                        let _ = packed_file.load_data();
                        packed_files.push(packed_file);
                    }
                }
            }
        }
    }

    // Get all the Loc PackedFiles from the main Loc PackFiles, if it's configured.
    if let Some(paths) = main_loc_pack_paths {
        for path in &paths {
            if let Ok(pack_file) = open_packfile(path.to_path_buf(), true) {

                // For each PackFile in the data.pack...
                for packed_file in pack_file.packed_files.iter() {

                    // If it's a Loc file...
                    if !packed_file.path.is_empty() && packed_file.path.last().unwrap().ends_with(".loc") {

                        // Clone the PackedFile, and add it to the list.
                        let mut packed_file = packed_file.clone();
                        let _ = packed_file.load_data();
                        packed_files.push(packed_file);
                    }
                }
            }
        }
    }

    // Get all the DB and Loc files from any of the dependencies, searching in both, /data and /content.
    for packfile in dependencies {

        // If the dependency PackFile is in the data folder...
        if let Some(ref paths) = data_packs_paths {
            for path in paths {
                if path.file_name().unwrap().to_string_lossy().as_ref() == *packfile {
                    if let Ok(pack_file) = open_packfile(path.to_path_buf(), true) {

                        // For each PackFile in the data.pack...
                        for packed_file in pack_file.packed_files.iter() {

                            // If it's a DB file...
                            if !packed_file.path.is_empty() && packed_file.path.starts_with(&["db".to_owned()]) {

                                // Clone the PackedFile, and add it to the list.
                                let mut packed_file = packed_file.clone();
                                let _ = packed_file.load_data();
                                packed_files.push(packed_file);
                            }
                        }
                    }

                    // Get all the Loc PackedFiles from the main Loc PackFile, if it's configured.
                    if let Ok(pack_file) = open_packfile(path.to_path_buf(), true) {

                        // For each PackFile in the data.pack...
                        for packed_file in pack_file.packed_files.iter() {

                            // If it's a Loc file...
                            if !packed_file.path.is_empty() && packed_file.path.last().unwrap().ends_with(".loc") {

                                // Clone the PackedFile, and add it to the list.
                                let mut packed_file = packed_file.clone();
                                let _ = packed_file.load_data();
                                packed_files.push(packed_file);
                            }
                        }
                    }
                }
            } 
        }

        // If the dependency PackFile is in the content folder...
        if let Some(ref paths) = content_packs_paths {
            for path in paths {
                if path.file_name().unwrap().to_string_lossy().as_ref() == *packfile {

                    // Get all the DB Tables from the main DB PackFile, if it's configured.
                    if let Ok(pack_file) = open_packfile(path.to_path_buf(), true) {

                        // For each PackFile in the data.pack...
                        for packed_file in pack_file.packed_files.iter() {

                            // If it's a DB file...
                            if !packed_file.path.is_empty() && packed_file.path.starts_with(&["db".to_owned()]) {

                                // Clone the PackedFile and add it to the PackedFiles List.
                                let mut packed_file = packed_file.clone();
                                let _ = packed_file.load_data();
                                packed_files.push(packed_file);
                            }
                        }
                    }

                    // Get all the Loc PackedFiles from the main Loc PackFile, if it's configured.
                    if let Ok(pack_file) = open_packfile(path.to_path_buf(), true) {

                        // For each PackFile in the data.pack...
                        for packed_file in pack_file.packed_files.iter() {

                            // If it's a Loc file...
                            if !packed_file.path.is_empty() && packed_file.path.last().unwrap().ends_with(".loc") {

                                // Clone the PackedFile and add it to the PackedFiles List.
                                let mut packed_file = packed_file.clone();
                                let _ = packed_file.load_data();
                                packed_files.push(packed_file);
                            }
                        }
                    }
                }
            } 
        }
    }

    // Return the new PackedFiles list.
    packed_files
}

/// This function is a special open function, to get all the fake DB files from the PAK file of the Game Selected,
/// if it does has one.
pub fn load_fake_dependency_packfiles() -> Vec<DB> {

    // Create the empty list.
    let mut db_files = vec![];

    // Get all the paths we need.
    if let Some(pak_file) = get_game_selected_pak_file() {
        if let Ok(pak_file) = File::open(pak_file) {
            let mut pak_file = BufReader::new(pak_file);
            let mut data = vec![];
            if pak_file.read_to_end(&mut data).is_ok() {
                if let Ok(pak_file) = deserialize(&data) {
                    db_files = pak_file;
                }
            }
        }
    }

    // Return the fake DB Table list.
    db_files
}

/// This function is another special open function, to get all the PackedFiles from every CA PackFile of a game.
/// It returns a fake PackFile with them.
pub fn load_all_ca_packfiles() -> Result<PackFile> {

    // Create the fake PackFile.
    let pfh_version = SUPPORTED_GAMES.get(&**GAME_SELECTED.lock().unwrap()).unwrap().id;
    let mut pack_file = PackFile::new_with_name(GAME_SELECTED.lock().unwrap().to_owned(), pfh_version);

    // Get all the paths we need and open them one by one.
    let packs_paths = if let Some(paths) = get_game_selected_data_packfiles_paths() { paths } else { Err(ErrorKind::GamePathNotConfigured)? };
    let mut ca_pack_files = vec![];
    for path in packs_paths {
        ca_pack_files.push(PackFile::read(path, true)?);
    }

    // Get all the PackedFiles from each PackFile. First Boot type, then Release type, then Patch type, then Movie type.
    let mut boot_files = vec![];
    for ca_pack_file in &ca_pack_files {
        if let PFHFileType::Boot = ca_pack_file.pfh_file_type {
            ca_pack_file.packed_files.iter().for_each(|x| boot_files.push(x.clone()));
        }
    }
    
    let mut release_files = vec![];
    for ca_pack_file in &ca_pack_files {
        if let PFHFileType::Release = ca_pack_file.pfh_file_type {
            ca_pack_file.packed_files.iter().for_each(|x| release_files.push(x.clone()));
        }
    }

    let mut patch_files = vec![];
    for ca_pack_file in &ca_pack_files {
        if let PFHFileType::Patch = ca_pack_file.pfh_file_type {
            ca_pack_file.packed_files.iter().for_each(|x| patch_files.push(x.clone()));
        }
    }

    // This may load custom PackFiles. The only way to fix this is to read the manifest and checking if they are there, but I don't know if it's in all the games.
    // TODO: Make this only load CA PackFiles.
    let mut movie_files = vec![];
    for ca_pack_file in &ca_pack_files {
        if let PFHFileType::Movie = ca_pack_file.pfh_file_type {
            ca_pack_file.packed_files.iter().for_each(|x| movie_files.push(x.clone()));
        }
    }

    // The priority in case of collision is:
    // - Same Type: First to come is the valid one.
    // - Different Type: Last to come is the valid one.
    boot_files.sort_by_key(|x| x.path.to_vec());
    boot_files.dedup_by_key(|x| x.path.to_vec());

    release_files.sort_by_key(|x| x.path.to_vec());
    release_files.dedup_by_key(|x| x.path.to_vec());

    patch_files.sort_by_key(|x| x.path.to_vec());
    patch_files.dedup_by_key(|x| x.path.to_vec());

    movie_files.sort_by_key(|x| x.path.to_vec());
    movie_files.dedup_by_key(|x| x.path.to_vec());

    pack_file.packed_files.append(&mut movie_files);
    pack_file.packed_files.append(&mut patch_files);
    pack_file.packed_files.append(&mut release_files);
    pack_file.packed_files.append(&mut boot_files);

    pack_file.packed_files.sort_by_key(|x| x.path.to_vec());
    pack_file.packed_files.dedup_by_key(|x| x.path.to_vec());

    // Set it as type "Other(200)", so we can easely identify it as fake in other places.
    pack_file.pfh_file_type = PFHFileType::Other(200);

    // Return the new PackedFiles list.
    Ok(pack_file)
}

/// This function is used to take an open PackFile, encode it and save it into the disk. We return
/// a result with a message of success or error.
/// It requires:
/// - pack_file: a &mut pack_file::PackFile. It's the PackFile we are going to save.
/// - new_path: an Option<PathBuf> with the path were we are going to save the PackFile. None if we
///   are saving it in the same path it's when we opened it.
pub fn save_packfile(
    mut pack_file: &mut PackFile,
    new_path: Option<PathBuf>,
    is_editing_of_ca_packfiles_allowed: bool
) -> Result<()> {

    // If any of the problematic masks in the header is set or is one of CA's, return an error.
    if !pack_file.is_editable(is_editing_of_ca_packfiles_allowed) { return Err(ErrorKind::PackFileIsNonEditable)? }

    // If we receive a new path, update it. Otherwise, ensure the file actually exists on disk.
    if let Some(path) = new_path { pack_file.file_path = path; }
    else if !pack_file.file_path.is_file() { return Err(ErrorKind::PackFileIsNotAFile)? }
    
    // And we try to save it.
    PackFile::save(&mut pack_file)
}

/// This function is used to add a file to a PackFile, processing it and turning it into a PackedFile.
/// It returns a success or error message, depending on whether the file has been added, or not.
/// It requires:
/// - pack_file: a &mut pack_file::PackFile. It's the PackFile where we are going add the file.
/// - file_path: a PathBuf with the current path of the file.
/// - tree_path: a Vec<String> with the path in the TreeView where we are going to add the file.
pub fn add_file_to_packfile(
    pack_file: &mut PackFile,
    file_path: &PathBuf,
    tree_path: Vec<String>
) -> Result<()> {

    // If there is already a PackedFile in that path...
    if pack_file.packedfile_exists(&tree_path) {

        // Create the theorical path of the PackedFile.
        let mut theorical_path = tree_path.to_vec();
        theorical_path.insert(0, pack_file.file_path.file_name().unwrap().to_string_lossy().to_string());

        // Get the destination PackedFile.
        let packed_file = &mut pack_file.packed_files.iter_mut().find(|x| x.path == tree_path).ok_or_else(|| Error::from(ErrorKind::PackedFileNotFound))?;

        // We get the data and his size...
        let mut file = BufReader::new(File::open(&file_path)?);
        let mut data = vec![];
        file.read_to_end(&mut data)?;
        packed_file.set_data(data);

        // Change his last modified time.
        packed_file.timestamp = get_last_modified_time_from_file(&file.get_ref());

        // And then, return sucess.
        Ok(())
    }

    // Otherwise, we add it as a new PackedFile.
    else {

        // We get the data and his size...
        let mut file = BufReader::new(File::open(&file_path)?);
        let mut data = vec![];
        file.read_to_end(&mut data)?;

        // And then we make a PackedFile with it and save it.
        let packed_files = vec![PackedFile::read_from_vec(tree_path, get_last_modified_time_from_file(&file.get_ref()), false, data); 1];
        pack_file.add_packedfiles(packed_files);
        Ok(())
    }
}

/// This function is used to add one or many PackedFiles to a PackFile (from another PackFile).
/// It returns a success or error message, depending on whether the PackedFile has been added, or not.
/// It requires:
/// - pack_file_source: a &pack_file::PackFile. It's the PackFile from we are going to take the PackedFile.
/// - pack_file_destination: a &mut pack_file::PackFile. It's the Destination PackFile for the PackedFile.
/// - path_type: PathType to add to the PackFile.
pub fn add_packedfile_to_packfile(
    pack_file_source: &PackFile,
    pack_file_destination: &mut PackFile,
    path_type: &PathType,
) -> Result<Vec<PathType>> {

    // Keep the PathTypes added so we can return them to the UI easely.
    let mut path_types_added = vec![];
    match path_type {

        // If the path is a file...
        PathType::File(path) => {

            // Check if the PackedFile already exists in the destination.
            if pack_file_destination.packedfile_exists(path) {

                // Get the destination PackedFile. If it fails, CTD because it's a code problem.
                let packed_file = &mut pack_file_destination.packed_files.iter_mut().find(|x| &x.path == path).ok_or_else(|| Error::from(ErrorKind::PackedFileNotFound))?;
                packed_file.set_data(pack_file_source.packed_files.iter().find(|x| &x.path == path).ok_or_else(|| Error::from(ErrorKind::PackedFileNotFound))?.get_data()?);
            }

            // Otherwise...
            else {

                // We get the PackedFile, clone it and add it to our own PackFile.
                let mut packed_file = pack_file_source.packed_files.iter().find(|x| &x.path == path).ok_or_else(|| Error::from(ErrorKind::PackedFileNotFound))?.clone();
                packed_file.load_data()?;
                pack_file_destination.add_packedfiles(vec![packed_file; 1]);
            }
            path_types_added.push(path_type.clone());
        }

        // If the path is a folder...
        PathType::Folder(ref path) => {

            // For each PackedFile inside the folder...
            for packed_file in pack_file_source.packed_files.iter() {

                // If it's one of the PackedFiles we want...
                if !packed_file.path.is_empty() && packed_file.path.starts_with(path) {

                    // Check if the PackedFile already exists in the destination.
                    if pack_file_destination.packedfile_exists(&packed_file.path) {

                        // Get the destination PackedFile.
                        let packed_file = &mut pack_file_destination.packed_files.iter_mut().find(|x| x.path == packed_file.path).ok_or_else(|| Error::from(ErrorKind::PackedFileNotFound))?;

                        // Then, we get his data.
                        let index = pack_file_source.packed_files.iter().position(|x| x.path == packed_file.path).unwrap();
                        packed_file.set_data(pack_file_source.packed_files[index].get_data()?);
                    }

                    // Otherwise...
                    else {

                        // We get the PackedFile, clone it and add it to our own PackFile.
                        let mut packed_file = pack_file_source.packed_files.iter().find(|x| x.path == packed_file.path).ok_or_else(|| Error::from(ErrorKind::PackedFileNotFound))?.clone();
                        packed_file.load_data()?;
                        pack_file_destination.add_packedfiles(vec![packed_file]);
                    }
                    path_types_added.push(PathType::File(packed_file.path.to_vec()));
                }
            }
        },

        // If the path is the PackFile...
        PathType::PackFile => {

            // For each PackedFile inside the folder...
            for packed_file in pack_file_source.packed_files.iter() {

                // Check if the PackedFile already exists in the destination.
                if pack_file_destination.packedfile_exists(&packed_file.path) {

                    // Get the destination PackedFile.
                    let packed_file = &mut pack_file_destination.packed_files.iter_mut().find(|x| x.path == packed_file.path).ok_or_else(|| Error::from(ErrorKind::PackedFileNotFound))?;

                    // Then, we get his data.
                    let index = pack_file_source.packed_files.iter().position(|x| x.path == packed_file.path).unwrap();
                    packed_file.set_data(pack_file_source.packed_files[index].get_data()?)
                }

                // Otherwise...
                else {

                    // We get the PackedFile.
                    let mut packed_file = pack_file_source.packed_files.iter().find(|x| x.path == packed_file.path).ok_or_else(|| Error::from(ErrorKind::PackedFileNotFound))?.clone();
                    packed_file.load_data()?;
                    pack_file_destination.add_packedfiles(vec![packed_file]);
                }
                path_types_added.push(PathType::File(packed_file.path.to_vec()));
            }
        },

        // In any other case, there is a problem somewhere. Otherwise, this is unreachable.
        _ => unreachable!()
    }
    Ok(path_types_added)
}

/// This function is used to delete a PackedFile or a group of PackedFiles of the provided types
/// from the PackFile. We just need the open PackFile and the PathTypes of the files/folders to delete.
pub fn delete_from_packfile(
    pack_file: &mut PackFile,
    item_types: &[PathType]
) -> Vec<PathType> {
    
    // First, we prepare the counters for the path types.
    let (mut file, mut folder, mut packfile, mut none) = (0, 0, 0, 0);

    // We need to "clean" the selected path list to ensure we don't pass stuff already deleted.
    let mut item_types_clean = vec![];
    for item_type_to_add in item_types {
        match item_type_to_add {
            PathType::File(ref path_to_add) => {
                let mut add_type = true;
                for item_type in item_types {
                    
                    // Skip the current file from checks.
                    if let PathType::File(ref path) = item_type {
                        if path == path_to_add { continue; }
                    }

                    // If the other one is a folder that contains it, dont add it.
                    else if let PathType::Folder(ref path) = item_type {
                        if path_to_add.starts_with(path) { 
                            add_type = false;
                            break;
                        }
                    }
                }
                if add_type { item_types_clean.push(item_type_to_add.clone()); }
            }

            PathType::Folder(ref path_to_add) => {
                let mut add_type = true;
                for item_type in item_types {

                    // If the other one is a folder that contains it, dont add it.
                    if let PathType::Folder(ref path) = item_type {
                        if path == path_to_add { continue; }
                        if path_to_add.starts_with(path) { 
                            add_type = false;
                            break;
                        }
                    }
                }
                if add_type { item_types_clean.push(item_type_to_add.clone()); }
            }

            // If we got the PackFile, remove everything.
            PathType::PackFile => {
                item_types_clean.clear();
                item_types_clean.push(item_type_to_add.clone());
                break;
            }
            PathType::None => unimplemented!(),
        }   
    }

    for item_type in &item_types_clean {
        match item_type {
            PathType::File(_) => file += 1,
            PathType::Folder(_) => folder += 1,
            PathType::PackFile => packfile += 1,
            PathType::None => none += 1,
        }
    }
    
    // Now we do some bitwise magic to get what type of selection combination we have.
    let mut contents: u8 = 0;
    if file != 0 { contents |= 1; } 
    if folder != 0 { contents |= 2; } 
    if packfile != 0 { contents |= 4; } 
    if none != 0 { contents |= 8; } 
    match contents {

        // Any combination of files and folders.
        1 | 2 | 3 => {
            for item_type in &item_types_clean {
                match item_type {
                    PathType::File(path) => {
   
                        let index = pack_file.packed_files.iter().position(|x| &x.path == path).unwrap();
                        pack_file.remove_packedfile(index);
                    },

                    PathType::Folder(path) => {
                    
                        // We create a vector to store the indexes of the files we are going to delete.
                        let mut indexes = vec![];
                        for (index, packed_file) in pack_file.packed_files.iter().enumerate() {

                            // If the PackedFile it's in our folder, add his index to the indexes list.
                            if !packed_file.path.is_empty() && packed_file.path.starts_with(&path) {
                                indexes.push(index);
                            }
                        }

                        // For each PackedFile we want to remove (in reverse), we remove it individually.
                        indexes.iter().rev().for_each(|index| pack_file.remove_packedfile(*index));
                    },

                    _ => unreachable!(),
                } 
            }
        },

        // If the PackFile is selected, get it just extract the PackFile and everything will get extracted with it.
        4 | 5 | 6 | 7 => pack_file.remove_all_packedfiles(),

        // No paths selected, none selected, invalid path selected, or invalid value. 
        0 | 8..=255 => {},
    }

    // Return the TreePathType list so the UI can delete them.
    item_types_clean
}

/// This function is used to extract a PackedFile or a folder from the PackFile.
/// It requires:
/// - pack_file: the PackFile from where we want to extract the PackedFile.
/// - item_types: the PathType of the PackedFiles we want to extract.
/// - extracted_path: the destination path of the file we want to extract.
///
/// NOTE: By COMPLETE I mean with the PackFile's name included.
pub fn extract_from_packfile(
    pack_file: &PackFile,
    item_types: &[PathType],
    extracted_path: &PathBuf,
) -> Result<String> {

    // These variables are here to keep track of what we have extracted and what files failed.
    let (mut file, mut folder, mut packfile, mut none) = (0, 0, 0, 0);
    let mut files_extracted = 0;
    let mut error_files = vec![];

    // We need to "clean" the selected path list to ensure we don't pass stuff already deleted.
    let mut item_types_clean = vec![];
    for item_type_to_add in item_types {
        match item_type_to_add {
            PathType::File(ref path_to_add) => {
                let mut add_type = true;
                for item_type in item_types {
                    
                    // Skip the current file from checks.
                    if let PathType::File(ref path) = item_type {
                        if path == path_to_add { continue; }
                    }

                    // If the other one is a folder that contains it, dont add it.
                    else if let PathType::Folder(ref path) = item_type {
                        if path_to_add.starts_with(path) { 
                            add_type = false;
                            break;
                        }
                    }
                }
                if add_type { item_types_clean.push(item_type_to_add.clone()); }
            }

            PathType::Folder(ref path_to_add) => {
                let mut add_type = true;
                for item_type in item_types {

                    // If the other one is a folder that contains it, dont add it.
                    if let PathType::Folder(ref path) = item_type {
                        if path == path_to_add { continue; }
                        if path_to_add.starts_with(path) { 
                            add_type = false;
                            break;
                        }
                    }
                }
                if add_type { item_types_clean.push(item_type_to_add.clone()); }
            }

            // If we got the PackFile, remove everything.
            PathType::PackFile => {
                item_types_clean.clear();
                item_types_clean.push(item_type_to_add.clone());
                break;
            }
            PathType::None => unimplemented!(),
        }   
    }

    for item_type in &item_types_clean {
        match item_type {
            PathType::File(_) => file += 1,
            PathType::Folder(_) => folder += 1,
            PathType::PackFile => packfile += 1,
            PathType::None => none += 1,
        }
    }

    // Now we do some bitwise magic to get what type of selection combination we have.
    let mut contents: u8 = 0;
    if file != 0 { contents |= 1; } 
    if folder != 0 { contents |= 2; } 
    if packfile != 0 { contents |= 4; } 
    if none != 0 { contents |= 8; } 
    match contents {

        // Any combination of files and folders.
        1 | 2 | 3 => {

            // For folders we check each PackedFile to see if it starts with the folder's path (it's in the folder).
            // There should be no duplicates here thanks to the filters from before.
            for item_type in &item_types_clean {
                match item_type {
                    PathType::File(path) => {
   
                        // We remove everything from his path up to the folder we want to extract (not included).
                        let packed_file = pack_file.packed_files.iter().find(|x| &x.path == path).ok_or_else(|| Error::from(ErrorKind::PackedFileNotFound))?;
                        let mut additional_path = packed_file.path.to_vec();
                        let file_name = additional_path.pop().unwrap();

                        // Get the destination path of our file, without the file at the end, and create his folder.
                        let mut current_path = extracted_path.clone().join(additional_path.iter().collect::<PathBuf>());
                        DirBuilder::new().recursive(true).create(&current_path)?;

                        // Finish the path and save the file.
                        current_path.push(&file_name);
                        let mut file = BufWriter::new(File::create(&current_path)?);
                        match file.write_all(&packed_file.get_data()?){
                            Ok(_) => files_extracted += 1,
                            Err(_) => error_files.push(format!("{:?}", current_path)),
                        }
                    },

                    PathType::Folder(path) => {
                    
                        for packed_file in &pack_file.packed_files {
                            if !path.is_empty() && packed_file.path.starts_with(&path) {
                               
                                // We remove everything from his path up to the folder we want to extract (not included).
                                let mut additional_path = packed_file.path.to_vec();
                                let file_name = additional_path.pop().unwrap();

                                // Get the destination path of our file, without the file at the end, and create his folder.
                                let mut current_path = extracted_path.clone().join(additional_path.iter().collect::<PathBuf>());
                                DirBuilder::new().recursive(true).create(&current_path)?;

                                // Finish the path and save the file.
                                current_path.push(&file_name);
                                let mut file = BufWriter::new(File::create(&current_path)?);
                                match file.write_all(&packed_file.get_data()?){
                                    Ok(_) => files_extracted += 1,
                                    Err(_) => error_files.push(format!("{:?}", current_path)),
                                }
                            }
                        }
                    },

                    _ => unreachable!(),
                } 
            }            
        },

        // If the PackFile is selected, get it just extract the PackFile and everything will get extracted with it.
        4 | 5 | 6 | 7 => {

            // For each PackedFile we have, just extracted in the folder we got, under the PackFile's folder.
            for packed_file in &pack_file.packed_files {

                // We remove everything from his path up to the folder we want to extract (not included).
                let mut additional_path = packed_file.path.to_vec();
                let file_name = additional_path.pop().unwrap();

                // Get the destination path of our file, without the file at the end, and create his folder.
                let mut current_path = extracted_path.clone().join(additional_path.iter().collect::<PathBuf>());
                DirBuilder::new().recursive(true).create(&current_path)?;

                // Finish the path and save the file.
                current_path.push(&file_name);
                let mut file = BufWriter::new(File::create(&current_path)?);
                match file.write_all(&packed_file.get_data()?){
                    Ok(_) => files_extracted += 1,
                    Err(_) => error_files.push(format!("{:?}", current_path)),
                }
            }
        },

        // No paths selected, none selected, invalid path selected, or invalid value. 
        0 | 8..=255 => return Err(ErrorKind::NonExistantFile)?,
    }

    // If there is any error in the list, report it.
    if !error_files.is_empty() {
        let error_files_string = error_files.iter().map(|x| format!("<li>{}</li>", x)).collect::<Vec<String>>();
        return Err(ErrorKind::ExtractError(error_files_string))?
    }

    // If we reach this, return success.
    Ok(format!("{} files extracted. No errors detected.", files_extracted))
}

/// This function is used to rename anything in the TreeView (PackFile not included).
/// It requires:
/// - pack_file: a &mut pack_file::PackFile. It's the PackFile opened.
/// - renaming_data: a series of (PathType to rename + New Names).
///
/// It returns the list of provided PathTypes that could be renamed, with their new names.
pub fn rename_packed_files(
    pack_file: &mut PackFile,
    renaming_data: &[(PathType, String)],
) -> Vec<(PathType, String)> {
    
    let mut renamed_data = vec![];
    for (item_type, new_name) in renaming_data {
        match item_type {
            PathType::File(ref path) => {
                
                // First we check if the name is valid, and ignore it if it's not.
                if new_name == path.last().unwrap() { continue; }
                else if new_name.is_empty() { continue; }
                
                // Then update the path with the new name, and try to change it.
                let mut new_path = path.to_vec();
                *new_path.last_mut().unwrap() = new_name.to_owned();

                if !pack_file.packedfile_exists(&new_path) {
                    if let Some(packed_file) = pack_file.packed_files.iter_mut().find(|x| &x.path == path) { 
                        packed_file.path = new_path;
                        renamed_data.push((item_type.clone(), new_name.to_owned())); 
                    }
                }
            }
            
            PathType::Folder(ref path) => {

                // Then update the path with the new name, and try to change it.
                let mut new_path = path.to_vec();
                *new_path.last_mut().unwrap() = new_name.to_owned();

                // If the folder doesn't exist yet, we change the name of the folder we want to rename
                // in the path of every file that starts with his path.
                if !pack_file.folder_exists(&new_path) {
                    let index_position = path.len() - 1;
                    for packed_file in &mut pack_file.packed_files {
                        if packed_file.path.starts_with(&path) {
                            packed_file.path.remove(index_position);
                            packed_file.path.insert(index_position, new_name.to_string());
                        }
                    }
                    renamed_data.push((item_type.clone(), new_name.to_owned())); 
                }
            }
            PathType::PackFile | PathType::None => continue,
        }
    }

    renamed_data
}

/*
--------------------------------------------------------
             PackedFile-Related Functions
--------------------------------------------------------
*/

/// This function saves the data of the edited Loc PackedFile in the main PackFile after a change has
/// been done by the user. Checking for valid characters is done before this, so be careful to not break it.
pub fn update_packed_file_data_loc(
    packed_file_data_decoded: &Loc,
    pack_file: &mut PackFile,
    path: &[String],
) {
    let packed_file = &mut pack_file.packed_files.iter_mut().find(|x| x.path == path).ok_or_else(|| Error::from(ErrorKind::PackedFileNotFound)).unwrap();
    packed_file.set_data(Loc::save(packed_file_data_decoded));
}

/// Like the other one, but this one requires a PackedFile.
pub fn update_packed_file_data_loc_2(
    packed_file_data_decoded: &Loc,
    packed_file: &mut PackedFile,
) {
    packed_file.set_data(Loc::save(packed_file_data_decoded));
}

/// This function saves the data of the edited DB PackedFile in the main PackFile after a change has
/// been done by the user. Checking for valid characters is done before this, so be careful to not break it.
pub fn update_packed_file_data_db(
    packed_file_data_decoded: &DB,
    pack_file: &mut PackFile,
    path: &[String],
) {

    let packed_file = &mut pack_file.packed_files.iter_mut().find(|x| x.path == path).ok_or_else(|| Error::from(ErrorKind::PackedFileNotFound)).unwrap();
    packed_file.set_data(DB::save(packed_file_data_decoded));
}

// Same as the other one, but it requires a PackedFile to modify instead the entire PackFile.
pub fn update_packed_file_data_db_2(
    packed_file_data_decoded: &DB,
    packed_file: &mut PackedFile,
) {
    packed_file.set_data(DB::save(packed_file_data_decoded));
}

/// This function saves the data of the edited Text PackedFile in the main PackFile after a change has
/// been done by the user. Checking for valid characters is done before this, so be careful to not break it.
pub fn update_packed_file_data_text(
    packed_file_data_decoded: &[u8],
    pack_file: &mut PackFile,
    path: &[String],
) {
    let packed_file = &mut pack_file.packed_files.iter_mut().find(|x| x.path == path).ok_or_else(|| Error::from(ErrorKind::PackedFileNotFound)).unwrap();
    packed_file.set_data(packed_file_data_decoded.to_vec());
}

/// This function saves the data of the edited RigidModel PackedFile in the main PackFile after a change has
/// been done by the user. Checking for valid characters is done before this, so be careful to not break it.
/// This can fail in case a 0-Padded String of the RigidModel fails his encoding, so we check that too.
pub fn update_packed_file_data_rigid(
    packed_file_data_decoded: &RigidModel,
    pack_file: &mut PackFile,
    path: &[String],
) -> Result<String> {
    let packed_file = &mut pack_file.packed_files.iter_mut().find(|x| x.path == path).ok_or_else(|| Error::from(ErrorKind::PackedFileNotFound)).unwrap();
    packed_file.set_data(RigidModel::save(packed_file_data_decoded)?);

    Ok(format!("RigidModel PackedFile updated successfully."))
}

/*
--------------------------------------------------------
         Special PackedFile-Related Functions
--------------------------------------------------------
*/

/// This function is used to patch and clean a PackFile exported with Terry, so the SiegeAI (if there
/// is SiegeAI implemented in the map) is patched and the extra useless .xml files are deleted.
/// It requires a mut ref to a decoded PackFile, and returns an String and the list of removed PackedFiles.
pub fn patch_siege_ai (
    pack_file: &mut PackFile
) -> Result<(String, Vec<PathType>)> {

    let mut files_patched = 0;
    let mut files_deleted = 0;
    let mut files_to_delete: Vec<Vec<String>> = vec![];
    let mut deleted_files_type: Vec<PathType> = vec![];
    let mut packfile_is_empty = true;
    let mut multiple_defensive_hill_hints = false;

    // For every PackedFile in the PackFile we check first if it's in the usual map folder, as we
    // don't want to touch files outside that folder.
    for i in &mut pack_file.packed_files {
        if i.path.starts_with(&["terrain".to_owned(), "tiles".to_owned(), "battle".to_owned(), "_assembly_kit".to_owned()]) &&
            i.path.last() != None {

            let x = i.path.last().unwrap().clone();
            packfile_is_empty = false;

            // If it's one of the possible candidates for Patching, we first check if it has
            // an Area Node in it, as that's the base for SiegeAI. If it has an Area Node,
            // we search the Defensive Hill and Patch it. After that, we check if there are
            // more Defensive Hills in the file. If there are more, we return success but
            // notify the modder that the file should have only one.
            if x == "bmd_data.bin"
                || x == "catchment_01_layer_bmd_data.bin"
                || x == "catchment_02_layer_bmd_data.bin"
                || x == "catchment_03_layer_bmd_data.bin"
                || x == "catchment_04_layer_bmd_data.bin"
                || x == "catchment_05_layer_bmd_data.bin"
                || x == "catchment_06_layer_bmd_data.bin"
                || x == "catchment_07_layer_bmd_data.bin"
                || x == "catchment_08_layer_bmd_data.bin"
                || x == "catchment_09_layer_bmd_data.bin" {

                    let mut data: Vec<u8> = i.get_data()?;
                    if data.windows(19).find(|window: &&[u8]
                        |String::from_utf8_lossy(window) == "AIH_SIEGE_AREA_NODE") != None {

                    let patch = "AIH_FORT_PERIMETER".to_string();
                    let index = data.windows(18)
                        .position(
                            |window: &[u8]
                            |String::from_utf8_lossy(window) == "AIH_DEFENSIVE_HILL");

                    if index != None {
                        for j in 0..18 {
                            data[index.unwrap() + (j as usize)] = patch.chars().nth(j).unwrap() as u8;
                        }
                        files_patched += 1;
                    }
                    if data.windows(18).find(|window: &&[u8]
                            |String::from_utf8_lossy(window) == "AIH_DEFENSIVE_HILL") != None {
                        multiple_defensive_hill_hints = true;
                    }
                }
                i.set_data(data);
            }

            // If it's an xml, we add it to the list of files_to_delete, as all the .xml files
            // in this folder are useless and only increase the size of the PackFile.
            else if x.ends_with(".xml") {
                files_to_delete.push(i.path.to_vec());
            }
        }
    }

    // If there are files to delete, we delete them.
    if !files_to_delete.is_empty() {
        for tree_path in &mut files_to_delete {
            let path_type = PathType::File(tree_path.to_vec());
            deleted_files_type.push(path_type);
        }

        // Delete the PackedFiles in one go.
        files_deleted = deleted_files_type.len();
        delete_from_packfile(pack_file, &deleted_files_type);
    }

    // And now we return success or error depending on what happened during the patching process.
    if packfile_is_empty {
        Err(ErrorKind::PatchSiegeAIEmptyPackFile)?
    }
    else if files_patched == 0 && files_deleted == 0 {
        Err(ErrorKind::PatchSiegeAINoPatchableFiles)?
    }
    else {
        if files_patched == 0 {
            Ok((format!("No file suitable for patching has been found.\n{} files deleted.", files_deleted), deleted_files_type))
        }
        else if multiple_defensive_hill_hints {
            if files_deleted == 0 {
                Ok((format!("{} files patched.\nNo file suitable for deleting has been found.\
                \n\n\
                WARNING: Multiple Defensive Hints have been found and we only patched the first one.\
                 If you are using SiegeAI, you should only have one Defensive Hill in the map (the \
                 one acting as the perimeter of your fort/city/castle). Due to SiegeAI being present, \
                 in the map, normal Defensive Hills will not work anyways, and the only thing they do \
                 is interfere with the patching process. So, if your map doesn't work properly after \
                 patching, delete all the extra Defensive Hill Hints. They are the culprit.",
                 files_patched), deleted_files_type))
            }
            else {
                Ok((format!("{} files patched.\n{} files deleted.\
                \n\n\
                WARNING: Multiple Defensive Hints have been found and we only patched the first one.\
                 If you are using SiegeAI, you should only have one Defensive Hill in the map (the \
                 one acting as the perimeter of your fort/city/castle). Due to SiegeAI being present, \
                 in the map, normal Defensive Hills will not work anyways, and the only thing they do \
                 is interfere with the patching process. So, if your map doesn't work properly after \
                 patching, delete all the extra Defensive Hill Hints. They are the culprit.",
                files_patched, files_deleted), deleted_files_type))
            }
        }
        else if files_deleted == 0 {
            Ok((format!("{} files patched.\nNo file suitable for deleting has been found.", files_patched), deleted_files_type))
        }
        else {
            Ok((format!("{} files patched.\n{} files deleted.", files_patched, files_deleted), deleted_files_type))
        }
    }
}

/// This function is used to patch a RigidModel 3D model from Total War: Attila to work in Total War:
/// Warhammer 1 and 2. The process to patch a RigidModel is simple:
/// - We update the version of the RigidModel from 6(Attila) to 7(Warhammer 1&2).
/// - We add 2 u32 to the Lods: a counter starting at 0, and a 0.
/// - We increase the start_offset of every Lod by (8*amount_of_lods).
/// - We may need to increase the zoom_factor of the first Lod to 1000.0, because otherwise sometimes the models
///   disappear when you move the camera far from them.
/// It requires a mut ref to a decoded PackFile, and returns an String (Result<Success, Error>).
pub fn patch_rigid_model_attila_to_warhammer (
    rigid_model: &mut RigidModel
) -> Result<String> {

    // If the RigidModel is an Attila RigidModel, we continue. Otherwise, return Error.
    match rigid_model.packed_file_header.packed_file_header_model_type {
        6 => {
            // We update his version.
            rigid_model.packed_file_header.packed_file_header_model_type = 7;

            // Next, we change the needed data for every Lod.
            for (index, lod) in rigid_model.packed_file_data.packed_file_data_lods_header.iter_mut().enumerate() {
                lod.mysterious_data_1 = Some(index as u32);
                lod.mysterious_data_2 = Some(0);
                lod.start_offset += 8 * rigid_model.packed_file_header.packed_file_header_lods_count;
            }
            Ok(format!("RigidModel patched succesfully."))
        },
        7 => Err(ErrorKind::RigidModelPatchToWarhammer("This is not an Attila's RigidModel, but a Warhammer one.".to_owned()))?,
        _ => Err(ErrorKind::RigidModelPatchToWarhammer("I don't even know from what game is this RigidModel.".to_owned()))?,
    }
}

/// This function is used to optimize the size of a PackFile. It does two things: removes unchanged rows
/// from tables (and if the table is empty, it removes it too) and it cleans the PackFile of extra .xml files 
/// often created by map editors. It requires just the PackFile to optimize and the dependency PackFile.
pub fn optimize_packfile(pack_file: &mut PackFile) -> Result<Vec<PathType>> {
    
    // List of PackedFiles to delete. This includes empty DB Tables and empty Loc PackedFiles.
    let mut files_to_delete: Vec<Vec<String>> = vec![];
    let mut deleted_files_type: Vec<PathType> = vec![];

    // Get a list of every Loc and DB PackedFiles in our dependency's files. For performance reasons, we decode every one of them here.
    // Otherwise, they may have to be decoded multiple times, making this function take ages to finish. 
    let game_locs = DEPENDENCY_DATABASE.lock().unwrap().iter()
        .filter(|x| x.path.last().unwrap().ends_with(".loc"))
        .filter_map(|x| x.get_data().ok())
        .filter_map(|x| Loc::read(&x).ok())
        .collect::<Vec<Loc>>();

    let mut game_dbs = if let Some(ref schema) = *SCHEMA.lock().unwrap() {
        DEPENDENCY_DATABASE.lock().unwrap().iter()
            .filter(|x| x.path.len() == 3 && x.path[0] == "db")
            .map(|x| (x.get_data(), x.path[1].to_owned()))
            .filter(|x| x.0.is_ok())
            .filter_map(|x| DB::read(&x.0.unwrap(), &x.1, &schema).ok())
            .collect::<Vec<DB>>()
    } else { vec![] };

    // Due to precision issues with float fields, we have to round every float field from the tables to 3 decimals max.
    game_dbs.iter_mut().for_each(|x| x.entries.iter_mut()
        .for_each(|x| x.iter_mut()
        .for_each(|x| if let DecodedData::Float(data) = x { *data = (*data * 1000f32).round() / 1000f32 })
    ));

    for mut packed_file in &mut pack_file.packed_files {

        // If it's a DB table and we have an schema...
        if packed_file.path.len() == 3 && packed_file.path[0] == "db" && !game_dbs.is_empty() {
            if let Some(ref schema) = *SCHEMA.lock().unwrap() {

                // Try to decode our table.
                let mut optimized_table = match DB::read(&(packed_file.get_data_and_keep_it()?), &packed_file.path[1], &schema) {
                    Ok(table) => table,
                    Err(_) => continue,
                };

                // We have to round our floats too.
                optimized_table.entries.iter_mut()
                    .for_each(|x| x.iter_mut()
                    .for_each(|x| if let DecodedData::Float(data) = x { *data = (*data * 1000f32).round() / 1000f32 })
                );

                // For each vanilla DB Table that coincide with our own, compare it row by row, cell by cell, with our own DB Table. Then delete in reverse every coincidence.
                for game_db in &game_dbs {
                    if game_db.db_type == optimized_table.db_type && game_db.version == optimized_table.version {
                        let rows_to_delete = optimized_table.entries.iter().enumerate().filter(|(_, entry)| game_db.entries.contains(entry)).map(|(row, _)| row).collect::<Vec<usize>>();
                        for row in rows_to_delete.iter().rev() {
                            optimized_table.entries.remove(*row);
                        } 
                    }
                }

                // Save the data to the PackFile and, if it's empty, add it to the deletion list.
                update_packed_file_data_db_2(&optimized_table, &mut packed_file);
                if optimized_table.entries.is_empty() { files_to_delete.push(packed_file.path.to_vec()); }
            }

            // Otherwise, we just check if it's empty. In that case, we delete it.
            else if let Ok((_, entry_count, _)) = DB::get_header_data(&(packed_file.get_data()?)) {
                if entry_count == 0 { files_to_delete.push(packed_file.path.to_vec()); }
            }
        }

        // If it's a Loc PackedFile and there are some Locs in our dependencies...
        else if packed_file.path.last().unwrap().ends_with(".loc") && !game_locs.is_empty() {

            // Try to decode our Loc. If it's empty, skip it and continue with the next one.
            let mut optimized_loc = match Loc::read(&(packed_file.get_data_and_keep_it()?)) {
                Ok(loc) => if !loc.entries.is_empty() { loc } else { continue },
                Err(_) => continue,
            };

            // For each vanilla Loc, compare it row by row, cell by cell, with our own Loc. Then delete in reverse every coincidence.
            for game_loc in &game_locs {
                let rows_to_delete = optimized_loc.entries.iter().enumerate().filter(|(_, entry)| game_loc.entries.contains(entry)).map(|(row, _)| row).collect::<Vec<usize>>();
                for row in rows_to_delete.iter().rev() {
                    optimized_loc.entries.remove(*row);
                } 
            }

            // Save the data to the PackFile and, if it's empty, add it to the deletion list.
            update_packed_file_data_loc_2(&optimized_loc, &mut packed_file);
            if optimized_loc.entries.is_empty() { files_to_delete.push(packed_file.path.to_vec()); }
        }
    }

    // If there are files to delete, get his type and delete them
    if !files_to_delete.is_empty() {
        for tree_path in &mut files_to_delete {
            let path_type = PathType::File(tree_path.to_vec());
            deleted_files_type.push(path_type);
        }

        // Delete the PackedFiles in one go.
        delete_from_packfile(pack_file, &deleted_files_type);
    }

    // Return the deleted file's types.
    Ok(deleted_files_type)
}
