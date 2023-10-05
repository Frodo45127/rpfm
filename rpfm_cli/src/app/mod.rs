//---------------------------------------------------------------------------//
// Copyright (c) 2017-2023 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! This module contains the input and command definitions for the tool.

use anyhow::{anyhow, Result};
use clap::{builder::PossibleValuesParser, Parser, Subcommand};
use csv::ReaderBuilder;

use std::path::PathBuf;

use rpfm_lib::games::pfh_file_type::PFHFileType;
use rpfm_lib::games::supported_games::SupportedGames;

//---------------------------------------------------------------------------//
//                          Struct/Enum Definitions
//---------------------------------------------------------------------------//

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub(crate) struct Cli {

    /// Make output more detailed.
    #[arg(short, long)]
    pub verbose: bool,

    /// Game we are using this tool for.
    #[arg(short, long, value_name = "GAME", value_parser = PossibleValuesParser::new(game_keys()))]
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

    /// Command to perform operations related with the dependencies cache.
    Dependencies {

        #[clap(subcommand)]
        commands: CommandsDependencies,
    },

    /// Command to perform operations over Pack files.
    Pack {

        #[clap(subcommand)]
        commands: CommandsPack,
    },

    /// Command to perform operations over Schemas.
    Schemas {

        #[clap(subcommand)]
        commands: CommandsSchemas,
    },

    /// Command to perform operations over PortraitSettings files.
    PortraitSettings {

        #[clap(subcommand)]
        commands: CommandsPortraitSettings,
    },
}

#[derive(Subcommand)]
pub enum CommandsAnimPack {

    /// List the contents of the provided AnimPack.
    List {

        /// Path of the Pack this operation will use.
        #[arg(short, long, required = true, value_name = "PATH")]
        pack_path: PathBuf,
    },

    /// Creates a new empty AnimPack in the provided path.
    Create {

        /// Path of the AnimPack this operation will use.
        #[arg(short, long, required = true, value_name = "PATH")]
        pack_path: PathBuf,
    },

    /// Adds a file/folder from disk to the AnimPack in the provided path.
    Add {

        /// Path of the AnimPack this operation will use.
        #[arg(short, long, required = true, value_name = "PACK_PATH")]
        pack_path: PathBuf,

        /// File to add, and folder within the AnimPack where to add it to, separated by comma. If no folder to add to is provided, it'll add the file in the root of the AnimPack.
        ///
        /// If the folder ends with /, the file will be added in that folder with its original name.
        /// If it doesn't, the last part of the path will be the new file's name.
        ///
        /// This can be repeated as many times as files you want to add.
        #[arg(short, long, required = false, num_args = 1.., value_parser = add_file_from_csv, value_name = "FILE_PATH,FOLDER_TO_ADD_TO")]
        file_path: Vec<(PathBuf, String)>,

        /// Folder to add, and folder within the AnimPack where to add it to, separated by comma. If no folder to add to is provided, it'll add the folder in the root of the AnimPack.
        ///
        /// This can be repeated as many times as folders you want to add.
        #[arg(short = 'F', long, required = false, num_args = 1.., value_parser = add_folder_from_csv, value_name = "FOLDER_PATH,FOLDER_TO_ADD_TO")]
        folder_path: Vec<(PathBuf, String)>,
    },

    /// Deletes a file/folder from the AnimPack in the provided path.
    Delete {

        /// Path of the AnimPack this operation will use.
        #[arg(short, long, required = true, value_name = "PACK_PATH")]
        pack_path: PathBuf,

        /// Full path of the file to delete.
        ///
        /// This can be repeated as many times as files you want to delete.
        #[arg(short, long, required = false, num_args = 1.., value_name = "FILE_PATH")]
        file_path: Vec<String>,

        /// Full path of the folder to delete.
        ///
        /// This can be repeated as many times as folders you want to delete.
        #[arg(short = 'F', long, required = false, num_args = 1.., value_name = "FOLDER_PATH")]
        folder_path: Vec<String>,
    },

    /// Extracts a file/folder from the AnimPack in the provided path to the specified path on disk, keeping the internal folder structure.
    Extract {

        /// Path of the AnimPack this operation will use.
        #[arg(short, long, required = true, value_name = "PACK_PATH")]
        pack_path: PathBuf,

        /// File to extract, and folder where to extract it to, separated by comma. If no folder to extract to is provided, it'll extract the file to the current folder.
        ///
        /// This can be repeated as many times as files you want to extract.
        #[arg(short, long, required = false, num_args = 1.., value_parser = extract_from_csv, value_name = "FILE_PATH_IN_PACK,FOLDER_TO_EXTRACT_TO")]
        file_path: Vec<(String, PathBuf)>,

        /// Folder to extract, and folder where to extract it to, separated by comma. If no folder to extract to is provided, it'll extract the folder to the current folder. If only '/' is provided as 'folder to extract', it'll extract the entire AnimPack.
        ///
        /// This can be repeated as many times as folders you want to extract.
        #[arg(short = 'F', long, required = false, num_args = 1.., value_parser = extract_from_csv, value_name = "FOLDER_PATH_IN_PACK,FOLDER_TO_EXTRACT_TO")]
        folder_path: Vec<(String, PathBuf)>,
    },
}

