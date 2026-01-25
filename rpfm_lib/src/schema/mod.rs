//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Schema system for defining Total War file formats.
//!
//! This module provides the infrastructure for defining and managing schemas that describe the binary
//! structure of Total War game files, primarily database tables and localization files.
//!
//! # Overview
//!
//! A [`Schema`] contains [`Definition`]s that specify the exact binary layout of different file types.
//! Each table can have multiple definitions to support different versions across game patches. The schema
//! system also supports runtime patches to override field properties without modifying the base schema.
//!
//! # Key Components
//!
//! - [`Schema`]: The main container holding all table definitions and patches for a game
//! - [`Definition`]: Describes one version of a table's structure (fields, types, constraints)
//! - [`Field`]: Represents a single column in a table with its type and metadata
//! - [`FieldType`]: The data type of a field (integers, strings, booleans, sequences, etc.)
//! - [`DefinitionPatch`]: Runtime modifications to field properties
//!
//! # Schema Versioning
//!
//! - Each game has its own schema file (e.g., `warhammer_3.ron`)
//! - The schema format version (currently v5) is tracked separately from table versions
//! - Legacy schema formats (like v4) can be automatically upgraded via [`Schema::update()`]
//!
//! # Loading and Saving
//!
//! Schemas are typically stored in RON format but can also be exported to JSON:
//!
//! ```no_run
//! use rpfm_lib::schema::Schema;
//! use std::path::Path;
//!
//! // Load a schema
//! let schema_path = Path::new("schemas/warhammer_3.ron");
//! let schema = Schema::load(schema_path, None)?;
//!
//! // Access table definitions
//! if let Some(defs) = schema.definitions_by_table_name("units_tables") {
//!     for def in defs {
//!         println!("Version {}: {} fields", def.version(), def.fields().len());
//!     }
//! }
//! # Ok::<(), rpfm_lib::error::RLibError>(())
//! ```
//!
//! # Patches
//!
//! Patches allow modifying field properties at runtime without changing the schema:
//!
//! ```no_run
//! use rpfm_lib::schema::Schema;
//! use std::path::Path;
//!
//! let schema = Schema::load(Path::new("schema.ron"), Some(Path::new("patches.ron")))?;
//!
//! // Check if a field has a patched value
//! if let Some(value) = schema.patch_value("units_tables", "key", "is_key") {
//!     println!("Patched is_key value: {}", value);
//! }
//! # Ok::<(), rpfm_lib::error::RLibError>(())
//! ```
//!
//! # Schema Repository
//!
//! Schemas are maintained in a separate Git repository and can be updated independently from RPFM itself.
//! The repository URL and branch are defined as constants in this module.

use getset::*;
use itertools::Itertools;
use rayon::prelude::*;
use ron::de::{from_bytes, from_str};
use ron::ser::{to_string_pretty, PrettyConfig};
use serde::{Serialize as SerdeSerialize, Serializer};
use serde_derive::{Serialize, Deserialize};

use std::cmp::Ordering;
use std::collections::{BTreeMap, HashMap};
use std::{fmt, fmt::Display};
use std::fs::{DirBuilder, File};
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::Path;

#[cfg(feature = "integration_assembly_kit")]use crate::integrations::assembly_kit::localisable_fields::RawLocalisableField;
#[cfg(feature = "integration_assembly_kit")]use crate::integrations::assembly_kit::table_definition::RawDefinition;
#[cfg(feature = "integration_assembly_kit")]use crate::integrations::assembly_kit::table_definition::RawField;
#[cfg(feature = "integration_log")] use crate::integrations::log::*;
#[cfg(feature = "integration_sqlite")] use rusqlite::types::Type;

use crate::error::Result;
use crate::files::table::DecodedData;
use crate::games::supported_games::SupportedGames;

// Legacy Schemas, to keep backwards compatibility during updates.
pub(crate) mod v4;

/// Name of the folder containing all the schemas.
///
/// This folder is located within the application's config directory and stores schema files
/// for each supported Total War game.
pub const SCHEMA_FOLDER: &str = "schemas";

/// URL of the remote Git repository containing the schema files.
///
/// This repository is used to fetch and update schema definitions for all supported games.
pub const SCHEMA_REPO: &str = "https://github.com/Frodo45127/rpfm-schemas";

/// Name of the Git remote to use when fetching schemas.
pub const SCHEMA_REMOTE: &str = "origin";

/// Name of the Git branch to use when fetching schemas.
pub const SCHEMA_BRANCH: &str = "master";

/// Current structural version of the Schema, for compatibility purposes.
///
/// This version number is incremented when the schema format itself changes
/// in a backwards-incompatible way.
const CURRENT_STRUCTURAL_VERSION: u16 = 5;

/// Invalid version marker for internal use.
///
/// This value is used for temporary or fake [`Definition`] instances that don't
/// represent actual file versions.
const INVALID_VERSION: i32 = -100;

/// Name for unnamed colour groups.
///
/// When RGB colour fields are merged but have no common prefix, this name is used
/// as the base name for the combined field.
pub const MERGE_COLOUR_NO_NAME: &str = "Unnamed Colour Group";

/// Suffix for merged colour field names.
///
/// This string is appended to the base name when creating a merged RGB colour field.
/// For example, `banner_colour` fields would become `banner_colour_hex`.
pub const MERGE_COLOUR_POST: &str = "_hex";

/// Fields that can be ignored in missing field checks.
///
/// These fields are legacy fields from older Assembly Kit versions that are not
/// actually used by the games and can be safely ignored during schema updates.
const IGNORABLE_FIELDS: [&str; 4] = ["s_ColLineage", "s_Generation", "s_GUID", "s_Lineage"];

//---------------------------------------------------------------------------//
//                              Enum & Structs
//---------------------------------------------------------------------------//

/// This type defines patches for specific table definitions, in a ColumnName -> [key -> value] format.
///
/// Patches allow runtime modification of schema fields without changing the base schema files.
/// They are stored separately and applied when loading definitions.
///
/// # Structure
///
/// The outer [`HashMap`] maps column names to their patches. The inner [`HashMap`] maps patch keys
/// to their values. For table-wide patches (not specific to a column), use the special column name `"-1"`.
///
/// # Example Patch Keys
///
/// - `"is_key"`: Override whether a field is a key field
/// - `"default_value"`: Override the default value
/// - `"is_filename"`: Override whether a field is a filename
/// - `"filename_relative_path"`: Override the relative filename path
/// - `"is_reference"`: Override reference information
/// - `"lookup"`: Override lookup columns
/// - `"lookup_hardcoded"`: Add hardcoded lookup values
/// - `"description"`: Override the field description
/// - `"not_empty"`: Mark the field as "cannot be empty"
/// - `"unused"`: Mark a field as unused
pub type DefinitionPatch = HashMap<String, HashMap<String, String>>;

/// Represents a complete schema file containing table definitions for a Total War game.
///
/// A [`Schema`] stores the structural definitions for all database tables in a Total War game.
/// Each table can have multiple [`Definition`] versions, allowing the schema to support
/// different versions of the same table across game patches.
///
/// # Structure
///
/// - `version`: The structural version of the schema format itself (currently 5)
/// - `definitions`: A map of table names to their version history
/// - `patches`: Runtime modifications to field properties
///
/// # Usage
///
/// ```no_run
/// use rpfm_lib::schema::Schema;
/// use std::path::Path;
///
/// let schema_path = Path::new("schemas/warhammer_3.ron");
/// let schema = Schema::load(schema_path, None)?;
///
/// // Get definitions for a specific table
/// if let Some(definitions) = schema.definitions_by_table_name("units_tables") {
///     println!("Found {} versions of units_tables", definitions.len());
/// }
/// # Ok::<(), rpfm_lib::error::RLibError>(())
/// ```
#[derive(Clone, PartialEq, Eq, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct Schema {

    /// The structural version of the schema format.
    ///
    /// This is incremented when the schema format itself changes in backwards-incompatible ways.
    version: u16,

    /// Map of table names to their version definitions.
    ///
    /// Each table can have multiple versions, stored as a [`Vec`] of [`Definition`] instances.
    /// Serialization orders this map alphabetically for consistent output.
    #[serde(serialize_with = "ordered_map_definitions")]
    definitions: HashMap<String, Vec<Definition>>,

    /// Map of table names to their patches.
    ///
    /// Patches allow runtime modification of field properties without changing the base schema.
    /// See [`DefinitionPatch`] for the patch structure.
    #[serde(serialize_with = "ordered_map_patches")]
    patches: HashMap<String, DefinitionPatch>,
}

/// Defines the structure of a specific version of a database table.
///
/// A [`Definition`] specifies the exact binary layout and field properties for one version
/// of a table. Tables can have multiple definitions in a schema to support different versions
/// across game patches.
///
/// # Version Numbers
///
/// - `-1`: Fake definition used internally for dependency resolution
/// - `0`: Unversioned files (tables without version markers in their binary format)
/// - `1+`: Versioned files with explicit version numbers
///
/// # Fields Processing
///
/// The raw [`fields`] list may undergo processing when accessed via [`fields_processed()`]:
/// - Bitwise fields are expanded into multiple boolean fields
/// - Enum fields are converted to string fields
/// - RGB colour triplets are merged into single ColourRGB fields
///
/// Unless you have a specific reason to do so, it is recommended to use [`fields_processed()`] instead of [`fields`].
///
/// # Localisation
///
/// Some tables have fields that are moved to separate LOC files during export:
/// - [`localised_fields`] lists these fields
/// - [`localised_key_order`] defines the key field order for LOC keys
///
/// [`fields_processed()`]: Definition::fields_processed
/// [`fields`]: Definition::fields
/// [`localised_fields`]: Definition::localised_fields
/// [`localised_key_order`]: Definition::localised_key_order
#[derive(Clone, PartialEq, Eq, Debug, Default, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct Definition {

    /// The version number of this table definition.
    ///
    /// See type-level documentation for version number meanings.
    version: i32,

    /// List of fields in the order they appear in the binary format.
    ///
    /// This is the raw field list. For the processed version (with bitwise expansion,
    /// enum conversion, etc.), use [`fields_processed()`].
    ///
    /// [`fields_processed()`]: Definition::fields_processed
    fields: Vec<Field>,

    /// Fields that are extracted to LOC files during export.
    ///
    /// These fields contain localisable text that gets separated from the main table data
    /// when exporting said table to binary format.
    localised_fields: Vec<Field>,

    /// Order of key fields when constructing localisation keys.
    ///
    /// This specifies the order in which key fields should be concatenated when
    /// creating LOC entry keys. Only applies to processed fields.
    localised_key_order: Vec<u32>,

    /// Runtime patches applied to this definition.
    ///
    /// These are loaded from the schema's patch set and applied when retrieving
    /// the definition. Not serialized - they come from the schema's patches field.
    #[serde(skip)]
    patches: DefinitionPatch
}

