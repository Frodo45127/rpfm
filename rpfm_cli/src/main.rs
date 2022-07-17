//---------------------------------------------------------------------------//
// Copyright (c) 2017-2022 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! This is a small CLI tool to interact with files used by Total War games.
//!
//! The purpouse of this tool is to allow users to automate certain parts of the mod building process.

use anyhow::Result;
use clap::Parser;

use std::path::PathBuf;
use std::process::exit;

use rpfm_lib::integrations::log::*;

use crate::app::{Cli, Commands, CommandsAnimPack, CommandsPack};
use crate::config::Config;

mod app;
mod commands;
mod config;

/// Guess you know what this function does....
fn main() {

    // Initialize the logging stuff here. This can fail depending on a lot of things, so trigger a console message if it fails.
    let logger = Logger::init(&PathBuf::from("."));
    if logger.is_err() {
        warn!("Logging initialization has failed. No logs will be saved.");
    }

    // Parse the entire cli command.
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

    // By default, print the game selected we're using, just in case some asshole starts complaining about broken Packs.
    if cli.verbose {
        info!("Game: {}", cli.game);
        info!("Verbose: {}", cli.verbose);
    }

    // Build the Config struct to remember the current configuration when processing stuff.
    let config = Config::new(&cli.game, cli.verbose);

    // Execute the commands.
    let result: Result<()> = match cli.command {
        Commands::Pack { commands } => match commands {
            CommandsPack::List { pack_path } => crate::commands::pack::list(&config, &pack_path),
            CommandsPack::Create { pack_path } => crate::commands::pack::create(&config, &pack_path),
            CommandsPack::Add { pack_path, file_path, folder_path } => crate::commands::pack::add(&config, &pack_path, &file_path, &folder_path),
            CommandsPack::Delete { pack_path, file_path, folder_path } => crate::commands::pack::delete(&config, &pack_path, &file_path, &folder_path),
            CommandsPack::Extract { pack_path, file_path, folder_path } => crate::commands::pack::extract(&config, &pack_path, &file_path, &folder_path),
            CommandsPack::SetFileType { pack_path, file_type } => crate::commands::pack::set_pack_type(&config, &pack_path, file_type),
        }

        Commands::AnimPack { commands } => match commands {
            CommandsAnimPack::List { pack_path } => crate::commands::animpack::list(&config, &pack_path),
            CommandsAnimPack::Create { pack_path } => crate::commands::animpack::create(&config, &pack_path),
            CommandsAnimPack::Add { pack_path, file_path, folder_path } => crate::commands::animpack::add(&config, &pack_path, &file_path, &folder_path),
            CommandsAnimPack::Delete { pack_path, file_path, folder_path } => crate::commands::animpack::delete(&config, &pack_path, &file_path, &folder_path),
            CommandsAnimPack::Extract { pack_path, file_path, folder_path } => crate::commands::animpack::extract(&config, &pack_path, &file_path, &folder_path),
        }
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
