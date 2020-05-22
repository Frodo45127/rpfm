//---------------------------------------------------------------------------//
// Copyright (c) 2017-2020 Ismael Gutiérrez González. All rights reserved.
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

use std::path::PathBuf;

use rpfm_error::{ErrorKind, Result};

use crate::common::{decoder::Decoder, encoder::Encoder};
use super::DecodedData;
use super::Table;

use crate::schema::*;

/// This represents the value that every LOC PackedFile has in their first 2 bytes.
const BYTEORDER_MARK: u16 = 65279; // FF FE

/// This represents the value that every LOC PackedFile has in their 2-5 bytes. The sixth byte is always a 0.
const PACKED_FILE_TYPE: &str = "LOC";

/// Size of the header of a LOC PackedFile.
pub const HEADER_SIZE: usize = 14;

/// This is the name used in TSV-exported Loc files to identify them as Loc Files.
pub const TSV_NAME_LOC: &str = "Loc PackedFile";

/// Extension used by Loc PackedFiles.
pub const EXTENSION: &str = ".loc";

//---------------------------------------------------------------------------//
//                              Enum & Structs
//---------------------------------------------------------------------------//

/// This stores the data of a decoded Localisation PackedFile in memory.
#[derive(PartialEq, Clone, Debug)]
pub struct Loc {

    /// The table's data, containing all the stuff needed to decode/encode it.
    table: Table,
}

//---------------------------------------------------------------------------//
//                           Implementation of Loc
//---------------------------------------------------------------------------//

/// Implementation of `Loc`.
impl Loc {

    /// This function creates a new empty `Loc` .
    pub fn new(definition: &Definition) -> Self {
        Self {
        	table: Table::new(definition),
        }
    }

    /// This function returns if the provided data corresponds to a LOC Table or not.
    pub fn is_loc(data: &[u8]) -> bool {
        if data.len() < HEADER_SIZE { return false }
        if BYTEORDER_MARK != data.decode_integer_u16(0).unwrap() { return false }
        if PACKED_FILE_TYPE != data.decode_string_u8(2, 3).unwrap() { return false }
        true
    }

    /// This function returns a copy of the definition of this Loc Table.
    pub fn get_definition(&self) -> Definition {
        self.table.get_definition()
    }

    /// This function returns a reference to the definition of this Loc Table.
    pub fn get_ref_definition(&self) -> &Definition {
        self.table.get_ref_definition()
    }

    /// This function returns a copy of the entries of this Loc Table.
    pub fn get_table_data(&self) -> Vec<Vec<DecodedData>> {
        self.table.get_table_data()
    }

    /// This function returns a reference to the entries of this Loc Table.
    pub fn get_ref_table_data(&self) -> &[Vec<DecodedData>] {
        self.table.get_ref_table_data()
    }

    /// This function returns the amount of entries in this Loc Table.
    pub fn get_entry_count(&self) -> usize {
        self.table.get_entry_count()
    }

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

    /// This function creates a new `Loc` from a `Vec<u8>`.
    pub fn read(packed_file_data: &[u8], schema: &Schema, return_incomplete: bool) -> Result<Self> {

        let (version, entry_count) = Self::read_header(packed_file_data)?;

        // Try to get the table_definition for this table, if exists.
        let versioned_file = schema.get_ref_versioned_file_loc();
        if versioned_file.is_err() && entry_count == 0 { return Err(ErrorKind::TableEmptyWithNoDefinition.into()) }
        let definition = versioned_file?.get_version(version);
        if definition.is_err() && entry_count == 0 { return Err(ErrorKind::TableEmptyWithNoDefinition.into()) }
        let definition = definition?;

        // Then try to decode all the entries.
        let mut index = HEADER_SIZE as usize;
        let mut table = Table::new(&definition);
        table.decode(&packed_file_data, entry_count, &mut index, return_incomplete)?;

        // If we are not in the last byte, it means we didn't parse the entire file, which means this file is corrupt.
        if index != packed_file_data.len() { return Err(ErrorKind::PackedFileSizeIsNotWhatWeExpect(packed_file_data.len(), index).into()) }

        // If we've reached this, we've succesfully decoded the table.
        Ok(Self {
            table,
        })
    }

