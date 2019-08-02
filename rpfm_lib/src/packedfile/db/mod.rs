//---------------------------------------------------------------------------//
// Copyright (c) 2017-2019 Ismael Gutiérrez González. All rights reserved.
// 
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
// 
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

/*!
Module with all the code to interact with DB Tables.

DB Tables are the files which controls a lot of the parameters used ingame, like units data,
effects data, projectile parameters.... It's what modders use the most.
!*/

use serde_derive::{Serialize, Deserialize};
use uuid::Uuid;

use rpfm_error::{ErrorKind, Result};

use super::DecodedData;
use crate::GAME_SELECTED;
use crate::common::{decoder::Decoder, encoder::Encoder};
use crate::schema::*;

/// If this sequence is found, the DB Table has a GUID after it.
const GUID_MARKER: &[u8] = &[253, 254, 252, 255];

/// If this sequence is found, the DB Table has a version number after it.
const VERSION_MARKER: &[u8] = &[252, 253, 254, 255];

//---------------------------------------------------------------------------//
//                              Enum & Structs
//---------------------------------------------------------------------------//

/// This holds an entire DB Table decoded in memory.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DB {
    
    /// The name of the table. Not his literal file name, but the name of the table it represents, usually, db/"this_name"/yourtable. Needed to get his `Definition`.
    pub name: String,
    
    /// Don't know his use, but it's in all the tables I've seen, always being `1` or `0`.
    pub mysterious_byte: bool,
    
    /// A copy of the `Definition` this table uses, so we don't have to check the schema everywhere.
    pub definition: Definition,
    
    /// The decoded entries of the table. This list is a Vec(rows) of a Vec(fields of a row) of DecodedData (decoded field).
    pub entries: Vec<Vec<DecodedData>>,
}

//---------------------------------------------------------------------------//
//                           Implementation of DB
//---------------------------------------------------------------------------//

/// Implementation of `DB`.
impl DB {

    /// This function creates a new empty `DB` from a definition and his name.
    pub fn new(
        name: &str, 
        definition: &Definition
    ) -> Self {
        Self{
            name: name.to_owned(),
            mysterious_byte: true,
            definition: definition.clone(),
            entries: vec![],
        }
    }

    /// This function creates a `DB` from a `Vec<u8>`.
    pub fn read(
        packed_file_data: &[u8],
        name: &str,
        master_schema: &Schema
    ) -> Result<Self> {

        // Get the header of the `DB`.
        let (version, mysterious_byte, entry_count, mut index) = Self::get_header(&packed_file_data)?;

        // These tables use the not-yet-implemented type "List" in the following versions:
        // - models_artillery: 0,
        // - models_artilleries: 0,
        // - models_building: 0, 3, 7.
        // - models_naval: 0, 6, 11.
        // - models_sieges: 2, 3.
        // So we disable everything for any problematic version of these tables.
        // TODO: Implement the needed type for these tables.
        if (name == "models_artillery_tables" && version == 0) ||
            (name == "models_artilleries_tables" && version == 0) ||
            (name == "models_building_tables" && (version == 0 ||
                                                    version == 3 ||
                                                    version == 7)) ||
            (name == "models_naval_tables" && (version == 0 ||
                                                    version == 6 ||
                                                    version == 11)) ||
            (name == "models_sieges_tables" && (version == 2 ||
                                                    version == 3))
        { return Err(ErrorKind::DBTableContainsListField)? }

        // Try to get the table_definition for this table, if exists.
        let versioned_file = master_schema.get_versioned_file_db(&name);
        if versioned_file.is_err() { if entry_count == 0 { Err(ErrorKind::DBTableEmptyWithNoDefinition)? }}
        let definition = versioned_file?.get_version(version);
        if definition.is_err() { if entry_count == 0 { Err(ErrorKind::DBTableEmptyWithNoDefinition)? }}
        let definition = definition?;

        // Then try to decode all the entries. 
        let entries = Self::get_decoded_rows(&definition.fields, &packed_file_data, entry_count, &mut index)?;

        // If we are not in the last byte, it means we didn't parse the entire file, which means this file is corrupt.
        if index != packed_file_data.len() { return Err(ErrorKind::PackedFileSizeIsNotWhatWeExpect(packed_file_data.len(), index))? }

        // If we've reached this, we've succesfully decoded the table.
        Ok(Self {
            name: name.to_owned(),
            mysterious_byte,
            definition: definition.clone(),
            entries,
        })
    }

