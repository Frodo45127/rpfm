//---------------------------------------------------------------------------//
// Copyright (c) 2017-2019 Ismael Gutiérrez González. All rights reserved.
// 
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
// 
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

// This is the RPFM Lib, a lib to decode/encode any kind of PackFile CA has to offer, including his contents.

use bincode::deserialize;

use lazy_static::lazy_static;

use std::fs::{File, DirBuilder};
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::Mutex;

use rpfm_error::{ErrorKind, Result};

use crate::common::*;
use crate::schema::Schema;
use crate::packedfile::db::DB;
use crate::packfile::{PackFile, PFHFileType, PathType};
use crate::packfile::packedfile::PackedFile;
use crate::settings::Settings;
use crate::games::{SupportedGames, get_supported_games_list};

pub mod common;
pub mod config;
pub mod games;
pub mod packedfile;
pub mod packfile;
pub mod schema;
pub mod settings;

// Statics, so we don't need to pass them everywhere to use them.
lazy_static! {

    /// List of supported games and their configuration. Their key is what we know as `folder_name`, used to identify the game and
    /// for "MyMod" folders.
    #[derive(Debug)]
    pub static ref SUPPORTED_GAMES: SupportedGames = get_supported_games_list();

    /// The current Settings and Shortcuts. To avoid reference and lock issues, this should be edited ONLY in the background thread.
    pub static ref SETTINGS: Arc<Mutex<Settings>> = Arc::new(Mutex::new(Settings::load(None).unwrap_or_else(|_|Settings::new())));

    /// The current GameSelected. Same as the one above, only edited from the background thread.
    pub static ref GAME_SELECTED: Arc<Mutex<String>> = Arc::new(Mutex::new(SETTINGS.lock().unwrap().settings_string["default_game"].to_owned()));

    /// PackedFiles from the dependencies of the currently open PackFile.
    pub static ref DEPENDENCY_DATABASE: Mutex<Vec<PackedFile>> = Mutex::new(vec![]);
    
    /// DB Files from the Pak File of the current game. Only for dependency checking.
    pub static ref FAKE_DEPENDENCY_DATABASE: Mutex<Vec<DB>> = Mutex::new(vec![]);

    /// Currently loaded schema.
    pub static ref SCHEMA: Arc<Mutex<Option<Schema>>> = Arc::new(Mutex::new(None));
}

pub const DOCS_BASE_URL: &str = "https://frodo45127.github.io/rpfm/";
pub const PATREON_URL: &str = "https://www.patreon.com/RPFM";

/*
--------------------------------------------------------
                PackFile-Related Functions
--------------------------------------------------------
*/


