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

/// This is the name used in TSV-exported Loc files to identify them as Loc Files.
pub const TSV_NAME: &str = "Loc PackedFile";

//---------------------------------------------------------------------------//
//                              Enum & Structs
//---------------------------------------------------------------------------//

/// This stores the data of a decoded Localisation PackedFile in memory.
#[derive(PartialEq, Clone, Debug)]
pub struct Loc {

	/// A copy of the `Definition` this table uses, so we don't have to check the schema everywhere.
    definition: Definition,

    /// The decoded entries of the table. This list is a Vec(rows) of a Vec(fields of a row) of DecodedData (decoded field).
    entries: Vec<Vec<DecodedData>>,
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
        let entries = Table::decode(&definition.fields, &packed_file_data, entry_count, &mut index)?;
        
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

    /// This function returns a reference to the definition of this Loc Table.
    pub fn get_ref_definition(&self) -> &Definition {
        &self.definition
    }

    /// This function returns a reference to the entries of this Loc Table.
    pub fn get_ref_table_data(&self) -> &Vec<Vec<DecodedData>> {
        &self.entries
    }

    /// This function returns a copy of the definition of this Loc Table.
    pub fn get_definition(&self) -> Definition {
        self.definition.clone()
    }

    /// This function returns a copy of the entries of this Loc Table.
    pub fn get_table_data(&self) -> Vec<Vec<DecodedData>> {
        self.entries.to_vec()
    }

    /// This function replaces the definition of this table with the one provided.
    ///
    /// This updates the table's data to follow the format marked by the new definition, so you can use it to *update* the version of your table.
    pub fn set_definition(&mut self, new_definition: &Definition) {

        // It's simple: we compare both schemas, and get the original and final positions of each column.
        // If a row is new, his original position is -1. If has been removed, his final position is -1.
        let mut positions: Vec<(i32, i32)> = vec![];
        for (new_pos, new_field) in new_definition.fields.iter().enumerate() {
            if let Some(old_pos) = self.definition.fields.iter().position(|x| x.name == new_field.name) {
                positions.push((old_pos as i32, new_pos as i32))
            } else { positions.push((-1, new_pos as i32)); }
        }

        // Then, for each field in the old definition, check if exists in the new one.
        for (old_pos, old_field) in self.definition.fields.iter().enumerate() {
            if !new_definition.fields.iter().any(|x| x.name == old_field.name) { positions.push((old_pos as i32, -1)); }
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
                    entry.push(DecodedData::default(&self.definition.fields[*new_pos as usize].field_type));
                }

                // Otherwise, we got a moved column. Grab his field from the old data and put it in his new place.
                else {
                    entry.push(row[*old_pos as usize].clone());
                }
            }
            new_entries.push(entry);
        }

        // Then, we finally replace our definition and our data.
        self.definition = new_definition.clone();
        self.entries = new_entries;
    }

    /// This function replaces the data of this table with the one provided.
    ///
    /// This can (and will) fail if the data is not of the format defined by the definition of the table.
    pub fn set_table_data(&mut self, data: &[Vec<DecodedData>]) -> Result<()> {
        for row in data {

            // First, we need to make sure all rows we have are exactly what we expect.
            if row.len() != self.definition.fields.len() { Err(ErrorKind::TableRowWrongFieldCount(self.definition.fields.len() as u32, row.len() as u32))? } 
            for (index, cell) in row.iter().enumerate() {

                // Next, we need to ensure each file is of the type we expected.
                if !DecodedData::is_field_type_correct(cell, self.definition.fields[index].field_type.clone()) { 
                    Err(ErrorKind::TableWrongFieldType(format!("{}", cell), format!("{}", self.definition.fields[index].field_type)))? 
                }
            }
        }
        Ok(())
    }

    /// This function is used to optimize the size of a Loc Table.
    ///
    /// It scans every line to check if it's a vanilla line, and remove it in that case. Also, if the entire 
    /// file is composed of only vanilla lines, it marks the entire PackedFile for removal.
    pub fn optimize_table(&mut self, vanilla_tables: &[&Self]) -> bool {
        
        // For each vanilla table, if it's the same table/version as our own, we check 
        let mut new_entries = Vec::with_capacity(self.entries.len());
        for entry in &self.entries {
            for vanilla_entries in vanilla_tables.iter().filter(|x| x.definition.version == self.definition.version).map(|x| &x.entries) {
                if vanilla_entries.contains(entry) { 
                    new_entries.push(entry.to_vec());
                    continue;
                }
            }
        }

        // Then we overwrite the entries and return if the table is empty or now, so we can optimize it further at `PackedFile` level.        
        self.entries = new_entries;
        self.entries.is_empty()
    }
}
