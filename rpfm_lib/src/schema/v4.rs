//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Legacy schema version 4 support module.
//!
//! This module provides backward compatibility for loading and upgrading schema files
//! from the version 4 format to the current version 5 format. It contains the old type
//! definitions and conversion logic needed to migrate existing schema files.
//!
//! # Purpose
//!
//! When RPFM's schema format changes in backwards-incompatible ways, legacy modules
//! like this are kept to allow automatic migration of existing schema files. This ensures
//! users can seamlessly upgrade their schemas without manual intervention.
//!
//! # Migration Process
//!
//! The [`SchemaV4::update()`] function handles the migration:
//! 1. Load the old v4 schema
//! 2. Convert to the new v5 format using [`From`] implementations
//! 3. Load and merge any existing patches
//! 4. Save the upgraded schema

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

/// Legacy version 4 schema structure.
///
/// This struct represents the schema file format used before version 5.
/// It is kept for backward compatibility and migration purposes only.
///
/// # Differences from Version 5
///
/// - Used a `versioned_files` vec instead of a `definitions` hashmap
/// - Had different file type variants (AnimFragment, AnimTable, etc.)
/// - Patches were stored in separate files, not integrated into the schema
///
/// See [`Schema`](crate::schema::Schema) for the current format.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct SchemaV4 {

    /// The structural version (always 3 or 4 for this format).
    version: u16,

    /// List of versioned file definitions grouped by type.
    versioned_files: Vec<VersionedFileV4>
}

/// Legacy versioned file type enumeration.
///
/// In version 4 schemas, different file types had their own enum variants.
/// Version 5 simplified this to just use DB tables.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum VersionedFileV4 {

    /// AnimFragment file definitions (no longer used in v5).
    AnimFragment(Vec<DefinitionV4>),

    /// AnimTable file definitions (no longer used in v5).
    AnimTable(Vec<DefinitionV4>),

    /// Database table definitions.
    ///
    /// Format: `(table_name, definitions_for_each_version)`
    DB(String, Vec<DefinitionV4>),

    /// Dependency manager definitions (no longer used in v5).
    DepManager(Vec<DefinitionV4>),

    /// Localisation file definitions (no longer used in v5).
    Loc(Vec<DefinitionV4>),

    /// Matched combat file definitions (no longer used in v5).
    MatchedCombat(Vec<DefinitionV4>),
}

/// Legacy version 4 table definition.
///
/// Defines the structure of one version of a table in the v4 schema format.
/// Converted to [`Definition`](crate::schema::Definition) during migration.
#[derive(Clone, PartialEq, Eq, PartialOrd, Debug, Default, Serialize, Deserialize)]
pub struct DefinitionV4 {

    /// Version number of this definition.
    ///
    /// - `-1`: Fake definition for dependency resolution
    /// - `0`: Unversioned files
    /// - `1+`: Versioned files
    version: i32,

    /// List of fields in binary order.
    fields: Vec<FieldV4>,

    /// Fields extracted to LOC files.
    localised_fields: Vec<FieldV4>,
}

/// Legacy version 4 field definition.
///
/// Defines a single field in the v4 schema format. All fields are public for
/// easy conversion. Converted to [`Field`](crate::schema::Field) during migration.
#[derive(Clone, PartialEq, Eq, PartialOrd, Debug, Serialize, Deserialize)]
pub struct FieldV4 {

    /// Field name (must match Assembly Kit definition).
    pub name: String,

    /// Field data type.
    pub field_type: FieldTypeV4,

    /// Whether this field is a primary key.
    pub is_key: bool,

    /// Default value for new rows.
    pub default_value: Option<String>,

    /// Whether this field contains a filename.
    pub is_filename: bool,

    /// Relative path(s) where files can be found.
    pub filename_relative_path: Option<String>,

    /// Foreign key reference `(table, column)`.
    pub is_reference: Option<(String, String)>,

    /// Lookup columns from referenced table.
    pub lookup: Option<Vec<String>>,

    /// Human-readable description.
    pub description: String,

    /// Position in Assembly Kit (-1 if unknown).
    pub ca_order: i16,

    /// Number of boolean columns to expand into.
    pub is_bitwise: i32,

    /// Enum value mappings.
    pub enum_values: BTreeMap<i32, String>,

    /// RGB colour group index.
    pub is_part_of_colour: Option<u8>,
}

