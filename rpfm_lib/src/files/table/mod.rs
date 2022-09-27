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
Module with all the code to interact with any kind of table data.

This module contains the struct `Table`, used to manage the decoded data of a table. For internal use only.
!*/

use csv::{StringRecordsIter, Writer};
use float_eq::float_eq;
use getset::*;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use serde_derive::{Serialize, Deserialize};

use std::borrow::Cow;
use std::collections::{BTreeMap, HashMap};
use std::fs::File;
use std::io::SeekFrom;

use crate::error::{RLibError, Result};
use crate::binary::{ReadBytes, WriteBytes};
use crate::schema::*;
use crate::utils::parse_str_as_bool;

mod local;
#[cfg(feature = "integration_sqlite")] mod sqlite;

//---------------------------------------------------------------------------//
//                              Enum & Structs
//---------------------------------------------------------------------------//

/// This struct contains the data of a Table-like PackedFile after being decoded.
///
/// This is for internal use. If you need to interact with this in any way, do it through the PackedFile that contains it, not directly.
#[derive(Clone, Debug, PartialEq, Getters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", set = "pub")]
pub struct Table {

    /// A copy of the `Definition` this table uses, so we don't have to check the schema everywhere.
    table_name: String,

    #[getset(skip)]
    definition: Definition,

    #[getset(skip)]
    definition_patch: DefinitionPatch,

    #[getset(skip)]
    table_data: TableData
}

/// Internal enum to hold the table's data.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum TableData {

    /// Variant to hold the table's data internally in memory.
    Local(Vec<Vec<DecodedData>>),

    /// Variant to hold the unique key of this table in the SQL backend.
    Sql(SQLData)
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SQLData {
    table_unique_id: u64,
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

/// PartialEq implementation of `DecodedData`. We need this implementation due to
/// the float comparison being... special.
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
                FieldType::SequenceU16(_) => DecodedData::SequenceU16(vec![]),
                FieldType::SequenceU32(_) => DecodedData::SequenceU32(vec![]),
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
                FieldType::SequenceU16(_) => DecodedData::SequenceU16(vec![]),
                FieldType::SequenceU32(_) => DecodedData::SequenceU32(vec![]),
            }
        }
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
                FieldType::I16 => Self::I16(if *data { 1 } else { 0 }),
                FieldType::I32 => Self::I32(if *data { 1 } else { 0 }),
                FieldType::I64 => Self::I64(if *data { 1 } else { 0 }),
                FieldType::ColourRGB => Self::ColourRGB(if *data { "FFFFFF" } else { "000000" }.to_owned()),
                FieldType::StringU8 => Self::StringU8(data.to_string()),
                FieldType::StringU16 => Self::StringU16(data.to_string()),
                FieldType::OptionalI16 => Self::OptionalI16(if *data { 1 } else { 0 }),
                FieldType::OptionalI32 => Self::OptionalI32(if *data { 1 } else { 0 }),
                FieldType::OptionalI64 => Self::OptionalI64(if *data { 1 } else { 0 }),
                FieldType::OptionalStringU8 => Self::OptionalStringU8(data.to_string()),
                FieldType::OptionalStringU16 => Self::OptionalStringU16(data.to_string()),
                FieldType::SequenceU16(_) => Self::SequenceU16(vec![]),
                FieldType::SequenceU32(_) => Self::SequenceU16(vec![]),
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
                FieldType::SequenceU16(_) => Self::SequenceU16(vec![]),
                FieldType::SequenceU32(_) => Self::SequenceU16(vec![]),
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
                FieldType::SequenceU16(_) => Self::SequenceU16(vec![]),
                FieldType::SequenceU32(_) => Self::SequenceU16(vec![]),
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
                FieldType::SequenceU16(_) => Self::SequenceU16(vec![]),
                FieldType::SequenceU32(_) => Self::SequenceU16(vec![]),
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
                FieldType::SequenceU16(_) => Self::SequenceU16(vec![]),
                FieldType::SequenceU32(_) => Self::SequenceU16(vec![]),
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
                FieldType::SequenceU16(_) => Self::SequenceU16(vec![]),
                FieldType::SequenceU32(_) => Self::SequenceU16(vec![]),
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
                FieldType::SequenceU16(_) => Self::SequenceU16(vec![]),
                FieldType::SequenceU32(_) => Self::SequenceU16(vec![]),
            }

            Self::SequenceU16(_) |
            Self::SequenceU32(_) => Self::new_from_type_and_value(new_field_type, &None)
        })
    }

    /// This function prints whatever you have in each variants to a String.
    pub fn data_to_string(&self) -> Cow<str> {
        match self {
            DecodedData::Boolean(data) => Cow::from(if *data { "true" } else { "false" }),
            DecodedData::F32(data) => Cow::from(format!("{:.4}", data)),
            DecodedData::F64(data) => Cow::from(format!("{:.4}", data)),
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
            DecodedData::SequenceU16(_) => Cow::from("SequenceU16"),
            DecodedData::SequenceU32(_) => Cow::from("SequenceU32"),
        }
    }

    /// This function tries to change the current data with the new one provided.
    ///
    /// It may fail if the new data is not pareseable to the type required of the current data.
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
// Implementations for `Table`.
//----------------------------------------------------------------//

/// Implementation of `Table`.
impl Table {

    /// This function creates a new Table from an existing definition.
    pub fn new(definition: &Definition, definition_patch: Option<&DefinitionPatch>, table_name: &str, use_sql_backend: bool) -> Self {
        let table_data = if use_sql_backend {
            TableData::Sql(SQLData {
                table_unique_id: rand::random::<u64>(),
            })
        } else {
            TableData::Local(vec![])
        };

        let definition_patch = if let Some(patch) = definition_patch { patch.clone() } else { HashMap::new() };

        Self {
            definition: definition.clone(),
            definition_patch,
            table_name: table_name.to_owned(),
            table_data
        }
    }

    /// This function returns a reference of the definition of this Table.
    pub fn definition(&self) -> &Definition {
        &self.definition
    }

    /// This function returns a reference of the definition patches of this Table.
    pub fn patches(&self) -> &DefinitionPatch {
        &self.definition_patch
    }
    /*

    /// This function returns the position of a column in a definition, or an error if the column is not found.
    pub fn get_column_position_by_name(&self, column_name: &str) -> Result<usize> {
        self.get_ref_definition().get_column_position_by_name(column_name)
    }

    /// This function replaces the definition of this table with the one provided.
    ///
    /// This updates the table's data to follow the format marked by the new definition, so you can use it to *update* the version of your table.
    pub fn set_definition(&mut self, new_definition: &Definition) {

        // It's simple: we compare both schemas, and get the original and final positions of each column.
        // If a column is new, his original position is -1. If has been removed, his final position is -1.
        let mut positions: Vec<(i32, i32)> = vec![];
        let new_fields_processed = new_definition.get_fields_processed();
        let old_fields_processed = self.definition.get_fields_processed();

        for (new_pos, new_field) in new_fields_processed.iter().enumerate() {
            if let Some(old_pos) = old_fields_processed.iter().position(|x| x.get_name() == new_field.get_name()) {
                positions.push((old_pos as i32, new_pos as i32))
            } else { positions.push((-1, new_pos as i32)); }
        }

        // Then, for each field in the old definition, check if exists in the new one.
        for (old_pos, old_field) in old_fields_processed.iter().enumerate() {
            if !new_fields_processed.iter().any(|x| x.get_name() == old_field.get_name()) { positions.push((old_pos as i32, -1)); }
        }

        // We sort the columns by their destination.
        positions.sort_by_key(|x| x.1);

        // Then, we create the new data using the old one and the column changes.
        let mut new_entries: Vec<Vec<DecodedData>> = vec![];
        for row in &mut self.entries {
            let mut entry = vec![];
            for (old_pos, new_pos) in &positions {

                // If the new position is -1, it means the column got removed. We skip it.
                if *new_pos == -1 { continue; }

                // If the old position is -1, it means we got a new column. We need to get his type and create a `Default` field with it.
                else if *old_pos == -1 {
                    entry.push(DecodedData::default(new_fields_processed[*new_pos as usize].get_ref_field_type(), &new_fields_processed[*new_pos as usize].get_default_value(None)));
                }

                // Otherwise, we got a moved column. Check here if it needs type conversion.
                else if new_fields_processed[*new_pos as usize].get_ref_field_type() != old_fields_processed[*old_pos as usize].get_ref_field_type() {
                    entry.push(row[*old_pos as usize].convert_between_types(new_fields_processed[*new_pos as usize].get_ref_field_type()).unwrap());
                }

                // If we reach this, we just got a moved column without any extra change.
                else {
                    entry.push(row[*old_pos as usize].clone());
                }
            }
            new_entries.push(entry);
        }

        // Then, we finally replace our definition and our data.
        self.definition = new_definition.clone();
        self.entries = new_entries;
    }*/

