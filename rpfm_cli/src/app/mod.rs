//---------------------------------------------------------------------------//
// Copyright (c) 2017-2022 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! This module contains the input and command definitions for the tool.

use anyhow::{anyhow, Result};
use clap::{Parser, Subcommand};
use csv::ReaderBuilder;

use std::path::PathBuf;

use rpfm_lib::games::pfh_file_type::PFHFileType;

//---------------------------------------------------------------------------//
//                          Struct/Enum Definitions
//---------------------------------------------------------------------------//

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
pub(crate) struct Cli {

    /// Make output more detailed.
    #[clap(short, long)]
    pub verbose: bool,

    // TODO: move this to a function that gets the games supported from then lib.
    /// Game we are using this tool for.
    #[clap(short, long, value_parser, value_name = "GAME", possible_values = &["warhammer_3", "troy", "three_kingdoms", "warhammer_2", "warhammer", "thrones_of_britannia", "attila", "rome_2", "shogun_2", "napoleon", "empire", "arena"])]
    pub game: String,

    #[clap(subcommand)]
    pub command: Commands,
}


#[derive(Subcommand)]
pub enum Commands {

    /// Command to perform operations over AnimPack files.
    AnimPack {

        #[clap(subcommand)]
        commands: CommandsAnimPack,
    },

    /// Command to perform operations over Pack files.
    Pack {

        #[clap(subcommand)]
        commands: CommandsPack,
    },

    /// Command to perform operations related with the dependencies cache.
    Dependencies {

        #[clap(subcommand)]
        commands: CommandsDependencies,
    },
}

#[derive(Subcommand)]
pub enum CommandsAnimPack {

    /// List the contents of the provided AnimPack.
    List {

        /// Path of the Pack this operation will use.
        #[clap(short, long, action, required = true, value_parser, name = "PATH")]
        pack_path: PathBuf,
    },

    /// Creates a new empty AnimPack in the provided path.
    Create {

        /// Path of the AnimPack this operation will use.
        #[clap(short, long, action, required = true, value_parser, name = "PATH")]
        pack_path: PathBuf,
    },

    /// Adds a file/folder from disk to the AnimPack in the provided path.
    Add {

        /// Path of the AnimPack this operation will use.
        #[clap(short, long, action, required = true, value_parser, name = "PACK_PATH")]
        pack_path: PathBuf,

        /// File to add, and folder within the AnimPack where to add it to, separated by comma. If no folder to add to is provided, it'll add the file in the root of the AnimPack.
        ///
        /// If the folder ends with /, the file will be added in that folder with its original name.
        /// If it doesn't, the last part of the path will be the new file's name.
        ///
        /// This can be repeated as many times as files you want to add.
        #[clap(short, long, action, required = false, multiple = true, value_parser = add_file_from_csv, name = "FILE_PATH,FOLDER_TO_ADD_TO")]
        file_path: Vec<(PathBuf, String)>,

        /// Folder to add, and folder within the AnimPack where to add it to, separated by comma. If no folder to add to is provided, it'll add the folder in the root of the AnimPack.
        ///
        /// This can be repeated as many times as folders you want to add.
        #[clap(short = 'F', long, action, required = false, multiple = true, value_parser = add_folder_from_csv, name = "FOLDER_PATH,FOLDER_TO_ADD_TO")]
        folder_path: Vec<(PathBuf, String)>,
    },

    /// Deletes a file/folder from the AnimPack in the provided path.
    Delete {

        /// Path of the AnimPack this operation will use.
        #[clap(short, long, action, required = true, value_parser, name = "PACK_PATH")]
        pack_path: PathBuf,

        /// Full path of the file to delete.
        ///
        /// This can be repeated as many times as files you want to delete.
        #[clap(short, long, action, required = false, multiple = true, value_parser, name = "FILE_PATH")]
        file_path: Vec<String>,

        /// Full path of the folder to delete.
        ///
        /// This can be repeated as many times as folders you want to delete.
        #[clap(short = 'F', long, action, required = false, multiple = true, value_parser, name = "FOLDER_PATH")]
        folder_path: Vec<String>,
    },

