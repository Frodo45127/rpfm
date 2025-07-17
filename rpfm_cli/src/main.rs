//---------------------------------------------------------------------------//
// Copyright (c) 2017-2024 Ismael Gutiérrez González. All rights reserved.
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
use lazy_static::lazy_static;

use std::path::PathBuf;
use std::process::exit;
use std::sync::{Arc, RwLock};

use rpfm_lib::integrations::log::*;

use crate::app::{Cli, Commands, CommandsAnimPack, CommandsDependencies, CommandsPack, CommandsPortraitSettings, CommandsSchemas};
use crate::config::*;

mod app;
mod commands;
mod config;

// Statics, so we don't need to pass them everywhere to use them.
lazy_static! {

    /// Sentry client guard, so we can reuse it later on and keep it in scope for the entire duration of the program.
    static ref SENTRY_GUARD: Arc<RwLock<ClientInitGuard>> = Arc::new(RwLock::new(Logger::init(&{
        init_config_path().expect("Error while trying to initialize config path. We're fucked.");
        error_path().unwrap_or_else(|_| PathBuf::from("."))
    }, true, true, release_name!()).unwrap()));
}

const SENTRY_DSN_KEY: &str = "https://1bee0e6bab154cd988b309096df932b8@o152833.ingest.sentry.io/4504850526699520";

/// Guess you know what this function does....
fn main() {

    // Setup sentry's dsn for error reporting.
    *SENTRY_DSN.write().unwrap() = SENTRY_DSN_KEY.to_owned();

    // Access the guard to make sure it gets initialized.
    if SENTRY_GUARD.read().unwrap().is_enabled() {
        info!("Sentry Logging support enabled. Starting...");
    } else {
        info!("Sentry Logging support disabled. Starting...");
    }

    // Parse the entire cli command.
    let cli = Cli::parse();

    if cli.verbose {
        info!("Game: {}", cli.game);
        info!("Verbose: {}", cli.verbose);
    }

    // Initialize the logging stuff here. This can fail depending on a lot of things, so trigger a console message if it fails.
    //
    // TODO: Figure out why we have two logger inits here.
    let logger = Logger::init(&PathBuf::from("."), cli.verbose, true, release_name!());
    if logger.is_err() && cli.verbose {
        warn!("Logging initialization has failed. No logs will be saved.");
    }

    // Build the Config struct to remember the current configuration when processing stuff.
    let config = Config::new(&cli.game, cli.verbose);

    // Execute the commands.
    let result: Result<()> = match cli.command {
        Commands::AnimPack { commands } => match commands {
            CommandsAnimPack::List { pack_path } => crate::commands::animpack::list(&config, &pack_path),
            CommandsAnimPack::Create { pack_path } => crate::commands::animpack::create(&config, &pack_path),
            CommandsAnimPack::Add { pack_path, file_path, folder_path } => crate::commands::animpack::add(&config, &pack_path, &file_path, &folder_path),
            CommandsAnimPack::Delete { pack_path, file_path, folder_path } => crate::commands::animpack::delete(&config, &pack_path, &file_path, &folder_path),
            CommandsAnimPack::Extract { pack_path, file_path, folder_path } => crate::commands::animpack::extract(&config, &pack_path, &file_path, &folder_path),
        }

        Commands::Dependencies { commands } => match commands {
            CommandsDependencies::Generate { pak_path, game_path, schema_path, assembly_kit_path } => crate::commands::dependencies::generate(&config, &pak_path, &game_path, &schema_path, &assembly_kit_path),
        }

        Commands::Pack { commands } => match commands {
            CommandsPack::List { pack_path } => crate::commands::pack::list(&config, &pack_path),
            CommandsPack::Create { pack_path } => crate::commands::pack::create(&config, &pack_path),
            CommandsPack::Add { pack_path, tsv_to_binary, file_path, folder_path } => crate::commands::pack::add(&config, &tsv_to_binary, &pack_path, &file_path, &folder_path),
            CommandsPack::Delete { pack_path, file_path, folder_path } => crate::commands::pack::delete(&config, &pack_path, &file_path, &folder_path),
            CommandsPack::Extract { pack_path, tables_as_tsv, file_path, folder_path } => crate::commands::pack::extract(&config, &tables_as_tsv, &pack_path, &file_path, &folder_path),
            CommandsPack::SetFileType { pack_path, file_type } => crate::commands::pack::set_pack_type(&config, &pack_path, file_type),
            CommandsPack::Diagnose { game_path, pak_path, schema_path, pack_path } => crate::commands::pack::diagnose(&config, &game_path, &pak_path, &schema_path, &pack_path),
            CommandsPack::Merge { save_pack_path, source_pack_paths } => crate::commands::pack::merge(&config, &save_pack_path, &source_pack_paths),
            CommandsPack::AddDependencyPack { pack_path, dependency_pack } => crate::commands::pack::add_dependency(&config, &pack_path, &dependency_pack),
            CommandsPack::RemoveDependencyPack { pack_path, dependency_pack } => crate::commands::pack::remove_dependency(&config, &pack_path, &dependency_pack),
            CommandsPack::RemoveAllDependencies { pack_path } => crate::commands::pack::remove_all_dependencies(&config, &pack_path),
        }

        Commands::Schemas { commands } => match commands {
            CommandsSchemas::Update { schema_path } => crate::commands::schema::update(&config, &schema_path),
            CommandsSchemas::ToJson { schemas_path } => crate::commands::schema::to_json(&config, &schemas_path),
        }

        Commands::PortraitSettings { commands } => match commands {
            CommandsPortraitSettings::FromJson { json_path, bin_path } => crate::commands::portrait_settings::from_json(&config, &json_path, &bin_path),
            CommandsPortraitSettings::ToJson { bin_path, json_path } => crate::commands::portrait_settings::to_json(&config, &bin_path, &json_path),
        }
    };

    // Output the result of the commands.
    match result {
        Ok(_) => exit(0),
        Err(error) => {
            error!("{error}");
            exit(1)
        },
    }
}
