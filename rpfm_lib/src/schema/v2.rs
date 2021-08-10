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
Module with the code to support migration operations from Schema V2 onwards.

Schema V2 was the one used by RPFM between 2.0 and 2.1. Was written in Ron, Versioned.
This module contains only the code needed for reading/writing Schemas V2 and for migrating them to Schemas V3.

In case it's not clear enough, this is for supporting legacy schemas, not intended to be used externally in ANY other way.

The basic structure of an V2 `Schema` is:
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
!*/

use rayon::prelude::*;
use ron::de::from_reader;
use ron::ser::{to_string_pretty, PrettyConfig};
use serde_derive::{Serialize, Deserialize};

use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::fs::File;
use std::io::{BufReader, Write};

use rpfm_error::Result;

use crate::config::get_config_path;
use crate::schema::SCHEMA_FOLDER;
use crate::SUPPORTED_GAMES;

use crate::schema::Schema as SchemaV3;
use crate::schema::VersionedFile as VersionedFileV3;
use crate::schema::Definition as DefinitionV3;
use crate::schema::FieldType as FieldTypeV3;
use crate::schema::Field as FieldV3;

//---------------------------------------------------------------------------//
//                              Enum & Structs
//---------------------------------------------------------------------------//

/// This struct represents a Schema File in memory, ready to be used to decode versioned PackedFiles.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct SchemaV2 {

    /// It stores the structural version of the Schema.
    version: u16,

    /// It stores the versioned files inside the Schema.
    pub versioned_files: Vec<VersionedFileV2>
}

/// This enum defines all types of versioned files that the schema system supports.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum VersionedFileV2 {

    /// It stores the name of the table, and a `Vec<Definition>` with the definitions for each version of that table decoded.
    DB(String, Vec<DefinitionV2>),

    /// It stores a `Vec<Definition>` to decode the dependencies of a PackFile.
    DepManager(Vec<DefinitionV2>),

    /// It stores a `Vec<Definition>` with the definitions for each version of Loc files decoded (currently, only version `1`).
    Loc(Vec<DefinitionV2>),
}

/// This struct contains all the data needed to decode a specific version of a versioned PackedFile.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct DefinitionV2 {

    /// The version of the PackedFile the definition is for. These versions are:
    /// - `-1`: for fake `Definition`, used for dependency resolving stuff.
    /// - `0`: for unversioned PackedFiles.
    /// - `1+`: for versioned PackedFiles.
    pub version: i32,

    /// This is a collection of all `Field`s the PackedFile uses, in the order it uses them.
    pub fields: Vec<FieldV2>,

    /// This is a list of all the fields from this definition that are moved to a Loc PackedFile on exporting.
    pub localised_fields: Vec<FieldV2>,
}

/// This struct holds all the relevant data do properly decode a field from a versioned PackedFile.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct FieldV2 {

    /// Name of the field. Should contain no spaces, using `_` instead.
    pub name: String,

    /// Type of the field.
    pub field_type: FieldTypeV2,

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
pub enum FieldTypeV2 {
    Boolean,
    Float,
    Integer,
    LongInteger,
    StringU8,
    StringU16,
    OptionalStringU8,
    OptionalStringU16,
    Sequence(DefinitionV2)
}

//---------------------------------------------------------------------------//
//                       Enum & Structs Implementations
//---------------------------------------------------------------------------//

impl SchemaV2 {

    /// This function adds a new `VersionedFile` to the schema. This checks if the provided `VersionedFile`
    /// already exists, and replace it if neccesary.
    pub fn add_versioned_file(&mut self, versioned_file: &VersionedFileV2) {
        match self.versioned_files.iter().position(|x| x.conflict(versioned_file)) {
            Some(position) => { self.versioned_files.splice(position..=position, [versioned_file.clone()].iter().cloned()); },
            None => self.versioned_files.push(versioned_file.clone()),
        }
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
                VersionedFileV2::DB(table_name_a, _) => {
                    match b {
                        VersionedFileV2::DB(table_name_b, _) => table_name_a.cmp(&table_name_b),
                        _ => Ordering::Less,
                    }
                }
                VersionedFileV2::DepManager(_) => {
                    match b {
                        VersionedFileV2::DB(_,_) => Ordering::Greater,
                        VersionedFileV2::DepManager(_) => Ordering::Equal,
                        VersionedFileV2::Loc(_) => Ordering::Less,
                    }
                }
                VersionedFileV2::Loc(_) => {
                    match b {
                        VersionedFileV2::Loc(_) => Ordering::Equal,
                        _ => Ordering::Greater,
                    }
                }
            }
        });
    }

    pub fn update() {
        println!("Importing schemas from V2 to V3");
        let mut legacy_schemas = SUPPORTED_GAMES.get_games().iter().map(|y| (y.get_game_key_name(), Self::load(&y.get_schema_name()))).filter_map(|(x, y)| if let Ok(y) = y { Some((x, From::from(&y))) } else { None }).collect::<BTreeMap<String, SchemaV3>>();
        println!("Amount of SchemasV2: {:?}", legacy_schemas.len());
        legacy_schemas.par_iter_mut().for_each(|(game, legacy_schema)| {
            if let Some(file_name) = SUPPORTED_GAMES.get_games().iter().filter_map(|y| if &y.get_game_key_name() == game { Some(y.get_schema_name()) } else { None }).find(|_| true) {
                if legacy_schema.save(&file_name).is_ok() {
                    println!("SchemaV2 for game {} updated to SchemaV3.", game);
                }
            }
        });
    }
}

