//---------------------------------------------------------------------------//
// Copyright (c) 2017-2024 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Loc files are key/value (kinda) tables that contain localised Strings.
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

use csv::{StringRecordsIter, Writer};
use getset::{Getters, Setters};
use rayon::prelude::*;
use serde_derive::{Serialize, Deserialize};

use std::borrow::Cow;
use std::collections::{BTreeMap, HashMap};
use std::fs::File;

use crate::binary::{ReadBytes, WriteBytes};
use crate::error::{RLibError, Result};
use crate::files::{DecodeableExtraData, Decodeable, EncodeableExtraData, Encodeable, table::{DecodedData, local::TableInMemory, Table}};
use crate::schema::*;
use crate::utils::check_size_mismatch;

/// This represents the value that every Loc file has in their first 2 bytes.
const BYTEORDER_MARK: u16 = 65279; // FF FE

/// This represents the value that every Loc file has in their 2-5 bytes. The sixth byte is always a 0.
const FILE_TYPE: &str = "LOC";

/// Size of the header of a Loc file.
const HEADER_SIZE: usize = 14;

/// This is the name used in TSV-exported Loc files to identify them as Loc files.
pub(crate) const TSV_NAME_LOC: &str = "Loc";
pub(crate) const TSV_NAME_LOC_OLD: &str = "Loc PackedFile";

/// Extension used by Loc files.
pub const EXTENSION: &str = ".loc";

/// Version used by Loc files. We've only seen version 1 so far, so we stick with that one.
const VERSION: i32 = 1;

#[cfg(test)] mod loc_test;

//---------------------------------------------------------------------------//
//                              Enum & Structs
//---------------------------------------------------------------------------//

/// This stores the data of a decoded Localisation file in memory.
#[derive(PartialEq, Clone, Debug, Getters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", set = "pub")]
pub struct Loc {

    /// The table's data, containing all the stuff needed to decode/encode it.
    table: TableInMemory,
}

//---------------------------------------------------------------------------//
//                           Implementation of Loc
//---------------------------------------------------------------------------//

impl Default for Loc {
    fn default() -> Self {
        Self::new()
    }
}

/// Implementation of `Loc`.
impl Loc {

    /// This function creates a new empty `Loc`.
    pub fn new() -> Self {
        let definition = Self::new_definition();

        Self {
            table: TableInMemory::new(&definition, None, TSV_NAME_LOC),
        }
    }

    /// This function returns the definition of a Loc table.
    pub(crate) fn new_definition() -> Definition {
        let mut definition = Definition::new(VERSION, None);
        let fields = vec![
            Field::new("key".to_owned(), FieldType::StringU16, true, Some("PLACEHOLDER".to_owned()), false, None, None, None, String::new(), 0, 0, BTreeMap::new(), None),
            Field::new("text".to_owned(), FieldType::StringU16, false, Some("PLACEHOLDER".to_owned()), false, None, None, None, String::new(), 0, 0, BTreeMap::new(), None),
            Field::new("tooltip".to_owned(), FieldType::Boolean, false, Some("PLACEHOLDER".to_owned()), false, None, None, None, String::new(), 0, 0, BTreeMap::new(), None),
        ];
        definition.set_fields(fields);
        definition
    }

    /// This function returns a reference of the definition used by the Loc table.
    pub fn definition(&self) -> &Definition {
        self.table.definition()
    }

    /// This function returns a reference to the entries of this Loc table.
    pub fn data(&self) -> Cow<[Vec<DecodedData>]> {
        self.table.data()
    }

    /// This function returns a reference to the entries of this Loc table.
    ///
    /// Make sure to keep the table structure valid for the table definition.
    pub fn data_mut(&mut self) -> &mut Vec<Vec<DecodedData>> {
        self.table.data_mut()
    }

    /// This function returns a valid empty (with default values if any) row for this table.
    pub fn new_row(&self) -> Vec<DecodedData> {
        self.table().new_row()
    }

    /// This function replaces the data of this table with the one provided.
    ///
    /// This can (and will) fail if the data is not in the format defined by the definition of the table.
    pub fn set_data(&mut self, data: &[Vec<DecodedData>]) -> Result<()> {
        self.table.set_data(data)
    }

