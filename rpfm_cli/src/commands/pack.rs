//---------------------------------------------------------------------------//
// Copyright (c) 2017-2022 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! This module contains the `PackFile` command's functions.

use std::collections::BTreeMap;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use rpfm_lib::{error::Result, files::Decodeable};
use rpfm_lib::files::{Container, DecodeableExtraData, pack::Pack};
use rpfm_lib::integrations::log::*;
use rpfm_lib::utils::last_modified_time_from_file;

use crate::config::Config;

//---------------------------------------------------------------------------//
// 							PackFile Command Variants
//---------------------------------------------------------------------------//
/*
/// This function adds one or more Files to a PackFile, then saves it.
pub fn add_files(
	config: &Config,
	packfile: &str,
	packed_file_path: &[&str],
	destination_path: &str
) -> Result<()> {
	if config.verbosity_level > 0 {
		info!("Adding File(s) to the PackFile: {}", packfile);
	}

	// Load the PackFile and the different PackedFiles to memory.
	let packfile_path = PathBuf::from(packfile);
	let mut packfile = PackFile::open_packfiles(&[packfile_path], true, false, false)?;

	let destination_path = if destination_path == "." { vec![] } else { destination_path.split('/').map(|x| x.to_owned()).collect::<Vec<String>>() };
    let packed_file_paths = packed_file_path.iter()
        .map(|x| {
            let mut full_path = destination_path.to_vec();
            full_path.append(&mut x.split('/').map(|x| x.to_owned()).collect());
            (PathBuf::from(x), full_path)
        })
        .collect::<Vec<(PathBuf, Vec<String>)>>();
	packfile.add_from_files(&packed_file_paths, true)?;
	let result = packfile.save(None);

    if config.verbosity_level > 0 {
        info!("File(s) added successfully to the PackFile.");
    }

    result
}

/// This function adds a Folder to a `PackFile`, then saves it.
pub fn add_folders(
	config: &Config,
	packfile: &str,
	folder_paths: &[&str],
    destination_path: &str
) -> Result<()> {
	if config.verbosity_level > 0 {
		info!("Adding Folder(s) to the PackFile: {}.", packfile);
	}

	// Load the PackFile and the different PackedFiles to memory.
	let packfile_path = PathBuf::from(packfile);
	let mut packfile = PackFile::open_packfiles(&[packfile_path], true, false, false)?;

    let destination_path = if destination_path == "." { vec![] } else { destination_path.split('/').map(|x| x.to_owned()).collect::<Vec<String>>() };
    let folder_paths = folder_paths.iter()
        .map(|x| {
            let full_path = destination_path.to_vec();
            (PathBuf::from(x), full_path)
        })
        .collect::<Vec<(PathBuf, Vec<String>)>>();

	packfile.add_from_folders(&folder_paths, &None, true, true)?;
	let result = packfile.save(None);

    if config.verbosity_level > 0 {
        info!("Folder(s) added successfully to the PackFile.");
    }

    result
}

/// This function deletes all the PackedFiles with the provided paths from the PackFile, then saves it.
pub fn delete_files(
    config: &Config,
    packfile: &str,
    paths: &[&str],
) -> Result<()> {
    if config.verbosity_level > 0 {
        paths.iter().for_each(|x| info!("Deleting the following file from a PackFile: {}", x));
    }

    // Load the PackFile and the different PackedFiles to memory.
    let packfile_path = PathBuf::from(packfile);
    let mut packfile = PackFile::open_packfiles(&[packfile_path], true, false, false)?;

    paths.iter().map(|x| x.split('/').map(|x| x.to_owned()).collect::<Vec<String>>())
        .for_each(|x| packfile.remove_packed_file_by_path(&x));
    let result = packfile.save(None);

    if config.verbosity_level > 0 {
        info!("Files successfully deleted from the PackFile.");
    }

    result
}

/// This function deletes all the Folders with the provided paths from the PackFile, then saves it.
pub fn delete_folders(
    config: &Config,
    packfile: &str,
    paths: &[&str],
) -> Result<()> {
    if config.verbosity_level > 0 {
        paths.iter().for_each(|x| info!("Deleting the following folder from a PackFile: {}", x));
    }

    // Load the PackFile and the different PackedFiles to memory.
    let packfile_path = PathBuf::from(packfile);
    let mut packfile = PackFile::open_packfiles(&[packfile_path], true, false, false)?;

    paths.iter().map(|x| x.split('/').map(|x| x.to_owned()).collect::<Vec<String>>())
        .for_each(|x| { packfile.remove_packed_files_by_type(&[PathType::Folder(x)]); });
    let result = packfile.save(None);

    if config.verbosity_level > 0 {
        info!("Folders successfully deleted from the PackFile.");
    }

    result
}
/// This function extracts all the PackedFiles with the provided paths from the PackFile to the provided directory, if it's valid.
pub fn extract_files(
	config: &Config,
	packfile: &str,
	paths: &[&str],
    destination_path: &str
) -> Result<()> {
	if config.verbosity_level > 0 {
        paths.iter().for_each(|x| info!("Extracting the following file from a PackFile: {}", x));
	}

    let destination_path = PathBuf::from(destination_path);
    if !destination_path.is_dir() {
        return Err(ErrorKind::IOReadFolder(destination_path).into());
    }

	// Load the PackFile and the different PackedFiles to memory.
	let packfile_path = PathBuf::from(packfile);
	let mut packfile = PackFile::open_packfiles(&[packfile_path], true, false, false)?;

	let result = paths.iter().map(|x| x.split('/').map(|x| x.to_owned()).collect::<Vec<String>>())
        .try_for_each(|x| packfile.extract_packed_file_by_path(&x, &destination_path, true));

    if config.verbosity_level > 0 {
        info!("Files successfully extracted from the PackFile.");
    }

    result
}

/// This function extracts all the folders with the provided paths from the PackFile to the provided directory, if it's valid.
pub fn extract_folders(
	config: &Config,
	packfile: &str,
	paths: &[&str],
    destination_path: &str
) -> Result<()> {
    if config.verbosity_level > 0 {
        paths.iter().for_each(|x| info!("Extracting the following folder from a PackFile: {}", x));
    }

    let destination_path = PathBuf::from(destination_path);
    if !destination_path.is_dir() {
        return Err(ErrorKind::IOReadFolder(destination_path).into());
    }

    // Load the PackFile and the different PackedFiles to memory.
    let packfile_path = PathBuf::from(packfile);
    let mut packfile = PackFile::open_packfiles(&[packfile_path], true, false, false)?;

    let paths = paths.iter().map(|x| x.split('/').map(|x| x.to_owned()).collect::<Vec<String>>()).map(PathType::Folder).collect::<Vec<PathType>>();
    packfile.extract_packed_files_by_type(&paths, &destination_path, true)?;

    if config.verbosity_level > 0 {
        info!("Folders successfully extracted from the PackFile.");
    }

    Ok(())
}
*/
/// This function list the contents of the provided Packfile.
pub fn list(config: &Config, path: &Path) -> Result<()> {

    if config.verbose {
		info!("Listing PackFile Contents.");
	}

    let mut reader = BufReader::new(File::open(path)?);
    let path_str = path.to_str().unwrap();

    let mut extra_data = DecodeableExtraData::default();
    extra_data.set_disk_file_path(Some(&path_str));
    extra_data.set_timestamp(last_modified_time_from_file(reader.get_ref())?);

    let pack = Pack::decode(&mut reader, Some(extra_data))?;
    let files: BTreeMap<_, _> = pack.files().iter().collect();
    for (path, _) in files {
        println!("{}", path);
    }

	Ok(())
}
/*
/// This function creates a new packfile with the provided path.
pub fn new_packfile(config: &Config, packfile: &str) -> Result<()> {
    if config.verbosity_level > 0 {
        info!("Creating New Packfile.");
    }

    match &config.game_selected {
        Some(game_selected) => {
            let packfile_path = PathBuf::from(packfile);
            let mut packfile = PackFile::new_with_name("x", game_selected.get_pfh_version_by_file_type(PFHFileType::Mod));
            packfile.save(Some(packfile_path))
        }
        None => Err(ErrorKind::NoHTMLError("No Game Selected provided.".to_owned()).into()),
    }
}
*/
