// In this file are all the functions that the UI needs to interact with the PackFile logic.
// As a rule, there should be no UI-related stuff in this module or his childrens.

// use std::fs::{File, DirBuilder, copy};
use std::fs::{File, DirBuilder};
use std::io::{Read, Write};
use std::path::PathBuf;
use std::io::BufReader;
use std::io::BufWriter;
use std::io::prelude::*;
use std::io::SeekFrom;

use common::*;
use error::{ErrorKind, Result};
use packedfile::loc::Loc;
use packedfile::db::DB;
use packedfile::rigidmodel::RigidModel;

pub mod packfile;

/*
--------------------------------------------------------
                PackFile-Related Functions
--------------------------------------------------------
*/

/// This function creates a new PackFile with the name received.
pub fn new_packfile(file_name: String, packfile_id: &str) -> packfile::PackFile {
    packfile::PackFile::new_with_name(file_name, packfile_id)
}

/// This function is used to open the PackFiles. It requires the path of the PackFile to open, and
/// it returns the PackFile decoded (if success) or an error message (if error).
pub fn open_packfile(pack_file_path: PathBuf) -> Result<packfile::PackFile> {

    // First, we get his name.
    let pack_file_name = pack_file_path.file_name().unwrap().to_str().unwrap().to_string();

    // If the name doesn't end in ".pack", we don't open it. It works, but it'll break some things.
    if pack_file_name.ends_with(".pack") {

        // We try to open the File.
        let mut pack_file = BufReader::new(File::open(&pack_file_path)?);

        // And then we try to read it into a PackFile.
        packfile::PackFile::read(&mut pack_file, pack_file_name, pack_file_path, false)
    }

    // Otherwise, return an error.
    else { Err(ErrorKind::OpenPackFileInvalidExtension)? }
}

/// This function is used to open the PackFiles. It requires the path of the PackFile to open, and
/// it returns the PackFile decoded (if success) or an error message (if error).
pub fn open_packfile_with_bufreader(pack_file_path: PathBuf) -> Result<(packfile::PackFile, BufReader<File>)> {

    // First, we get his name.
    let pack_file_name = pack_file_path.file_name().unwrap().to_str().unwrap().to_string();

    // If the name doesn't end in ".pack", we don't open it. It works, but it'll break some things.
    if pack_file_name.ends_with(".pack") {

        // We try to open the File.
        let mut pack_file = BufReader::new(File::open(&pack_file_path)?);

        // And then we try to read it into a PackFile.
        packfile::PackFile::read(&mut pack_file, pack_file_name, pack_file_path, true).map(|result| (result, pack_file))
    }

    // Otherwise, return an error.
    else { Err(ErrorKind::OpenPackFileInvalidExtension)? }
}

