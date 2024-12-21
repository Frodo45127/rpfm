//---------------------------------------------------------------------------//
// Copyright (c) 2017-2024 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! This module contains the `Dependencies` command functions.

use anyhow::{anyhow, Result};

use std::path::{Path, PathBuf};

use rpfm_extensions::dependencies::Dependencies;
use rpfm_lib::integrations::log::*;
use rpfm_lib::schema::Schema;

use crate::config::Config;

//---------------------------------------------------------------------------//
//                          Dependencies Command Variants
//---------------------------------------------------------------------------//

/// This function generates a dependencies cache for the game_path provided and saves it to a file.
pub fn generate(config: &Config, pak_path: &Path, game_path: &Path, schema_path: &Path, assembly_kit_path: &Option<PathBuf>) -> Result<()> {
    if config.verbose {
        info!("Generating dependencies at the following path: {}.", pak_path.to_string_lossy().to_string());
    }

    match &config.game {
        Some(game_info) => {

            let schema = Schema::load(schema_path, None)?;
            let mut dependencies = Dependencies::generate_dependencies_cache(&Some(schema), game_info, game_path, assembly_kit_path, false)?;
            dependencies.save(pak_path)?;

            if config.verbose {
                info!("Dependencies generated at path {}.", pak_path.to_string_lossy().to_string());
            }

            Ok(())
        }
        None => Err(anyhow!("No Game provided.")),
    }
}