/// Defines a single field within a table definition.
///
/// A [`Field`] describes one column in a database table, including its data type, constraints,
/// and metadata. Fields can be modified at runtime via schema patches.
///
/// # Field Types
///
/// See [`FieldType`] for the supported data types (integers, strings, sequences, etc.).
///
/// # Field Attributes and Constraints
///
/// - **Key Fields**: When `is_key` is true, the field is part of the table's primary key
/// - **References**: Fields can reference columns in other tables for foreign key relationships
/// - **Lookups**: Additional columns from referenced tables to display in the UI
/// - **Filenames**: Fields that contain file paths within the game's VFS
/// - **Bitwise**: Numeric fields that should be split into multiple boolean columns
/// - **Enums**: Numeric fields with named values
/// - **Colours**: Fields that are part of an RGB triplet
///
/// # Patching
///
/// Most field properties can be overridden via schema patches. Use the accessor methods
/// (e.g., [`is_key()`], [`default_value()`]) rather than direct field access to ensure
/// patches are applied.
///
/// [`is_key()`]: Field::is_key
/// [`default_value()`]: Field::default_value
#[derive(Clone, PartialEq, Eq, Debug, Setters, Serialize, Deserialize)]
#[getset(set = "pub")]
pub struct Field {

    /// Name of the field.
    ///
    /// Must match the field name from the Assembly Kit table definition (usually snake_case, but not always).
    name: String,

    /// Data type of the field.
    ///
    /// Determines how the field's binary data is interpreted.
    field_type: FieldType,

    /// Whether this field is part of the table's primary key.
    ///
    /// Can be overridden via patches. Use [`is_key()`] to get the patched value.
    ///
    /// [`is_key()`]: Field::is_key
    is_key: bool,

    /// Default value for this field when creating new rows.
    ///
    /// Can be overridden via patches. Use [`default_value()`] to get the patched value.
    ///
    /// [`default_value()`]: Field::default_value
    default_value: Option<String>,

    /// Whether this field contains a filename/path.
    ///
    /// Can be overridden via patches. Use [`is_filename()`] to get the patched value.
    ///
    /// [`is_filename()`]: Field::is_filename
    is_filename: bool,

    /// Semicolon-separated list of relative paths where files for this field can be found.
    ///
    /// Only applicable when `is_filename` is true. Can be overridden via patches.
    /// Use [`filename_relative_path()`] to get the parsed, patched value.
    ///
    /// [`filename_relative_path()`]: Field::filename_relative_path
    filename_relative_path: Option<String>,

    /// Foreign key reference to another table.
    ///
    /// Format: `Some((table_name, column_name))` where `table_name` doesn't include
    /// the `_tables` suffix. Can be overridden via patches.
    /// Use [`is_reference()`] to get the patched value.
    ///
    /// [`is_reference()`]: Field::is_reference
    is_reference: Option<(String, String)>,

    /// Additional columns from the referenced table to show in lookups.
    ///
    /// Only applicable when `is_reference` is Some. Can be overridden via patches.
    /// Use [`lookup()`] to get the patched value.
    ///
    /// [`lookup()`]: Field::lookup
    lookup: Option<Vec<String>>,

    /// Human-readable description of the field's purpose.
    ///
    /// Can be overridden via patches. Use [`description()`] to get the patched value.
    ///
    /// [`description()`]: Field::description
    description: String,

    /// Visual position in CA's Assembly Kit table editor.
    ///
    /// `-1` means the position is unknown. This is used to maintain column order
    /// consistency with the Assembly Kit.
    ca_order: i16,

    /// Number of boolean columns this field should be split into.
    ///
    /// Only applicable to numeric fields. A value > 1 means the field should be
    /// expanded into that many boolean columns when processed.
    is_bitwise: i32,

    /// Named values for this field when treated as an enum.
    ///
    /// Maps integer values to their string names. When non-empty, the field
    /// is treated as a string enum in processed fields.
    ///
    /// NOTE: When possible, prefer using lookups instead of enum_values.
    enum_values: BTreeMap<i32, String>,

    /// Index of the RGB colour group this field belongs to.
    ///
    /// When set, this field is part of a 3-field RGB triplet that should be
    /// merged into a single ColourRGB field when processed.
    is_part_of_colour: Option<u8>,

    /// Whether this field is unused by the game.
    ///
    /// Not serialized - determined via patches at runtime.
    /// Use [`unused()`] to get the patched value.
    ///
    /// [`unused()`]: Field::unused
    #[serde(skip_serializing, skip_deserializing)]
    unused: bool,
}

/// Supported data types for table fields.
///
/// This enum defines all field types that can be encoded/decoded from Total War database tables.
/// Each variant corresponds to a specific binary representation in the game files.
///
/// # Basic Types
///
/// - **Boolean**: 1-byte boolean value
/// - **Integers**: Signed integers of various sizes (I16, I32, I64)
/// - **Floats**: Floating-point numbers (F32, F64)
/// - **Strings**: Length-prefixed strings with [`u8`] or [`u16`] length markers
///
/// # Optional Types
///
/// Optional types use a 1-byte flag followed by the value if present:
/// - **OptionalI16**, **OptionalI32**, **OptionalI64**: Optional integers
/// - **OptionalStringU8**, **OptionalStringU16**: Optional strings
///
/// # Complex Types
///
/// - **ColourRGB**: 6-character hexadecimal RGB colour (e.g., "FF0000" for red)
/// - **SequenceU16**, **SequenceU32**: Arrays with [`u16`] or [`u32`] length prefix
///
/// # Sequences
///
/// Sequence types contain a nested [`Definition`] that describes the structure of each
/// array element. The length prefix determines how many elements follow.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum FieldType {
    /// 1-byte boolean value (0 = false, 1 = true).
    Boolean,

    /// 32-bit floating-point number.
    F32,

    /// 64-bit floating-point number.
    F64,

    /// 16-bit signed integer.
    I16,

    /// 32-bit signed integer.
    I32,

    /// 64-bit signed integer.
    I64,

    /// RGB colour as a 6-character hexadecimal string (e.g., "FF0000").
    ColourRGB,

    /// UTF-8 encoded string with [`u16`] length prefix (max 65535 bytes).
    StringU8,

    /// UTF-16 encoded string with [`u16`] length prefix (max 65535 characters).
    StringU16,

    /// Optional 16-bit signed integer (1-byte flag + value if present).
    OptionalI16,

    /// Optional 32-bit signed integer (1-byte flag + value if present).
    OptionalI32,

    /// Optional 64-bit signed integer (1-byte flag + value if present).
    OptionalI64,

    /// Optional UTF-8 encoded string (1-byte flag + [`u16`] length prefix + string if present).
    OptionalStringU8,

    /// Optional UTF-16 encoded string (1-byte flag + [`u16`] length prefix + string if present).
    OptionalStringU16,

    /// Array with [`u16`] element count followed by elements matching the nested definition.
    SequenceU16(Box<Definition>),

    /// Array with [`u32`] element count followed by elements matching the nested definition.
    SequenceU32(Box<Definition>)
}

//---------------------------------------------------------------------------//
//                       Enum & Structs Implementations
//---------------------------------------------------------------------------//

/// Implementation of [`Schema`].
impl Schema {

    /// Saves patches to a local patches file, merging with existing patches.
    ///
    /// This function loads existing patches from the file, merges the provided patches with them,
    /// and writes the combined patch set back to the file in RON format.
    ///
    /// # Arguments
    ///
    /// * `patches` - The patches to add or update
    /// * `path` - Path to the local patches file (must exist)
    ///
    /// # Returns
    ///
    /// Returns [`Ok`] if successful, or an error if:
    /// - The file cannot be read or written
    /// - The file contains invalid patch data
    ///
    /// # Example
    ///
    /// ```no_run
    /// use std::collections::HashMap;
    /// use std::path::Path;
    /// use rpfm_lib::schema::{Schema, DefinitionPatch};
    ///
    /// let mut patches: HashMap<String, DefinitionPatch> = HashMap::new();
    /// // Add patches...
    ///
    /// Schema::save_patches(&patches, Path::new("my_patches.ron"))?;
    /// # Ok::<(), rpfm_lib::error::RLibError>(())
    /// ```
    pub fn save_patches(patches: &HashMap<String, DefinitionPatch>, path: &Path) -> Result<()> {
        let mut file = BufReader::new(File::open(path)?);
        let mut data = Vec::with_capacity(file.get_ref().metadata()?.len() as usize);
        file.read_to_end(&mut data)?;
        let mut local_patches: HashMap<String, DefinitionPatch> = from_bytes(&data)?;

        Self::add_patches_to_patch_set(&mut local_patches, patches);

        let mut file = BufWriter::new(File::create(path)?);
        let config = PrettyConfig::default();
        file.write_all(to_string_pretty(&local_patches, config)?.as_bytes())?;

        Ok(())
    }

    /// Removes all local patches for a specific table.
    ///
    /// This function loads the patches file, removes all patches for the specified table,
    /// and writes the updated patch set back to the file.
    ///
    /// # Arguments
    ///
    /// * `table_name` - Name of the table to remove patches for
    /// * `path` - Path to the local patches file
    ///
    /// # Returns
    ///
    /// Returns [`Ok`] if successful, even if no there were no patches to remove, or an error
    /// if file I/O fails.
    pub fn remove_patches_for_table(table_name: &str, path: &Path) -> Result<()> {
        let mut file = BufReader::new(File::open(path)?);
        let mut data = Vec::with_capacity(file.get_ref().metadata()?.len() as usize);
        file.read_to_end(&mut data)?;
        let mut local_patches: HashMap<String, DefinitionPatch> = from_bytes(&data)?;

        local_patches.remove(table_name);

        let mut file = BufWriter::new(File::create(path)?);
        let config = PrettyConfig::default();
        file.write_all(to_string_pretty(&local_patches, config)?.as_bytes())?;

        Ok(())
    }

