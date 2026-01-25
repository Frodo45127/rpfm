//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Table data structures and encoding/decoding for Total War game data.
//!
//! # Overview
//!
//! This module provides the core abstraction for working with structured table data in Total War games.
//! Tables are the primary format for storing game data like units, buildings, technologies, factions,
//! and thousands of other data types. They function similarly to database tables with typed columns
//! and rows of data.
//!
//! # Table Data Flow
//!
//! ```text
//! ┌──────────────┐
//! │  DB File     │  Binary format on disk
//! │  (Header +   │  Contains: version, GUID, entry count
//! │   Table Data)│
//! └──────┬───────┘
//!        │ decode (requires Definition from schema)
//!        ↓
//! ┌──────────────┐
//! │ TableInMemory│  In-memory representation
//! │ (Rows of     │  Vec<Vec<DecodedData>>
//! │  DecodedData)│
//! └──────┬───────┘
//!        │ export/import
//!        ↓
//! ┌──────────────┐
//! │ TSV/SQL/etc  │  Human-readable formats
//! └──────────────┘
//! ```
//!
//! # Key Components
//!
//! ## Table Trait
//!
//! The [`Table`] trait defines the interface for all table-like structures. It provides:
//! - Access to table metadata (name, definition, patches)
//! - Row data access (immutable and mutable)
//! - Column queries and row generation
//! - Data validation and replacement
//!
//! ## DecodedData Enum
//!
//! The [`DecodedData`] enum represents all possible data types in Total War tables:
//! - **Primitives**: Boolean, integers (I16/I32/I64), floats (F32/F64)
//! - **Strings**: UTF-8 (StringU8) and UTF-16 (StringU16)
//! - **Optional types**: OptionalI16/I32/I64, OptionalStringU8/U16
//! - **Special types**: ColourRGB (merged R/G/B values), SequenceU16/U32 (binary blobs)
//!
//! ## TableInMemory
//!
//! The concrete implementation of the `Table` trait for in-memory table manipulation.
//! Supports encoding/decoding to/from binary format, TSV import/export, and SQL operations.
//!
//! # Encoding/Decoding
//!
//! Tables are encoded in a binary format defined by a [`Definition`] from the schema system:
//!
//! 1. **Decoding Process**:
//!    - Read table header (version, GUID, entry count) from DB file
//!    - Use Definition to determine column structure and types
//!    - Decode each row according to field types
//!    - Apply postprocessing (bitwise fields, enums, color merging)
//!
//! 2. **Encoding Process**:
//!    - Apply preprocessing (split colors, encode enums, combine bitwise fields)
//!    - Write each field according to its type
//!    - Update DB file header with entry count
//!
//! # Special Field Types
//!
//! ## Bitwise Fields
//!
//! Integer fields marked as "bitwise" in the definition are split into multiple boolean columns:
//!
//! ```text
//! Binary: single i32 with flags 0b00001011
//! Decoded: multiple boolean columns [true, true, false, true]
//! ```
//!
//! ## Enum Fields
//!
//! Integer fields with enum values defined in the schema show string values:
//!
//! ```text
//! Binary: i32 value 2
//! Decoded: StringU8("cavalry") based on enum mapping
//! ```
//!
//! ## Split Color Fields
//!
//! RGB color components are merged into a single ColourRGB field:
//!
//! ```text
//! Binary: three i32 fields (red=255, green=128, blue=0)
//! Decoded: single ColourRGB("#FF8000")
//! ```
//!
//! # Usage Example
//!
//! ```ignore
//! use rpfm_lib::files::table::{Table, DecodedData};
//! use rpfm_lib::files::table::local::TableInMemory;
//! use rpfm_lib::schema::Definition;
//!
//! // Decode a table
//! let mut table = TableInMemory::decode(&mut reader, &definition, &name, &patches)?;
//!
//! // Access rows
//! for row in table.data().iter() {
//!     if let DecodedData::StringU8(name) = &row[0] {
//!         println!("Row name: {}", name);
//!     }
//! }
//!
//! // Modify data
//! let mut new_row = table.new_row();
//! new_row[0] = DecodedData::StringU8("new_unit".to_string());
//! table.data_mut().push(new_row);
//!
//! // Encode back to binary
//! table.encode(&mut writer)?;
//! ```
//!
//! # Integration
//!
//! This module integrates with:
//! - **Schema module**: Provides [`Definition`] for table structure
//! - **DB module**: Container format for table data with headers
//! - **Binary module**: Low-level I/O operations
//! - **Assembly Kit**: Import/export of official modding tools data
//!
//! # See Also
//!
//! - [`local::TableInMemory`] - Main implementation
//! - [`DecodedData`] - Data type enum
//! - [`crate::files::db`] - DB file container format
//! - [`crate::schema::Definition`] - Table schema definitions

use base64::{Engine, engine::general_purpose::STANDARD};
use float_eq::float_eq;
use serde_derive::{Serialize, Deserialize};

use std::borrow::Cow;
use std::collections::{BTreeMap, HashMap};
use std::hash::{Hash, Hasher};
use std::io::SeekFrom;

use crate::error::{RLibError, Result};
use crate::binary::{ReadBytes, WriteBytes};
use crate::schema::*;
use crate::utils::parse_str_as_bool;

pub mod local;

//---------------------------------------------------------------------------//
//                              Enum & Structs
//---------------------------------------------------------------------------//

/// Abstract interface for table-like data structures.
///
/// This trait defines the common interface for all table implementations in RPFM.
/// Tables are row-based data structures with typed columns defined by a [`Definition`].
///
/// # Thread Safety
///
/// Implementations must be `Send + Sync` to support concurrent access and modification
/// in multi-threaded environments.
///
/// # Implementations
///
/// - [`local::TableInMemory`] - In-memory table with full encode/decode support
///
/// # Usage
///
/// ```ignore
/// fn process_table<T: Table>(table: &T) {
///     // Access metadata
///     let name = table.name();
///     let definition = table.definition();
///
///     // Iterate rows
///     for row in table.data().iter() {
///         // Process each row
///     }
/// }
/// ```
pub trait Table: Send + Sync {

    /// Returns the table name (e.g., "units_tables", "factions_tables").
    fn name(&self) -> &str;

    /// Returns the table's schema definition.
    ///
    /// The definition specifies column structure, types, constraints, and version information.
    fn definition(&self) -> &Definition;

    /// Returns definition patches applied to this table.
    ///
    /// Patches allow runtime modification of definitions for specific tables without
    /// changing the base schema.
    fn patches(&self) -> &DefinitionPatch;

