//---------------------------------------------------------------------------//
// Copyright (c) 2017-2020 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

use log::info;
use std::path::PathBuf;

use rpfm_error::{ErrorKind, Result};
use rpfm_lib::packedfile::table::db::DB;
use rpfm_lib::schema::Schema;
use rpfm_lib::SUPPORTED_GAMES;

use crate::config::Config;

//---------------------------------------------------------------------------//
// 							DB/Loc Command Variants
//---------------------------------------------------------------------------//

/// This function imports a TSV file into a binary DB/Loc file.
///
/// If no destination path was provided, it leaves the DB/Loc File in the same place as the tsv file, with the same name.
pub fn import_tsv(
    config: &Config,
    source_paths: &[&str],
) -> Result<()> {

	if config.verbosity_level > 0 {
		source_paths.iter().for_each(|x| info!("Import TSV File as Binary File: {}", x));
	}

    match &config.game_selected {
        Some(game_selected) => {
            let schema = Schema::load(&SUPPORTED_GAMES[&**game_selected].schema)?;
        	let source_paths = source_paths.iter().map(PathBuf::from).collect::<Vec<PathBuf>>();
        	let result = DB::import_tsv_to_binary_file(&schema, &source_paths);
            info!("All TSV files imported to binary.");
            result
        },
        None => Err(ErrorKind::NoHTMLError("No Game Selected provided.".to_owned()).into()),
    }
}

/// This function imports a TSV file into a binary DB/Loc file.
///
/// If no destination path was provided, it leaves the DB/Loc File in the same place as the tsv file, with the same name.
pub fn export_tsv(
    config: &Config,
    source_paths: &[&str],
) -> Result<()> {
	if config.verbosity_level > 0 {
		source_paths.iter().for_each(|x| info!("Export Binary File as TSV: {}", x));
	}

    match &config.game_selected {
        Some(game_selected) => {
            let schema = Schema::load(&SUPPORTED_GAMES[&**game_selected].schema)?;
            let source_paths = source_paths.iter().map(PathBuf::from).collect::<Vec<PathBuf>>();
            let result = DB::export_tsv_from_binary_file(&schema, &source_paths);
            info!("All binary files exported to TSV.");
            result
        },
        None => Err(ErrorKind::NoHTMLError("No Game Selected provided.".to_owned()).into()),
    }
}
