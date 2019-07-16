//---------------------------------------------------------------------------//
// Copyright (c) 2017-2019 Ismael Gutiérrez González. All rights reserved.
// 
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
// 
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

// In this file we define the PackedFile type DB for decoding and encoding it.
// This is the type used by database files.

// The structure of a header is:
// - (optional) 4 bytes for the GUID marker.
// - (optional) The GUID in u16 bytes, with the 2 first being his lenght in chars (bytes / 2).
// - (optional) 4 bytes for the Version marker, if it have it.
// - (optional) 4 bytes for the Version, in u32 reversed.
// 1 misteryous byte
// 4 bytes for the entry count, in u32 reversed.

use serde_derive::{Serialize, Deserialize};
use uuid::Uuid;

use rpfm_error::{ErrorKind, Result};

use super::DecodedData;
use crate::GAME_SELECTED;
use crate::common::coding_helpers::*;
use crate::schema::*;

/// These two const are the markers we need to check in the header of every DB file.
const GUID_MARKER: &[u8] = &[253, 254, 252, 255];
const VERSION_MARKER: &[u8] = &[252, 253, 254, 255];

/// `DB`: This stores the data of a decoded DB PackedFile in memory.
/// It stores the PackedFile divided in multiple parts:
/// - db_type: the name of the table's definition (usually, db/"this_name"/yourtable).
/// - version: the version of our tabledefinition used to decode/encode this table. If there is no VERSION_MARKER, we default to 0.
/// - mysterious_byte: don't know his use, but it's in all the tables.
/// - table_definition: a copy of the tabledefinition used by this table, so we don't have to check the schema everywhere.
/// - entries: a list of decoded entries. This list is a Vec(rows) of a Vec(fields of a row) of DecodedData (decoded field).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DB {
    pub db_type: String,
    pub version: i32,
    pub mysterious_byte: u8,
    pub table_definition: TableDefinition,
    pub entries: Vec<Vec<DecodedData>>,
}

/// Implementation of "DB".
impl DB {

    /// This function creates a new empty DB PackedFile.
    pub fn new(db_type: &str, version: i32, table_definition: &TableDefinition) -> Self {
        Self{
            db_type: db_type.to_owned(),
            version,
            mysterious_byte: 1,
            table_definition: table_definition.clone(),
            entries: vec![],
        }
    }