    /// Returns the table's row data.
    ///
    /// Returns a `Cow` to allow zero-copy access for immutable operations while
    /// supporting owned data when needed.
    fn data(&'_ self) -> Cow<'_, [Vec<DecodedData>]>;

    /// Returns a mutable reference to the table's row data.
    ///
    /// # Safety
    ///
    /// Using this method makes you responsible for maintaining data validity:
    /// - Each row must have the correct number of columns
    /// - Each cell must match the type specified in the definition
    /// - Data integrity is not automatically validated
    fn data_mut(&mut self) -> &mut Vec<Vec<DecodedData>>;

    /// Sets the table name.
    fn set_name(&mut self, val: String);

    /// Replaces the table's definition and migrates data to match the new schema.
    ///
    /// This method enables **version migration**: converting table data from one schema
    /// version to another. When the definition changes:
    /// - New columns are added with default values
    /// - Removed columns are dropped
    /// - Type changes are converted where possible
    /// - Data is validated against the new schema
    ///
    /// # Use Cases
    ///
    /// - Updating tables after game patches change the schema
    /// - Converting tables between game versions
    /// - Applying definition modifications from patches
    fn set_definition(&mut self, new_definition: &Definition);

    /// Replaces the table's data with the provided rows.
    ///
    /// # Validation
    ///
    /// This method validates that:
    /// - Each row has the correct number of columns
    /// - Each cell matches the expected type from the definition
    ///
    /// # Errors
    ///
    /// Returns an error if validation fails.
    fn set_data(&mut self, data: &[Vec<DecodedData>]) -> Result<()>;

    /// Returns the column index for a given column name.
    ///
    /// Returns `None` if no column with the specified name exists.
    /// Column names are case-sensitive.
    fn column_position_by_name(&self, column_name: &str) -> Option<usize>;

    /// Returns `true` if the table contains no rows.
    fn is_empty(&self) -> bool;

    /// Returns the number of rows in the table.
    fn len(&self) -> usize;

    /// Creates a new empty row with default values for all columns.
    ///
    /// Default values are determined by:
    /// 1. Field-specific default values from the definition/patches
    /// 2. Type-specific defaults (0 for numbers, empty string for text, false for booleans)
    fn new_row(&self) -> Vec<DecodedData> {
        let definition = self.definition();
        let schema_patches = Some(self.patches());

        definition.fields_processed().iter()
            .map(|field|
                match field.field_type() {
                    FieldType::Boolean => {
                        if let Some(default_value) = field.default_value(schema_patches) {
                            if default_value.to_lowercase() == "true" {
                                DecodedData::Boolean(true)
                            } else {
                                DecodedData::Boolean(false)
                            }
                        } else {
                            DecodedData::Boolean(false)
                        }
                    }
                    FieldType::F32 => {
                        if let Some(default_value) = field.default_value(schema_patches) {
                            if let Ok(default_value) = default_value.parse::<f32>() {
                                DecodedData::F32(default_value)
                            } else {
                                DecodedData::F32(0.0)
                            }
                        } else {
                            DecodedData::F32(0.0)
                        }
                    },
                    FieldType::F64 => {
                        if let Some(default_value) = field.default_value(schema_patches) {
                            if let Ok(default_value) = default_value.parse::<f64>() {
                                DecodedData::F64(default_value)
                            } else {
                                DecodedData::F64(0.0)
                            }
                        } else {
                            DecodedData::F64(0.0)
                        }
                    },
                    FieldType::I16 => {
                        if let Some(default_value) = field.default_value(schema_patches) {
                            if let Ok(default_value) = default_value.parse::<i16>() {
                                DecodedData::I16(default_value)
                            } else {
                                DecodedData::I16(0)
                            }
                        } else {
                            DecodedData::I16(0)
                        }
                    },
                    FieldType::I32 => {
                        if let Some(default_value) = field.default_value(schema_patches) {
                            if let Ok(default_value) = default_value.parse::<i32>() {
                                DecodedData::I32(default_value)
                            } else {
                                DecodedData::I32(0)
                            }
                        } else {
                            DecodedData::I32(0)
                        }
                    },
                    FieldType::I64 => {
                        if let Some(default_value) = field.default_value(schema_patches) {
                            if let Ok(default_value) = default_value.parse::<i64>() {
                                DecodedData::I64(default_value)
                            } else {
                                DecodedData::I64(0)
                            }
                        } else {
                            DecodedData::I64(0)
                        }
                    },

                    FieldType::ColourRGB => {
                        if let Some(default_value) = field.default_value(schema_patches) {
                            if u32::from_str_radix(&default_value, 16).is_ok() {
                                DecodedData::ColourRGB(default_value)
                            } else {
                                DecodedData::ColourRGB("000000".to_owned())
                            }
                        } else {
                            DecodedData::ColourRGB("000000".to_owned())
                        }
                    },
                    FieldType::StringU8 => {
                        if let Some(default_value) = field.default_value(schema_patches) {
                            DecodedData::StringU8(default_value)
                        } else {
                            DecodedData::StringU8(String::new())
                        }
                    }
                    FieldType::StringU16 => {
                        if let Some(default_value) = field.default_value(schema_patches) {
                            DecodedData::StringU16(default_value)
                        } else {
                            DecodedData::StringU16(String::new())
                        }
                    }

                    FieldType::OptionalI16 => {
                        if let Some(default_value) = field.default_value(schema_patches) {
                            if let Ok(default_value) = default_value.parse::<i16>() {
                                DecodedData::OptionalI16(default_value)
                            } else {
                                DecodedData::OptionalI16(0)
                            }
                        } else {
                            DecodedData::OptionalI16(0)
                        }
                    },
                    FieldType::OptionalI32 => {
                        if let Some(default_value) = field.default_value(schema_patches) {
                            if let Ok(default_value) = default_value.parse::<i32>() {
                                DecodedData::OptionalI32(default_value)
                            } else {
                                DecodedData::OptionalI32(0)
                            }
                        } else {
                            DecodedData::OptionalI32(0)
                        }
                    },
                    FieldType::OptionalI64 => {
                        if let Some(default_value) = field.default_value(schema_patches) {
                            if let Ok(default_value) = default_value.parse::<i64>() {
                                DecodedData::OptionalI64(default_value)
                            } else {
                                DecodedData::OptionalI64(0)
                            }
                        } else {
                            DecodedData::OptionalI64(0)
                        }
                    },

                    FieldType::OptionalStringU8 => {
                        if let Some(default_value) = field.default_value(schema_patches) {
                            DecodedData::OptionalStringU8(default_value)
                        } else {
                            DecodedData::OptionalStringU8(String::new())
                        }
                    }
                    FieldType::OptionalStringU16 => {
                        if let Some(default_value) = field.default_value(schema_patches) {
                            DecodedData::OptionalStringU16(default_value)
                        } else {
                            DecodedData::OptionalStringU16(String::new())
                        }
                    },
                    FieldType::SequenceU16(_) => DecodedData::SequenceU16(vec![0, 0]),
                    FieldType::SequenceU32(_) => DecodedData::SequenceU32(vec![0, 0, 0, 0])
                }
            )
            .collect()
    }

    /// This function tries to find all rows with the provided data, if they exists in this table.
    fn rows_containing_data(&self, column_name: &str, data: &str) -> Option<(usize, Vec<usize>)>;
}

/// Runtime representation of table cell data supporting 16 different data types.
///
/// This enum provides a type-safe wrapper around the various data types that can appear
/// in Total War game data tables. Each variant corresponds to a [`FieldType`] from the
/// schema definition.
///
/// # Data Type Categories
///
/// ## Numeric Types
/// - **Boolean**: True/false values
/// - **F32/F64**: Floating-point numbers (32-bit and 64-bit precision)
/// - **I16/I32/I64**: Signed integers (16-bit, 32-bit, 64-bit)
///
/// ## String Types
/// - **StringU8/StringU16**: UTF-8 strings with length prefix (u8 or u16)
/// - **ColourRGB**: Hexadecimal color strings (e.g., "FF0000" for red)
///
/// ## Optional Types
/// - **OptionalI16/I32/I64**: Integer types that can represent "null" via special values
/// - **OptionalStringU8/U16**: String types that can be empty to represent "null"
///
/// ## Sequence Types
/// - **SequenceU16/U32**: Binary-encoded nested table data
///
/// Sequences represent **recursive table structures** - tables embedded within a single
/// cell. The raw bytes encode a complete nested table with its own rows and columns.
/// Used in complex tables like unit models where each row can contain sub-tables of
/// equipment variants or LOD levels.
///
/// # Type Conversion
///
/// The enum provides automatic conversion between compatible types via
/// [`convert_between_types`](DecodedData::convert_between_types):
///
/// - Numeric types convert with casting (may lose precision)
/// - Booleans convert to 0/1 for numbers, "true"/"false" for strings
/// - Strings parse to numbers (returns error if invalid)
/// - Sequences only convert between U16 and U32 variants
///
/// # String Representation
///
/// All variants can be converted to strings via
/// [`data_to_string`](DecodedData::data_to_string):
///
/// - Numbers format with appropriate precision (floats: 4 decimals)
/// - Sequences encode as base64 for display/export
///
/// # Usage Example
///
/// ```rust
/// # use rpfm_lib::schema::FieldType;
/// # use rpfm_lib::files::table::DecodedData;
/// // Create from type and default value
/// let health = DecodedData::new_from_type_and_value(
///     &FieldType::I32,
///     &Some("100".to_string())
/// );
///
/// // Parse from user input
/// let name = DecodedData::new_from_type_and_string(
///     &FieldType::StringU8,
///     "Empire Swordsmen"
/// ).unwrap();
///
/// // Convert between types
/// let health_float = health.convert_between_types(&FieldType::F32).unwrap();
///
/// // Display value
/// println!("Health: {}", health.data_to_string());
/// ```
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum DecodedData {
    /// Boolean true/false value.
    Boolean(bool),

    /// 32-bit floating-point number.
    F32(f32),

    /// 64-bit floating-point number.
    F64(f64),

    /// 16-bit signed integer.
    I16(i16),

    /// 32-bit signed integer.
    I32(i32),

    /// 64-bit signed integer.
    I64(i64),

    /// RGB color as hex string (e.g., "FF0000" for red).
    ColourRGB(String),

    /// UTF-8 string with u16 length prefix.
    StringU8(String),

    /// UTF-16 string with u16 length prefix.
    StringU16(String),

    /// Optional 16-bit signed integer.
    OptionalI16(i16),

    /// Optional 32-bit signed integer.
    OptionalI32(i32),

    /// Optional 64-bit signed integer.
    OptionalI64(i64),

    /// Optional UTF-8 string with u16 length prefix.
    OptionalStringU8(String),

    /// Optional UTF-16 string with u16 length prefix.
    OptionalStringU16(String),

    /// Binary-encoded nested table with u16 entry count.
    SequenceU16(Vec<u8>),

    /// Binary-encoded nested table with u32 entry count.
    SequenceU32(Vec<u8>)
}

//----------------------------------------------------------------//
// Implementations for `DecodedData`.
//----------------------------------------------------------------//

/// Eq and PartialEq implementation of `DecodedData`. We need this implementation due to
/// the float comparison being... special.
impl Eq for DecodedData {}
impl PartialEq for DecodedData {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (DecodedData::Boolean(x), DecodedData::Boolean(y)) => x == y,
            (DecodedData::F32(x), DecodedData::F32(y)) => float_eq!(x, y, abs <= 0.0001),
            (DecodedData::F64(x), DecodedData::F64(y)) => float_eq!(x, y, abs <= 0.0001),
            (DecodedData::I16(x), DecodedData::I16(y)) => x == y,
            (DecodedData::I32(x), DecodedData::I32(y)) => x == y,
            (DecodedData::I64(x), DecodedData::I64(y)) => x == y,
            (DecodedData::ColourRGB(x), DecodedData::ColourRGB(y)) => x == y,
            (DecodedData::StringU8(x), DecodedData::StringU8(y)) => x == y,
            (DecodedData::StringU16(x), DecodedData::StringU16(y)) => x == y,
            (DecodedData::OptionalI16(x), DecodedData::OptionalI16(y)) => x == y,
            (DecodedData::OptionalI32(x), DecodedData::OptionalI32(y)) => x == y,
            (DecodedData::OptionalI64(x), DecodedData::OptionalI64(y)) => x == y,
            (DecodedData::OptionalStringU8(x), DecodedData::OptionalStringU8(y)) => x == y,
            (DecodedData::OptionalStringU16(x), DecodedData::OptionalStringU16(y)) => x == y,
            (DecodedData::SequenceU16(x), DecodedData::SequenceU16(y)) => x == y,
            (DecodedData::SequenceU32(x), DecodedData::SequenceU32(y)) => x == y,
            _ => false
        }
    }
}

