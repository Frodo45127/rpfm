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
Module with all the code to interact with Loc Tables.

Loc Tables are the files which contain all the localisation strings used by the game.
They're just tables with a key, a text, and a boolean column.
!*/

use rpfm_error::{Error, ErrorKind, Result};

use crate::common::{decoder::Decoder, encoder::Encoder};
use super::DecodedData;
use crate::SCHEMA;
use crate::schema::*;

/// This represents the value that every LOC PackedFile has in their first 2 bytes.
const BYTEORDER_MARK: u16 = 65279; // FF FE

/// This represents the value that every LOC PackedFile has in their 2-5 bytes.
const PACKED_FILE_TYPE: &str = "LOC";

/// This represents the value that every LOC PackedFile has in their 6-10 bytes.
const PACKED_FILE_VERSION: i32 = 1;

//---------------------------------------------------------------------------//
//                              Enum & Structs
//---------------------------------------------------------------------------//

/// This stores the data of a decoded Localisation PackedFile in memory.
#[derive(Clone, Debug, Default)]
pub struct Loc {

    /// The decoded entries of the table. This list is a Vec(rows) of a Vec(fields of a row) of DecodedData (decoded field).
    pub entries: Vec<Vec<DecodedData>>,
}

//---------------------------------------------------------------------------//
//                           Implementation of Loc
//---------------------------------------------------------------------------//

/// Implementation of `Loc`.
impl Loc {

    /// This function creates a new empty `Loc` .
    pub fn new() -> Self {
        Self { entries: vec![] }
    }

    /// This function creates a new `Loc` from a `Vec<u8>`.
    pub fn read(packed_file_data: &[u8]) -> Result<Self> {

        // A valid Loc PackedFile has at least 14 bytes. This ensures they exists before anything else.
        if packed_file_data.len() < 14 { return Err(ErrorKind::LocPackedFileIsNotALocPackedFile)? }

        // More checks to ensure this is a valid Loc PAckedFile.
        if BYTEORDER_MARK != packed_file_data.decode_integer_u16(0)? { return Err(ErrorKind::LocPackedFileIsNotALocPackedFile)? }
        if PACKED_FILE_TYPE != packed_file_data.decode_string_u8(2, 3)? { return Err(ErrorKind::LocPackedFileIsNotALocPackedFile)? }
        if PACKED_FILE_VERSION != packed_file_data.decode_integer_i32(6)? { return Err(ErrorKind::LocPackedFileIsNotALocPackedFile)? }
        let entry_count = packed_file_data.decode_integer_u32(10)?;

        // Get all the entries and return the Loc.
        let schema: Schema = (*SCHEMA.lock().unwrap()).clone().ok_or_else(|| Error::from(ErrorKind::SchemaNotFound))?;
        let definition = schema.get_versioned_file_loc()?.get_version(PACKED_FILE_VERSION)?;
        let mut entries = vec![];
        let mut index = 14 as usize;
        for row in 0..entry_count {
            let mut decoded_row = vec![];
            for column in 0..definition.fields.len() {

                // Loc Tables only have cells of type StringU16 and Bool. No need to check the rest of the types.
                let decoded_cell = match &definition.fields[column].field_type {
                    FieldType::Boolean => {
                        if let Ok(data) = packed_file_data.decode_packedfile_bool(index, &mut index) { DecodedData::Boolean(data) }
                        else { return Err(ErrorKind::HelperDecodingEncodingError(format!("<p>Error trying to decode the <i><b>Row {}, Cell {}</b></i> as a <b><i>Boolean</b></i> value: the value is not a boolean, or there are insufficient bytes left to decode it as a boolean value.</p>", row + 1, column + 1)))? }
                    }
                    FieldType::StringU16 => {
                        if let Ok(mut data) = packed_file_data.decode_packedfile_string_u16(index, &mut index) { 
                            data = data.replace("\t", "\\t").replace("\n", "\\n");
                            DecodedData::StringU16(data)
                        }
                        else { return Err(ErrorKind::HelperDecodingEncodingError(format!("<p>Error trying to decode the <i><b>Row {}, Cell {}</b></i> as an <b><i>UTF-16 String</b></i> value: the value is not a valid UTF-16 String, or there are insufficient bytes left to decode it as an UTF-16 String.</p>", row + 1, column + 1)))? }
                    }

                    // If we have any other type, stop. Should never happen, but better to avoid these kind of problems...
                    _ => return Err(ErrorKind::LocPackedFileCorrupted)?
                };
                decoded_row.push(decoded_cell);
            }
            entries.push(decoded_row);
        }

        // If we are not in the last byte, it means we didn't parse the entire file, which means this file is corrupt.
        if index != packed_file_data.len() { return Err(ErrorKind::PackedFileSizeIsNotWhatWeExpect(packed_file_data.len(), index))? }
        Ok(Self { entries })
    }

    /// This function takes a `Loc` and encodes it to `Vec<u8>`.
    pub fn save(&self) -> Vec<u8> {

        // Create the vector to hold them all.
        let mut packed_file: Vec<u8> = vec![];

        // Encode the header.
        packed_file.encode_integer_u16(BYTEORDER_MARK);
        packed_file.encode_string_u8(PACKED_FILE_TYPE);
        packed_file.push(0);
        packed_file.encode_integer_i32(PACKED_FILE_VERSION);
        packed_file.encode_integer_u32(self.entries.len() as u32);

        // Encode the data. In Locs we only have StringU16 and Booleans, so we can safetly ignore the rest.
        for row in &self.entries {        
            for cell in row {
                match *cell {
                    DecodedData::Boolean(data) => packed_file.encode_bool(data),
                    DecodedData::StringU16(ref data) => packed_file.encode_packedfile_string_u16(&data.replace("\\t", "\t").replace("\\n", "\n")),
                    _ => unreachable!()
                }
            }
        }

        packed_file
    }
}
