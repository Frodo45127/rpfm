//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
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
```ignore
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
use ron::de::from_bytes;
use serde_derive::{Serialize, Deserialize};

use std::collections::{BTreeMap, HashMap};
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

use crate::error::Result;
use crate::schema::Schema as SchemaV5;
use crate::schema::Definition as DefinitionV5;
use crate::schema::FieldType as FieldTypeV5;
use crate::schema::Field as FieldV5;

//---------------------------------------------------------------------------//
//                              Enum & Structs
//---------------------------------------------------------------------------//

/// This struct represents a Schema File in memory, ready to be used to decode versioned PackedFiles.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct SchemaV4 {

    /// It stores the structural version of the Schema.
    version: u16,

    /// It stores the versioned files inside the Schema.
    versioned_files: Vec<VersionedFileV4>
}

/// This enum defines all types of versioned files that the schema system supports.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum VersionedFileV4 {

    /// It stores a `Vec<Definition>` with the definitions for each version of AnimFragment files decoded.
    AnimFragment(Vec<DefinitionV4>),

    /// It stores a `Vec<Definition>` with the definitions for each version of AnomTable files decoded.
    AnimTable(Vec<DefinitionV4>),

    /// It stores the name of the table, and a `Vec<Definition>` with the definitions for each version of that table decoded.
    DB(String, Vec<DefinitionV4>),

    /// It stores a `Vec<Definition>` to decode the dependencies of a PackFile.
    DepManager(Vec<DefinitionV4>),

    /// It stores a `Vec<Definition>` with the definitions for each version of Loc files decoded (currently, only version `1`).
    Loc(Vec<DefinitionV4>),

    /// It stores a `Vec<Definition>` with the definitions for each version of MatchedCombat files decoded.
    MatchedCombat(Vec<DefinitionV4>),
}

/// This struct contains all the data needed to decode a specific version of a versioned PackedFile.
#[derive(Clone, PartialEq, Eq, PartialOrd, Debug, Default, Serialize, Deserialize)]
pub struct DefinitionV4 {

    /// The version of the PackedFile the definition is for. These versions are:
    /// - `-1`: for fake `Definition`, used for dependency resolving stuff.
    /// - `0`: for unversioned PackedFiles.
    /// - `1+`: for versioned PackedFiles.
    version: i32,

    /// This is a collection of all `Field`s the PackedFile uses, in the order it uses them.
    fields: Vec<FieldV4>,

    /// This is a list of all the fields from this definition that are moved to a Loc PackedFile on exporting.
    localised_fields: Vec<FieldV4>,
}

/// This struct holds all the relevant data do properly decode a field from a versioned PackedFile.
#[derive(Clone, PartialEq, Eq, PartialOrd, Debug, Serialize, Deserialize)]
pub struct FieldV4 {

    /// Name of the field. Should contain no spaces, using `_` instead.
    pub name: String,

    /// Type of the field.
    pub field_type: FieldTypeV4,

    /// `True` if the field is a `Key` field of a table. `False` otherwise.
    pub is_key: bool,

    /// The default value of the field.
    pub default_value: Option<String>,

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
    pub enum_values: BTreeMap<i32, String>,

    /// If the field is part of a 3-part RGB column set, and which one (R, G or B) it is.
    pub is_part_of_colour: Option<u8>,
}

/// This enum defines every type of field the lib can encode/decode.
#[derive(Clone, PartialEq, Eq, PartialOrd, Debug, Serialize, Deserialize)]
pub enum FieldTypeV4 {
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
    SequenceU16(Box<DefinitionV4>),
    SequenceU32(Box<DefinitionV4>)
}

/// This struct represents a bunch of Schema Patches in memory.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, Default)]
pub struct SchemaPatches {

    /// It stores the patches split by games.
    patches: HashMap<String, SchemaPatch>
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, Default)]
pub struct SchemaPatch{

    /// It stores a list of per-table, per-column patches.
    tables: HashMap<String, HashMap<String, HashMap<String, String>>>,
}

//---------------------------------------------------------------------------//
//                       Enum & Structs Implementations
//---------------------------------------------------------------------------//

/// Implementation of `SchemaV4`.
impl SchemaV4 {

    /// This function loads a `Schema` to memory from a file in the `schemas/` folder.
    pub fn load(path: &Path) -> Result<Self> {
        let mut file = BufReader::new(File::open(path)?);
        let mut data = Vec::with_capacity(file.get_ref().metadata()?.len() as usize);
        file.read_to_end(&mut data)?;
        from_bytes(&data).map_err(From::from)
    }

    /// This function tries to update the Schema at the provided Path to a more recent format.
    pub fn update(schema_path: &Path, patches_path: &Path, game_name: &str) -> Result<()> {
        let schema_legacy = Self::load(schema_path)?;
        let mut schema = SchemaV5::from(&schema_legacy);

        // Fix for empty dependencies, again.
        schema.definitions.par_iter_mut().for_each(|(table_name, definitions)| {
            definitions.iter_mut().for_each(|definition| {
                definition.fields.iter_mut().for_each(|field| {
                    if let Some((ref_table, ref_column)) = field.is_reference(None) {
                        if ref_table.trim().is_empty() || ref_column.trim().is_empty() {
                            dbg!(&table_name);
                            dbg!(field.name());
                            field.is_reference = None;
                        }
                    }
                })
            })
        });

        let schema_patches = SchemaPatches::load(patches_path);
        if let Ok(schema_patches) = schema_patches {
            if let Some(patches) = schema_patches.patches.get(game_name) {
                schema.patches = patches.tables.clone();
            }
        }

        // Disable saving until 4.0 releases.
        schema.save(schema_path)?;
        Ok(())
    }
}

/// Implementation of `Definition`.
impl DefinitionV4 {

    /// This function creates a new empty `Definition` for the version provided.
    pub fn new(version: i32) -> DefinitionV4 {
        DefinitionV4 {
            version,
            localised_fields: vec![],
            fields: vec![],
        }
    }

    /// This function returns the version of the provided definition.
    pub fn version(&self) -> i32 {
        self.version
    }

    /// This function returns a mutable reference to the list of fields in the definition.
    pub fn fields_mut(&mut self) -> &mut Vec<FieldV4> {
        &mut self.fields
    }

    /// This function returns the localised fields of the provided definition
    pub fn localised_fields_mut(&mut self) -> &mut Vec<FieldV4> {
        &mut self.localised_fields
    }

}

/// Default implementation of `FieldType`.
impl Default for FieldV4 {
    fn default() -> Self {
        Self {
            name: String::from("new_field"),
            field_type: FieldTypeV4::StringU8,
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

/// Default implementation of `SchemaV4`.
impl Default for SchemaV4 {
    fn default() -> Self {
        Self {
            version: 3,
            versioned_files: vec![]
        }
    }
}


impl From<&SchemaV4> for SchemaV5 {
    fn from(legacy_schema: &SchemaV4) -> Self {
        let mut schema = Self::default();
        legacy_schema.versioned_files.iter()
            .filter_map(|versioned| if let VersionedFileV4::DB(name, definitions) = versioned { Some((name, definitions)) } else { None })
            .for_each(|(name, definitions)| {
                definitions.iter().for_each(|definition| {
                    schema.add_definition(name, &From::from(definition));
                })
            });
        schema
    }
}

impl From<&DefinitionV4> for DefinitionV5 {
    fn from(legacy_table_definition: &DefinitionV4) -> Self {
        let mut definition = Self::new(legacy_table_definition.version, None);

        let fields = legacy_table_definition.fields.iter().map(From::from).collect::<Vec<FieldV5>>();
        definition.set_fields(fields);

        let fields = legacy_table_definition.localised_fields.iter().map(From::from).collect::<Vec<FieldV5>>();
        definition.set_localised_fields(fields);

        definition
    }
}

impl From<&FieldV4> for FieldV5 {
    fn from(legacy_field: &FieldV4) -> Self {
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

impl From<&FieldTypeV4> for FieldTypeV5 {
    fn from(legacy_field_type: &FieldTypeV4) -> Self {
        match legacy_field_type {
            FieldTypeV4::Boolean => Self::Boolean,
            FieldTypeV4::I16 => Self::I16,
            FieldTypeV4::I32 => Self::I32,
            FieldTypeV4::I64 => Self::I64,
            FieldTypeV4::F32 => Self::F32,
            FieldTypeV4::F64 => Self::F64,
            FieldTypeV4::ColourRGB => Self::ColourRGB,
            FieldTypeV4::StringU8 => Self::StringU8,
            FieldTypeV4::StringU16 => Self::StringU16,
            FieldTypeV4::OptionalStringU8 => Self::OptionalStringU8,
            FieldTypeV4::OptionalStringU16 => Self::OptionalStringU16,
            FieldTypeV4::SequenceU16(sequence) => Self::SequenceU16(Box::new(From::from(&**sequence))),
            FieldTypeV4::SequenceU32(sequence) => Self::SequenceU32(Box::new(From::from(&**sequence))),
        }
    }
}

impl SchemaPatches {

    /// This function loads a `SchemaPatches` to memory from a file in the `schemas/` folder.
    pub fn load(file_path: &Path) -> Result<Self> {
        let mut file = BufReader::new(File::open(file_path)?);
        let mut data = Vec::with_capacity(file.get_ref().metadata()?.len() as usize);
        file.read_to_end(&mut data)?;
        from_bytes(&data).map_err(From::from)
    }
}
