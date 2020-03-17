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
Module with all the code to interact with Schemas.

This module contains all the code related with the schemas used by this lib to decode many PackedFile types.

The basic structure of an `Schema` is:
```rust
(
    version: 2,
    versioned_files: [
        DB("_kv_battle_ai_ability_usage_variables_tables", [
            (
                version: 0,
                fields: [
                    (
                        name: "key",
                        field_type: StringU8,
                        is_key: true,
                        default_value: None,
                        max_length: 0,
                        is_filename: false,
                        filename_relative_path: None,
                        is_reference: None,
                        lookup: None,
                        description: "",
                        ca_order: -1,
                    ),
                    (
                        name: "value",
                        field_type: Float,
                        is_key: false,
                        default_value: None,
                        max_length: 0,
                        is_filename: false,
                        filename_relative_path: None,
                        is_reference: None,
                        lookup: None,
                        description: "",
                        ca_order: -1,
                    ),
                ],
                localised_fields: [],
            ),
        ]),
    ],
)
```

Inside the schema there are `VersionedFile` variants of different types, with a Vec of `Definition`, one for each version of that PackedFile supported.
!*/

use rayon::prelude::*;
use reqwest::blocking;
use ron::de::{from_str, from_reader};
use ron::ser::{to_string_pretty, PrettyConfig};
use serde_derive::{Serialize, Deserialize};

use std::cmp::Ordering;
use std::fs::{DirBuilder, File};
use std::{fmt, fmt::Display};
use std::io::{BufReader, Read, Write};

use rpfm_error::{ErrorKind, Result};

use crate::assembly_kit::localisable_fields::RawLocalisableField;
use crate::assembly_kit::table_definition::{RawDefinition, RawField};
use crate::DEPENDENCY_DATABASE;
use crate::SUPPORTED_GAMES;
use crate::config::get_config_path;
use crate::packedfile::table::db::DB;
use self::versions::VersionsFile;

// Legacy Schemas, to keep backwards compatibility during updates.
pub(crate) mod v1;
pub(crate) mod v0;
pub mod versions;

/// Name of the schema versions file.
const SCHEMA_VERSIONS_FILE: &str = "versions.ron";

/// Name of the folder containing all the schemas.
const SCHEMA_FOLDER: &str = "schemas";

/// URL of the remote repository's schema folder. Master branch.
const SCHEMA_UPDATE_URL_MASTER: &str = "https://raw.githubusercontent.com/Frodo45127/rpfm/master/schemas/";

/// URL of the remote repository's schema folder. Develop branch.
const SCHEMA_UPDATE_URL_DEVELOP: &str = "https://raw.githubusercontent.com/Frodo45127/rpfm/develop/schemas/";

/// Current structural version of the Schema, for compatibility purpouses.
const CURRENT_STRUCTURAL_VERSION: u16 = 2;

//---------------------------------------------------------------------------//
//                              Enum & Structs
//---------------------------------------------------------------------------//

/// This struct represents a Schema File in memory, ready to be used to decode versioned PackedFiles.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct Schema {

    /// It stores the structural version of the Schema.
    version: u16,

    /// It stores the versioned files inside the Schema.
    versioned_files: Vec<VersionedFile>
}

/// This enum defines all types of versioned files that the schema system supports.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum VersionedFile {

    /// It stores the name of the table, and a `Vec<Definition>` with the definitions for each version of that table decoded.
    DB(String, Vec<Definition>),

    /// It stores a `Vec<Definition>` to decode the dependencies of a PackFile.
    DepManager(Vec<Definition>),

    /// It stores a `Vec<Definition>` with the definitions for each version of Loc files decoded (currently, only version `1`).
    Loc(Vec<Definition>),
}

/// This struct contains all the data needed to decode a specific version of a versioned PackedFile.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct Definition {

    /// The version of the PackedFile the definition is for. These versions are:
    /// - `-1`: for fake `Definition`, used for dependency resolving stuff.
    /// - `0`: for unversioned PackedFiles.
    /// - `1+`: for versioned PackedFiles.
    pub version: i32,

    /// This is a collection of all `Field`s the PackedFile uses, in the order it uses them.
    pub fields: Vec<Field>,

    /// This is a list of all the fields from this definition that are moved to a Loc PackedFile on exporting.
    pub localised_fields: Vec<Field>,
}

/// This struct holds all the relevant data do properly decode a field from a versioned PackedFile.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct Field {

    /// Name of the field. Should contain no spaces, using `_` instead.
    pub name: String,

    /// Type of the field.
    pub field_type: FieldType,

    /// `True` if the field is a `Key` field of a table. `False` otherwise.
    pub is_key: bool,

    /// The default value of the field.
    pub default_value: Option<String>,

    /// The max allowed lenght for the data in the field.
    pub max_length: i32,

    /// If the field's data corresponds to a filename.
    pub is_filename: bool,

    /// Path where the file in the data of the field can be, if it's restricted to one path.
    pub filename_relative_path: Option<String>,

    /// `Some(referenced_table, referenced_column)` if the field is referencing another table/column. `None` otherwise.
    pub is_reference: Option<(String, String)>,

    /// `Some(referenced_columns)` if the field is using another column/s from the referenced table for lookup values.
    pub lookup: Option<Vec<String>>,

    /// Aclarative description of what the field is for.
    pub description: String,

    /// Visual position in CA's Table. `-1` means we don't know its position.
    pub ca_order: i16,
}

/// This enum defines every type of field the lib can encode/decode.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum FieldType {
    Boolean,
    Float,
    Integer,
    LongInteger,
    StringU8,
    StringU16,
    OptionalStringU8,
    OptionalStringU16,
    Sequence(Definition)
}

