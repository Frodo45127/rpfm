//---------------------------------------------------------------------------//
// Copyright (c) 2017-2024 Ismael Gutiérrez González. All rights reserved.
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

use std::collections::HashMap;
use std::fs::{File, read_dir};
use std::io::BufReader;
use std::path::{Path, PathBuf};

use crate::error::{Result, RLibError};
use crate::games::GameInfo;
use crate::files::db::DB;
use crate::schema::*;
#[cfg(feature = "integration_log")] use crate::integrations::log::info;

use self::localisable_fields::RawLocalisableFields;
use self::table_data::RawTable;
use self::table_definition::{RawDefinition, RawRelationshipsTable};

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

const RAW_DEFINITION_EXTENSION_V0: &str = "xsd";
//const RAW_DATA_EXTENSION_V0: &str = RAW_DATA_EXTENSION_V2;


/// Theses tables are blacklisted because:
/// - "translated_texts.xml": just translations.
/// - "TWaD_form_descriptions.xml": it's not a table.
const BLACKLISTED_TABLES: [&str; 4] = ["translated_texts.xml", "TWaD_form_descriptions.xml", "GroupFormation.xsd", "TExc_Effects.xsd"];

/// Special table containing what I will never have: relationships.
const EXTRA_RELATIONSHIPS_TABLE_NAME: &str = "TWaD_relationships";

//---------------------------------------------------------------------------//
// Functions to process the Raw DB Tables from the Assembly Kit.
//---------------------------------------------------------------------------//