/// This function is used to take an open PackFile, encode it and save it into the disk. We return
/// a result with a message of success or error.
/// It requires:
/// - pack_file: a &mut pack_file::PackFile. It's the PackFile we are going to save.
/// - new_path: an Option<PathBuf> with the path were we are going to save the PackFile. None if we
///   are saving it in the same path it's when we opened it.
pub fn save_packfile(
    mut pack_file: &mut packfile::PackFile,
    new_path: Option<PathBuf>
) -> Result<()> {

    // If we haven't received a new_path, we assume the path is the original path of the file.
    // If that one is empty too (should never happen), we panic and cry.
    let pack_file_path = match new_path {

        // If we have received a new path...
        Some(new_path) => {

            // Update the data of the PackFile's path.
            pack_file.extra_data.file_name = new_path.file_name().unwrap().to_string_lossy().as_ref().to_owned();
            pack_file.extra_data.file_path = new_path;
            pack_file.extra_data.file_path.to_path_buf()
        },

        // If we haven't received a new path...
        None => {

            // If the current path exists, use it.
            if pack_file.extra_data.file_path.exists() {
                pack_file.extra_data.file_path.to_path_buf()
            }

            // Unless I fuck up the code in an impressive way, this is unreachable.
            else { unreachable!() }
        }
    };

    // We try to create the File.
    let mut file = BufWriter::new(File::create(pack_file_path)?);

    // And we try to save it.
    packfile::PackFile::save(&mut pack_file, &mut file)
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
    if pack_file.data.packedfile_exists(&tree_path) {

        // Create the theorical path of the PackedFile.
        let mut theorical_path = tree_path.to_vec();
        theorical_path.insert(0, pack_file.extra_data.file_name.to_owned());

        // Get the destination PackedFile. If it fails, CTD because it's a code problem.
        let packed_file = if let TreePathType::File((_, index)) = get_type_of_selected_path(&theorical_path, pack_file) {
            &mut pack_file.data.packed_files[index]
        } else { unreachable!() };

        // We get the data and his size...
        packed_file.data = vec![];
        let mut file = BufReader::new(File::open(&file_path)?);
        file.read_to_end(&mut packed_file.data)?;
        packed_file.size = packed_file.data.len() as u32;

        // Change his last modified time.
        packed_file.timestamp = get_last_modified_time_from_file(&file.get_ref());

        // And then, return sucess.
        Ok(())
    }

    // Otherwise, we add it as a new PackedFile.
    else {

        // We get the data and his size...
        let mut file_data = vec![];
        let mut file = BufReader::new(File::open(&file_path)?);
        file.read_to_end(&mut file_data)?;
        let file_size = file_data.len() as u32;

        // And then we make a PackedFile with it and save it.
        let packed_files = vec![packfile::PackedFile::read(file_size, get_last_modified_time_from_file(&file.get_ref()), tree_path, file_data); 1];
        pack_file.add_packedfiles(packed_files);
        Ok(())
    }
}