    /// This function returns the position of a column in a definition, or None if the column is not found.
    pub fn column_position_by_name(&self, column_name: &str) -> Option<usize> {
        self.table().column_position_by_name(column_name)
    }

    /// This function returns the amount of entries in this Loc Table.
    pub fn len(&self) -> usize {
        self.table.len()
    }

    /// This function returns if the Loc Table is empty.
    pub fn is_empty(&self) -> bool {
        self.table.is_empty()
    }

    /// This function replaces the definition of this table with the one provided.
    ///
    /// This updates the table's data to follow the format marked by the new definition, so you can use it to *update* the version of your table.
    pub fn set_definition(&mut self, new_definition: &Definition) {
        self.table.set_definition(new_definition);
    }

    /// This function tries to read the header of a Loc file from a reader.
    pub fn read_header<R: ReadBytes>(data: &mut R) -> Result<(i32, u32)> {

        // A valid Loc PackedFile has at least 14 bytes. This ensures they exists before anything else.
        if data.len()? < HEADER_SIZE as u64 {
            return Err(RLibError::DecodingLocNotALocTable)
        }

        // More checks to ensure this is a valid Loc file.
        if BYTEORDER_MARK != data.read_u16()? {
            return Err(RLibError::DecodingLocNotALocTable)
        }

        if FILE_TYPE != data.read_string_u8(3)? {
            return Err(RLibError::DecodingLocNotALocTable)
        }

        let _ = data.read_u8()?;
        let version = data.read_i32()?;
        let entry_count = data.read_u32()?;

        Ok((version, entry_count))
    }

    /// This function merges the data of a few Loc tables into a new Loc table.
    pub fn merge(sources: &[&Self]) -> Result<Self> {
        let mut new_table = Self::new();
        let sources = sources.par_iter()
            .map(|table| {
                let mut table = table.table().clone();
                table.set_definition(new_table.definition());
                table
            })
            .collect::<Vec<_>>();

        let new_data = sources.par_iter()
            .map(|table| table.data().to_vec())
            .flatten()
            .collect::<Vec<_>>();
        new_table.set_data(&new_data)?;

        Ok(new_table)
    }

    /// This function imports a TSV file into a decoded Loc file.
    pub fn tsv_import(records: StringRecordsIter<File>, field_order: &HashMap<u32, String>) -> Result<Self> {
        let definition = Self::new_definition();
        let table = TableInMemory::tsv_import(records, &definition, field_order, TSV_NAME_LOC, None)?;
        let loc = Loc::from(table);
        Ok(loc)
    }

    /// This function exports a decoded Loc file into a TSV file.
    pub fn tsv_export(&self, writer: &mut Writer<File>, table_path: &str) -> Result<()> {
        self.table.tsv_export(writer, table_path, true)
    }
}

impl Decodeable for Loc {

    fn decode<R: ReadBytes>(data: &mut R, _extra_data: &Option<DecodeableExtraData>) -> Result<Self> {

        // Version is always 1, so we ignore it.
        let (_version, entry_count) = Self::read_header(data)?;

        let definition = Self::new_definition();
        let table = TableInMemory::decode(data, &definition, &HashMap::new(), Some(entry_count), false, TSV_NAME_LOC)?;

        // If we are not in the last byte, it means we didn't parse the entire file, which means this file is corrupt.
        check_size_mismatch(data.stream_position()? as usize, data.len()? as usize)?;

        Ok(Self {
            table,
        })
    }
}

impl Encodeable for Loc {

    fn encode<W: WriteBytes>(&mut self, buffer: &mut W, _extra_data: &Option<EncodeableExtraData>) -> Result<()> {
        buffer.write_u16(BYTEORDER_MARK)?;
        buffer.write_string_u8(FILE_TYPE)?;
        buffer.write_u8(0)?;
        buffer.write_i32(*self.table.definition().version())?;
        buffer.write_u32(self.table.len() as u32)?;

        self.table.encode(buffer)
    }
}

/// Implementation to create a `Loc` from a `Table` directly.
impl From<TableInMemory> for Loc {
    fn from(mut table: TableInMemory) -> Self {
        table.set_table_name(TSV_NAME_LOC.to_owned());
        Self {
            table,
        }
    }
}