    /// This function takes a `DB` and encodes it to `Vec<u8>`.
    pub fn save(&self) -> Vec<u8> {
        let mut packed_file: Vec<u8> = vec![];

        // Napoleon and Empire do not have GUID, and adding it to their tables crash both games.
        // So for those two games, we ignore the GUID_MARKER and the GUID itself.
        let game_selected = GAME_SELECTED.lock().unwrap().to_owned();
        if game_selected != "empire" && game_selected != "napoleon" {
            packed_file.extend_from_slice(GUID_MARKER);
            packed_file.encode_packedfile_string_u16(&format!("{}", Uuid::new_v4()));
        }
        packed_file.extend_from_slice(VERSION_MARKER);
        packed_file.encode_integer_i32(self.definition.version);
        packed_file.encode_bool(self.mysterious_byte);
        packed_file.encode_integer_u32(self.entries.len() as u32);

        Self::set_decoded_rows(&self.entries, &mut packed_file);

        // Return the encoded PackedFile.
        packed_file
    }

    /// This functions decodes the header part of a `DB` from a `Vec<u8>`.
    ///
    /// The data returned is:
    /// - `version`: the version of this table.
    /// - `mysterious_byte`: don't know.
    /// - `entry_count`: amount of entries this `DB` has.
    /// - `index`: position where the header ends. Useful if you want to decode the data of the `DB` after this.
    pub fn get_header(packed_file_data:&[u8]) -> Result<(i32, bool, u32, usize)> {

        // 5 is the minimum amount of bytes a valid DB Table can have. If there is less, either the table is broken,
        // or the data is not from a DB Table.
        if packed_file_data.len() < 5 { return Err(ErrorKind::DBTableIsNotADBTable)? }

        // Create the index that we'll use to decode the entire table.
        let mut index = 0;

        // If there is a GUID_MARKER, skip it together with the GUID itself (4 bytes for the marker, 74 for the GUID).
        // About this GUID, it's something that gets randomly generated every time you export a table with DAVE. Not useful.
        if &packed_file_data.get_bytes_checked(0, 4)? == &GUID_MARKER { index += 78; }

        // If there is a VERSION_MARKER, we get the version (4 bytes for the marker, 4 for the version). Otherwise, we default to 0.
        let version = if packed_file_data.get_bytes_checked(index, 4)? == VERSION_MARKER {
            index += 4;
            packed_file_data.decode_packedfile_integer_i32(index, &mut index)?
        } else { 0 };

        // We get the rest of the data from the header.
        let mysterious_byte = packed_file_data.decode_packedfile_bool(index, &mut index)?;
        let entry_count = packed_file_data.decode_packedfile_integer_u32(index, &mut index)?;
        Ok((version, mysterious_byte, entry_count, index))
    }

