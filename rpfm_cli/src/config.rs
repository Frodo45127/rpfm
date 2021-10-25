//---------------------------------------------------------------------------//
// Copyright (c) 2017-2020 Ismael Gutiérrez González. All rights reserved.
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

use rpfm_error::Result;
use rpfm_lib::games::*;
use rpfm_lib::settings::init_config_path;
use rpfm_lib::SUPPORTED_GAMES;

/// This struct serves to hold the configuration used during the execution of the program.
pub struct Config {
	pub game_selected: Option<GameInfo>,
	pub verbosity_level: u8,
}

/// Implementation of `Config`.
impl Config {

	/// This function creates a new Config struct configured for the provided game.
	pub fn new(game_selected: String, verbosity_level: u8) -> Result<Self> {
		init_config_path()?;
		Ok(Self {
            game_selected: Some(SUPPORTED_GAMES.get_supported_game_from_key(&game_selected)?.clone()),
			verbosity_level,
		})
	}
}
