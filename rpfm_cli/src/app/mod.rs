//---------------------------------------------------------------------------//
// Copyright (c) 2017-2020 Ismael Gutiérrez González. All rights reserved.
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

/// Version of the program, to get it more easily if needed.
const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Full name of the program, to get it more easily.
const PROGRAM_NAME: &str = "Rusted PackFile Manager - CLI Version";

/// Author of the program, to get it more easily.
const AUTHOR: &str = env!("CARGO_PKG_AUTHORS");


//---------------------------------------------------------------------------//
//                          App helpers
//---------------------------------------------------------------------------//

/// This function initialize the main app with all its commands. To be used at the start of the program.
pub fn initialize_app<'a, 'b>() -> App<'a, 'b> {

    // Matches: here we build the entire command parsing for the tool, courtesy of Clap.
    // Also, clap autogenerates certain commands, like help and version, so those are not needed.
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
            .help("Sets the 'Game' all the commands will be tailored to. This affects what schemas will be used when dealing with DB Tables, the format of the PackFiles… If it's not set, the default game from the settings will be used.")
            .possible_values(&["three_kingdoms", "warhammer_2", "warhammer", "thrones_of_britannia", "attila", "rome_2", "shogun_2", "napoleon", "empire", "arena"])
            .takes_value(true))

        // `AssKit DB Path` flag. This is required for certain operations requiring the dependencies cache.
        .arg(Arg::with_name("asskit_db_path")
            .short("a")
            .long("assdb")
            .value_name("ASSKIT DB PATH")
            .help("Sets the 'Asskit Raw DB Path'. Used for certain operations depending on the dependencies cache.")
            .required(false)
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

        // `Diagnostic` Subcommand. To check for errors between PackFiles.
        .subcommand(SubCommand::with_name("diagnostic")
            .about("Allows you to perform diagnostic-related operations over specific sets of PackFiles.")
            .arg(Arg::with_name("check")
                .short("c")
                .long("check")
                .value_name("PACKFILES TO CHECK, IN LOAD ORDER")
                .help("Performs a diagnostics check over the PackFiles provided.")
                .takes_value(true)
                .min_values(1)))

        // `PackFile` Subcommand. Every command that edits PackFiles in any way goes here.
        .subcommand(SubCommand::with_name("packfile")
            .about("Allows PackFile editing.")

            // `Add Files` option. Requires you provided the destination folder in the PackFile and at least one file.
            .arg(Arg::with_name("add-files")
                .short("a")
                .long("add-files")
                .value_name("DESTINATION FOLDER IN THE PACKFILE - FILE PATHS")
                .help("Adds one or more files to the PackFile. If one of the files already exists, it'll replace it.")
                .takes_value(true)
                .min_values(2))

            // `Add Folders` option. Requires you provided the destination folder in the PackFile and at least one folder.
            .arg(Arg::with_name("add-folders")
                .short("A")
                .long("add-folders")
                .value_name("DESTINATION FOLDER IN THE PACKFILE - FOLDER PATHS")
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
            // `Extract Files` option. Requires you to provide the destination folder and the path of the files to extract.
            .arg(Arg::with_name("extract-files")
                .short("e")
                .long("extract-files")
                .value_name("DESTINATION FOLDER - FILE PATHS")
                .help("Extracts one or more files from the PackFile.")
                .takes_value(true)
                .min_values(2))

            // `Extract Folders` option. Requires you to provide the destination folder and the path of the folders to delete.
            .arg(Arg::with_name("extract-folders")
                .short("E")
                .long("extract-folders")
                .value_name("DESTINATION FOLDER - FOLDER PATHS")
                .help("Extracts one or more folders from the PackFile.")
                .takes_value(true)
                .min_values(2))

            // `List` option.
            .arg(Arg::with_name("list")
                .short("l")
                .long("list")
                .help("Lists the contents of the PackFile."))

            // `New Packfile` option. The destination is the path of the PackFile you provided before.
            .arg(Arg::with_name("new-packfile")
                .short("n")
                .long("new-packfile")
                .help("Creates a new empty Packfile with the provided path.")))

        // `Table` Subcommand. Every command that allows you to manipulate DB/Loc Tables in any way goes here.
        .subcommand(SubCommand::with_name("table")
            .about("Allows you to manipulate DB/LOC Tables in multiple ways.")

            // `Import TSV` option. To import DB/Loc `PackedFiles` from TSV.
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
            .about("Allows you to perform certain operations with schemas.")
            .arg(Arg::with_name("update")
                .help("Allows you to keep your schemas up-to-date.")
                .short("u")
                .long("update")
                .takes_value(false))
            .arg(Arg::with_name("to-json")
                .help("Allows you to convert all schemas from Ron to Json.")
                .short("j")
                .long("json")
                .takes_value(false))
            .arg(Arg::with_name("to-xml")
                .help("Allows you to convert all schemas from Ron to XML.")
                .short("x")
                .long("xml")
                .takes_value(false)))

}
