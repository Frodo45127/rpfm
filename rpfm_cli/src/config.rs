//---------------------------------------------------------------------------//
// Copyright (c) 2017-2023 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! This module contains the basic configuration for the CLI tool.
//!
//! It has to be initialized at the beginning, before any command gets executed.

use anyhow::{anyhow, Result};
use directories::ProjectDirs;

use std::fs::DirBuilder;
use std::path::PathBuf;

use rpfm_lib::games::{*, supported_games::SupportedGames};

/// This struct serves to hold the configuration used during the execution of the program.
pub struct Config {
	pub game: Option<GameInfo>,
	pub verbose: bool,
}

impl Config {

	/// This function creates a new Config struct configured for the provided game.
	pub fn new(game: &str, verbose: bool) -> Self {
        let supported_games = SupportedGames::default();
		Self {
            game: supported_games.game(game).cloned(),
			verbose,
		}
	}
}

/// Function to initialize the config folder, so RPFM can use it to store his stuff.
///
/// This can fail, so if this fails, better stop the program and check why it failed.
#[must_use = "Many things depend on this folder existing. So better check this worked."]
pub fn init_config_path() -> Result<()> {

    DirBuilder::new().recursive(true).create(error_path()?)?;

    Ok(())
}

/// This function returns the current config path, or an error if said path is not available.
///
/// Note: On `Debug´ mode this project is the project from where you execute one of RPFM's programs, which should be the root of the repo.
pub fn config_path() -> Result<PathBuf> {
    if cfg!(debug_assertions) { std::env::current_dir().map_err(From::from) } else {
        match ProjectDirs::from("com", "FrodoWazEre", "rpfm") {
            Some(proj_dirs) => Ok(proj_dirs.config_dir().to_path_buf()),
            None => Err(anyhow!("Failed to get the config path."))
        }
    }
}

/// This function returns the path where crash logs are stored.
pub fn error_path() -> Result<PathBuf> {
    Ok(config_path()?.join("error"))
}