impl Hash for DecodedData {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            DecodedData::Boolean(y) => y.hash(state),
            DecodedData::F32(y) => y.to_string().hash(state),
            DecodedData::F64(y) => y.to_string().hash(state),
            DecodedData::I16(y) => y.hash(state),
            DecodedData::I32(y) => y.hash(state),
            DecodedData::I64(y) => y.hash(state),
            DecodedData::ColourRGB(y) => y.hash(state),
            DecodedData::StringU8(y) => y.hash(state),
            DecodedData::StringU16(y) => y.hash(state),
            DecodedData::OptionalI16(y) => y.hash(state),
            DecodedData::OptionalI32(y) => y.hash(state),
            DecodedData::OptionalI64(y) => y.hash(state),
            DecodedData::OptionalStringU8(y) => y.hash(state),
            DecodedData::OptionalStringU16(y) => y.hash(state),
            DecodedData::SequenceU16(y) => y.hash(state),
            DecodedData::SequenceU32(y) => y.hash(state),
        }
    }
}

/// Implementation of `DecodedData`.
impl DecodedData {

    /// Creates a new `DecodedData` of the specified type with an optional default value.
    ///
    /// This function is the primary constructor for creating table cell data. It handles
    /// type-specific initialization and default value parsing.
    ///
    /// # Behavior
    ///
    /// - If `default_value` is `Some`, attempts to parse it according to `field_type`
    /// - If parsing fails or `default_value` is `None`, uses type-specific defaults:
    ///   - Numeric types: 0 or 0.0
    ///   - Booleans: false
    ///   - Strings: empty string
    ///   - Colors: "000000" (black) or "" depending on default_value presence
    ///   - Sequences: minimal valid data ([0, 0] or [0, 0, 0, 0])
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use rpfm_lib::schema::FieldType;
    /// # use rpfm_lib::files::table::DecodedData;
    /// // With default value
    /// let health = DecodedData::new_from_type_and_value(
    ///     &FieldType::I32,
    ///     &Some("100".to_string())
    /// );
    ///
    /// // Without default (uses type default)
    /// let name = DecodedData::new_from_type_and_value(
    ///     &FieldType::StringU8,
    ///     &None
    /// );
    /// ```
    pub fn new_from_type_and_value(field_type: &FieldType, default_value: &Option<String>) -> Self {
        match default_value {
            Some(default_value) => match field_type {
                FieldType::Boolean => if let Ok(value) = parse_str_as_bool(default_value) { DecodedData::Boolean(value) } else { DecodedData::Boolean(false) },
                FieldType::F32 => if let Ok(value) = default_value.parse::<f32>() { DecodedData::F32(value) } else { DecodedData::F32(0.0) },
                FieldType::F64 => if let Ok(value) = default_value.parse::<f64>() { DecodedData::F64(value) } else { DecodedData::F64(0.0) },
                FieldType::I16 => if let Ok(value) = default_value.parse::<i16>() { DecodedData::I16(value) } else { DecodedData::I16(0) },
                FieldType::I32 => if let Ok(value) = default_value.parse::<i32>() { DecodedData::I32(value) } else { DecodedData::I32(0) },
                FieldType::I64 => if let Ok(value) = default_value.parse::<i64>() { DecodedData::I64(value) } else { DecodedData::I64(0) },
                FieldType::ColourRGB => if u32::from_str_radix(default_value, 16).is_ok() {
                    DecodedData::ColourRGB(default_value.to_owned())
                } else {
                    DecodedData::ColourRGB("000000".to_owned())
                },

                FieldType::StringU8 => DecodedData::StringU8(default_value.to_owned()),
                FieldType::StringU16 => DecodedData::StringU16(default_value.to_owned()),
                FieldType::OptionalI16 => if let Ok(value) = default_value.parse::<i16>() { DecodedData::I16(value) } else { DecodedData::I16(0) },
                FieldType::OptionalI32 => if let Ok(value) = default_value.parse::<i32>() { DecodedData::I32(value) } else { DecodedData::I32(0) },
                FieldType::OptionalI64 => if let Ok(value) = default_value.parse::<i64>() { DecodedData::I64(value) } else { DecodedData::I64(0) },
                FieldType::OptionalStringU8 => DecodedData::OptionalStringU8(default_value.to_owned()),
                FieldType::OptionalStringU16 => DecodedData::OptionalStringU16(default_value.to_owned()),

                // For these two ignore the default value.
                FieldType::SequenceU16(_) => DecodedData::SequenceU16(vec![0, 0]),
                FieldType::SequenceU32(_) => DecodedData::SequenceU32(vec![0, 0, 0, 0]),
            }
            None => match field_type {
                FieldType::Boolean => DecodedData::Boolean(false),
                FieldType::F32 => DecodedData::F32(0.0),
                FieldType::F64 => DecodedData::F64(0.0),
                FieldType::I16 => DecodedData::I16(0),
                FieldType::I32 => DecodedData::I32(0),
                FieldType::I64 => DecodedData::I64(0),
                FieldType::ColourRGB => DecodedData::ColourRGB("".to_owned()),
                FieldType::StringU8 => DecodedData::StringU8("".to_owned()),
                FieldType::StringU16 => DecodedData::StringU16("".to_owned()),
                FieldType::OptionalI16 => DecodedData::OptionalI16(0),
                FieldType::OptionalI32 => DecodedData::OptionalI32(0),
                FieldType::OptionalI64 => DecodedData::OptionalI64(0),
                FieldType::OptionalStringU8 => DecodedData::OptionalStringU8("".to_owned()),
                FieldType::OptionalStringU16 => DecodedData::OptionalStringU16("".to_owned()),
                FieldType::SequenceU16(_) => DecodedData::SequenceU16(vec![0, 0]),
                FieldType::SequenceU32(_) => DecodedData::SequenceU32(vec![0, 0, 0, 0]),
            }
        }
    }

    /// Parses a string value into a `DecodedData` of the specified type.
    ///
    /// This function is used when importing data from text formats (TSV, user input, etc.)
    /// and requires strict validation - parsing failures return an error.
    ///
    /// # Parsing Rules
    ///
    /// - **Boolean**: Accepts "true"/"false", "1"/"0", "yes"/"no" (case-insensitive)
    /// - **Numeric**: Standard Rust parsing (scientific notation supported for floats)
    /// - **String/Color**: Direct assignment (no validation for colors)
    /// - **Sequence**: Converts string bytes to `Vec<u8>`
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Numeric value cannot be parsed (invalid format or out of range)
    /// - Boolean value is not recognized
    ///
    /// # Examples
    ///
    /// ```ignore
    /// # use rpfm_lib::schema::FieldType;
    /// # use rpfm_lib::files::table::DecodedData;
    /// // Parse valid values
    /// let damage = DecodedData::new_from_type_and_string(&FieldType::I32, "42")?;
    /// let enabled = DecodedData::new_from_type_and_string(&FieldType::Boolean, "true")?;
    ///
    /// // This returns an error - invalid integer
    /// let invalid = DecodedData::new_from_type_and_string(&FieldType::I32, "not a number");
    /// assert!(invalid.is_err());
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn new_from_type_and_string(field_type: &FieldType, value: &str) -> Result<Self> {
        Ok(match field_type {
            FieldType::Boolean => Self::Boolean(parse_str_as_bool(value)?),
            FieldType::F32 => Self::F32(value.parse::<f32>()?),
            FieldType::F64 => Self::F64(value.parse::<f64>()?),
            FieldType::I16 => Self::I16(value.parse::<i16>()?),
            FieldType::I32 => Self::I32(value.parse::<i32>()?),
            FieldType::I64 => Self::I64(value.parse::<i64>()?),
            FieldType::ColourRGB => Self::ColourRGB(value.to_string()),
            FieldType::StringU8 => Self::StringU8(value.to_string()),
            FieldType::StringU16 => Self::StringU16(value.to_string()),
            FieldType::OptionalI16 => Self::OptionalI16(value.parse::<i16>()?),
            FieldType::OptionalI32 => Self::OptionalI32(value.parse::<i32>()?),
            FieldType::OptionalI64 => Self::OptionalI64(value.parse::<i64>()?),
            FieldType::OptionalStringU8 => Self::OptionalStringU8(value.to_string()),
            FieldType::OptionalStringU16 => Self::OptionalStringU16(value.to_string()),
            FieldType::SequenceU16(_) => Self::SequenceU16(value.as_bytes().to_vec()),
            FieldType::SequenceU32(_) => Self::SequenceU32(value.as_bytes().to_vec()),
        })
    }

