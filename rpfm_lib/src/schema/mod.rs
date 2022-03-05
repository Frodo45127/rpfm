//---------------------------------------------------------------------------//
// Copyright (c) 2017-2022 Ismael Gutiérrez González. All rights reserved.
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
    version: 3,
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
                        is_filename: false,
                        filename_relative_path: None,
                        is_reference: None,
                        lookup: None,
                        description: "",
                        ca_order: -1,
                        is_bitwise: 0,
                        enum_values: {},
                        is_part_of_colour: None,
                    ),
                    (
                        name: "value",
                        field_type: F32,
                        is_key: false,
                        default_value: None,
                        is_filename: false,
                        filename_relative_path: None,
                        is_reference: None,
                        lookup: None,
                        description: "",
                        ca_order: -1,
                        is_bitwise: 0,
                        enum_values: {},
                        is_part_of_colour: None,
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

use git2::{Reference, ReferenceFormat, Repository, Signature, StashFlags, build::CheckoutBuilder};
use itertools::Itertools;
use rayon::prelude::*;
use ron::de::from_bytes;
use ron::ser::{to_string_pretty, PrettyConfig};
use serde_derive::{Serialize, Deserialize};

use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::fs::{DirBuilder, File};
use std::{fmt, fmt::Display};
use std::io::{BufReader, Read, Write};
use std::process::Command as SystemCommand;

use rpfm_error::{Error, ErrorKind, Result};

use crate::assembly_kit::localisable_fields::RawLocalisableField;
use crate::assembly_kit::table_definition::{RawDefinition, RawField};
use crate::common::get_schemas_path;
use crate::dependencies::Dependencies;
use crate::settings::get_config_path;
use crate::{SETTINGS, SCHEMA_PATCHES, GAME_SELECTED};
use crate::SUPPORTED_GAMES;

// Legacy Schemas, to keep backwards compatibility during updates.
pub(crate) mod v3;
pub(crate) mod v2;
pub(crate) mod v1;
pub(crate) mod v0;
pub mod patch;

/// Name of the folder containing all the schemas.
pub const SCHEMA_FOLDER: &str = "schemas";

const BINARY_EXTENSION: &str = ".bin";

pub const SCHEMA_REPO: &str = "https://github.com/Frodo45127/rpfm-schemas";
pub const REMOTE: &str = "origin";
pub const BRANCH: &str = "master";

/// Current structural version of the Schema, for compatibility purposes.
const CURRENT_STRUCTURAL_VERSION: u16 = 4;

/// Name for unamed colour groups.
pub const MERGE_COLOUR_NO_NAME: &str = "Unnamed Colour Group";

pub const MERGE_COLOUR_POST: &str = "_hex";

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

    /// It stores a `Vec<Definition>` with the definitions for each version of AnimFragment files decoded.
    AnimFragment(Vec<Definition>),

    /// It stores a `Vec<Definition>` with the definitions for each version of AnomTable files decoded.
    AnimTable(Vec<Definition>),

    /// It stores the name of the table, and a `Vec<Definition>` with the definitions for each version of that table decoded.
    DB(String, Vec<Definition>),

    /// It stores a `Vec<Definition>` to decode the dependencies of a PackFile.
    DepManager(Vec<Definition>),

    /// It stores a `Vec<Definition>` with the definitions for each version of Loc files decoded (currently, only version `1`).
    Loc(Vec<Definition>),

    /// It stores a `Vec<Definition>` with the definitions for each version of MatchedCombat files decoded.
    MatchedCombat(Vec<Definition>),
}

/// This struct contains all the data needed to decode a specific version of a versioned PackedFile.
#[derive(Clone, PartialEq, Eq, PartialOrd, Debug, Default, Serialize, Deserialize)]
pub struct Definition {

    /// The version of the PackedFile the definition is for. These versions are:
    /// - `-1`: for fake `Definition`, used for dependency resolving stuff.
    /// - `0`: for unversioned PackedFiles.
    /// - `1+`: for versioned PackedFiles.
    version: i32,

    /// This is a collection of all `Field`s the PackedFile uses, in the order it uses them.
    fields: Vec<Field>,

    /// This is a list of all the fields from this definition that are moved to a Loc PackedFile on exporting.
    localised_fields: Vec<Field>,
}

/// This struct holds all the relevant data do properly decode a field from a versioned PackedFile.
#[derive(Clone, PartialEq, Eq, PartialOrd, Debug, Serialize, Deserialize)]
pub struct Field {

    /// Name of the field. Should contain no spaces, using `_` instead.
    name: String,

    /// Type of the field.
    field_type: FieldType,

    /// `True` if the field is a `Key` field of a table. `False` otherwise.
    is_key: bool,

    /// The default value of the field.
    default_value: Option<String>,

    /// If the field's data corresponds to a filename.
    is_filename: bool,

    /// Path where the file in the data of the field can be, if it's restricted to one path.
    filename_relative_path: Option<String>,

    /// `Some(referenced_table, referenced_column)` if the field is referencing another table/column. `None` otherwise.
    is_reference: Option<(String, String)>,

    /// `Some(referenced_columns)` if the field is using another column/s from the referenced table for lookup values.
    lookup: Option<Vec<String>>,

    /// Aclarative description of what the field is for.
    description: String,

    /// Visual position in CA's Table. `-1` means we don't know its position.
    ca_order: i16,

    /// Variable to tell if this column is a bitwise column (spanned accross multiple columns) or not. Only applicable to numeric fields.
    is_bitwise: i32,

    /// Variable that specifies the "Enum" values for each value in this field.
    enum_values: BTreeMap<i32, String>,