//---------------------------------------------------------------------------//
//                       Enum & Structs Implementations
//---------------------------------------------------------------------------//

/// Implementation of `Schema`.
impl Schema {

    /// This function adds a new `VersionedFile` to the schema. This checks if the provided `VersionedFile`
    /// already exists, and replace it if neccesary.
    pub fn add_versioned_file(&mut self, versioned_file: &VersionedFile) {
        match self.versioned_files.par_iter().position_any(|x| x.conflict(versioned_file)) {
            Some(position) => { self.versioned_files.splice(position..=position, [versioned_file.clone()].iter().cloned()); },
            None => self.versioned_files.push(versioned_file.clone()),
        }
    }

    /// This function returns the structural version of the provided Schema.
    pub fn get_version(&self) -> u16 {
        self.version
    }

    /// This function returns a copy of a specific `VersionedFile` of DB Type from the provided `Schema`.
    pub fn get_versioned_file_db(&self, table_name: &str) -> Result<VersionedFile> {
        self.versioned_files.par_iter().filter(|x| x.is_db())
            .cloned()
            .find_any(|x| if let VersionedFile::DB(name,_) = x { name == table_name } else { false }
        ).ok_or_else(|| From::from(ErrorKind::SchemaVersionedFileNotFound))
    }

    /// This function returns a reference to a specific `VersionedFile` of DB Type from the provided `Schema`.
    pub fn get_ref_versioned_file_db(&self, table_name: &str) -> Result<&VersionedFile> {
        self.versioned_files.par_iter().filter(|x| x.is_db())
            .find_any(|x| if let VersionedFile::DB(name,_) = x { name == table_name } else { false }
        ).ok_or_else(|| From::from(ErrorKind::SchemaVersionedFileNotFound))
    }

    /// This function returns a mutable reference to a specific `VersionedFile` of DB Type from the provided `Schema`.
    pub fn get_ref_mut_versioned_file_db(&mut self, table_name: &str) -> Result<&mut VersionedFile> {
        self.versioned_files.par_iter_mut().filter(|x| x.is_db())
            .find_any(|x| if let VersionedFile::DB(name,_) = x { name == table_name } else { false }
        ).ok_or_else(|| From::from(ErrorKind::SchemaVersionedFileNotFound))
    }

    /// This function returns a copy of a specific `VersionedFile` of Dependency Manager Type from the provided `Schema`.
    ///
    /// By default, we assume there is only one Dependency Manager `VersionedFile` in the `Schema`, so we return that one if we find it.
    pub fn get_versioned_file_dep_manager(&self) -> Result<VersionedFile> {
        self.versioned_files.par_iter().cloned().find_any(|x| x.is_dep_manager()).ok_or_else(|| From::from(ErrorKind::SchemaVersionedFileNotFound))
    }

    /// This function returns a reference to a specific `VersionedFile` of Dependency Manager Type from the provided `Schema`.
    ///
    /// By default, we assume there is only one Dependency Manager `VersionedFile` in the `Schema`, so we return that one if we find it.
    pub fn get_ref_versioned_file_dep_manager(&self) -> Result<&VersionedFile> {
        self.versioned_files.par_iter().find_any(|x| x.is_dep_manager()).ok_or_else(|| From::from(ErrorKind::SchemaVersionedFileNotFound))
    }

    /// This function returns a mutable reference to a specific `VersionedFile` of Dependency Manager Type from the provided `Schema`.
    ///
    /// By default, we assume there is only one Dependency Manager `VersionedFile` in the `Schema`, so we return that one if we find it.
    pub fn get_ref_mut_versioned_file_dep_manager(&mut self) -> Result<&mut VersionedFile> {
        self.versioned_files.par_iter_mut().find_any(|x| x.is_dep_manager()).ok_or_else(|| From::from(ErrorKind::SchemaVersionedFileNotFound))
    }

    /// This function returns a copy of a specific `VersionedFile` of Loc Type from the provided `Schema`.
    ///
    /// By default, we assume there is only one Loc `VersionedFile` in the `Schema`, so we return that one if we find it.
    pub fn get_versioned_file_loc(&self) -> Result<VersionedFile> {
        self.versioned_files.par_iter().find_any(|x| x.is_loc()).cloned().ok_or_else(|| From::from(ErrorKind::SchemaVersionedFileNotFound))
    }

    /// This function returns a reference to a specific `VersionedFile` of Loc Type from the provided `Schema`.
    ///
    /// By default, we assume there is only one Loc `VersionedFile` in the `Schema`, so we return that one if we find it.
    pub fn get_ref_versioned_file_loc(&self) -> Result<&VersionedFile> {
        self.versioned_files.par_iter().find_any(|x| x.is_loc()).ok_or_else(|| From::from(ErrorKind::SchemaVersionedFileNotFound))
    }

    /// This function returns a mutable reference to a specific `VersionedFile` of Loc Type from the provided `Schema`.
    ///
    /// By default, we assume there is only one Loc `VersionedFile` in the `Schema`, so we return that one if we find it.
    pub fn get_ref_mut_versioned_file_loc(&mut self) -> Result<&mut VersionedFile> {
        self.versioned_files.par_iter_mut().find_any(|x| x.is_loc()).ok_or_else(|| From::from(ErrorKind::SchemaVersionedFileNotFound))
    }

    /// This function returns a copy of all the `VersionedFile` in the provided `Schema`.
    pub fn get_versioned_file_all(&self) -> Vec<VersionedFile> {
        self.versioned_files.to_vec()
    }