    /// This function replaces the data of this table with the one provided.
    ///
    /// This can (and will) fail if the data is not of the format defined by the definition of the table.
    pub fn set_data(&mut self, pool: Option<&Pool<SqliteConnectionManager>>, data: &[Vec<DecodedData>]) -> Result<()> {
        match self.table_data {
            TableData::Local(ref mut table_data) => {
                for row in data {

                    // First, we need to make sure all rows we have are exactly what we expect.
                    let fields_processed = self.definition.fields_processed();
                    if row.len() != fields_processed.len() {
                        return Err(RLibError::TableRowWrongFieldCount(fields_processed.len(), row.len()).into())
                    }

                    for (index, cell) in row.iter().enumerate() {

                        // Next, we need to ensure each file is of the type we expected.
                        let field = fields_processed.get(index).unwrap();
                        if !cell.is_field_type_correct(field.field_type()) {
                            return Err(RLibError::EncodingTableWrongFieldType(FieldType::from(cell).to_string(), field.field_type().to_string()))
                        }
                    }
                }

                // If we passed all the checks, replace the data.
                *table_data = data.to_vec();
                Ok(())
            }

            // TODO: Make this work for sql_backed tables.
            TableData::Sql(ref mut _table_data) => {
                match pool {
                    Some(_pool) => todo!(),
                    None => Err(RLibError::MissingSQLitePool),
                }
            }
        }
    }