#[derive(Subcommand)]
pub enum CommandsDependencies {

    /// Generate the dependencies cache for a specific game.
    Generate {

        /// Path where the dependencies cache will be saved.
        #[arg(short = 'P', long, required = true, value_name = "PAK2_PATH")]
        pak_path: PathBuf,

        /// Path of the game the dependencies cache is for.
        #[arg(short, long, required = true, value_name = "GAME_PATH")]
        game_path: PathBuf,

        /// Path of the assembly kit the dependencies cache is for.
        ///
        /// Optional.
        #[arg(short, long, required = false, value_name = "ASSEMBLY_KIT_PATH")]
        assembly_kit_path: Option<PathBuf>,
    }
}

#[derive(Subcommand)]
pub enum CommandsPack {

    /// List the contents of the provided Pack.
    SetFileType {

        /// Path of the Pack this operation will use.
        #[arg(short, long, required = true, value_name = "PATH")]
        pack_path: PathBuf,

        /// Full path of the file to delete.
        #[arg(short, long, required = true, num_args = 1, value_name = "PACK_TYPE", value_parser = parse_pfh_file_type)]
        file_type: PFHFileType,
    },

    /// List the contents of the provided Pack.
    List {

        /// Path of the Pack this operation will use.
        #[arg(short, long, required = true, value_name = "PATH")]
        pack_path: PathBuf,
    },

    /// Creates a new empty Pack in the provided path.
    Create {

        /// Path of the Pack this operation will use.
        #[arg(short, long, required = true, value_name = "PATH")]
        pack_path: PathBuf,
    },

    /// Adds a file/folder from disk to the Pack in the provided path.
    Add {

        /// Path of the Pack this operation will use.
        #[arg(short, long, required = true, value_name = "PACK_PATH")]
        pack_path: PathBuf,

        /// If enabled, if a tsv file is detected in the files to add, the program will try to import it to binary before adding it to the Pack.
        ///
        /// It requires the path of the Schema you want to use for definition resolving.
        #[arg(short, long, required = false, value_name = "SCHEMA_PATH")]
        tsv_to_binary: Option<PathBuf>,

        /// File to add, and folder within the Pack where to add it to, separated by semicolon. If no folder to add to is provided, it'll add the file in the root of the Pack.
        ///
        /// If the folder ends with /, the file will be added in that folder with its original name.
        /// If it doesn't, the last part of the path will be the new file's name.
        ///
        /// This can be repeated as many times as files you want to add.
        #[arg(short, long, required = false, num_args = 1.., value_parser = add_file_from_csv, value_name = "FILE_PATH;FOLDER_TO_ADD_TO")]
        file_path: Vec<(PathBuf, String)>,

        /// Folder to add, and folder within the Pack where to add it to, separated by semicolon. If no folder to add to is provided, it'll add the folder in the root of the Pack.
        ///
        /// This can be repeated as many times as folders you want to add.
        #[arg(short = 'F', long, required = false, num_args = 1.., value_parser = add_folder_from_csv, value_name = "FOLDER_PATH;FOLDER_TO_ADD_TO")]
        folder_path: Vec<(PathBuf, String)>,
    },

    /// Deletes a file/folder from the Pack in the provided path.
    Delete {

        /// Path of the Pack this operation will use.
        #[arg(short, long, required = true, value_name = "PACK_PATH")]
        pack_path: PathBuf,

        /// Full path of the file to delete.
        ///
        /// This can be repeated as many times as files you want to delete.
        #[arg(short, long, required = false, num_args = 1.., value_name = "FILE_PATH")]
        file_path: Vec<String>,

        /// Full path of the folder to delete.
        ///
        /// This can be repeated as many times as folders you want to delete.
        #[arg(short = 'F', long, required = false, num_args = 1.., value_name = "FOLDER_PATH")]
        folder_path: Vec<String>,
    },