    /// Removes all local patches for a specific field in a table.
    ///
    /// This function loads the patches file, removes all patches for the specified table's field,
    /// and writes the updated patch set back to the file. Other fields in the table are unaffected.
    ///
    /// # Arguments
    ///
    /// * `table_name` - Name of the table containing the field
    /// * `field_name` - Name of the field to remove patches for
    /// * `path` - Path to the local patches file
    ///
    /// # Returns
    ///
    /// Returns [`Ok`] if successful, even if no there were no patches to remove, or an error
    /// if file I/O fails.
    pub fn remove_patches_for_table_and_field(table_name: &str, field_name: &str, path: &Path) -> Result<()> {
        let mut file = BufReader::new(File::open(path)?);
        let mut data = Vec::with_capacity(file.get_ref().metadata()?.len() as usize);
        file.read_to_end(&mut data)?;
        let mut local_patches: HashMap<String, DefinitionPatch> = from_bytes(&data)?;

        if let Some(table_patches) = local_patches.get_mut(table_name) {
            table_patches.remove(field_name);
        }

        let mut file = BufWriter::new(File::create(path)?);
        let config = PrettyConfig::default();
        file.write_all(to_string_pretty(&local_patches, config)?.as_bytes())?;

        Ok(())
    }

    /// Retrieves a specific patch value for a table's column.
    ///
    /// # Arguments
    ///
    /// * `table_name` - Name of the table
    /// * `column_name` - Name of the column
    /// * `key` - Patch key (e.g., "is_key", "default_value")
    ///
    /// # Returns
    ///
    /// Returns the patch value if found, or [`None`] otherwise.
    pub fn patch_value(&self, table_name: &str, column_name: &str, key: &str) -> Option<&String> {
        self.patches.get(table_name)?.get(column_name)?.get(key)
    }

    /// Retrieves all patches for a specific table.
    ///
    /// # Arguments
    ///
    /// * `table_name` - Name of the table
    ///
    /// # Returns
    ///
    /// Returns the table's patches if found, or [`None`] otherwise.
    pub fn patches_for_table(&self, table_name: &str) -> Option<&DefinitionPatch> {
        self.patches.get(table_name)
    }

    /// Merges patches into an existing patch set.
    ///
    /// This function adds the provided patches to the patch set, merging them with any
    /// existing patches. If a patch already exists for a table/column/key combination,
    /// it will be extended with the new values.
    ///
    /// # Arguments
    ///
    /// * `patch_set` - The patch set to merge into (modified in place)
    /// * `patches` - The patches to add
    ///
    /// # Note
    ///
    /// After adding patches, you must re-retrieve any definitions you've already retrieved
    /// for the patches to take effect, as patches are applied when retrieving definitions.
    pub fn add_patches_to_patch_set(patch_set: &mut HashMap<String, DefinitionPatch>, patches: &HashMap<String, DefinitionPatch>) {
        patches.iter().for_each(|(table_name, column_patch)| {
            match patch_set.get_mut(table_name) {
                Some(column_patch_current) => {
                    column_patch.iter().for_each(|(column_name, patch)| {
                        match column_patch_current.get_mut(column_name) {
                            Some(patch_current) => patch_current.extend(patch.clone()),
                            None => {
                                column_patch_current.insert(column_name.to_owned(), patch.clone());
                            }
                        }
                    });
                }
                None => {
                    patch_set.insert(table_name.to_owned(), column_patch.clone());
                }
            }
        });
    }

    /// Adds or updates a table definition in the schema.
    ///
    /// If a definition with the same version already exists for this table, it will be replaced.
    /// Otherwise, the definition is added to the table's version list.
    ///
    /// # Arguments
    ///
    /// * `table_name` - Name of the table
    /// * `definition` - The definition to add or update
    pub fn add_definition(&mut self, table_name: &str, definition: &Definition) {
        match self.definitions.get_mut(table_name) {
            Some(definitions) => {
                match definitions.iter_mut().find(|def| def.version() == definition.version()) {
                    Some(def) => *def = definition.to_owned(),
                    None => definitions.push(definition.to_owned()),
                }
            },
            None => { self.definitions.insert(table_name.to_owned(), vec![definition.to_owned()]); },
        }
    }

    /// Removes a specific table definition version from the schema.
    ///
    /// # Arguments
    ///
    /// * `table_name` - Name of the table
    /// * `version` - Version number of the definition to remove
    pub fn remove_definition(&mut self, table_name: &str, version: i32) {
        if let Some(definitions) = self.definitions.get_mut(table_name) {
            let mut index_to_delete = vec![];
            for (index, definition) in definitions.iter().enumerate() {
                if definition.version == version {
                    index_to_delete.push(index);
                }
            }

            index_to_delete.iter().rev().for_each(|index| { definitions.remove(*index); });
        }
    }

    /// Returns a cloned copy of all definitions for a table.
    ///
    /// # Arguments
    ///
    /// * `table_name` - Name of the table
    ///
    /// # Returns
    ///
    /// Returns a cloned vector of all definitions for the table, or [`None`] if not found.
    pub fn definitions_by_table_name_cloned(&self, table_name: &str) -> Option<Vec<Definition>> {
        self.definitions.get(table_name).cloned()
    }

    /// Returns a reference to all definitions for a table.
    ///
    /// # Arguments
    ///
    /// * `table_name` - Name of the table
    ///
    /// # Returns
    ///
    /// Returns a reference to the vector of definitions, or [`None`] if not found.
    pub fn definitions_by_table_name(&self, table_name: &str) -> Option<&Vec<Definition>>  {
        self.definitions.get(table_name)
    }

    /// Returns a mutable reference to all definitions for a table.
    ///
    /// # Arguments
    ///
    /// * `table_name` - Name of the table
    ///
    /// # Returns
    ///
    /// Returns a mutable reference to the vector of definitions, or [`None`] if not found.
    pub fn definitions_by_table_name_mut(&mut self, table_name: &str) -> Option<&mut Vec<Definition>>  {
        self.definitions.get_mut(table_name)
    }

    /// Returns the newest compatible definition for a table based on candidate versions.
    ///
    /// This function first tries to find a definition matching the highest version number
    /// from the candidates (typically from a dependency database). If that fails, it
    /// falls back to the first (newest) definition in the schema.
    ///
    /// # Arguments
    ///
    /// * `table_name` - Name of the table
    /// * `candidates` - List of candidate definitions (typically from dependencies)
    ///
    /// # Returns
    ///
    /// Returns the best matching definition, or [`None`] if the table is not found.
    pub fn definition_newer(&self, table_name: &str, candidates: &[Definition]) -> Option<&Definition> {

        // Version is... complicated. We don't really want the last one, but the last one compatible with our game.
        // So we have to try to get it first from the Dependency Database first. If that fails, we fall back to the schema.
        if let Some(definition) = candidates.iter().max_by(|x, y| x.version().cmp(y.version())) {
            self.definition_by_name_and_version(table_name, *definition.version())
        }

        // If there was no coincidence in the dependency database... we risk ourselves getting the last definition we have for
        // that db from the schema.
        else{
            self.definitions.get(table_name)?.first()
        }
    }

    /// Returns a reference to a specific table definition by name and version.
    ///
    /// # Arguments
    ///
    /// * `table_name` - Name of the table
    /// * `table_version` - Version number of the definition
    ///
    /// # Returns
    ///
    /// Returns the definition if found, or [`None`] otherwise.
    pub fn definition_by_name_and_version(&self, table_name: &str, table_version: i32) -> Option<&Definition>  {
        self.definitions.get(table_name)?.iter().find(|definition| *definition.version() == table_version)
    }

    /// Returns a mutable reference to a specific table definition by name and version.
    ///
    /// # Arguments
    ///
    /// * `table_name` - Name of the table
    /// * `table_version` - Version number of the definition
    ///
    /// # Returns
    ///
    /// Returns the definition if found, or [`None`] otherwise.
    pub fn definition_by_name_and_version_mut(&mut self, table_name: &str, table_version: i32) -> Option<&mut Definition>  {
        self.definitions.get_mut(table_name)?.iter_mut().find(|definition| *definition.version() == table_version)
    }

    /// Loads a [`Schema`] from a RON file, optionally merging local patches.
    ///
    /// This function loads a schema from a `.ron` file and applies any patches from both
    /// the schema itself and an optional local patches file. Patches from the local file
    /// are merged with schema patches and applied to all definitions.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the schema `.ron` file
    /// * `local_patches` - Optional path to a local patches file
    ///
    /// # Returns
    ///
    /// Returns the loaded schema with all patches applied, or an error if loading fails.
    pub fn load(path: &Path, local_patches: Option<&Path>) -> Result<Self> {
        let mut file = BufReader::new(File::open(path)?);
        let mut data = Vec::with_capacity(file.get_ref().metadata()?.len() as usize);
        file.read_to_end(&mut data)?;
        let mut schema: Self = from_bytes(&data)?;
        let mut patches = schema.patches().clone();

        // If we got local patches, add them to the patches list.
        //
        // NOTE: we separate the patches from the schemas because otherwise an schema edit will save local patches into the schema,
        // and we want them to remain local.
        if let Some(path) = local_patches {
            if let Ok(file) = File::open(path) {
                let mut file = BufReader::new(file);
                let mut data = Vec::with_capacity(file.get_ref().metadata()?.len() as usize);
                file.read_to_end(&mut data)?;
                if let Ok(local_patches) = from_bytes::<HashMap<String, DefinitionPatch>>(&data) {
                    Self::add_patches_to_patch_set(&mut patches, &local_patches);
                }
            }
        }

        // Preload all patches to their respective definitions.
        for (table_name, patches) in &patches {
            if let Some(definitions) = schema.definitions_by_table_name_mut(table_name) {
                for definition in definitions {
                    definition.set_patches(patches.clone());
                }
            }
        }

        Ok(schema)
    }

    /// Loads a [`Schema`] from a JSON file.
    ///
    /// Similar to [`load()`], but reads from a JSON file instead of RON. Applies all
    /// patches from the schema to the definitions.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the schema `.json` file
    ///
    /// # Returns
    ///
    /// Returns the loaded schema with patches applied, or an error if loading fails.
    ///
    /// [`load()`]: Schema::load
    pub fn load_json(path: &Path) -> Result<Self> {
        let mut file = BufReader::new(File::open(path)?);
        let mut data = Vec::with_capacity(file.get_ref().metadata()?.len() as usize);
        file.read_to_end(&mut data)?;
        let mut schema: Self = serde_json::from_slice(&data)?;

        // Preload all patches to their respective definitions.
        for (table_name, patches) in schema.patches().clone() {
            if let Some(definitions) = schema.definitions_by_table_name_mut(&table_name) {
                for definition in definitions {
                    definition.set_patches(patches.clone());
                }
            }
        }

        Ok(schema)
    }