    /// If the field is part of a 3-part RGB column set, and which one (R, G or B) it is.
    is_part_of_colour: Option<u8>,
}

/// This enum defines every type of field the lib can encode/decode.
#[derive(Clone, PartialEq, Eq, PartialOrd, Debug, Serialize, Deserialize)]
pub enum FieldType {
    Boolean,
    F32,
    F64,
    I16,
    I32,
    I64,
    ColourRGB,
    StringU8,
    StringU16,
    OptionalStringU8,
    OptionalStringU16,
    SequenceU16(Definition),
    SequenceU32(Definition)
}

/// This enum controls the possible responses from the server when asking if there is a new Schema update.
#[derive(Debug, Serialize, Deserialize)]
pub enum APIResponseSchema {
    NewUpdate,
    NoUpdate,
    NoLocalFiles,
}

//---------------------------------------------------------------------------//
//                       Enum & Structs Implementations
//---------------------------------------------------------------------------//

/// Implementation of `Schema`.
impl Schema {

    /// This function adds a new `VersionedFile` to the schema. This checks if the provided `VersionedFile`
    /// already exists, and replace it if necessary.
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

    /// This function returns a copy of a specific `VersionedFile` of AnimFragment Type from the provided `Schema`.
    ///
    /// By default, we assume there is only one AnimFragment `VersionedFile` in the `Schema`, so we return that one if we find it.
    pub fn get_versioned_file_anim_fragment(&self) -> Result<VersionedFile> {
        self.versioned_files.par_iter().find_any(|x| x.is_anim_fragment()).cloned().ok_or_else(|| From::from(ErrorKind::SchemaVersionedFileNotFound))
    }

    /// This function returns a reference to a specific `VersionedFile` of AnimFragment Type from the provided `Schema`.
    ///
    /// By default, we assume there is only one AnimFragment `VersionedFile` in the `Schema`, so we return that one if we find it.
    pub fn get_ref_versioned_file_anim_fragment(&self) -> Result<&VersionedFile> {
        self.versioned_files.par_iter().find_any(|x| x.is_anim_fragment()).ok_or_else(|| From::from(ErrorKind::SchemaVersionedFileNotFound))
    }

    /// This function returns a mutable reference to a specific `VersionedFile` of AnimFragment Type from the provided `Schema`.
    ///
    /// By default, we assume there is only one AnimFragment `VersionedFile` in the `Schema`, so we return that one if we find it.
    pub fn get_ref_mut_versioned_file_anim_fragment(&mut self) -> Result<&mut VersionedFile> {
        self.versioned_files.par_iter_mut().find_any(|x| x.is_anim_fragment()).ok_or_else(|| From::from(ErrorKind::SchemaVersionedFileNotFound))
    }

    /// This function returns a copy of a specific `VersionedFile` of AnimTable Type from the provided `Schema`.
    ///
    /// By default, we assume there is only one AnimTable `VersionedFile` in the `Schema`, so we return that one if we find it.
    pub fn get_versioned_file_animtable(&self) -> Result<VersionedFile> {
        self.versioned_files.par_iter().find_any(|x| x.is_animtable()).cloned().ok_or_else(|| From::from(ErrorKind::SchemaVersionedFileNotFound))
    }

    /// This function returns a reference to a specific `VersionedFile` of AnimTable Type from the provided `Schema`.
    ///
    /// By default, we assume there is only one AnimTable `VersionedFile` in the `Schema`, so we return that one if we find it.
    pub fn get_ref_versioned_file_animtable(&self) -> Result<&VersionedFile> {
        self.versioned_files.par_iter().find_any(|x| x.is_animtable()).ok_or_else(|| From::from(ErrorKind::SchemaVersionedFileNotFound))
    }

