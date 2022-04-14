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
                        max_length: 0,
                        is_filename: false,
                        filename_relative_path: None,
                        is_reference: None,
                        lookup: None,
                        description: "",
                        ca_order: -1,
                        is_bitwise: 0,
                        enum_values: {},
                    ),
                    (
                        name: "value",
                        field_type: F32,
                        is_key: false,
                        default_value: None,
                        max_length: 0,
                        is_filename: false,
                        filename_relative_path: None,
                        is_reference: None,
                        lookup: None,
                        description: "",
                        ca_order: -1,
                        is_bitwise: 0,
                        enum_values: {},
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
use ron::de::from_reader;
use ron::ser::{to_string_pretty, PrettyConfig};
use serde_derive::{Serialize, Deserialize};

use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::fs::{DirBuilder, File};
use std::io::{BufReader, Write};

use rpfm_error::{ErrorKind, Result};

use crate::schema::SCHEMA_FOLDER;
use crate::settings::get_config_path;
use crate::SUPPORTED_GAMES;

use crate::schema::Schema as SchemaV4;
use crate::schema::VersionedFile as VersionedFileV4;
use crate::schema::Definition as DefinitionV4;
use crate::schema::FieldType as FieldTypeV4;
use crate::schema::Field as FieldV4;

//---------------------------------------------------------------------------//
//                              Enum & Structs
//---------------------------------------------------------------------------//

/// This struct represents a Schema File in memory, ready to be used to decode versioned PackedFiles.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct SchemaV3 {

    /// It stores the structural version of the Schema.
    version: u16,

    /// It stores the versioned files inside the Schema.
    versioned_files: Vec<VersionedFileV3>
}

/// This enum defines all types of versioned files that the schema system supports.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum VersionedFileV3 {

    /// It stores a `Vec<Definition>` with the definitions for each version of AnimFragment files decoded.
    AnimFragment(Vec<DefinitionV3>),

    /// It stores a `Vec<Definition>` with the definitions for each version of AnomTable files decoded.
    AnimTable(Vec<DefinitionV3>),

    /// It stores the name of the table, and a `Vec<Definition>` with the definitions for each version of that table decoded.
    DB(String, Vec<DefinitionV3>),

    /// It stores a `Vec<Definition>` to decode the dependencies of a PackFile.
    DepManager(Vec<DefinitionV3>),

    /// It stores a `Vec<Definition>` with the definitions for each version of Loc files decoded (currently, only version `1`).
    Loc(Vec<DefinitionV3>),

    /// It stores a `Vec<Definition>` with the definitions for each version of MatchedCombat files decoded.
    MatchedCombat(Vec<DefinitionV3>),
}

/// This struct contains all the data needed to decode a specific version of a versioned PackedFile.
#[derive(Clone, PartialEq, Eq, PartialOrd, Debug, Default, Serialize, Deserialize)]
pub struct DefinitionV3 {

    /// The version of the PackedFile the definition is for. These versions are:
    /// - `-1`: for fake `Definition`, used for dependency resolving stuff.
    /// - `0`: for unversioned PackedFiles.
    /// - `1+`: for versioned PackedFiles.
    version: i32,

    /// This is a collection of all `Field`s the PackedFile uses, in the order it uses them.
    fields: Vec<FieldV3>,

    /// This is a list of all the fields from this definition that are moved to a Loc PackedFile on exporting.
    localised_fields: Vec<FieldV3>,
}

/// This struct holds all the relevant data do properly decode a field from a versioned PackedFile.
#[derive(Clone, PartialEq, Eq, PartialOrd, Debug, Serialize, Deserialize)]
pub struct FieldV3 {

    /// Name of the field. Should contain no spaces, using `_` instead.
    pub name: String,

    /// Type of the field.
    pub field_type: FieldTypeV3,

    /// `True` if the field is a `Key` field of a table. `False` otherwise.
    pub is_key: bool,

    /// The default value of the field.
    pub default_value: Option<String>,

    /// The max allowed length for the data in the field.
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

    /// Variable to tell if this column is a bitwise column (spanned accross multiple columns) or not. Only applicable to numeric fields.
    pub is_bitwise: i32,