    /// Saves the schema to a RON file.
    ///
    /// This function saves the schema to a `.ron` file, automatically:
    /// - Creating parent directories if needed
    /// - Sorting definitions by version (newest first)
    /// - Cleaning up invalid references
    /// - Moving certain patches from definitions to schema patches
    ///
    /// # Arguments
    ///
    /// * `path` - Path where the schema file should be saved
    ///
    /// # Returns
    ///
    /// Returns [`Ok`] if saved successfully, or an error if file I/O fails.
    pub fn save(&mut self, path: &Path) -> Result<()> {

        // Make sure the path exists to avoid problems with updating schemas.
        if let Some(parent_folder) = path.parent() {
            DirBuilder::new().recursive(true).create(parent_folder)?;
        }

        let mut file = BufWriter::new(File::create(path)?);
        let config = PrettyConfig::default();

        let mut patches = HashMap::new();

        // Make sure all definitions are properly sorted by version number.
        self.definitions.iter_mut().for_each(|(table_name, definitions)| {
            definitions.sort_by(|a, b| b.version().cmp(a.version()));

            // Fix for empty dependencies, again.
            definitions.iter_mut().for_each(|definition| {
                definition.fields.iter_mut().for_each(|field| {
                    if let Some((ref_table, ref_column)) = field.is_reference(None) {
                        if ref_table.trim().is_empty() || ref_column.trim().is_empty() {
                            field.is_reference = None;
                        }
                    }
                });

                // Move any lookup_hardcoded patches to schema patches.
                if definition.patches.values().any(|x| x.keys().any(|y| y == "lookup_hardcoded")) {
                    let mut def_patches = definition.patches().clone();
                    def_patches.retain(|_, value| {
                        value.retain(|key, _| key == "lookup_hardcoded");
                        !value.is_empty()
                    });
                    patches.insert(table_name.to_owned(), def_patches);
                }

                // Move any unused patches to schema patches.
                if definition.patches.values().any(|x| x.keys().any(|y| y == "unused")) {
                    let mut def_patches = definition.patches().clone();
                    def_patches.retain(|_, value| {
                        value.retain(|key, _| key == "unused");
                        !value.is_empty()
                    });
                    patches.insert(table_name.to_owned(), def_patches);
                }
            })
        });

        Self::add_patches_to_patch_set(self.patches_mut(), &patches);

        file.write_all(to_string_pretty(&self, config)?.as_bytes())?;
        Ok(())
    }

    /// Saves the schema to a JSON file.
    ///
    /// This function saves the schema to a `.json` file at the specified path, automatically:
    /// - Creating parent directories if needed
    /// - Changing the extension to `.json`
    /// - Sorting definitions by version (newest first)
    /// - Pretty-printing the JSON output
    ///
    /// # Arguments
    ///
    /// * `path` - Path where the schema file should be saved (extension will be changed to `.json`)
    ///
    /// # Returns
    ///
    /// Returns [`Ok`] if saved successfully, or an error if file I/O or serialization fails.
    pub fn save_json(&mut self, path: &Path) -> Result<()> {
        let mut path = path.to_path_buf();
        path.set_extension("json");

        // Make sure the path exists to avoid problems with updating schemas.
        if let Some(parent_folder) = path.parent() {
            DirBuilder::new().recursive(true).create(parent_folder)?;
        }

        let mut file = BufWriter::new(File::create(&path)?);

        // Make sure all definitions are properly sorted by version number.
        self.definitions.iter_mut().for_each(|(_, definitions)| {
            definitions.sort_by(|a, b| b.version().cmp(a.version()));
        });

        file.write_all(serde_json::to_string_pretty(&self)?.as_bytes())?;
        Ok(())
    }

    /// Exports all schema files in a folder to JSON format.
    ///
    /// This function loads all schema files (`.ron`) for supported games from the specified folder
    /// and saves them as `.json` files in the same location. This is primarily used for
    /// compatibility with external tools that prefer JSON.
    ///
    /// # Arguments
    ///
    /// * `schema_folder_path` - Path to the folder containing schema `.ron` files
    ///
    /// # Returns
    ///
    /// Returns [`Ok`] if all schemas are successfully exported, or an error if any operation fails.
    ///
    /// # Note
    ///
    /// This function processes schemas in parallel for better performance.
    pub fn export_to_json(schema_folder_path: &Path) -> Result<()> {
        let games = SupportedGames::default();

        games.games_sorted().par_iter().map(|x| x.schema_file_name()).try_for_each(|schema_file| {
            let mut schema_path = schema_folder_path.to_owned();
            schema_path.push(schema_file);

            let mut schema = Schema::load(&schema_path, None)?;
            schema_path.set_extension("json");
            schema.save_json(&schema_path)?;
            Ok(())
        })
    }

    /// Updates a schema from a legacy format to the current format.
    ///
    /// This function handles migration of schema files from older structural versions (e.g., v4)
    /// to the current structural version (v5). It automatically detects the schema version and
    /// applies the necessary transformations.
    ///
    /// # Arguments
    ///
    /// * `schema_path` - Path to the schema file to update
    /// * `schema_patches_path` - Path to the schema patches file
    /// * `game_name` - Name of the game this schema is for
    ///
    /// # Returns
    ///
    /// Returns [`Ok`] if the update succeeds, or an error if the update process fails.
    pub fn update(schema_path: &Path, schema_patches_path: &Path, game_name: &str) -> Result<()>{
        v4::SchemaV4::update(schema_path, schema_patches_path, game_name)
    }

    /// Returns all columns that reference fields in the specified table.
    ///
    /// This function searches through all table definitions in the schema to find fields
    /// that have foreign key references pointing to the provided table's fields.
    ///
    /// # Arguments
    ///
    /// * `table_name` - Name of the table to find references to
    /// * `definition` - Definition of the table (used to get the field list)
    ///
    /// # Returns
    ///
    /// Returns a map where:
    /// - Keys are local field names from the provided definition
    /// - Values are maps of `table_name -> Vec<field_name>` containing all referencing fields
    ///
    /// # Example
    ///
    /// For a `factions_tables` table, this might return:
    /// ```text
    /// {
    ///   "key": {
    ///     "units_tables": ["faction_key"],
    ///     "characters_tables": ["faction_key", "home_faction_key"]
    ///   }
    /// }
    /// ```
    pub fn referencing_columns_for_table(&self, table_name: &str, definition: &Definition) -> HashMap<String, HashMap<String, Vec<String>>> {

        // Iterate over all definitions and find the ones referencing our table/field.
        let fields_processed = definition.fields_processed();
        let definitions = self.definitions();
        let table_name_no_tables = table_name.to_owned().drain(..table_name.len() - 7).collect::<String>();

        fields_processed.iter().filter_map(|field| {

            let references = definitions.par_iter().filter_map(|(ver_name, ver_definitions)| {
                let mut references = ver_definitions.iter().filter_map(|ver_definition| {
                    let ver_patches = Some(ver_definition.patches());
                    let references = ver_definition.fields_processed().iter().filter_map(|ver_field| {
                        if let Some((source_table_name, source_column_name)) = ver_field.is_reference(ver_patches) {
                            if table_name_no_tables == source_table_name && field.name() == source_column_name {
                                Some(ver_field.name().to_owned())
                            } else { None }
                        } else { None }
                    }).collect::<Vec<String>>();
                    if references.is_empty() {
                        None
                    } else {
                        Some(references)
                    }
                }).flatten().collect::<Vec<String>>();
                if references.is_empty() {
                    None
                } else {
                    references.sort();
                    references.dedup();
                    Some((ver_name.to_owned(), references))
                }
            }).collect::<HashMap<String, Vec<String>>>();
            if references.is_empty() {
                None
            } else {
                Some((field.name().to_owned(), references))
            }
        }).collect()
    }

    /// Returns all tables and columns that reference the specified column, and whether LOC files may be affected.
    ///
    /// This function performs a recursive search to find all fields that reference the specified column,
    /// including indirect references (fields that reference fields that reference the target column).
    /// It also checks if changing the column would affect localisation keys.
    ///
    /// # Arguments
    ///
    /// * `table_name` - Name of the table containing the column (with or without `_tables` suffix)
    /// * `column_name` - Name of the column to find references to
    /// * `fields` - The table's field list
    /// * `localised_fields` - The table's localised field list
    ///
    /// # Returns
    ///
    /// Returns a tuple of:
    /// - A map of `table_name -> Vec<field_name>` containing all referencing fields (recursively)
    /// - A boolean indicating if LOC files may need updates (true if the column is a key field and the table has localised fields)
    ///
    /// # Note
    ///
    /// Recursion is supported for table references, but not for LOC field detection.
    pub fn tables_and_columns_referencing_our_own(
        &self,
        table_name: &str,
        column_name: &str,
        fields: &[Field],
        localised_fields: &[Field]
    ) -> (BTreeMap<String, Vec<String>>, bool) {

        // Make sure the table name is correct.
        let short_table_name = if table_name.ends_with("_tables") { table_name.split_at(table_name.len() - 7).0 } else { table_name };
        let mut tables: BTreeMap<String, Vec<String>> = BTreeMap::new();

        // We get all the db definitions from the schema, then iterate all of them to find what tables/columns reference our own.
        for (ref_table_name, ref_definition) in self.definitions() {
            let mut columns: Vec<String> = vec![];
            for ref_version in ref_definition {
                let ref_fields = ref_version.fields_processed();
                let ref_patches = Some(ref_version.patches());
                let ref_fields_localised = ref_version.localised_fields();
                for ref_field in &ref_fields {
                    if let Some((ref_ref_table, ref_ref_field)) = ref_field.is_reference(ref_patches) {

                        // As this applies to all versions of a table, skip repeated fields.
                        if ref_ref_table == short_table_name && ref_ref_field == column_name && !columns.iter().any(|x| x == ref_field.name()) {
                            columns.push(ref_field.name().to_owned());

                            // If we find a referencing column, get recursion working to check if there is any column referencing this one that needs to be edited.
                            let (ref_of_ref, _) = self.tables_and_columns_referencing_our_own(ref_table_name, ref_field.name(), &ref_fields, ref_fields_localised);
                            for refs in &ref_of_ref {
                                match tables.get_mut(refs.0) {
                                    Some(columns) => for value in refs.1 {
                                        if !columns.contains(value) {
                                            columns.push(value.to_owned());
                                        }
                                    }
                                    None => { tables.insert(refs.0.to_owned(), refs.1.to_vec()); },
                                }
                            }
                        }
                    }
                }
            }

            // Only add them if we actually found columns.
            if !columns.is_empty() {
                tables.insert(ref_table_name.to_owned(), columns);
            }
        }

        // Also, check if we have to be careful about localised fields.
        let patches = self.patches().get(table_name);
        let has_loc_fields = if let Some(field) = fields.iter().find(|x| x.name() == column_name) {
            (field.is_key(patches) || field.name() == "key") && !localised_fields.is_empty()
        } else { false };

        (tables, has_loc_fields)
    }
    /// Loads patches from a RON-formatted string.
    ///
    /// # Arguments
    ///
    /// * `patch` - RON-formatted string containing patches
    ///
    /// # Returns
    ///
    /// Returns the parsed patches, or an error if the string is not valid RON.
    pub fn load_patches_from_str(patch: &str) -> Result<HashMap<String, DefinitionPatch>> {
        from_str(patch).map_err(From::from)
    }