/// Legacy version 4 field type enumeration.
///
/// Defines the data types available in v4 schemas. Nearly identical to v5,
/// but missing the Optional integer types (OptionalI16, OptionalI32, OptionalI64).
#[derive(Clone, PartialEq, Eq, PartialOrd, Debug, Serialize, Deserialize)]
pub enum FieldTypeV4 {
    /// 1-byte boolean.
    Boolean,
    /// 32-bit float.
    F32,
    /// 64-bit float.
    F64,
    /// 16-bit signed integer.
    I16,
    /// 32-bit signed integer.
    I32,
    /// 64-bit signed integer.
    I64,
    /// RGB colour (hex string).
    ColourRGB,
    /// UTF-8 encoded string with [`u16`] length prefix.
    StringU8,
    /// UTF-16 encoded string with [`u16`] length prefix.
    StringU16,
    /// Optional UTF-8 encoded string.
    OptionalStringU8,
    /// Optional UTF-16 encoded string.
    OptionalStringU16,
    /// Array with u16 count.
    SequenceU16(Box<DefinitionV4>),
    /// Array with u32 count.
    SequenceU32(Box<DefinitionV4>)
}

/// Legacy version 4 patches container.
///
/// In v4, patches were stored separately from schemas and organized by game.
/// In v5, patches are integrated directly into the schema.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, Default)]
pub struct SchemaPatches {

    /// Patches organized by game name.
    patches: HashMap<String, SchemaPatch>
}

/// Legacy version 4 per-game patches.
///
/// Contains all table patches for a specific game.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, Default)]
pub struct SchemaPatch{

    /// Table patches in the format: `table_name -> column_name -> patch_key -> patch_value`.
    tables: HashMap<String, HashMap<String, HashMap<String, String>>>,
}

//---------------------------------------------------------------------------//
//                       Enum & Structs Implementations
//---------------------------------------------------------------------------//

/// Implementation of [`SchemaV4`].
impl SchemaV4 {

    /// Loads a v4 schema from a RON file.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the v4 schema file
    ///
    /// # Returns
    ///
    /// Returns the loaded v4 schema, or an error if loading fails.
    pub fn load(path: &Path) -> Result<Self> {
        let mut file = BufReader::new(File::open(path)?);
        let mut data = Vec::with_capacity(file.get_ref().metadata()?.len() as usize);
        file.read_to_end(&mut data)?;
        from_bytes(&data).map_err(From::from)
    }

    /// Upgrades a v4 schema file to the current v5 format.
    ///
    /// This function:
    /// 1. Loads the v4 schema
    /// 2. Converts it to v5 format
    /// 3. Loads and merges patches from the patches file
    /// 4. Cleans up invalid references
    /// 5. Saves the upgraded v5 schema
    ///
    /// # Arguments
    ///
    /// * `schema_path` - Path to the v4 schema file (will be overwritten with v5)
    /// * `patches_path` - Path to the v4 patches file
    /// * `game_name` - Name of the game to extract patches for
    ///
    /// # Returns
    ///
    /// Returns [`Ok`] if the upgrade succeeds, or an error otherwise.
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

        schema.save(schema_path)?;
        Ok(())
    }
}

/// Implementation of [`DefinitionV4`].
impl DefinitionV4 {

    /// Creates a new empty v4 definition.
    ///
    /// # Arguments
    ///
    /// * `version` - Version number for this definition
    ///
    /// # Returns
    ///
    /// Returns a new empty definition.
    pub fn new(version: i32) -> DefinitionV4 {
        DefinitionV4 {
            version,
            localised_fields: vec![],
            fields: vec![],
        }
    }

    /// Returns the version number.
    pub fn version(&self) -> i32 {
        self.version
    }

    /// Returns a mutable reference to the fields list.
    pub fn fields_mut(&mut self) -> &mut Vec<FieldV4> {
        &mut self.fields
    }

    /// Returns a mutable reference to the localised fields list.
    pub fn localised_fields_mut(&mut self) -> &mut Vec<FieldV4> {
        &mut self.localised_fields
    }

}

/// Default implementation for [`FieldV4`].
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

/// Default implementation for [`SchemaV4`].
impl Default for SchemaV4 {
    fn default() -> Self {
        Self {
            version: 3,
            versioned_files: vec![]
        }
    }
}


/// Converts a v4 schema to the current v5 format.
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

/// Converts a v4 definition to the current v5 format.
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

/// Converts a v4 field to the current v5 format.
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

/// Converts a v4 field type to the current v5 format.
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

/// Implementation of [`SchemaPatches`].
impl SchemaPatches {

    /// Loads v4 patches from a RON file.
    ///
    /// # Arguments
    ///
    /// * `file_path` - Path to the patches file
    ///
    /// # Returns
    ///
    /// Returns the loaded patches, or an error if loading fails.
    pub fn load(file_path: &Path) -> Result<Self> {
        let mut file = BufReader::new(File::open(file_path)?);
        let mut data = Vec::with_capacity(file.get_ref().metadata()?.len() as usize);
        file.read_to_end(&mut data)?;
        from_bytes(&data).map_err(From::from)
    }
}
