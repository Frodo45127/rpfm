//---------------------------------------------------------------------------//
// Copyright (c) 2017-2022 Ismael Gutiérrez González. All rights reserved.
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

use rpfm_lib::games::{*, supported_games::SupportedGames};

/// This struct serves to hold the configuration used during the execution of the program.
pub struct Config {
	pub game: Option<GameInfo>,
	pub verbose: bool,
}

impl Config {

	/// This function creates a new Config struct configured for the provided game.
	pub fn new(game: &str, verbose: bool) -> Self {
        let supported_games = SupportedGames::new();
		Self {
            game: supported_games.game(game).cloned(),
			verbose,
		}
	}
}