    /// Loads definitions from a RON-formatted string.
    ///
    /// # Arguments
    ///
    /// * `definition` - RON-formatted string containing table definitions
    ///
    /// # Returns
    ///
    /// Returns the parsed definitions, or an error if the string is not valid RON.
    pub fn load_definitions_from_str(definition: &str) -> Result<HashMap<String, Definition>> {
        from_str(definition).map_err(From::from)
    }

    /// Exports patches to a RON-formatted string.
    ///
    /// # Arguments
    ///
    /// * `patches` - The patches to export
    ///
    /// # Returns
    ///
    /// Returns the RON-formatted string, or an error if serialization fails.
    pub fn export_patches_to_str(patches: &HashMap<String, DefinitionPatch>) -> Result<String> {
        let config = PrettyConfig::default();
        ron::ser::to_string_pretty(&patches, config).map_err(From::from)
    }

    /// Exports definitions to a RON-formatted string.
    ///
    /// # Arguments
    ///
    /// * `definitions` - The definitions to export
    ///
    /// # Returns
    ///
    /// Returns the RON-formatted string, or an error if serialization fails.
    pub fn export_definitions_to_str(definitions: &HashMap<String, Definition>) -> Result<String> {
        let config = PrettyConfig::default();
        ron::ser::to_string_pretty(&definitions, config).map_err(From::from)
    }

    /// Uploads patches to Sentry for debugging/analysis.
    ///
    /// This function serializes the patches to RON format and sends them to Sentry as an
    /// informational event for tracking schema changes and debugging purposes.
    ///
    /// # Arguments
    ///
    /// * `sentry_guard` - The Sentry client guard
    /// * `game_name` - Name of the game the patches are for
    /// * `patches` - The patches to upload
    ///
    /// # Returns
    ///
    /// Returns [`Ok`] if the upload succeeds, or an error if serialization or upload fails.
    ///
    /// # Feature
    ///
    /// This function requires the `integration_log` feature.
    #[cfg(feature = "integration_log")]
    pub fn upload_patches(sentry_guard: &ClientInitGuard, game_name: &str, patches: HashMap<String, DefinitionPatch>) -> Result<()> {
        let level = Level::Info;
        let message = format!("Schema Patch for: {} - {}.", game_name, crate::utils::current_time()?);
        let config = PrettyConfig::default();
        let mut data = vec![];
        ron::ser::to_writer_pretty(&mut data, &patches, config)?;
        let file_name = "patch.txt";

        Logger::send_event(sentry_guard, level, &message, Some((file_name, &data)))
    }

    /// Uploads definitions to Sentry for debugging/analysis.
    ///
    /// This function serializes the definitions to RON format and sends them to Sentry as an
    /// informational event for tracking schema changes and debugging purposes.
    ///
    /// # Arguments
    ///
    /// * `sentry_guard` - The Sentry client guard
    /// * `game_name` - Name of the game the definitions are for
    /// * `definitions` - The definitions to upload
    ///
    /// # Returns
    ///
    /// Returns [`Ok`] if the upload succeeds, or an error if serialization or upload fails.
    ///
    /// # Feature
    ///
    /// This function requires the `integration_log` feature.
    #[cfg(feature = "integration_log")]
    pub fn upload_definitions(sentry_guard: &ClientInitGuard, game_name: &str, definitions: HashMap<String, Definition>) -> Result<()> {
        let level = Level::Info;
        let message = format!("Schema Definition for: {} - {}.", game_name, crate::utils::current_time()?);
        let config = PrettyConfig::default();
        let mut data = vec![];
        ron::ser::to_writer_pretty(&mut data, &definitions, config)?;
        let file_name = "definition.txt";

        Logger::send_event(sentry_guard, level, &message, Some((file_name, &data)))
    }
}

/// Implementation of [`Definition`].
impl Definition {

    /// Creates a new empty definition for a specific version.
    ///
    /// # Arguments
    ///
    /// * `version` - The version number for this definition
    /// * `schema_patches` - Optional patches to apply to this definition
    ///
    /// # Returns
    ///
    /// Returns a new empty definition with no fields.
    pub fn new(version: i32, schema_patches: Option<&DefinitionPatch>) -> Definition {
        Definition {
            version,
            localised_fields: vec![],
            fields: vec![],
            localised_key_order: vec![],
            patches: schema_patches.cloned().unwrap_or_default(),
        }
    }

    /// Creates a new definition with the specified fields.
    ///
    /// # Arguments
    ///
    /// * `version` - The version number for this definition
    /// * `fields` - The table's field list
    /// * `loc_fields` - The localised fields list
    /// * `schema_patches` - Optional patches to apply to this definition
    ///
    /// # Returns
    ///
    /// Returns a new definition with the provided fields.
    pub fn new_with_fields(version: i32, fields: &[Field], loc_fields: &[Field], schema_patches: Option<&DefinitionPatch>) -> Definition {
        Definition {
            version,
            localised_fields: loc_fields.to_vec(),
            fields: fields.to_vec(),
            localised_key_order: vec![],
            patches: schema_patches.cloned().unwrap_or_default(),
        }
    }

    /// Returns reference and lookup information for all fields with foreign key references.
    ///
    /// This function extracts foreign key information from all fields in the definition
    /// that have a reference to another table.
    ///
    /// # Returns
    ///
    /// Returns a map where:
    /// - Keys are field indices (as [`i32`])
    /// - Values are tuples of `(referenced_table, referenced_column, optional_lookup_columns)`
    ///
    /// Only fields with `is_reference` set are included in the result.
    pub fn reference_data(&self) -> BTreeMap<i32, (String, String, Option<Vec<String>>)> {
        self.fields.iter()
            .enumerate()
            .filter(|x| x.1.is_reference.is_some())
            .map(|x| (x.0 as i32, (x.1.is_reference.clone().unwrap().0, x.1.is_reference.clone().unwrap().1, x.1.lookup.clone())))
            .collect()
    }