    /// Validates that this `DecodedData` matches the expected `FieldType`.
    ///
    /// This is used during table validation to ensure type safety - verifying that
    /// runtime data matches the schema definition.
    ///
    /// # Returns
    ///
    /// - `true` if the variant matches the field type
    /// - `false` if there's a type mismatch
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use rpfm_lib::schema::FieldType;
    /// # use rpfm_lib::files::table::DecodedData;
    /// let data = DecodedData::I32(42);
    ///
    /// assert!(data.is_field_type_correct(&FieldType::I32));
    /// assert!(!data.is_field_type_correct(&FieldType::StringU8));
    /// ```
    pub fn is_field_type_correct(&self, field_type: &FieldType) -> bool {
        match self {
            DecodedData::Boolean(_) => field_type == &FieldType::Boolean,
            DecodedData::F32(_) => field_type == &FieldType::F32,
            DecodedData::F64(_) => field_type == &FieldType::F64,
            DecodedData::I16(_) => field_type == &FieldType::I16,
            DecodedData::I32(_) => field_type == &FieldType::I32,
            DecodedData::I64(_) => field_type == &FieldType::I64,
            DecodedData::ColourRGB(_) => field_type == &FieldType::ColourRGB,
            DecodedData::StringU8(_) => field_type == &FieldType::StringU8,
            DecodedData::StringU16(_) => field_type == &FieldType::StringU16,
            DecodedData::OptionalI16(_) => field_type == &FieldType::OptionalI16,
            DecodedData::OptionalI32(_) => field_type == &FieldType::OptionalI32,
            DecodedData::OptionalI64(_) => field_type == &FieldType::OptionalI64,
            DecodedData::OptionalStringU8(_) => field_type == &FieldType::OptionalStringU8,
            DecodedData::OptionalStringU16(_) => field_type == &FieldType::OptionalStringU16,
            DecodedData::SequenceU16(_) => matches!(field_type, FieldType::SequenceU16(_)),
            DecodedData::SequenceU32(_) => matches!(field_type, FieldType::SequenceU32(_)),
        }
    }

    /// Converts this data to a different `FieldType`, performing automatic type coercion.
    ///
    /// This function enables schema migration and type changes by converting data between
    /// compatible types. Conversion rules vary by type - some are lossless, others may
    /// truncate or fail.
    ///
    /// # Conversion Rules
    ///
    /// ## Numeric Conversions
    /// - **Between numeric types**: Cast with potential precision/range loss
    /// - **To Boolean**: `true` if value >= 1, otherwise `false`
    /// - **To String**: Format with `to_string()`
    ///
    /// ## Boolean Conversions
    /// - **To numeric**: `true` → 1, `false` → 0
    /// - **To string**: "true" or "false"
    /// - **To color**: "FFFFFF" (white) or "000000" (black)
    ///
    /// ## String Conversions
    /// - **To numeric**: Parse string (returns error if invalid)
    /// - **To Boolean**: Parse "true"/"false"/"1"/"0" (returns error if invalid)
    /// - **Between string types**: Direct copy
    ///
    /// ## Sequence Conversions
    /// - **U16 ↔ U32**: Adjust padding bytes to match new entry count size
    /// - **To other types**: Returns default value for target type (data is lost)
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - String cannot be parsed to target numeric/boolean type
    /// - Conversion is fundamentally incompatible
    ///
    /// # Performance Note
    ///
    /// Converting to the same type performs a clone operation. Use `clone()` directly
    /// if you need to copy data without type checking overhead.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// # use rpfm_lib::schema::FieldType;
    /// # use rpfm_lib::files::table::DecodedData;
    /// // Numeric conversions
    /// let int_value = DecodedData::I32(42);
    /// let float_value = int_value.convert_between_types(&FieldType::F32)?;
    ///
    /// // String to number (can fail)
    /// let text = DecodedData::StringU8("123".to_string());
    /// let number = text.convert_between_types(&FieldType::I32)?;
    ///
    /// // Invalid conversion returns error
    /// let invalid = DecodedData::StringU8("not a number".to_string());
    /// assert!(invalid.convert_between_types(&FieldType::I32).is_err());
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn convert_between_types(&self, new_field_type: &FieldType) -> Result<Self> {
        Ok(match self {
            Self::Boolean(ref data) => match new_field_type {
                FieldType::Boolean => self.clone(),
                FieldType::F32 => Self::F32(if *data { 1.0 } else { 0.0 }),
                FieldType::F64 => Self::F64(if *data { 1.0 } else { 0.0 }),
                FieldType::I16 => Self::I16(i16::from(*data)),
                FieldType::I32 => Self::I32(i32::from(*data)),
                FieldType::I64 => Self::I64(i64::from(*data)),
                FieldType::ColourRGB => Self::ColourRGB(if *data { "FFFFFF" } else { "000000" }.to_owned()),
                FieldType::StringU8 => Self::StringU8(data.to_string()),
                FieldType::StringU16 => Self::StringU16(data.to_string()),
                FieldType::OptionalI16 => Self::OptionalI16(i16::from(*data)),
                FieldType::OptionalI32 => Self::OptionalI32(i32::from(*data)),
                FieldType::OptionalI64 => Self::OptionalI64(i64::from(*data)),
                FieldType::OptionalStringU8 => Self::OptionalStringU8(data.to_string()),
                FieldType::OptionalStringU16 => Self::OptionalStringU16(data.to_string()),
                FieldType::SequenceU16(_) => Self::SequenceU16(vec![0, 0]),
                FieldType::SequenceU32(_) => Self::SequenceU32(vec![0, 0, 0, 0]),
            }

            Self::F32(ref data) => match new_field_type {
                FieldType::Boolean => Self::Boolean(data >= &1.0),
                FieldType::F32 => self.clone(),
                FieldType::F64 => Self::F64(*data as f64),
                FieldType::I16 => Self::I16(*data as i16),
                FieldType::I32 => Self::I32(*data as i32),
                FieldType::I64 => Self::I64(*data as i64),
                FieldType::ColourRGB => Self::ColourRGB(data.to_string()),
                FieldType::StringU8 => Self::StringU8(data.to_string()),
                FieldType::StringU16 => Self::StringU16(data.to_string()),
                FieldType::OptionalI16 => Self::OptionalI16(*data as i16),
                FieldType::OptionalI32 => Self::OptionalI32(*data as i32),
                FieldType::OptionalI64 => Self::OptionalI64(*data as i64),
                FieldType::OptionalStringU8 => Self::OptionalStringU8(data.to_string()),
                FieldType::OptionalStringU16 => Self::OptionalStringU16(data.to_string()),
                FieldType::SequenceU16(_) => Self::SequenceU16(vec![0, 0]),
                FieldType::SequenceU32(_) => Self::SequenceU32(vec![0, 0, 0, 0]),
            }

            Self::F64(ref data) => match new_field_type {
                FieldType::Boolean => Self::Boolean(data >= &1.0),
                FieldType::F32 => Self::F32(*data as f32),
                FieldType::F64 => self.clone(),
                FieldType::I16 => Self::I16(*data as i16),
                FieldType::I32 => Self::I32(*data as i32),
                FieldType::I64 => Self::I64(*data as i64),
                FieldType::ColourRGB => Self::ColourRGB(data.to_string()),
                FieldType::StringU8 => Self::StringU8(data.to_string()),
                FieldType::StringU16 => Self::StringU16(data.to_string()),
                FieldType::OptionalI16 => Self::OptionalI16(*data as i16),
                FieldType::OptionalI32 => Self::OptionalI32(*data as i32),
                FieldType::OptionalI64 => Self::OptionalI64(*data as i64),
                FieldType::OptionalStringU8 => Self::OptionalStringU8(data.to_string()),
                FieldType::OptionalStringU16 => Self::OptionalStringU16(data.to_string()),
                FieldType::SequenceU16(_) => Self::SequenceU16(vec![0, 0]),
                FieldType::SequenceU32(_) => Self::SequenceU32(vec![0, 0, 0, 0]),
            }

            Self::OptionalI16(ref data) |
            Self::I16(ref data) => match new_field_type {
                FieldType::Boolean => Self::Boolean(data >= &1),
                FieldType::F32 => Self::F32(*data as f32),
                FieldType::F64 => Self::F64(*data as f64),
                FieldType::I16 => self.clone(),
                FieldType::I32 => Self::I32(*data as i32),
                FieldType::I64 => Self::I64(*data as i64),
                FieldType::ColourRGB => Self::ColourRGB(data.to_string()),
                FieldType::StringU8 => Self::StringU8(data.to_string()),
                FieldType::StringU16 => Self::StringU16(data.to_string()),
                FieldType::OptionalI16 => Self::OptionalI16(*data),
                FieldType::OptionalI32 => Self::OptionalI32(*data as i32),
                FieldType::OptionalI64 => Self::OptionalI64(*data as i64),
                FieldType::OptionalStringU8 => Self::OptionalStringU8(data.to_string()),
                FieldType::OptionalStringU16 => Self::OptionalStringU16(data.to_string()),
                FieldType::SequenceU16(_) => Self::SequenceU16(vec![0, 0]),
                FieldType::SequenceU32(_) => Self::SequenceU32(vec![0, 0, 0, 0]),
            }

            Self::OptionalI32(ref data) |
            Self::I32(ref data) => match new_field_type {
                FieldType::Boolean => Self::Boolean(data >= &1),
                FieldType::F32 => Self::F32(*data as f32),
                FieldType::F64 => Self::F64(*data as f64),
                FieldType::I16 => Self::I16(*data as i16),
                FieldType::I32 => self.clone(),
                FieldType::I64 => Self::I64(*data as i64),
                FieldType::ColourRGB => Self::ColourRGB(data.to_string()),
                FieldType::StringU8 => Self::StringU8(data.to_string()),
                FieldType::StringU16 => Self::StringU16(data.to_string()),
                FieldType::OptionalI16 => Self::OptionalI16(*data as i16),
                FieldType::OptionalI32 => Self::OptionalI32(*data),
                FieldType::OptionalI64 => Self::OptionalI64(*data as i64),
                FieldType::OptionalStringU8 => Self::OptionalStringU8(data.to_string()),
                FieldType::OptionalStringU16 => Self::OptionalStringU16(data.to_string()),
                FieldType::SequenceU16(_) => Self::SequenceU16(vec![0, 0]),
                FieldType::SequenceU32(_) => Self::SequenceU32(vec![0, 0, 0, 0]),
            }

            Self::OptionalI64(ref data) |
            Self::I64(ref data) => match new_field_type {
                FieldType::Boolean => Self::Boolean(data >= &1),
                FieldType::F32 => Self::F32(*data as f32),
                FieldType::F64 => Self::F64(*data as f64),
                FieldType::I16 => Self::I16(*data as i16),
                FieldType::I32 => Self::I32(*data as i32),
                FieldType::I64 => self.clone(),
                FieldType::ColourRGB => Self::ColourRGB(data.to_string()),
                FieldType::StringU8 => Self::StringU8(data.to_string()),
                FieldType::StringU16 => Self::StringU16(data.to_string()),
                FieldType::OptionalI16 => Self::OptionalI16(*data as i16),
                FieldType::OptionalI32 => Self::OptionalI32(*data as i32),
                FieldType::OptionalI64 => Self::OptionalI64(*data),
                FieldType::OptionalStringU8 => Self::OptionalStringU8(data.to_string()),
                FieldType::OptionalStringU16 => Self::OptionalStringU16(data.to_string()),
                FieldType::SequenceU16(_) => Self::SequenceU16(vec![0, 0]),
                FieldType::SequenceU32(_) => Self::SequenceU32(vec![0, 0, 0, 0]),
            }

            Self::ColourRGB(ref data) |
            Self::StringU8(ref data) |
            Self::StringU16(ref data) |
            Self::OptionalStringU8(ref data) |
            Self::OptionalStringU16(ref data) => match new_field_type {
                FieldType::Boolean => Self::Boolean(parse_str_as_bool(data)?),
                FieldType::F32 => Self::F32(data.parse::<f32>()?),
                FieldType::F64 => Self::F64(data.parse::<f64>()?),
                FieldType::I16 => Self::I16(data.parse::<i16>()?),
                FieldType::I32 => Self::I32(data.parse::<i32>()?),
                FieldType::I64 => Self::I64(data.parse::<i64>()?),
                FieldType::ColourRGB => Self::ColourRGB(data.to_string()),
                FieldType::StringU8 => Self::StringU8(data.to_string()),
                FieldType::StringU16 => Self::StringU16(data.to_string()),
                FieldType::OptionalI16 => Self::OptionalI16(data.parse::<i16>()?),
                FieldType::OptionalI32 => Self::OptionalI32(data.parse::<i32>()?),
                FieldType::OptionalI64 => Self::OptionalI64(data.parse::<i64>()?),
                FieldType::OptionalStringU8 => Self::OptionalStringU8(data.to_string()),
                FieldType::OptionalStringU16 => Self::OptionalStringU16(data.to_string()),
                FieldType::SequenceU16(_) => Self::SequenceU16(vec![0, 0]),
                FieldType::SequenceU32(_) => Self::SequenceU32(vec![0, 0, 0, 0]),
            }

            Self::SequenceU16(data) => match new_field_type {
                FieldType::SequenceU16(_) => Self::SequenceU16(data.to_vec()),
                FieldType::SequenceU32(_) => Self::SequenceU32({
                    let mut vec = vec![];
                    vec.extend_from_slice(&data[0..2]);
                    vec.extend_from_slice(&[0, 0]);
                    vec.extend_from_slice(&data[2..]);
                    vec
                }),
                _ => Self::new_from_type_and_value(new_field_type, &None),
            }
            Self::SequenceU32(data) => match new_field_type {
                FieldType::SequenceU16(_) => Self::SequenceU16({
                    let mut vec = data[0..2].to_vec();
                    vec.extend_from_slice(&data[4..]);
                    vec
                }),
                FieldType::SequenceU32(_) => Self::SequenceU32(data.to_vec()),
                _ => Self::new_from_type_and_value(new_field_type, &None),
            }
        })
    }