    /// Variable that specifies the "Enum" values for each value in this field.
    pub enum_values: BTreeMap<i32, String>
}

/// This enum defines every type of field the lib can encode/decode.
#[derive(Clone, PartialEq, Eq, PartialOrd, Debug, Serialize, Deserialize)]
pub enum FieldTypeV3 {
    Boolean,
    F32,
    I16,
    I32,
    I64,
    StringU8,
    StringU16,
    OptionalStringU8,
    OptionalStringU16,
    SequenceU16(DefinitionV3),
    SequenceU32(DefinitionV3)
}

//---------------------------------------------------------------------------//
//                       Enum & Structs Implementations
//---------------------------------------------------------------------------//

/// Implementation of `SchemaV3`.
impl SchemaV3 {

    /// This function adds a new `VersionedFile` to the schema. This checks if the provided `VersionedFile`
    /// already exists, and replace it if necessary.
    pub fn add_versioned_file(&mut self, versioned_file: &VersionedFileV3) {
        match self.versioned_files.par_iter().position_any(|x| x.conflict(versioned_file)) {
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

        // Make sure the path exists to avoid problems with updating schemas.
        DirBuilder::new().recursive(true).create(&file_path)?;

        file_path.push(schema_file);
        let mut file = File::create(&file_path)?;
        let config = PrettyConfig::default();

        self.sort();

        // Make sure all definitions are properly sorted.
        self.versioned_files.iter_mut().for_each(|x| {
            match x {
                VersionedFileV3::AnimFragment(ref mut versions) |
                VersionedFileV3::AnimTable(ref mut versions) |
                VersionedFileV3::DB(_, ref mut versions) |
                VersionedFileV3::DepManager(ref mut versions) |
                VersionedFileV3::Loc(ref mut versions) |
                VersionedFileV3::MatchedCombat(ref mut versions) => {
                    // Sort them by version number.
                    versions.sort_by(|a, b| b.get_version().cmp(&a.get_version()));
                }
            }
        });
        file.write_all(to_string_pretty(&self, config)?.as_bytes())?;
        Ok(())
    }

    /// This function sorts a `Schema` alphabetically, so the schema diffs are more or less clean.
    pub fn sort(&mut self) {
        self.versioned_files.sort_by(|a, b| {
            match a {
                VersionedFileV3::AnimFragment(_) => {
                    match b {
                        VersionedFileV3::AnimFragment(_) => Ordering::Equal,
                        _ => Ordering::Less,
                    }
                }
                VersionedFileV3::AnimTable(_) => {
                    match b {
                        VersionedFileV3::AnimFragment(_) => Ordering::Greater,
                        VersionedFileV3::AnimTable(_) => Ordering::Equal,
                        _ => Ordering::Less,
                    }
                }
                VersionedFileV3::DB(table_name_a, _) => {
                    match b {
                        VersionedFileV3::AnimFragment(_) => Ordering::Greater,
                        VersionedFileV3::AnimTable(_) => Ordering::Greater,
                        VersionedFileV3::DB(table_name_b, _) => table_name_a.cmp(table_name_b),
                        _ => Ordering::Less,
                    }
                }
                VersionedFileV3::DepManager(_) => {
                    match b {
                        VersionedFileV3::AnimFragment(_) => Ordering::Greater,
                        VersionedFileV3::AnimTable(_) => Ordering::Greater,
                        VersionedFileV3::DB(_,_) => Ordering::Greater,
                        VersionedFileV3::DepManager(_) => Ordering::Equal,
                        VersionedFileV3::Loc(_) => Ordering::Less,
                        VersionedFileV3::MatchedCombat(_) => Ordering::Less,
                    }
                }
                VersionedFileV3::Loc(_) => {
                    match b {
                        VersionedFileV3::Loc(_) => Ordering::Equal,
                        VersionedFileV3::MatchedCombat(_) => Ordering::Less,
                        _ => Ordering::Greater,
                    }
                }
                VersionedFileV3::MatchedCombat(_) => {
                    match b {
                        VersionedFileV3::MatchedCombat(_) => Ordering::Equal,
                        _ => Ordering::Greater,
                    }
                }
            }
        });
    }

    pub fn update() {
        println!("Importing schemas from V3 to V4");
        let mut legacy_schemas = SUPPORTED_GAMES.get_games().iter().map(|y| (y.get_game_key_name(), Self::load(y.get_schema_name()))).filter_map(|(x, y)| if let Ok(y) = y { Some((x, From::from(&y))) } else { None }).collect::<BTreeMap<String, SchemaV4>>();
        println!("Amount of SchemasV3: {:?}", legacy_schemas.len());
        legacy_schemas.par_iter_mut().for_each(|(game, legacy_schema)| {
            if let Some(file_name) = SUPPORTED_GAMES.get_games().iter().filter_map(|y| if &y.get_game_key_name() == game { Some(y.get_schema_name()) } else { None }).find(|_| true) {
                if legacy_schema.save(file_name).is_ok() {
                    println!("SchemaV3 for game {} updated to SchemaV4.", game);
                }
            }
        });
    }
}

/// Implementation of `VersionedFile`.
impl VersionedFileV3 {