impl VersionedFileV2 {

    /// This function returns true if both `VersionFile` are conflicting (they're the same, but their definitions may be different).
    pub fn conflict(&self, secondary: &Self) -> bool {
        match &self {
            Self::DB(table_name, _) => match &secondary {
                Self::DB(secondary_table_name, _) => table_name == secondary_table_name,
                Self::DepManager( _) => false,
                Self::Loc( _) => false,
            },
            Self::Loc(_) => secondary.is_loc(),
            Self::DepManager( _) => secondary.is_dep_manager(),
        }
    }

    /// This function returns true if the provided `VersionedFile` is a Dependency Manager Definition. Otherwise, it returns false.
    pub fn is_dep_manager(&self) -> bool {
        matches!(*self, Self::DepManager(_))
    }

    /// This function returns true if the provided `VersionedFile` is a Loc Definition. Otherwise, it returns false.
    pub fn is_loc(&self) -> bool {
        matches!(*self, Self::Loc(_))
    }
}

impl DefinitionV2 {
    pub fn new(version: i32) -> DefinitionV2 {
        DefinitionV2 {
            version,
            fields: vec![],
            localised_fields: vec![],
        }
    }
}

impl Default for FieldV2 {
    fn default() -> Self {
        Self {
            name: String::from("new_field"),
            field_type: FieldTypeV2::StringU8,
            is_key: false,
            default_value: None,
            max_length: 0,
            is_filename: false,
            filename_relative_path: None,
            is_reference: None,
            lookup: None,
            description: String::from(""),
            ca_order: -1,
        }
    }
}


impl From<&SchemaV2> for SchemaV3 {
    fn from(legacy_schema: &SchemaV2) -> Self {
        let mut schema = Self::default();
        legacy_schema.versioned_files.iter().map(From::from).for_each(|x| schema.versioned_files.push(x));
        schema
    }
}

impl From<&VersionedFileV2> for VersionedFileV3 {
    fn from(legacy_table_definitions: &VersionedFileV2) -> Self {
        match legacy_table_definitions {
            VersionedFileV2::DB(name, definitions) => Self::DB(name.to_string(), definitions.iter().map(From::from).collect()),
            VersionedFileV2::DepManager(definitions) => Self::DepManager(definitions.iter().map(From::from).collect()),
            VersionedFileV2::Loc(definitions) => Self::Loc(definitions.iter().map(From::from).collect()),
        }
    }
}

impl From<&DefinitionV2> for DefinitionV3 {
    fn from(legacy_table_definition: &DefinitionV2) -> Self {
        let mut definition = Self::new(legacy_table_definition.version);
        legacy_table_definition.fields.iter().map(From::from).for_each(|x| definition.fields.push(x));
        legacy_table_definition.localised_fields.iter().map(From::from).for_each(|x| definition.localised_fields.push(x));
        definition
    }
}

impl From<&FieldV2> for FieldV3 {
    fn from(legacy_field: &FieldV2) -> Self {
        Self {
            name: legacy_field.name.to_owned(),
            field_type: From::from(&legacy_field.field_type),
            is_key: legacy_field.is_key,
            default_value: legacy_field.default_value.clone(),
            max_length: legacy_field.max_length,
            is_filename: legacy_field.is_filename,
            filename_relative_path: legacy_field.filename_relative_path.clone(),
            is_reference: legacy_field.is_reference.clone(),
            lookup: legacy_field.lookup.clone(),
            description: legacy_field.description.to_owned(),
            ca_order: legacy_field.ca_order,
            ..Default::default()
        }
    }
}

impl From<&FieldTypeV2> for FieldTypeV3 {
    fn from(legacy_field_type: &FieldTypeV2) -> Self {
        match legacy_field_type {
            FieldTypeV2::Boolean => Self::Boolean,
            FieldTypeV2::Float => Self::F32,
            FieldTypeV2::Integer => Self::I32,
            FieldTypeV2::LongInteger => Self::I64,
            FieldTypeV2::StringU8 => Self::StringU8,
            FieldTypeV2::StringU16 => Self::StringU16,
            FieldTypeV2::OptionalStringU8 => Self::OptionalStringU8,
            FieldTypeV2::OptionalStringU16 => Self::OptionalStringU16,
            FieldTypeV2::Sequence(sequence) => Self::SequenceU32(From::from(sequence)),
        }
    }
}

/// Default implementation of `SchemaV3`.
impl Default for SchemaV2 {
    fn default() -> Self {
        Self {
            version: 2,
            versioned_files: vec![]
        }
    }
}