    /// This function creates a new decoded DB from a encoded PackedFile. This assumes the PackedFile is
    /// a DB PackedFile. It'll crash otherwise.
    pub fn read(
        packed_file_data: &[u8],
        db_type: &str,
        master_schema: &Schema
    ) -> Result<Self> {

        // Create the index that we'll use to decode the entire table.
        let mut index = 0;

        // Checks to ensure this is a decodeable DB Table.
        if packed_file_data.len() < 5 { return Err(ErrorKind::DBTableIsNotADBTable)? }

        // If there is a GUID_MARKER, skip it together with the GUID itself (4 bytes for the marker, 74 for the GUID).
        if &packed_file_data[index..(index + 4)] == GUID_MARKER { index += 78; }

        // If there is a VERSION_MARKER, we get the version. Otherwise, we default to 0.
        let version = 
            if (index + 4) < packed_file_data.len() {
                if &packed_file_data[index..(index + 4)] == VERSION_MARKER { 
                    if (index + 8) < packed_file_data.len() { 
                        index += 8;
                        decode_integer_i32(&packed_file_data[(index - 4)..(index)])?
                    } else { return Err(ErrorKind::DBTableIsNotADBTable)? }
                } else { 0 }
            } else { return Err(ErrorKind::DBTableIsNotADBTable)? };

        // We get the rest of the data from the header.
        let mysterious_byte = if (index) < packed_file_data.len() { packed_file_data[index] } else { return Err(ErrorKind::DBTableIsNotADBTable)? };
        index += 1;
        let entry_count = if (index + 4) <= packed_file_data.len() { decode_packedfile_integer_u32(&packed_file_data[(index)..(index + 4)], &mut index)? } else { return Err(ErrorKind::DBTableIsNotADBTable)? };

        // These tables use the not-yet-implemented type "List" in the following versions:
        // - models_artillery: 0,
        // - models_artilleries: 0,
        // - models_building: 0, 3, 7.
        // - models_naval: 0, 6, 11.
        // - models_sieges: 2, 3.
        // So we disable everything for any problematic version of these tables.
        // TODO: Implement the needed type for these tables.
        if (db_type == "models_artillery_tables" && version == 0) ||
            (db_type == "models_artilleries_tables" && version == 0) ||
            (db_type == "models_building_tables" && (version == 0 ||
                                                    version == 3 ||
                                                    version == 7)) ||
            (db_type == "models_naval_tables" && (version == 0 ||
                                                    version == 6 ||
                                                    version == 11)) ||
            (db_type == "models_sieges_tables" && (version == 2 ||
                                                    version == 3))
        { return Err(ErrorKind::DBTableContainsListField)? }

        // Try to get the table_definition for this table, if exists.
        if let Some(table_definition) = Self::get_schema(db_type, version, master_schema) {
            let mut entries = vec![];
            for row in 0..entry_count {

                let mut decoded_row = vec![];
                for column in 0..table_definition.fields.len() {

                    let decoded_cell = match table_definition.fields[column].field_type {
                        FieldType::Boolean => {
                            if packed_file_data.get(index).is_some() { 
                                if let Ok(data) = decode_packedfile_bool(packed_file_data[index], &mut index) { DecodedData::Boolean(data) }
                                else { return Err(ErrorKind::HelperDecodingEncodingError(format!("<p>Error trying to decode the <i><b>Row {}, Cell {}</b></i> as a <b><i>Boolean</b></i> value: the value is not a boolean.</p>", row + 1, column + 1)))? }}
                            else { return Err(ErrorKind::HelperDecodingEncodingError(format!("<p>Error trying to decode the <i><b>Row {}, Cell {}</b></i> as a <b><i>Boolean</b></i> value: insufficient bytes to decode.</p>", row + 1, column + 1)))? }
                        }
                        FieldType::Float => {
                            if packed_file_data.get(index + 3).is_some() {
                                if let Ok(data) = decode_packedfile_float_f32(&packed_file_data[index..(index + 4)], &mut index) { DecodedData::Float(data) }
                                else { return Err(ErrorKind::HelperDecodingEncodingError(format!("<p>Error trying to decode the <i><b>Row {}, Cell {}</b></i> as a <b><i>F32</b></i> value: the value is not a valid F32.</p>", row + 1, column + 1)))? }}
                            else { return Err(ErrorKind::HelperDecodingEncodingError(format!("<p>Error trying to decode the <i><b>Row {}, Cell {}</b></i> as a <b><i>F32</b></i> value: insufficient bytes to decode.</p>", row + 1, column + 1)))? }
                        }
                        FieldType::Integer => {
                            if packed_file_data.get(index + 3).is_some() {
                                if let Ok(data) = decode_packedfile_integer_i32(&packed_file_data[index..(index + 4)], &mut index) { DecodedData::Integer(data) }
                                else { return Err(ErrorKind::HelperDecodingEncodingError(format!("<p>Error trying to decode the <i><b>Row {}, Cell {}</b></i> as a <b><i>I32</b></i> value: the value is not a valid I32.</p>", row + 1, column + 1)))? }}
                            else { return Err(ErrorKind::HelperDecodingEncodingError(format!("<p>Error trying to decode the <i><b>Row {}, Cell {}</b></i> as a <b><i>I32</b></i> value: insufficient bytes to decode.</p>", row + 1, column + 1)))? }
                        }
                        FieldType::LongInteger => {
                            if packed_file_data.get(index + 7).is_some() {
                                if let Ok(data) = decode_packedfile_integer_i64(&packed_file_data[index..(index + 8)], &mut index) { DecodedData::LongInteger(data) }
                                else { return Err(ErrorKind::HelperDecodingEncodingError(format!("<p>Error trying to decode the <i><b>Row {}, Cell {}</b></i> as a <b><i>I64</b></i> value: the value is not a valid I64.</p>", row + 1, column + 1)))? }}
                            else { return Err(ErrorKind::HelperDecodingEncodingError(format!("<p>Error trying to decode the <i><b>Row {}, Cell {}</b></i> as a <b><i>I64</b></i> value: insufficient bytes to decode.</p>", row + 1, column + 1)))? }
                        }
                        FieldType::StringU8 => {
                            if packed_file_data.get(index + 1).is_some() { 
                                if let Ok(data) = decode_packedfile_string_u8(&packed_file_data[index..], &mut index) { DecodedData::StringU8(data) }
                                else { return Err(ErrorKind::HelperDecodingEncodingError(format!("<p>Error trying to decode the <i><b>Row {}, Cell {}</b></i> as an <b><i>UTF-8 String</b></i> value: the value is not a valid UTF-8 String.</p>", row + 1, column + 1)))? }}
                            else { return Err(ErrorKind::HelperDecodingEncodingError(format!("<p>Error trying to decode the <i><b>Row {}, Cell {}</b></i> as an <b><i>UTF-8 String</b></i> value: insufficient bytes to decode.</p>", row + 1, column + 1)))? }
                        }
                        FieldType::StringU16 => {
                            if packed_file_data.get(index + 1).is_some() { 
                                if let Ok(data) = decode_packedfile_string_u16(&packed_file_data[index..], &mut index) { DecodedData::StringU16(data) }
                                else { return Err(ErrorKind::HelperDecodingEncodingError(format!("<p>Error trying to decode the <i><b>Row {}, Cell {}</b></i> as an <b><i>UTF-16 String</b></i> value: the value is not a valid UTF-16 String.</p>", row + 1, column + 1)))? }}
                            else { return Err(ErrorKind::HelperDecodingEncodingError(format!("<p>Error trying to decode the <i><b>Row {}, Cell {}</b></i> as an <b><i>UTF-16 String</b></i> value: insufficient bytes to decode.</p>", row + 1, column + 1)))? }
                        }
                        FieldType::OptionalStringU8 => {
                            if packed_file_data.get(index).is_some() { 
                                if let Ok(data) = decode_packedfile_optional_string_u8(&packed_file_data[index..], &mut index) { DecodedData::OptionalStringU8(data) }
                                else { return Err(ErrorKind::HelperDecodingEncodingError(format!("<p>Error trying to decode the <i><b>Row {}, Cell {}</b></i> as an <b><i>Optional UTF-8 String</b></i> value: the value is not a valid Optional UTF-8 String.</p>", row + 1, column + 1)))? }}
                            else { return Err(ErrorKind::HelperDecodingEncodingError(format!("<p>Error trying to decode the <i><b>Row {}, Cell {}</b></i> as an <b><i>Optional UTF-8 String</b></i> value: insufficient bytes to decode.</p>", row + 1, column + 1)))? }
                        }
                        FieldType::OptionalStringU16 => {
                            if packed_file_data.get(index).is_some() { 
                                if let Ok(data) = decode_packedfile_optional_string_u16(&packed_file_data[index..], &mut index) { DecodedData::OptionalStringU16(data) }
                                else { return Err(ErrorKind::HelperDecodingEncodingError(format!("<p>Error trying to decode the <i><b>Row {}, Cell {}</b></i> as an <b><i>Optional UTF-16 String</b></i> value: the value is not a valid Optional UTF-16 String.</p>", row + 1, column + 1)))? }}
                            else { return Err(ErrorKind::HelperDecodingEncodingError(format!("<p>Error trying to decode the <i><b>Row {}, Cell {}</b></i> as an <b><i>Optional UTF-16 String</b></i> value: insufficient bytes to decode.</p>", row + 1, column + 1)))? }
                        }
                    };
                    decoded_row.push(decoded_cell);
                }
                entries.push(decoded_row);
            }

            // If we are not in the last byte, it means we didn't parse the entire file, which means this file is corrupt.
            if index != packed_file_data.len() { return Err(ErrorKind::PackedFileSizeIsNotWhatWeExpect(packed_file_data.len(), index))? }

            // If we've reached this, we've succesfully decoded the table.
            Ok(Self {
                db_type: db_type.to_owned(),
                version,
                mysterious_byte,
                table_definition: table_definition.clone(),
                entries,
            })
        }

        // Otherwise, we report the specific problem.
        else if entry_count == 0 { Err(ErrorKind::DBTableEmptyWithNoTableDefinition)? }
        else { Err(ErrorKind::SchemaTableDefinitionNotFound)? }
    }