/// This function is used to add one or many PackedFiles to a PackFile (from another PackFile).
/// It returns a success or error message, depending on whether the PackedFile has been added, or not.
/// It requires:
/// - pack_file_source_buffer: &mut BufReader<File>, over the source PackFile.
/// - pack_file_source: a &pack_file::PackFile. It's the PackFile from we are going to take the PackedFile.
/// - pack_file_destination: a &mut pack_file::PackFile. It's the Destination PackFile for the PackedFile.
/// - complete_tree_path: the complete path (with PackFile) of the PackedFile or PackedFiles we want to add. A &[String] it is.
pub fn add_packedfile_to_packfile(
    pack_file_source_buffer: &mut BufReader<File>,
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
        TreePathType::File((_, index)) => {

            // Check if the PackedFile already exists in the destination.
            if pack_file_destination.data.packedfile_exists(real_path) {

                // Get the destination PackedFile. If it fails, CTD because it's a code problem.
                let mut packed_file = if let TreePathType::File((_, index)) = get_type_of_selected_path(complete_path, pack_file_destination) {
                    &mut pack_file_destination.data.packed_files[index]
                } else { unreachable!() };

                // Then, we get his data.
                packed_file.data = vec![0; pack_file_source.data.packed_files[index].size as usize];
                pack_file_source_buffer.seek(SeekFrom::Start(pack_file_source.packed_file_indexes[index]))?;
                pack_file_source_buffer.read_exact(&mut packed_file.data)?;

                // Return success.
                Ok(())
            }

            // Otherwise...
            else {

                // We get the PackedFile.
                let mut packed_file = pack_file_source.data.packed_files[index].clone();

                // Then, we get his data.
                packed_file.data = vec![0; packed_file.size as usize];
                pack_file_source_buffer.seek(SeekFrom::Start(pack_file_source.packed_file_indexes[index]))?;
                pack_file_source_buffer.read_exact(&mut packed_file.data)?;

                // Add it to the PackFile.
                pack_file_destination.add_packedfiles(vec![packed_file; 1]);

                // Return success.
                Ok(())
            }
        }

        // If the path is a folder...
        TreePathType::Folder(_) => {

            // For each PackedFile inside the folder...
            for (index, packed_file) in pack_file_source.data.packed_files.iter().enumerate() {

                // If it's one of the PackedFiles we want...
                if packed_file.path.starts_with(real_path) {

                    // Check if the PackedFile already exists in the destination.
                    if pack_file_destination.data.packedfile_exists(&packed_file.path) {

                        // Create the theorical path of the PackedFile.
                        let mut theorical_path = packed_file.path.to_vec();
                        theorical_path.insert(0, pack_file_source.extra_data.file_name.to_owned());

                        // Get the destination PackedFile. If it fails, CTD because it's a code problem.
                        let mut packed_file = if let TreePathType::File((_, index)) = get_type_of_selected_path(&theorical_path, pack_file_destination) {
                            &mut pack_file_destination.data.packed_files[index]
                        } else { unreachable!() };

                        // Then, we get his data.
                        packed_file.data = vec![0; pack_file_source.data.packed_files[index].size as usize];
                        pack_file_source_buffer.seek(SeekFrom::Start(pack_file_source.packed_file_indexes[index]))?;
                        pack_file_source_buffer.read_exact(&mut packed_file.data)?;
                    }

                    // Otherwise...
                    else {

                        // We get the PackedFile.
                        let mut packed_file = packed_file.clone();

                        // Then, we get his data.
                        packed_file.data = vec![0; packed_file.size as usize];
                        pack_file_source_buffer.seek(SeekFrom::Start(pack_file_source.packed_file_indexes[index]))?;
                        pack_file_source_buffer.read_exact(&mut packed_file.data)?;

                        // Add it to the PackFile.
                        pack_file_destination.add_packedfiles(vec![packed_file; 1]);
                    }
                }
            }

            // Return success.
            Ok(())
        },

        // If the path is the PackFile...
        TreePathType::PackFile => {

            // For each PackedFile inside the folder...
            for (index, packed_file) in pack_file_source.data.packed_files.iter().enumerate() {

                // Check if the PackedFile already exists in the destination.
                if pack_file_destination.data.packedfile_exists(&packed_file.path) {

                    // Create the theorical path of the PackedFile.
                    let mut theorical_path = packed_file.path.to_vec();
                    theorical_path.insert(0, pack_file_source.extra_data.file_name.to_owned());

                    // Get the destination PackedFile. If it fails, CTD because it's a code problem.
                    let mut packed_file = if let TreePathType::File((_, index)) = get_type_of_selected_path(&theorical_path, pack_file_destination) {
                        &mut pack_file_destination.data.packed_files[index]
                    } else { unreachable!() };

                    // Then, we get his data.
                    packed_file.data = vec![0; pack_file_source.data.packed_files[index].size as usize];
                    pack_file_source_buffer.seek(SeekFrom::Start(pack_file_source.packed_file_indexes[index]))?;
                    pack_file_source_buffer.read_exact(&mut packed_file.data)?;
                }

                // Otherwise...
                else {

                    // We get the PackedFile.
                    let mut packed_file = packed_file.clone();

                    // Then, we get his data.
                    packed_file.data = vec![0; packed_file.size as usize];
                    pack_file_source_buffer.seek(SeekFrom::Start(pack_file_source.packed_file_indexes[index]))?;
                    pack_file_source_buffer.read_exact(&mut packed_file.data)?;

                    // Add it to the PackFile.
                    pack_file_destination.add_packedfiles(vec![packed_file; 1]);
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
) {

    // Get what it's what we want to delete.
    match get_type_of_selected_path(tree_path, pack_file) {

        // If it's a file, easy job.
        TreePathType::File(packed_file_data) => pack_file.remove_packedfile(packed_file_data.1),

        // If it's a folder... it's a bit tricky.
        TreePathType::Folder(tree_path) => {

            // We create a vector to store the indexes of the files we are going to delete.
            let mut indexes = vec![];

            // For each PackedFile in our PackFile...
            for (index, packed_file) in pack_file.data.packed_files.iter().enumerate() {

                // If the PackedFile it's in our folder...
                if packed_file.path.starts_with(&tree_path) {

                    // Add his index to the indexes list.
                    indexes.push(index);
                }
            }

            // For each PackedFile we want to remove (in reverse), we remove it individually.
            indexes.iter().rev().for_each(|index| pack_file.remove_packedfile(*index));
        },

        // If it's a PackFile, easy job. For non-existant files, return an error.
        TreePathType::PackFile => pack_file.remove_all_packedfiles(),
        TreePathType::None => unreachable!(),
    }
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

        // If it's a file...
        TreePathType::File(packed_file_data) => {

            // We try to create the File.
            let mut file = BufWriter::new(File::create(&extracted_path)?);

            // And try to write it.
            match file.write_all(&pack_file.data.packed_files[packed_file_data.1].data){
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
            for packed_file in &pack_file.data.packed_files {

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
                    match file.write_all(&packed_file.data) {
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
            for packed_file in &pack_file.data.packed_files {

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
                match file.write_all(&packed_file.data){
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

/// This function is used to rename anything in the TreeView (yes, PackFile included).
/// It requires:
/// - pack_file: a &mut pack_file::PackFile. It's the PackFile opened.
/// - tree_path: a Vec<String> with the tree_path of the file to rename.
/// - new_name: the new name of the file to rename.
pub fn rename_packed_file(
    pack_file: &mut packfile::PackFile,
    tree_path: &[String],
    new_name: &str
) -> Result<()> {

    // First we check if the name is valid, and return an error if the new name is invalid.
    if new_name == tree_path.last().unwrap() {
        Err(ErrorKind::UnchangedInput)?
    }
    else if new_name.is_empty() {
        Err(ErrorKind::EmptyInput)?
    }
    else if new_name.contains(' ') {
        Err(ErrorKind::InvalidInput)?
    }

    // If we reach this point, we can rename the file/folder.
    else {
        match get_type_of_selected_path(tree_path, pack_file) {
            TreePathType::File(packed_file_data) => {
                // Now we create the new tree_path, while conserving the old one for checks
                let tree_path = packed_file_data.0;
                let index = packed_file_data.1;
                let mut new_tree_path = tree_path.to_owned();
                new_tree_path.pop();
                new_tree_path.push(new_name.to_string());

                if !pack_file.data.packedfile_exists(&new_tree_path) {
                    pack_file.data.packed_files[index as usize].path.pop();
                    pack_file.data.packed_files[index as usize].path.push(new_name.to_string());
                    Ok(())
                }
                else {
                    Err(ErrorKind::NameAlreadyInUseInThisPath)?
                }
            }
            TreePathType::Folder(tree_path) => {
                let mut new_tree_path = tree_path.to_owned();
                new_tree_path.pop();
                new_tree_path.push(new_name.to_string());

                // If the folder doesn't exist yet, we change the name of the folder we want to rename
                // in the path of every file that starts with his path.
                if !pack_file.data.folder_exists(&new_tree_path) {
                    let index_position = tree_path.len() - 1;
                    for packed_file in &mut pack_file.data.packed_files {
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
    index: usize,
) {
    let mut packed_file_data_encoded = Loc::save(packed_file_data_decoded).to_vec();
    let packed_file_data_encoded_size = packed_file_data_encoded.len() as u32;

    // Replace the old raw data of the PackedFile with the new one, and update his size.
    pack_file.data.packed_files[index].data.clear();
    pack_file.data.packed_files[index].data.append(&mut packed_file_data_encoded);
    pack_file.data.packed_files[index].size = packed_file_data_encoded_size;
}

/// This function saves the data of the edited DB PackedFile in the main PackFile after a change has
/// been done by the user. Checking for valid characters is done before this, so be careful to not break it.
pub fn update_packed_file_data_db(
    packed_file_data_decoded: &DB,
    pack_file: &mut packfile::PackFile,
    index: usize,
) {
    let mut packed_file_data_encoded = DB::save(packed_file_data_decoded);
    let packed_file_data_encoded_size = packed_file_data_encoded.len() as u32;

    // Replace the old raw data of the PackedFile with the new one, and update his size.
    pack_file.data.packed_files[index].data.clear();
    pack_file.data.packed_files[index].data.append(&mut packed_file_data_encoded);
    pack_file.data.packed_files[index].size = packed_file_data_encoded_size;
}

/// This function saves the data of the edited Text PackedFile in the main PackFile after a change has
/// been done by the user. Checking for valid characters is done before this, so be careful to not break it.
pub fn update_packed_file_data_text(
    packed_file_data_decoded: &[u8],
    pack_file: &mut packfile::PackFile,
    index: usize,
) {
    let mut packed_file_data_encoded = packed_file_data_decoded.to_vec();
    let packed_file_data_encoded_size = packed_file_data_encoded.len() as u32;

    // Replace the old raw data of the PackedFile with the new one, and update his size.
    pack_file.data.packed_files[index].data.clear();
    pack_file.data.packed_files[index].data.append(&mut packed_file_data_encoded);
    pack_file.data.packed_files[index].size = packed_file_data_encoded_size;
}

/// This function saves the data of the edited RigidModel PackedFile in the main PackFile after a change has
/// been done by the user. Checking for valid characters is done before this, so be careful to not break it.
/// This can fail in case a 0-Padded String of the RigidModel fails his encoding, so we check that too.
pub fn update_packed_file_data_rigid(
    packed_file_data_decoded: &RigidModel,
    pack_file: &mut packfile::PackFile,
    index: usize,
) -> Result<String> {
    let mut packed_file_data_encoded = RigidModel::save(packed_file_data_decoded)?.to_vec();
    let packed_file_data_encoded_size = packed_file_data_encoded.len() as u32;

    // Replace the old raw data of the PackedFile with the new one, and update his size.
    pack_file.data.packed_files[index].data.clear();
    pack_file.data.packed_files[index].data.append(&mut packed_file_data_encoded);
    pack_file.data.packed_files[index].size = packed_file_data_encoded_size;
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
    for i in &mut pack_file.data.packed_files {
        if i.path.starts_with(&["terrain".to_owned(), "tiles".to_owned(), "battle".to_owned(), "_assembly_kit".to_owned()]) &&
            i.path.last() != None {

            let x = i.path.last().unwrap();
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

                if i.data.windows(19).find(|window: &&[u8]
                        |String::from_utf8_lossy(window) == "AIH_SIEGE_AREA_NODE") != None {

                    let patch = "AIH_FORT_PERIMETER".to_string();
                    let index = i.data.windows(18)
                        .position(
                            |window: &[u8]
                            |String::from_utf8_lossy(window) == "AIH_DEFENSIVE_HILL");

                    if index != None {
                        for j in 0..18 {
                            i.data[index.unwrap() + (j as usize)] = patch.chars().nth(j).unwrap() as u8;
                        }
                        files_patched += 1;
                    }
                    if i.data.windows(18).find(|window: &&[u8]
                            |String::from_utf8_lossy(window) == "AIH_DEFENSIVE_HILL") != None {
                        multiple_defensive_hill_hints = true;
                    }
                }
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
            let file_name = vec![pack_file.extra_data.file_name.to_owned()];
            tree_path.splice(0..0, file_name.iter().cloned());

            // Get his type before deleting it.
            deleted_files_type.push(get_type_of_selected_path(&tree_path, &pack_file));

            // Delete the PackedFile.
            delete_from_packfile(pack_file, tree_path);
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