    /// Converts the data to a human-readable string representation.
    ///
    /// This function formats data for display in UI, export to text formats (TSV),
    /// or logging. The formatting is optimized for readability rather than
    /// round-trip serialization.
    ///
    /// # Formatting Rules
    ///
    /// - **Boolean**: "true" or "false"
    /// - **Floats (F32/F64)**: Fixed 4 decimal places (e.g., "3.1416")
    /// - **Integers**: Decimal format (e.g., "42")
    /// - **Strings/Colors**: Direct output
    /// - **Sequences**: Base64-encoded binary data
    ///
    /// # Performance
    ///
    /// Returns `Cow<str>` to avoid allocation for string types (zero-copy),
    /// while formatting numeric types into owned strings only when needed.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use rpfm_lib::files::table::DecodedData;
    /// assert_eq!(DecodedData::Boolean(true).data_to_string(), "true");
    /// assert_eq!(DecodedData::I32(42).data_to_string(), "42");
    /// assert_eq!(DecodedData::F32(3.14159).data_to_string(), "3.1416");
    /// ```
    pub fn data_to_string(&'_ self) -> Cow<'_, str> {
        match self {
            DecodedData::Boolean(data) => Cow::from(if *data { "true" } else { "false" }),
            DecodedData::F32(data) => Cow::from(format!("{data:.4}")),
            DecodedData::F64(data) => Cow::from(format!("{data:.4}")),
            DecodedData::I16(data) => Cow::from(data.to_string()),
            DecodedData::I32(data) => Cow::from(data.to_string()),
            DecodedData::I64(data) => Cow::from(data.to_string()),
            DecodedData::OptionalI16(data) => Cow::from(data.to_string()),
            DecodedData::OptionalI32(data) => Cow::from(data.to_string()),
            DecodedData::OptionalI64(data) => Cow::from(data.to_string()),
            DecodedData::ColourRGB(data) |
            DecodedData::StringU8(data) |
            DecodedData::StringU16(data) |
            DecodedData::OptionalStringU8(data) |
            DecodedData::OptionalStringU16(data) => Cow::from(data),
            DecodedData::SequenceU16(data) |
            DecodedData::SequenceU32(data) => Cow::from(STANDARD.encode(data)),
        }
    }

    /// Updates the value while preserving the current type.
    ///
    /// This method parses the provided string and updates the internal value,
    /// maintaining the same `DecodedData` variant. Used for in-place editing
    /// in table cells.
    ///
    /// # Parsing
    ///
    /// - **Numeric types**: Parse from string (scientific notation supported for floats)
    /// - **Boolean**: Parse "true"/"false"/"1"/"0"/"yes"/"no" (case-insensitive)
    /// - **String/Color**: Direct assignment (no validation)
    /// - **Sequence**: Converts string bytes to `Vec<u8>`
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Numeric value cannot be parsed (invalid format or out of range)
    /// - Boolean value is not recognized
    ///
    /// # Examples
    ///
    /// ```ignore
    /// # use rpfm_lib::files::table::DecodedData;
    /// let mut health = DecodedData::I32(100);
    /// health.set_data("150")?;
    /// assert_eq!(health.data_to_string(), "150");
    ///
    /// // Type is preserved - this fails because "abc" isn't a valid integer
    /// let result = health.set_data("abc");
    /// assert!(result.is_err());
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn set_data(&mut self, new_data: &str) -> Result<()> {
        match self {
            Self::Boolean(data) => *data = parse_str_as_bool(new_data)?,
            Self::F32(data) => *data = new_data.parse::<f32>()?,
            Self::F64(data) => *data = new_data.parse::<f64>()?,
            Self::I16(data) => *data = new_data.parse::<i16>()?,
            Self::I32(data) => *data = new_data.parse::<i32>()?,
            Self::I64(data) => *data = new_data.parse::<i64>()?,
            Self::ColourRGB(data) => *data = new_data.to_string(),
            Self::StringU8(data) => *data = new_data.to_string(),
            Self::StringU16(data) => *data = new_data.to_string(),
            Self::OptionalI16(data) => *data = new_data.parse::<i16>()?,
            Self::OptionalI32(data) => *data = new_data.parse::<i32>()?,
            Self::OptionalI64(data) => *data = new_data.parse::<i64>()?,
            Self::OptionalStringU8(data) => *data = new_data.to_string(),
            Self::OptionalStringU16(data) => *data = new_data.to_string(),
            Self::SequenceU16(data) => *data = new_data.as_bytes().to_vec(),
            Self::SequenceU32(data) => *data = new_data.as_bytes().to_vec(),
        };

        Ok(())
    }
}

//----------------------------------------------------------------//
// Util functions for tables.
//----------------------------------------------------------------//

/// Escapes newlines and tabs for TSV export.
///
/// Converts literal newline (`\n`) and tab (`\t`) characters to their escaped
/// representations (`\\n` and `\\t`) to prevent breaking TSV file structure.
///
/// # Performance
///
/// Only allocates/modifies if the string actually contains newlines or tabs,
/// using `memchr` for fast scanning. This optimization is critical because
/// this function is called for every string cell during table encoding.
///
/// # In-place Modification
///
/// The string is modified in-place to avoid unnecessary allocations.
fn escape_special_chars(data: &mut String) {

    // When performed on mass, this takes 25% of the time to decode a table. Only do it if we really have characters to replace.
    if memchr::memchr(b'\n', data.as_bytes()).is_some() || memchr::memchr(b'\t', data.as_bytes()).is_some() {
        let mut output = Vec::with_capacity(data.len() + 10);
        for c in data.bytes() {
            match c {
                b'\n' => output.extend_from_slice(b"\\\\n"),
                b'\t' => output.extend_from_slice(b"\\\\t"),
                _ => output.push(c),
            }
        }

        unsafe { *data.as_mut_vec() = output };
    }
}

/// Unescapes newlines and tabs from TSV import.
///
/// Converts escaped representations (`\\n` and `\\t`) back to their literal
/// characters (`\n` and `\t`). This is the inverse of [`escape_special_chars`].
///
/// # Returns
///
/// A new `String` with escape sequences replaced by actual whitespace characters.
fn unescape_special_chars(data: &str) -> String {
    data.replace("\\\\t", "\t").replace("\\\\n", "\n")
}

//----------------------------------------------------------------//
// Decoding and encoding functions for tables.
//----------------------------------------------------------------//

/// Decodes a binary table into a vector of rows, each containing decoded field data.
///
/// This function reads binary data according to the provided table definition and converts
/// it into structured `DecodedData` values. It handles both root tables (with externally
/// provided entry count) and nested tables (where entry count is read from the data).
///
/// # Arguments
///
/// * `data` - Binary data source implementing `ReadBytes`.
/// * `definition` - Schema definition describing the table structure.
/// * `entry_count` - Entry count for root tables, or `None` for nested tables (reads u32 from data).
/// * `return_incomplete` - If `true`, returns partially decoded rows on field errors.
/// * `altered_flag` - Set to `true` if any data was modified during decoding.
///
/// # Returns
///
/// A vector of rows, where each row is a vector of `DecodedData` values.
pub(crate) fn decode_table<R: ReadBytes>(data: &mut R, definition: &Definition, entry_count: Option<u32>, return_incomplete: bool, altered_flag: &mut bool) -> Result<Vec<Vec<DecodedData>>> {

    // If we received an entry count, it's the root table. If not, it's a nested one.
    let entry_count = match entry_count {
        Some(entry_count) => entry_count,
        None => data.read_u32()?,
    };

    // Do not specify size here, because a badly written definition can end up triggering an OOM crash if we do.
    let fields = definition.fields();
    let mut table = if entry_count < 10_000 { Vec::with_capacity(entry_count as usize) } else { vec![] };

    for row in 0..entry_count {
        table.push(decode_row(data, fields, row, return_incomplete, &Some(definition.patches()), altered_flag)?);
    }

    Ok(table)
}

/// Decodes a single row of table data from binary format.
///
/// Iterates through each field in the definition, decoding the binary data and applying
/// any necessary postprocessing (bitwise expansion, enum conversion, colour merging).
///
/// # Arguments
///
/// * `data` - Binary data source.
/// * `fields` - Field definitions for this row.
/// * `row` - Current row index (0-based, used for error messages).
/// * `return_incomplete` - If `true`, returns partial row data on field decode errors.
/// * `patches` - Optional definition patches to apply.
/// * `altered_flag` - Set to `true` if data was modified during decoding.
fn decode_row<R: ReadBytes>(data: &mut R, fields: &[Field], row: u32, return_incomplete: bool, patches: &Option<&DefinitionPatch>, altered_flag: &mut bool) -> Result<Vec<DecodedData>> {
    let mut split_colours: BTreeMap<u8, HashMap<String, u8>> = BTreeMap::new();
    let mut row_data = Vec::with_capacity(fields.len());
    for (column, field) in fields.iter().enumerate() {

        // Decode the field, then apply any postprocess operation we need.
        let column = column as u32;
        let field_data = match decode_field(data, field, row, column, altered_flag) {
            Ok(data) => data,
            Err(error) => {
                if return_incomplete {
                    return Ok(row_data);
                } else {
                    return Err(error);
                }
            }
        };
        decode_field_postprocess(&mut row_data, field_data, field, &mut split_colours, patches, altered_flag)
    }

    decode_row_postprocess(&mut row_data, &mut split_colours)?;

    Ok(row_data)
}

/// Decodes a single field from binary data based on its type.
///
/// Reads the appropriate number of bytes from the data source and converts them
/// to the corresponding `DecodedData` variant. String fields have special characters escaped.
///
/// # Arguments
///
/// * `data` - Binary data source.
/// * `field` - Field definition specifying the type to decode.
/// * `row` - Current row index (1-based in error messages).
/// * `column` - Current column index (1-based in error messages).
/// * `altered_flag` - Set to `true` if data was modified during decoding.
///
/// # Errors
///
/// Returns `RLibError::DecodingTableFieldError` if the field cannot be decoded,
/// or `RLibError::DecodingTableFieldSequenceDataError` for sequence field errors.
fn decode_field<R: ReadBytes>(data: &mut R, field: &Field, row: u32, column: u32, altered_flag: &mut bool) -> Result<DecodedData> {
    match field.field_type() {
        FieldType::Boolean => {
            data.read_bool()
                .map(DecodedData::Boolean)
                .map_err(|_| RLibError::DecodingTableFieldError(row + 1, column + 1, "Boolean".to_string()))
        }
        FieldType::F32 => {
            if let Ok(data) = data.read_f32() { Ok(DecodedData::F32(data)) }
            else { Err(RLibError::DecodingTableFieldError(row + 1, column + 1, "F32".to_string())) }
        }
        FieldType::F64 => {
            if let Ok(data) = data.read_f64() { Ok(DecodedData::F64(data)) }
            else { Err(RLibError::DecodingTableFieldError(row + 1, column + 1, "F64".to_string())) }
        }
        FieldType::I16 => {
            if let Ok(data) = data.read_i16() { Ok(DecodedData::I16(data))  }
            else { Err(RLibError::DecodingTableFieldError(row + 1, column + 1, "I16".to_string())) }
        }
        FieldType::I32 => {
            if let Ok(data) = data.read_i32() { Ok(DecodedData::I32(data)) }
            else { Err(RLibError::DecodingTableFieldError(row + 1, column + 1, "I32".to_string())) }
        }
        FieldType::I64 => {
            if let Ok(data) = data.read_i64() { Ok(DecodedData::I64(data)) }
            else { Err(RLibError::DecodingTableFieldError(row + 1, column + 1, "I64".to_string())) }
        }
        FieldType::ColourRGB => {
            if let Ok(data) = data.read_string_colour_rgb() { Ok(DecodedData::ColourRGB(data)) }
            else { Err(RLibError::DecodingTableFieldError(row + 1, column + 1, "Colour RGB".to_string())) }
        }
        FieldType::StringU8 => {
            if let Ok(mut data) = data.read_sized_string_u8() {
                escape_special_chars(&mut data);
                Ok(DecodedData::StringU8(data)) }
            else { Err(RLibError::DecodingTableFieldError(row + 1, column + 1, "UTF-8 String".to_string())) }
        }
        FieldType::StringU16 => {
            if let Ok(mut data) = data.read_sized_string_u16() {
                escape_special_chars(&mut data);
                Ok(DecodedData::StringU16(data)) }
            else { Err(RLibError::DecodingTableFieldError(row + 1, column + 1, "UTF-16 String".to_string())) }
        }
        FieldType::OptionalI16 => {
            if let Ok(data) = data.read_optional_i16() { Ok(DecodedData::OptionalI16(data)) }
            else { Err(RLibError::DecodingTableFieldError(row + 1, column + 1, "Optional I16".to_string())) }
        }
        FieldType::OptionalI32 => {
            if let Ok(data) = data.read_optional_i32() { Ok(DecodedData::OptionalI32(data)) }
            else { Err(RLibError::DecodingTableFieldError(row + 1, column + 1, "Optional I32".to_string())) }
        }
        FieldType::OptionalI64 => {
            if let Ok(data) = data.read_optional_i64() { Ok(DecodedData::OptionalI64(data)) }
            else { Err(RLibError::DecodingTableFieldError(row + 1, column + 1, "Optional I64".to_string())) }
        }

        FieldType::OptionalStringU8 => {
            if let Ok(mut data) = data.read_optional_string_u8() {
                escape_special_chars(&mut data);
                Ok(DecodedData::OptionalStringU8(data)) }
            else { Err(RLibError::DecodingTableFieldError(row + 1, column + 1, "Optional UTF-8 String".to_string())) }
        }
        FieldType::OptionalStringU16 => {
            if let Ok(mut data) = data.read_optional_string_u16() {
                escape_special_chars(&mut data);
                Ok(DecodedData::OptionalStringU16(data)) }
            else { Err(RLibError::DecodingTableFieldError(row + 1, column + 1, "Optional UTF-16 String".to_string())) }
        }

        FieldType::SequenceU16(definition) => {
            let start = data.stream_position()?;
            let entry_count = data.read_u16()?;
            match decode_table(data, definition, Some(entry_count as u32), false, altered_flag) {
                Ok(_) => {
                    let end = data.stream_position()? - start;
                    data.seek(SeekFrom::Start(start))?;
                    let blob = data.read_slice(end as usize, false)?;
                    Ok(DecodedData::SequenceU16(blob))
                }
                Err(error) => Err(RLibError::DecodingTableFieldSequenceDataError(row + 1, column + 1, error.to_string(), "SequenceU16".to_string()))
            }
        }

        FieldType::SequenceU32(definition) => {
            let start = data.stream_position()?;
            let entry_count = data.read_u32()?;
            match decode_table(data, definition, Some(entry_count), false, altered_flag) {
                Ok(_) => {
                    let end = data.stream_position()? - start;
                    data.seek(SeekFrom::Start(start))?;
                    let blob = data.read_slice(end as usize, false)?;
                    Ok(DecodedData::SequenceU32(blob))
                }
                Err(error) => Err(RLibError::DecodingTableFieldSequenceDataError(row + 1, column + 1, error.to_string(), "SequenceU32".to_string()))
            }
        }
    }
}

/// Applies row-level postprocessing after all fields have been decoded.
///
/// Merges split colour fields (r/g/b components stored as separate numeric fields)
/// into combined `ColourRGB` values. The merged colours are appended to the row data.
///
/// # Arguments
///
/// * `row_data` - The decoded row data to append merged colours to.
/// * `split_colours` - Map of colour group indices to their channel values (r/g/b).
///
/// # Errors
///
/// Returns `RLibError::DecodingTableCombinedColour` if colour values cannot be parsed.
fn decode_row_postprocess(row_data: &mut Vec<DecodedData>, split_colours: &mut BTreeMap<u8, HashMap<String, u8>>) -> Result<()> {
    for split_colour in split_colours.values() {
        let mut colour_hex = "".to_owned();
        if let Some(r) = split_colour.get("r") {
            colour_hex.push_str(&format!("{r:02X?}"));
        }

        if let Some(r) = split_colour.get("red") {
            colour_hex.push_str(&format!("{r:02X?}"));
        }

        if let Some(g) = split_colour.get("g") {
            colour_hex.push_str(&format!("{g:02X?}"));
        }

        if let Some(g) = split_colour.get("green") {
            colour_hex.push_str(&format!("{g:02X?}"));
        }

        if let Some(b) = split_colour.get("b") {
            colour_hex.push_str(&format!("{b:02X?}"));
        }

        if let Some(b) = split_colour.get("blue") {
            colour_hex.push_str(&format!("{b:02X?}"));
        }

        if u32::from_str_radix(&colour_hex, 16).is_ok() {
            row_data.push(DecodedData::ColourRGB(colour_hex));
        } else {
            return Err(RLibError::DecodingTableCombinedColour);
        }
    }

    Ok(())
}

/// Applies field-level postprocessing after decoding a single field.
///
/// Handles special field transformations:
/// - **Bitwise fields**: Expands integer into multiple boolean values based on bit positions.
/// - **Enum fields**: Converts integer values to their string representations.
/// - **Split colour fields**: Collects r/g/b components for later merging into `ColourRGB`.
/// - **Normal fields**: Adds the decoded data directly to the row.
///
/// # Arguments
///
/// * `row_data` - The row being built, receives the processed field data.
/// * `data` - The decoded field data to process.
/// * `field` - Field definition with metadata (bitwise count, enum values, colour group).
/// * `split_colours` - Accumulator for split colour field components.
/// * `_patches` - Definition patches (currently unused).
/// * `_altered_flag` - Modification flag (currently unused).
fn decode_field_postprocess(row_data: &mut Vec<DecodedData>, data: DecodedData, field: &Field, split_colours: &mut BTreeMap<u8, HashMap<String, u8>>, _patches: &Option<&DefinitionPatch>, _altered_flag: &mut bool) {

    // If the field is a bitwise, split it into multiple fields. This is currently limited to integer types.
    if field.is_bitwise() > 1 {
        if [FieldType::I16, FieldType::I32, FieldType::I64].contains(field.field_type()) {
            let data = match data {
                DecodedData::I16(ref data) => *data as i64,
                DecodedData::I32(ref data) => *data as i64,
                DecodedData::I64(ref data) => *data,
                _ => unimplemented!()
            };

            for bitwise_column in 0..field.is_bitwise() {
                row_data.push(DecodedData::Boolean(data & (1 << bitwise_column) != 0));
            }
        }
    }

    // If the field has enum values, we turn it into a string. Same as before, only for integer types.
    else if !field.enum_values().is_empty() {
        if [FieldType::I16, FieldType::I32, FieldType::I64].contains(field.field_type()) {
            let data = match data {
                DecodedData::I16(ref data) => *data as i32,
                DecodedData::I32(ref data) => *data,
                DecodedData::I64(ref data) => *data as i32,
                _ => unimplemented!()
            };
            match field.enum_values().get(&data) {
                Some(data) => row_data.push(DecodedData::StringU8(data.to_owned())),
                None => row_data.push(DecodedData::StringU8(data.to_string()))
            }
        }
    }

    // If the field is part of an split colour field group, don't add it. We'll separate it from the rest, then merge them into a ColourRGB field.
    else if let Some(colour_index) = field.is_part_of_colour() {
        if [FieldType::I16, FieldType::I32, FieldType::I64, FieldType::F32, FieldType::F64].contains(field.field_type()) {
            let data = match data {
                DecodedData::I16(ref data) => *data as u8,
                DecodedData::I32(ref data) => *data as u8,
                DecodedData::I64(ref data) => *data as u8,
                DecodedData::F32(ref data) => *data as u8,
                DecodedData::F64(ref data) => *data as u8,
                _ => unimplemented!()
            };

            // This can be r, g, b, red, green, blue.
            let colour_split = field.name().rsplitn(2, '_').collect::<Vec<&str>>();
            let colour_channel = colour_split[0].to_lowercase();
            match split_colours.get_mut(&colour_index) {
                Some(colour_pack) => {
                    colour_pack.insert(colour_channel, data);
                }
                None => {
                    let mut colour_pack = HashMap::new();
                    colour_pack.insert(colour_channel, data);
                    split_colours.insert(colour_index, colour_pack);
                }
            }
        }
    }
    /*
    // Numeric fields are processed as i32. We need to write them back into their original type here.
    else if field.is_numeric(*patches) {
        let data = match data {
            DecodedData::I64(ref data) |
            DecodedData::OptionalI64(ref data) => *data as i32,
            DecodedData::StringU8(ref data) |
            DecodedData::StringU16(ref data) |
            DecodedData::OptionalStringU8(ref data) |
            DecodedData::OptionalStringU16(ref data) => match data.parse::<i32>() {
                Ok(data) => data,
                Err(_) => {

                    // For what I could see, this happens when loading a table that has invalid data on the key field,
                    // which can happen accidentally when loading tables not intended for the game the definition is for,
                    // or regularly on tables with incorrectly inputted data. In those cases, we turn the value to 87654321
                    // and flag the table so it's know this has happened.
                    *altered_flag |= true;
                    87654321
                }
            },
            _ => unimplemented!()
        };

        row_data.push(DecodedData::I32(data));
    }*/

    else {
        row_data.push(data);
    }
}

/// Encodes table data from decoded format back to binary.
///
/// Converts structured `DecodedData` rows back into binary format according to the
/// table definition. Handles special cases like bitwise fields, enum conversions,
/// and split colour fields.
///
/// # Arguments
///
/// * `entries` - Table rows to encode, each row is a vector of `DecodedData`.
/// * `data` - Output buffer implementing `WriteBytes`.
/// * `definition` - Schema definition describing the table structure.
/// * `patches` - Optional definition patches to apply.
///
/// # Special Handling
///
/// - **Split colours**: Extracts r/g/b values from merged `ColourRGB` fields.
/// - **Bitwise fields**: Combines consecutive boolean values into a single integer.
/// - **Enum fields**: Converts string values back to their integer keys.
/// - **Strings**: Unescapes special characters before writing.
///
/// # Errors
///
/// Returns an error if:
/// - Row has wrong number of fields.
/// - Field type doesn't match expected type for the column.
pub fn encode_table<W: WriteBytes>(entries: &[Vec<DecodedData>], data: &mut W, definition: &Definition, patches: &Option<&DefinitionPatch>) -> Result<()> {

    // Get the table data in local format, no matter in what backend we stored it.
    let fields = definition.fields();
    let fields_processed = definition.fields_processed();

    // Get the colour positions of the tables, if any.
    let combined_colour_positions = fields.iter().filter_map(|field| {
        if let Some(colour_group) = field.is_part_of_colour() {
            let colour_split = field.name().rsplitn(2, '_').collect::<Vec<&str>>();
            let colour_field_name: String = if colour_split.len() == 2 { format!("{}{}", colour_split[1].to_lowercase(), MERGE_COLOUR_POST) } else { format!("{}_{}", MERGE_COLOUR_NO_NAME.to_lowercase(), colour_group) };

            definition.column_position_by_name(&colour_field_name).map(|x| (colour_field_name, x))
        } else { None }
    }).collect::<HashMap<String, usize>>();

    for row in entries.iter() {

        // First, we need to make sure we have the amount of fields we expected to be in the row.
        if row.len() != fields_processed.len() {
            return Err(RLibError::TableRowWrongFieldCount(fields_processed.len(), row.len()))
        }

        // The way we process it is, we iterate over the definition fields (because it's what we need to write)
        // and write the fields getting only what we need from the table data.
        let mut data_column = 0;
        for field in fields {

            // First special situation: join back split colour columns, if the field is a split colour.
            if let Some(colour_group) = field.is_part_of_colour() {
                let field_name = field.name().to_lowercase();
                let colour_split = field_name.rsplitn(2, '_').collect::<Vec<&str>>();
                let colour_channel = colour_split[0];
                let colour_field_name = if colour_split.len() == 2 {
                    format!("{}{}", colour_split[1], MERGE_COLOUR_POST)
                } else {
                    format!("{}_{}", MERGE_COLOUR_NO_NAME.to_lowercase(), colour_group)
                };

                if let Some(data_column) = combined_colour_positions.get(&colour_field_name) {
                    match &row[*data_column] {
                        DecodedData::ColourRGB(field_data) => {

                            // Encode the full colour, then grab the byte of our field.
                            let mut encoded = vec![];
                            encoded.write_string_colour_rgb(field_data)?;

                            let field_data =
                                if colour_channel == "r" || colour_channel == "red" { encoded[2] }
                                else if colour_channel == "g" || colour_channel == "green" { encoded[1] }
                                else if colour_channel == "b" || colour_channel == "blue" { encoded[0] }
                            else { 0 };

                            // Only these types can be split colours.
                            match field.field_type() {
                                FieldType::I16 => data.write_i16(field_data as i16)?,
                                FieldType::I32 => data.write_i32(field_data as i32)?,
                                FieldType::I64 => data.write_i64(field_data as i64)?,
                                FieldType::F32 => data.write_f32(field_data as f32)?,
                                FieldType::F64 => data.write_f64(field_data as f64)?,
                                _ => return Err(RLibError::EncodingTableWrongFieldType(FieldType::from(&row[*data_column]).to_string(), field.field_type().to_string()))
                            }


                        },
                        _ => return Err(RLibError::EncodingTableWrongFieldType(FieldType::from(&row[*data_column]).to_string(), field.field_type().to_string()))
                    }
                }
            }

            // Second special situation: bitwise columns.
            else if field.is_bitwise() > 1 {
                let mut field_data: i64 = 0;

                // Bitwise columns are always consecutive booleans.
                for bitwise_column in 0..field.is_bitwise() {
                    if let DecodedData::Boolean(boolean) = row[data_column] {
                        if boolean {
                            field_data |= 1 << bitwise_column;
                        }
                    }

                    else {
                        return Err(RLibError::EncodingTableWrongFieldType(FieldType::from(&row[data_column]).to_string(), field.field_type().to_string()))
                    }

                    data_column += 1;
                }

                // Only integer types can be bitwise.
                match field.field_type() {
                    FieldType::I16 => data.write_i16(field_data as i16)?,
                    FieldType::I32 => data.write_i32(field_data as i32)?,
                    FieldType::I64 => data.write_i64(field_data)?,
                    _ => return Err(RLibError::EncodingTableWrongFieldType(FieldType::from(&row[data_column]).to_string(), field.field_type().to_string()))
                }
            }
            /*
            // Numeric fields are processed as i32. We need to write them back into their original type here.
            else if field.is_numeric(*patches) {
                match &row[data_column] {
                    DecodedData::I32(field_data) => {
                        match field.field_type() {
                            FieldType::I64 => data.write_i64(*field_data as i64)?,
                            FieldType::OptionalI64 => {
                                data.write_bool(true)?;
                                data.write_i64(*field_data as i64)?;
                            },
                            FieldType::StringU8 => data.write_sized_string_u8(&field_data.to_string())?,
                            FieldType::StringU16 => data.write_sized_string_u16(&field_data.to_string())?,
                            FieldType::OptionalStringU8 => data.write_optional_string_u8(&field_data.to_string())?,
                            FieldType::OptionalStringU16 => data.write_optional_string_u16(&field_data.to_string())?,
                            _ => return Err(RLibError::EncodingTableWrongFieldType(field_data.to_string(), field.field_type().to_string())),
                        }
                    }
                    _ => return Err(RLibError::EncodingTableWrongFieldType(FieldType::from(&row[data_column]).to_string(), field.field_type().to_string())),
                }
            }*/

            // If no special behavior has been needed, encode the field as a normal field, except for strings.
            else {

                match &row[data_column] {
                    DecodedData::Boolean(field_data) => data.write_bool(*field_data)?,
                    DecodedData::F32(field_data) => data.write_f32(*field_data)?,
                    DecodedData::F64(field_data) => data.write_f64(*field_data)?,
                    DecodedData::I16(field_data) => data.write_i16(*field_data)?,
                    DecodedData::I32(field_data) => data.write_i32(*field_data)?,
                    DecodedData::I64(field_data) => data.write_i64(*field_data)?,
                    DecodedData::ColourRGB(field_data) => data.write_string_colour_rgb(field_data)?,
                    DecodedData::OptionalI16(field_data) => {
                        data.write_bool(true)?;
                        data.write_i16(*field_data)?
                    },
                    DecodedData::OptionalI32(field_data) => {
                        data.write_bool(true)?;
                        data.write_i32(*field_data)?
                    },
                    DecodedData::OptionalI64(field_data) => {
                        data.write_bool(true)?;
                        data.write_i64(*field_data)?
                    },

                    // String fields may need preprocessing applied to them before encoding.
                    DecodedData::StringU8(field_data) |
                    DecodedData::StringU16(field_data) |
                    DecodedData::OptionalStringU8(field_data) |
                    DecodedData::OptionalStringU16(field_data) => {

                        // String files may be representations of enums (as integer => string) for ease of use.
                        // If so, we need to find the underlying integer key of our string and encode that.
                        if !field.enum_values().is_empty() {
                            let field_data = match field.enum_values()
                                .iter()
                                .find_map(|(x, y)|
                                    if y.to_lowercase() == field_data.to_lowercase() { Some(x) } else { None }) {
                                Some(value) => {
                                    match field.field_type() {
                                        FieldType::I16 => DecodedData::I16(*value as i16),
                                        FieldType::I32 => DecodedData::I32(*value),
                                        FieldType::I64 => DecodedData::I64(*value as i64),
                                        _ => return Err(RLibError::EncodingTableWrongFieldType(field_data.to_string(), field.field_type().to_string()))
                                    }
                                }
                                None => match row[data_column].convert_between_types(field.field_type()) {
                                    Ok(data) => data,
                                    Err(_) => {
                                        let default_value = field.default_value(*patches);
                                        DecodedData::new_from_type_and_value(field.field_type(), &default_value)
                                    }
                                }
                            };

                            // If there are no problems, encode the data.
                            match field_data {
                                DecodedData::I16(field_data) => data.write_i16(field_data)?,
                                DecodedData::I32(field_data) => data.write_i32(field_data)?,
                                DecodedData::I64(field_data) => data.write_i64(field_data)?,
                                _ => return Err(RLibError::EncodingTableWrongFieldType(field_data.data_to_string().to_string(), field.field_type().to_string()))
                            }
                        }
                        else {

                            // If there are no problems, encode the data.
                            match field.field_type() {
                                FieldType::StringU8 => data.write_sized_string_u8(&unescape_special_chars(field_data))?,
                                FieldType::StringU16 => data.write_sized_string_u16(&unescape_special_chars(field_data))?,
                                FieldType::OptionalStringU8 => data.write_optional_string_u8(&unescape_special_chars(field_data))?,
                                FieldType::OptionalStringU16 => data.write_optional_string_u16(&unescape_special_chars(field_data))?,
                                _ => return Err(RLibError::EncodingTableWrongFieldType(field_data.to_string(), field.field_type().to_string()))
                            }
                        }
                    }

                    // Make sure we at least have the counter before writing. We need at least that.
                    DecodedData::SequenceU16(field_data) => {
                        if field_data.len() < 2 {
                            data.write_all(&[0, 0])?
                        } else {
                            data.write_all(field_data)?
                        }
                    },
                    DecodedData::SequenceU32(field_data) => {
                        if field_data.len() < 4 {
                            data.write_all(&[0, 0, 0, 0])?
                        } else {
                            data.write_all(field_data)?
                        }
                    }
                }

                data_column += 1;
            }
        }
    }

    Ok(())
}
