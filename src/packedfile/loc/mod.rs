//---------------------------------------------------------------------------//
// Copyright (c) 2017-2019 Ismael Gutiérrez González. All rights reserved.
// 
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
// 
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

// In this file we define the PackedFile type Loc for decoding and encoding it.
// This is the type used by localisation files.

use crate::common::coding_helpers::*;
use crate::error::{ErrorKind, Result};
use super::DecodedData;

/// This const represents the value that every LOC PackedFile has in their first 2 bytes.
const BYTEORDER_MARK: u16 = 65279; // FF FE

/// This const represents the value that every LOC PackedFile has in their 2-5 bytes.
const PACKED_FILE_TYPE: &str = "LOC";

/// This const represents the value that every LOC PackedFile has in their 6-10 bytes.
const PACKED_FILE_VERSION: u32 = 1;

/// `Loc`: This stores the data of a decoded Localisation PackedFile in memory.
/// It stores the PackedFile's data in a Vec<LocEntry>.
#[derive(Clone, Debug)]
pub struct Loc {
    pub entries: Vec<Vec<DecodedData>>,
}

/// Implementation of "Loc".
impl Loc {

    /// This function creates a new empty Loc PackedFile.
    pub fn new() -> Self {
        Self { entries: vec![] }
    }

    /// This function creates a new decoded Loc from the data of a PackedFile.
    pub fn read(packed_file_data: &[u8]) -> Result<Self> {

        // A valid Loc PackedFile has at least 14 bytes. This ensures they exists before anything else.
        if packed_file_data.len() < 14 { return Err(ErrorKind::LocPackedFileIsNotALocPackedFile)? }

        // More checks to ensure this is a valid Loc PAckedFile.
        if BYTEORDER_MARK != decode_integer_u16(&packed_file_data[0..2])? { return Err(ErrorKind::LocPackedFileIsNotALocPackedFile)? }
        if PACKED_FILE_TYPE != decode_string_u8(&packed_file_data[2..5])? { return Err(ErrorKind::LocPackedFileIsNotALocPackedFile)? }
        if PACKED_FILE_VERSION != decode_integer_u32(&packed_file_data[6..10])? { return Err(ErrorKind::LocPackedFileIsNotALocPackedFile)? }
        let entry_count = decode_integer_u32(&packed_file_data[10..14])?;

        // Get all the entries and return the Loc.
        let mut entries = vec![];
        let mut index = 14 as usize;
        for _ in 0..entry_count {

            // Decode the three fields escaping \t and \n to avoid weird behavior.
            let mut entry = vec![];
            if index < packed_file_data.len() { 
                let mut key = decode_packedfile_string_u16(&packed_file_data[index..], &mut index)?;
                key = key.replace("\t", "\\t").replace("\n", "\\n");
                entry.push(DecodedData::StringU16(key));
            } else { return Err(ErrorKind::LocPackedFileCorrupted)? };

            if index < packed_file_data.len() { 
                let mut text = decode_packedfile_string_u16(&packed_file_data[index..], &mut index)?;
                text = text.replace("\t", "\\t").replace("\n", "\\n");
                entry.push(DecodedData::StringU16(text));
            } else { return Err(ErrorKind::LocPackedFileCorrupted)? };
            
            if index < packed_file_data.len() { 
                let tooltip = decode_packedfile_bool(packed_file_data[index], &mut index)?;
                entry.push(DecodedData::Boolean(tooltip));
            } else { return Err(ErrorKind::LocPackedFileCorrupted)? };
            
            entries.push(entry);
        }

        // If we are not in the last byte, it means we didn't parse the entire file, which means this file is corrupt.
        if index != packed_file_data.len() { return Err(ErrorKind::PackedFileSizeIsNotWhatWeExpect(packed_file_data.len(), index))? }

        Ok(Self { entries })

    }

    /// This function takes a LocHeader and a LocData and put them together in a Vec<u8>, encoding an
    /// entire LocFile ready to write on disk.
    pub fn save(&self) -> Vec<u8> {

        // Create the vector to hold them all.
        let mut packed_file: Vec<u8> = vec![];

        // Encode the header.
        packed_file.extend_from_slice(&encode_integer_u16(BYTEORDER_MARK));
        packed_file.extend_from_slice(&encode_string_u8(PACKED_FILE_TYPE));
        packed_file.push(0);
        packed_file.extend_from_slice(&encode_integer_u32(PACKED_FILE_VERSION));
        packed_file.extend_from_slice(&encode_integer_u32(self.entries.len() as u32));

        // Encode the data. In Locs we only have StringU16 and Booleans, so we can safetly ignore the rest.
        for row in &self.entries {        
            for cell in row {
                match *cell {
                    DecodedData::Boolean(data) => packed_file.push(encode_bool(data)),
                    DecodedData::StringU16(ref data) => packed_file.extend_from_slice(&encode_packedfile_string_u16(&data.replace("\\t", "\t").replace("\\n", "\n"))),
                    _ => unreachable!()
                }
            }
        }

        // And return the encoded PackedFile.
        packed_file
    }
}