    /// This function returns true if the provided `VersionedFile` is an AnimFragment Definition. Otherwise, it returns false.
    pub fn is_anim_fragment(&self) -> bool {
        matches!(*self, VersionedFileV3::AnimFragment(_))
    }

    /// This function returns true if the provided `VersionedFile` is an AnimTable Definition. Otherwise, it returns false.
    pub fn is_animtable(&self) -> bool {
        matches!(*self, VersionedFileV3::AnimTable(_))
    }

    /// This function returns true if the provided `VersionedFile` is a DB Definition. Otherwise, it returns false.
    pub fn is_db(&self) -> bool {
        matches!(*self, VersionedFileV3::DB(_,_))
    }

    /// This function returns true if the provided `VersionedFile` is a Dependency Manager Definition. Otherwise, it returns false.
    pub fn is_dep_manager(&self) -> bool {
        matches!(*self, VersionedFileV3::DepManager(_))
    }

    /// This function returns true if the provided `VersionedFile` is a Loc Definition. Otherwise, it returns false.
    pub fn is_loc(&self) -> bool {
        matches!(*self, VersionedFileV3::Loc(_))
    }

    /// This function returns true if the provided `VersionedFile` is an MatchedCombat Definition. Otherwise, it returns false.
    pub fn is_matched_combat(&self) -> bool {
        matches!(*self, VersionedFileV3::MatchedCombat(_))
    }

    /// This function returns true if both `VersionFile` are conflicting (they're the same, but their definitions may be different).
    pub fn conflict(&self, secondary: &VersionedFileV3) -> bool {
        match &self {
            VersionedFileV3::AnimFragment(_) => secondary.is_anim_fragment(),
            VersionedFileV3::AnimTable(_) => secondary.is_animtable(),
            VersionedFileV3::DB(table_name,_) => match &secondary {
                VersionedFileV3::DB(secondary_table_name, _) => table_name == secondary_table_name,
                _ => false,
            },
            VersionedFileV3::Loc(_) => secondary.is_loc(),
            VersionedFileV3::DepManager(_) => secondary.is_dep_manager(),
            VersionedFileV3::MatchedCombat(_) => secondary.is_matched_combat(),
        }
    }

    /// This function returns a reference to a specific version of a definition, if it finds it.
    pub fn get_version(&self, version: i32) -> Result<&DefinitionV3> {
        match &self {
            VersionedFileV3::AnimFragment(versions) |
            VersionedFileV3::AnimTable(versions) |
            VersionedFileV3::DB(_, versions) |
            VersionedFileV3::DepManager(versions) |
            VersionedFileV3::Loc(versions) |
            VersionedFileV3::MatchedCombat(versions) => versions.iter().find(|x| x.version == version).ok_or_else(|| From::from(ErrorKind::SchemaDefinitionNotFound)),
        }
    }
}

/// Implementation of `Definition`.
impl DefinitionV3 {

    /// This function creates a new empty `Definition` for the version provided.
    pub fn new(version: i32) -> DefinitionV3 {
        DefinitionV3 {
            version,
            localised_fields: vec![],
            fields: vec![],
        }
    }