    /// Extracts a file/folder from the AnimPack in the provided path to the specified path on disk, keeping the internal folder structure.
    Extract {

        /// Path of the AnimPack this operation will use.
        #[clap(short, long, action, required = true, value_parser, name = "PACK_PATH")]
        pack_path: PathBuf,

        /// File to extract, and folder where to extract it to, separated by comma. If no folder to extract to is provided, it'll extract the file to the current folder.
        ///
        /// This can be repeated as many times as files you want to extract.
        #[clap(short, long, action, required = false, multiple = true, value_parser = extract_from_csv, name = "FILE_PATH_IN_PACK,FOLDER_TO_EXTRACT_TO")]
        file_path: Vec<(String, PathBuf)>,

        /// Folder to extract, and folder where to extract it to, separated by comma. If no folder to extract to is provided, it'll extract the folder to the current folder. If only '/' is provided as 'folder to extract', it'll extract the entire AnimPack.
        ///
        /// This can be repeated as many times as folders you want to extract.
        #[clap(short = 'F', long, action, required = false, multiple = true, value_parser = extract_from_csv, name = "FOLDER_PATH_IN_PACK,FOLDER_TO_EXTRACT_TO")]
        folder_path: Vec<(String, PathBuf)>,
    },
}

#[derive(Subcommand)]
pub enum CommandsPack {

    /// List the contents of the provided Pack.
    SetFileType {

        /// Path of the Pack this operation will use.
        #[clap(short, long, action, required = true, value_parser, name = "PATH")]
        pack_path: PathBuf,

        /// Full path of the file to delete.
        #[clap(short, long, required = true, multiple = false, value_parser = pfh_file_type_from_str, name = "PACK_TYPE", possible_values = &["boot", "release", "patch", "mod", "movie"])]
        file_type: PFHFileType,
    },

    /// List the contents of the provided Pack.
    List {

        /// Path of the Pack this operation will use.
        #[clap(short, long, action, required = true, value_parser, name = "PATH")]
        pack_path: PathBuf,
    },

    /// Creates a new empty Pack in the provided path.
    Create {

        /// Path of the Pack this operation will use.
        #[clap(short, long, action, required = true, value_parser, name = "PATH")]
        pack_path: PathBuf,
    },

    /// Adds a file/folder from disk to the Pack in the provided path.
    Add {

        /// Path of the Pack this operation will use.
        #[clap(short, long, action, required = true, value_parser, name = "PACK_PATH")]
        pack_path: PathBuf,

        /// If enabled, if a tsv file is detected in the files to add, the program will try to import it to binary before adding it to the Pack.
        ///
        /// It requires the path of the Schema you want to use for definition resolving.
        #[clap(short, long, action, required = false, value_parser, name = "SCHEMA_PATH")]
        tsv_to_binary: PathBuf,

        /// File to add, and folder within the Pack where to add it to, separated by semicolon. If no folder to add to is provided, it'll add the file in the root of the Pack.
        ///
        /// If the folder ends with /, the file will be added in that folder with its original name.
        /// If it doesn't, the last part of the path will be the new file's name.
        ///
        /// This can be repeated as many times as files you want to add.
        #[clap(short, long, action, required = false, multiple = true, value_parser = add_file_from_csv, name = "FILE_PATH,FOLDER_TO_ADD_TO")]
        file_path: Vec<(PathBuf, String)>,

        /// Folder to add, and folder within the Pack where to add it to, separated by semicolon. If no folder to add to is provided, it'll add the folder in the root of the Pack.
        ///
        /// This can be repeated as many times as folders you want to add.
        #[clap(short = 'F', long, action, required = false, multiple = true, value_parser = add_folder_from_csv, name = "FOLDER_PATH,FOLDER_TO_ADD_TO")]
        folder_path: Vec<(PathBuf, String)>,
    },

    /// Deletes a file/folder from the Pack in the provided path.
    Delete {

        /// Path of the Pack this operation will use.
        #[clap(short, long, action, required = true, value_parser, name = "PACK_PATH")]
        pack_path: PathBuf,

        /// Full path of the file to delete.
        ///
        /// This can be repeated as many times as files you want to delete.
        #[clap(short, long, action, required = false, multiple = true, value_parser, name = "FILE_PATH")]
        file_path: Vec<String>,

        /// Full path of the folder to delete.
        ///
        /// This can be repeated as many times as folders you want to delete.
        #[clap(short = 'F', long, action, required = false, multiple = true, value_parser, name = "FOLDER_PATH")]
        folder_path: Vec<String>,
    },

