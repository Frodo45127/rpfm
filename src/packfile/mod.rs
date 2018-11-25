// In this file are all the functions that the UI needs to interact with the PackFile logic.
// As a rule, there should be no UI-related stuff in this module or his childrens.

// use std::fs::{File, DirBuilder, copy};
use std::fs::{File, DirBuilder};
use std::io::{Read, Write};
use std::path::PathBuf;
use std::io::BufReader;
use std::io::BufWriter;

use SUPPORTED_GAMES;
use GAME_SELECTED;
use common::*;
use error::{Error, ErrorKind, Result};
use packfile::packfile::PFHFileType;
use packedfile::loc::Loc;
use packedfile::db::DB;
use packedfile::db::schemas::Schema;
use packedfile::rigidmodel::RigidModel;
use settings::Settings;

pub mod packfile;

/*
--------------------------------------------------------
                PackFile-Related Functions
--------------------------------------------------------
*/

/// This function creates a new PackFile with the name received.
pub fn new_packfile(file_name: String, pfh_version: packfile::PFHVersion) -> packfile::PackFile {
    packfile::PackFile::new_with_name(file_name, pfh_version)
}

/// This function is used to open the PackFiles. It requires the path of the PackFile to open, and
/// it returns the PackFile decoded (if success) or an error message (if error).
pub fn open_packfile(pack_file_path: PathBuf, use_lazy_loading: bool) -> Result<packfile::PackFile> {

    // If the name doesn't end in ".pack", we don't open it. It works, but it'll break some things.
    if pack_file_path.file_name().unwrap().to_str().unwrap().ends_with(".pack") {
        packfile::PackFile::read(pack_file_path, use_lazy_loading)
    }

    // Otherwise, return an error.
    else { Err(ErrorKind::OpenPackFileInvalidExtension)? }
}