/// This function updates the current Schema with the information of the provided Assembly Kit.
///
/// Some notes:
/// - This works only over already decoded tables (no new definitions are created).
/// - This decodes localisable fields as proper localisable fiels, separating them from the rest.
/// - This only updates the current versions of the tables, not older ones.
pub fn update_schema_from_raw_files(
    schema: &mut Schema,
    game_info: &GameInfo,
    ass_kit_path: &Path,
    schema_path: &Path,
    tables_to_skip: &[&str],
    tables_to_check: &HashMap<String, Vec<DB>>
) -> Result<Option<HashMap<String, Vec<String>>>> {

    // This has to do a different process depending on the `raw_db_version`.
    let raw_db_version = game_info.raw_db_version();
    let (raw_definitions, raw_localisable_fields, raw_extra_relationships) = match raw_db_version {
        2 | 1 => {

            // This one is notably missing in Warhammer 2, so it's optional.
            let raw_localisable_fields: Option<RawLocalisableFields> =
                if let Ok(file_path) = get_raw_localisable_fields_path(ass_kit_path, *raw_db_version) {
                    let file = BufReader::new(File::open(file_path)?);
                    from_reader(file).ok()
                } else { None };

            // Same, this is optional.
            let raw_extra_relationships: Option<RawRelationshipsTable> =
                if let Ok(file_path) = get_raw_extra_relationships_path(ass_kit_path, *raw_db_version) {
                    let file = BufReader::new(File::open(file_path)?);
                    from_reader(file).ok()
                } else { None };

            (RawDefinition::read_all(ass_kit_path, *raw_db_version, tables_to_skip)?, raw_localisable_fields, raw_extra_relationships)
        }

        // For these ones, we expect the path to point to the folder with each game's table folder.
        0 => (RawDefinition::read_all(ass_kit_path, *raw_db_version, tables_to_skip)?, None, None),
        _ => return Err(RLibError::AssemblyKitUnsupportedVersion(*raw_db_version)),
    };

    let mut unfound_fields = schema.definitions_mut().par_iter_mut().flat_map(|(table_name, definitions)| {
        let name = &table_name[0..table_name.len() - 7];
        let mut unfound_fields = vec![];
        if let Some(raw_definition) = raw_definitions.iter().filter(|x| x.name.is_some()).find(|x| &(x.name.as_ref().unwrap())[0..x.name.as_ref().unwrap().len() - 4] == name) {

            // We need to get the version from the vanilla files to know what definition to update.
            if let Some(vanilla_tables) = tables_to_check.get(table_name) {
                for vanilla_table in vanilla_tables {
                    if let Some(definition) = definitions.iter_mut().find(|x| x.version() == vanilla_table.definition().version()) {
                        definition.update_from_raw_definition(raw_definition, &mut unfound_fields);

                        // Check in the extra relationships for missing relations.
                        if let Some(ref raw_extra_relationships) = raw_extra_relationships {
                            raw_extra_relationships.relationships.iter()
                                .filter(|relation| relation.table_name == name)
                                .for_each(|relation| {
                                    if let Some(field) = definition.fields_mut().iter_mut().find(|x| x.name() == relation.column_name) {
                                        field.set_is_reference(Some((relation.foreign_table_name.to_owned(), relation.foreign_column_name.to_owned())));
                                    }
                                }
                            );
                        }

                        if let Some(ref raw_localisable_fields) = raw_localisable_fields {
                            definition.update_from_raw_localisable_fields(raw_definition, &raw_localisable_fields.fields)
                        }

                        // Not the best way to do it, but it works.
                        definition.patches_mut().clear();

                        // Add unused field info.
                        for raw_field in &raw_definition.fields {
                            if raw_field.highlight_flag.is_some() {
                                if raw_field.highlight_flag.clone().unwrap() == "#c8c8c8" {
                                    let mut hashmap = HashMap::new();
                                    hashmap.insert("unused".to_owned(), "true".to_owned());

                                    definition.patches_mut().insert(raw_field.name.to_string(), hashmap);
                                }
                            }
                        }

                        // Update the patches with description data if found. We only support single-key tables for this.
                        if raw_definition.fields.iter().any(|x| x.name == "description") &&
                            definition.fields().iter().all(|x| x.name() != "description") &&
                            definition.localised_fields().iter().all(|x| x.name() != "description"){
                            let fields_processed = definition.fields_processed();
                            let mut data = vec![];

                            // Calculate the key field. Here we may have problems with keys set by patches, so we do some... guessing.
                            let key_field = fields_processed.iter().find(|x| x.is_key(Some(definition.patches())));
                            let raw_key_field = raw_definition.fields.iter().find(|x| x.primary_key == "1");
                            let key_field = if let Some(raw_key_field) = raw_key_field {
                                Some(raw_key_field)
                            } else if let Some(key_field) = key_field {
                                raw_definition.fields.iter().find(|x| x.name == key_field.name())
                            } else {
                                None
                            };

                            if let Some(raw_key_field) = key_field {
                                if let Some(_) = raw_definition.fields.iter().find(|x| x.name == "description") {
                                    if let Ok(raw_table) = RawTable::read(raw_definition, ass_kit_path, *raw_db_version) {
                                        for row in raw_table.rows {
                                            if let Some(key_field) = row.fields.iter().find(|field| field.field_name == raw_key_field.name) {
                                                if let Some(description_field) = row.fields.iter().find(|field| field.field_name == "description") {
                                                    data.push(format!("{};;;;;{}", key_field.field_data, description_field.field_data));
                                                }
                                            }
                                        }
                                    }
                                }
                            }

                            if !data.is_empty() {
                                let key_field = fields_processed.iter().find(|x| x.is_key(Some(definition.patches()))).unwrap();
                                let mut hashmap = HashMap::new();
                                hashmap.insert("lookup_hardcoded".to_owned(), data.join(":::::"));

                                definition.patches_mut().insert(key_field.name().to_string(), hashmap);
                            }
                        }
                    }
                }
            }
        }

        unfound_fields
    }).collect::<Vec<String>>();

    // Sort and remove the known non-exported ones.
    unfound_fields.sort();
    unfound_fields.retain(|table| !game_info.ak_lost_fields().contains(table));

    #[cfg(feature = "integration_log")] info!("Update from raw: fields still not found :{:#?}", unfound_fields);

    schema.save(schema_path)?;

    let mut unfound_hash: HashMap<String, Vec<String>> = HashMap::new();
    for un in &unfound_fields {
        let split = un.split('/').collect::<Vec<_>>();
        if split.len() == 2 {
            match unfound_hash.get_mut(split[0]) {
                Some(fields) => fields.push(split[1].to_string()),
                None => { unfound_hash.insert(split[0].to_string(), vec![split[1].to_string()]); }
            }
        }
    }

    Ok(Some(unfound_hash))
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
                        if (
                            (version == 1 || version == 2) &&
                            file_path.is_file() &&
                            file_name.starts_with(RAW_DEFINITION_NAME_PREFIX_V2) &&
                            !file_name.starts_with("TWaD_TExc") &&
                            !RAW_DEFINITION_IGNORED_FILES_V2.contains(&file_name)
                        ) || (
                            version == 0 &&
                            file_path.is_file() &&
                            file_path.extension().unwrap() == RAW_DEFINITION_EXTENSION_V0
                        ) {
                            file_list.push(file_path);
                        }
                    }
                    Err(_) => return Err(RLibError::ReadFileFolderError(current_path.to_string_lossy().to_string())),
                }
            }
        }
        Err(_) => return Err(RLibError::ReadFileFolderError(current_path.to_string_lossy().to_string())),
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
                    Err(_) => return Err(RLibError::ReadFileFolderError(current_path.to_string_lossy().to_string())),
                }
            }
        }
        Err(_) => return Err(RLibError::ReadFileFolderError(current_path.to_string_lossy().to_string())),
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
                    Err(_) => return Err(RLibError::ReadFileFolderError(current_path.to_string_lossy().to_string())),
                }
            }
        }
        Err(_) => return Err(RLibError::ReadFileFolderError(current_path.to_string_lossy().to_string())),
    }

    // If we didn't find the file, return an error.
    Err(RLibError::AssemblyKitLocalisableFieldsNotFound)
}

pub fn get_raw_extra_relationships_path(current_path: &Path, version: i16) -> Result<PathBuf> {
    match read_dir(current_path) {
        Ok(files_in_current_path) => {
            for file in files_in_current_path {
                match file {
                    Ok(file) => {
                        let file_path = file.path();
                        let file_name = file_path.file_stem().unwrap().to_str().unwrap();
                        if (version == 1 || version == 2) && file_path.is_file() && file_name == EXTRA_RELATIONSHIPS_TABLE_NAME {
                            return Ok(file_path)
                        }
                    }
                    Err(_) => return Err(RLibError::ReadFileFolderError(current_path.to_string_lossy().to_string())),
                }
            }
        }
        Err(_) => return Err(RLibError::ReadFileFolderError(current_path.to_string_lossy().to_string())),
    }

    // If we didn't find the file, return an error.
    Err(RLibError::AssemblyKitExtraRelationshipsNotFound)
}