    /// Extracts a file/folder from the Pack in the provided path to the specified path on disk, keeping the internal folder structure.
    Extract {

        /// Path of the Pack this operation will use.
        #[clap(short, long, action, required = true, value_parser, name = "PACK_PATH")]
        pack_path: PathBuf,

        /// If enabled, if a decoded DB or Loc file is extracted, it'll be extracted as a TSV file.
        ///
        /// It requires the path of the Schema you want to use for definition resolving.
        #[clap(short, long, action, required = false, value_parser, name = "SCHEMA_PATH")]
        tables_as_tsv: PathBuf,

        /// File to extract, and folder where to extract it to, separated by semicolon. If no folder to extract to is provided, it'll extract the file to the current folder.
        ///
        /// This can be repeated as many times as files you want to extract.
        #[clap(short, long, action, required = false, multiple = true, value_parser = extract_from_csv, name = "FILE_PATH_IN_PACK,FOLDER_TO_EXTRACT_TO")]
        file_path: Vec<(String, PathBuf)>,

        /// Folder to extract, and folder where to extract it to, separated by semicolon. If no folder to extract to is provided, it'll extract the folder to the current folder. If only '/' is provided as 'folder to extract', it'll extract the entire Pack.
        ///
        /// This can be repeated as many times as folders you want to extract.
        #[clap(short = 'F', long, action, required = false, multiple = true, value_parser = extract_from_csv, name = "FOLDER_PATH_IN_PACK,FOLDER_TO_EXTRACT_TO")]
        folder_path: Vec<(String, PathBuf)>,
    },

    /// Performs a diagnostics check over the Pack/s in the provided path to the specified path on disk. The results will be returned in json.
    Diagnose {

        /// Path of the game the Pack diagnosed is for.
        #[clap(short, long, action, required = true, value_parser, name = "GAME_PATH")]
        game_path: PathBuf,

        /// Path of the dependencies cache to be used.
        ///
        /// If you don't have one, generate it with the `dependencies generate` command.
        #[clap(short = 'P', long, action, required = true, value_parser, name = "PAK2_PATH")]
        pak_path: PathBuf,

        /// Path of the schema for the game the Pack/s is for.
        #[clap(short, long, action, required = true, value_parser, name = "SCHEMA_PATH")]
        schema_path: PathBuf,

        /// Path of the Pack this operation will use.
        ///
        /// You can specify multiple packs to perform a diagnostics check over all of them.
        #[clap(short, long, action, required = true, multiple = true, value_parser, name = "PACK_PATH")]
        pack_path: Vec<PathBuf>,
    }
}


#[derive(Subcommand)]
pub enum CommandsDependencies {

    /// Generate the dependencies cache for a specific game.
    Generate {

        /// Path where the dependencies cache will be saved.
        #[clap(short = 'P', long, action, required = true, value_parser, name = "PAK2_PATH")]
        pak_path: PathBuf,

        /// Path of the game the dependencies cache is for.
        #[clap(short, long, action, required = true, value_parser, name = "GAME_PATH")]
        game_path: PathBuf,

        /// Path of the assembly kit the dependencies cache is for.
        ///
        /// Optional.
        #[clap(short, long, action, required = false, value_parser, name = "ASSEMBLY_KIT_PATH")]
        assembly_kit_path: Option<PathBuf>,
    }
}

//---------------------------------------------------------------------------//
//                                Validators
//---------------------------------------------------------------------------//

/// Add file to Pack validation function.
fn add_file_from_csv(src: &str) -> Result<(PathBuf, String)> {
    let mut reader = ReaderBuilder::new()
        .delimiter(b';')
        .quoting(true)
        .has_headers(false)
        .flexible(true)
        .from_reader(src.as_bytes());

    for record in reader.records() {
        let record = record?;

        if record.len() == 2 {
            let source = PathBuf::from(&record[0]);
            if !source.is_file() {
                return Err(anyhow!("Path {} doesn't belong to a valid file.", &record[0]));
            }

            let dest = if record[1].ends_with('/') {
                record[1].to_owned() + &source.file_name().unwrap().to_string_lossy()
            } else {
                record[1].to_owned()
            };

            return Ok((source, dest))
        } else if record.len() == 1 {
            let source = PathBuf::from(&record[0]);
            if !source.is_file() {
                return Err(anyhow!("Path {} doesn't belong to a valid file.", &record[0]));
            }

            let dest = String::new();
            return Ok((source, dest))
        } else {
            return Err(anyhow!("Incorrect CSV input."));
        }
    }

    Ok((PathBuf::new(), String::new()))
}