/// This function is a special open function, to get all the DB and LOC PackedFiles for a game, and a mod if that mode requires another mod.
/// It returns all the PackedFiles in a big Vec<PackedFile>.
pub fn load_dependency_packfiles(settings: &Settings, dependencies: &[String]) -> Vec<packfile::PackedFile> {

    // Create the empty list.
    let mut packed_files = vec![];

    // Get all the paths we need.
    let main_db_pack_paths = get_game_selected_db_pack_path(settings);
    let main_loc_pack_paths = get_game_selected_loc_pack_path(settings);

    let data_packs_paths = get_game_selected_data_packfiles_paths(settings);
    let content_packs_paths = get_game_selected_content_packfiles_paths(settings);

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
                if path.file_name().unwrap().to_string_lossy().as_ref().to_owned() == *packfile {
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
                if path.file_name().unwrap().to_string_lossy().as_ref().to_owned() == *packfile {

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

/// This function is another special open function, to get all the PackedFiles from every CA PackFile of a game.
/// It returns a fake PackFile with them.
pub fn load_all_ca_packfiles(settings: &Settings) -> Result<packfile::PackFile> {

    // Create the fake PackFile.
    let pfh_version = SUPPORTED_GAMES.get(&**GAME_SELECTED.lock().unwrap()).unwrap().id;
    let mut pack_file = packfile::PackFile::new_with_name(GAME_SELECTED.lock().unwrap().to_owned(), pfh_version);

    // Get all the paths we need and open them one by one.
    let packs_paths = if let Some(paths) = get_game_selected_data_packfiles_paths(settings) { paths } else { Err(ErrorKind::GamePathNotConfigured)? };
    let mut ca_pack_files = vec![];
    for path in packs_paths {
        ca_pack_files.push(packfile::PackFile::read(path, true)?);
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
    mut pack_file: &mut packfile::PackFile,
    new_path: Option<PathBuf>,
    is_editing_of_ca_packfiles_allowed: bool
) -> Result<()> {

    // If any of the problematic masks in the header is set or is one of CA's, return an error.
    if !pack_file.is_editable(is_editing_of_ca_packfiles_allowed) { return Err(ErrorKind::PackFileIsNonEditable)? }

    // If we receive a new path, update it. Otherwise, ensure the file actually exists on disk.
    if let Some(path) = new_path { pack_file.file_path = path; }
    else if !pack_file.file_path.is_file() { return Err(ErrorKind::PackFileIsNotAFile)? }
    
    // And we try to save it.
    packfile::PackFile::save(&mut pack_file)
}

/// This function is used to add a file to a PackFile, processing it and turning it into a PackedFile.
/// It returns a success or error message, depending on whether the file has been added, or not.
/// It requires:
/// - pack_file: a &mut pack_file::PackFile. It's the PackFile where we are going add the file.
/// - file_path: a PathBuf with the current path of the file.
/// - tree_path: a Vec<String> with the path in the TreeView where we are going to add the file.
pub fn add_file_to_packfile(
    pack_file: &mut packfile::PackFile,
    file_path: &PathBuf,
    tree_path: Vec<String>
) -> Result<()> {

    // If there is already a PackedFile in that path...
    if pack_file.packedfile_exists(&tree_path) {

        // Create the theorical path of the PackedFile.
        let mut theorical_path = tree_path.to_vec();
        theorical_path.insert(0, pack_file.file_path.file_name().unwrap().to_string_lossy().to_string());

        // Get the destination PackedFile.
        let packed_file = &mut pack_file.packed_files.iter_mut().find(|x| x.path == tree_path).ok_or(Error::from(ErrorKind::PackedFileNotFound))?;

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
        let packed_files = vec![packfile::PackedFile::read(get_last_modified_time_from_file(&file.get_ref()), tree_path, data); 1];
        pack_file.add_packedfiles(packed_files);
        Ok(())
    }
}

/// This function is used to add one or many PackedFiles to a PackFile (from another PackFile).
/// It returns a success or error message, depending on whether the PackedFile has been added, or not.
/// It requires:
/// - pack_file_source: a &pack_file::PackFile. It's the PackFile from we are going to take the PackedFile.
/// - pack_file_destination: a &mut pack_file::PackFile. It's the Destination PackFile for the PackedFile.
/// - complete_tree_path: the complete path (with PackFile) of the PackedFile or PackedFiles we want to add. A &[String] it is.
pub fn add_packedfile_to_packfile(
    pack_file_source: &packfile::PackFile,
    pack_file_destination: &mut packfile::PackFile,
    complete_path: &[String],
) -> Result<()> {

    // Get the type of whatever we want to add.
    let path_type = get_type_of_selected_path(complete_path, pack_file_source);

    // Get the real path (without PackFile).
    let real_path = if complete_path.len() > 1 { &complete_path[1..] } else { &[] };

    // Act depending on the PackedFile type.
    match path_type {

        // If the path is a file...
        TreePathType::File(path) => {

            // Check if the PackedFile already exists in the destination.
            if pack_file_destination.packedfile_exists(real_path) {

                // Get the destination PackedFile. If it fails, CTD because it's a code problem.
                let packed_file = &mut pack_file_destination.packed_files.iter_mut().find(|x| x.path == path).ok_or(Error::from(ErrorKind::PackedFileNotFound))?;
                packed_file.set_data(pack_file_source.packed_files.iter().find(|x| x.path == path).ok_or(Error::from(ErrorKind::PackedFileNotFound))?.get_data()?);

                // Return success.
                Ok(())
            }

            // Otherwise...
            else {

                // We get the PackedFile, clone it and add it to our own PackFile.
                let mut packed_file = pack_file_source.packed_files.iter().find(|x| x.path == path).ok_or(Error::from(ErrorKind::PackedFileNotFound))?.clone();
                packed_file.load_data()?;
                pack_file_destination.add_packedfiles(vec![packed_file; 1]);

                // Return success.
                Ok(())
            }
        }

        // If the path is a folder...
        TreePathType::Folder(_) => {

            // For each PackedFile inside the folder...
            for packed_file in pack_file_source.packed_files.iter() {

                // If it's one of the PackedFiles we want...
                if !packed_file.path.is_empty() && packed_file.path.starts_with(real_path) {

                    // Check if the PackedFile already exists in the destination.
                    if pack_file_destination.packedfile_exists(&packed_file.path) {

                        // Get the destination PackedFile.
                        let packed_file = &mut pack_file_destination.packed_files.iter_mut().find(|x| x.path == packed_file.path).ok_or(Error::from(ErrorKind::PackedFileNotFound))?;

                        // Then, we get his data.
                        let index = pack_file_source.packed_files.iter().position(|x| x.path == packed_file.path).unwrap();
                        packed_file.set_data(pack_file_source.packed_files[index].get_data()?);
                    }

                    // Otherwise...
                    else {

                        // We get the PackedFile, clone it and add it to our own PackFile.
                        let mut packed_file = pack_file_source.packed_files.iter().find(|x| x.path == packed_file.path).ok_or(Error::from(ErrorKind::PackedFileNotFound))?.clone();
                        packed_file.load_data()?;
                        pack_file_destination.add_packedfiles(vec![packed_file]);
                    }
                }
            }

            // Return success.
            Ok(())
        },

        // If the path is the PackFile...
        TreePathType::PackFile => {

            // For each PackedFile inside the folder...
            for packed_file in pack_file_source.packed_files.iter() {

                // Check if the PackedFile already exists in the destination.
                if pack_file_destination.packedfile_exists(&packed_file.path) {

                    // Get the destination PackedFile.
                    let packed_file = &mut pack_file_destination.packed_files.iter_mut().find(|x| x.path == packed_file.path).ok_or(Error::from(ErrorKind::PackedFileNotFound))?;

                    // Then, we get his data.
                    let index = pack_file_source.packed_files.iter().position(|x| x.path == packed_file.path).unwrap();
                    packed_file.set_data(pack_file_source.packed_files[index].get_data()?)
                }

                // Otherwise...
                else {

                    // We get the PackedFile.
                    let mut packed_file = pack_file_source.packed_files.iter().find(|x| x.path == packed_file.path).ok_or(Error::from(ErrorKind::PackedFileNotFound))?.clone();
                    packed_file.load_data()?;
                    pack_file_destination.add_packedfiles(vec![packed_file]);
                }
            }

            // Return success.
            Ok(())
        },

        // In any other case, there is a problem somewhere. Otherwise, this is unreachable.
        _ => unreachable!()
    }
}

/// This function is used to delete a PackedFile or a group of PackedFiles under the same tree_path
/// from the PackFile. We just need the open PackFile and the tree_path of the file/folder to delete.
pub fn delete_from_packfile(
    pack_file: &mut packfile::PackFile,
    tree_path: &[String]
) -> Result<()> {

    // Get what it's what we want to delete.
    match get_type_of_selected_path(tree_path, pack_file) {

        // If it's a file, easy job.
        TreePathType::File(packed_file_path) => {
            let index = pack_file.packed_files.iter().position(|x| x.path == packed_file_path).unwrap();
            pack_file.remove_packedfile(index);
        }

        // If it's a folder... it's a bit tricky.
        TreePathType::Folder(tree_path) => {

            // We create a vector to store the indexes of the files we are going to delete.
            let mut indexes = vec![];

            // For each PackedFile in our PackFile...
            for (index, packed_file) in pack_file.packed_files.iter().enumerate() {

                // If the PackedFile it's in our folder...
                if !packed_file.path.is_empty() && packed_file.path.starts_with(&tree_path) {

                    // Add his index to the indexes list.
                    indexes.push(index);
                }
            }

            // For each PackedFile we want to remove (in reverse), we remove it individually.
            indexes.iter().rev().for_each(|index| pack_file.remove_packedfile(*index));
        },

        // If it's a PackFile, easy job. For non-existant files, return an error so the UI does nothing.
        TreePathType::PackFile => pack_file.remove_all_packedfiles(),
        TreePathType::None => Err(ErrorKind::Generic)?,
    }

    // Return success.
    Ok(())
}

/// This function is used to extract a PackedFile or a folder from the PackFile.
/// It requires:
/// - pack_file: the PackFile from where we want to extract the PackedFile.
/// - tree_path: the COMPLETE tree_path of the PackedFile we want to extract.
/// - extracted_path: the destination path of the file we want to extract.
pub fn extract_from_packfile(
    pack_file: &packfile::PackFile,
    tree_path: &[String],
    extracted_path: &PathBuf
) -> Result<String> {

    // Get what it's what we want to extract.
    match get_type_of_selected_path(tree_path, pack_file) {

        // If it's a file, we try to create and write the file.
        TreePathType::File(path) => {
            let mut file = BufWriter::new(File::create(&extracted_path)?);
            match file.write_all(&pack_file.packed_files.iter().find(|x| x.path == path).ok_or(Error::from(ErrorKind::PackedFileNotFound))?.get_data()?){
                Ok(_) => Ok(format!("File extracted successfully:\n{}", extracted_path.display())),
                Err(_) => Err(ErrorKind::ExtractError(vec![format!("<li>{}</li>", extracted_path.display().to_string());1]))?
            }
        },

        // If it's a folder...
        TreePathType::Folder(tree_path) => {

            // These variables are here to keep track of what we have extracted and what files failed.
            let mut files_extracted = 0;
            let mut error_files = vec![];

            // For each PackedFile we have...
            for packed_file in &pack_file.packed_files {

                // If it's one we need to extract...
                if packed_file.path.starts_with(&tree_path) {

                    // We remove everything from his path up to the folder we want to extract (included).
                    let mut additional_path = packed_file.path.to_vec();
                    additional_path.drain(..(tree_path.len()));

                    // Remove the name of the file from the path and keep it.
                    let file_name = additional_path.pop().unwrap();

                    // Get the destination path of our file, without the file at the end.
                    let mut current_path = extracted_path.clone().join(additional_path.iter().collect::<PathBuf>());

                    // Create that directory.
                    DirBuilder::new().recursive(true).create(&current_path)?;

                    // Get the full path of the file.
                    current_path.push(&file_name);

                    // Try to create the file.
                    let mut file = BufWriter::new(File::create(&current_path)?);

                    // And try to write it. If any of the files throws an error, add it to the list and continue.
                    match file.write_all(&packed_file.get_data()?) {
                        Ok(_) => files_extracted += 1,
                        Err(_) => error_files.push(format!("{:?}", current_path)),
                    }
                }
            }

            // If there is any error in the list, report it.
            if !error_files.is_empty() {
                let error_files_string = error_files.iter().map(|x| format!("<li>{}</li>", x)).collect::<Vec<String>>();
                return Err(ErrorKind::ExtractError(error_files_string))?
            }

            // If we reach this, return success.
            Ok(format!("{} files extracted. No errors detected.", files_extracted))
        },

        // If it's the PackFile...
        TreePathType::PackFile => {

            // These variables are here to keep track of what we have extracted and what files failed.
            let mut files_extracted = 0;
            let mut error_files = vec![];

            // For each PackedFile we have...
            for packed_file in &pack_file.packed_files {

                // We remove everything from his path up to the folder we want to extract (not included).
                let mut additional_path = packed_file.path.to_vec();

                // Remove the name of the file from the path and keep it.
                let file_name = additional_path.pop().unwrap();

                // Get the destination path of our file, with the name of the PackFile, without the file at the end.
                let mut current_path = extracted_path.clone().join(additional_path.iter().collect::<PathBuf>());

                // Create that directory.
                DirBuilder::new().recursive(true).create(&current_path)?;

                // Get the full path of the file.
                current_path.push(&file_name);

                // Try to create the file.
                let mut file = BufWriter::new(File::create(&current_path)?);

                // And try to write it. If any of the files throws an error, add it to the list and continue.
                match file.write_all(&packed_file.get_data()?){
                    Ok(_) => files_extracted += 1,
                    Err(_) => error_files.push(format!("{:?}", current_path)),
                }
            }

            // If there is any error in the list, report it.
            if !error_files.is_empty() {
                let error_files_string = error_files.iter().map(|x| format!("<li>{}</li>", x)).collect::<Vec<String>>();
                return Err(ErrorKind::ExtractError(error_files_string))?
            }

            // If we reach this, return success.
            Ok(format!("{} files extracted. No errors detected.", files_extracted))
        }

        // If it doesn't exist, there has been a bug somewhere else. Otherwise, this situation will never happen.
        TreePathType::None => unreachable!(),
    }
}

/// This function is used to rename anything in the TreeView (PackFile not included).
/// It requires:
/// - pack_file: a &mut pack_file::PackFile. It's the PackFile opened.
/// - tree_path: a Vec<String> with the tree_path of the file to rename. It needs to be complete.
/// - new_name: the new name of the file to rename.
pub fn rename_packed_file(
    pack_file: &mut packfile::PackFile,
    tree_path: &[String],
    new_name: &str
) -> Result<()> {

    // First we check if the name is valid, and return an error if the new name is invalid.
    if new_name == tree_path.last().unwrap() { Err(ErrorKind::UnchangedInput)? }
    else if new_name.is_empty() { Err(ErrorKind::EmptyInput)? }

    // If we reach this point, we can rename the file/folder.
    else {
        match get_type_of_selected_path(tree_path, pack_file) {
            TreePathType::File(path) => {

                // PackedFiles cannot have spaces in their name.
                if new_name.contains(' ') { Err(ErrorKind::InvalidInput)? }

                // Now we create the new path, while conserving the old one for checks
                let mut new_path = path.to_owned();
                new_path.pop();
                new_path.push(new_name.to_string());

                if !pack_file.packedfile_exists(&new_path) {
                    pack_file.packed_files.iter_mut().find(|x| x.path == path).ok_or(Error::from(ErrorKind::PackedFileNotFound))?.path = new_path;
                    Ok(())
                }
                else { Err(ErrorKind::NameAlreadyInUseInThisPath)? }
            }
            TreePathType::Folder(tree_path) => {
                let mut new_tree_path = tree_path.to_owned();
                new_tree_path.pop();
                new_tree_path.push(new_name.to_string());

                // If the folder doesn't exist yet, we change the name of the folder we want to rename
                // in the path of every file that starts with his path.
                if !pack_file.folder_exists(&new_tree_path) {
                    let index_position = tree_path.len() - 1;
                    for packed_file in &mut pack_file.packed_files {
                        if packed_file.path.starts_with(&tree_path) {
                            packed_file.path.remove(index_position);
                            packed_file.path.insert(index_position, new_name.to_string());
                        }
                    }
                    Ok(())
                }
                else {
                    Err(ErrorKind::NameAlreadyInUseInThisPath)?
                }
            }
            TreePathType::PackFile |
            TreePathType::None => unreachable!(),
        }
    }
}

/// This function is used to apply a prefix to any PackedFile under a path. It returns a tuple with the old
/// paths and the new name for each PackedFile.
/// It requires:
/// - pack_file: a &mut pack_file::PackFile. It's the PackFile opened.
/// - folder_path: a Vec<String> with the tree_path of the folder with the files to rename. It needs to be incomplete.
/// - prefix: the prefix to apply to each PackedFile.
pub fn apply_prefix_to_packed_files(
    pack_file: &mut packfile::PackFile,
    folder_path: &[String],
    prefix: &str
) -> Result<(Vec<Vec<String>>)> {

    // First we check if the prefix is valid, and return an error if the prefix is invalid.
    if prefix.is_empty() { Err(ErrorKind::EmptyInput)? }
    else if prefix.contains(' ') { Err(ErrorKind::InvalidInput)? }

    // If we reach this point, we can safely add the prefix to the PackedFiles.
    else {

        // There is a situation where an old path and a new path can generate a duplicate. 
        // Here is not a problem, but there is no way to prevent it in the UI, so we have to deal with it here.
        let old_paths = pack_file.packed_files.iter().filter(|x| x.path.starts_with(&folder_path)).map(|x| x.path.to_vec()).collect::<Vec<Vec<String>>>();
        let mut new_paths = old_paths.to_vec();
        new_paths.iter_mut().for_each(|x| *x.last_mut().unwrap() = format!("{}{}", prefix, *x.last().unwrap()));

        // Check if ANY of the new paths is also present in the old paths.
        for path in &new_paths {
            if old_paths.contains(&path) { return Err(ErrorKind::InvalidInput)? }
        }

        pack_file.packed_files.iter_mut().filter(|x| x.path.starts_with(&folder_path)).for_each(|x| *x.path.last_mut().unwrap() = format!("{}{}", prefix, *x.path.last().unwrap()));

        // If there were no errors, return the list of changed paths.
        Ok(old_paths)
    }
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
    pack_file: &mut packfile::PackFile,
    path: &[String],
) {
    let packed_file = &mut pack_file.packed_files.iter_mut().find(|x| x.path == path).ok_or(Error::from(ErrorKind::PackedFileNotFound)).unwrap();
    packed_file.set_data(Loc::save(packed_file_data_decoded));
}

/// Like the other one, but this one requires a PackedFile.
pub fn update_packed_file_data_loc_2(
    packed_file_data_decoded: &Loc,
    packed_file: &mut packfile::PackedFile,
) {
    packed_file.set_data(Loc::save(packed_file_data_decoded));
}

/// This function saves the data of the edited DB PackedFile in the main PackFile after a change has
/// been done by the user. Checking for valid characters is done before this, so be careful to not break it.
pub fn update_packed_file_data_db(
    packed_file_data_decoded: &DB,
    pack_file: &mut packfile::PackFile,
    path: &[String],
) {

    let packed_file = &mut pack_file.packed_files.iter_mut().find(|x| x.path == path).ok_or(Error::from(ErrorKind::PackedFileNotFound)).unwrap();
    packed_file.set_data(DB::save(packed_file_data_decoded));
}

// Same as the other one, but it requires a PackedFile to modify instead the entire PackFile.
pub fn update_packed_file_data_db_2(
    packed_file_data_decoded: &DB,
    packed_file: &mut packfile::PackedFile,
) {
    packed_file.set_data(DB::save(packed_file_data_decoded));
}

/// This function saves the data of the edited Text PackedFile in the main PackFile after a change has
/// been done by the user. Checking for valid characters is done before this, so be careful to not break it.
pub fn update_packed_file_data_text(
    packed_file_data_decoded: &[u8],
    pack_file: &mut packfile::PackFile,
    path: &[String],
) {
    let packed_file = &mut pack_file.packed_files.iter_mut().find(|x| x.path == path).ok_or(Error::from(ErrorKind::PackedFileNotFound)).unwrap();
    packed_file.set_data(packed_file_data_decoded.to_vec());
}

/// This function saves the data of the edited RigidModel PackedFile in the main PackFile after a change has
/// been done by the user. Checking for valid characters is done before this, so be careful to not break it.
/// This can fail in case a 0-Padded String of the RigidModel fails his encoding, so we check that too.
pub fn update_packed_file_data_rigid(
    packed_file_data_decoded: &RigidModel,
    pack_file: &mut packfile::PackFile,
    path: &[String],
) -> Result<String> {
    let packed_file = &mut pack_file.packed_files.iter_mut().find(|x| x.path == path).ok_or(Error::from(ErrorKind::PackedFileNotFound)).unwrap();
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
    pack_file: &mut packfile::PackFile
) -> Result<(String, Vec<TreePathType>)> {

    let mut files_patched = 0;
    let mut files_deleted = 0;
    let mut files_to_delete: Vec<Vec<String>> = vec![];
    let mut deleted_files_type: Vec<TreePathType> = vec![];
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

            // Due to the rework of the "delete_from_packfile" function, we need to give it a complete
            // path to delete, so we "complete" his path before deleting.
            let file_name = vec![pack_file.get_file_name()];
            tree_path.splice(0..0, file_name.iter().cloned());

            // Get his type before deleting it.
            deleted_files_type.push(get_type_of_selected_path(&tree_path, &pack_file));

            // Delete the PackedFile. This cannot really fail during this process, so we can ignore this result.
            delete_from_packfile(pack_file, tree_path).unwrap();
            files_deleted += 1;
        }
    }

    // And now we return success or error depending on what happened during the patching process.
    if packfile_is_empty {
        Err(ErrorKind::PatchSiegeAIEmptyPackFile)?
    }
    else if files_patched == 0 && files_deleted == 0 {
        Err(ErrorKind::PatchSiegeAINoPatchableFiles)?
    }
    else if files_patched >= 0 || files_deleted >= 0 {
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
    else { unreachable!() }
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

/*
/// This function is used to turn a bunch of catchment files into Prefabs. It requires:
/// - name_list: the list of names for the new prefabs.
/// - game_path: the base path of our game, to know where to put the xml files for the prefabs.
/// - catchment_indexes: a list with the indexes of all the files we want to turn into prefabs.
/// - pack_file: the PackFile we want to create the prefabs from.
pub fn create_prefab_from_catchment(
    name_list: &[String],
    game_path: &PathBuf,
    catchment_indexes: &[usize],
    pack_file: &Rc<RefCell<packfile::PackFile>>,
) -> Result<String> {

    // Create a new PackFile to store all the new prefabs, and make it "Movie" type.
    let mut prefab_pack_file = new_packfile(format!("_prefab_{}", &pack_file.borrow().extra_data.file_name), &pack_file.borrow().header.id);
    prefab_pack_file.header.pack_file_type = 4;

    // Pair together the catchment indexes with the name list.
    let prefab_list = catchment_indexes.iter().zip(name_list.iter());

    // For each prefab we want to create...
    for (index, prefab) in prefab_list.enumerate() {

        // Add the PackedFile to the new PackFile.
        prefab_pack_file.add_packedfiles(vec![pack_file.borrow().data.packed_files[*prefab.0].clone()]);

        // Change his path to point to the prefab folder.
        prefab_pack_file.data.packed_files[index].path = vec!["prefabs".to_owned(), format!("{}.bmd", prefab.1)];

        // Get the path of the Terry's raw files for the map of our prefab.
        let mut terry_map_path = game_path.to_path_buf().join(PathBuf::from("assembly_kit/raw_data/terrain/tiles/battle/_assembly_kit"));
        terry_map_path.push(&pack_file.borrow().data.packed_files[*prefab.0].path[4]);

        // If the map folder doesn't exist, return error.
        if !terry_map_path.is_dir() { return Err(format_err!("The following map's original folder couldn't be found:\n{:?}", terry_map_path)) }

        // Get the ".terry" file of the map.
        let terry_file = get_files_from_subdir(&terry_map_path).unwrap().iter().filter(|x| x.file_name().unwrap().to_string_lossy().as_ref().to_owned().ends_with(".terry")).cloned().collect::<Vec<PathBuf>>();

        // If the terry file wasn't found, return error.
        if terry_file.is_empty() { return Err(format_err!("The following map's .terry file couldn't be found:\n{:?}", terry_map_path)) }

        // Read it to a String so we can examine it properly.
        let mut file = BufReader::new(File::open(&terry_file[0])?);
        let mut terry_file_string = String::new();
        file.read_to_string(&mut terry_file_string)?;

        // Get the ID of the current catchment (catchment_XX) from the ".terry" file of his map.
        let catchment_name = &pack_file.borrow().data.packed_files[*prefab.0].path[5][..12];

        let line = match terry_file_string.find(&format!("bmd_export_type=\"{}\"/>", catchment_name)) {
            Some(line) => line,
            None => return Err(format_err!("The layer of \"{}\" couldn't be found in the following map's .terry file:\n{:?}", catchment_name, terry_map_path)),
        };
        terry_file_string.truncate(line);

        let id_index = match terry_file_string.rfind(" id=\"") {
            Some(id_index) => id_index,
            None => return Err(format_err!("The id of the layer of \"{}\" couldn't be found in the following map's .terry file:\n{:?}", catchment_name, terry_map_path))
        };
        let id_layer = &terry_file_string[(id_index + 5)..(id_index + 20)];

        // Get the corresponding layer file.
        let mut layer_file = terry_file[0].to_path_buf();
        let layer_file_name = layer_file.file_stem().unwrap().to_string_lossy().as_ref().to_owned();
        layer_file.pop();
        layer_file.push(format!("{}.{}.layer", layer_file_name, id_layer));

        // Get the path where the raw files of the prefabs should be stored.
        let mut prefabs_terry_path = game_path.to_path_buf().join(PathBuf::from("assembly_kit/raw_data/art/prefabs/battle/custom_prefabs"));

        // We check that path exists, and create it if it doesn't.
        if !prefabs_terry_path.is_dir() {
            DirBuilder::new().recursive(true).create(&prefabs_terry_path)?;
        }

        // Get the full path for the prefab's layer and terry files.
        let mut prefabs_terry_path_layer = prefabs_terry_path.to_path_buf();
        let mut prefabs_terry_path_terry = prefabs_terry_path.to_path_buf();
        prefabs_terry_path_layer.push(format!("{}.{}.layer", prefab.1, id_layer));
        prefabs_terry_path_terry.push(format!("{}.terry", prefab.1));

        // Try to copy the layer file to his destination.
        copy(layer_file, prefabs_terry_path_layer).map_err(Error::from)?;

        // Try to write the prefab's terry file into his destination.
        let mut file = BufWriter::new(File::create(&prefabs_terry_path_terry)?);
        let prefab_terry_file = format!("<?xml version=\"1.0\" encoding=\"UTF-8\"?>
            <project version=\"20\" id=\"15afc3311fc3488\">
              <pc type=\"QTU::ProjectPrefab\">
                <data database=\"battle\"/>
              </pc>
              <pc type=\"QTU::Scene\">
                <data version=\"25\">
                  <entity id=\"{}\" name=\"Default\">
                    <ECFileLayer export=\"true\" bmd_export_type=\"\"/>
                  </entity>
                </data>
              </pc>
              <pc type=\"QTU::Terrain\"/>
            </project>"
            , id_layer
        );
        file.write_all(prefab_terry_file.as_bytes()).map_err(Error::from)?;
    }

    // Get the PackFile name here, so we have no problems with references.
    let prefab_pack_file_name = &prefab_pack_file.extra_data.file_name.to_owned();

    // At the end, save the new PackFile.
    save_packfile(&mut prefab_pack_file, Some(game_path.to_path_buf().join(PathBuf::from(format!("data/{}", prefab_pack_file_name)))))?;

    // If nothing failed, return success.
    Ok("Prefabs successfully created.".to_owned())
}
*/

/// This function is used to optimize the size of a PackFile. It does two things: removes unchanged rows
/// from tables (and if the table is empty, it removes it too) and it cleans the PackFile of extra .xml files 
/// often created by map editors. It requires just the PackFile to optimize and the dependency PackFile.
pub fn optimize_packfile(
    pack_file: &mut packfile::PackFile,
    game_packed_files: &[packfile::PackedFile],
    schema: &Option<Schema>
) -> Result<Vec<TreePathType>> {

    // List of PackedFiles to delete. This includes empty DB Tables and empty Loc PackedFiles.
    let mut files_to_delete: Vec<Vec<String>> = vec![];
    let mut deleted_files_type: Vec<TreePathType> = vec![];

    // Get a list of every Loc and DB PackedFiles in our dependency's files. For performance reasons, we decode every one of them here.
    // Otherwise, they may have to be decoded multiple times, making this function take ages to finish. 
    let game_locs = game_packed_files.iter()
        .filter(|x| x.path.last().unwrap().ends_with(".loc"))
        .filter_map(|x| x.get_data().ok())
        .filter_map(|x| Loc::read(&x).ok())
        .collect::<Vec<Loc>>();

    let game_dbs = if let Some(schema) = schema {
        game_packed_files.iter()
            .filter(|x| x.path.len() == 3 && x.path[0] == "db")
            .map(|x| (x.get_data(), x.path[1].to_owned()))
            .filter(|x| x.0.is_ok())
            .filter_map(|x| DB::read(&x.0.unwrap(), &x.1, &schema).ok())
            .collect::<Vec<DB>>()
    } else { vec![] };

    for mut packed_file in &mut pack_file.packed_files {

        // If it's a DB table and we have an schema...
        if packed_file.path.len() == 3 && packed_file.path[0] == "db" && !game_dbs.is_empty() {
            if let Some(schema) = schema {

                // Try to decode our table.
                let mut optimized_table = match DB::read(&(packed_file.get_data_and_keep_it()?), &packed_file.path[1], &schema) {
                    Ok(table) => table,
                    Err(_) => continue,
                };

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

    // If there are files to delete, we delete them.
    if !files_to_delete.is_empty() {
        for tree_path in &mut files_to_delete {

            // Due to the rework of the "delete_from_packfile" function, we need to give it a complete
            // path to delete, so we "complete" his path before deleting.
            let file_name = vec![pack_file.file_path.file_name().unwrap().to_string_lossy().to_string()];
            tree_path.splice(0..0, file_name.iter().cloned());

            // Get his type before deleting it.
            deleted_files_type.push(get_type_of_selected_path(&tree_path, &pack_file));

            // Delete the PackedFile.
            delete_from_packfile(pack_file, tree_path).unwrap();
        }
    }

    // Return the deleted file's types.
    Ok(deleted_files_type)
}
