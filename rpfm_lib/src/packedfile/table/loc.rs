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

use rpfm_error::{ErrorKind, Result};

use crate::common::{decoder::Decoder, encoder::Encoder};
use super::DecodedData;
use super::Table;

use crate::schema::*;

/// This represents the value that every LOC PackedFile has in their first 2 bytes.
const BYTEORDER_MARK: u16 = 65279; // FF FE

/// This represents the value that every LOC PackedFile has in their 2-5 bytes. The sixth byte is always a 0.
const PACKED_FILE_TYPE: &str = "LOC";

//---------------------------------------------------------------------------//
//                              Enum & Structs
//---------------------------------------------------------------------------//

/// This stores the data of a decoded Localisation PackedFile in memory.
#[derive(Clone, Debug)]
pub struct Loc {

	/// A copy of the `Definition` this table uses, so we don't have to check the schema everywhere.
    pub definition: Definition,

    /// The decoded entries of the table. This list is a Vec(rows) of a Vec(fields of a row) of DecodedData (decoded field).
    pub entries: Vec<Vec<DecodedData>>,
}

//---------------------------------------------------------------------------//
//                           Implementation of Loc
//---------------------------------------------------------------------------//

/// Implementation of `Loc`.
impl Loc {

    /// This function creates a new empty `Loc` .
    pub fn new(definition: &Definition) -> Self {
        Self { 
        	definition: definition.clone(),
        	entries: vec![],
        }
    }

    /// This function creates a new `Loc` from a `Vec<u8>`.
    pub fn read(packed_file_data: &[u8], schema: &Schema) -> Result<Self> {

        // A valid Loc PackedFile has at least 14 bytes. This ensures they exists before anything else.
        if packed_file_data.len() < 14 { return Err(ErrorKind::LocPackedFileIsNotALocPackedFile)? }

        // More checks to ensure this is a valid Loc PAckedFile.
        if BYTEORDER_MARK != packed_file_data.decode_integer_u16(0)? { return Err(ErrorKind::LocPackedFileIsNotALocPackedFile)? }
        if PACKED_FILE_TYPE != packed_file_data.decode_string_u8(2, 3)? { return Err(ErrorKind::LocPackedFileIsNotALocPackedFile)? }
        let version = packed_file_data.decode_integer_i32(6)?;
        let entry_count = packed_file_data.decode_integer_u32(10)?;

        // Try to get the table_definition for this table, if exists.
        let versioned_file = schema.get_versioned_file_loc();
        if versioned_file.is_err() && entry_count == 0 { Err(ErrorKind::TableEmptyWithNoDefinition)? }
        let definition = versioned_file?.get_version(version);
        if definition.is_err() && entry_count == 0 { Err(ErrorKind::TableEmptyWithNoDefinition)? }
        let definition = definition?;

        // Then try to decode all the entries.
        let mut index = 14 as usize;
        let entries = Table::decode(&definition.fields, &packed_file_data, entry_count, &mut index)?;;
        
        // If we are not in the last byte, it means we didn't parse the entire file, which means this file is corrupt.
        if index != packed_file_data.len() { return Err(ErrorKind::PackedFileSizeIsNotWhatWeExpect(packed_file_data.len(), index))? }
        
        // If we've reached this, we've succesfully decoded the table.
        Ok(Self {
            definition: definition.clone(),
            entries,
        })
    }

    /// This function takes a `Loc` and encodes it to `Vec<u8>`.
    pub fn save(&self) -> Result<Vec<u8>> {

        // Create the vector to hold them all.
        let mut packed_file: Vec<u8> = vec![];

        // Encode the header.
        packed_file.encode_integer_u16(BYTEORDER_MARK);
        packed_file.encode_string_u8(PACKED_FILE_TYPE);
        packed_file.push(0);
        packed_file.encode_integer_i32(self.definition.version);
        packed_file.encode_integer_u32(self.entries.len() as u32);

        // Encode the data.
        Table::encode(&self.entries, &self.definition.fields, &mut packed_file)?;

        // Return the encoded `PackedFile`.
        Ok(packed_file)
    }
}