    /// This function returns a reference to all the `VersionedFile` in the provided `Schema`.
    pub fn get_ref_versioned_file_all(&self) -> Vec<&VersionedFile> {
        self.versioned_files.par_iter().collect()
    }

    /// This function returns a mutable reference to all the `VersionedFile` in the provided `Schema`.
    pub fn get_ref_mut_versioned_file_all(&mut self) -> Vec<&mut VersionedFile> {
        self.versioned_files.par_iter_mut().collect()
    }

    /// This function returns a copy of all the `VersionedFile` in the provided `Schema` of type `DB`.
    pub fn get_versioned_file_db_all(&self) -> Vec<VersionedFile> {
        self.versioned_files.par_iter().filter(|x| x.is_db()).cloned().collect()
    }

    /// This function returns a reference to all the `VersionedFile` in the provided `Schema` of type `DB`.
    pub fn get_ref_versioned_file_db_all(&self) -> Vec<&VersionedFile> {
        self.versioned_files.par_iter().filter(|x| x.is_db()).collect()
    }

    /// This function returns a mutable reference to all the `VersionedFile` in the provided `Schema` of type `DB`.
    pub fn get_ref_mut_versioned_file_db_all(&mut self) -> Vec<&mut VersionedFile> {
        self.versioned_files.par_iter_mut().filter(|x| x.is_db()).collect()
    }

    /// This function returns the last compatible definition of a DB Table.
    ///
    /// As we may have versions from other games, we first need to check for the last definition in the dependency database.
    /// If that fails, we try to get it from the schema.
    pub fn get_ref_last_definition_db(&self, table_name: &str) -> Result<&Definition> {

        // Version is... complicated. We don't really want the last one, but the last one compatible with our game.
        // So we have to try to get it first from the Dependency Database first. If that fails, we fall back to the schema.
        if let Some(vanilla_table) = DEPENDENCY_DATABASE.lock().unwrap().iter_mut()
            .filter(|x| x.get_path().len() == 3)
            .find(|x| x.get_path()[0] == "db" && x.get_path()[1] == *table_name) {
            match DB::read_header(&vanilla_table.get_ref_mut_raw().get_data_and_keep_it().unwrap()) {
                Ok(data) => self.get_ref_versioned_file_db(table_name)?.get_version(data.0),
                Err(error) => Err(error),
            }
        }

        // If there was no coincidence in the dependency database... we risk ourselfs getting the last definition we have for
        // that db from the schema.
        else{
            let versioned_file = self.get_ref_versioned_file_db(table_name)?;
            if let VersionedFile::DB(_,definitions) = versioned_file {
                if let Some(definition) = definitions.get(0) {
                    Ok(definition)
                }
                else { Err(ErrorKind::SchemaDefinitionNotFound.into()) }
            } else { Err(ErrorKind::SchemaVersionedFileNotFound.into()) }
        }
    }

    /// This function returns the last compatible definition of a Loc Table.
    pub fn get_ref_last_definition_loc(&self) -> Result<&Definition> {
        let versioned_file = self.get_ref_versioned_file_loc()?;
        if let VersionedFile::Loc(definitions) = versioned_file {
            if let Some(definition) = definitions.get(0) {
                Ok(definition)
            }
            else { Err(ErrorKind::SchemaDefinitionNotFound.into()) }
        } else { Err(ErrorKind::SchemaVersionedFileNotFound.into()) }
    }

    /// This function loads a `Schema` to memory from a file in the `schemas/` folder.
    pub fn load(schema_file: &str) -> Result<Self> {
        let mut file_path = get_config_path()?.join(SCHEMA_FOLDER);
        file_path.push(schema_file);

        let file = BufReader::new(File::open(&file_path)?);
        from_reader(file).map_err(From::from)
    }

    /// This function saves a `Schema` from memory to a file in the `schemas/` folder.
    pub fn save(&mut self, schema_file: &str) -> Result<()> {
        let mut file_path = get_config_path()?.join(SCHEMA_FOLDER);

        // Make sure the path exists to avoid problems with updating schemas.
        DirBuilder::new().recursive(true).create(&file_path)?;

        file_path.push(schema_file);
        let mut file = File::create(&file_path)?;
        let config = PrettyConfig::default();

        self.sort();
        file.write_all(to_string_pretty(&self, config)?.as_bytes())?;
        Ok(())
    }

    /// This function sorts a `Schema` alphabetically, so the schema diffs are more or less clean.
    pub fn sort(&mut self) {
        self.versioned_files.sort_by(|a, b| {
            match a {
                VersionedFile::DB(table_name_a, _) => {
                    match b {
                        VersionedFile::DB(table_name_b, _) => table_name_a.cmp(&table_name_b),
                        _ => Ordering::Less,
                    }
                }
                VersionedFile::DepManager(_) => {
                    match b {
                        VersionedFile::DB(_,_) => Ordering::Greater,
                        VersionedFile::DepManager(_) => Ordering::Equal,
                        VersionedFile::Loc(_) => Ordering::Less,
                    }
                }
                VersionedFile::Loc(_) => {
                    match b {
                        VersionedFile::Loc(_) => Ordering::Equal,
                        _ => Ordering::Greater,
                    }
                }
            }
        });
    }

    /// This function exports all the schema files from the `schemas/` folder to `.json`.
    ///
    /// For compatibility purpouses.
    pub fn export_to_json(&self) -> Result<()> {
        for schema_file in SUPPORTED_GAMES.iter().map(|x| &x.1.schema) {
            let schema = Schema::load(&schema_file)?;

            let mut file_path = get_config_path()?.join(SCHEMA_FOLDER);
            file_path.push(schema_file);
            file_path.set_extension("json");

            let mut file = File::create(&file_path)?;
            file.write_all(serde_json::to_string_pretty(&schema)?.as_bytes())?;
        }
        Ok(())
    }