    /// This function takes an entire DB and encode it to Vec<u8>, so it can be written in the disk.
    /// It returns a Vec<u8> with the entire DB encoded in it.
    pub fn save(&self) -> Vec<u8> {
        let mut packed_file: Vec<u8> = vec![];

        // Napoleon and Empire do not have GUID, and adding it to their tables crash both games.
        // So for those two games, we ignore the GUID_MARKER and the GUID itself.
        let game_selected = GAME_SELECTED.lock().unwrap().to_owned();
        if game_selected != "empire" && game_selected != "napoleon" {
            packed_file.extend_from_slice(GUID_MARKER);
            packed_file.extend_from_slice(&encode_packedfile_string_u16(&format!("{}", Uuid::new_v4())));
        }
        packed_file.extend_from_slice(VERSION_MARKER);
        packed_file.extend_from_slice(&encode_integer_i32(self.version));
        packed_file.push(self.mysterious_byte);
        packed_file.extend_from_slice(&encode_integer_u32(self.entries.len() as u32));

        for row in &self.entries {        
            for cell in row {
                match *cell {
                    DecodedData::Boolean(data) => packed_file.push(encode_bool(data)),
                    DecodedData::Float(data) => packed_file.extend_from_slice(&encode_float_f32(data)),
                    DecodedData::Integer(data) => packed_file.extend_from_slice(&encode_integer_i32(data)),
                    DecodedData::LongInteger(data) => packed_file.extend_from_slice(&encode_integer_i64(data)),
                    DecodedData::StringU8(ref data) => packed_file.extend_from_slice(&encode_packedfile_string_u8(data)),
                    DecodedData::StringU16(ref data) => packed_file.extend_from_slice(&encode_packedfile_string_u16(data)),
                    DecodedData::OptionalStringU8(ref data) => packed_file.extend_from_slice(&encode_packedfile_optional_string_u8(data)),
                    DecodedData::OptionalStringU16(ref data) => packed_file.extend_from_slice(&encode_packedfile_optional_string_u16(data)),
                }
            }
        }

        // Return the encoded PackedFile.
        packed_file
    }

