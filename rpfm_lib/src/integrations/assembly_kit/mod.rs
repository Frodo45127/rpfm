//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Assembly Kit integration for importing official table definitions.
//!
//! This module provides functionality to parse and import table structure information from
//! Creative Assembly's official Assembly Kit tools. This allows RPFM's schemas to stay
//! synchronized with the official game data formats.
//!
//! # Assembly Kit Versions
//!
//! Different Total War games use different Assembly Kit formats:
//!
//! - **Version 0**: Empire Total War and Napoleon Total War
//!   - Uses `.xsd` XML schema files
//!   - Simpler structure without relationship metadata
//!
//! - **Version 1**: Shogun 2 Total War
//!   - Uses XML files with `TWaD_` prefix
//!   - Includes localisable field information
//!
//! - **Version 2**: Rome 2 and later (including Warhammer series, Three Kingdoms, Troy, etc.)
//!   - Enhanced XML format with full relationship metadata
//!   - Separate relationship and localisable fields files
//!   - Field-level metadata (descriptions, highlight flags for unused fields)
//!
//! # Main Functionality
//!
//! ## Schema Updates
//!
//! The primary function [`update_schema_from_raw_files()`] processes Assembly Kit files to:
//! - Update field types, keys, and default values
//! - Import foreign key relationships
//! - Detect localisable (translatable) fields
//! - Mark unused fields (via highlight flags)
//! - Extract hardcoded lookup data from description fields
//!
//! ## File Parsing
//!
//! The module can parse several Assembly Kit file types:
//! - **Table definitions** (`TWaD_*.xml` or `*.xsd`): Field structure and types
//! - **Localisable fields** (`TExc_LocalisableFields.xml`): Translation-ready fields
//! - **Relationships** (`TWaD_relationships.xml`): Foreign key relationships
//! - **Table data**: Sample data for generating hardcoded lookups
//!
//! # Submodules
//!
//! - [`table_definition`]: XML parsing for table structure definitions
//! - [`table_data`]: XML parsing for table sample data
//! - [`localisable_fields`]: XML parsing for localisable field lists
//!
//! # Example Usage
//!
//! ```ignore
//! use rpfm_lib::integrations::assembly_kit::update_schema_from_raw_files;
//! use rpfm_lib::schema::Schema;
//! use rpfm_lib::games::supported_games::{SupportedGames, KEY_WARHAMMER_3};
//! use std::path::Path;
//! use std::collections::HashMap;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let mut schema = Schema::load(Path::new("schemas/warhammer_3.ron"), None)?;
//! let supported_games = SupportedGames::default();
//! let game_info = supported_games.game(&KEY_WARHAMMER_3).unwrap();
//! let ass_kit_path = Path::new("C:/Program Files/Steam/steamapps/common/Total War WARHAMMER III/assembly_kit");
//! let schema_path = Path::new("schemas/warhammer_3.ron");
//! let tables_to_check = HashMap::new(); // Load vanilla tables here
//!
//! let unfound_fields = update_schema_from_raw_files(
//!     &mut schema,
//!     game_info,
//!     ass_kit_path,
//!     schema_path,
//!     &[],
//!     &tables_to_check,
//! )?;
//! # Ok(())
//! # }
//! ```

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

/// Filename of the localisable fields XML file in Assembly Kit v2+.
const LOCALISABLE_FILES_FILE_NAME_V2: &str = "TExc_LocalisableFields";

/// File prefix for table definition XMLs in Assembly Kit v2+.
const RAW_DEFINITION_NAME_PREFIX_V2: &str = "TWaD_";

/// Definition files to ignore (metadata files, not actual tables).
const RAW_DEFINITION_IGNORED_FILES_V2: [&str; 5] = [
    "TWaD_schema_validation",
    "TWaD_relationships",
    "TWaD_validation",
    "TWaD_tables",
    "TWaD_queries",
];