    /// This function generates a diff between the local schema files and the remote ones and drops it in the config folder.
    ///
    /// If it detects that you're using the git repo (debug), it adds the diff to the proper place in the docs_src folder instead.
    pub fn generate_schema_diff() -> Result<()> {

        // To avoid doing a lot of useless checking, we only check for schemas with different version.
        let local_schema_versions: VersionsFile = from_reader(BufReader::new(File::open(get_config_path()?.join(SCHEMA_FOLDER).join(SCHEMA_VERSIONS_FILE))?))?;
        let current_schema_versions: VersionsFile = from_str(&blocking::get(&format!("{}{}", SCHEMA_UPDATE_URL_MASTER, SCHEMA_VERSIONS_FILE))?.text()?)?;
        let mut schemas_to_update = vec![];

        // If the game's schema is not in the repo (when adding a new game's support) skip it.
        for (game, version_local) in local_schema_versions.get() {
            let version_current = if let Some(version_current) = current_schema_versions.get().get(game) { version_current } else { continue };
            if version_local != version_current { schemas_to_update.push((game.to_owned(), version_local)); }
        }

        for (game_name, game) in SUPPORTED_GAMES.iter() {

            // Skip all the games with an unchanged version.
            let schema_name = &game.schema;
            let mut schema_version = 0;
            let mut skip_it = true;
            for (schema_to_update, schema_version_to_update) in &mut schemas_to_update {
                if schema_to_update == schema_name {
                    skip_it = false;
                    schema_version = **schema_version_to_update;
                    break;
                }
            }
            if skip_it { continue; }

            // For this, first we get both schemas. Then, compare them table by table looking for differences.
            // Uncomment and tweak the commented schema_current to test against a local schema.
            let schema_local = Schema::load(schema_name).unwrap();
            //let schema_current = Schema::load("schema_att.json").unwrap();
            let schema_current: Schema = blocking::get(&format!("{}/{}", SCHEMA_UPDATE_URL_MASTER, schema_name))?.json()?;

            // Lists to store the different types of differences.
            let mut diff = String::new();
            let mut new_tables = vec![];
            let mut new_versions: Vec<String> = vec![];
            let mut new_corrections: Vec<String> = vec![];

            // For each table, we need to check EVERY possible difference.
            for table_local in schema_local.versioned_files.iter().filter(|x| x.is_db()) {
                if let VersionedFile::DB(name_local, versions_local) = table_local {
                    match schema_current.get_ref_versioned_file_db(name_local) {

                        // If we find it, we have to check if it has changes. If it has them, then we analize them.
                        Ok(table_current) => {
                            if let VersionedFile::DB(_, versions_current) = table_current {
                                if table_local != table_current {
                                    for version_local in versions_local {
                                        match versions_current.iter().find(|x| x.version == version_local.version) {

                                            // If the version has been found, it's a correction for a current version. So we check every
                                            // field for references.
                                            Some(version_current) => version_local.get_pretty_diff(&version_current, &name_local, &mut new_corrections),

                                            // If the version hasn't been found, is a new version. We have to compare it with
                                            // the old one and get his changes.
                                            None => {

                                                // If we have more versions, get the highest one before the one we have. Tables are automatically
                                                // sorted on save, so we can just get the first one of the current list.
                                                if versions_local.len() > 1 {
                                                    let old_version = &versions_current[0];
                                                    version_local.get_pretty_diff(&old_version, &name_local, &mut new_versions);
                                                }
                                            },
                                        }
                                    }
                                }
                            }
                        }

                        // If the table hasn't been found, it's a new table we decoded.
                        Err(_) => new_tables.push(name_local.to_owned()),
                    }
                }
            }

            // Here we put together all the differences.
            for (index, table) in new_tables.iter().enumerate() {
                if index == 0 {
                    diff.push_str("- **New tables decoded**:\n");
                }
                diff.push_str(&format!("  - *{}*.", table));
                diff.push_str("\n");

                if index == new_tables.len() - 1 {
                    diff.push_str("\n");
                }
            }

            for (index, version) in new_versions.iter().enumerate() {
                if index == 0 {
                    diff.push_str("- **Updated Tables**:\n");
                }
                diff.push_str(version);
                diff.push_str("\n");

                if index == new_versions.len() - 1 {
                    diff.push_str("\n");
                }
            }

            for (index, correction) in new_corrections.iter().enumerate() {
                if index == 0 {
                    diff.push_str("- **Fixed Tables**:\n");
                }
                diff.push_str(correction);
                diff.push_str("\n");

                if index == new_corrections.len() - 1 {
                    diff.push_str("\n");
                }
            }

            // If it's not empty, save it. Otherwise, we just ignore it.
            if !diff.is_empty() {

                // If we are in debug mode, save it to his proper file in the docs.
                if cfg!(debug_assertions) {
                    let mut docs_path = std::env::current_dir().unwrap().to_path_buf();
                    docs_path.push("docs_src");
                    docs_path.push("changelogs_tables");
                    docs_path.push(game_name);
                    docs_path.push(&format!("{:03}.md", schema_version));

                    let mut docs_changelog_path = docs_path.to_path_buf();
                    docs_changelog_path.pop();
                    docs_changelog_path.push("changelog.md");

                    // Fix the text so it has the MarkDown title before writing it.
                    diff.insert_str(0, &format!("# {:03}\n\nIt contains the following changes:\n\n", schema_version));
                    let mut file = File::create(docs_path)?;
                    file.write_all(diff.as_bytes())?;

                    // Now, we have to add the file with includes to his respective changelog.
                    let mut base_file = String::new();
                    BufReader::new(File::open(&docs_changelog_path)?).read_to_string(&mut base_file)?;
                    let include_index_line = base_file.find("-----------------------------------").unwrap();
                    let include_data_line = base_file.rfind("-----------------------------------").unwrap();
                    base_file.insert_str(include_data_line + 35, &format!("\n{{{{ #include {:03}.md }}}}", schema_version));
                    base_file.insert_str(include_index_line + 35, &format!("\n- [{:03}](#{:03})", schema_version, schema_version));
                    let mut file = File::create(docs_changelog_path)?;
                    file.write_all(base_file.as_bytes())?;
                }

                // Otherwise, save it to a file in RPFM's folder.
                else {
                    let mut changes_path = get_config_path()?.to_path_buf();
                    changes_path.push(&format!("changelog_{}.txt", schema_name));
                    let mut file = File::create(changes_path)?;
                    file.write_all(diff.as_bytes())?;
                }

            }
        }

        // If everything worked, return success.
        Ok(())
    }

