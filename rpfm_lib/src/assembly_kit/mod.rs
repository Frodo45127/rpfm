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
Module with all the code to interact with the Assembly Kit's Files.

This module contains all the code related with the integrations with the Assembly Kit.
To differentiate between the different types of Assembly Kit, there are multiple versions:
- `0`: Empire and Napoleon.
- `1`: Shogun 2.
- `2`: Anything since Rome 2.
!*/

use rayon::prelude::*;
use serde_xml_rs::from_reader;

use std::borrow::BorrowMut;
use std::fs::{File, read_dir};
use std::io::BufReader;
use std::path::{Path, PathBuf};

use rpfm_error::{Result, ErrorKind};

use crate::assembly_kit::table_definition::RawDefinition;
use crate::assembly_kit::localisable_fields::RawLocalisableFields;
use crate::{GAME_SELECTED, SCHEMA};
use crate::dependencies::Dependencies;
use crate::packfile::PathType;
use crate::packedfile::table::db::DB;
use crate::schema::*;

pub mod localisable_fields;
pub mod table_data;
pub mod table_definition;

const LOCALISABLE_FILES_FILE_NAME_V2: &str = "TExc_LocalisableFields";

const RAW_DEFINITION_NAME_PREFIX_V2: &str = "TWaD_";
const RAW_DEFINITION_IGNORED_FILES_V2: [&str; 5] = [
    "TWaD_schema_validation",
    "TWaD_relationships",
    "TWaD_validation",
    "TWaD_tables",
    "TWaD_queries",
];

//const RAW_DEFINITION_EXTENSION_V2: &str = ".xml";
//const RAW_DATA_EXTENSION_V2: &str = RAW_DEFINITION_EXTENSION_V2;

const RAW_DEFINITION_EXTENSION_V0: &str = ".xsd";
//const RAW_DATA_EXTENSION_V0: &str = RAW_DATA_EXTENSION_V2;

const BLACKLISTED_TABLES: [&str; 1] = ["translated_texts.xml"];

//---------------------------------------------------------------------------//
// Functions to process the Raw DB Tables from the Assembly Kit.
//---------------------------------------------------------------------------//

