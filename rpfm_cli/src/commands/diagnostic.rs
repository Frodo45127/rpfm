//---------------------------------------------------------------------------//
// Copyright (c) 2017-2022 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! This module contains the `Diagnostic` command's functions.


use log::info;

use std::path::PathBuf;

use rpfm_error::{ErrorKind, Result};

use rpfm_lib::dependencies::Dependencies;
use rpfm_lib::diagnostics::Diagnostics;
use rpfm_lib::packedfile::PackedFileType;
use rpfm_lib::packfile::PackFile;
use rpfm_lib::schema::Schema;
use rpfm_lib::SCHEMA;


use crate::config::Config;

//---------------------------------------------------------------------------//
//                          PackFile Command Variants
//---------------------------------------------------------------------------//

/// This function checks the PackFiles on the paths received for errors.
pub fn check(
    config: &Config,
    pack_files: &[&str],
    asskit_path: Option<&str>
) -> Result<()> {
    if config.verbosity_level > 0 {
        info!("Checking the following PackFiles for errors: {:?}", pack_files);
    }

    // Prepare the diagnostic data. If it fails, try to regenerate the dependencies.
    let mut dependencies = Dependencies::default();
    match &config.game_selected {
        Some(game_selected) => {
            *SCHEMA.write().unwrap() = Some(Schema::load(game_selected.get_schema_name())?);
            let asskit_path = asskit_path.map(PathBuf::from);

            // Load the PackFiles to check to memory.
            let pack_file_paths = pack_files.iter().map(PathBuf::from).collect::<Vec<PathBuf>>();
            let mut pack_file = PackFile::open_packfiles(&pack_file_paths, true, false, false)?;

            // Force decoding of table/locs, so they're in memory for the diagnostics to work.
            if let Some(ref schema) = *SCHEMA.read().unwrap() {
                let mut packed_files = pack_file.get_ref_mut_packed_files_by_types(&[PackedFileType::DB, PackedFileType::Loc], false);
                packed_files.iter_mut().for_each(|x| {
                    let _ = x.decode_no_locks(schema);
                });
            }

            if dependencies.rebuild(&[], false).is_err() {
                if config.verbosity_level > 0 {
                    info!("Dependencies rebuild failed. Regenerating…");
                }

                let version = game_selected.get_raw_db_version();

                dependencies.generate_dependencies_cache(&asskit_path, version)?;
                dependencies.save_to_binary()?;
                dependencies.rebuild(&[], false)?;
            }

            let mut diagnostics = Diagnostics::default();
            diagnostics.check(&pack_file, &mut dependencies);

            println!("{}", serde_json::to_string_pretty(&diagnostics)?);

            if config.verbosity_level > 0 {
                info!("File(s) added successfully to the PackFile.");
            }
        },
        None => return Err(ErrorKind::NoHTMLError("No Game Selected provided.".to_owned()).into()),
    }

    Ok(())
}