    /// This functions returns the version and entry count of a DB Table, without decoding the entire table. It just emulates what the `read` function does.
    pub fn get_header_data(packed_file_data: &[u8]) -> Result<(i32, u32, usize)> {

        // Create the index that we'll use to decode the entire table.
        let mut index = 0;

        // Checks to ensure this is a decodeable DB Table.
        if packed_file_data.len() < 5 { return Err(ErrorKind::DBTableIsNotADBTable)? }

        // If there is a GUID_MARKER, skip it together with the GUID itself (4 bytes for the marker, 74 for the GUID).
        if &packed_file_data[index..(index + 4)] == GUID_MARKER { index += 78; }

        // If there is a VERSION_MARKER, we get the version. Otherwise, we default to 0.
        let version = 
            if (index + 4) < packed_file_data.len() {
                if &packed_file_data[index..(index + 4)] == VERSION_MARKER { 
                    if (index + 8) < packed_file_data.len() { 
                        index += 8;
                        decode_integer_i32(&packed_file_data[(index - 4)..(index)])?
                    } else { return Err(ErrorKind::DBTableIsNotADBTable)? }
                } else { 0 }
            } else { return Err(ErrorKind::DBTableIsNotADBTable)? };

        index += 1;
        let entry_count = if (index + 4) <= packed_file_data.len() { decode_packedfile_integer_u32(&packed_file_data[(index)..(index + 4)], &mut index)? } else { return Err(ErrorKind::DBTableIsNotADBTable)? };

        Ok((version, entry_count, index))
    }

    /// This function gets the schema corresponding to the table we passed it, if it exists.
    pub fn get_schema(db_name: &str, version: i32, schema: &Schema) -> Option<TableDefinition> {
        if let Some(index_table_definitions) = schema.get_table_definitions(db_name) {
            if let Some(index_table_versions) = schema.tables_definitions[index_table_definitions].get_table_version(version) {
                if !schema.tables_definitions[index_table_definitions].versions[index_table_versions].fields.is_empty() {
                    return Some(schema.tables_definitions[index_table_definitions].versions[index_table_versions].clone())
                }
            }
        }
        None
    }

    /// This function gets all the schemas corresponding to the table we passed it, if any of them exists.
    pub fn get_schema_versions_list(db_name: &str, schema: &Schema) -> Option<Vec<TableDefinition>> {
        if let Some(index_table_definitions) = schema.get_table_definitions(db_name) {
            if !schema.tables_definitions[index_table_definitions].versions.is_empty() {
                return Some(schema.tables_definitions[index_table_definitions].versions.to_vec())
            }
        }
        None
    }

    /// This function removes from the schema the version of a table with the provided version.
    pub fn remove_table_version(table_name: &str, version: i32, schema: &mut Schema) -> Result<()> {
        if let Some(index_table_definitions) = schema.get_table_definitions(table_name) {
            if let Some(index_table_versions) = schema.tables_definitions[index_table_definitions].get_table_version(version) {
                schema.tables_definitions[index_table_definitions].versions.remove(index_table_versions);
                return Ok(())
            }
            unreachable!();
        }
        unreachable!();
    }
}