    /// Returns the processed field list with transformations applied.
    ///
    /// This function processes the raw field list and applies various transformations:
    /// - **Bitwise fields**: Expanded into multiple boolean fields (e.g., `flags` → `flags_1`, `flags_2`, etc.)
    /// - **Enum fields**: Converted to StringU8 fields
    /// - **Colour fields**: RGB triplets merged into single ColourRGB fields
    /// - **Numeric fields**: Converted to I32 fields (with patches)
    ///
    /// This is the field list that should be used for UI display and data editing.
    ///
    /// # Returns
    ///
    /// Returns the processed field list with all transformations applied.
    pub fn fields_processed(&self) -> Vec<Field> {
        let mut split_colour_fields: BTreeMap<u8, Field> = BTreeMap::new();
        let patches = Some(self.patches());
        let mut fields = self.fields().iter()
            .filter_map(|x|
                if x.is_bitwise() > 1 {
                    let unused = x.unused(patches);
                    let mut fields = vec![x.clone(); x.is_bitwise() as usize];
                    fields.iter_mut().enumerate().for_each(|(index, field)| {
                        field.set_name(format!("{}_{}", field.name(), index + 1));
                        field.set_field_type(FieldType::Boolean);
                        field.set_unused(unused);
                    });
                    Some(fields)
                }

                else if !x.enum_values().is_empty() {
                    let mut field = x.clone();
                    field.set_field_type(FieldType::StringU8);
                    Some(vec![field; 1])
                }

                else if let Some(colour_index) = x.is_part_of_colour() {
                    match split_colour_fields.get_mut(&colour_index) {

                        // If found, add the default value to the other previously known default value.
                        Some(field) => {
                            let default_value = match x.default_value(None) {
                                Some(default_value) => {
                                    if x.name.ends_with("_r") || x.name.ends_with("_red") || x.name == "r" || x.name == "red" {
                                        field.default_value.clone().map(|df| {
                                            format!("{:X}{}", default_value.parse::<i32>().unwrap_or(0), &df[2..])
                                        })
                                    } else if x.name.ends_with("_g") || x.name.ends_with("_green") || x.name == "g" || x.name == "green" {
                                        field.default_value.clone().map(|df| {
                                            format!("{}{:X}{}", &df[..2], default_value.parse::<i32>().unwrap_or(0), &df[4..])
                                        })
                                    } else if x.name.ends_with("_b") || x.name.ends_with("_blue") || x.name == "b" || x.name == "blue" {
                                        field.default_value.clone().map(|df| {
                                            format!("{}{:X}", &df[..4], default_value.parse::<i32>().unwrap_or(0))
                                        })
                                    } else {
                                        Some("000000".to_owned())
                                    }
                                }
                                None => Some("000000".to_owned())
                            };

                            // Update the default value with the one for this colour.
                            field.set_default_value(default_value);

                            if !field.unused(patches) {
                                field.set_unused(x.unused(patches));
                            }
                        },
                        None => {
                            let unused = x.unused(patches);
                            let colour_split = x.name().rsplitn(2, '_').collect::<Vec<&str>>();
                            let colour_field_name = if colour_split.len() == 2 {
                                format!("{}{}", colour_split[1].to_lowercase(), MERGE_COLOUR_POST)
                            } else {
                                format!("{}_{}", MERGE_COLOUR_NO_NAME.to_lowercase(), colour_index)
                            };

                            let mut field = x.clone();
                            field.set_name(colour_field_name);
                            field.set_field_type(FieldType::ColourRGB);
                            field.set_unused(unused);

                            // We need to fix the default value so it's a ColourRGB one.
                            let default_value = match field.default_value(None) {
                                Some(default_value) => {
                                    if x.name.ends_with("_r") || x.name.ends_with("_red") || x.name == "r" || x.name == "red" {
                                        Some(format!("{:X}0000", default_value.parse::<i32>().unwrap_or(0)))
                                    } else if x.name.ends_with("_g") || x.name.ends_with("_green") || x.name == "g" || x.name == "green" {
                                        Some(format!("00{:X}00", default_value.parse::<i32>().unwrap_or(0)))
                                    } else if x.name.ends_with("_b") || x.name.ends_with("_blue") || x.name == "b" || x.name == "blue" {
                                        Some(format!("0000{:X}", default_value.parse::<i32>().unwrap_or(0)))
                                    } else {
                                        Some("000000".to_owned())
                                    }
                                }
                                None => Some("000000".to_owned())
                            };

                            field.set_default_value(default_value);

                            split_colour_fields.insert(colour_index, field);
                        }
                    }

                    None
                }

                else if x.is_numeric(patches) {
                    let mut field = x.clone();
                    field.set_field_type(FieldType::I32);
                    Some(vec![field; 1])
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

    /// Returns the original raw field corresponding to a processed field index.
    ///
    /// This function maps a field from the processed field list back to its original
    /// raw field definition. This is useful when you need to access the underlying
    /// field data before transformations like bitwise expansion.
    ///
    /// # Arguments
    ///
    /// * `index` - Index in the processed field list
    ///
    /// # Returns
    ///
    /// Returns the original field from the raw field list.
    ///
    /// # Panics
    ///
    /// Panics if the field is not found (which should never happen for valid indices).
    ///
    /// # Note
    ///
    /// This function does not work correctly with combined colour fields, as they don't
    /// have a direct 1:1 mapping to a single raw field.
    pub fn original_field_from_processed(&self, index: usize) -> Field {
        let fields = self.fields();
        let processed = self.fields_processed();

        let field_processed = &processed[index];
        let name = if field_processed.is_bitwise() > 1 {
            let mut name = field_processed.name().to_owned();
            name.drain(..name.rfind('_').unwrap()).collect::<String>()
        }
        else {field_processed.name().to_owned() };

        fields.iter().find(|x| *x.name() == name).unwrap().clone()
    }

    /// Returns the processed field list sorted by either key fields or CA order.
    ///
    /// This function returns the processed fields sorted according to the specified criteria.
    ///
    /// # Arguments
    ///
    /// * `key_first` - If `true`, sorts key fields first, then non-key fields. If `false`, sorts by CA order.
    ///
    /// # Returns
    ///
    /// Returns the sorted field list. Fields with `ca_order == -1` are left in their original order
    /// when sorting by CA order.
    pub fn fields_processed_sorted(&self, key_first: bool) -> Vec<Field> {
        let mut fields = self.fields_processed();
        let patches = Some(self.patches());
        fields.sort_by(|a, b| {
            if key_first {
                if a.is_key(patches) && b.is_key(patches) { Ordering::Equal }
                else if a.is_key(patches) && !b.is_key(patches) { Ordering::Less }
                else if !a.is_key(patches) && b.is_key(patches) { Ordering::Greater }
                else { Ordering::Equal }
            }
            else if a.ca_order() == -1 || b.ca_order() == -1 { Ordering::Equal }
            else { a.ca_order().cmp(&b.ca_order()) }
        });
        fields
    }

    /// Returns the position of a column in the processed field list by name.
    ///
    /// # Arguments
    ///
    /// * `column_name` - Name of the column to find
    ///
    /// # Returns
    ///
    /// Returns the column's index in the processed field list, or [`None`] if not found.
    pub fn column_position_by_name(&self, column_name: &str) -> Option<usize> {
        self.fields_processed()
            .iter()
            .position(|x| x.name() == column_name)
    }

    /// Returns the positions of all key columns in the processed field list.
    ///
    /// # Returns
    ///
    /// Returns a vector of indices for all fields marked as key fields.
    pub fn key_column_positions(&self) -> Vec<usize> {
        self.fields_processed()
            .iter()
            .enumerate()
            .filter(|(_, x)| x.is_key(Some(self.patches())))
            .map(|(x, _)| x)
            .collect::<Vec<_>>()
    }

    /// Returns the positions of all key columns sorted by CA order.
    ///
    /// This function returns key column positions in the same order as they appear in
    /// CA's Assembly Kit, rather than the binary order. This is primarily needed for
    /// `twad_key_deletes` functionality, which uses CA's ordering.
    ///
    /// # Returns
    ///
    /// Returns a vector of key column indices sorted by their `ca_order` value.
    pub fn key_column_positions_by_ca_order(&self) -> Vec<usize> {
        let fields_processed = self.fields_processed();
        let mut keys = fields_processed
            .iter()
            .enumerate()
            .filter(|(_, x)| x.is_key(Some(self.patches())))
            .map(|(x, _)| x)
            .collect::<Vec<_>>();

        keys.sort_by_key(|x| fields_processed[*x].ca_order);
        keys
    }

    /// Generates a SQL `CREATE TABLE` statement for this definition.
    ///
    /// This function creates a SQL statement suitable for creating a table in SQLite
    /// with the structure defined by this definition. The table includes additional
    /// metadata columns (`pack_name`, `file_name`, `is_vanilla`) for tracking data sources.
    ///
    /// # Arguments
    ///
    /// * `table_name` - Name for the SQL table
    ///
    /// # Returns
    ///
    /// Returns the SQL `CREATE TABLE` statement as a string.
    ///
    /// # Note
    ///
    /// Foreign key constraints are intentionally disabled because Total War tables
    /// (especially in mods) often have referential integrity issues. The function
    /// only creates a primary key constraint on the key fields.
    ///
    /// # Feature
    ///
    /// This function requires the `integration_sqlite` feature.
    #[cfg(feature = "integration_sqlite")]
    pub fn map_to_sql_create_table_string(&self, table_name: &str) -> String {
        let patches = Some(self.patches());
        let fields_sorted = self.fields_processed();
        let fields_query = fields_sorted.iter().map(|field| field.map_to_sql_string(patches)).collect::<Vec<_>>().join(",");

        let local_keys_join = fields_sorted.iter().filter_map(|field| if field.is_key(patches) { Some(format!("\"{}\"", field.name()))} else { None }).collect::<Vec<_>>().join(",");
        let local_keys = format!("CONSTRAINT unique_key PRIMARY KEY (\"pack_name\", \"file_name\", {local_keys_join})");
        //let foreign_keys = fields_sorted.iter()
        //    .filter_map(|field| field.is_reference(patches).clone().map(|(ref_table, ref_column)| (field.name(), ref_table, ref_column)))
        //    .map(|(loc_name, ref_table, ref_field)| format!("CONSTRAINT fk_{table_name} FOREIGN KEY (\"{loc_name}\") REFERENCES {ref_table}(\"{ref_field}\") ON UPDATE CASCADE ON DELETE CASCADE"))
        //    .collect::<Vec<_>>()
        //    .join(",");

        //if foreign_keys.is_empty() {
            if local_keys_join.is_empty() {
                format!("CREATE TABLE \"{}_v{}\" (\"pack_name\" STRING NOT NULL, \"file_name\" STRING NOT NULL, \"is_vanilla\" INTEGER DEFAULT 0, {})",
                    table_name.replace('\"', "'"),
                    self.version(),
                    fields_query
                )
            } else {
                format!("CREATE TABLE \"{}_v{}\" (\"pack_name\" STRING NOT NULL, \"file_name\" STRING NOT NULL, \"is_vanilla\" INTEGER DEFAULT 0, {}, {})",
                    table_name.replace('\"', "'"),
                    self.version(),
                    fields_query,
                    local_keys
                )
            }
        /*} else if local_keys_join.is_empty() {
            format!("CREATE TABLE \"{}_v{}\" (\"table_unique_id\" INTEGER DEFAULT 0, {}, {})",
                table_name.replace('\"', "'"),
                self.version(),
                fields_query,
                foreign_keys
            )
        } else {
            format!("CREATE TABLE \"{}_v{}\" (\"table_unique_id\" INTEGER DEFAULT 0, {}, {}, {})",
                table_name.replace('\"', "'"),
                self.version(),
                fields_query,
                local_keys,
                foreign_keys
            )
        }*/
    }

    /// Generates the column list for a SQL `INSERT INTO` statement.
    ///
    /// This function creates the column name list portion of an `INSERT INTO` statement,
    /// including the metadata columns and all processed fields.
    ///
    /// # Returns
    ///
    /// Returns a string like `("pack_name", "file_name", "is_vanilla", "field1", "field2", ...)`.
    ///
    /// # Feature
    ///
    /// This function requires the `integration_sqlite` feature.
    #[cfg(feature = "integration_sqlite")]
    pub fn map_to_sql_insert_into_string(&self) -> String {
        let fields_sorted = self.fields_processed();
        let fields_query = fields_sorted.iter().map(|field| format!("\"{}\"", field.name())).collect::<Vec<_>>().join(",");
        let fields_query = format!("(\"pack_name\", \"file_name\", \"is_vanilla\", {fields_query})");

        fields_query
    }

    /// Updates field properties from Assembly Kit raw definition data.
    ///
    /// This function updates the definition's fields with data extracted from the Assembly Kit,
    /// matching fields by name and updating specific properties. Fields not found in the
    /// Assembly Kit are added to the `unfound_fields` list for reporting.
    ///
    /// # Updated Properties
    ///
    /// - `is_key`: Primary key status
    /// - `default_value`: Default value for new rows
    /// - `filename_relative_path`: Path hints for filename fields
    /// - `is_filename`: Whether the field contains a filename
    /// - `is_reference`: Foreign key reference information
    /// - `lookup`: Lookup column information
    /// - `description`: Field description
    /// - `ca_order`: Visual position in Assembly Kit
    /// - `is_part_of_colour`: Auto-detected RGB colour field grouping
    ///
    /// # Arguments
    ///
    /// * `raw_definition` - The Assembly Kit definition data
    /// * `unfound_fields` - List to append unfound field names to (format: `"table_name/field_name"`)
    ///
    /// # Note
    ///
    /// Fields in `IGNORABLE_FIELDS` are automatically skipped and not reported as unfound.
    ///
    /// # Feature
    ///
    /// This function requires the `integration_assembly_kit` feature.
    #[cfg(feature = "integration_assembly_kit")]
    pub fn update_from_raw_definition(&mut self, raw_definition: &RawDefinition, unfound_fields: &mut Vec<String>) {
        let raw_table_name = &raw_definition.name.as_ref().unwrap()[..raw_definition.name.as_ref().unwrap().len() - 4];
        let mut combined_fields = BTreeMap::new();
        for (index, raw_field) in raw_definition.fields.iter().enumerate() {

            let mut found = false;
            for field in &mut self.fields {
                if field.name == raw_field.name {
                    if (raw_field.primary_key == "1" && !field.is_key) || (raw_field.primary_key == "0" && field.is_key) {
                        field.is_key = raw_field.primary_key == "1";
                    }

                    if raw_field.default_value.is_some() {
                        field.default_value = raw_field.default_value.clone();
                    }

                    if let Some(ref path) = raw_field.filename_relative_path {
                        let mut new_path = path.to_owned();
                        if path.contains(",") {
                            new_path = path.split(',').map(|x| x.trim()).join(";");
                        }

                        field.filename_relative_path = Some(new_path);
                    }

                    // Some fields are marked as filename, but only have fragment paths, which do not seem to correlate to game file paths.
                    // We need to disable those to avoid false positives on diagnostics.
                    field.is_filename = match raw_field.is_filename {
                        Some(_) => !(raw_field.fragment_path.is_some() && raw_field.filename_relative_path.is_none()),
                        None => false,
                    };

                    // Make sure to cleanup any old invalid definition.
                    if let Some(ref description) = raw_field.field_description {
                        field.description = description.to_owned();
                    } else {
                        field.description = String::new();
                    }

                    // We reset these so we don't inherit wrong references from older tables.
                    field.is_reference = Default::default();
                    field.lookup = Default::default();
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

                    field.ca_order = index as i16;

                    // Detect and group colour fields.
                    let is_numeric = matches!(field.field_type, FieldType::I16 | FieldType::I32 | FieldType::I64 | FieldType::F32 | FieldType::F64);

                    if is_numeric && (
                        field.name.ends_with("_r") ||
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
                        field.name == "blue"
                    ) {
                        let colour_split = field.name.rsplitn(2, '_').collect::<Vec<&str>>();
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
                    found = true;
                    break;
                }
            }

            if !found {

                // We need to check if it's a loc field before reporting it as unfound.
                for loc_field in self.localised_fields() {
                    if loc_field.name == raw_field.name {
                        found = true;
                        break;
                    }
                }

                // We automatically ignore certain old fields that have nothing to do with the game's data.
                if !found && !IGNORABLE_FIELDS.contains(&&*raw_field.name) {
                    unfound_fields.push(format!("{}/{}", raw_table_name, raw_field.name));
                }
            }
        }
    }

    /// Populates the `localised_fields` list from Assembly Kit data.
    ///
    /// This function identifies fields that should be extracted to LOC files based on
    /// Assembly Kit localisable field data and updates the definition's `localised_fields` list.
    /// All identified localised fields are set to [`FieldType::StringU8`] for consistency.
    ///
    /// # Arguments
    ///
    /// * `raw_definition` - The Assembly Kit table definition
    /// * `raw_localisable_fields` - List of all localisable fields from the Assembly Kit
    ///
    /// # Feature
    ///
    /// This function requires the `integration_assembly_kit` feature.
    #[cfg(feature = "integration_assembly_kit")]
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

            self.localised_fields = localisable_fields.iter().map(|x| From::from(*x)).collect();

            // Set their type to StringU8 for consistency.
            self.localised_fields.iter_mut().for_each(|field| field.field_type = FieldType::StringU8);
        }
    }
}

/// Implementation of `Field`.
impl Field {

    /// Creates a new field with the specified properties.
    ///
    /// # Arguments
    ///
    /// * `name` - Field name
    /// * `field_type` - Data type of the field
    /// * `is_key` - Whether this field is part of the primary key
    /// * `default_value` - Optional default value
    /// * `is_filename` - Whether this field contains a filename
    /// * `filename_relative_path` - Optional path hints for filename fields
    /// * `is_reference` - Optional foreign key reference `(table, column)`
    /// * `lookup` - Optional lookup columns
    /// * `description` - Field description
    /// * `ca_order` - Visual position in Assembly Kit
    /// * `is_bitwise` - Number of boolean columns to expand into (0 or 1 = no expansion)
    /// * `enum_values` - Map of integer values to string names for enum fields
    /// * `is_part_of_colour` - Optional RGB colour group index
    ///
    /// # Returns
    ///
    /// Returns a new [`Field`] instance with the specified properties.
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
            is_part_of_colour,
            unused: false
        }
    }

