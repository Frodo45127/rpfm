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

use csv::{ReaderBuilder, WriterBuilder, QuoteStyle};
use serde_derive::{Serialize, Deserialize};

use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

use crate::common::coding_helpers::*;
use crate::error::{Error, ErrorKind, Result};
use super::SerializableToTSV;

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
    pub entries: Vec<LocEntry>,
}

/// `LocEntry`: This stores an entry of a decoded Localisation PackedFile in memory.
/// It stores the entry's data in multiple parts:
/// - key: the "key" column of the entry.
/// - text: the text you'll see ingame.
/// - tooltip (bool): this one I believe it was to enable or disable certain lines ingame.
#[derive(PartialEq, Clone, Debug, Serialize, Deserialize)]
pub struct LocEntry {
    pub key: String,
    pub text: String,
    pub tooltip: bool,
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
            let mut key = if index < packed_file_data.len() { decode_packedfile_string_u16(&packed_file_data[index..], &mut index)? } else { return Err(ErrorKind::LocPackedFileCorrupted)? };
            let mut text = if index < packed_file_data.len() { decode_packedfile_string_u16(&packed_file_data[index..], &mut index)? } else { return Err(ErrorKind::LocPackedFileCorrupted)? };
            let tooltip = if index < packed_file_data.len() { decode_packedfile_bool(packed_file_data[index], &mut index)? } else { return Err(ErrorKind::LocPackedFileCorrupted)? };
            key = key.replace("\t", "\\t").replace("\n", "\\n");
            text = text.replace("\t", "\\t").replace("\n", "\\n");

            let entry = LocEntry::new(key, text, tooltip);
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

        // Encode the data.
        for entry in &self.entries {
            packed_file.append(&mut encode_packedfile_string_u16(&entry.key.replace("\\t", "\t").replace("\\n", "\n")));
            packed_file.append(&mut encode_packedfile_string_u16(&entry.text.replace("\\t", "\t").replace("\\n", "\n")));
            packed_file.push(encode_bool(entry.tooltip));
        }

        // And return the encoded PackedFile.
        packed_file
    }
}

/// Implementation of "LocDataEntry"
impl LocEntry {

    /// This function takes the key, text and tooltip values and makes a LocDataEntry with them.
    pub fn new(key: String, text: String, tooltip: bool) -> Self {
        Self {
            key,
            text,
            tooltip,
        }
    }
}

/// Implementation of `SerializableToTSV` for `LocData`.
impl SerializableToTSV for Loc {

    /// This function imports a TSV file and loads his contents into a Loc PackedFile.
    fn import_tsv(
        &mut self,
        tsv_file_path: &PathBuf,
        _db_name: &str
    ) -> Result<()> {

        // We want the reader to have no quotes, tab as delimiter and custom headers, because otherwise
        // Excel, Libreoffice and all the programs that edit this kind of files break them on save.
        match ReaderBuilder::new()
            .delimiter(b'\t')
            .quoting(false)
            .has_headers(false)
            .flexible(true)
            .from_path(&tsv_file_path) {

            Ok(mut reader) => {

                // If we succesfully read the TSV file into a reader, check the first two lines to ensure it's a valid Loc TSV.
                let mut entries = vec![];
                for (row, record) in reader.records().enumerate() {
                    if let Ok(record) = record {

                        if row == 0 { 
                            if record.get(0).unwrap_or("error") != "Loc PackedFile" {
                                return Err(ErrorKind::ImportTSVWrongTypeLoc)?;
                            }
                        }

                        // The second row is just to help people in other programs, not needed to be check.
                        else if row == 1 { continue }

                        // Then read the rest of the rows as a normal TSV.
                        else {
                            let mut entry = LocEntry::new(String::new(), String::new(), true);

                            if let Some(key) = record.get(0) { entry.key = key.to_owned(); } else { return Err(ErrorKind::ImportTSVIncorrectRow(row, 0))?; }
                            if let Some(text) = record.get(1) { entry.text = text.to_owned(); } else { return Err(ErrorKind::ImportTSVIncorrectRow(row, 1))?; }
                            if let Some(tooltip) = record.get(2) { 
                                let tooltip = tooltip.to_lowercase();
                                if tooltip == "true" || tooltip == "1" { entry.tooltip = true; }
                                else if tooltip == "false" || tooltip == "0" { entry.tooltip = false; }
                                else { return Err(ErrorKind::ImportTSVIncorrectRow(row, 2))?; }
                            }

                            entries.push(entry)
                        }
                    }

                    else { return Err(ErrorKind::ImportTSVIncorrectRow(row, 0))?; }
                }

                // If we reached this point without errors, we replace the old data with the new one and return success
                self.entries = entries;
                Ok(())
            }

            // If we couldn't read the TSV file, return error.
            Err(error) => Err(Error::from(error))
        }
    }

    /// This function creates a TSV file with the contents of a Loc PackedFile.
    fn export_tsv(
        &self, 
        packed_file_path: &PathBuf, 
        _db_info: (&str, i32)
    ) -> Result<()> {

        // We want the writer to have no quotes, tab as delimiter and custom headers, because otherwise
        // Excel, Libreoffice and all the programs that edit this kind of files break them on save.
        let mut writer = WriterBuilder::new()
            .delimiter(b'\t')
            .quote_style(QuoteStyle::Never)
            .has_headers(false)
            .flexible(true)
            .from_writer(vec![]);

        // The first two rows are info for RPFM, so we have to add them it manually. 
        writer.serialize("Loc PackedFile")?;
        writer.serialize(("Key", "Text", "Tooltip"))?;

        // Then we serialize each entry in the Loc PackedFile.
        for entry in &self.entries { writer.serialize(entry)?; }

        // Then, we try to write it on disk. If there is an error, report it.
        if let Ok(mut file) = File::create(&packed_file_path) {
            if file.write_all(String::from_utf8(writer.into_inner().unwrap())?.as_bytes()).is_err() { Err(ErrorKind::IOGeneric)? }
        } else { Err(ErrorKind::IOGeneric)? }

        Ok(())
    }
}