    /// This function returns a mutable reference to a specific `VersionedFile` of AnimTable Type from the provided `Schema`.
    ///
    /// By default, we assume there is only one AnimTable `VersionedFile` in the `Schema`, so we return that one if we find it.
    pub fn get_ref_mut_versioned_file_animtable(&mut self) -> Result<&mut VersionedFile> {
        self.versioned_files.par_iter_mut().find_any(|x| x.is_animtable()).ok_or_else(|| From::from(ErrorKind::SchemaVersionedFileNotFound))
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

    /// This function returns a copy of a specific `VersionedFile` of MatchedCombat Type from the provided `Schema`.
    ///
    /// By default, we assume there is only one MatchedCombat `VersionedFile` in the `Schema`, so we return that one if we find it.
    pub fn get_versioned_file_matched_combat(&self) -> Result<VersionedFile> {
        self.versioned_files.par_iter().find_any(|x| x.is_matched_combat()).cloned().ok_or_else(|| From::from(ErrorKind::SchemaVersionedFileNotFound))
    }

    /// This function returns a reference to a specific `VersionedFile` of MatchedCombat Type from the provided `Schema`.
    ///
    /// By default, we assume there is only one MatchedCombat `VersionedFile` in the `Schema`, so we return that one if we find it.
    pub fn get_ref_versioned_file_matched_combat(&self) -> Result<&VersionedFile> {
        self.versioned_files.par_iter().find_any(|x| x.is_matched_combat()).ok_or_else(|| From::from(ErrorKind::SchemaVersionedFileNotFound))
    }

    /// This function returns a mutable reference to a specific `VersionedFile` of MatchedCombat Type from the provided `Schema`.
    ///
    /// By default, we assume there is only one MatchedCombat `VersionedFile` in the `Schema`, so we return that one if we find it.
    pub fn get_ref_mut_versioned_file_matched_combat(&mut self) -> Result<&mut VersionedFile> {
        self.versioned_files.par_iter_mut().find_any(|x| x.is_matched_combat()).ok_or_else(|| From::from(ErrorKind::SchemaVersionedFileNotFound))
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
    pub fn get_ref_last_definition_db(&self, table_name: &str, dependencies: &Dependencies) -> Result<&Definition> {

        // Version is... complicated. We don't really want the last one, but the last one compatible with our game.
        // So we have to try to get it first from the Dependency Database first. If that fails, we fall back to the schema.
        if let Some(table) = dependencies.get_db_tables_from_cache(table_name, true, false)?.iter()
            .max_by(|x, y| x.get_ref_definition().get_version().cmp(&y.get_ref_definition().get_version())) {
            self.get_ref_versioned_file_db(table_name)?.get_version(table.get_ref_definition().get_version())
        }

        // If there was no coincidence in the dependency database... we risk ourselves getting the last definition we have for
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

        let mut file = BufReader::new(File::open(&file_path)?);
        let mut data = Vec::with_capacity(file.get_ref().metadata()?.len() as usize);
        file.read_to_end(&mut data)?;
        from_bytes(&data).map_err(From::from)
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

        // Make sure all definitions are properly sorted.
        self.versioned_files.iter_mut().for_each(|x| {
            match x {
                VersionedFile::AnimFragment(ref mut versions) |
                VersionedFile::AnimTable(ref mut versions) |
                VersionedFile::DB(_, ref mut versions) |
                VersionedFile::DepManager(ref mut versions) |
                VersionedFile::Loc(ref mut versions) |
                VersionedFile::MatchedCombat(ref mut versions) => {
                    // Sort them by version number.
                    versions.sort_by(|a, b| b.get_version().cmp(&a.get_version()));
                }
            }
        });
        file.write_all(to_string_pretty(&self, config)?.as_bytes())?;
        Ok(())
    }

    /// This function loads a `Schema` to memory from a file in the `schemas/` folder.
    pub fn load_from_binary(schema_file: &str) -> Result<Self> {
        let mut file_path = get_config_path()?.join(SCHEMA_FOLDER);
        file_path.push(schema_file);
        file_path.set_extension(BINARY_EXTENSION);

        let mut file = BufReader::new(File::open(&file_path)?);
        let mut data = Vec::with_capacity(file.get_ref().metadata()?.len() as usize);
        file.read_to_end(&mut data)?;
        bincode::deserialize(&data).map_err(From::from)
    }

    /// This function saves a `Schema` from memory to a file in the `schemas/` folder.
    pub fn save_to_binary(&mut self, schema_file: &str) -> Result<()> {
        let mut file_path = get_config_path()?.join(SCHEMA_FOLDER);

        // Make sure the path exists to avoid problems with updating schemas.
        DirBuilder::new().recursive(true).create(&file_path)?;

        file_path.push(schema_file);
        file_path.set_extension(BINARY_EXTENSION);
        let file = File::create(&file_path)?;

        self.sort();
        bincode::serialize_into(file, &self).map_err(From::from)
    }

    /// This function sorts a `Schema` alphabetically, so the schema diffs are more or less clean.
    pub fn sort(&mut self) {
        self.versioned_files.sort_by(|a, b| {
            match a {
                VersionedFile::AnimFragment(_) => {
                    match b {
                        VersionedFile::AnimFragment(_) => Ordering::Equal,
                        _ => Ordering::Less,
                    }
                }
                VersionedFile::AnimTable(_) => {
                    match b {
                        VersionedFile::AnimFragment(_) => Ordering::Greater,
                        VersionedFile::AnimTable(_) => Ordering::Equal,
                        _ => Ordering::Less,
                    }
                }
                VersionedFile::DB(table_name_a, _) => {
                    match b {
                        VersionedFile::AnimFragment(_) => Ordering::Greater,
                        VersionedFile::AnimTable(_) => Ordering::Greater,
                        VersionedFile::DB(table_name_b, _) => table_name_a.cmp(table_name_b),
                        _ => Ordering::Less,
                    }
                }
                VersionedFile::DepManager(_) => {
                    match b {
                        VersionedFile::AnimFragment(_) => Ordering::Greater,
                        VersionedFile::AnimTable(_) => Ordering::Greater,
                        VersionedFile::DB(_,_) => Ordering::Greater,
                        VersionedFile::DepManager(_) => Ordering::Equal,
                        VersionedFile::Loc(_) => Ordering::Less,
                        VersionedFile::MatchedCombat(_) => Ordering::Less,
                    }
                }
                VersionedFile::Loc(_) => {
                    match b {
                        VersionedFile::Loc(_) => Ordering::Equal,
                        VersionedFile::MatchedCombat(_) => Ordering::Less,
                        _ => Ordering::Greater,
                    }
                }
                VersionedFile::MatchedCombat(_) => {
                    match b {
                        VersionedFile::MatchedCombat(_) => Ordering::Equal,
                        _ => Ordering::Greater,
                    }
                }
            }
        });
    }

    /// This function exports all the schema files from the `schemas/` folder to `.json`.
    ///
    /// For compatibility purposes.
    pub fn export_to_json() -> Result<()> {
        for schema_file in SUPPORTED_GAMES.get_games().iter().map(|x| x.get_schema_name()) {
            let schema = Schema::load(schema_file)?;

            let mut file_path = get_config_path()?.join(SCHEMA_FOLDER);
            file_path.push(schema_file);
            file_path.set_extension("json");

            let mut file = File::create(&file_path)?;
            file.write_all(serde_json::to_string_pretty(&schema)?.as_bytes())?;
        }
        Ok(())
    }

    /// This function exports all the schema files from the `schemas/` folder to `.xml`.
    ///
    /// For compatibility purposes.
    pub fn export_to_xml() -> Result<()> {
        for schema_file in SUPPORTED_GAMES.get_games().iter().map(|x| x.get_schema_name()) {
            let schema = Schema::load(schema_file)?;

            let mut file_path = get_config_path()?.join(SCHEMA_FOLDER);
            file_path.push(schema_file);
            file_path.set_extension("xml");

            let mut file = File::create(&file_path)?;
            file.write_all(quick_xml::se::to_string(&schema)?.as_bytes())?;
        }
        Ok(())
    }

    /// This function allow us to update all Schemas from any legacy version into the current one.
    ///
    /// NOTE FOR DEV: If you make a new Schema Version, add its update function here.
    pub fn update() {
        v0::SchemaV0::update();
        v1::SchemaV1::update();
        v2::SchemaV2::update();
        v3::SchemaV3::update();
    }

    /// This function checks if there is a new schema update in the schema repo.
    pub fn check_update() -> Result<APIResponseSchema> {

        let schema_path = get_schemas_path()?;
        let mut repo = match Repository::open(&schema_path) {
            Ok(repo) => repo,

            // If this fails, it means we either we don´t have the schemas downloaded, or we have a folder without the .git folder.
            Err(_) => return Ok(APIResponseSchema::NoLocalFiles),
        };

        // Just in case there are loose changes, stash them.
        // Ignore a fail on this, as it's possible we don't have contents to stash.
        let current_branch_name = Reference::normalize_name(repo.head()?.name().unwrap(), ReferenceFormat::ALLOW_ONELEVEL)?.to_lowercase();
        let master_refname = format!("refs/heads/{}", BRANCH);

        let signature = Signature::now("RPFM Updater", "-")?;
        let stash_id = repo.stash_save(&signature, &format!("Stashed changes before checking for updates from branch {}", current_branch_name), Some(StashFlags::INCLUDE_UNTRACKED));

        // In case we're not in master, checkout the master branch.
        if current_branch_name != master_refname {
            repo.set_head(&master_refname)?;
        }

        // Fetch the info of the master branch.
        repo.find_remote(REMOTE)?.fetch(&[BRANCH], None, None)?;
        let analysis = {
            let fetch_head = repo.find_reference("FETCH_HEAD")?;
            let fetch_commit = repo.reference_to_annotated_commit(&fetch_head)?;
            repo.merge_analysis(&[&fetch_commit])?
        };

        // Reset the repo to his original state after the check
        if current_branch_name != master_refname {
            let _ = repo.set_head(&current_branch_name);
        }
        if stash_id.is_ok() {
            let _ = repo.stash_pop(0, None);
        }

        if analysis.0.is_up_to_date() {
            Ok(APIResponseSchema::NoUpdate)
        }

        // If the branch is a fast-forward, or has diverged, ask for an update.
        else if analysis.0.is_fast_forward() || analysis.0.is_normal() || analysis.0.is_none() || analysis.0.is_unborn() {
            Ok(APIResponseSchema::NewUpdate)
        }

        // Otherwise, it means the branches diverged. This may be due to local changes or due to me diverging the master branch with a force push.
        else {
            Err(ErrorKind::SchemaUpdateError.into())
        }
    }

    /// This function downloads the latest revision of the schema repository.
    pub fn update_schema_repo() -> Result<()> {
        let schema_path = get_schemas_path()?;
        let mut repo = match Repository::open(&schema_path) {
            Ok(repo) => repo,
            Err(_) => {

                // If it fails to open, it means either we don't have the .git folder, or we don't have a folder at all.
                // In either case, recreate it and redownload the schemas repo. No more steps are needed here.
                // On windows, remove the read-only flags before doing anything else, or this will fail.
                if cfg!(target_os = "windows") {
                    let path = schema_path.to_string_lossy().to_string() + "\\*.*";
                    let _ = SystemCommand::new("attrib").arg("-r").arg(path).arg("/s").output();
                }
                let _ = std::fs::remove_dir_all(&schema_path);
                DirBuilder::new().recursive(true).create(&schema_path)?;
                match Repository::clone(SCHEMA_REPO, &schema_path) {
                    Ok(_) => return Ok(()),
                    Err(_) => return Err(ErrorKind::SchemaUpdateError.into()),
                }
            }
        };

        // Just in case there are loose changes, stash them.
        // Ignore a fail on this, as it's possible we don't have contents to stash.
        let current_branch_name = Reference::normalize_name(repo.head()?.name().unwrap(), ReferenceFormat::ALLOW_ONELEVEL)?.to_lowercase();
        let master_refname = format!("refs/heads/{}", BRANCH);

        let signature = Signature::now("RPFM Updater", "-")?;
        let stash_id = repo.stash_save(&signature, &format!("Stashed changes before update from branch {}", current_branch_name), Some(StashFlags::INCLUDE_UNTRACKED));

        // In case we're not in master, checkout the master branch.
        if current_branch_name != master_refname {
            repo.set_head(&master_refname)?;
        }

        // If it worked, now we have to do a pull from master. Sadly, git2-rs does not support pull.
        // Instead, we kinda force a fast-forward. Made in StackOverflow.
        repo.find_remote(REMOTE)?.fetch(&[BRANCH], None, None)?;
        let (analysis, fetch_commit_id) = {
            let fetch_head = repo.find_reference("FETCH_HEAD")?;
            let fetch_commit = repo.reference_to_annotated_commit(&fetch_head)?;
            (repo.merge_analysis(&[&fetch_commit])?, fetch_commit.id())
        };

        // If we're up to date, nothing more is needed.
        if analysis.0.is_up_to_date() {

            // Reset the repo to his original state after the check
            if current_branch_name != master_refname {
                let _ = repo.set_head(&current_branch_name);
            }
            if stash_id.is_ok() {
                let _ = repo.stash_pop(0, None);
            }
            Err(ErrorKind::NoSchemaUpdatesAvailable.into())
        }

        // If we can do a fast-forward, we do it. This is the preferred option.
        else if analysis.0.is_fast_forward() {
            let mut reference = repo.find_reference(&master_refname)?;
            reference.set_target(fetch_commit_id, "Fast-Forward")?;
            repo.set_head(&master_refname)?;
            repo.checkout_head(Some(CheckoutBuilder::default().force())).map_err(From::from)
        }

        // If not, we face multiple problems:
        // - If there are uncommitted changes: covered by the stash.
        // - If we're not in the branch: covered by the branch switch.
        // - If the branches diverged: this one... the cleanest way to deal with it should be redownload the repo.
        else if analysis.0.is_normal() || analysis.0.is_none() || analysis.0.is_unborn() {

            // On windows, remove the read-only flags before doing anything else, or this will fail.
            if cfg!(target_os = "windows") {
                let path = schema_path.to_string_lossy().to_string() + "\\*.*";
                let _ = SystemCommand::new("attrib").arg("-r").arg(path).arg("/s").output();
            }
            let _ = std::fs::remove_dir_all(&schema_path);
            Self::update_schema_repo()
        }
        else {

            // Reset the repo to his original state after the check
            if current_branch_name != master_refname {
                let _ = repo.set_head(&current_branch_name);
            }
            if stash_id.is_ok() {
                let _ = repo.stash_pop(0, None);
            }

            Err(ErrorKind::SchemaUpdateError.into())
        }
    }
}

/// Implementation of `VersionedFile`.
impl VersionedFile {

    /// This function returns true if the provided `VersionedFile` is an AnimFragment Definition. Otherwise, it returns false.
    pub fn is_anim_fragment(&self) -> bool {
        matches!(*self, VersionedFile::AnimFragment(_))
    }

    /// This function returns true if the provided `VersionedFile` is an AnimTable Definition. Otherwise, it returns false.
    pub fn is_animtable(&self) -> bool {
        matches!(*self, VersionedFile::AnimTable(_))
    }

    /// This function returns true if the provided `VersionedFile` is a DB Definition. Otherwise, it returns false.
    pub fn is_db(&self) -> bool {
        matches!(*self, VersionedFile::DB(_,_))
    }

    /// This function returns true if the provided `VersionedFile` is a Dependency Manager Definition. Otherwise, it returns false.
    pub fn is_dep_manager(&self) -> bool {
        matches!(*self, VersionedFile::DepManager(_))
    }

    /// This function returns true if the provided `VersionedFile` is a Loc Definition. Otherwise, it returns false.
    pub fn is_loc(&self) -> bool {
        matches!(*self, VersionedFile::Loc(_))
    }

    /// This function returns true if the provided `VersionedFile` is an MatchedCombat Definition. Otherwise, it returns false.
    pub fn is_matched_combat(&self) -> bool {
        matches!(*self, VersionedFile::MatchedCombat(_))
    }

    /// This function returns true if both `VersionFile` are conflicting (they're the same, but their definitions may be different).
    pub fn conflict(&self, secondary: &VersionedFile) -> bool {
        match &self {
            VersionedFile::AnimFragment(_) => secondary.is_anim_fragment(),
            VersionedFile::AnimTable(_) => secondary.is_animtable(),
            VersionedFile::DB(table_name,_) => match &secondary {
                VersionedFile::DB(secondary_table_name, _) => table_name == secondary_table_name,
                _ => false,
            },
            VersionedFile::Loc(_) => secondary.is_loc(),
            VersionedFile::DepManager(_) => secondary.is_dep_manager(),
            VersionedFile::MatchedCombat(_) => secondary.is_matched_combat(),
        }
    }

    /// This function returns a reference to a specific version of a definition, if it finds it.
    pub fn get_version(&self, version: i32) -> Result<&Definition> {
        match &self {
            VersionedFile::AnimFragment(versions) |
            VersionedFile::AnimTable(versions) |
            VersionedFile::DB(_, versions) |
            VersionedFile::DepManager(versions) |
            VersionedFile::Loc(versions) |
            VersionedFile::MatchedCombat(versions) => versions.iter().find(|x| x.version == version).ok_or_else(|| From::from(ErrorKind::SchemaDefinitionNotFound)),
        }
    }

    /// This function returns a reference to all the alternative definitions of a VersionedFile.
    pub fn get_version_alternatives(&self) -> Vec<&Definition> {
        match &self {
            VersionedFile::AnimFragment(versions) |
            VersionedFile::AnimTable(versions) |
            VersionedFile::DB(_, versions) |
            VersionedFile::DepManager(versions) |
            VersionedFile::Loc(versions) |
            VersionedFile::MatchedCombat(versions) => versions.iter().filter(|x| x.version <= 0).collect::<Vec<&Definition>>(),
        }
    }

    /// This function returns a mutable reference to a specific version of a definition, if it finds it.
    pub fn get_ref_mut_version(&mut self, version: i32) -> Result<&mut Definition> {
        match self {
            VersionedFile::AnimFragment(versions) |
            VersionedFile::AnimTable(versions) |
            VersionedFile::DB(_, versions) |
            VersionedFile::DepManager(versions) |
            VersionedFile::Loc(versions) |
            VersionedFile::MatchedCombat(versions) => versions.iter_mut().find(|x| x.version == version).ok_or_else(|| From::from(ErrorKind::SchemaDefinitionNotFound)),
        }
    }


    /// This function returns the list of the versions in the provided `VersionedFile`.
    pub fn get_version_list(&self) -> &[Definition] {
        match &self {
            VersionedFile::AnimFragment(versions) |
            VersionedFile::AnimTable(versions) |
            VersionedFile::DB(_, versions) |
            VersionedFile::DepManager(versions) |
            VersionedFile::Loc(versions) |
            VersionedFile::MatchedCombat(versions) => versions,
        }
    }

    /// This function adds the provided version to the provided `VersionedFile`, replacing an existing version if there is a conflict.
    pub fn add_version(&mut self, version: &Definition) {
        match self {
            VersionedFile::AnimFragment(ref mut versions) |
            VersionedFile::AnimTable(ref mut versions) |
            VersionedFile::DB(_, ref mut versions) |
            VersionedFile::DepManager(ref mut versions) |
            VersionedFile::Loc(ref mut versions) |
            VersionedFile::MatchedCombat(ref mut versions) => match versions.iter().position(|x| x.version == version.version) {
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
            VersionedFile::AnimFragment(versions) |
            VersionedFile::AnimTable(versions) |
            VersionedFile::DB(_, versions) |
            VersionedFile::DepManager(versions) |
            VersionedFile::Loc(versions) |
            VersionedFile::MatchedCombat(versions) => if let Some(position) = versions.iter_mut().position(|x| x.version == version) { versions.remove(position); }
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

    /// This function returns the version of the provided definition.
    pub fn get_version(&self) -> i32 {
        self.version
    }

    /// This function returns a reference to the list of fields in the definition.
    pub fn get_ref_fields(&self) -> &[Field] {
        &self.fields
    }

    /// This function returns a mutable reference to the list of fields in the definition.
    pub fn get_ref_mut_fields(&mut self) -> &mut Vec<Field> {
        &mut self.fields
    }

    /// This function returns the reference and lookup data of a definition.
    pub fn get_reference_data(&self) -> BTreeMap<i32, (String, String, Option<Vec<String>>)> {
        self.fields.iter()
            .enumerate()
            .filter(|x| x.1.is_reference.is_some())
            .map(|x| (x.0 as i32, (x.1.is_reference.clone().unwrap().0, x.1.is_reference.clone().unwrap().1, x.1.lookup.clone())))
            .collect()
    }

    /// This function returns the localised fields of the provided definition
    pub fn get_localised_fields(&self) -> &[Field] {
        &self.localised_fields
    }

    /// This function returns the localised fields of the provided definition
    pub fn get_ref_mut_localised_fields(&mut self) -> &mut Vec<Field> {
        &mut self.localised_fields
    }

    /// This function returns the list of fields a table contains, after it has been expanded/changed due to the attributes of each field.
    pub fn get_fields_processed(&self) -> Vec<Field> {
        let mut split_colour_fields: BTreeMap<u8, Field> = BTreeMap::new();
        let mut fields = self.get_ref_fields().iter()
            .filter_map(|x|
                if x.get_is_bitwise() > 1 {
                    let mut fields = vec![x.clone(); x.get_is_bitwise() as usize];
                    fields.iter_mut().enumerate().for_each(|(index, field)| {
                        field.set_name(&format!("{}_{}", field.get_name(), index + 1));
                        field.set_field_type(FieldType::Boolean);
                    });
                    Some(fields)
                }

                else if !x.get_enum_values().is_empty() {
                    let mut field = x.clone();
                    field.set_field_type(FieldType::StringU8);
                    Some(vec![field; 1])
                }

                else if let Some(colour_index) = x.get_is_part_of_colour() {
                    if split_colour_fields.get(&colour_index).is_none() {
                        let colour_split = x.get_name().rsplitn(2, "_").collect::<Vec<&str>>();
                        let colour_field_name = if colour_split.len() == 2 { format!("{}{}", colour_split[1].to_lowercase(), MERGE_COLOUR_POST) } else { MERGE_COLOUR_NO_NAME.to_lowercase() };

                        let mut field = x.clone();
                        field.set_name(&colour_field_name);
                        field.set_field_type(FieldType::ColourRGB);
                        split_colour_fields.insert(colour_index, field);
                    }

                    None
                }

                else {
                    Some(vec![x.clone(); 1])
                }
            )
            .flatten()
            .collect::<Vec<Field>>();

        // Second pass to add the combined colour fields.
        fields.append(&mut split_colour_fields.values().cloned().collect::<Vec<Field>>());
        fields
    }

    /// Note, this doesn't work with combined fields.
    pub fn get_original_field_from_processed(&self, index: usize) -> Field {
        let fields = self.get_ref_fields();
        let processed = self.get_fields_processed();

        let field_processed = &processed[index];
        let name = if field_processed.get_is_bitwise() > 1 {
            let mut name = field_processed.get_name().to_owned();
            name.drain(..name.rfind('_').unwrap()).collect::<String>()
        }
        else {field_processed.get_name().to_owned() };

        fields.iter().find(|x| x.get_name() == name).unwrap().clone()
    }

    /// This function returns the field list of a definition, properly sorted.
    pub fn get_fields_sorted(&self) -> Vec<Field> {
        let mut fields = self.get_fields_processed().to_vec();
        fields.sort_by(|a, b| {
            if SETTINGS.read().unwrap().settings_bool["tables_use_old_column_order"] {
                if a.get_is_key() && b.get_is_key() { Ordering::Equal }
                else if a.get_is_key() && !b.get_is_key() { Ordering::Less }
                else if !a.get_is_key() && b.get_is_key() { Ordering::Greater }
                else { Ordering::Equal }
            }
            else if a.get_ca_order() == -1 || b.get_ca_order() == -1 { Ordering::Equal }
            else { a.get_ca_order().cmp(&b.get_ca_order()) }
        });
        fields
    }

    /// This function returns the position of a column in a definition, or an error if the column is not found.
    pub fn get_column_position_by_name(&self, column_name: &str) -> Result<usize> {
        self.get_fields_processed()
            .iter()
            .position(|x| x.get_name() == column_name)
            .ok_or_else(|| Error::from(ErrorKind::ColumnNotFoundInTable(column_name.to_owned())))
    }

    /// This function updates the fields in the provided definition with the data in the provided RawDefinition.
    ///
    /// Not all data is updated though, only:
    /// - Is Key.
    /// - Max Length.
    /// - Default Value.
    /// - Filename Relative Path.
    /// - Is Filename.
    /// - Is Reference.
    /// - Lookup.
    /// - CA Order.
    pub fn update_from_raw_definition(&mut self, raw_definition: &RawDefinition) {
        let raw_table_name = &raw_definition.name.as_ref().unwrap()[..raw_definition.name.as_ref().unwrap().len() - 4];
        let mut combined_fields = BTreeMap::new();
        for (index, raw_field) in raw_definition.fields.iter().enumerate() {
            for field in &mut self.fields {
                if field.name == raw_field.name {
                    if (raw_field.primary_key == "1" && !field.is_key) || (raw_field.primary_key == "0" && field.is_key) {
                        field.is_key = raw_field.primary_key == "1";
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
                            if !table.is_empty() && !columns.is_empty() && !columns[0].is_empty() {
                                field.is_reference = Some((table.to_owned(), columns[0].to_owned()));
                                if columns.len() > 1 {
                                    field.lookup = Some(columns[1..].to_vec());
                                }
                            }
                        }
                    }

                    field.is_filename = raw_field.is_filename.is_some();
                    field.ca_order = index as i16;

                    // Detect and group colour fiels.
                    let is_numeric = if let FieldType::I16 = field.field_type { true }
                    else if let FieldType::I32 = field.field_type { true }
                    else if let FieldType::I64 = field.field_type { true }
                    else { false };

                    if is_numeric && raw_table_name != "factions" {
                        if field.name.ends_with("_r") ||
                            field.name.ends_with("_g") ||
                            field.name.ends_with("_b") ||
                            field.name.ends_with("_red") ||
                            field.name.ends_with("_green") ||
                            field.name.ends_with("_blue") ||
                            field.name == "r" ||
                            field.name == "g" ||
                            field.name == "b" ||
                            field.name == "red" ||
                            field.name == "green" ||
                            field.name == "blue" {
                            let colour_split = field.name.rsplitn(2, "_").collect::<Vec<&str>>();
                            let colour_field_name = if colour_split.len() == 2 { format!("{}{}", colour_split[1].to_lowercase(), MERGE_COLOUR_POST) } else { MERGE_COLOUR_NO_NAME.to_lowercase() };

                            match combined_fields.get(&colour_field_name) {
                                Some(group_key) => field.is_part_of_colour = Some(*group_key),
                                None => {
                                    let group_key = combined_fields.keys().len() as u8 + 1;
                                    combined_fields.insert(colour_field_name.to_owned(), group_key);
                                    field.is_part_of_colour = Some(group_key);
                                }
                            }

                        }
                    }
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
            if !self.fields.iter().any(|x| x.name == field_current.name) {
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
        is_filename: bool,
        filename_relative_path: Option<String>,
        is_reference: Option<(String, String)>,
        lookup: Option<Vec<String>>,
        description: String,
        ca_order: i16,
        is_bitwise: i32,
        enum_values: BTreeMap<i32, String>,
        is_part_of_colour: Option<u8>,
    ) -> Self {
        Self {
            name,
            field_type,
            is_key,
            default_value,
            is_filename,
            filename_relative_path,
            is_reference,
            lookup,
            description,
            ca_order,
            is_bitwise,
            enum_values,
            is_part_of_colour
        }
    }

    /// Setter for the `name` field.
    pub fn set_name(&mut self, name: &str) {
        self.name = name.to_owned();
    }

    /// Getter for the `name` field.
    pub fn get_name(&self) -> &str {
        &self.name
    }

    /// Setter for the `field_type` field.
    pub fn set_field_type(&mut self, field_type: FieldType) {
        self.field_type = field_type;
    }

    /// Getter for the `field_type` field.
    pub fn get_field_type(&self) -> FieldType {
        self.field_type.clone()
    }

    /// Getter for a reference of the `field_type` field.
    pub fn get_ref_field_type(&self) -> &FieldType {
        &self.field_type
    }

    /// Getter for a mutable reference of the `field_type` field.
    pub fn get_ref_mut_field_type(&mut self) -> &mut FieldType {
        &mut self.field_type
    }

    /// Getter for the `is_key` field.
    pub fn get_is_key(&self) -> bool {
        self.is_key
    }

    /// Getter for the `default_value` field.
    pub fn get_default_value(&self, table_name: Option<&str>) -> Option<String> {
        if let Some(table_name) = table_name {
            let game = GAME_SELECTED.read().unwrap().get_game_key_name();
            if let Some(default_value) = SCHEMA_PATCHES.read().unwrap().get_data(&game, table_name, self.get_name(), "default_value") {
                return Some(default_value);
            }
        }

        self.default_value.clone()
    }

    /// Getter for the `is_filename` field.
    pub fn get_is_filename(&self) -> bool {
        self.is_filename
    }

    /// Getter for the `filename_relative_path` field.
    pub fn get_filename_relative_path(&self) -> &Option<String> {
        &self.filename_relative_path
    }

    /// Getter for the `is_reference` field.
    pub fn get_is_reference(&self) -> &Option<(String, String)>{
        &self.is_reference
    }

    /// Getter for the `lookup` field.
    pub fn get_lookup(&self) -> &Option<Vec<String>> {
        &self.lookup
    }

    /// Getter for the `description` field.
    pub fn get_description(&self) -> &str {
        &self.description
    }

    /// Getter for the `ca_order` field.
    pub fn get_ca_order(&self) -> i16 {
        self.ca_order
    }

    /// Getter for the `is_bitwise` field.
    pub fn get_is_bitwise(&self) -> i32 {
        self.is_bitwise
    }

    /// Getter for the `enum_values` field.
    pub fn get_enum_values(&self) -> &BTreeMap<i32, String> {
        &self.enum_values
    }

    /// Getter for the `enum_values` field, in an option.
    pub fn get_enum_values_to_option(&self) -> Option<BTreeMap<i32, String>> {
        if self.enum_values.is_empty() { None }
        else { Some(self.enum_values.clone()) }
    }

    /// Getter for the `enum_values` field in a string format.
    pub fn get_enum_values_to_string(&self) -> String {
        self.enum_values.iter().map(|(x, y)| format!("{},{}", x, y)).join(";")
    }

    /// Getter for the `is_part_of_colour` field.
    pub fn get_is_part_of_colour(&self) -> Option<u8> {
        self.is_part_of_colour
    }

    /// Getter for the `cannot_be_empty` field.
    pub fn get_cannot_be_empty(&self, table_name: Option<&str>) -> bool {
        if let Some(table_name) = table_name {
            let game = GAME_SELECTED.read().unwrap().get_game_key_name();
            if let Some(cannot_be_empty) = SCHEMA_PATCHES.read().unwrap().get_data(&game, table_name, self.get_name(), "not_empty") {
                return cannot_be_empty.parse::<bool>().unwrap_or(false);
            }
        }

        false
    }

    /// Getter for the `explanation` field for schema patches.
    pub fn get_schema_patch_explanation(&self, table_name: Option<&str>) -> String {
        if let Some(table_name) = table_name {
            let game = GAME_SELECTED.read().unwrap().get_game_key_name();
            if let Some(explanation) = SCHEMA_PATCHES.read().unwrap().get_data(&game, table_name, self.get_name(), "explanation") {
                return explanation;
            }
        }
        String::new()
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
            is_filename: false,
            filename_relative_path: None,
            is_reference: None,
            lookup: None,
            description: String::from(""),
            ca_order: -1,
            is_bitwise: 0,
            enum_values: BTreeMap::new(),
            is_part_of_colour: None,
        }
    }
}

/// Display implementation of `FieldType`.
impl Display for FieldType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            FieldType::Boolean => write!(f, "Boolean"),
            FieldType::F32 => write!(f, "F32"),
            FieldType::F64 => write!(f, "F64"),
            FieldType::I16 => write!(f, "I16"),
            FieldType::I32 => write!(f, "I32"),
            FieldType::I64 => write!(f, "I64"),
            FieldType::ColourRGB => write!(f, "ColourRGB"),
            FieldType::StringU8 => write!(f, "StringU8"),
            FieldType::StringU16 => write!(f, "StringU16"),
            FieldType::OptionalStringU8 => write!(f, "OptionalStringU8"),
            FieldType::OptionalStringU16 => write!(f, "OptionalStringU16"),
            FieldType::SequenceU16(sequence) => write!(f, "SequenceU16 of: {:#?}", sequence),
            FieldType::SequenceU32(sequence) => write!(f, "SequenceU32 of: {:#?}", sequence),
        }
    }
}

/// Implementation of `From<&RawDefinition>` for `Definition.
impl From<&RawDefinition> for Definition {
    fn from(raw_definition: &RawDefinition) -> Self {
        let mut definition = Self::new(-100);
        definition.fields = raw_definition.fields.iter().map(From::from).collect();
        definition
    }
}


/// Implementation of `From<&RawField>` for `Field.
impl From<&RawField> for Field {
    fn from(raw_field: &RawField) -> Self {
        let field_type = match &*raw_field.field_type {
            "yesno" => FieldType::Boolean,
            "single" => FieldType::F32,
            "double" => FieldType::F64,
            "integer" => FieldType::I32,
            "autonumber" | "card64" => FieldType::I64,
            "colour" => FieldType::ColourRGB,
            "expression" | "text" => {
                if raw_field.required == "1" {
                    FieldType::StringU8
                }
                else {
                    FieldType::OptionalStringU8
                }
            },
            _ => FieldType::StringU8,
        };

        let (is_reference, lookup) = if let Some(x) = &raw_field.column_source_table {
            if let Some(y) = &raw_field.column_source_column {
                if y.len() > 1 { (Some((x.to_owned(), y[0].to_owned())), Some(y[1..].to_vec()))}
                else { (Some((x.to_owned(), y[0].to_owned())), None) }
            } else { (None, None) }
        }
        else { (None, None) };

        Self {
            name: raw_field.name.to_owned(),
            field_type,
            is_key: raw_field.primary_key == "1",
            default_value: raw_field.default_value.clone(),
            is_filename: raw_field.is_filename.is_some(),
            filename_relative_path: raw_field.filename_relative_path.clone(),
            is_reference,
            lookup,
            description: if let Some(x) = &raw_field.field_description { x.to_owned() } else { String::new() },
            ..Default::default()
        }
    }
}
