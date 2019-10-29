//---------------------------------------------------------------------------//
// Copyright (c) 2017-2019 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

use bytesize::ByteSize;
use log::info;
use prettytable::{Table, row, cell};

use std::path::PathBuf;

use rpfm_error::Result;
use rpfm_lib::packedfile::get_packed_file_type;
use rpfm_lib::packfile::{PackFile, PathType};

use crate::config::Config;

//---------------------------------------------------------------------------//
// 							PackFile Command Variants
//---------------------------------------------------------------------------//

/// This function adds a File/Folder to a PackFile, then saves it.
pub fn add_file(
	config: &Config,
	packfile: &str,
	packed_file_path: &str,
	destination_path: Option<&str>
) -> Result<()> {
	if config.verbosity_level > 0 {
		info!("Operation: Add File to PackFile.");
	}

	// Load the PackFile and the different PackedFiles to memory.
	let packfile_path = PathBuf::from(packfile);
	let mut packfile = PackFile::open_packfiles(&[packfile_path], true, false, false)?;

	let packed_file_path = PathBuf::from(packed_file_path);
	let destination_path = match destination_path {
		Some(path) => path.split('/').map(|x| x.to_owned()).collect::<Vec<String>>(),
		None => vec![packed_file_path.file_name().unwrap().to_str().unwrap().to_owned()],
	};

	packfile.add_from_file(&packed_file_path, destination_path, true)?;
	packfile.save(None)
}

/// This function adds a Folder to a `PackFile`, then saves it.
pub fn add_folder(
	config: &Config,
	packfile: &str,
	packed_file_path: &str,
) -> Result<()> {
	if config.verbosity_level > 0 {
		info!("Operation: Add Folder to PackFile.");
	}

	// Load the PackFile and the different PackedFiles to memory.
	let packfile_path = PathBuf::from(packfile);
	let mut packfile = PackFile::open_packfiles(&[packfile_path], true, false, false)?;

	let packed_file_path = PathBuf::from(packed_file_path);
	packfile.add_from_folder(&packed_file_path, true)?;
	packfile.save(None)
}

/// This function deletes all the provided paths from the PackFile, then saves it.
pub fn delete_file(
	config: &Config,
	packfile: &str,
	path: &str,
) -> Result<()> {
	if config.verbosity_level > 0 {
		info!("Operation: Delete File from PackFile.");
	}

	// Load the PackFile and the different PackedFiles to memory.
	let packfile_path = PathBuf::from(packfile);
	let mut packfile = PackFile::open_packfiles(&[packfile_path], true, false, false)?;

	let path = path.split('/').map(|x| x.to_owned()).collect::<Vec<String>>();
	packfile.remove_packed_file_by_path(&path);
	packfile.save(None)
}

pub fn delete_folder(
	config: &Config,
	packfile: &str,
	path: &str,
) -> Result<()> {
	if config.verbosity_level > 0 {
		info!("Operation: Delete Folder from PackFile.");
	}

	// Load the PackFile and the different PackedFiles to memory.
	let packfile_path = PathBuf::from(packfile);
	let mut packfile = PackFile::open_packfiles(&[packfile_path], true, false, false)?;

	let path = path.split('/').map(|x| x.to_owned()).collect::<Vec<String>>();
	packfile.remove_packed_files_by_type(&[PathType::Folder(path)]);
	packfile.save(None)
}

/// This function list the contents of the provided Packfile.
pub fn list_packfile_contents(config: &Config, packfile: &str) -> Result<()> {
	if config.verbosity_level > 0 {
		info!("Operation: List PackFile Contents.");
	}
	let packfile_path = PathBuf::from(packfile);
	let packfile = PackFile::open_packfiles(&[packfile_path], true, false, false)?;

	let mut table = Table::new();
    table.add_row(row!["PackedFile Path", "Type", "Size"]);
    for file in packfile.get_ref_all_packed_files() {
    	let packedfile_type = get_packed_file_type(&file.get_path());
    	let size = ByteSize::kib((file.get_size() / 1024).into());
    	table.add_row(row![file.get_path().join("/"), packedfile_type, size]);
    }

	table.printstd();
	Ok(())

}