    /// This function allow us to update all Schemas from any legacy version into the current one.
    ///
    /// NOTE FOR DEV: If you make a new Schema Version, add its update function here.
    pub fn update() {
        v0::SchemaV0::update();
        v1::SchemaV1::update();
    }
}

/// Implementation of `VersionedFile`.
impl VersionedFile {

    /// This function returns true if the provided `VersionedFile` is a DB Definition. Otherwise, it returns false.
    pub fn is_db(&self) -> bool {
        match *self {
            VersionedFile::DB(_,_) => true,
            _ => false,
        }
    }

    /// This function returns true if the provided `VersionedFile` is a Dependency Manager Definition. Otherwise, it returns false.
    pub fn is_dep_manager(&self) -> bool {
        match *self {
            VersionedFile::DepManager(_) => true,
            _ => false,
        }
    }

    /// This function returns true if the provided `VersionedFile` is a Loc Definition. Otherwise, it returns false.
    pub fn is_loc(&self) -> bool {
        match *self {
            VersionedFile::Loc(_) => true,
            _ => false,
        }
    }

    /// This function returns true if both `VersionFile` are conflicting (they're the same, but their definitions may be different).
    pub fn conflict(&self, secondary: &VersionedFile) -> bool {
        match &self {
            VersionedFile::DB(table_name, _) => match &secondary {
                VersionedFile::DB(secondary_table_name, _) => table_name == secondary_table_name,
                VersionedFile::DepManager( _) => false,
                VersionedFile::Loc( _) => false,
            },
            VersionedFile::Loc(_) => secondary.is_loc(),
            VersionedFile::DepManager( _) => secondary.is_dep_manager(),
        }
    }

    /// This function returns a reference to a specific version of a definition, if it finds it.
    pub fn get_version(&self, version: i32) -> Result<&Definition> {
        match &self {
            VersionedFile::DB(_, versions) => versions.iter().find(|x| x.version == version).ok_or_else(|| From::from(ErrorKind::SchemaDefinitionNotFound)),
            VersionedFile::DepManager(versions) => versions.iter().find(|x| x.version == version).ok_or_else(|| From::from(ErrorKind::SchemaDefinitionNotFound)),
            VersionedFile::Loc(versions) => versions.iter().find(|x| x.version == version).ok_or_else(|| From::from(ErrorKind::SchemaDefinitionNotFound)),
        }
    }

    /// This function returns the list of the versions in the provided `VersionedFile`.
    pub fn get_version_list(&self) -> &[Definition] {
        match &self {
            VersionedFile::DB(_, versions) => versions,
            VersionedFile::DepManager(versions) => versions,
            VersionedFile::Loc(versions) => versions,
        }
    }

    /// This function adds the provided version to the provided `VersionedFile`, replacing an existing version if there is a conflict.
    pub fn add_version(&mut self, version: &Definition) {
        match self {
            VersionedFile::DB(_, ref mut versions) => match versions.iter().position(|x| x.version == version.version) {
                Some(position) => { versions.splice(position..=position, [version].iter().cloned().cloned()); },
                None => versions.push(version.clone()),
            }
            VersionedFile::DepManager(ref mut versions) => match versions.iter().position(|x| x.version == version.version) {
                Some(position) => { versions.splice(position..=position, [version].iter().cloned().cloned()); },
                None => versions.push(version.clone()),
            }
            VersionedFile::Loc(ref mut versions) => match versions.iter().position(|x| x.version == version.version) {
                Some(position) => { versions.splice(position..=position, [version].iter().cloned().cloned()); },
                None => versions.push(version.clone()),
            }
        }
    }

    /// This function tries to remove a specific version from the provided `VersionedFile`.
    ///
    /// If the version doesn't exist, it does nothing.
    pub fn remove_version(&mut self, version: i32) {
        match self {
            VersionedFile::DB(_, versions) =>  if let Some(position) = versions.iter_mut().position(|x| x.version == version) { versions.remove(position); }
            VersionedFile::DepManager(versions) => if let Some(position) = versions.iter_mut().position(|x| x.version == version) { versions.remove(position); }
            VersionedFile::Loc(versions) => if let Some(position) = versions.iter_mut().position(|x| x.version == version) { versions.remove(position); }
        }
    }
}

/// Implementation of `Definition`.
impl Definition {