/// File extension for table definition files in Assembly Kit v0 (Empire/Napoleon).
const RAW_DEFINITION_EXTENSION_V0: &str = "xsd";

/// Files that should not be processed as tables.
///
/// These are excluded because:
/// - `translated_texts.xml`: Contains only translation data, not table structure
/// - `TWaD_form_descriptions.xml`: UI form description, not a data table
/// - `GroupFormation.xsd`: Special formation data
/// - `TExc_Effects.xsd`: Effects metadata
const BLACKLISTED_TABLES: [&str; 4] = ["translated_texts.xml", "TWaD_form_descriptions.xml", "GroupFormation.xsd", "TExc_Effects.xsd"];

/// Filename for the extra relationships metadata file.
///
/// This file contains foreign key relationships not embedded in the table definitions.
const EXTRA_RELATIONSHIPS_TABLE_NAME: &str = "TWaD_relationships";

//---------------------------------------------------------------------------//
// Functions to process the Raw DB Tables from the Assembly Kit.
//---------------------------------------------------------------------------//

/// Updates an existing schema with metadata from Assembly Kit files.
///
/// This function parses Assembly Kit XML files and updates the provided schema with:
/// - Field types, keys, and constraints
/// - Foreign key relationships
/// - Localisable field information
/// - Unused field markers (via highlight flags)
/// - Hardcoded lookup data extracted from description columns
///
/// # Arguments
///
/// * `schema` - The schema to update (modified in place)
/// * `game_info` - Game-specific information (includes Assembly Kit version)
/// * `ass_kit_path` - Path to the Assembly Kit installation directory
/// * `schema_path` - Path where the updated schema should be saved
/// * `tables_to_skip` - List of table names to ignore during import
/// * `tables_to_check` - Map of table names to vanilla DB files for version detection
///
/// # Returns
///
/// Returns `Some(HashMap)` containing tables and fields that couldn't be matched,
/// or `None` if all fields were successfully imported.
///
/// # Important Notes
///
/// - This function **does not create new table definitions**, it only updates existing ones
/// - Only the current version of each table is updated (not historical versions)
/// - Localisable fields are properly separated into the `localised_fields` list
/// - The schema is automatically saved to disk after updates
/// - Fields listed in the game's `ak_lost_fields` are not reported as unfound
///
/// # Errors
///
/// Returns an error if:
/// - The Assembly Kit version is unsupported
/// - XML files cannot be parsed
/// - The schema cannot be saved
///
/// # Example
///
/// ```ignore
/// # use rpfm_lib::integrations::assembly_kit::update_schema_from_raw_files;
/// # use rpfm_lib::schema::Schema;
/// # use rpfm_lib::games::supported_games::{SupportedGames, KEY_WARHAMMER_3};
/// # use std::path::Path;
/// # use std::collections::HashMap;
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let mut schema = Schema::load(Path::new("schemas/warhammer_3.ron"), None)?;
/// let supported_games = SupportedGames::default();
/// let game_info = supported_games.game(&KEY_WARHAMMER_3).unwrap();
///
/// let unfound = update_schema_from_raw_files(
///     &mut schema,
///     game_info,
///     Path::new("C:/Program Files/Steam/.../assembly_kit"),
///     Path::new("schemas/warhammer_3.ron"),
///     &[], // No tables to skip
///     &HashMap::new(), // Vanilla tables
/// )?;
///
/// if let Some(unfound_fields) = unfound {
///     println!("Could not match {} tables", unfound_fields.len());
/// }
/// # Ok(())
/// # }
/// ```
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
                            if raw_field.highlight_flag.is_some() && raw_field.highlight_flag.clone().unwrap() == "#c8c8c8" {
                                let mut hashmap = HashMap::new();
                                hashmap.insert("unused".to_owned(), "true".to_owned());

                                definition.patches_mut().insert(raw_field.name.to_string(), hashmap);
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
                                if raw_definition.fields.iter().any(|x| x.name == "description") {
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

                        // In older games, names_tables type and gender need hardcoded lookups to bypass an issue with mismatching types between ak and game files.
                        if table_name == "names_tables" {
                            let fields_processed = definition.fields_processed();

                            if let Some(field) = fields_processed.iter().find(|x| x.name() == "type") {
                                if *field.field_type() == FieldType::I32 {
                                    let mut hashmap = HashMap::new();
                                    let data = [
                                        String::from("0;;;;;forename"),
                                        String::from("1;;;;;family_name"),
                                        String::from("2;;;;;clan_name"),
                                        String::from("3;;;;;other")
                                    ];

                                    hashmap.insert("lookup_hardcoded".to_owned(), data.join(":::::"));
                                    definition.patches_mut().insert(field.name().to_string(), hashmap);
                                }
                            }

                            if let Some(field) = fields_processed.iter().find(|x| x.name() == "gender") {
                                if *field.field_type() == FieldType::I32 {
                                    let mut hashmap = HashMap::new();
                                    let data = [
                                        String::from("0;;;;;m"),
                                        String::from("1;;;;;f"),
                                        String::from("2;;;;;b"),
                                    ];

                                    hashmap.insert("lookup_hardcoded".to_owned(), data.join(":::::"));
                                    definition.patches_mut().insert(field.name().to_string(), hashmap);
                                }
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

    #[cfg(feature = "integration_log")] info!("Update from raw: fields still not found :{unfound_fields:#?}");

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

/// Returns paths to all table definition files in an Assembly Kit directory.
///
/// This function scans the provided directory and returns paths to XML files containing
/// table structure definitions, filtering by Assembly Kit version.
///
/// # Arguments
///
/// * `current_path` - Directory containing Assembly Kit definition files
/// * `version` - Assembly Kit version (0, 1, or 2)
///
/// # Returns
///
/// Returns a sorted vector of paths to definition files, or an error if the directory cannot be read.
///
/// # File Selection
///
/// - **Version 0** (Empire/Napoleon): `.xsd` files
/// - **Version 1/2** (Shogun 2+): Files starting with `TWaD_` (excluding ignored metadata files)
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


/// Returns paths to all table data files in an Assembly Kit directory.
///
/// This function scans for XML files containing sample table data (as opposed to definitions).
///
/// # Arguments
///
/// * `current_path` - Directory containing Assembly Kit data files
/// * `version` - Assembly Kit version (0, 1, or 2)
///
/// # Returns
///
/// Returns a sorted vector of paths to data files, or an error if the directory cannot be read.
///
/// # File Selection
///
/// - **Version 0** (Empire/Napoleon): XML files without `.xsd` extension
/// - **Version 1/2** (Shogun 2+): XML files that don't start with `TWaD_`
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

/// Returns the path to the localisable fields metadata file.
///
/// This file (`TExc_LocalisableFields.xml`) contains the list of fields that should be
/// extracted to `.loc` translation files.
///
/// # Arguments
///
/// * `current_path` - Directory containing Assembly Kit files
/// * `version` - Assembly Kit version (1 or 2; version 0 is not supported)
///
/// # Returns
///
/// Returns the path to the localisable fields file, or an error if not found.
///
/// # Note
///
/// This file is optional in some Assembly Kits (notably absent in Warhammer 2).
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

/// Returns the path to the extra relationships metadata file.
///
/// This file (`TWaD_relationships.xml`) contains foreign key relationship information
/// that isn't embedded in the table definition files themselves.
///
/// # Arguments
///
/// * `current_path` - Directory containing Assembly Kit files
/// * `version` - Assembly Kit version (1 or 2; version 0 is not supported)
///
/// # Returns
///
/// Returns the path to the relationships file, or an error if not found.
///
/// # Note
///
/// This file is optional and may not be present in all Assembly Kits.
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
