//---------------------------------------------------------------------------//
// Copyright (c) 2017-2020 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

/*!
This crate is the `CLI` version of RPFM, who fought in the splitting war as a new power and managed to stablish itself by the end of the war.
!*/

use colored::*;
use log::{error, info, warn};

use std::env;
use std::process::exit;

use crate::config::Config;
use crate::logger::initialize_logs;
use crate::app::initialize_app;

// Modules used by this tool.
pub mod app;
pub mod commands;
pub mod config;
pub mod logger;

/// Guess you know what this function does....
fn main() {

    // Initialize the logging stuff here. This can fail depending on a lot of things, so trigger a console message if it fails.
    if initialize_logs().is_err() {
        warn!("Logging initialization has failed. No logs will be saved.");
    }

    // Initialize the App itself.
    let mut app = initialize_app();

    // If no arguments where provided, trigger the "help" message. Otherwise, get the matches and continue.
    if env::args_os().len() <= 1 { app.print_help().unwrap(); exit(0) }
    let matches = app.get_matches();

    // Set the verbosity level and game selected, based on the arguments provided.
    let verbosity_level = if matches.occurrences_of("v") > 3 { 3 } else { matches.occurrences_of("v") as u8 };
    let packfile = matches.value_of("packfile");
    let asskit_db_path = matches.value_of("assdb");
    let game_selected = match matches.value_of("game") {
        Some(game) => game.to_owned(),
        None => "three_kingdoms".to_owned(),
    };

    // By default, print the game selected we're using, just in case some asshole starts complaining about broken PackFiles.
    if verbosity_level > 0 {
        info!("Game Selected: {}", game_selected);
        info!("Verbosity level: {}", verbosity_level);
    }

    // Build the Config struct to remember the current configuration when processing stuff.
    let config = match Config::new(game_selected, verbosity_level) {
        Ok(config) => config,
        Err(error) => { error!("{} {}","Error:".red().bold(), error.to_terminal()); exit(1) }
    };

    // If we reached here, execute the commands.
    let result = match matches.subcommand() {
        ("diagnostic", Some(matches)) => commands::command_diagnostic(&config, matches, asskit_db_path),
        ("packfile", Some(matches)) => commands::command_packfile(&config, matches, packfile),
        ("table", Some(matches)) => commands::command_table(&config, matches, packfile),
        ("schema", Some(matches)) => commands::command_schema(&config, matches),
        _ => { Ok(()) }
    };

    // Output the result of the commands.
    match result {
        Ok(_) => exit(0),
        Err(error) => { error!("{}", error.to_terminal()); exit(1) },
    }
}
