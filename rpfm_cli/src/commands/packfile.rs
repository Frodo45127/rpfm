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
use prettytable::{Table, row, cell};

use std::path::PathBuf;

use rpfm_error::Result;
use rpfm_lib::packedfile::get_packed_file_type;
use rpfm_lib::add_file_to_packfile;
use rpfm_lib::delete_from_packfile;
use rpfm_lib::packfile::PathType;

use crate::config::Config;

//---------------------------------------------------------------------------//
// 							PackFile Command Variants
//---------------------------------------------------------------------------//

/// This function adds a File/Folder to a PackFile, then saves it.
pub fn add_to_packfile(
	config: &Config, 
	packfile: &str, 
	packed_file_path: &str, 
	destination_path: Option<&str>
) -> Result<()> {
	if config.verbosity_level > 0 {
		println!("Operation: Add File/Folder to PackFile.");
	}

	// Load the PackFile and the different PackedFiles to memory.
	let packfile_path = PathBuf::from(packfile);
	let mut packfile = rpfm_lib::open_packfiles(&[packfile_path], false, true, false)?;

	let packed_file_path = PathBuf::from(packed_file_path);
	let destination_path = match destination_path {
		Some(path) => path.split('/').map(|x| x.to_owned()).collect::<Vec<String>>(),
		None => vec![packed_file_path.file_name().unwrap().to_str().unwrap().to_owned()],
	};

	add_file_to_packfile(&mut packfile, &packed_file_path, destination_path)?;
	packfile.save()
}

/// This function deletes all the provided paths from the PackFile, then saves it.
pub fn delete_file_from_packfile(
	config: &Config, 
	packfile: &str, 
	path: &str, 
) -> Result<()> {
	if config.verbosity_level > 0 {
		println!("Operation: Add File/Folder to PackFile.");
	}

	// Load the PackFile and the different PackedFiles to memory.
	let packfile_path = PathBuf::from(packfile);
	let mut packfile = rpfm_lib::open_packfiles(&[packfile_path], false, true, false)?;

	let path = path.split('/').map(|x| x.to_owned()).collect::<Vec<String>>();
	delete_from_packfile(&mut packfile, &[PathType::File(path)]);
	packfile.save()
}

/// This function list the contents of the provided Packfile.
pub fn list_packfile_contents(config: &Config, packfile: &str) -> Result<()> {
	if config.verbosity_level > 0 {
		println!("Operation: List PackFile Contents.");
	}
	let packfile_path = PathBuf::from(packfile);
	let packfile = rpfm_lib::open_packfiles(&[packfile_path], false, true, false)?;
    
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