    pub fn len(&self, pool: Option<&Pool<SqliteConnectionManager>>) -> Result<usize> {
        match &self.table_data {
            TableData::Local(data) => Ok(data.len()),
            TableData::Sql(sqldata) => match pool {
                Some(pool) => Self::count_table(pool, &self.table_name, *self.definition().version(), sqldata.table_unique_id).map(|x| x as usize),
                None => Err(RLibError::MissingSQLitePool),
            }
        }
    }

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
            table.push(Self::decode_row(data, fields, row, return_incomplete)?);
        }

        Ok(table)
    }

    fn decode_row<R: ReadBytes>(data: &mut R, fields: &[Field], row: u32, return_incomplete: bool) -> Result<Vec<DecodedData>> {
        let mut split_colours: BTreeMap<u8, HashMap<String, u8>> = BTreeMap::new();
        let mut row_data = Vec::with_capacity(fields.len());
        for (column, field) in fields.iter().enumerate() {

            // Decode the field, then apply any postprocess operation we need.
            let column = column as u32;
            let field_data = match Self::decode_field(data, field, row, column) {
                Ok(data) => data,
                Err(error) => {
                    if return_incomplete {
                        return Ok(row_data);
                    } else {
                        return Err(error);
                    }
                }
            };
            Self::decode_field_postprocess(&mut row_data, field_data, field, &mut split_colours)
        }

        Self::decode_row_postprocess(&mut row_data, &mut split_colours)?;

        Ok(row_data)
    }

    fn decode_field<R: ReadBytes>(data: &mut R, field: &Field, row: u32, column: u32) -> Result<DecodedData> {
        match field.field_type() {
            FieldType::Boolean => {
                data.read_bool()
                    .map(|data| DecodedData::Boolean(data))
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
                if let Ok(data) = data.read_sized_string_u8() { Ok(DecodedData::StringU8(Self::escape_special_chars(&data))) }
                else { Err(RLibError::DecodingTableFieldError(row + 1, column + 1, "UTF-8 String".to_string())) }
            }
            FieldType::StringU16 => {
                if let Ok(data) = data.read_sized_string_u16() { Ok(DecodedData::StringU16(Self::escape_special_chars(&data))) }
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
                if let Ok(data) = data.read_optional_string_u8() { Ok(DecodedData::OptionalStringU8(Self::escape_special_chars(&data))) }
                else { Err(RLibError::DecodingTableFieldError(row + 1, column + 1, "Optional UTF-8 String".to_string())) }
            }
            FieldType::OptionalStringU16 => {
                if let Ok(data) = data.read_optional_string_u16() { Ok(DecodedData::OptionalStringU16(Self::escape_special_chars(&data))) }
                else { Err(RLibError::DecodingTableFieldError(row + 1, column + 1, "Optional UTF-16 String".to_string())) }
            }

            FieldType::SequenceU16(definition) => {
                let start = data.stream_position()?;
                let entry_count = data.read_u16()?;
                match Self::decode_table(data, definition, Some(entry_count as u32), false) {
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
                match Self::decode_table(data, definition, Some(entry_count), false) {
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
                colour_hex.push_str(&format!("{:02X?}", r));
            }

            if let Some(r) = split_colour.get("red") {
                colour_hex.push_str(&format!("{:02X?}", r));
            }

            if let Some(g) = split_colour.get("g") {
                colour_hex.push_str(&format!("{:02X?}", g));
            }

            if let Some(g) = split_colour.get("green") {
                colour_hex.push_str(&format!("{:02X?}", g));
            }

            if let Some(b) = split_colour.get("b") {
                colour_hex.push_str(&format!("{:02X?}", b));
            }

            if let Some(b) = split_colour.get("blue") {
                colour_hex.push_str(&format!("{:02X?}", b));
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
                let colour_split = field.name().rsplitn(2, "_").collect::<Vec<&str>>();
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

    pub(crate) fn encode<W: WriteBytes>(&self, data: &mut W, schema_patches: &Option<&DefinitionPatch>, pool: &Option<&Pool<SqliteConnectionManager>>) -> Result<()> {

        // Get the table data in local format, no matter in what backend we stored it.
        let entries = self.data(pool)?;
        let fields = self.definition.fields();
        let fields_processed = self.definition.fields_processed();

        // Get the colour positions of the tables, if any.
        let combined_colour_positions = fields.iter().filter_map(|field| {
            if field.is_part_of_colour().is_some() {
                let colour_split = field.name().rsplitn(2, "_").collect::<Vec<&str>>();
                let colour_field_name: String = if colour_split.len() == 2 { format!("{}{}", colour_split[1].to_lowercase(), MERGE_COLOUR_POST) } else { MERGE_COLOUR_NO_NAME.to_lowercase() };

                self.definition.column_position_by_name(&colour_field_name).map(|x| (colour_field_name, x))
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
                if field.is_part_of_colour().is_some() {
                    let field_name = field.name().to_lowercase();
                    let colour_split = field_name.rsplitn(2, "_").collect::<Vec<&str>>();
                    let colour_channel = colour_split[0];
                    let colour_field_name = if colour_split.len() == 2 {
                        format!("{}{}", colour_split[1], MERGE_COLOUR_POST)
                    } else {
                        MERGE_COLOUR_NO_NAME.to_owned()
                    };

                    if let Some(data_column) = combined_colour_positions.get(&colour_field_name) {
                        match &row[*data_column] {
                            DecodedData::ColourRGB(field_data) => {

                                // Encode the full colour, then grab the byte of our field.
                                let mut encoded = vec![];
                                encoded.write_string_colour_rgb(&field_data)?;

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
                        DecodedData::ColourRGB(field_data) => data.write_string_colour_rgb(&field_data)?,
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
                                            let default_value = field.default_value(*schema_patches);
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
                                    FieldType::StringU8 => data.write_sized_string_u8(&Self::unescape_special_chars(&field_data))?,
                                    FieldType::StringU16 => data.write_sized_string_u16(&Self::unescape_special_chars(&field_data))?,
                                    FieldType::OptionalStringU8 => data.write_optional_string_u8(&Self::unescape_special_chars(&field_data))?,
                                    FieldType::OptionalStringU16 => data.write_optional_string_u16(&Self::unescape_special_chars(&field_data))?,
                                    _ => return Err(RLibError::EncodingTableWrongFieldType(field_data.to_string(), field.field_type().to_string()))
                                }
                            }
                        }

                        // Make sure we at least have the counter before writing. We need at least that.
                        DecodedData::SequenceU16(field_data) => {
                            if field_data.len() < 2 {
                                data.write_all(&vec![0, 0])?
                            } else {
                                data.write_all(&field_data)?
                            }
                        },
                        DecodedData::SequenceU32(field_data) => {
                            if field_data.len() < 4 {
                                data.write_all(&vec![0, 0, 0, 0])?
                            } else {
                                data.write_all(&field_data)?
                            }
                        }
                    }

                    data_column += 1;
                }
            }
        }

        Ok(())
    }

    /// This function returns the data stored in the table.
    pub fn data(&self, pool: &Option<&Pool<SqliteConnectionManager>>) -> Result<Cow<[Vec<DecodedData>]>> {

        // Get the table data in local format, no matter in what backend we stored it.
        match self.table_data {
            TableData::Local(ref data) => Ok(Cow::from(data)),
            TableData::Sql(ref sqldata) => match pool {
                Some(pool) => {
                    let fields_processed = self.definition().fields_processed();
                    let version = self.definition().version();
                    let data = Self::select_all_from_table(pool, &self.table_name, *version, sqldata.table_unique_id, &fields_processed)?;
                    Ok(Cow::from(data))
                }
                None => Err(RLibError::MissingSQLitePool),
            },
        }
    }

    /// This function returns a new empty row for the provided definition.
    pub fn new_row(definition: &Definition, schema_patches: Option<&DefinitionPatch>) -> Vec<DecodedData> {
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
                            DecodedData::StringU8(default_value.to_owned())
                        } else {
                            DecodedData::StringU8(String::new())
                        }
                    }
                    FieldType::StringU16 => {
                        if let Some(default_value) = field.default_value(schema_patches) {
                            DecodedData::StringU16(default_value.to_owned())
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
                            DecodedData::OptionalStringU8(default_value.to_owned())
                        } else {
                            DecodedData::OptionalStringU8(String::new())
                        }
                    }
                    FieldType::OptionalStringU16 => {
                        if let Some(default_value) = field.default_value(schema_patches) {
                            DecodedData::OptionalStringU16(default_value.to_owned())
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

/*
    /// This function encodes all the fields of a table to raw bytes.
    fn encode(&self, mut packed_file: &mut Vec<u8>) -> Result<()> {
        let fields = self.definition.get_ref_fields();
        let fields_processed = self.definition.get_fields_processed();
        for row in &self.entries {

            // First, we need to make sure all rows we're going to encode are exactly what we expect.
            if row.len() != fields_processed.len() { return Err(ErrorKind::TableRowWrongFieldCount(fields_processed.len() as u32, row.len() as u32).into()) }
            let mut data_column = 0;

            let combined_colour_positions = fields.iter().filter_map(|field| {
                if field.get_is_part_of_colour().is_some() {
                    let colour_split = field.get_name().rsplitn(2, "_").collect::<Vec<&str>>();
                    let colour_field_name: String = if colour_split.len() == 2 { format!("{}{}", colour_split[1].to_lowercase(), MERGE_COLOUR_POST) } else { MERGE_COLOUR_NO_NAME.to_lowercase() };

                    self.definition.get_column_position_by_name(&colour_field_name).ok().map(|x| (colour_field_name, x))
                } else { None }
            }).collect::<HashMap<String, usize>>();

            for field in fields {
                if field.get_is_part_of_colour().is_some() {
                    let colour_split = field.get_name().rsplitn(2, "_").collect::<Vec<&str>>();
                    let colour_channel = colour_split[0].to_lowercase();
                    let colour_field_name = if colour_split.len() == 2 { format!("{}{}", colour_split[1].to_lowercase(), MERGE_COLOUR_POST) } else { MERGE_COLOUR_NO_NAME.to_lowercase() };

                    if let Some(data_column) = combined_colour_positions.get(&colour_field_name) {
                        match row[*data_column] {
                            DecodedData::ColourRGB(data) => {
                                let mut encoded = vec![];
                                encoded.encode_integer_u32(data);
                                let data = if colour_channel == "r" || colour_channel == "red" { encoded[2] }
                                else if colour_channel == "g" || colour_channel == "green" { encoded[1] }
                                else if colour_channel == "b" || colour_channel == "blue" { encoded[0] }
                                else { 0 };

                                match field.get_field_type() {
                                    FieldType::I16 => packed_file.encode_integer_i16(data as i16),
                                    FieldType::I32 => packed_file.encode_integer_i32(data as i32),
                                    FieldType::I64 => packed_file.encode_integer_i64(data as i64),
                                    FieldType::F32 => packed_file.encode_float_f32(data as f32),
                                    FieldType::F64 => packed_file.encode_float_f64(data as f64),
                                    _ => return Err(ErrorKind::TableWrongFieldType(format!("{}", row[*data_column]), format!("{}", field.get_ref_field_type())).into())
                                }

                            },
                            _ => return Err(ErrorKind::TableWrongFieldType(format!("{}", row[*data_column]), format!("{}", field.get_ref_field_type())).into())
                        }
                    }
                }

                else if field.get_is_bitwise() > 1 {
                    let mut data: i64 = 0;
                    for bitwise_column in 0..field.get_is_bitwise() {
                        if let DecodedData::Boolean(boolean) = row[data_column] {
                            if boolean {
                                data |= 1 << bitwise_column;
                            }
                        }

                        else {
                            return Err(ErrorKind::TableWrongFieldType(format!("{}", row[data_column]), format!("{}", field.get_ref_field_type())).into())
                        }

                        data_column += 1;
                    }

                    // If there are no problems, encode the data.
                    match field.get_field_type() {
                        FieldType::I16 => packed_file.encode_integer_i16(data as i16),
                        FieldType::I32 => packed_file.encode_integer_i32(data as i32),
                        FieldType::I64 => packed_file.encode_integer_i64(data),
                        _ => return Err(ErrorKind::TableWrongFieldType(format!("{}", row[data_column]), format!("{}", field.get_ref_field_type())).into())
                    }
                }

                else {

                    match row[data_column] {
                        DecodedData::Boolean(data) => packed_file.encode_bool(data),
                        DecodedData::F32(data) => packed_file.encode_float_f32(data),
                        DecodedData::F64(data) => packed_file.encode_float_f64(data),
                        DecodedData::I16(data) => packed_file.encode_integer_i16(data),
                        DecodedData::I32(data) => packed_file.encode_integer_i32(data),
                        DecodedData::I64(data) => packed_file.encode_integer_i64(data),
                        DecodedData::ColourRGB(data) => packed_file.encode_integer_colour_rgb(data),
                        DecodedData::StringU8(ref data) |
                        DecodedData::StringU16(ref data) |
                        DecodedData::OptionalStringU8(ref data) |
                        DecodedData::OptionalStringU16(ref data) => {

                            // If the field has enum values, try to match them. If the matching fails, try to just encode them.
                            // If that fails, put a default value on that cell.
                            let values = field.get_enum_values();
                            if !values.is_empty() {
                                let data = match values.iter().find(|(_, y)| y.to_lowercase() == data.to_lowercase()) {
                                    Some((x, _)) => {
                                        match field.get_field_type() {
                                            FieldType::I16 => DecodedData::I16(*x as i16),
                                            FieldType::I32 => DecodedData::I32(*x),
                                            FieldType::I64 => DecodedData::I64(*x as i64),
                                            _ => return Err(ErrorKind::TableWrongFieldType(format!("{}", row[data_column]), format!("{}", field.get_ref_field_type())).into())
                                        }
                                    }
                                    None => match row[data_column].convert_between_types(field.get_ref_field_type()) {
                                        Ok(data) => data,
                                        Err(_) => DecodedData::default(field.get_ref_field_type(), &field.get_default_value(None))
                                    }
                                };

                                // If there are no problems, encode the data.
                                match data {
                                    DecodedData::I16(data) => packed_file.encode_integer_i16(data),
                                    DecodedData::I32(data) => packed_file.encode_integer_i32(data),
                                    DecodedData::I64(data) => packed_file.encode_integer_i64(data),
                                    _ => return Err(ErrorKind::TableWrongFieldType(format!("{}", row[data_column]), format!("{}", field.get_ref_field_type())).into())
                                }
                            }
                            else {

                                // If there are no problems, encode the data.
                                match row[data_column] {
                                    DecodedData::StringU8(ref data) => packed_file.encode_packedfile_string_u8(&Self::unescape_special_chars(data)),
                                    DecodedData::StringU16(ref data) => packed_file.encode_packedfile_string_u16(&Self::unescape_special_chars(data)),
                                    DecodedData::OptionalStringU8(ref data) => packed_file.encode_packedfile_optional_string_u8(&Self::unescape_special_chars(data)),
                                    DecodedData::OptionalStringU16(ref data) => packed_file.encode_packedfile_optional_string_u16(&Self::unescape_special_chars(data)),
                                    _ => return Err(ErrorKind::TableWrongFieldType(format!("{}", row[data_column]), format!("{}", field.get_ref_field_type())).into())
                                }
                            }
                        }
                        DecodedData::SequenceU16(ref data) => {
                            if let FieldType::SequenceU16(_) = fields[data_column].get_ref_field_type() {
                                packed_file.encode_integer_u16(data.entries.len() as u16);
                                data.encode(&mut packed_file)?;
                            }
                        },
                        DecodedData::SequenceU32(ref data) => {
                            if let FieldType::SequenceU32(_) = fields[data_column].get_ref_field_type() {
                                packed_file.encode_integer_u32(data.entries.len() as u32);
                                data.encode(&mut packed_file)?;
                            }
                        },
                    }

                    data_column += 1;
                }
            }
        }

        Ok(())
    }*/
/*
    /// This function decodes all the fields of a table from raw bytes into a `INSERT INTO` SQL Query.
    ///
    /// If return_incomplete == true, this function will return an error with the incompletely decoded table when it fails.
    fn decode_to_query(&self,
        definition: &Definition,
        data: &[u8],
        entry_count: Option<u32>,
        mut index: &mut usize,
        is_nested: bool,
        return_incomplete: bool,
    ) -> Result<String> {

        // If we received an entry count, it's the root table. If not, it's a nested one.
        let entry_count = match entry_count {
            Some(entry_count) => entry_count,
            None => data.decode_packedfile_integer_u32(*index, index)?,
        };

        // Do not specify size here, because a badly written definition can end up triggering an OOM crash if we do.
        let fields = definition.fields();
        let mut query = if is_nested {
            let column_names = fields.iter().map(|field| format!("\"{}\"", field.name())).collect::<Vec<_>>().join(",");
            format!("INSERT INTO {} (source, file_name, {}) VALUES (?, ...), (?, ...); ", self.table_name, column_names)
        } else {
            String::new()
        };

        for row in 0..entry_count {

            // TODO: Fix the source value here.
            let mut row_values = format!("({}, {},", 0, self.file_name);

            let mut split_colour_fields: BTreeMap<u8, HashMap<String, u8>> = BTreeMap::new();

            for column in 0..fields.len() {
                let field = &fields[column];
                let decoded_cell = match field.field_type() {
                    FieldType::Boolean => {
                        if let Ok(data) = data.decode_packedfile_bool(*index, &mut index) { Ok((data as i32).to_string()) }
                        else { Err(anyhow!("<p>Error trying to decode the <i><b>Row {}, Cell {}</b></i> as a <b><i>Boolean</b></i> value: either the value is not a boolean, or there are insufficient bytes left to decode it as a boolean value.</p>", row + 1, column + 1)) }
                    }
                    FieldType::F32 => {
                        if let Ok(data) = data.decode_packedfile_float_f32(*index, &mut index) { Ok(format!("{:.4}", data)) }
                        else { Err(anyhow!("<p>Error trying to decode the <i><b>Row {}, Cell {}</b></i> as a <b><i>F32</b></i> value: either the value is not a valid F32, or there are insufficient bytes left to decode it as a F32 value.</p>", row + 1, column + 1)) }
                    }
                    FieldType::F64 => {
                        if let Ok(data) = data.decode_packedfile_float_f64(*index, &mut index) { Ok(format!("{:.4}", data)) }
                        else { Err(anyhow!("<p>Error trying to decode the <i><b>Row {}, Cell {}</b></i> as a <b><i>F64</b></i> value: either the value is not a valid F64, or there are insufficient bytes left to decode it as a F64 value.</p>", row + 1, column + 1)) }
                    }
                    FieldType::I16 => {
                        if let Ok(data) = data.decode_packedfile_integer_i16(*index, &mut index) { Ok(data.to_string()) }
                        else { Err(anyhow!("<p>Error trying to decode the <i><b>Row {}, Cell {}</b></i> as a <b><i>I16</b></i> value: either the value is not a valid I16, or there are insufficient bytes left to decode it as an I16 value.</p>", row + 1, column + 1)) }
                    }
                    FieldType::I32 => {
                        if let Ok(data) = data.decode_packedfile_integer_i32(*index, &mut index) { Ok(data.to_string()) }
                        else { Err(anyhow!("<p>Error trying to decode the <i><b>Row {}, Cell {}</b></i> as a <b><i>I32</b></i> value: either the value is not a valid I32, or there are insufficient bytes left to decode it as an I32 value.</p>", row + 1, column + 1)) }
                    }
                    FieldType::I64 => {
                        if let Ok(data) = data.decode_packedfile_integer_i64(*index, &mut index) { Ok(data.to_string()) }
                        else { Err(anyhow!("<p>Error trying to decode the <i><b>Row {}, Cell {}</b></i> as a <b><i>I64</b></i> value: either the value is not a valid I64, or there are insufficient bytes left to decode it as an I64 value.</p>", row + 1, column + 1)) }
                    }
                    FieldType::ColourRGB => {
                        if let Ok(data) = data.decode_packedfile_string_colour_rgb(*index, &mut index) { Ok(data) }
                        else { Err(anyhow!("<p>Error trying to decode the <i><b>Row {}, Cell {}</b></i> as a <b><i>Colour RGB</b></i> value: either the value is not a valid RGB value, or there are insufficient bytes left to decode it as an RGB value.</p>", row + 1, column + 1)) }
                    }
                    FieldType::StringU8 => {
                        if let Ok(data) = data.decode_packedfile_string_u8(*index, &mut index) { Ok(Self::escape_special_chars(&data)) }
                        else { Err(anyhow!("<p>Error trying to decode the <i><b>Row {}, Cell {}</b></i> as an <b><i>UTF-8 String</b></i> value: either the value is not a valid UTF-8 String, or there are insufficient bytes left to decode it as an UTF-8 String.</p>", row + 1, column + 1)) }
                    }
                    FieldType::StringU16 => {
                        if let Ok(data) = data.decode_packedfile_string_u16(*index, &mut index) { Ok(Self::escape_special_chars(&data)) }
                        else { Err(anyhow!("<p>Error trying to decode the <i><b>Row {}, Cell {}</b></i> as an <b><i>UTF-16 String</b></i> value: either the value is not a valid UTF-16 String, or there are insufficient bytes left to decode it as an UTF-16 String.</p>", row + 1, column + 1)) }
                    }
                    FieldType::OptionalI16 => {
                        if let Ok(data) = data.decode_packedfile_optional_integer_i16(*index, &mut index) { Ok(data.to_string()) }
                        else { Err(anyhow!("<p>Error trying to decode the <i><b>Row {}, Cell {}</b></i> as an <b><i>Optional I16</b></i> value: either the value is not a valid Optional I16, or there are insufficient bytes left to decode it as an Optional I16 value.</p>", row + 1, column + 1)) }
                    }
                    FieldType::OptionalI32 => {
                        if let Ok(data) = data.decode_packedfile_optional_integer_i32(*index, &mut index) { Ok(data.to_string()) }
                        else { Err(anyhow!("<p>Error trying to decode the <i><b>Row {}, Cell {}</b></i> as an <b><i>Optional I32</b></i> value: either the value is not a valid Optional I32, or there are insufficient bytes left to decode it as an Optional I32 value.</p>", row + 1, column + 1)) }
                    }
                    FieldType::OptionalI64 => {
                        if let Ok(data) = data.decode_packedfile_optional_integer_i64(*index, &mut index) { Ok(data.to_string()) }
                        else { Err(anyhow!("<p>Error trying to decode the <i><b>Row {}, Cell {}</b></i> as an <b><i>Optional I64</b></i> value: either the value is not a valid Optional I64, or there are insufficient bytes left to decode it as an Optional I64 value.</p>", row + 1, column + 1)) }
                    }

                    FieldType::OptionalStringU8 => {
                        if let Ok(data) = data.decode_packedfile_optional_string_u8(*index, &mut index) { Ok(Self::escape_special_chars(&data)) }
                        else { Err(anyhow!("<p>Error trying to decode the <i><b>Row {}, Cell {}</b></i> as an <b><i>Optional UTF-8 String</b></i> value: either the value is not a valid Optional UTF-8 String, or there are insufficient bytes left to decode it as an Optional UTF-8 String.</p>", row + 1, column + 1)) }
                    }
                    FieldType::OptionalStringU16 => {
                        if let Ok(data) = data.decode_packedfile_optional_string_u16(*index, &mut index) { Ok(Self::escape_special_chars(&data)) }
                        else { Err(anyhow!("<p>Error trying to decode the <i><b>Row {}, Cell {}</b></i> as an <b><i>Optional UTF-16 String</b></i> value: either the value is not a valid Optional UTF-16 String, or there are insufficient bytes left to decode it as an Optional UTF-16 String.</p>", row + 1, column + 1)) }
                    }

                    // This type is just a recursive type.
                    FieldType::SequenceU16(definition) => {
                        let start = *index;
                        let end = *index;
                        let blob = &data[start..end];

                        if let Ok(entry_count) = data.decode_packedfile_integer_u16(*index, &mut index) {
                            let mut sub_table = Table::new(definition);
                            sub_table.decode(data, entry_count.into(), index, return_incomplete)?;
                            Ok(DecodedData::SequenceU16(Box::new(sub_table))) }
                        else { Err(anyhow!("<p>Error trying to get the Entry Count of<i><b>Row {}, Cell {}</b></i>: the value is not a valid U32, or there are insufficient bytes left to decode it as an U32 value.</p>", row + 1, column + 1)) }
                    }

                    // This type is just a recursive type.
                    FieldType::SequenceU32(definition) => {
                        if let Ok(entry_count) = data.decode_packedfile_integer_u32(*index, &mut index) {
                            let mut sub_table = Table::new(definition);
                            sub_table.decode(data, entry_count, index, return_incomplete)?;
                            Ok(DecodedData::SequenceU32(Box::new(sub_table))) }
                        else { Err(anyhow!("<p>Error trying to get the Entry Count of<i><b>Row {}, Cell {}</b></i>: the value is not a valid U32, or there are insufficient bytes left to decode it as an U32 value.</p>", row + 1, column + 1)) }
                    }
                };

                match decoded_cell {
                    Ok(data) =>  {

                        // If the field is a bitwise, split it into multiple fields. This is currently limited to integer types.
                        if field.is_bitwise() > 1 {
                            if [FieldType::I16, FieldType::I32, FieldType::I64].contains(field.field_type()) {
                                if let Ok(data) = data.parse::<i64>() {
                                    let values = (0..field.is_bitwise()).map(|bitwise_column| format!("{}", (data & (1 << bitwise_column) != 0) as u8)).collect::<Vec<_>>().join(",");
                                    row_values.push_str(&values);
                                }
                            }
                        }

                        // If the field has enum values, we turn it into a string. Same as before, only for integer types.
                        else if !field.enum_values().is_empty() {
                            if [FieldType::I16, FieldType::I32, FieldType::I64].contains(field.field_type()) {
                                if let Ok(data) = data.parse::<i64>() {
                                    match field.enum_values().get(&data) {
                                        Some(data) => row_values.push_str(&(data.to_owned() + ",")),
                                        None => row_values.push_str(&(data.to_string() + ","))
                                    }
                                }
                            }
                        }

                        // If the field is part of an split colour field group, don't add it. We'll separate it from the rest, then merge them into a ColourRGB field.
                        else if let Some(colour_index) = field.is_part_of_colour() {
                            if [FieldType::I16, FieldType::I32, FieldType::I64, FieldType::F32, FieldType::F64].contains(field.field_type()) {
                                if let Ok(data) = data.parse::<u8>() {

                                    // This can be r, g, b, red, green, blue.
                                    let colour_split = field.name().rsplitn(2, "_").collect::<Vec<&str>>();
                                    let colour_channel = colour_split[0].to_lowercase();
                                    match split_colour_fields.get_mut(&colour_index) {
                                        Some(colour_pack) => { colour_pack.insert(colour_channel, data); }
                                        None => {
                                            let mut colour_pack = HashMap::new();
                                            colour_pack.insert(colour_channel, data);
                                            split_colour_fields.insert(colour_index, colour_pack);
                                        }
                                    }
                                }
                            }
                        }

                        else {
                            row_values.push_str(data + ",");
                        }
                    },
                    Err(error) => if return_incomplete { return Err(ErrorKind::TableIncompleteError(format!("{}", error), serialize(self)?).into()) }
                    else { return Err(error.into()) }
                }
            }

            for split_colour in split_colour_fields.values() {
                let mut colour_hex = "".to_owned();
                if let Some(r) = split_colour.get("r") {
                    colour_hex.push_str(&format!("{:02X?}", r));
                }

                if let Some(r) = split_colour.get("red") {
                    colour_hex.push_str(&format!("{:02X?}", r));
                }

                if let Some(g) = split_colour.get("g") {
                    colour_hex.push_str(&format!("{:02X?}", g));
                }

                if let Some(g) = split_colour.get("green") {
                    colour_hex.push_str(&format!("{:02X?}", g));
                }

                if let Some(b) = split_colour.get("b") {
                    colour_hex.push_str(&format!("{:02X?}", b));
                }

                if let Some(b) = split_colour.get("blue") {
                    colour_hex.push_str(&format!("{:02X?}", b));
                }

                if u32::from_str_radix(&colour_hex, 16).is_ok() {
                    row_values.push_str(&(colour_hex + ","));
                } else {
                    return Err(anyhow!("Error decoding combined colour."));
                }
            }

            row_values.pop();
            row_values.push_str("),");

            query.push_str(&row_values);
        }

        // Remove the last comma, and set it so it replaces duplicates.
        query.pop();

        Ok(query)
    }*/
/*
    /// This function encodes all the fields of a table to raw bytes.
    fn encode(&self, mut packed_file: &mut Vec<u8>) -> Result<()> {
        let fields = self.definition.get_ref_fields();
        let fields_processed = self.definition.get_fields_processed();
        for row in &self.entries {

            // First, we need to make sure all rows we're going to encode are exactly what we expect.
            if row.len() != fields_processed.len() { return Err(ErrorKind::TableRowWrongFieldCount(fields_processed.len() as u32, row.len() as u32).into()) }
            let mut data_column = 0;

            let combined_colour_positions = fields.iter().filter_map(|field| {
                if field.get_is_part_of_colour().is_some() {
                    let colour_split = field.get_name().rsplitn(2, "_").collect::<Vec<&str>>();
                    let colour_field_name: String = if colour_split.len() == 2 { format!("{}{}", colour_split[1].to_lowercase(), MERGE_COLOUR_POST) } else { MERGE_COLOUR_NO_NAME.to_lowercase() };

                    self.definition.get_column_position_by_name(&colour_field_name).ok().map(|x| (colour_field_name, x))
                } else { None }
            }).collect::<HashMap<String, usize>>();

            for field in fields {
                if field.get_is_part_of_colour().is_some() {
                    let colour_split = field.get_name().rsplitn(2, "_").collect::<Vec<&str>>();
                    let colour_channel = colour_split[0].to_lowercase();
                    let colour_field_name = if colour_split.len() == 2 { format!("{}{}", colour_split[1].to_lowercase(), MERGE_COLOUR_POST) } else { MERGE_COLOUR_NO_NAME.to_lowercase() };

                    if let Some(data_column) = combined_colour_positions.get(&colour_field_name) {
                        match row[*data_column] {
                            DecodedData::ColourRGB(data) => {
                                let mut encoded = vec![];
                                encoded.encode_integer_u32(data);
                                let data = if colour_channel == "r" || colour_channel == "red" { encoded[2] }
                                else if colour_channel == "g" || colour_channel == "green" { encoded[1] }
                                else if colour_channel == "b" || colour_channel == "blue" { encoded[0] }
                                else { 0 };

                                match field.get_field_type() {
                                    FieldType::I16 => packed_file.encode_integer_i16(data as i16),
                                    FieldType::I32 => packed_file.encode_integer_i32(data as i32),
                                    FieldType::I64 => packed_file.encode_integer_i64(data as i64),
                                    FieldType::F32 => packed_file.encode_float_f32(data as f32),
                                    FieldType::F64 => packed_file.encode_float_f64(data as f64),
                                    _ => return Err(ErrorKind::TableWrongFieldType(format!("{}", row[*data_column]), format!("{}", field.get_ref_field_type())).into())
                                }

                            },
                            _ => return Err(ErrorKind::TableWrongFieldType(format!("{}", row[*data_column]), format!("{}", field.get_ref_field_type())).into())
                        }
                    }
                }

                else if field.get_is_bitwise() > 1 {
                    let mut data: i64 = 0;
                    for bitwise_column in 0..field.get_is_bitwise() {
                        if let DecodedData::Boolean(boolean) = row[data_column] {
                            if boolean {
                                data |= 1 << bitwise_column;
                            }
                        }

                        else {
                            return Err(ErrorKind::TableWrongFieldType(format!("{}", row[data_column]), format!("{}", field.get_ref_field_type())).into())
                        }

                        data_column += 1;
                    }

                    // If there are no problems, encode the data.
                    match field.get_field_type() {
                        FieldType::I16 => packed_file.encode_integer_i16(data as i16),
                        FieldType::I32 => packed_file.encode_integer_i32(data as i32),
                        FieldType::I64 => packed_file.encode_integer_i64(data),
                        _ => return Err(ErrorKind::TableWrongFieldType(format!("{}", row[data_column]), format!("{}", field.get_ref_field_type())).into())
                    }
                }

                else {

                    match row[data_column] {
                        DecodedData::Boolean(data) => packed_file.encode_bool(data),
                        DecodedData::F32(data) => packed_file.encode_float_f32(data),
                        DecodedData::F64(data) => packed_file.encode_float_f64(data),
                        DecodedData::I16(data) => packed_file.encode_integer_i16(data),
                        DecodedData::I32(data) => packed_file.encode_integer_i32(data),
                        DecodedData::I64(data) => packed_file.encode_integer_i64(data),
                        DecodedData::ColourRGB(data) => packed_file.encode_integer_colour_rgb(data),
                        DecodedData::StringU8(ref data) |
                        DecodedData::StringU16(ref data) |
                        DecodedData::OptionalStringU8(ref data) |
                        DecodedData::OptionalStringU16(ref data) => {

                            // If the field has enum values, try to match them. If the matching fails, try to just encode them.
                            // If that fails, put a default value on that cell.
                            let values = field.get_enum_values();
                            if !values.is_empty() {
                                let data = match values.iter().find(|(_, y)| y.to_lowercase() == data.to_lowercase()) {
                                    Some((x, _)) => {
                                        match field.get_field_type() {
                                            FieldType::I16 => DecodedData::I16(*x as i16),
                                            FieldType::I32 => DecodedData::I32(*x),
                                            FieldType::I64 => DecodedData::I64(*x as i64),
                                            _ => return Err(ErrorKind::TableWrongFieldType(format!("{}", row[data_column]), format!("{}", field.get_ref_field_type())).into())
                                        }
                                    }
                                    None => match row[data_column].convert_between_types(field.get_ref_field_type()) {
                                        Ok(data) => data,
                                        Err(_) => DecodedData::default(field.get_ref_field_type(), &field.get_default_value(None))
                                    }
                                };

                                // If there are no problems, encode the data.
                                match data {
                                    DecodedData::I16(data) => packed_file.encode_integer_i16(data),
                                    DecodedData::I32(data) => packed_file.encode_integer_i32(data),
                                    DecodedData::I64(data) => packed_file.encode_integer_i64(data),
                                    _ => return Err(ErrorKind::TableWrongFieldType(format!("{}", row[data_column]), format!("{}", field.get_ref_field_type())).into())
                                }
                            }
                            else {

                                // If there are no problems, encode the data.
                                match row[data_column] {
                                    DecodedData::StringU8(ref data) => packed_file.encode_packedfile_string_u8(&Self::unescape_special_chars(data)),
                                    DecodedData::StringU16(ref data) => packed_file.encode_packedfile_string_u16(&Self::unescape_special_chars(data)),
                                    DecodedData::OptionalStringU8(ref data) => packed_file.encode_packedfile_optional_string_u8(&Self::unescape_special_chars(data)),
                                    DecodedData::OptionalStringU16(ref data) => packed_file.encode_packedfile_optional_string_u16(&Self::unescape_special_chars(data)),
                                    _ => return Err(ErrorKind::TableWrongFieldType(format!("{}", row[data_column]), format!("{}", field.get_ref_field_type())).into())
                                }
                            }
                        }
                        DecodedData::SequenceU16(ref data) => {
                            if let FieldType::SequenceU16(_) = fields[data_column].get_ref_field_type() {
                                packed_file.encode_integer_u16(data.entries.len() as u16);
                                data.encode(&mut packed_file)?;
                            }
                        },
                        DecodedData::SequenceU32(ref data) => {
                            if let FieldType::SequenceU32(_) = fields[data_column].get_ref_field_type() {
                                packed_file.encode_integer_u32(data.entries.len() as u32);
                                data.encode(&mut packed_file)?;
                            }
                        },
                    }

                    data_column += 1;
                }
            }
        }

        Ok(())
    }

    /// This function returns a new empty row for the provided definition.
    pub fn get_new_row(definition: &Definition, table_name: Option<&str>) -> Vec<DecodedData> {
        definition.get_fields_processed().iter()
            .map(|field|
                match field.get_ref_field_type() {
                    FieldType::Boolean => {
                        if let Some(default_value) = field.get_default_value(table_name) {
                            if default_value.to_lowercase() == "true" {
                                vec![DecodedData::Boolean(true)]
                            } else {
                                vec![DecodedData::Boolean(false)]
                            }
                        } else {
                            vec![DecodedData::Boolean(false)]
                        }
                    }
                    FieldType::F32 => {
                        if let Some(default_value) = field.get_default_value(table_name) {
                            if let Ok(default_value) = default_value.parse::<f32>() {
                                vec![DecodedData::F32(default_value); 1]
                            } else {
                                vec![DecodedData::F32(0.0); 1]
                            }
                        } else {
                            vec![DecodedData::F32(0.0); 1]
                        }
                    },
                    FieldType::F64 => {
                        if let Some(default_value) = field.get_default_value(table_name) {
                            if let Ok(default_value) = default_value.parse::<f64>() {
                                vec![DecodedData::F64(default_value); 1]
                            } else {
                                vec![DecodedData::F64(0.0); 1]
                            }
                        } else {
                            vec![DecodedData::F64(0.0); 1]
                        }
                    },
                    FieldType::I16 => {
                        if let Some(default_value) = field.get_default_value(table_name) {
                            if let Ok(default_value) = default_value.parse::<i16>() {
                                vec![DecodedData::I16(default_value); 1]
                            } else {
                                vec![DecodedData::I16(0); 1]
                            }
                        } else {
                            vec![DecodedData::I16(0); 1]
                        }
                    },
                    FieldType::I32 => {
                        if let Some(default_value) = field.get_default_value(table_name) {
                            if let Ok(default_value) = default_value.parse::<i32>() {
                                vec![DecodedData::I32(default_value); 1]
                            } else {
                                vec![DecodedData::I32(0); 1]
                            }
                        } else {
                            vec![DecodedData::I32(0); 1]
                        }
                    },
                    FieldType::I64 => {
                        if let Some(default_value) = field.get_default_value(table_name) {
                            if let Ok(default_value) = default_value.parse::<i64>() {
                                vec![DecodedData::I64(default_value); 1]
                            } else {
                                vec![DecodedData::I64(0); 1]
                            }
                        } else {
                            vec![DecodedData::I64(0); 1]
                        }
                    },

                    // TODO: make this take a string as default value.
                    FieldType::ColourRGB => {
                        if let Some(default_value) = field.get_default_value(table_name) {
                            if let Ok(default_value) = u32::from_str_radix(&default_value, 16) {
                                vec![DecodedData::ColourRGB(default_value); 1]
                            } else {
                                vec![DecodedData::ColourRGB(0); 1]
                            }
                        } else {
                            vec![DecodedData::ColourRGB(0); 1]
                        }
                    },
                    FieldType::StringU8 => {
                        if let Some(default_value) = field.get_default_value(table_name) {
                            vec![DecodedData::StringU8(default_value.to_owned()); 1]
                        } else {
                            vec![DecodedData::StringU8(String::new()); 1]
                        }
                    }
                    FieldType::StringU16 => {
                        if let Some(default_value) = field.get_default_value(table_name) {
                            vec![DecodedData::StringU16(default_value.to_owned()); 1]
                        } else {
                            vec![DecodedData::StringU16(String::new()); 1]
                        }
                    }
                    FieldType::OptionalStringU8 => {
                        if let Some(default_value) = field.get_default_value(table_name) {
                            vec![DecodedData::OptionalStringU8(default_value.to_owned()); 1]
                        } else {
                            vec![DecodedData::OptionalStringU8(String::new()); 1]
                        }
                    }
                    FieldType::OptionalStringU16 => {
                        if let Some(default_value) = field.get_default_value(table_name) {
                            vec![DecodedData::OptionalStringU16(default_value.to_owned()); 1]
                        } else {
                            vec![DecodedData::OptionalStringU16(String::new()); 1]
                        }
                    },
                    FieldType::SequenceU16(ref definition) => vec![DecodedData::SequenceU16(Box::new(Table::new(definition))); 1],
                    FieldType::SequenceU32(ref definition) => vec![DecodedData::SequenceU32(Box::new(Table::new(definition))); 1]
                }
            )
            .flatten()
            .collect()

    }

    /// This function returns the list of table/columns that reference the provided columns, and if there may be a loc entry that changing our column may need a change.
    ///
    /// This supports more than one reference level, except for locs.
    /// TODO: Make loc editions be as deep as needed.
    pub fn get_tables_and_columns_referencing_our_own(
        schema_option: &Option<Schema>,
        table_name: &str,
        column_name: &str,
        definition: &Definition
    ) -> Option<(BTreeMap<String, Vec<String>>, bool)> {
        if let Some(ref schema) = *schema_option {

            // Make sure the table name is correct.
            let short_table_name = if table_name.ends_with("_tables") { table_name.split_at(table_name.len() - 7).0 } else { table_name };
            let mut tables: BTreeMap<String, Vec<String>> = BTreeMap::new();

            // We get all the db definitions from the schema, then iterate all of them to find what tables/columns reference our own.
            for versioned_file in schema.get_ref_versioned_file_db_all() {
                if let VersionedFile::DB(ref_table_name, ref_definition) = versioned_file {
                    let mut columns: Vec<String> = vec![];
                    for ref_version in ref_definition {
                        for ref_field in ref_version.get_fields_processed() {
                            if let Some((ref_ref_table, ref_ref_field)) = ref_field.get_is_reference() {

                                // As this applies to all versions of a table, skip repeated fields.
                                if ref_ref_table == short_table_name && ref_ref_field == column_name && !columns.iter().any(|x| x == ref_field.get_name()) {
                                    columns.push(ref_field.get_name().to_owned());

                                    // If we find a referencing column, get recursion working to check if there is any column referencing this one that needs to be edited.
                                    if let Some((ref_of_ref, _)) = Self::get_tables_and_columns_referencing_our_own(schema_option, ref_table_name, ref_field.get_name(), ref_version) {
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
                    }

                    // Only add them if we actually found columns.
                    if !columns.is_empty() {
                        tables.insert(ref_table_name.to_owned(), columns);
                    }
                }
            }

            // Also, check if we have to be careful about localised fields.
            let has_loc_fields = if let Some(field) = definition.get_fields_processed().iter().find(|x| x.get_name() == column_name) {
                (field.get_is_key() || field.get_name() == "key") && !definition.get_localised_fields().is_empty()
            } else { false };

            Some((tables, has_loc_fields))
        } else {
           None
        }
    }

    /// This function tries to find the row and column of the provided data, if it exists in this table.
    pub fn get_source_location_of_reference_data(
        &self,
        column_name: &str,
        row_data: &str
    ) -> Option<(usize, usize)> {
        if let Some(column_index) = self.get_ref_definition().get_fields_processed().iter().position(|x| x.get_name() == column_name) {
            for (row_index, row) in self.get_ref_table_data().iter().enumerate() {
                if let Some(cell_data) = row.get(column_index) {
                    if cell_data.data_to_string() == row_data {
                        return Some((column_index, row_index))
                    }
                }
            }
        }

        None
    }

    /// This function tries to find all rows with the provided data, if they exists in this table.
    pub fn get_location_of_reference_data(
        &self,
        column_name: &str,
        row_data: &str
    ) -> Option<(usize, Vec<usize>)> {
        let mut row_indexes = vec![];

        if let Some(column_index) = self.get_ref_definition().get_fields_processed().iter().position(|x| x.get_name() == column_name) {
            for (row_index, row) in self.get_ref_table_data().iter().enumerate() {
                if let Some(cell_data) = row.get(column_index) {
                    match cell_data {
                        DecodedData::StringU8(cell_data) |
                        DecodedData::StringU16(cell_data) |
                        DecodedData::OptionalStringU8(cell_data) |
                        DecodedData::OptionalStringU16(cell_data) => {
                            if cell_data == row_data {
                                row_indexes.push(row_index);
                            }
                        }
                        _ => {}
                    }
                }
            }

            if row_indexes.is_empty() {
                None
            } else {
                Some((column_index, row_indexes))
            }
        } else {
            None
        }

    } */

    //----------------------------------------------------------------//
    // TSV Functions for tables.
    //----------------------------------------------------------------//

    /// This function tries to imports a TSV file on the path provided into a binary db table.
    pub(crate) fn tsv_import(records: StringRecordsIter<File>, definition: &Definition, field_order: &HashMap<u32, String>, table_name: &str, schema_patches: Option<&DefinitionPatch>) -> Result<Self> {
        let mut table = Table::new(&definition, None, table_name, false);
        let mut entries = vec![];

        let fields_processed = definition.fields_processed();

        for (row, record) in records.enumerate() {
            match record {
                Ok(record) => {
                    let mut entry = Self::new_row(&definition, schema_patches);
                    for (column, field) in record.iter().enumerate() {

                        // Get the column name from the header, and try to map it to a column in the table's.
                        if let Some(column_name) = field_order.get(&(column as u32)) {
                            if let Some(column_number) = fields_processed.iter().position(|x| x.name() == column_name) {

                                entry[column_number] = match fields_processed[column_number].field_type() {
                                    FieldType::Boolean => parse_str_as_bool(field).map(|x| DecodedData::Boolean(x)).map_err(|_| RLibError::ImportTSVIncorrectRow(row, column))?,
                                    FieldType::F32 => DecodedData::F32(field.parse::<f32>().map_err(|_| RLibError::ImportTSVIncorrectRow(row, column))?),
                                    FieldType::F64 => DecodedData::F64(field.parse::<f64>().map_err(|_| RLibError::ImportTSVIncorrectRow(row, column))?),
                                    FieldType::I16 => DecodedData::I16(field.parse::<i16>().map_err(|_| RLibError::ImportTSVIncorrectRow(row, column))?),
                                    FieldType::I32 => DecodedData::I32(field.parse::<i32>().map_err(|_| RLibError::ImportTSVIncorrectRow(row, column))?),
                                    FieldType::I64 => DecodedData::I64(field.parse::<i64>().map_err(|_| RLibError::ImportTSVIncorrectRow(row, column))?),
                                    FieldType::OptionalI16 => DecodedData::OptionalI16(field.parse::<i16>().map_err(|_| RLibError::ImportTSVIncorrectRow(row, column))?),
                                    FieldType::OptionalI32 => DecodedData::OptionalI32(field.parse::<i32>().map_err(|_| RLibError::ImportTSVIncorrectRow(row, column))?),
                                    FieldType::OptionalI64 => DecodedData::OptionalI64(field.parse::<i64>().map_err(|_| RLibError::ImportTSVIncorrectRow(row, column))?),
                                    FieldType::ColourRGB => DecodedData::ColourRGB(u32::from_str_radix(field, 16).map(|x| x.to_string()).map_err(|_| RLibError::ImportTSVIncorrectRow(row, column))?),
                                    FieldType::StringU8 => DecodedData::StringU8(field.to_owned()),
                                    FieldType::StringU16 => DecodedData::StringU16(field.to_owned()),
                                    FieldType::OptionalStringU8 => DecodedData::OptionalStringU8(field.to_owned()),
                                    FieldType::OptionalStringU16 => DecodedData::OptionalStringU16(field.to_owned()),

                                    // For now fail on Sequences. These are a bit special and I don't know if the're even possible in TSV.
                                    // TODO: Export sequences as json strings or base64 strings.
                                    FieldType::SequenceU16(_) => return Err(RLibError::ImportTSVIncorrectRow(row, column).into()),
                                    FieldType::SequenceU32(_) => return Err(RLibError::ImportTSVIncorrectRow(row, column).into())
                                }
                            }
                        }
                    }
                    entries.push(entry);
                }
                Err(_) => return Err(RLibError::ImportTSVIncorrectRow(row, 0).into()),
            }
        }

        // If we reached this point without errors, we replace the old data with the new one and return success.
        table.set_data(None, &entries)?;
        Ok(table)
    }

    /// This function exports the provided data to a TSV file.
    pub(crate) fn tsv_export(&self, writer: &mut Writer<File>, table_path: &str) -> Result<()> {

        let fields_processed = self.definition().fields_processed();
        let fields_sorted = self.definition().fields_processed_sorted(true);
        let fields_sorted_properly = fields_sorted.iter()
            .map(|field_sorted| (fields_processed.iter().position(|field| field == field_sorted).unwrap(), field_sorted))
            .collect::<Vec<(_,_)>>();

        // We serialize the info of the table (name and version) in the first line, and the column names in the second one.
        let metadata = (format!("#{};{};{}", self.table_name(), self.definition().version(), table_path), vec![String::new(); fields_sorted_properly.len() - 1]);
        writer.serialize(fields_sorted_properly.iter().map(|(_, field)| field.name()).collect::<Vec<&str>>())?;
        writer.serialize(metadata)?;

        // Then we serialize each entry in the DB Table.
        let entries = self.data(&None)?;
        for entry in &*entries {
            let sorted_entry = fields_sorted_properly.iter()
                .map(|(index, _)| &entry[*index])
                .collect::<Vec<&DecodedData>>();
            writer.serialize(sorted_entry)?;
        }

        writer.flush().map_err(From::from)
    }

    /*
    /// This function imports a TSV file into a new Table File.
    fn import_tsv_to_binary_file(
        schema: &Schema,
        source_path: &Path,
        destination_path: &Path,
    ) -> Result<()> {

        // We want the reader to have no quotes, tab as delimiter and custom headers, because otherwise
        // Excel, Libreoffice and all the programs that edit this kind of files break them on save.
        let mut reader = ReaderBuilder::new()
            .delimiter(b'\t')
            .quoting(false)
            .has_headers(true)
            .flexible(true)
            .from_path(&source_path)?;

        // If we successfully load the TSV file into a reader, check the first line to get the column list.
        let field_order = reader.headers()?.iter().enumerate().map(|(x, y)| (x as u32, y.to_owned())).collect::<BTreeMap<u32, String>>();
        let mut entries = vec![];
        let mut fields_processed = vec![];
        let mut definition = Definition::new(-1);
        let mut table_type = String::new();
        for (row, record) in reader.records().enumerate() {
            if let Ok(record) = record {

                // The second line contains the TSV metadata.
                if row == 0 {

                    let has_legacy_structure = if let Some(table_type) = record.get(1) { table_type != "" } else { false };

                    // If we have at least 2 fields, use the legacy behavior.
                    let record_data = if has_legacy_structure {
                        record.iter().map(|x| x.to_owned()).collect::<Vec<String>>()
                    }

                    // Otherwise, use the new behavior.
                    else if let Some(metadata) = record.get(0) {
                        metadata.split(';').map(|x| x.to_owned()).collect::<Vec<String>>()
                    }

                    // Otherwise, is an error.
                    else {
                        return Err(RLibError::ImportTSVWrongTypeTable.into())
                    };

                    // Get the type and version of the table, then the definition.
                    table_type = if let Some(table_type) = record_data.get(0) {
                        let mut table_type = table_type.to_owned();
                        if table_type.starts_with("#") {
                            table_type.remove(0);
                        }
                        table_type
                    } else { return Err(RLibError::ImportTSVWrongTypeTable.into()) };
                    let table_version = if let Some(table_version) = record_data.get(1) { table_version.parse::<i32>().map_err(|_| Error::from(RLibError::ImportTSVInvalidVersion))? } else { return Err(ErrorKind::ImportTSVInvalidVersion.into()) };

                    definition = if table_type == loc::TSV_NAME_LOC { schema.get_ref_versioned_file_loc()?.get_version(table_version)?.clone() }
                    else { schema.get_ref_versioned_file_db(&table_type)?.get_version(table_version)?.clone() };
                    fields_processed = definition.get_fields_processed();
                }

                else {

                    let mut entry = Self::get_new_row(&definition, Some(&table_type));
                    for (column, field) in record.iter().enumerate() {

                        // Get the column name from the header, and try to map it to a column in the table's.
                        if let Some(column_name) = field_order.get(&(column as u32)) {
                            if let Some(column_number) = fields_processed.iter().position(|x| x.get_name() == column_name) {

                                entry[column_number] = match fields_processed[column_number].get_ref_field_type() {
                                    FieldType::Boolean => {
                                        let value = field.to_lowercase();
                                        if value == "true" || value == "1" { DecodedData::Boolean(true) }
                                        else if value == "false" || value == "0" { DecodedData::Boolean(false) }
                                        else { return Err(ErrorKind::ImportTSVIncorrectRow(row, column).into()); }
                                    }
                                    FieldType::F32 => DecodedData::F32(field.parse::<f32>().map_err(|_| Error::from(ErrorKind::ImportTSVIncorrectRow(row, column)))?),
                                    FieldType::F64 => DecodedData::F64(field.parse::<f64>().map_err(|_| Error::from(ErrorKind::ImportTSVIncorrectRow(row, column)))?),
                                    FieldType::I16 => DecodedData::I16(field.parse::<i16>().map_err(|_| Error::from(ErrorKind::ImportTSVIncorrectRow(row, column)))?),
                                    FieldType::I32 => DecodedData::I32(field.parse::<i32>().map_err(|_| Error::from(ErrorKind::ImportTSVIncorrectRow(row, column)))?),
                                    FieldType::I64 => DecodedData::I64(field.parse::<i64>().map_err(|_| Error::from(ErrorKind::ImportTSVIncorrectRow(row, column)))?),
                                    FieldType::ColourRGB => DecodedData::ColourRGB(u32::from_str_radix(field, 16).map_err(|_| Error::from(ErrorKind::ImportTSVIncorrectRow(row, column)))?),
                                    FieldType::StringU8 => DecodedData::StringU8(field.to_owned()),
                                    FieldType::StringU16 => DecodedData::StringU16(field.to_owned()),
                                    FieldType::OptionalStringU8 => DecodedData::OptionalStringU8(field.to_owned()),
                                    FieldType::OptionalStringU16 => DecodedData::OptionalStringU16(field.to_owned()),

                                    // For now fail on Sequences. These are a bit special and I don't know if the're even possible in TSV.
                                    FieldType::SequenceU16(_) => return Err(ErrorKind::ImportTSVIncorrectRow(row, column).into()),
                                    FieldType::SequenceU32(_) => return Err(ErrorKind::ImportTSVIncorrectRow(row, column).into())
                                }
                            }
                        }
                    }
                    entries.push(entry);
                }
            }

            else { return Err(ErrorKind::ImportTSVIncorrectRow(row, 0).into()); }
        }

        // If we reached this point without errors, we create the File in memory and add the entries to it.
        let data = if table_type == loc::TSV_NAME_LOC {
            let mut file = loc::Loc::new(&definition);
            file.set_table_data(&entries)?;
            file.save()
        }
        else {
            let mut file = db::DB::new(&table_type, None, &definition);
            file.set_table_data(&entries)?;
            file.save()
        }?;

        // Then, we try to write it on disk. If there is an error, report it.
        let mut file = BufWriter::new(File::create(&destination_path)?);
        file.write_all(&data)?;

        // If all worked, return success.
        Ok(())
    }

    /// This function exports the provided data to a TSV file.
    fn export_tsv(
        &self,
        path: &Path,
        table_name: &str,
        file_path: &[String],
    ) -> Result<()> {

        // Make sure the folder actually exists.
        let mut folder_path = path.to_path_buf();
        folder_path.pop();
        DirBuilder::new().recursive(true).create(&folder_path)?;

        // We want the writer to have no quotes, tab as delimiter and custom headers, because otherwise
        // Excel, Libreoffice and all the programs that edit this kind of files break them on save.
        let mut writer = WriterBuilder::new()
            .delimiter(b'\t')
            .quote_style(QuoteStyle::Never)
            .has_headers(false)
            .flexible(true)
            .from_path(path)?;

        let fields_sorted = self.definition.get_fields_sorted();
        let sorted_indexes = fields_sorted.iter()
            .map(|field_sorted| self.definition.get_fields_processed().iter().position(|field| field == field_sorted).unwrap())
            .collect::<Vec<usize>>();

        // We serialize the info of the table (name and version) in the first line, and the column names in the second one.
        let metadata = ("#".to_owned() + table_name + ";" + &self.definition.get_version().to_string() + ";" + &file_path.join("/"), (0..sorted_indexes.len() - 1).map(|_| "".to_owned()).collect::<Vec<String>>());
        writer.serialize(fields_sorted.iter().map(|x| x.get_name().to_owned()).collect::<Vec<String>>())?;
        writer.serialize(metadata)?;

        // Then we serialize each entry in the DB Table.
        for entry in &self.entries {
            let sorted_entry = sorted_indexes.iter()
                .map(|index| &entry[*index])
                .map(|data| if let DecodedData::ColourRGB(_) = data { DecodedData::StringU8(data.data_to_string()) } else { data.clone() })
                .collect::<Vec<DecodedData>>();
            writer.serialize(&sorted_entry)?;
        }

        writer.flush().map_err(From::from)
    }

    /// This function exports the provided file to a TSV file..
    fn export_tsv_from_binary_file(
        schema: &Schema,
        source_path: &Path,
        destination_path: &Path
    ) -> Result<()> {

        // We want the writer to have no quotes, tab as delimiter and custom headers, because otherwise
        // Excel, Libreoffice and all the programs that edit this kind of files break them on save.
        let mut writer = WriterBuilder::new()
            .delimiter(b'\t')
            .quote_style(QuoteStyle::Never)
            .has_headers(false)
            .flexible(true)
            .from_path(destination_path)?;

        // We don't know what type this file is, so we try to decode it as a Loc. If that fails, we try
        // to decode it as a DB using the name of his parent folder. If that fails too, run before it explodes!
        let mut file = BufReader::new(File::open(source_path)?);
        let mut data = vec![];
        file.read_to_end(&mut data)?;

        let (table_type, version, entries) = if let Ok(data) = loc::Loc::read(&data, schema, false) {
            (loc::TSV_NAME_LOC, data.get_definition().get_version(), data.get_table_data())
        }
        else {
            let table_type = source_path.parent().unwrap().file_name().unwrap().to_str().unwrap();
            if let Ok(data) = db::DB::read(&data, table_type, schema, false) { (table_type, data.get_definition().get_version(), data.get_table_data()) }
            else { return Err(ErrorKind::ImportTSVWrongTypeTable.into()) }
        };

        let definition = if table_type == loc::TSV_NAME_LOC { schema.get_ref_versioned_file_loc()?.get_version(version)?.clone() }
        else { schema.get_ref_versioned_file_db(table_type)?.get_version(version)?.clone() };

        let fields_sorted = definition.get_fields_sorted();
        let sorted_indexes = fields_sorted.iter()
            .map(|field_sorted| definition.get_fields_processed().iter().position(|field| field == field_sorted).unwrap())
            .collect::<Vec<usize>>();

        // We serialize the info of the table (name and version) in the first line, and the column names in the second one.
        let metadata = ("#".to_owned() + table_type + ";" + &version.to_string(), (0..sorted_indexes.len() - 1).map(|_| "".to_owned()).collect::<Vec<String>>());
        writer.serialize(fields_sorted.iter().map(|x| x.get_name().to_owned()).collect::<Vec<String>>())?;
        writer.serialize(metadata)?;

        // Then we serialize each entry in the DB Table.
        for entry in entries {
            let sorted_entry = sorted_indexes.iter()
                .map(|index| &entry[*index])
                .map(|data| if let DecodedData::ColourRGB(_) = data { DecodedData::StringU8(data.data_to_string()) } else { data.clone() })
                .collect::<Vec<DecodedData>>();
            writer.serialize(&sorted_entry)?;
        }

        writer.flush().map_err(From::from)
    }
*/

    //----------------------------------------------------------------//
    // Util functions for tables.
    //----------------------------------------------------------------//

    /// This function escapes certain characters of the provided string.
    fn escape_special_chars(data: &str)-> String {
         let mut output = Vec::with_capacity(data.len() + 10);
         for c in data.as_bytes() {
            match c {
                b'\n' => output.extend_from_slice(b"\\\\n"),
                b'\t' => output.extend_from_slice(b"\\\\t"),
                _ => output.push(*c),
            }
        }
        unsafe { String::from_utf8_unchecked(output) }
    }

    /// This function unescapes certain characters of the provided string.
    fn unescape_special_chars(data: &str)-> String {
         data.replace("\\\\t", "\t").replace("\\\\n", "\n")
    }
}
/*
/// Implementation of `From<&RawTable>` for `Table`.
impl From<&RawTable> for Table {
    fn from(raw_table: &RawTable) -> Self {
        if let Some(ref raw_definition) = raw_table.definition {
            let mut table = Self::new(&From::from(raw_definition));
            for row in &raw_table.rows {
                let mut entry = vec![];

                // Some games (Thrones, Attila, Rome 2 and Shogun 2) may have missing fields when said field is empty.
                // To compensate it, if we don't find a field from the definition in the table, we add it empty.
                for field_def in table.definition.get_ref_fields() {
                    let mut exists = false;
                    for field in &row.fields {
                        if field_def.get_name() == field.field_name {
                            exists = true;
                            entry.push(match field_def.get_ref_field_type() {
                                FieldType::Boolean => DecodedData::Boolean(field.field_data == "true" || field.field_data == "1"),
                                FieldType::F32 => DecodedData::F32(if let Ok(data) = field.field_data.parse::<f32>() { data } else { 0.0 }),
                                FieldType::F64 => DecodedData::F64(if let Ok(data) = field.field_data.parse::<f64>() { data } else { 0.0 }),
                                FieldType::I16 => DecodedData::I16(if let Ok(data) = field.field_data.parse::<i16>() { data } else { 0 }),
                                FieldType::I32 => DecodedData::I32(if let Ok(data) = field.field_data.parse::<i32>() { data } else { 0 }),
                                FieldType::I64 => DecodedData::I64(if let Ok(data) = field.field_data.parse::<i64>() { data } else { 0 }),
                                FieldType::ColourRGB => DecodedData::ColourRGB(if let Ok(data) = u32::from_str_radix(&field.field_data, 16) { data } else { 0 }),
                                FieldType::StringU8 => DecodedData::StringU8(if field.field_data == "Frodo Best Waifu" { String::new() } else { field.field_data.to_string() }),
                                FieldType::StringU16 => DecodedData::StringU16(if field.field_data == "Frodo Best Waifu" { String::new() } else { field.field_data.to_string() }),
                                FieldType::OptionalStringU8 => DecodedData::OptionalStringU8(if field.field_data == "Frodo Best Waifu" { String::new() } else { field.field_data.to_string() }),
                                FieldType::OptionalStringU16 => DecodedData::OptionalStringU16(if field.field_data == "Frodo Best Waifu" { String::new() } else { field.field_data.to_string() }),

                                // This type is not used in the raw tables so, if we find it, we skip it.
                                FieldType::SequenceU16(_) | FieldType::SequenceU32(_) => continue,
                            });
                            break;
                        }
                    }

                    // If the field doesn't exist, we create it empty.
                    if !exists {
                        entry.push(DecodedData::OptionalStringU8(String::new()));
                    }
                }
                table.entries.push(entry);
            }
            table
        }
        else {
            Self::new(&Definition::new(-100))
        }
    }
}
*/