    /// This function creates a new empty `Definition` for the version provided.
    pub fn new(version: i32) -> Definition {
        Definition {
            version,
            localised_fields: vec![],
            fields: vec![],
        }
    }

    /// This function updates the fields in the provided definition with the data in the provided RawDefinition.
    ///
    /// Not all data is updated though, only:
    /// - Is Key.
    /// - Max Lenght.
    /// - Default Value.
    /// - Filename Relative Path.
    /// - Is Filename.
    /// - Is Reference.
    /// - Lookup.
    /// - CA Order.
    pub fn update_from_raw_definition(&mut self, raw_definition: &RawDefinition) {
        for (index, raw_field) in raw_definition.fields.iter().enumerate() {
            for field in &mut self.fields {
                if field.name == raw_field.name {
                    if (raw_field.primary_key == "1" && !field.is_key) || (raw_field.primary_key == "0" && field.is_key) {
                        field.is_key = raw_field.primary_key == "1";
                    }

                    if let Some(ref lenght) = raw_field.max_length {
                        if let Ok(lenght) = lenght.parse::<i32>() {
                            field.max_length = lenght;
                        }
                    }

                    if raw_field.default_value.is_some() {
                        field.default_value = raw_field.default_value.clone();
                    }

                    if raw_field.filename_relative_path.is_some() {
                        field.filename_relative_path = raw_field.filename_relative_path.clone();
                    }

                    if let Some(ref description) = raw_field.field_description {
                        field.description = description.to_owned();
                    }

                    if let Some(ref table) = raw_field.column_source_table {
                        if let Some(ref columns) = raw_field.column_source_column {
                            if !columns.is_empty() {
                                field.is_reference = Some((table.to_owned(), columns[0].to_owned()));
                                if columns.len() > 1 {
                                    field.lookup = Some(columns[1..].to_vec());
                                }
                            }
                        }
                    }

                    field.is_filename = raw_field.is_filename.is_some();
                    field.ca_order = index as i16;
                    break;
                }
            }
        }
    }

