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
Module with the code to support migration operations from Schema V1 onwards.

Schema V1 was the one used by RPFM during the development of 2.0. Was written in Ron, Unversioned.
This module contains only the code needed for reading/writing Schemas V1 and for migrating them to Schemas V2.

In case it's not clear enough, this is for supporting legacy schemas, not intended to be used externally in ANY other way.

The basic structure of an V1 `Schema` is:
```rust
([
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
                ),
            ],
        ),
    ]),
])
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

use super::v2::*;

//---------------------------------------------------------------------------//
//                              Enum & Structs
//---------------------------------------------------------------------------//

/// This struct represents a SchemaV1 File in memory, ready to be used to decode versioned PackedFiles.
#[derive(Clone, PartialEq, Eq, Debug, Default, Serialize, Deserialize)]
pub struct SchemaV1(pub(crate) Vec<VersionedFileV1>);

/// This enum defines all types of versioned files that the schema system supports.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum VersionedFileV1 {

    /// It stores the name of the table, and a `Vec<Definition>` with the definitions for each version of that table decoded.
    DB(String, Vec<DefinitionV1>),

    /// It stores a `Vec<Definition>` to decode the dependencies of a PackFile.
    DepManager(Vec<DefinitionV1>),

    /// It stores a `Vec<Definition>` with the definitions for each version of Loc files decoded (currently, only version `1`).
    Loc(Vec<DefinitionV1>),
}

/// This struct contains all the data needed to decode a specific version of a versioned PackedFile.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct DefinitionV1 {

    /// The version of the PackedFile the definition is for. These versions are:
    /// - `-1`: for fake `Definition`, used for dependency resolving stuff.
    /// - `0`: for unversioned PackedFiles.
    /// - `1+`: for versioned PackedFiles.
    pub version: i32,

    /// This is a collection of all `Field`s the PackedFile uses, in the order it uses them.
    pub fields: Vec<FieldV1>,
}

/// This struct holds all the relevant data do properly decode a field from a versioned PackedFile.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct FieldV1 {

    /// Name of the field. Should contain no spaces, using `_` instead.
    pub name: String,

    /// Type of the field.
    pub field_type: FieldTypeV1,

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
}

/// This enum defines every type of field the lib can encode/decode.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum FieldTypeV1 {
    Boolean,
    Float,
    Integer,
    LongInteger,
    StringU8,
    StringU16,
    OptionalStringU8,
    OptionalStringU16,
    Sequence(DefinitionV1)
}

//---------------------------------------------------------------------------//
//                       Enum & Structs Implementations
//---------------------------------------------------------------------------//

impl SchemaV1 {

    /// This function adds a new `VersionedFile` to the schema. This checks if the provided `VersionedFile`
    /// already exists, and replace it if neccesary.
    pub fn add_versioned_file(&mut self, versioned_file: &VersionedFileV1) {
        match self.0.iter().position(|x| x.conflict(versioned_file)) {
            Some(position) => { self.0.splice(position..=position, [versioned_file.clone()].iter().cloned()); },
            None => self.0.push(versioned_file.clone()),
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
        self.0.sort_by(|a, b| {
            match a {
                VersionedFileV1::DB(table_name_a, _) => {
                    match b {
                        VersionedFileV1::DB(table_name_b, _) => table_name_a.cmp(&table_name_b),
                        _ => Ordering::Less,
                    }
                }
                VersionedFileV1::DepManager(_) => {
                    match b {
                        VersionedFileV1::DB(_,_) => Ordering::Greater,
                        VersionedFileV1::DepManager(_) => Ordering::Equal,
                        VersionedFileV1::Loc(_) => Ordering::Less,
                    }
                }
                VersionedFileV1::Loc(_) => {
                    match b {
                        VersionedFileV1::Loc(_) => Ordering::Equal,
                        _ => Ordering::Greater,
                    }
                }
            }
        });
    }

    pub fn update() {
        println!("Importing schemas from V1 to V2");
        let mut legacy_schemas = SUPPORTED_GAMES.get_games().iter().map(|y| (y.get_game_key_name(), Self::load(&y.get_schema_name()))).filter_map(|(x, y)| if let Ok(y) = y { Some((x, From::from(&y))) } else { None }).collect::<BTreeMap<String, SchemaV2>>();
        println!("Amount of SchemasV1: {:?}", legacy_schemas.len());
        legacy_schemas.par_iter_mut().for_each(|(game, legacy_schema)| {
            if let Some(file_name) = SUPPORTED_GAMES.get_games().iter().filter_map(|y| if &y.get_game_key_name() == game { Some(y.get_schema_name()) } else { None }).find(|_| true) {
                if legacy_schema.save(&file_name).is_ok() {
                    println!("SchemaV1 for game {} updated to SchemaV2.", game);
                }
            }
        });
    }
}

impl VersionedFileV1 {

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

impl DefinitionV1 {
    pub fn new(version: i32) -> DefinitionV1 {
        DefinitionV1 {
            version,
            fields: vec![],
        }
    }
}

impl Default for FieldV1 {
    fn default() -> Self {
        Self {
            name: String::from("new_field"),
            field_type: FieldTypeV1::StringU8,
            is_key: false,
            default_value: None,
            max_length: 0,
            is_filename: false,
            filename_relative_path: None,
            is_reference: None,
            lookup: None,
            description: String::from(""),
        }
    }
}


impl From<&SchemaV1> for SchemaV2 {
    fn from(legacy_schema: &SchemaV1) -> Self {
        let mut schema = Self::default();
        legacy_schema.0.iter().map(From::from).for_each(|x| schema.versioned_files.push(x));
        schema
    }
}

impl From<&VersionedFileV1> for VersionedFileV2 {
    fn from(legacy_table_definitions: &VersionedFileV1) -> Self {
        match legacy_table_definitions {
            VersionedFileV1::DB(name, definitions) => Self::DB(name.to_string(), definitions.iter().map(From::from).collect()),
            VersionedFileV1::DepManager(definitions) => Self::DepManager(definitions.iter().map(From::from).collect()),
            VersionedFileV1::Loc(definitions) => Self::Loc(definitions.iter().map(From::from).collect()),
        }
    }
}

impl From<&DefinitionV1> for DefinitionV2 {
    fn from(legacy_table_definition: &DefinitionV1) -> Self {
        let mut definition = Self::new(legacy_table_definition.version);
        legacy_table_definition.fields.iter().map(From::from).for_each(|x| definition.fields.push(x));
        definition
    }
}

impl From<&FieldV1> for FieldV2 {
    fn from(legacy_field: &FieldV1) -> Self {
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
            ..Default::default()
        }
    }
}

impl From<&FieldTypeV1> for FieldTypeV2 {
    fn from(legacy_field_type: &FieldTypeV1) -> Self {
        match legacy_field_type {
            FieldTypeV1::Boolean => Self::Boolean,
            FieldTypeV1::Float => Self::Float,
            FieldTypeV1::Integer => Self::Integer,
            FieldTypeV1::LongInteger => Self::LongInteger,
            FieldTypeV1::StringU8 => Self::StringU8,
            FieldTypeV1::StringU16 => Self::StringU16,
            FieldTypeV1::OptionalStringU8 => Self::OptionalStringU8,
            FieldTypeV1::OptionalStringU16 => Self::OptionalStringU16,
            FieldTypeV1::Sequence(sequence) => Self::Sequence(From::from(sequence)),
        }
    }
}