    //----------------------------------------------------------------------//
    // Manual getter implementations with patch support
    //----------------------------------------------------------------------//

    /// Returns the field name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the field's data type.
    pub fn field_type(&self) -> &FieldType {
        &self.field_type
    }

    /// Returns whether this field is a key field, applying patches if provided.
    ///
    /// # Arguments
    ///
    /// * `schema_patches` - Optional patches to check for overrides
    ///
    /// # Returns
    ///
    /// Returns `true` if the field is a key field (either by base definition or patch).
    pub fn is_key(&self, schema_patches: Option<&DefinitionPatch>) -> bool {
        if let Some(schema_patches) = schema_patches {
            if let Some(patch) = schema_patches.get(self.name()) {
                if let Some(field_patch) = patch.get("is_key") {
                    return field_patch.parse().unwrap_or(false);
                }
            }
        }

        self.is_key
    }

    /// Returns the field's default value, applying patches if provided.
    ///
    /// # Arguments
    ///
    /// * `schema_patches` - Optional patches to check for overrides
    ///
    /// # Returns
    ///
    /// Returns the default value if set (either by base definition or patch).
    pub fn default_value(&self, schema_patches: Option<&DefinitionPatch>) -> Option<String> {
        if let Some(schema_patches) = schema_patches {
            if let Some(patch) = schema_patches.get(self.name()) {
                if let Some(field_patch) = patch.get("default_value") {
                    return Some(field_patch.to_string());
                }
            }
        }

        self.default_value.clone()
    }

    /// Returns whether this field contains a filename, applying patches if provided.
    ///
    /// # Arguments
    ///
    /// * `schema_patches` - Optional patches to check for overrides
    ///
    /// # Returns
    ///
    /// Returns `true` if the field contains a filename path.
    pub fn is_filename(&self, schema_patches: Option<&DefinitionPatch>) -> bool {
        if let Some(schema_patches) = schema_patches {
            if let Some(patch) = schema_patches.get(self.name()) {
                if let Some(field_patch) = patch.get("is_filename") {
                    return field_patch.parse().unwrap_or(false);
                }
            }
        }

        self.is_filename
    }

    /// Returns the filename relative paths, applying patches if provided.
    ///
    /// The paths are split by semicolons and backslashes are converted to forward slashes.
    ///
    /// # Arguments
    ///
    /// * `schema_patches` - Optional patches to check for overrides
    ///
    /// # Returns
    ///
    /// Returns a vector of relative path strings, or [`None`] if no paths are defined.
    pub fn filename_relative_path(&self, schema_patches: Option<&DefinitionPatch>) -> Option<Vec<String>> {
        if let Some(schema_patches) = schema_patches {
            if let Some(patch) = schema_patches.get(self.name()) {
                if let Some(field_patch) = patch.get("filename_relative_path") {
                    return Some(field_patch.replace('\\', "/").split(';').map(|x| x.to_string()).collect::<Vec<String>>());
                }
            }
        }

        self.filename_relative_path.clone().map(|x| x.replace('\\', "/").split(';').map(|x| x.to_string()).collect::<Vec<String>>())
    }

    /// Returns the foreign key reference information, applying patches if provided.
    ///
    /// # Arguments
    ///
    /// * `schema_patches` - Optional patches to check for overrides
    ///
    /// # Returns
    ///
    /// Returns `Some((table_name, column_name))` if this field references another table,
    /// or [`None`] if it doesn't. The table name does not include the `_tables` suffix.
    pub fn is_reference(&self, schema_patches: Option<&DefinitionPatch>) -> Option<(String,String)> {
        if let Some(schema_patches) = schema_patches {
            if let Some(patch) = schema_patches.get(self.name()) {
                if let Some(field_patch) = patch.get("is_reference") {
                    let split = field_patch.splitn(2, ';').collect::<Vec<_>>();
                    if split.len() == 2 {
                        return Some((split[0].to_string(), split[1].to_string()));
                    }
                }
            }
        }

        self.is_reference.clone()
    }

    /// Returns the lookup column list, applying patches if provided.
    ///
    /// Lookup columns are additional columns from the referenced table that should
    /// be displayed in the UI alongside the referenced field.
    ///
    /// # Arguments
    ///
    /// * `schema_patches` - Optional patches to check for overrides
    ///
    /// # Returns
    ///
    /// Returns a vector of column names to look up, or [`None`] if no lookups are defined.
    pub fn lookup(&self, schema_patches: Option<&DefinitionPatch>) -> Option<Vec<String>> {
        if let Some(schema_patches) = schema_patches {
            if let Some(patch) = schema_patches.get(self.name()) {
                if let Some(field_patch) = patch.get("lookup") {
                    return Some(field_patch.split(';').map(|x| x.to_string()).collect());
                }
            }
        }

        self.lookup.clone()
    }

    /// Returns the lookup column list without applying patches.
    ///
    /// # Returns
    ///
    /// Returns a vector of column names from the base definition, ignoring any patches.
    pub fn lookup_no_patch(&self) -> Option<Vec<String>> {
        self.lookup.clone()
    }

    /// Returns hardcoded lookup values from patches.
    ///
    /// Hardcoded lookups provide predefined value mappings that don't require
    /// querying the referenced table. This is useful for performance or when
    /// the referenced table is not available.
    ///
    /// # Arguments
    ///
    /// * `schema_patches` - Optional patches to check for hardcoded values
    ///
    /// # Returns
    ///
    /// Returns a map of key values to their display strings. Returns an empty
    /// map if no hardcoded lookups are defined.
    pub fn lookup_hardcoded(&self, schema_patches: Option<&DefinitionPatch>) -> HashMap<String, String> {
        if let Some(schema_patches) = schema_patches {
            if let Some(patch) = schema_patches.get(self.name()) {
                if let Some(field_patch) = patch.get("lookup_hardcoded") {
                    let entries = field_patch.split(":::::").map(|x| x.split(";;;;;").collect::<Vec<_>>()).collect::<Vec<_>>();
                    let mut hashmap = HashMap::new();
                    for entry in entries {
                        hashmap.insert(entry[0].to_owned(), entry[1].to_owned());
                    }
                    return hashmap;
                }
            }
        }

        HashMap::new()
    }