/// Add folder to Pack validation function.
fn add_folder_from_csv(src: &str) -> Result<(PathBuf, String)> {
    let mut reader = ReaderBuilder::new()
        .delimiter(b';')
        .quoting(true)
        .has_headers(false)
        .flexible(true)
        .from_reader(src.as_bytes());

    for record in reader.records() {
        let record = record?;

        if record.len() == 2 {
            let source = PathBuf::from(&record[0]);
            if !source.is_dir() {
                return Err(anyhow!("Path {} doesn't belong to a valid folder.", &record[0]));
            }

            let dest = record[1].to_owned();
            return Ok((source, dest))
        } else if record.len() == 1 {
            let source = PathBuf::from(&record[0]);
            if !source.is_dir() {
                return Err(anyhow!("Path {} doesn't belong to a valid folder.", &record[0]));
            }

            let dest = String::new();
            return Ok((source, dest))
        } else {
            return Err(anyhow!("Incorrect CSV input."));
        }
    }

    Ok((PathBuf::new(), String::new()))
}

/// Extract file/folder from Pack validation function.
fn extract_from_csv(src: &str) -> Result<(String, PathBuf)> {
    let mut reader = ReaderBuilder::new()
        .delimiter(b';')
        .quoting(true)
        .has_headers(false)
        .flexible(true)
        .from_reader(src.as_bytes());

    for record in reader.records() {
        let record = record?;

        if record.len() == 2 {
            let source = record[0].to_owned();
            let dest = PathBuf::from(&record[1]);

            return Ok((source, dest))
        } else if record.len() == 1 {
            let source = record[0].to_owned();
            let dest = PathBuf::new();

            return Ok((source, dest))
        } else {
            return Err(anyhow!("Incorrect CSV input."));
        }
    }

    Ok((String::new(), PathBuf::new()))
}

/// PFHFileType from &str validator.
fn pfh_file_type_from_str(src: &str) -> Result<PFHFileType> {
    PFHFileType::try_from(src).map_err(From::from)
}