    /// This function populates the `localised_fields` of a definition with data from the assembly kit.
    pub fn update_from_raw_localisable_fields(&mut self, raw_definition: &RawDefinition, raw_localisable_fields: &[RawLocalisableField]) {
        let raw_table_name = &raw_definition.name.as_ref().unwrap()[..raw_definition.name.as_ref().unwrap().len() - 4];
        let localisable_fields_names = raw_localisable_fields.iter()
            .filter(|x| x.table_name == raw_table_name)
            .map(|x| &*x.field)
            .collect::<Vec<&str>>();

        if !localisable_fields_names.is_empty() {
            let localisable_fields = raw_definition.fields.iter()
                .filter(|x| localisable_fields_names.contains(&&*x.name))
                .collect::<Vec<&RawField>>();

            let fields = localisable_fields.iter().map(|x| From::from(*x)).collect();
            self.localised_fields = fields;
        }
    }
/*
    /// This function creates a new `Definition` from an imported definition from the Assembly Kit.
    ///
    /// Note that this imports the loc fields (they need to be removed manually later) and it doesn't
    /// import the version (this... I think I can do some trick for it).
    pub fn new_from_assembly_kit(imported_table_definition: &RawDefinition, version: i32, table_name: &str) -> Self {
        let mut fields = vec![];
        for (position, field) in imported_table_definition.fields.iter().enumerate() {

            // First, we need to disable a number of known fields that are not in the final tables. We
            // check if the current field is one of them, and ignore it if it's.
            // TODO: Get this list directly from the Assembly Kit.
            if field.name == "game_expansion_key" || // This one exists in one of the advices tables.
                field.name == "localised_text" ||
                field.name == "localised_name" ||
                field.name == "localised_tooltip" ||
                field.name == "description" ||
                field.name == "objectives_team_1" ||
                field.name == "objectives_team_2" ||
                field.name == "short_description_text" ||
                field.name == "historical_description_text" ||
                field.name == "strengths_weaknesses_text" ||
                field.name == "onscreen" ||
                field.name == "onscreen_text" ||
                field.name == "onscreen_name" ||
                field.name == "onscreen_description" ||
                field.name == "on_screen_name" ||
                field.name == "on_screen_description" ||
                field.name == "on_screen_target" {
                continue;
            }
            let field_name = field.name.to_owned();
            let field_is_key = field.primary_key == "1";
            let field_is_reference = if field.column_source_table != None {
                Some((field.column_source_table.clone().unwrap().to_owned(), field.column_source_column.clone().unwrap()[0].to_owned()))
            }
            else {None};

            let field_type = match &*field.field_type {
                "yesno" => FieldType::Boolean,
                "single" | "decimal" | "double" => FieldType::Float,
                "autonumber" => FieldType::LongInteger, // Not always true, but better than nothing.
                "integer" => {

                    // In Warhammer 2 these tables are wrong in the definition schema.
                    if table_name.starts_with("_kv") {
                        FieldType::Float
                    }
                    else {
                        FieldType::Integer
                    }
                },
                "text" => {

                    // Key fields are ALWAYS REQUIRED. This fixes it's detection.
                    if field.name == "key" {
                        FieldType::StringU8
                    }
                    else {
                        match &*field.required {
                            "1" => {
                                // In Warhammer 2 this table has his "value" field broken.
                                if table_name == "_kv_winds_of_magic_params_tables" && field.name == "value" {
                                    FieldType::Float
                                }
                                else {
                                    FieldType::StringU8
                                }
                            },
                            "0" => FieldType::OptionalStringU8,

                            // If we reach this point, we set it to OptionalStringU16. Not because it is it
                            // (we don't have a way to distinguish String types) but to know what fields
                            // reach this point.
                            _ => FieldType::OptionalStringU16,
                        }
                    }
                }
                // If we reach this point, we set it to StringU16. Not because it is it
                // (we don't have a way to distinguish String types) but to know what fields
                // reach this point.
                _ => FieldType::StringU16,

            };

            let field_description = match field.field_description {
                Some(ref description) => description.to_owned(),
                None => String::new(),
            };

            let new_field = Field::new(
                field_name,
                field_type,
                field_is_key,
                None,
                0,
                false,
                None,
                field_is_reference,
                None,
                field_description,
                position as i16
            );
            fields.push(new_field);
        }

        Self {
            version,
            localised_fields: vec![],
            fields,
        }
    }

    /// This function creates a new fake `Definition` from an imported definition from the Assembly Kit.
    ///
    /// For use with the raw table processing.
    pub fn new_fake_from_assembly_kit(imported_table_definition: &RawDefinition, table_name: &str) -> Definition {
        let mut fields = vec![];
        for (position, field) in imported_table_definition.fields.iter().enumerate() {

            let field_name = field.name.to_owned();
            let field_is_key = field.primary_key == "1";
            let field_is_reference = if field.column_source_table != None {
                Some((field.column_source_table.clone().unwrap().to_owned(), field.column_source_column.clone().unwrap()[0].to_owned()))
            }
            else {None};

            let field_type = match &*field.field_type {
                "yesno" => FieldType::Boolean,
                "single" | "decimal" | "double" => FieldType::Float,
                "autonumber" => FieldType::LongInteger, // Not always true, but better than nothing.
                "integer" => {

                    // In Warhammer 2 these tables are wrong in the definition schema.
                    if table_name.starts_with("_kv") {
                        FieldType::Float
                    }
                    else {
                        FieldType::Integer
                    }
                },
                "text" => {

                    // Key fields are ALWAYS REQUIRED. This fixes it's detection.
                    if field.name == "key" {
                        FieldType::StringU8
                    }
                    else {
                        match &*field.required {
                            "1" => {
                                // In Warhammer 2 this table has his "value" field broken.
                                if table_name == "_kv_winds_of_magic_params_tables" && field.name == "value" {
                                    FieldType::Float
                                }
                                else {
                                    FieldType::StringU8
                                }
                            },
                            "0" => FieldType::OptionalStringU8,

                            // If we reach this point, we set it to OptionalStringU16. Not because it is it
                            // (we don't have a way to distinguish String types) but to know what fields
                            // reach this point.
                            _ => FieldType::OptionalStringU16,
                        }
                    }
                }
                // If we reach this point, we set it to StringU16. Not because it is it
                // (we don't have a way to distinguish String types) but to know what fields
                // reach this point.
                _ => FieldType::StringU16,

            };

            let field_description = match field.field_description {
                Some(ref description) => description.to_owned(),
                None => String::new(),
            };

            let new_field = Field::new(
                field_name,
                field_type,
                field_is_key,
                None,
                0,
                false,
                None,
                field_is_reference,
                None,
                field_description,
                position as i16
            );
            fields.push(new_field);
        }

        Definition {
            version: -1,
            localised_fields: vec![],
            fields,
        }
    }
*/
    /// This function generates a MarkDown-encoded diff of two versions of an specific table and adds it to the provided changes list.
    pub fn get_pretty_diff(
        &self,
        version_current: &Self,
        table_name: &str,
        changes: &mut Vec<String>,
    ) {

        // Here it's were things get complex. We have to get, field by field, and check:
        // - If they exists.
        // - If they are in the same position. (TODO)
        // - If they are different, in which case we have to check on what.
        // Changed fields have: Vec<(field_name, vec<(changed_variant, (before, after))>)>.
        let mut new_fields: Vec<Field> = vec![];
        let mut changed_fields: Vec<(String, Vec<(String, (String, String))>)> = vec![];
        let mut removed_fields: Vec<String> = vec![];
        for field_local in &self.fields {
            match version_current.fields.iter().find(|x| x.name == field_local.name) {
                Some(field_current) => {

                    // If they are different, we have to find what do they have different, so we
                    // only show that in the changelog.
                    let mut changes = vec![];
                    if field_local != field_current {
                        if field_local.field_type != field_current.field_type {
                            changes.push(("Type".to_owned(), (format!("{}", field_current.field_type), format!("{}", field_local.field_type))));
                        }

                        if field_local.is_key != field_current.is_key {
                            changes.push(("Is Key".to_owned(), (format!("{}", field_current.is_key), format!("{}", field_local.is_key))));
                        }

                        if field_local.is_reference != field_current.is_reference {
                            changes.push(("Is Reference".to_owned(),
                                (
                                    if let Some((ref_table, ref_column)) = &field_current.is_reference { format!("{}, {}", ref_table, ref_column) }
                                    else { String::new() },
                                    if let Some((ref_table, ref_column)) = &field_local.is_reference { format!("{}, {}", ref_table, ref_column) }
                                    else { String::new() }
                                )
                            ));
                        }

                        if field_local.description != field_current.description {
                            changes.push(("Description".to_owned(), (field_current.description.to_owned(), field_local.description.to_owned())));
                        }
                    }

                    if !changes.is_empty() {
                        changed_fields.push((field_local.name.to_owned(), changes));
                    }
                },

                // If the field doesn't exists, it's new.
                None => new_fields.push(field_local.clone()),
            }
        }

        // We have to check for removed fields too.
        for field_current in &version_current.fields {
            if self.fields.iter().find(|x| x.name == field_current.name).is_none() {
                removed_fields.push(field_current.name.to_owned());
            }
        }

        if !new_fields.is_empty() || !changed_fields.is_empty() || !removed_fields.is_empty() {
            changes.push(format!("  - ***{}***:", table_name));
        }

        for (index, new_field) in new_fields.iter().enumerate() {
            if index == 0 { changes.push("    - **New fields**:".to_owned()); }
            changes.push(format!("      - ***{}***:", new_field.name));
            changes.push(format!("        - **Type**: *{}*.", new_field.field_type));
            changes.push(format!("        - **Is Key**: *{}*.", new_field.is_key));
            if let Some((ref_table, ref_column)) = &new_field.is_reference {
                changes.push(format!("        - **Is Reference**: *{}*/*{}*.", ref_table, ref_column));
            }
            if !new_field.description.is_empty() {
                changes.push(format!("        - **Description**: *{}*.", new_field.description));
            }
        }

        for (index, changed_field) in changed_fields.iter().enumerate() {
            if index == 0 { changes.push("    - **Changed fields**:".to_owned()); }
            changes.push(format!("      - **{}**:", changed_field.0));

            for changed_variant in &changed_field.1 {
                changes.push(format!("        - ***{}***: *{}* => *{}*.", changed_variant.0, (changed_variant.1).0, (changed_variant.1).1));
            }
        }

        for (index, removed_field) in removed_fields.iter().enumerate() {
            if index == 0 { changes.push("    - **Removed fields**:".to_owned()); }
            changes.push(format!("      - *{}*.", removed_field));
        }
    }
}