    /// Returns the field description, applying patches if provided.
    ///
    /// # Arguments
    ///
    /// * `schema_patches` - Optional patches to check for overrides
    ///
    /// # Returns
    ///
    /// Returns the field's description text. May be empty if no description is set.
    pub fn description(&self, schema_patches: Option<&DefinitionPatch>) -> String {
        if let Some(schema_patches) = schema_patches {
            if let Some(patch) = schema_patches.get(self.name()) {
                if let Some(field_patch) = patch.get("description") {
                    return field_patch.to_owned();
                }
            }
        }

        self.description.to_owned()
    }

    /// Returns the CA order value.
    ///
    /// This represents the visual position of the field in CA's Assembly Kit.
    /// A value of `-1` indicates the position is unknown.
    pub fn ca_order(&self) ->  i16 {
        self.ca_order
    }

    /// Returns the bitwise expansion count.
    ///
    /// # Returns
    ///
    /// - `0` or `1`: No bitwise expansion
    /// - `> 1`: Number of boolean columns this field should be expanded into
    pub fn is_bitwise(&self) -> i32 {
        self.is_bitwise
    }

    /// Returns the enum value mappings.
    ///
    /// # Returns
    ///
    /// Returns a reference to the map of integer values to their string names.
    /// Empty if this field is not an enum.
    pub fn enum_values(&self) -> &BTreeMap<i32,String> {
        &self.enum_values
    }

    /// Returns the enum values as an [`Option`].
    pub fn enum_values_to_option(&self) -> Option<BTreeMap<i32, String>> {
        if self.enum_values.is_empty() { None }
        else { Some(self.enum_values.clone()) }
    }

    /// Returns the enum values as a semicolon-separated string.
    ///
    /// # Returns
    ///
    /// Returns a string in the format `"value1,name1;value2,name2;..."`.
    pub fn enum_values_to_string(&self) -> String {
        self.enum_values.iter().map(|(x, y)| format!("{x},{y}")).collect::<Vec<String>>().join(";")
    }

    /// Returns the RGB colour group index.
    ///
    /// # Returns
    ///
    /// Returns the colour group index if this field is part of an RGB triplet,
    /// or [`None`] if it's not a colour field.
    pub fn is_part_of_colour(&self) -> Option<u8>{
        self.is_part_of_colour
    }

    /// Returns whether this field should be treated as numeric (currently always `false`).
    ///
    /// This is a placeholder for future functionality and currently always returns `false`.
    ///
    /// # Arguments
    ///
    /// * `_schema_patches` - Unused (reserved for future use)
    pub fn is_numeric(&self, _schema_patches: Option<&DefinitionPatch>) -> bool {
        false
        /*
        if let Some(schema_patches) = schema_patches {
            if let Some(patch) = schema_patches.get(self.name()) {
                if let Some(is_numeric) = patch.get("is_numeric") {
                    return is_numeric.parse::<bool>().unwrap_or(false);
                }
            }
        }

        false*/
    }

    /// Returns whether this field cannot be empty, checking patches.
    ///
    /// # Arguments
    ///
    /// * `schema_patches` - Optional patches to check for the `not_empty` flag
    ///
    /// # Returns
    ///
    /// Returns `true` if the field is marked as "cannot be empty" via a patch.
    pub fn cannot_be_empty(&self, schema_patches: Option<&DefinitionPatch>) -> bool {
        if let Some(schema_patches) = schema_patches {
            if let Some(patch) = schema_patches.get(self.name()) {
                if let Some(cannot_be_empty) = patch.get("not_empty") {
                    return cannot_be_empty.parse::<bool>().unwrap_or(false);
                }
            }
        }

        false
    }

    /// Returns whether this field is unused by the game.
    ///
    /// Fields marked as unused are still present in the binary format but are not
    /// actually used by the game logic. This information is primarily determined via patches.
    ///
    /// # Arguments
    ///
    /// * `schema_patches` - Optional patches to check for the `unused` flag
    ///
    /// # Returns
    ///
    /// Returns `true` if the field is marked as unused (either in the base definition or via patch).
    pub fn unused(&self, schema_patches: Option<&DefinitionPatch>) -> bool {

        // By default all fields are used, except the ones set through patches. If it's already marked unused, return early.
        self.unused || {

            if let Some(schema_patches) = schema_patches {
                if let Some(patch) = schema_patches.get(self.name()) {
                    if let Some(cannot_be_empty) = patch.get("unused") {
                        return cannot_be_empty.parse::<bool>().unwrap_or(false);
                    }
                }
            }

            false
        }
    }

    /// Generates a SQL column definition string for this field.
    ///
    /// This function creates the SQL column definition portion for use in a
    /// `CREATE TABLE` statement, including the data type and optional default value.
    ///
    /// # Arguments
    ///
    /// * `schema_patches` - Optional patches to apply when getting the default value
    ///
    /// # Returns
    ///
    /// Returns a string like `"field_name" INTEGER DEFAULT "value"`.
    ///
    /// # Feature
    ///
    /// This function requires the `integration_sqlite` feature.
    #[cfg(feature = "integration_sqlite")]
    pub fn map_to_sql_string(&self, schema_patches: Option<&DefinitionPatch>) -> String {
        let mut string = format!(" \"{}\" {:?} ", self.name(), self.field_type().map_to_sql_type());

        if let Some(default_value) = self.default_value(schema_patches) {
            string.push_str(&format!(" DEFAULT \"{}\"", default_value.replace("\"", "\"\"")));
        }

        string
    }
}

impl FieldType {

    /// Maps this field type to its corresponding SQLite type.
    ///
    /// This function converts RPFM's field types to their appropriate SQLite equivalents
    /// for database operations.
    ///
    /// # Returns
    ///
    /// Returns the SQLite [`Type`] that best represents this field type:
    /// - Numeric types → [`Type::Integer`] or [`Type::Real`]
    /// - String types → [`Type::Text`]
    /// - Sequence types → [`Type::Blob`]
    ///
    /// # Feature
    ///
    /// This function requires the `integration_sqlite` feature.
    #[cfg(feature = "integration_sqlite")]
    pub fn map_to_sql_type(&self) -> Type {
        match self {
            FieldType::Boolean => Type::Integer,
            FieldType::F32 => Type::Real,
            FieldType::F64 => Type::Real,
            FieldType::I16 => Type::Integer,
            FieldType::I32 => Type::Integer,
            FieldType::I64 => Type::Integer,
            FieldType::ColourRGB => Type::Text,
            FieldType::StringU8 => Type::Text,
            FieldType::StringU16 => Type::Text,
            FieldType::OptionalI16 => Type::Integer,
            FieldType::OptionalI32 => Type::Integer,
            FieldType::OptionalI64 => Type::Integer,
            FieldType::OptionalStringU8 => Type::Text,
            FieldType::OptionalStringU16 => Type::Text,
            FieldType::SequenceU16(_) => Type::Blob,
            FieldType::SequenceU32(_) => Type::Blob,
        }
    }
}
//---------------------------------------------------------------------------//
//                         Extra Implementations
//---------------------------------------------------------------------------//

/// Default implementation of `Schema`.
impl Default for Schema {
    fn default() -> Self {
        Self {
            version: CURRENT_STRUCTURAL_VERSION,
            definitions: HashMap::new(),
            patches: HashMap::new()
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
            unused: false,
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
            FieldType::OptionalI16 => write!(f, "OptionalI16"),
            FieldType::OptionalI32 => write!(f, "OptionalI32"),
            FieldType::OptionalI64 => write!(f, "OptionalI64"),
            FieldType::OptionalStringU8 => write!(f, "OptionalStringU8"),
            FieldType::OptionalStringU16 => write!(f, "OptionalStringU16"),
            FieldType::SequenceU16(_) => write!(f, "SequenceU16"),
            FieldType::SequenceU32(_) => write!(f, "SequenceU32"),
        }
    }
}

/// Implementation of `From<&RawDefinition>` for `Definition.
impl From<&DecodedData> for FieldType {
    fn from(data: &DecodedData) -> Self {
        match data {
            DecodedData::Boolean(_) => FieldType::Boolean,
            DecodedData::F32(_) => FieldType::F32,
            DecodedData::F64(_) => FieldType::F64,
            DecodedData::I16(_) => FieldType::I16,
            DecodedData::I32(_) => FieldType::I32,
            DecodedData::I64(_) => FieldType::I64,
            DecodedData::ColourRGB(_) => FieldType::ColourRGB,
            DecodedData::StringU8(_) => FieldType::StringU8,
            DecodedData::StringU16(_) => FieldType::StringU16,
            DecodedData::OptionalI16(_) => FieldType::OptionalI16,
            DecodedData::OptionalI32(_) => FieldType::OptionalI32,
            DecodedData::OptionalI64(_) => FieldType::OptionalI64,
            DecodedData::OptionalStringU8(_) => FieldType::OptionalStringU8,
            DecodedData::OptionalStringU16(_) => FieldType::OptionalStringU16,
            DecodedData::SequenceU16(_) => FieldType::SequenceU16(Box::new(Definition::new(INVALID_VERSION, None))),
            DecodedData::SequenceU32(_) => FieldType::SequenceU32(Box::new(Definition::new(INVALID_VERSION, None))),
        }
    }
}

/// Special serializer function to sort the definitions HashMap before serializing.
fn ordered_map_definitions<S>(value: &HashMap<String, Vec<Definition>>, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer, {
    let ordered: BTreeMap<_, _> = value.iter().collect();
    ordered.serialize(serializer)
}

/// Special serializer function to sort the patches HashMap before serializing.
fn ordered_map_patches<S>(value: &HashMap<String, HashMap<String, HashMap<String, String>>>, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer, {
    let ordered: BTreeMap<_, BTreeMap<_, BTreeMap<_, _>>> = value.iter().map(|(a, x)| (a, x.iter().map(|(b, y)| (b, y.iter().collect())).collect())).collect();
    ordered.serialize(serializer)
}
