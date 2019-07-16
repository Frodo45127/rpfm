//---------------------------------------------------------------------------//
// Copyright (c) 2017-2019 Ismael Gutiérrez González. All rights reserved.
// 
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
// 
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

use std::path::PathBuf;

use rpfm_error::Result;
use rpfm_lib::packedfile::{import_tsv_to_binary_file, export_tsv_from_binary_file};

use crate::config::Config;

//---------------------------------------------------------------------------//
// 							DB/Loc Command Variants
//---------------------------------------------------------------------------//

/// This function imports a TSV file into a binary DB/Loc file. 
///
/// If no destination path was provided, it leaves the DB/Loc File in the same place as the tsv file, with the same name.
pub fn import_tsv(config: &Config, source_path: &str, destination_path: Option<&str>) -> Result<()> {
	if config.verbosity_level > 0 {
		println!("Operation: Import TSV File into Binary DB/Loc File.");
	}

	// Get the paths to pass to the import function.
	let source_path = PathBuf::from(source_path);
	let destination_path = match destination_path {
		Some(path) => PathBuf::from(path),
		None => {
			let mut path = source_path.to_path_buf();
			path.set_extension("");
			path
		}
	};

	import_tsv_to_binary_file(&config.schema, &source_path, &destination_path)
}

/// This function imports a TSV file into a binary DB/Loc file. 
///
/// If no destination path was provided, it leaves the DB/Loc File in the same place as the tsv file, with the same name.
pub fn export_tsv(config: &Config, source_path: &str, destination_path: Option<&str>) -> Result<()> {
	if config.verbosity_level > 0 {
		println!("Operation: Export Binary DB/Loc File into a TSV File.");
	}

	// Get the paths to pass to the import function.
	let source_path = PathBuf::from(source_path);
	let destination_path = match destination_path {
		Some(path) => PathBuf::from(path),
		None => {
			let mut path = source_path.to_path_buf();
			path.set_extension("tsv");
			path
		}
	};

	export_tsv_from_binary_file(&config.schema, &source_path, &destination_path)
}