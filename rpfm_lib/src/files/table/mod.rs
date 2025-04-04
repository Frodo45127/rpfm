//---------------------------------------------------------------------------//
// Copyright (c) 2017-2024 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

/*!
Module with all the code to interact with any kind of table data.

This module contains the struct `Table`, used to manage the decoded data of a table. For internal use only.
!*/

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

/// Trait for structs with table-like data.
pub trait Table: Send + Sync {

    // Getters
    fn name(&self) -> &str;

    /// This function returns a reference of the definition of this Table.
    fn definition(&self) -> &Definition;

    /// This function returns a reference of the definition patches of this Table.
    fn patches(&self) -> &DefinitionPatch;

    /// This function returns the data stored in the table.
    fn data(&self) -> Cow<[Vec<DecodedData>]>;

    /// This function returns a mutable reference to the data of the table.
    ///
    /// Note that using this makes you responsible of keeping the structure of the table "valid".
    fn data_mut(&mut self) -> &mut Vec<Vec<DecodedData>>;

    // Setters
    fn set_name(&mut self, val: String);

    /// This function replaces the definition of this table with the one provided.
    ///
    /// This updates the table's data to follow the format marked by the new definition, so you can use it to *update* the version of your table.
    fn set_definition(&mut self, new_definition: &Definition);

    /// This function replaces the data of this table with the one provided.
    ///
    /// This can (and will) fail if the data is not of the format defined by the definition of the table.
    fn set_data(&mut self, data: &[Vec<DecodedData>]) -> Result<()>;

    /// This function returns the position of a column in a definition before sorting, or None if the column is not found.
    fn column_position_by_name(&self, column_name: &str) -> Option<usize>;

    fn is_empty(&self) -> bool;
    fn len(&self) -> usize;

    /// This function returns a new empty row for the provided definition.
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

/// This enum is used to store different types of data in a unified way. Used, for example, to store the data from each field in a DB Table.
///
/// NOTE: `Sequence` it's a recursive type. A Sequence/List means you got a repeated sequence of fields
/// inside a single field. Used, for example, in certain model tables.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum DecodedData {
    Boolean(bool),
    F32(f32),
    F64(f64),
    I16(i16),
    I32(i32),
    I64(i64),
    ColourRGB(String),
    StringU8(String),
    StringU16(String),
    OptionalI16(i16),
    OptionalI32(i32),
    OptionalI64(i64),
    OptionalStringU8(String),
    OptionalStringU16(String),
    SequenceU16(Vec<u8>),
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

    /// This function initializes and returns a `DecodedData` of the requested type.
    pub fn new_from_type_and_value(field_type: &FieldType, default_value: &Option<String>) -> Self {
        match default_value {
            Some(default_value) => match field_type {
                FieldType::Boolean => if let Ok(value) = parse_str_as_bool(default_value) { DecodedData::Boolean(value) } else { DecodedData::Boolean(false) },
                FieldType::F32 => if let Ok(value) = default_value.parse::<f32>() { DecodedData::F32(value) } else { DecodedData::F32(0.0) },
                FieldType::F64 => if let Ok(value) = default_value.parse::<f64>() { DecodedData::F64(value) } else { DecodedData::F64(0.0) },
                FieldType::I16 => if let Ok(value) = default_value.parse::<i16>() { DecodedData::I16(value) } else { DecodedData::I16(0) },
                FieldType::I32 => if let Ok(value) = default_value.parse::<i32>() { DecodedData::I32(value) } else { DecodedData::I32(0) },
                FieldType::I64 => if let Ok(value) = default_value.parse::<i64>() { DecodedData::I64(value) } else { DecodedData::I64(0) },
                FieldType::ColourRGB => DecodedData::ColourRGB(default_value.to_owned()),
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

    /// This function creates a new DecodedData of the requested type.
    ///
    /// Returns error if the value is not parseable by the provided FieldType.
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

    /// This functions checks if the type of an specific `DecodedData` is the one it should have, according to the provided `FieldType`.
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

    /// This function tries to convert the provided data to the provided fieldtype. This can fail in so many ways you should always check the result.
    ///
    /// NOTE: If you pass the same type as it already has, this becomes an expensive way of cloning.
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
                FieldType::Boolean => Self::Boolean(data > &1.0),
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
                FieldType::Boolean => Self::Boolean(data > &1.0),
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
                FieldType::Boolean => Self::Boolean(data > &1),
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
                FieldType::Boolean => Self::Boolean(data > &1),
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
                FieldType::Boolean => Self::Boolean(data > &1),
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

    /// This function prints whatever you have in each variants to a String.
    pub fn data_to_string(&self) -> Cow<str> {
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

    /// This function tries to change the current data with the new one provided.
    ///
    /// It may fail if the new data is not parseable to the type required of the current data.
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

/// This function escapes certain characters of the provided string.
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

/// This function unescapes certain characters of the provided string.
fn unescape_special_chars(data: &str) -> String {
    data.replace("\\\\t", "\t").replace("\\\\n", "\n")
}

//----------------------------------------------------------------//
// Decoding and encoding functions for tables.
//----------------------------------------------------------------//

pub(crate) fn decode_table<R: ReadBytes>(data: &mut R, definition: &Definition, entry_count: Option<u32>, return_incomplete: bool) -> Result<Vec<Vec<DecodedData>>> {

    // If we received an entry count, it's the root table. If not, it's a nested one.
    let entry_count = match entry_count {
        Some(entry_count) => entry_count,
        None => data.read_u32()?,
    };

    // Do not specify size here, because a badly written definition can end up triggering an OOM crash if we do.
    let fields = definition.fields();
    let mut table = if entry_count < 10_000 { Vec::with_capacity(entry_count as usize) } else { vec![] };

    for row in 0..entry_count {
        table.push(decode_row(data, fields, row, return_incomplete)?);
    }

    Ok(table)
}

fn decode_row<R: ReadBytes>(data: &mut R, fields: &[Field], row: u32, return_incomplete: bool) -> Result<Vec<DecodedData>> {
    let mut split_colours: BTreeMap<u8, HashMap<String, u8>> = BTreeMap::new();
    let mut row_data = Vec::with_capacity(fields.len());
    for (column, field) in fields.iter().enumerate() {

        // Decode the field, then apply any postprocess operation we need.
        let column = column as u32;
        let field_data = match decode_field(data, field, row, column) {
            Ok(data) => data,
            Err(error) => {
                if return_incomplete {
                    return Ok(row_data);
                } else {
                    return Err(error);
                }
            }
        };
        decode_field_postprocess(&mut row_data, field_data, field, &mut split_colours)
    }

    decode_row_postprocess(&mut row_data, &mut split_colours)?;

    Ok(row_data)
}

fn decode_field<R: ReadBytes>(data: &mut R, field: &Field, row: u32, column: u32) -> Result<DecodedData> {
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
            match decode_table(data, definition, Some(entry_count as u32), false) {
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
            match decode_table(data, definition, Some(entry_count), false) {
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

fn decode_field_postprocess(row_data: &mut Vec<DecodedData>, data: DecodedData, field: &Field, split_colours: &mut BTreeMap<u8, HashMap<String, u8>>) {

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

    else {
        row_data.push(data);
    }
}

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