    /// This function tries to read the header of a Loc PackedFile from raw data.
    pub fn read_header(packed_file_data: &[u8]) -> Result<(i32, u32)> {

        // A valid Loc PackedFile has at least 14 bytes. This ensures they exists before anything else.
        if packed_file_data.len() < HEADER_SIZE { return Err(ErrorKind::LocPackedFileIsNotALocPackedFile.into()) }

        // More checks to ensure this is a valid Loc PAckedFile.
        if BYTEORDER_MARK != packed_file_data.decode_integer_u16(0)? { return Err(ErrorKind::LocPackedFileIsNotALocPackedFile.into()) }
        if PACKED_FILE_TYPE != packed_file_data.decode_string_u8(2, 3)? { return Err(ErrorKind::LocPackedFileIsNotALocPackedFile.into()) }
        let version = packed_file_data.decode_integer_i32(6)?;
        let entry_count = packed_file_data.decode_integer_u32(10)?;

        Ok((version, entry_count))
    }

    /// This function takes a `Loc` and encodes it to `Vec<u8>`.
    pub fn save(&self) -> Result<Vec<u8>> {

        // Create the vector to hold them all.
        let mut packed_file: Vec<u8> = vec![];

        // Encode the header.
        packed_file.encode_integer_u16(BYTEORDER_MARK);
        packed_file.encode_string_u8(PACKED_FILE_TYPE);
        packed_file.push(0);
        packed_file.encode_integer_i32(self.table.definition.version);
        packed_file.encode_integer_u32(self.table.entries.len() as u32);

        // Encode the data.
        self.table.encode(&mut packed_file)?;

        // Return the encoded `PackedFile`.
        Ok(packed_file)
    }

    /// This function is used to optimize the size of a Loc Table.
    ///
    /// It scans every line to check if it's a vanilla line, and remove it in that case. Also, if the entire
    /// file is composed of only vanilla lines, it marks the entire PackedFile for removal.
    pub fn optimize_table(&mut self, vanilla_tables: &[&Self]) -> bool {

        // For each vanilla table, if it's the same table/version as our own, we check
        let mut new_entries = Vec::with_capacity(self.table.get_entry_count());
        let entries = self.get_ref_table_data();
        let definition = self.get_ref_definition();

        // To do it faster, make a freaking big table with all the vanilla entries together.
        let mut vanilla_table = vanilla_tables.iter()
            .filter(|x| x.get_ref_definition().version == definition.version)
            .map(|x| x.get_ref_table_data())
            .flatten();

        for entry in entries {
            if vanilla_table.find(|x| x == &entry).is_none() {
                new_entries.push(entry.to_vec());
            }
        }

        // Then we overwrite the entries and return if the table is empty or now, so we can optimize it further at `PackedFile` level.
        let _ = self.table.set_table_data(&new_entries);
        self.table.get_ref_table_data().is_empty()
    }

    /// This function imports a TSV file into a decoded table.
    pub fn import_tsv(
        definition: &Definition,
        path: &PathBuf,
        name: &str,
    ) -> Result<Self> {
        let table = Table::import_tsv(definition, path, name)?;
        Ok(Loc::from(table))
    }

    /// This function exports the provided data to a TSV file.
    pub fn export_tsv(
        &self,
        path: &PathBuf,
        table_name: &str,
    ) -> Result<()> {
        self.table.export_tsv(path, table_name)
    }
}

/// Implementation to create a `Loc` from a `Table`.
impl From<Table> for Loc {
    fn from(table: Table) -> Self {
        Self {
            table,
        }
    }
}