    /// Extracts a file/folder from the Pack in the provided path to the specified path on disk, keeping the internal folder structure.
    Extract {

        /// Path of the Pack this operation will use.
        #[arg(short, long, required = true, value_name = "PACK_PATH")]
        pack_path: PathBuf,

        /// If enabled, if a decoded DB or Loc file is extracted, it'll be extracted as a TSV file.
        ///
        /// It requires the path of the Schema you want to use for definition resolving.
        #[arg(short, long, required = false, value_name = "SCHEMA_PATH")]
        tables_as_tsv: Option<PathBuf>,

        /// File to extract, and folder where to extract it to, separated by semicolon. If no folder to extract to is provided, it'll extract the file to the current folder.
        ///
        /// This can be repeated as many times as files you want to extract.
        #[arg(short, long, required = false, num_args = 1.., value_parser = extract_from_csv, value_name = "FILE_PATH_IN_PACK;FOLDER_TO_EXTRACT_TO")]
        file_path: Vec<(String, PathBuf)>,

        /// Folder to extract, and folder where to extract it to, separated by semicolon. If no folder to extract to is provided, it'll extract the folder to the current folder. If only '/' is provided as 'folder to extract', it'll extract the entire Pack.
        ///
        /// This can be repeated as many times as folders you want to extract.
        #[arg(short = 'F', long, required = false, num_args = 1.., value_parser = extract_from_csv, value_name = "FOLDER_PATH_IN_PACK;FOLDER_TO_EXTRACT_TO")]
        folder_path: Vec<(String, PathBuf)>,
    },

    /// Performs a diagnostics check over the Pack/s in the provided path to the specified path on disk. The results will be returned in json.
    Diagnose {

        /// Path of the game the Pack diagnosed is for.
        #[arg(short, long, required = true, value_name = "GAME_PATH")]
        game_path: PathBuf,

        /// Path of the dependencies cache to be used.
        ///
        /// If you don't have one, generate it with the `dependencies generate` command.
        #[arg(short = 'P', long, required = true, value_name = "PAK2_PATH")]
        pak_path: PathBuf,

        /// Path of the schema for the game the Pack/s is for.
        #[arg(short, long, required = true, value_name = "SCHEMA_PATH")]
        schema_path: PathBuf,

        /// Path of the Pack this operation will use.
        ///
        /// You can specify multiple packs to perform a diagnostics check over all of them.
        #[arg(short, long, required = true, num_args = 1.., value_name = "PACK_PATH")]
        pack_path: Vec<PathBuf>,
    },

    /// Merges all the Packs provided into a single Pack and saves it to the provided save path.
    Merge {

        /// Path where the merged Pack will be saved.
        #[arg(short = 'p', long, required = true, value_name = "SAVE_PACK_PATH")]
        save_pack_path: PathBuf,

        /// Path of the Packs this operation will use.
        ///
        /// Priority for conflicting files is determined by the order of the Packs in the command.
        #[arg(short = 's', long, required = true, num_args = 1.., value_name = "SOURCE_PACK_PATHS")]
        source_pack_paths: Vec<PathBuf>,
    },
}

#[derive(Subcommand)]
pub enum CommandsSchemas {

    /// Update the schemas from the main schema repo into the provided folder.
    Update {

        /// Path where the schemas will be downloaded.
        #[arg(short, long, required = true, value_name = "SCHEMA_PATH")]
        schema_path: PathBuf,
    },

    /// Convert all the schemas in the provided folder from Ron to Json.
    ToJson {

        /// Path where the schemas are located.
        #[arg(short, long, required = true, value_name = "SCHEMAS_PATH")]
        schemas_path: PathBuf,
    }
}

#[derive(Subcommand)]
pub enum CommandsPortraitSettings {

    /// Convert a JSon file to a binary Portrait Settings.
    FromJson {

        /// Path of the Json file.
        #[arg(short, long, required = true, value_name = "JSON_PATH")]
        json_path: PathBuf,

        /// Path of the resulting PortraitSettings file.
        #[arg(short, long, required = true, value_name = "BIN_PATH")]
        bin_path: PathBuf,
    },

    /// Convert a binary Portrait Settings file to JSon.
    ToJson {

        /// Path of the PortraitSettings file.
        #[arg(short, long, required = true, value_name = "BIN_PATH")]
        bin_path: PathBuf,

        /// Path of the resulting Json file.
        #[arg(short, long, required = true, value_name = "JSON_PATH")]
        json_path: PathBuf,
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

    if let Some(Ok(record)) = reader.records().next() {
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

    if let Some(Ok(record)) = reader.records().next() {
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

    if let Some(Ok(record)) = reader.records().next() {
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

/// Function to get the supported game keys.
fn game_keys() -> Vec<&'static str> {
    let supported_games = SupportedGames::default();
    supported_games.game_keys_sorted().to_vec()
}

fn parse_pfh_file_type(src: &str) -> Result<PFHFileType> {
    PFHFileType::try_from(src).map_err(From::from)
}
