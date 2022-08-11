//---------------------------------------------------------------------------//
// Copyright (c) 2017-2022 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Anims tables are tables containing data about unit animations.
//!
//! They're used for string translation in all Total War games since Empire. One thing to take into account
//! when you're using a language other than english is that in all games up to Troy, the game will only load
//! the main `localisation.loc` file. It'll not load individual loc files.
//!
//! # Loc Structure
//!
//! ## Header
//!
//! | Bytes | Type     | Data                                           |
//! | ----- | -------- | ---------------------------------------------- |
//! | 2     | [u16]    | Byteorder mark. Always 0xFF0xFE.               |
//! | 3     | StringU8 | FileType String. Always LOC.                   |
//! | 1     | [u8]     | Unknown, always 0. Maybe part of the fileType? |
//! | 4     | [u32]    | Version of the table. Always 1.                |
//! | 4     | [u32]    | Amount of entries on the table.                |
//!
//! ## Data
//!
//! | Bytes | Type            | Data              |
//! | ----- | --------------- | ----------------- |
//! | *     | Sized StringU16 | Localisation key. |
//! | *     | Sized StringU16 | Localised string. |
//! | 1     | [bool]          | Unknown.          |

use getset::{Getters, Setters};
use serde_derive::{Serialize, Deserialize};

use std::borrow::Cow;
use std::collections::BTreeMap;

use crate::binary::{ReadBytes, WriteBytes};
use crate::error::{RLibError, Result};
use crate::files::{DecodeableExtraData, Decodeable, EncodeableExtraData, Encodeable, table::{DecodedData, Table}};
use crate::schema::*;
use crate::utils::check_size_mismatch;

/// Base path of an animation table. This is an special type of bin, stored only in this folder.
pub const BASE_PATH: &str = "animations";

pub const EXTENSION: &str = "_tables.bin";

/// Size of the header of a MatchedCombat PackedFile.
pub const HEADER_SIZE: usize = 8;

#[cfg(test)] mod anims_table_test;

//---------------------------------------------------------------------------//
//                              Enum & Structs
//---------------------------------------------------------------------------//

/// This stores the data of a decoded matched combat file in memory.
#[derive(PartialEq, Clone, Debug, Getters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", set = "pub")]
pub struct AnimsTable {

    /// The table's data, containing all the stuff needed to decode/encode it.
    table: Table,
}

//---------------------------------------------------------------------------//
//                      Implementation of MatchedCombat
//---------------------------------------------------------------------------//

/// Implementation of `MatchedCombat`.
impl AnimsTable {

    /// This function creates a new empty `MatchedCombat`.
    pub fn new(definition: &Definition) -> Self {
        Self {
            table: Table::new(definition, "", false),
        }
    }

    /// This function returns the definition of a Loc table.
    pub(crate) fn new_definition(version: i32) -> Definition {
        let mut subdefinition = Definition::new(-1);
        let mut subfields = Vec::with_capacity(2);
        subfields.push(Field::new("uk_1".to_owned(), FieldType::StringU8, true, None, false, None, None, None, String::new(), 0, 0, BTreeMap::new(), None));
        subfields.push(Field::new("uk_1".to_owned(), FieldType::I32, true, None, false, None, None, None, String::new(), 0, 0, BTreeMap::new(), None));
        subdefinition.set_fields(subfields);

        let mut definition = Definition::new(version);
        let mut fields = Vec::with_capacity(6);
        fields.push(Field::new("unit_1_key".to_owned(), FieldType::StringU8, true, Some("PLACEHOLDER".to_owned()), false, None, None, None, String::new(), 0, 0, BTreeMap::new(), None));
        fields.push(Field::new("unit_1_text".to_owned(), FieldType::StringU8, false, Some("PLACEHOLDER".to_owned()), false, None, None, None, String::new(), 0, 0, BTreeMap::new(), None));
        fields.push(Field::new("unit_1_text".to_owned(), FieldType::StringU8, false, Some("PLACEHOLDER".to_owned()), false, None, None, None, String::new(), 0, 0, BTreeMap::new(), None));
        fields.push(Field::new("unit_1_text".to_owned(), FieldType::StringU8, false, Some("PLACEHOLDER".to_owned()), false, None, None, None, String::new(), 0, 0, BTreeMap::new(), None));
        fields.push(Field::new("unit_1_uk_1".to_owned(), FieldType::SequenceU32(Box::new(subdefinition)), true, None, false, None, None, None, String::new(), 0, 0, BTreeMap::new(), None));
        fields.push(Field::new("unit_1_uk_2".to_owned(), FieldType::I16, true, None, false, None, None, None, String::new(), 0, 0, BTreeMap::new(), None));

        definition.set_fields(fields);
        definition
    }

    /// This function returns a reference of the definition used by the Loc table.
    pub fn definition(&self) -> &Definition {
        self.table.definition()
    }

    /// This function returns a reference to the entries of this Loc table.
    pub fn data(&self) -> Result<Cow<[Vec<DecodedData>]>> {
        self.table.data(&None)
    }

/*
    /// This function replaces the definition of this table with the one provided.
    ///
    /// This updates the table's data to follow the format marked by the new definition, so you can use it to *update* the version of your table.
    pub fn set_definition(&mut self, new_definition: Definition) {
        self.table.set_definition(new_definition);
    }
    /// This function replaces the data of this table with the one provided.
    ///
    /// This can (and will) fail if the data is not of the format defined by the definition of the table.
    pub fn set_table_data(&mut self, data: &[Vec<DecodedData>]) -> Result<()> {
        self.table.set_table_data(data)
    }*/

    /// This function tries to read the header of a Matched Combat file from a reader.
    pub fn read_header<R: ReadBytes>(data: &mut R) -> Result<(i32, u32)> {

        // A valid Loc PackedFile has at least 14 bytes. This ensures they exists before anything else.
        if data.len()? < HEADER_SIZE as u64 {
            return Err(RLibError::DecodingMatchedCombatNotAMatchedCombatTable)
        }

        let version = data.read_i32()?;
        let entry_count = data.read_u32()?;

        Ok((version, entry_count))
    }
}

impl Decodeable for AnimsTable {

    fn decode<R: ReadBytes>(data: &mut R, extra_data: &Option<DecodeableExtraData>) -> Result<Self> {
        let extra_data = extra_data.as_ref().ok_or(RLibError::DecodingMissingExtraData)?;
        let table_name = extra_data.table_name.ok_or(RLibError::DecodingMissingExtraData)?;

        let (version, entry_count) = Self::read_header(data)?;
        let definition = Self::new_definition(version);
        let table = Table::decode(&None, data, &definition, Some(entry_count), true, table_name)?;

        // If we are not in the last byte, it means we didn't parse the entire file, which means this file is corrupt.
        check_size_mismatch(data.stream_position()? as usize, data.len()? as usize)?;

        Ok(Self {
            table,
        })
    }
}

impl Encodeable for AnimsTable {

    fn encode<W: WriteBytes>(&mut self, buffer: &mut W, _extra_data: &Option<EncodeableExtraData>) -> Result<()> {
        buffer.write_i32(*self.table.definition().version())?;
        buffer.write_u32(self.table.len(None)? as u32)?;

        self.table.encode(buffer, &None, &None)
    }
}

/// Implementation to create a `AnimsTable` from a `Table` directly.
impl From<Table> for AnimsTable {
    fn from(table: Table) -> Self {
        Self {
            table,
        }
    }
}
