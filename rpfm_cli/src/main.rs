//---------------------------------------------------------------------------//
// Copyright (c) 2017-2019 Ismael Gutiérrez González. All rights reserved.
// 
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
// 
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

// This is the CLI version of RPFM.

use clap::{Arg, App, SubCommand};
use colored::*;
use log::{error, info, warn};
use simplelog::{CombinedLogger, LevelFilter, TerminalMode, TermLogger, WriteLogger};
use simple_logger;

use std::env;
use std::process::exit;
use std::fs::File;

use rpfm_error::ctd::CrashReport;
use rpfm_lib::settings::Settings;
use rpfm_lib::config::get_config_path;

use crate::config::Config;

const VERSION: &str = env!("CARGO_PKG_VERSION");
const PROGRAM_NAME: &str = "Rusted PackFile Manager - CLI Version";
const AUTHOR: &str = env!("CARGO_PKG_AUTHORS");

// Modules used by this tool.
pub mod commands;
pub mod config;

// Guess you know what this function does....
fn main() {

    // In Release Builds, initiallize the logger, so we get messages in the terminal and recorded to disk.
    if !cfg!(debug_assertions) { 
        CrashReport::init().unwrap();
        CombinedLogger::init(
            vec![
                TermLogger::new(LevelFilter::Info, simplelog::Config::default(), TerminalMode::Mixed).unwrap(),
                WriteLogger::new(LevelFilter::Info, simplelog::Config::default(), File::create(get_config_path().unwrap().join("rpfm_cli.log")).unwrap()),
            ]
        ).unwrap();
    }

    // Simplelog do not work properly with custom terminals, like the one in Sublime Text. So, for debug builds,
    // we use simple_logger instead.
    else {
        simple_logger::init().unwrap();
    }

    // Matches: here we build the entire command parsing for the tool, courtesy of Clap.
    // Also, clap autogenerates certaing commands, like help and version, so those are not needed.
    let mut app = App::new(PROGRAM_NAME)
        .version(VERSION)
        .author(AUTHOR)
        .about("CLI Version of RPFM. Ready to automate the most boring parts of your modding.")
        .arg(Arg::with_name("v")
            .short("v")
            .multiple(true)
            .help("Sets the level of verbosity"))

        .arg(Arg::with_name("settings")
            .short("s")
            .long("settings")
            .value_name("FILE")
            .help("Sets a custom Settings File. Otherwise, RPFM's normal Setting file will be used. If that one doesn't exist, default settings will be used.")
            .takes_value(true))

        .arg(Arg::with_name("game")
            .short("g")
            .long("game")
            .value_name("GAME")
            .help("Sets the 'Game' all the commands will be tailored too. This affects what schemas will be use when dealing with DB Tables, the format of the PackFiles... If it's not set, the default game from the settings will be used.")
            .possible_values(&["three_kingdoms", "warhammer_2", "warhammer", "thrones_of_britannia", "attila", "rome_2", "shogun_2", "napoleon", "empire", "arena"])
            .takes_value(true))

        // `PackFile` Subcommand. Every command that edits PackFiles in any way goes here.
        .subcommand(SubCommand::with_name("packfile")
            .about("Allows PackFile editing.")

            // PackFile path. Next, we ask for the operation we want to do.
            .arg(Arg::with_name("packfile")
                .help("Path of the PackFile to edit.")
                .value_name("PACKFILE PATH")
                .required(true)
                .index(1))
            
            .arg(Arg::with_name("add")
                .short("a")
                .long("add")
                .value_name("FILE/FOLDER PATHS")
                .help("Adds one or more files/folders to the PackFile. If one of the files already exists, it'll replace it.")
                .takes_value(true)
                .min_values(1))
            .arg(Arg::with_name("delete")
                .short("d")
                .long("delete")
                .value_name("FILE/FOLDER PATHS")
                .help("Deletes one or more files/folders from the PackFile.")
                .takes_value(true)
                .min_values(1))
            .arg(Arg::with_name("list")
                .short("l")
                .long("list")
                .help("Lists the contents of the PackFile.")))

        // `Table` Subcommand. Every command that allows you to manipulate DB/Loc Tables in any way goes here.
        .subcommand(SubCommand::with_name("table")
            .about("Allows you to manipulate in multiple ways DB/LOC Tables.")
            .arg(Arg::with_name("import")
                .short("i")
                .long("import")
                .value_name("TSV FILE - DESTINATION FILE")
                .help("Import a compatible TSV file as a DB/LOC table.")
                .takes_value(true)
                .min_values(1)
                .max_values(2))
            .arg(Arg::with_name("export")
                .short("e")
                .long("export")
                .value_name("DB FILE - DESTINATION FILE")
                .help("Export a DB/LOC Table's data to a TSV file.")
                .takes_value(true)
                .min_values(1)
                .max_values(2)))

        // `Schemas` Subcommand. Basically, here goes commands destined to keep schemas up-to-date.
        .subcommand(SubCommand::with_name("schema")
            .about("Allows you to keep your schemas up-to-date.")
            .arg(Arg::with_name("update")
                .short("u")
                .long("update")
                .takes_value(false)));

    // If no arguments where provided, trigger the "help" message. Otherwise, get the matches and continue.
    if env::args_os().len() <= 1 { app.print_help().unwrap(); exit(0) }
    let matches = app.clone().get_matches();

    // Set the verbosity level.
    let verbosity_level = matches.occurrences_of("v"); 

    // Tries to load the settings in this order:
    // - The provider settings file.
    // - The default RPFM file.
    // - A default settings set.
    let settings = match matches.value_of("settings") {
        Some(settings_path) => {
            match Settings::load_from_file(settings_path) {
                Ok(settings) => { if verbosity_level > 0 { info!("Loaded settings from: {}", settings_path); } settings },
                Err(_) => { if verbosity_level > 0 { warn!("Failed to load settings from: {}. Loaded default settings instead.", settings_path); } Settings::new() },
            }
        },
        None => {
            match Settings::load() {
                Ok(settings) => { if verbosity_level > 0 { info!("Loaded settings from RPFM settings folder."); } settings },
                Err(_) => { if verbosity_level > 0 { warn!("Failed to load settings from RPFM settings folder. Loaded default settings instead."); } Settings::new() },
            }
        }
    };

    // Get the game selected.
    let game_selected = match matches.value_of("game") {
        Some(game) => game.to_owned(),
        None => settings.settings_string["default_game"].to_owned(),
    };

    // By default, print the game selected we're using, just in case some asshole starts complaining about broken PackFiles.
    if verbosity_level > 0 {
        info!("Game Selected: {}", game_selected);
        info!("Verbosity level: {}", if verbosity_level > 3 { 3 } else { verbosity_level });
    }

    // Build the Config struct to remember the current configuration when processing stuff.
    let config = match Config::new(game_selected, settings, verbosity_level) {
        Ok(config) => config,
        Err(error) => { error!("{} {}","Error:".red().bold(), error.to_terminal()); exit(1) }
    };

    // Code for PackFile commands.
    let result = match matches.subcommand() {
        ("packfile", Some(matches)) => commands::command_packfile(&config, matches),
        ("table", Some(matches)) => commands::command_table(&config, matches),
        ("schema", Some(matches)) => commands::command_schema(&config, matches),
        _ => { app.print_help().unwrap(); Ok(()) }
    };

    match result {
        Ok(_) => exit(0),
        Err(error) => { error!("{}", error.to_terminal()); exit(1) },
    }
}