    /// This function gets the fields data for the provided definition.
    fn get_decoded_rows(
        fields: &[Field],
        data: &[u8],
        entry_count: u32,
        mut index: &mut usize, 
    ) -> Result<Vec<Vec<DecodedData>>> {
        let mut entries = vec![];
        for row in 0..entry_count {
            let mut decoded_row = vec![];
            for column in 0..fields.len() {
                let decoded_cell = match &fields[column].field_type {
                    FieldType::Boolean => {
                        if let Ok(data) = data.decode_packedfile_bool(*index, &mut index) { DecodedData::Boolean(data) }
                        else { return Err(ErrorKind::HelperDecodingEncodingError(format!("<p>Error trying to decode the <i><b>Row {}, Cell {}</b></i> as a <b><i>Boolean</b></i> value: the value is not a boolean, or there are insufficient bytes left to decode it as a boolean value.</p>", row + 1, column + 1)))? }
                    }
                    FieldType::Float => {
                        if let Ok(data) = data.decode_packedfile_float_f32(*index, &mut index) { DecodedData::Float(data) }
                        else { return Err(ErrorKind::HelperDecodingEncodingError(format!("<p>Error trying to decode the <i><b>Row {}, Cell {}</b></i> as a <b><i>F32</b></i> value: the value is not a valid F32, or there are insufficient bytes left to decode it as a F32 value.</p>", row + 1, column + 1)))? }
                    }
                    FieldType::Integer => {
                        if let Ok(data) = data.decode_packedfile_integer_i32(*index, &mut index) { DecodedData::Integer(data) }
                        else { return Err(ErrorKind::HelperDecodingEncodingError(format!("<p>Error trying to decode the <i><b>Row {}, Cell {}</b></i> as a <b><i>I32</b></i> value: the value is not a valid I32, or there are insufficient bytes left to decode it as an I32 value.</p>", row + 1, column + 1)))? }
                    }
                    FieldType::LongInteger => {
                        if let Ok(data) = data.decode_packedfile_integer_i64(*index, &mut index) { DecodedData::LongInteger(data) }
                        else { return Err(ErrorKind::HelperDecodingEncodingError(format!("<p>Error trying to decode the <i><b>Row {}, Cell {}</b></i> as a <b><i>I64</b></i> value: either the value is not a valid I64, or there are insufficient bytes left to decode it as an I64 value.</p>", row + 1, column + 1)))? }
                    }
                    FieldType::StringU8 => {
                        if let Ok(data) = data.decode_packedfile_string_u8(*index, &mut index) { DecodedData::StringU8(data) }
                        else { return Err(ErrorKind::HelperDecodingEncodingError(format!("<p>Error trying to decode the <i><b>Row {}, Cell {}</b></i> as an <b><i>UTF-8 String</b></i> value: the value is not a valid UTF-8 String, or there are insufficient bytes left to decode it as an UTF-8 String.</p>", row + 1, column + 1)))? }
                    }
                    FieldType::StringU16 => {
                        if let Ok(data) = data.decode_packedfile_string_u16(*index, &mut index) { DecodedData::StringU16(data) }
                        else { return Err(ErrorKind::HelperDecodingEncodingError(format!("<p>Error trying to decode the <i><b>Row {}, Cell {}</b></i> as an <b><i>UTF-16 String</b></i> value: the value is not a valid UTF-16 String, or there are insufficient bytes left to decode it as an UTF-16 String.</p>", row + 1, column + 1)))? }
                    }
                    FieldType::OptionalStringU8 => {
                        if let Ok(data) = data.decode_packedfile_optional_string_u8(*index, &mut index) { DecodedData::OptionalStringU8(data) }
                        else { return Err(ErrorKind::HelperDecodingEncodingError(format!("<p>Error trying to decode the <i><b>Row {}, Cell {}</b></i> as an <b><i>Optional UTF-8 String</b></i> value: the value is not a valid Optional UTF-8 String, or there are insufficient bytes left to decode it as an Optional UTF-8 String.</p>", row + 1, column + 1)))? }    
                    }
                    FieldType::OptionalStringU16 => {
                        if let Ok(data) = data.decode_packedfile_optional_string_u16(*index, &mut index) { DecodedData::OptionalStringU16(data) }
                        else { return Err(ErrorKind::HelperDecodingEncodingError(format!("<p>Error trying to decode the <i><b>Row {}, Cell {}</b></i> as an <b><i>Optional UTF-16 String</b></i> value: the value is not a valid Optional UTF-16 String, or there are insufficient bytes left to decode it as an Optional UTF-16 String.</p>", row + 1, column + 1)))? }
                    }

                    // This type is just a recursive type.
                    FieldType::Sequence(fields) => {
                        if let Ok(entry_count) = data.decode_packedfile_integer_u32(*index, &mut index) { DecodedData::Sequence(Self::get_decoded_rows(&*fields, &data, entry_count, index)?) }
                        else { return Err(ErrorKind::HelperDecodingEncodingError(format!("<p>Error trying to get the Entry Count of<i><b>Row {}, Cell {}</b></i>: the value is not a valid U32, or there are insufficient bytes left to decode it as an U32 value.</p>", row + 1, column + 1)))? }
                    }
                };
                decoded_row.push(decoded_cell);
            }
            entries.push(decoded_row);
        }
        Ok(entries)
    }

    /// This function returns the encoded version of the provided entry list.
    fn set_decoded_rows(
        entries: &[Vec<DecodedData>],
        mut packed_file: &mut Vec<u8>, 
    ) {
        for row in entries {        
            for cell in row {
                match *cell {
                    DecodedData::Boolean(data) => packed_file.encode_bool(data),
                    DecodedData::Float(data) => packed_file.encode_float_f32(data),
                    DecodedData::Integer(data) => packed_file.encode_integer_i32(data),
                    DecodedData::LongInteger(data) => packed_file.encode_integer_i64(data),
                    DecodedData::StringU8(ref data) => packed_file.encode_packedfile_string_u8(data),
                    DecodedData::StringU16(ref data) => packed_file.encode_packedfile_string_u16(data),
                    DecodedData::OptionalStringU8(ref data) => packed_file.encode_packedfile_optional_string_u8(data),
                    DecodedData::OptionalStringU16(ref data) => packed_file.encode_packedfile_optional_string_u16(data),
                    DecodedData::Sequence(ref data) => {
                        packed_file.encode_integer_u32(data.len() as u32);
                        Self::set_decoded_rows(&data, &mut packed_file);
                    },
                }
            }
        }
    }
}