/// This function returns all the DB and Loc PackedFiles from PackFiles marked as *dependency* of the currently open PackFile.
///
/// This means it loads:
/// - The game selected vanilla DB and Loc PackedFiles.
/// - Any DB and Loc in a PackFile in the currently open PackFile's PackFile list.
pub fn load_dependency_packfiles(dependencies: &[String]) -> Vec<PackedFile> {

    // Create the empty list.
    let mut packed_files = vec![];

    // Get all the paths we need.
    let main_db_pack_paths = get_game_selected_db_pack_path(&*GAME_SELECTED.lock().unwrap());
    let main_loc_pack_paths = get_game_selected_loc_pack_path(&*GAME_SELECTED.lock().unwrap());

    let data_packs_paths = get_game_selected_data_packfiles_paths(&*GAME_SELECTED.lock().unwrap());
    let content_packs_paths = get_game_selected_content_packfiles_paths(&*GAME_SELECTED.lock().unwrap());

    // Get all the DB Tables from the main DB PackFiles, if it's configured.
    if let Some(paths) = main_db_pack_paths {
        for path in &paths {
            if let Ok(pack_file) = open_packfiles(&[path.to_path_buf()], false, true, false) {

                // For each PackFile in the data.pack...
                for packed_file in pack_file.get_ref_packed_files_by_path_start(&["db".to_owned()]) {

                    // If it's a DB file...
                    if !packed_file.get_path().is_empty() && packed_file.get_path().starts_with(&["db".to_owned()]) {

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
            if let Ok(pack_file) = open_packfiles(&[path.to_path_buf()], false, true, false) {

                // For each PackFile in the data.pack...
                for packed_file in pack_file.get_ref_packed_files_by_path_end(&[".lock".to_owned()]) {

                    // If it's a Loc file...
                    if !packed_file.get_path().is_empty() && packed_file.get_path().last().unwrap().ends_with(".loc") {

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
                    if let Ok(pack_file) = open_packfiles(&[path.to_path_buf()], false, true, false) {

                        // For each PackFile in the data.pack...
                        for packed_file in pack_file.get_ref_packed_files_by_path_start(&["db".to_owned()]) {

                            // If it's a DB file...
                            if !packed_file.get_path().is_empty() && packed_file.get_path().starts_with(&["db".to_owned()]) {

                                // Clone the PackedFile, and add it to the list.
                                let mut packed_file = packed_file.clone();
                                let _ = packed_file.load_data();
                                packed_files.push(packed_file);
                            }
                        }
                    }

                    // Get all the Loc PackedFiles from the main Loc PackFile, if it's configured.
                    if let Ok(pack_file) = open_packfiles(&[path.to_path_buf()], false, true, false) {

                        // For each PackFile in the data.pack...
                        for packed_file in pack_file.get_ref_packed_files_by_path_end(&[".lock".to_owned()])  {

                            // If it's a Loc file...
                            if !packed_file.get_path().is_empty() && packed_file.get_path().last().unwrap().ends_with(".loc") {

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
                    if let Ok(pack_file) = open_packfiles(&[path.to_path_buf()], false, true, false) {

                        // For each PackFile in the data.pack...
                        for packed_file in pack_file.get_ref_packed_files_by_path_start(&["db".to_owned()]) {

                            // If it's a DB file...
                            if !packed_file.get_path().is_empty() && packed_file.get_path().starts_with(&["db".to_owned()]) {

                                // Clone the PackedFile and add it to the PackedFiles List.
                                let mut packed_file = packed_file.clone();
                                let _ = packed_file.load_data();
                                packed_files.push(packed_file);
                            }
                        }
                    }

                    // Get all the Loc PackedFiles from the main Loc PackFile, if it's configured.
                    if let Ok(pack_file) = open_packfiles(&[path.to_path_buf()], false, true, false) {

                        // For each PackFile in the data.pack...
                        for packed_file in pack_file.get_ref_packed_files_by_path_end(&[".lock".to_owned()])  {

                            // If it's a Loc file...
                            if !packed_file.get_path().is_empty() && packed_file.get_path().last().unwrap().ends_with(".loc") {

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
    if let Ok(pak_file) = get_game_selected_pak_file(&*GAME_SELECTED.lock().unwrap()) {
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

/// This function allows you to open one PackFile, or multiple PackFiles as one. It also takes care of duplicates, 
/// loading the duplicate PackedFile that will get loaded by the game itself.
///
/// It requires:
/// - `packs_paths`: Path list of all the PackFiles we want to open.
/// - `ignore_mods`: If true, all mod packfiles from the provided list will be ignored.
/// - `use_lazy_loading`: If true, all Packfiles will be loaded using Lazy-Loading.
/// - `lock_packfile_type`: If true, the PackFile Type will be changed to disable saving.
pub fn open_packfiles(
    packs_paths: &[PathBuf],
    ignore_mods: bool,
    use_lazy_loading: bool,
    lock_packfile_type: bool
) -> Result<PackFile> {

    // If we just have one PackFile, just open it. No fancy logic needed.
    if packs_paths.len() == 1 {
        if packs_paths[0].file_name().unwrap().to_str().unwrap().ends_with(".pack") {
            PackFile::read(packs_paths[0].to_path_buf(), use_lazy_loading)
        } else { Err(ErrorKind::OpenPackFileInvalidExtension)? }

    }

    // Otherwise, open all of them into a Fake PackFile and take care of the duplicated files like the game will do.
    else {

        // Create the fake PackFile.
        let pfh_version = SUPPORTED_GAMES.get(&**GAME_SELECTED.lock().unwrap()).unwrap().pfh_version;
        let pfh_name = if ignore_mods { GAME_SELECTED.lock().unwrap().to_owned() } else { String::from("merged_mod.pack")};
        let mut pack_file = PackFile::new_with_name(&pfh_name, pfh_version);

        // Get all the paths we need and open them one by one.
        let mut pack_files = vec![];
        for path in packs_paths {
            if path.file_name().unwrap().to_str().unwrap().ends_with(".pack") {
                pack_files.push(PackFile::read(path.to_path_buf(), use_lazy_loading)?);
            } else { Err(ErrorKind::OpenPackFileInvalidExtension)?}
        }

        // Get all the PackedFiles from each PackFile. First Boot type, then Release type, then Patch type, then Mod type, then Movie type.
        let mut boot_files = vec![];
        for pack_file in &pack_files {
            if let PFHFileType::Boot = pack_file.get_pfh_file_type() {
                boot_files = pack_file.get_all_packed_files();
            }
        }
        
        let mut release_files = vec![];
        for pack_file in &pack_files {
            if let PFHFileType::Release = pack_file.get_pfh_file_type() {
                release_files = pack_file.get_all_packed_files();
            }
        }

        let mut patch_files = vec![];
        for pack_file in &pack_files {
            if let PFHFileType::Patch = pack_file.get_pfh_file_type() {
                patch_files = pack_file.get_all_packed_files();
            }
        }

        let mut mod_files = vec![];
        if !ignore_mods {
            for pack_file in &pack_files {
                if let PFHFileType::Mod = pack_file.get_pfh_file_type() {
                    mod_files = pack_file.get_all_packed_files();
                }
            }
        }

        let mut movie_files = vec![];
        for pack_file in &pack_files {
            if let PFHFileType::Movie = pack_file.get_pfh_file_type() {
                movie_files = pack_file.get_all_packed_files();
            }
        }

        // The priority in case of collision is:
        // - Same Type: First to come is the valid one.
        // - Different Type: Last to come is the valid one.
        boot_files.sort_by_key(|x| x.get_path().to_vec());
        boot_files.dedup_by_key(|x| x.get_path().to_vec());

        release_files.sort_by_key(|x| x.get_path().to_vec());
        release_files.dedup_by_key(|x| x.get_path().to_vec());

        patch_files.sort_by_key(|x| x.get_path().to_vec());
        patch_files.dedup_by_key(|x| x.get_path().to_vec());

        mod_files.sort_by_key(|x| x.get_path().to_vec());
        mod_files.dedup_by_key(|x| x.get_path().to_vec());

        movie_files.sort_by_key(|x| x.get_path().to_vec());
        movie_files.dedup_by_key(|x| x.get_path().to_vec());

        pack_file.add_packed_files(&movie_files);
        pack_file.add_packed_files(&patch_files);
        pack_file.add_packed_files(&release_files);
        pack_file.add_packed_files(&boot_files);

        // Set it as type "Other(200)", so we can easely identify it as fake in other places.
        // Used to lock the CA Files.
        if lock_packfile_type {
            pack_file.set_pfh_file_type(PFHFileType::Other(200));
        }
    
        // Return the new PackedFiles list.
        Ok(pack_file)
    }
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
    if let Some(path) = new_path { pack_file.set_file_path(&path)?; }
    else if !pack_file.get_file_path().is_file() { return Err(ErrorKind::PackFileIsNotAFile)? }
    
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
        theorical_path.insert(0, pack_file.get_file_name());

        // Get the destination PackedFile.
        let packed_file = pack_file.get_ref_mut_packed_file_by_path(&tree_path).unwrap();

        // We get the data and his size...
        let mut file = BufReader::new(File::open(&file_path)?);
        let mut data = vec![];
        file.read_to_end(&mut data)?;
        packed_file.set_data(data);

        // Change his last modified time.
        packed_file.set_timestamp(get_last_modified_time_from_file(&file.get_ref()));
    }

    // Otherwise, we add it as a new PackedFile.
    else {

        // We get the data and his size...
        let mut file = BufReader::new(File::open(&file_path)?);
        let mut data = vec![];
        file.read_to_end(&mut data)?;

        // And then we make a PackedFile with it and save it.
        let packed_files = vec![PackedFile::read_from_vec(tree_path, pack_file.get_file_name(), get_last_modified_time_from_file(&file.get_ref()), false, data); 1];
        let added_paths = pack_file.add_packed_files(&packed_files);
        if added_paths.len() < packed_files.len() { Err(ErrorKind::ReservedFiles)? }
    }
    Ok(())
}

/// This function is used to add one or more PackedFiles to a PackFile (from another PackFile).
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
    let reserved_packed_file_names = PackFile::get_reserved_packed_file_names();
    let mut path_types_added = vec![];
    match path_type {

        // If the path is a file...
        PathType::File(path) => {

            // Check if the PackedFile already exists in the destination.
            if !reserved_packed_file_names.contains(&path) {
                if pack_file_destination.packedfile_exists(path) {

                    // Get the destination PackedFile. If it fails, CTD because it's a code problem.
                    let packed_file_source = pack_file_source.get_ref_packed_file_by_path(&path).unwrap();
                    let packed_file_destination = pack_file_destination.get_ref_mut_packed_file_by_path(&path).unwrap();
                    packed_file_destination.set_data(packed_file_source.get_data()?);
                }

                // Otherwise...
                else {

                    // We get the PackedFile, clone it and add it to our own PackFile.
                    let mut packed_file = pack_file_source.get_ref_packed_file_by_path(path).unwrap().clone();
                    packed_file.load_data()?;
                    pack_file_destination.add_packed_files(&[packed_file]);
                }
                path_types_added.push(path_type.clone());
            }
        }

        // If the path is a folder...
        PathType::Folder(ref path) => {

            // For each PackedFile inside the folder...
            for packed_file in pack_file_source.get_ref_packed_files_by_path_start(path) {

                // If it's one of the PackedFiles we want...
                if !packed_file.get_path().is_empty() && packed_file.get_path().starts_with(path) {

                    // Check if the PackedFile already exists in the destination.
                    if !reserved_packed_file_names.contains(&packed_file.get_path().to_vec()) {
                        if pack_file_destination.packedfile_exists(&packed_file.get_path()) {

                            // Get the destination PackedFile.
                            let packed_file = &mut pack_file_destination.get_ref_mut_packed_file_by_path(packed_file.get_path()).unwrap();

                            // Then, we get his data.
                            let index = pack_file_source.get_ref_packed_file_by_path(packed_file.get_path()).unwrap().get_data()?;
                            packed_file.set_data(index);
                        }

                        // Otherwise...
                        else {

                            // We get the PackedFile, clone it and add it to our own PackFile.
                            let mut packed_file = pack_file_source.get_ref_packed_file_by_path(packed_file.get_path()).unwrap().clone();
                            packed_file.load_data()?;
                            pack_file_destination.add_packed_files(&[packed_file]);
                        }
                        path_types_added.push(PathType::File(packed_file.get_path().to_vec()));
                    }
                }
            }
        },

        // If the path is the PackFile...
        PathType::PackFile => {

            // For each PackedFile inside the folder...
            for packed_file in pack_file_source.get_ref_all_packed_files() {

                // Check if the PackedFile already exists in the destination.
                if !reserved_packed_file_names.contains(&packed_file.get_path().to_vec()) {
                    if pack_file_destination.packedfile_exists(&packed_file.get_path()) {

                        // Get the destination PackedFile.
                        let mut packed_file_destination = pack_file_destination.get_ref_mut_packed_file_by_path(packed_file.get_path()).unwrap().clone();
                        packed_file_destination.set_data(packed_file.get_data()?);
                    }

                    // Otherwise...
                    else {

                        // We get the PackedFile.
                        let mut packed_file = pack_file_source.get_ref_packed_file_by_path(packed_file.get_path()).unwrap().clone();
                        packed_file.load_data()?;
                        pack_file_destination.add_packed_files(&[packed_file]);
                    }
                    path_types_added.push(PathType::File(packed_file.get_path().to_vec()));
                }
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
                    PathType::File(path) => pack_file.remove_packed_file_by_path(path),
                    PathType::Folder(path) => pack_file.remove_packed_files_by_path_start(path),
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
                        let packed_file = pack_file.get_ref_packed_file_by_path(path).unwrap();
                        let mut additional_path = packed_file.get_path().to_vec();
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
                    
                        for packed_file in pack_file.get_ref_packed_files_by_path_start(path) {
                            if !path.is_empty() && packed_file.get_path().starts_with(&path) {
                               
                                // We remove everything from his path up to the folder we want to extract (not included).
                                let mut additional_path = packed_file.get_path().to_vec();
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
            for packed_file in pack_file.get_ref_all_packed_files() {

                // We remove everything from his path up to the folder we want to extract (not included).
                let mut additional_path = packed_file.get_path().to_vec();
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
    for i in &mut pack_file.get_all_packed_files() {
        if i.get_path().starts_with(&["terrain".to_owned(), "tiles".to_owned(), "battle".to_owned(), "_assembly_kit".to_owned()]) &&
            i.get_path().last() != None {

            let x = i.get_path().last().unwrap().clone();
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
                files_to_delete.push(i.get_path().to_vec());
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

