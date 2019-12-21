//---------------------------------------------------------------------------//
// Copyright (c) 2017-2019 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! App module for the CLI tool.
//!
//! This contains the helpers for the initialization of the app.

use clap::{Arg, App, SubCommand};

/// Version of the program, to get it more easely if needed.
const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Full name of the program, to get it more easely.
const PROGRAM_NAME: &str = "Rusted PackFile Manager - CLI Version";

/// Author of the program, to get it more easely.
const AUTHOR: &str = env!("CARGO_PKG_AUTHORS");


//---------------------------------------------------------------------------//
//                          App helpers
//---------------------------------------------------------------------------//

/// This function initialize the main app with all its commands. To be used at the start of the program.
pub fn initialize_app<'a, 'b>() -> App<'a, 'b> {

    // Matches: here we build the entire command parsing for the tool, courtesy of Clap.
    // Also, clap autogenerates certaing commands, like help and version, so those are not needed.
    App::new(PROGRAM_NAME)
        .version(VERSION)
        .author(AUTHOR)
        .about("CLI Version of RPFM. Ready to automate the most boring parts of your modding.")

        //---------------------------//
        // Flags
        //---------------------------//

        // `Verbosity flag`.
        .arg(Arg::with_name("v")
            .short("v")
            .long("verbose")
            .multiple(true)
            .help("Sets the level of verbosity"))

        // `Game` flag. This is required for Game-Specific operations, like saving PackFiles for an specific game, or reading tables.
        .arg(Arg::with_name("game")
            .short("g")
            .long("game")
            .value_name("GAME")
            .help("Sets the 'Game' all the commands will be tailored too. This affects what schemas will be use when dealing with DB Tables, the format of the PackFiles... If it's not set, the default game from the settings will be used.")
            .possible_values(&["three_kingdoms", "warhammer_2", "warhammer", "thrones_of_britannia", "attila", "rome_2", "shogun_2", "napoleon", "empire", "arena"])
            .takes_value(true))

        // `PackFile` Path. This is required for some commands.
        .arg(Arg::with_name("packfile")
            .short("p")
            .long("packfile")
            .help("Path of the PackFile to edit.")
            .value_name("PACKFILE PATH")
            .required(false)
            .takes_value(true))

        //---------------------------//
        // Commands
        //---------------------------//

        // `PackFile` Subcommand. Every command that edits PackFiles in any way goes here.
        .subcommand(SubCommand::with_name("packfile")
            .about("Allows PackFile editing.")

            // `Add File` option. Requires you provided a file.
            .arg(Arg::with_name("add-files")
                .short("a")
                .long("add-files")
                .value_name("FILE PATHS")
                .help("Adds one or more files to the PackFile. If one of the files already exists, it'll replace it.")
                .takes_value(true)
                .min_values(2))

            // `Add Folder` option. Requires you to provide the path for the folders folder.
            .arg(Arg::with_name("add-folders")
                .short("A")
                .long("add-folders")
                .value_name("FOLDER PATHS")
                .help("Adds one or more files/folders to the PackFile. If one of the files already exists, it'll replace it.")
                .takes_value(true)
                .min_values(1))

            // `Delete File` option. Requires you to provide the path of the files to delete.
            .arg(Arg::with_name("delete-files")
                .short("d")
                .long("delete-files")
                .value_name("FILE PATHS")
                .help("Deletes one or more files from the PackFile.")
                .takes_value(true)
                .min_values(1))

            // `Delete Folder` option. Requires you to provide the path of the folders to delete.
            .arg(Arg::with_name("delete-folders")
                .short("D")
                .long("delete-folders")
                .value_name("FOLDER PATHS")
                .help("Deletes one or more folders from the PackFile.")
                .takes_value(true)
                .min_values(1))

            // `List` option.
            .arg(Arg::with_name("list")
                .short("l")
                .long("list")
                .help("Lists the contents of the PackFile.")))

        // `Table` Subcommand. Every command that allows you to manipulate DB/Loc Tables in any way goes here.
        .subcommand(SubCommand::with_name("table")
            .about("Allows you to manipulate in multiple ways DB/LOC Tables.")

            // `EImort TSV` option. To import DB/Loc `PackedFiles` from TSV.
            .arg(Arg::with_name("import")
                .short("i")
                .long("import")
                .value_name("TSV FILE - DESTINATION FILE")
                .help("Import a compatible TSV file as a DB/LOC table.")
                .takes_value(true)
                .min_values(1)
                .max_values(2))

            // `Export TSV` option. To export DB/Loc `PackedFiles` to TSV.
            .arg(Arg::with_name("export")
                .short("e")
                .long("export")
                .value_name("DB FILE - DESTINATION FILE")
                .help("Export a DB/LOC Table's data to a TSV file.")
                .takes_value(true)
                .min_values(1)
                .max_values(2)))

        // `Schema` Subcommand. Basically, here goes commands destined to keep schemas up-to-date.
        .subcommand(SubCommand::with_name("schema")
            .about("Allows you to keep your schemas up-to-date.")
            .arg(Arg::with_name("update")
                .short("u")
                .long("update")
                .takes_value(false)))

}
