//---------------------------------------------------------------------------//
// Copyright (c) 2017-2022 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Matched Combat files are tables containing data about matched animations between units.
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

use std::borrow::Cow;
use std::collections::BTreeMap;

use crate::binary::{ReadBytes, WriteBytes};
use crate::error::{RLibError, Result};
use crate::files::{DecodeableExtraData, Decodeable, EncodeableExtraData, Encodeable, table::{DecodedData, Table}};
use crate::schema::*;
use crate::utils::check_size_mismatch;

/// Size of the header of an AnimFragment PackedFile.
pub const HEADER_SIZE: usize = 0;

/// Base path of an animation table. This is an special type of bin, stored only in this folder.
pub const BASE_PATH: [&str; 1] = ["animations"];

/// Extension of AnimFragment PackedFiles.
pub const EXTENSIONS: [&str; 2] = [".frg", ".bin"];

#[cfg(test)] mod anim_fragment_test;

//---------------------------------------------------------------------------//
//                              Enum & Structs
//---------------------------------------------------------------------------//

/// This stores the data of a decoded matched combat file in memory.
#[derive(PartialEq, Clone, Debug, Getters, Setters)]
#[getset(get = "pub", set = "pub")]
pub struct AnimFragment {

    skeleton_1: String,
    skeleton_2: String,
    min_id: i32,
    max_id: i32,
    unknown_bool: bool,

    /// The table's data, containing all the stuff needed to decode/encode it.
    table: Table,
}

//---------------------------------------------------------------------------//
//                      Implementation of MatchedCombat
//---------------------------------------------------------------------------//

/// Implementation of `MatchedCombat`.
impl AnimFragment {

    /// This function creates a new empty `MatchedCombat`.
    pub fn new(definition: &Definition) -> Self {
        Self {
            skeleton_1: String::new(),
            skeleton_2: String::new(),
            min_id: 0,
            max_id: 0,
            unknown_bool: false,
            table: Table::new(definition, "", false),
        }
    }

    /// This function returns the definition of a Loc table.
    pub(crate) fn new_definition(version: i32) -> Definition {
        let mut definition = Definition::new(version);
        let mut fields = Vec::with_capacity(12);
        fields.push(Field::new("id".to_owned(), FieldType::I32, true, None, false, None, None, None, String::new(), 0, 0, BTreeMap::new(), None));
        fields.push(Field::new("slot".to_owned(), FieldType::I32, true, None, false, None, None, None, String::new(), 0, 0, BTreeMap::new(), None));

        fields.push(Field::new("file_name".to_owned(), FieldType::StringU8, true, None, false, None, None, None, String::new(), 0, 0, BTreeMap::new(), None));
        fields.push(Field::new("metadata".to_owned(), FieldType::StringU8, true, Some("PLACEHOLDER".to_owned()), false, None, None, None, String::new(), 0, 0, BTreeMap::new(), None));
        fields.push(Field::new("metadata_sound".to_owned(), FieldType::StringU8, false, Some("PLACEHOLDER".to_owned()), false, None, None, None, String::new(), 0, 0, BTreeMap::new(), None));
        fields.push(Field::new("skeleton_type".to_owned(), FieldType::StringU8, false, Some("PLACEHOLDER".to_owned()), false, None, None, None, String::new(), 0, 0, BTreeMap::new(), None));
        fields.push(Field::new("blend_in_time".to_owned(), FieldType::F32, true, None, false, None, None, None, String::new(), 0, 0, BTreeMap::new(), None));
        fields.push(Field::new("selection_weight".to_owned(), FieldType::F32, true, None, false, None, None, None, String::new(), 0, 0, BTreeMap::new(), None));
        fields.push(Field::new("unknown_3".to_owned(), FieldType::I32, true, None, false, None, None, None, String::new(), 0, 0, BTreeMap::new(), None));
        fields.push(Field::new("weapon_bone".to_owned(), FieldType::I32, true, None, false, None, None, None, String::new(), 0, 0, BTreeMap::new(), None));
        fields.push(Field::new("unknown_4".to_owned(), FieldType::StringU8, false, Some("PLACEHOLDER".to_owned()), false, None, None, None, String::new(), 0, 0, BTreeMap::new(), None));
        fields.push(Field::new("unknown_5".to_owned(), FieldType::Boolean, false, None, false, None, None, None, String::new(), 0, 0, BTreeMap::new(), None));

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
    pub fn set_definition(&mut self, new_definition: &Definition) {
        self.table.set_definition(new_definition);
    }

    /// This function replaces the data of this table with the one provided.
    ///
    /// This can (and will) fail if the data is not of the format defined by the definition of the table.
    pub fn set_table_data(&mut self, data: &[Vec<DecodedData>]) -> Result<()> {
        self.table.set_table_data(data)
    }
    */
    /// This function tries to read the header of a Matched Combat file from a reader.
    pub fn read_header<R: ReadBytes>(data: &mut R) -> Result<(String, String, i32, i32, bool, u32)> {

        // A valid Loc PackedFile has at least 14 bytes. This ensures they exists before anything else.
        if data.len()? < HEADER_SIZE as u64 {
            return Err(RLibError::DecodingMatchedCombatNotAMatchedCombatTable)
        }

        let skeleton_1 = data.read_sized_string_u8()?;
        let skeleton_2 = data.read_sized_string_u8()?;

        let min_id = data.read_i32()?;
        let max_id = data.read_i32()?;
        let unknown_bool = data.read_bool()?;
        let entry_count = data.read_u32()?;

        Ok((skeleton_1, skeleton_2, min_id, max_id, unknown_bool, entry_count))
    }
}

impl Decodeable for AnimFragment {

    fn decode<R: ReadBytes>(data: &mut R, extra_data: Option<DecodeableExtraData>) -> Result<Self> {
        let extra_data = extra_data.ok_or(RLibError::DecodingMissingExtraData)?;
        let table_name = extra_data.table_name.ok_or(RLibError::DecodingMissingExtraData)?;

        let (skeleton_1, skeleton_2, min_id, max_id, unknown_bool, entry_count) = Self::read_header(data)?;
        let definition = Self::new_definition(0);
        let table = Table::decode(&None, data, &definition, Some(entry_count), false, table_name)?;

        // If we are not in the last byte, it means we didn't parse the entire file, which means this file is corrupt.
        check_size_mismatch(data.stream_position()? as usize, data.len()? as usize)?;

        Ok(Self {
            skeleton_1,
            skeleton_2,
            min_id,
            max_id,
            unknown_bool,
            table,
        })
    }
}

impl Encodeable for AnimFragment {

    fn encode<W: WriteBytes>(&mut self, buffer: &mut W, _extra_data: Option<EncodeableExtraData>) -> Result<()> {
        buffer.write_sized_string_u8(&self.skeleton_1)?;
        buffer.write_sized_string_u8(&self.skeleton_2)?;
        buffer.write_i32(self.min_id)?;
        buffer.write_i32(self.max_id)?;
        buffer.write_bool(self.unknown_bool)?;

        buffer.write_u32(self.table.len(None)? as u32)?;

        self.table.encode(buffer, &None, &None, &None)
    }
}

/// Implementation to create a `AnimFragment` from a `Table` directly.
impl From<Table> for AnimFragment {
    fn from(table: Table) -> Self {
        Self {
            skeleton_1: String::new(),
            skeleton_2: String::new(),
            min_id: 0,
            max_id: 0,
            unknown_bool: false,
            table,
        }
    }
}
