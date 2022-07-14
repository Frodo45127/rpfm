//---------------------------------------------------------------------------//
// Copyright (c) 2017-2022 Ismael Gutiérrez González. All rights reserved.
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


//use colored::*;
//use log::{error, info, warn};

use crate::config::Config;
use app::CommandsPack;
//use std::env;
use clap::Parser;
use std::path::PathBuf;
use std::process::exit;

use rpfm_lib::integrations::log::*;
//use crate::config::Config;
//use crate::logger::initialize_logs;
use crate::app::{Cli, Commands};

use anyhow::Result;

// Modules used by this tool.
pub mod app;
pub mod commands;
pub mod config;
//pub mod logger;

/// Guess you know what this function does....
fn main() {

    // Initialize the logging stuff here. This can fail depending on a lot of things, so trigger a console message if it fails.
    let logger = Logger::init(&PathBuf::from("."));
    if logger.is_err() {
        warn!("Logging initialization has failed. No logs will be saved.");
    }

    let cli = Cli::parse();

    /*
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
    */
    // By default, print the game selected we're using, just in case some asshole starts complaining about broken PackFiles.
    if cli.verbose {
        info!("Game: {}", cli.game);
        info!("Verbose: {}", cli.verbose);
    }

    // Build the Config struct to remember the current configuration when processing stuff.
    let config = Config::new(&cli.game, cli.verbose);

    // If we reached here, execute the commands.
    let result: Result<()> = match cli.command {
        Commands::Pack { commands } => match commands {
            CommandsPack::List { pack_path } => crate::commands::pack::list(&config, &pack_path),
            CommandsPack::Create { pack_path } => crate::commands::pack::create(&config, &pack_path),
            CommandsPack::Add { pack_path, file_path, folder_path } => crate::commands::pack::add(&config, &pack_path, &file_path, &folder_path),
            CommandsPack::Delete { pack_path, file_path, folder_path } => crate::commands::pack::delete(&config, &pack_path, &file_path, &folder_path),
            CommandsPack::Extract { pack_path, file_path, folder_path } => crate::commands::pack::extract(&config, &pack_path, &file_path, &folder_path),
        }
        //Some(("diagnostic", matches)) => commands::command_diagnostic(&config, matches, asskit_db_path),
        //Some(("packfile", matches)) => commands::command_packfile(&config, matches, packfile),
        //Some(("table", matches)) => commands::command_table(&config, matches, packfile),
        //Some(("schema", matches)) => commands::command_schema(&config, matches),
        //_ => { Ok(()) }
    };

    // Output the result of the commands.
    match result {
        Ok(_) => exit(0),
        Err(error) => {
            error!("{}", error);
            exit(1)
        },
    }
}