    /// This function returns the version of the provided definition.
    pub fn get_version(&self) -> i32 {
        self.version
    }

    /// This function returns a mutable reference to the list of fields in the definition.
    pub fn get_ref_mut_fields(&mut self) -> &mut Vec<FieldV3> {
        &mut self.fields
    }

    /// This function returns the localised fields of the provided definition
    pub fn get_ref_mut_localised_fields(&mut self) -> &mut Vec<FieldV3> {
        &mut self.localised_fields
    }

}

/// Default implementation of `FieldType`.
impl Default for FieldV3 {
    fn default() -> Self {
        Self {
            name: String::from("new_field"),
            field_type: FieldTypeV3::StringU8,
            is_key: false,
            default_value: None,
            max_length: 0,
            is_filename: false,
            filename_relative_path: None,
            is_reference: None,
            lookup: None,
            description: String::from(""),
            ca_order: -1,
            is_bitwise: 0,
            enum_values: BTreeMap::new(),
        }
    }
}

/// Default implementation of `SchemaV3`.
impl Default for SchemaV3 {
    fn default() -> Self {
        Self {
            version: 3,
            versioned_files: vec![]
        }
    }
}


impl From<&SchemaV3> for SchemaV4 {
    fn from(legacy_schema: &SchemaV3) -> Self {
        let mut schema = Self::default();
        legacy_schema.versioned_files.iter().map(From::from).for_each(|x| schema.add_versioned_file(&x));
        schema
    }
}

impl From<&VersionedFileV3> for VersionedFileV4 {
    fn from(legacy_table_definitions: &VersionedFileV3) -> Self {
        match legacy_table_definitions {
            VersionedFileV3::AnimFragment(definitions) => Self::AnimFragment(definitions.iter().map(From::from).collect()),
            VersionedFileV3::AnimTable(definitions) => Self::AnimTable(definitions.iter().map(From::from).collect()),
            VersionedFileV3::DB(name, definitions) => Self::DB(name.to_string(), definitions.iter().map(From::from).collect()),
            VersionedFileV3::DepManager(definitions) => Self::DepManager(definitions.iter().map(From::from).collect()),
            VersionedFileV3::Loc(definitions) => Self::Loc(definitions.iter().map(From::from).collect()),
            VersionedFileV3::MatchedCombat(definitions) => Self::MatchedCombat(definitions.iter().map(From::from).collect()),
        }
    }
}

impl From<&DefinitionV3> for DefinitionV4 {
    fn from(legacy_table_definition: &DefinitionV3) -> Self {
        let mut definition = Self::new(legacy_table_definition.version);
        legacy_table_definition.fields.iter().map(From::from).for_each(|x| definition.get_ref_mut_fields().push(x));
        legacy_table_definition.localised_fields.iter().map(From::from).for_each(|x| definition.get_ref_mut_localised_fields().push(x));
        definition
    }
}

impl From<&FieldV3> for FieldV4 {
    fn from(legacy_field: &FieldV3) -> Self {
        Self {
            name: legacy_field.name.to_owned(),
            field_type: From::from(&legacy_field.field_type),
            is_key: legacy_field.is_key,
            default_value: legacy_field.default_value.clone(),
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

impl From<&FieldTypeV3> for FieldTypeV4 {
    fn from(legacy_field_type: &FieldTypeV3) -> Self {
        match legacy_field_type {
            FieldTypeV3::Boolean => Self::Boolean,
            FieldTypeV3::F32 => Self::F32,
            FieldTypeV3::I16 => Self::I16,
            FieldTypeV3::I32 => Self::I32,
            FieldTypeV3::I64 => Self::I64,
            FieldTypeV3::StringU8 => Self::StringU8,
            FieldTypeV3::StringU16 => Self::StringU16,
            FieldTypeV3::OptionalStringU8 => Self::OptionalStringU8,
            FieldTypeV3::OptionalStringU16 => Self::OptionalStringU16,
            FieldTypeV3::SequenceU16(sequence) => Self::SequenceU16(Box::new(From::from(&*sequence))),
            FieldTypeV3::SequenceU32(sequence) => Self::SequenceU32(Box::new(From::from(&*sequence))),
        }
    }
}