/// Implementation of `Field`.
impl Field {

    /// This function creates a `Field` using the provided data.
    pub fn new(
        name: String,
        field_type: FieldType,
        is_key: bool,
        default_value: Option<String>,
        max_length: i32,
        is_filename: bool,
        filename_relative_path: Option<String>,
        is_reference: Option<(String, String)>,
        lookup: Option<Vec<String>>,
        description: String,
        ca_order: i16,
    ) -> Self {
        Self {
            name,
            field_type,
            is_key,
            default_value,
            max_length,
            is_filename,
            filename_relative_path,
            is_reference,
            lookup,
            description,
            ca_order
        }
    }
}

/// Default implementation of `Schema`.
impl Default for Schema {
    fn default() -> Self {
        Self {
            version: CURRENT_STRUCTURAL_VERSION,
            versioned_files: vec![]
        }
    }
}

/// Default implementation of `FieldType`.
impl Default for Field {
    fn default() -> Self {
        Self {
            name: String::from("new_field"),
            field_type: FieldType::StringU8,
            is_key: false,
            default_value: None,
            max_length: 0,
            is_filename: false,
            filename_relative_path: None,
            is_reference: None,
            lookup: None,
            description: String::from(""),
            ca_order: -1
        }
    }
}

/// Display implementation of `FieldType`.
impl Display for FieldType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            FieldType::Boolean => write!(f, "Boolean"),
            FieldType::Float => write!(f, "Float"),
            FieldType::Integer => write!(f, "Integer"),
            FieldType::LongInteger => write!(f, "Long Integer"),
            FieldType::StringU8 => write!(f, "StringU8"),
            FieldType::StringU16 => write!(f, "StringU16"),
            FieldType::OptionalStringU8 => write!(f, "OptionalStringU8"),
            FieldType::OptionalStringU16 => write!(f, "OptionalStringU16"),
            FieldType::Sequence(sequence) => write!(f, "Sequence of: {:#?}", sequence),
        }
    }
}

/// Implementation of `From<&RawDefinition>` for `Definition.
impl From<&RawDefinition> for Definition {
    fn from(raw_definition: &RawDefinition) -> Self {
        let mut definition = Self::new(-1);
        definition.fields = raw_definition.fields.iter().map(From::from).collect();
        definition
    }
}


/// Implementation of `From<&RawField>` for `Field.
impl From<&RawField> for Field {
    fn from(raw_field: &RawField) -> Self {
        let field_type = match &*raw_field.field_type {
            "Boolean" => FieldType::Boolean,
            "Float" => FieldType::Float,
            "Integer" => FieldType::Integer,
            "LongInteger" => FieldType::LongInteger,
            "StringU8" => FieldType::StringU8,
            "StringU16" => FieldType::StringU16,
            "OptionalStringU8" => FieldType::OptionalStringU8,
            "OptionalStringU16" => FieldType::OptionalStringU16,
            _ => FieldType::StringU8,
        };

        let max_length = if let Some(x) = &raw_field.max_length {
            if let Ok(y) = x.parse::<i32>() { y }
            else { 0 }
        } else { 0 };

        let (is_reference, lookup) = if let Some(x) = &raw_field.column_source_table {
            if let Some(y) = &raw_field.column_source_column {
                if y.len() > 1 { (Some((x.to_owned(), y[0].to_owned())), Some(y[1..].to_vec()))}
                else { (Some((x.to_owned(), y[0].to_owned())), None) }
            } else { (None, None) }
        }
        else { (None, None) };

        let mut field = Self::default();
        field.name = raw_field.name.to_owned();
        field.field_type = field_type;
        field.is_key = raw_field.primary_key == "1";
        field.default_value = raw_field.default_value.clone();
        field.max_length = max_length;
        field.is_filename = raw_field.is_filename.is_some();
        field.filename_relative_path = raw_field.filename_relative_path.clone();
        field.is_reference = is_reference;
        field.lookup = lookup;
        field.description = if let Some(x) = &raw_field.field_description { x.to_owned() } else { String::new() };
        field
    }
}