/// This function updates the current Schema with the information of the provided Assembly Kit.
///
/// Some notes:
/// - This works only over already decoded tables (no new definitions are created).
/// - This decodes localisable fields as proper localisable fiels, separating them from the rest.
/// - This only updates the current versions of the tables, not older ones.
pub fn update_schema_from_raw_files(ass_kit_path: Option<PathBuf>, dependencies: &Dependencies) -> Result<()> {
    let mut schema_writable = SCHEMA.write().unwrap();
    let schema_referenced: &mut Option<Schema> = schema_writable.borrow_mut();
    if let Some(ref mut schema) = schema_referenced {

        // This has to do a different process depending on the `raw_db_version`.
        let raw_db_version = GAME_SELECTED.read().unwrap().get_raw_db_version();
        match raw_db_version {
            2 | 1 => {

                let mut ass_kit_schemas_path =
                    if raw_db_version == 1 {
                        if let Some(path) = ass_kit_path { path }
                        else { return Err(ErrorKind::SchemaNotFound.into()) }
                    }
                    else if let Ok(path) = GAME_SELECTED.read().unwrap().get_assembly_kit_path() { path }
                    else { return Err(ErrorKind::SchemaNotFound.into()) };

                ass_kit_schemas_path.push("raw_data");
                ass_kit_schemas_path.push("db");

                // This one is notably missing in Warhammer 2, so it's optional.
                let raw_localisable_fields: Option<RawLocalisableFields> =
                    if let Ok(file_path) = get_raw_localisable_fields_path(&ass_kit_schemas_path, raw_db_version) {
                        let file = BufReader::new(File::open(&file_path)?);
                        from_reader(file).ok()
                    } else { None };

                let (raw_definitions, _) = RawDefinition::read_all(&ass_kit_schemas_path, raw_db_version, false, dependencies)?;
                schema.get_ref_mut_versioned_file_db_all().par_iter_mut().for_each(|versioned_file| {
                    if let VersionedFile::DB(table_name, definitions) = versioned_file {
                        let name = &table_name[0..table_name.len() - 7];
                        if let Some(raw_definition) = raw_definitions.iter().filter(|x| x.name.is_some()).find(|x| &(x.name.as_ref().unwrap())[0..x.name.as_ref().unwrap().len() - 4] == name) {
                            if let Ok((ref mut vanilla_tables, ref mut _error_paths)) = dependencies.get_packedfiles_from_game_files(&[PathType::Folder(vec!["db".to_owned(), table_name.to_owned()])]) {
                                if !vanilla_tables.is_empty() {
                                    let vanilla_table = &mut vanilla_tables[0];
                                    if let Ok(vanilla_table_data) = vanilla_table.get_raw_data_and_keep_it() {
                                        if let Ok((version, _, _, _, _)) = DB::read_header(&vanilla_table_data) {
                                            if let Some(ref mut definition) = definitions.iter_mut().find(|x| x.get_version() == version) {
                                                definition.update_from_raw_definition(raw_definition);
                                                if let Some(ref raw_localisable_fields) = raw_localisable_fields {
                                                    definition.update_from_raw_localisable_fields(raw_definition, &raw_localisable_fields.fields)
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                });
                schema.save(GAME_SELECTED.read().unwrap().get_schema_name())
            }
            _ => { Err(ErrorKind::AssemblyKitUnsupportedVersion(raw_db_version).into()) }
        }
    }

    else { Err(ErrorKind::SchemaNotFound.into()) }
}

//---------------------------------------------------------------------------//
// Utility functions to process raw files from the Assembly Kit.
//---------------------------------------------------------------------------//

/// This function returns all the raw Assembly Kit Table Definition files from the provided folder.
///
/// Yoy must provide it the folder with the definitions inside, and the version of the game to process.
pub fn get_raw_definition_paths(current_path: &Path, version: i16) -> Result<Vec<PathBuf>> {

    let mut file_list: Vec<PathBuf> = vec![];
    match read_dir(current_path) {
        Ok(files_in_current_path) => {
            for file in files_in_current_path {
                match file {
                    Ok(file) => {
                        let file_path = file.path();
                        let file_name = file_path.file_stem().unwrap().to_str().unwrap();
                        if version == 1 || version == 2 {
                            if file_path.is_file() &&
                                file_name.starts_with(RAW_DEFINITION_NAME_PREFIX_V2) &&
                                !file_name.starts_with("TWaD_TExc") &&
                                !RAW_DEFINITION_IGNORED_FILES_V2.contains(&file_name) {
                                file_list.push(file_path);
                            }
                        }

                        else if version == 0 &&
                            file_path.is_file() &&
                            file_name.ends_with(RAW_DEFINITION_EXTENSION_V0) {
                            file_list.push(file_path);
                        }
                    }
                    Err(_) => return Err(ErrorKind::IOReadFile(current_path.to_path_buf()).into()),
                }
            }
        }
        Err(_) => return Err(ErrorKind::IOReadFolder(current_path.to_path_buf()).into()),
    }

    // Sort the files alphabetically.
    file_list.sort();
    Ok(file_list)
}


/// This function returns all the raw Assembly Kit Table Data files from the provided folder.
///
/// Yoy must provide it the folder with the tables inside, and the version of the game to process.
pub fn get_raw_data_paths(current_path: &Path, version: i16) -> Result<Vec<PathBuf>> {

    let mut file_list: Vec<PathBuf> = vec![];
    match read_dir(current_path) {
        Ok(files_in_current_path) => {
            for file in files_in_current_path {
                match file {
                    Ok(file) => {
                        let file_path = file.path();
                        let file_name = file_path.file_stem().unwrap().to_str().unwrap();
                        if version == 1 || version == 2 {
                            if file_path.is_file() && !file_name.starts_with(RAW_DEFINITION_NAME_PREFIX_V2) {
                                file_list.push(file_path);
                            }
                        }

                        else if version == 0 &&
                            file_path.is_file() &&
                            !file_name.ends_with(RAW_DEFINITION_EXTENSION_V0) {
                            file_list.push(file_path);
                        }
                    }
                    Err(_) => return Err(ErrorKind::IOReadFile(current_path.to_path_buf()).into()),
                }
            }
        }
        Err(_) => return Err(ErrorKind::IOReadFolder(current_path.to_path_buf()).into()),
    }

    // Sort the files alphabetically.
    file_list.sort();
    Ok(file_list)
}

/// This function returns the path of the raw Assembly Kit `Localisable Fields` table from the provided folder.
///
/// Yoy must provide it the folder with the table inside, and the version of the game to process.
/// NOTE: Version 0 is not yet supported.
pub fn get_raw_localisable_fields_path(current_path: &Path, version: i16) -> Result<PathBuf> {
    match read_dir(current_path) {
        Ok(files_in_current_path) => {
            for file in files_in_current_path {
                match file {
                    Ok(file) => {
                        let file_path = file.path();
                        let file_name = file_path.file_stem().unwrap().to_str().unwrap();
                        if (version == 1 || version == 2) && file_path.is_file() && file_name == LOCALISABLE_FILES_FILE_NAME_V2 {
                            return Ok(file_path)
                        }
                    }
                    Err(_) => return Err(ErrorKind::IOReadFile(current_path.to_path_buf()).into()),
                }
            }
        }
        Err(_) => return Err(ErrorKind::IOReadFolder(current_path.to_path_buf()).into()),
    }

    // If we didn't find the file, return an error.
    Err(ErrorKind::AssemblyKitLocalisableFieldsNotFound.into())
}