//---------------------------------------------------------------------------//
//                          App helpers
//---------------------------------------------------------------------------//
/*
/// This function initialize the main app with all its commands. To be used at the start of the program.
pub fn initialize_app<'a>() -> Command<'a> {

    // Matches: here we build the entire command parsing for the tool, courtesy of Clap.
    // Also, clap autogenerates certain commands, like help and version, so those are not needed.
    Command::new(PROGRAM_NAME)
        .version(VERSION)
        .author(AUTHOR)
        .about("CLI Version of RPFM. Ready to automate the most boring parts of your modding.")

        //---------------------------//
        // Flags
        //---------------------------//

        // `Verbosity flag`.
        .arg(Arg::new("v")
            .short('v')
            .long("verbose")
            .multiple_occurrences(true)
            .help("Sets the level of verbosity"))

        // `Game` flag. This is required for Game-Specific operations, like saving PackFiles for an specific game, or reading tables.
        .arg(Arg::new("game")
            .short('g')
            .long("game")
            .value_name("GAME")
            .help("Sets the 'Game' all the commands will be tailored to. This affects what schemas will be used when dealing with DB Tables, the format of the PackFiles… If it's not set, the default game from the settings will be used.")
            .possible_values(&["warhammer_3", "troy", "three_kingdoms", "warhammer_2", "warhammer", "thrones_of_britannia", "attila", "rome_2", "shogun_2", "napoleon", "empire", "arena"])
            .takes_value(true))

        // `AssKit DB Path` flag. This is required for certain operations requiring the dependencies cache.
        .arg(Arg::new("asskit_db_path")
            .short('a')
            .long("assdb")
            .value_name("ASSKIT DB PATH")
            .help("Sets the 'Asskit Raw DB Path'. Used for certain operations depending on the dependencies cache.")
            .required(false)
            .takes_value(true))

        // `PackFile` Path. This is required for some commands.
        .arg(Arg::new("packfile")
            .short('p')
            .long("packfile")
            .help("Path of the PackFile to edit.")
            .value_name("PACKFILE PATH")
            .required(false)
            .takes_value(true))

        //---------------------------//
        // Commands
        //---------------------------//

        // `Diagnostic` Subcommand. To check for errors between PackFiles.
        .subcommand(Command::new("diagnostic")
            .about("Allows you to perform diagnostic-related operations over specific sets of PackFiles.")
            .arg(Arg::new("check")
                .short('c')
                .long("check")
                .value_name("PACKFILES TO CHECK, IN LOAD ORDER")
                .help("Performs a diagnostics check over the PackFiles provided.")
                .takes_value(true)
                .min_values(1)))

        // `PackFile` Subcommand. Every command that edits PackFiles in any way goes here.
        .subcommand(Command::new("packfile")
            .about("Allows PackFile editing.")

            // `Add Files` option. Requires you provided the destination folder in the PackFile and at least one file.
            .arg(Arg::new("add-files")
                .short('a')
                .long("add-files")
                .value_name("DESTINATION FOLDER IN THE PACKFILE - FILE PATHS")
                .help("Adds one or more files to the PackFile. If one of the files already exists, it'll replace it.")
                .takes_value(true)
                .min_values(2))

            // `Add Folders` option. Requires you provided the destination folder in the PackFile and at least one folder.
            .arg(Arg::new("add-folders")
                .short('A')
                .long("add-folders")
                .value_name("DESTINATION FOLDER IN THE PACKFILE - FOLDER PATHS")
                .help("Adds one or more files/folders to the PackFile. If one of the files already exists, it'll replace it.")
                .takes_value(true)
                .min_values(1))

            // `Delete File` option. Requires you to provide the path of the files to delete.
            .arg(Arg::new("delete-files")
                .short('d')
                .long("delete-files")
                .value_name("FILE PATHS")
                .help("Deletes one or more files from the PackFile.")
                .takes_value(true)
                .min_values(1))

            // `Delete Folder` option. Requires you to provide the path of the folders to delete.
            .arg(Arg::new("delete-folders")
                .short('D')
                .long("delete-folders")
                .value_name("FOLDER PATHS")
                .help("Deletes one or more folders from the PackFile.")
                .takes_value(true)
                .min_values(1))
            // `Extract Files` option. Requires you to provide the destination folder and the path of the files to extract.
            .arg(Arg::new("extract-files")
                .short('e')
                .long("extract-files")
                .value_name("DESTINATION FOLDER - FILE PATHS")
                .help("Extracts one or more files from the PackFile.")
                .takes_value(true)
                .min_values(2))

            // `Extract Folders` option. Requires you to provide the destination folder and the path of the folders to delete.
            .arg(Arg::new("extract-folders")
                .short('E')
                .long("extract-folders")
                .value_name("DESTINATION FOLDER - FOLDER PATHS")
                .help("Extracts one or more folders from the PackFile.")
                .takes_value(true)
                .min_values(2))

            // `List` option.
            .arg(Arg::new("list")
                .short('l')
                .long("list")
                .help("Lists the contents of the PackFile."))

            // `New Packfile` option. The destination is the path of the PackFile you provided before.
            .arg(Arg::new("new-packfile")
                .short('n')
                .long("new-packfile")
                .help("Creates a new empty Packfile with the provided path.")))

        // `Table` Subcommand. Every command that allows you to manipulate DB/Loc Tables in any way goes here.
        .subcommand(Command::new("table")
            .about("Allows you to manipulate DB/LOC Tables in multiple ways.")

            // `Import TSV` option. To import DB/Loc `PackedFiles` from TSV.
            .arg(Arg::new("import")
                .short('i')
                .long("import")
                .value_name("TSV FILE - DESTINATION FILE")
                .help("Import a compatible TSV file as a DB/LOC table.")
                .takes_value(true)
                .min_values(1)
                .max_values(2))

            // `Export TSV` option. To export DB/Loc `PackedFiles` to TSV.
            .arg(Arg::new("export")
                .short('e')
                .long("export")
                .value_name("DB FILE - DESTINATION FILE")
                .help("Export a DB/LOC Table's data to a TSV file.")
                .takes_value(true)
                .min_values(1)
                .max_values(2)))

        // `Schema` Subcommand. Basically, here goes commands destined to keep schemas up-to-date.
        .subcommand(Command::new("schema")
            .about("Allows you to perform certain operations with schemas.")
            .arg(Arg::new("update")
                .help("Allows you to keep your schemas up-to-date.")
                .short('u')
                .long("update")
                .takes_value(false))
            .arg(Arg::new("to-json")
                .help("Allows you to convert all schemas from Ron to Json.")
                .short('j')
                .long("json")
                .takes_value(false))
            .arg(Arg::new("to-xml")
                .help("Allows you to convert all schemas from Ron to XML.")
                .short('x')
                .long("xml")
                .takes_value(false)))

}
*/
