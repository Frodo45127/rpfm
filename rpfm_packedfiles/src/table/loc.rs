//---------------------------------------------------------------------------//
// Copyright (c) 2017-2022 Ismael Gutiérrez González. All rights reserved.
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

use rayon::prelude::*;

use std::cmp::Ordering;
use std::collections::HashSet;
use std::path::Path;

use rpfm_error::{ErrorKind, Result};

use crate::common::{decoder::Decoder, encoder::Encoder};
use crate::packedfile::Dependencies;
use super::DecodedData;
use super::Table;

use crate::SCHEMA;
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

    /// This function returns a reference to the underlying table.
    pub fn get_ref_table(&self) -> &Table {
        &self.table
    }

    /// This function returns a copy of the entries of this Loc Table.
    pub fn get_table_data(&self) -> Vec<Vec<DecodedData>> {
        self.table.get_table_data()
    }

    /// This function returns a reference to the entries of this Loc Table.
    pub fn get_ref_table_data(&self) -> &[Vec<DecodedData>] {
        self.table.get_ref_table_data()
    }

    /// This function returns the position of a column in a definition, or an error if the column is not found.
    pub fn get_column_position_by_name(&self, column_name: &str) -> Result<usize> {
        self.table.get_column_position_by_name(column_name)
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

    /// This function returns a valid empty row for this table.
    pub fn get_new_row(&self) -> Vec<DecodedData> {
        Table::get_new_row(self.get_ref_definition(), None)
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
        let mut table = Table::new(definition);
        table.decode(packed_file_data, entry_count, &mut index, return_incomplete)?;

        // If we are not in the last byte, it means we didn't parse the entire file, which means this file is corrupt.
        if index != packed_file_data.len() { return Err(ErrorKind::PackedFileSizeIsNotWhatWeExpect(packed_file_data.len(), index).into()) }

        // If we've reached this, we've successfully decoded the table.
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
        packed_file.encode_integer_i32(self.table.definition.get_version());
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

        // For each vanilla table, if it's the same table/version as our own, we check it
        let mut entries = self.get_ref_table_data().to_vec();
        let definition = self.get_ref_definition();
        let first_key = definition.get_fields_sorted().iter().position(|x| x.get_is_key()).unwrap_or(0);

        // To do it faster, make a freaking big table with all the vanilla entries together.
        let vanilla_table = vanilla_tables.iter()
            .filter(|x| x.get_ref_definition().get_version() == definition.get_version())
            .map(|x| x.get_ref_table_data().to_vec())
            .flatten()
            .map(|x| serde_json::to_string(&x).unwrap())
            .collect::<HashSet<String>>();

        // Remove ITM and ITNR entries, sort the remaining ones by keys, and dedup them.
        let new_row = self.get_new_row();
        entries.retain(|entry| !vanilla_table.contains(&serde_json::to_string(entry).unwrap()) && entry != &new_row);

        // Sort the table so it can be dedupd. Sorting floats is a pain in the ass.
        entries.par_sort_by(|a, b| a[first_key].partial_cmp(&b[first_key]).unwrap_or(Ordering::Equal));
        entries.dedup();

        // Then we overwrite the entries and return if the table is empty or now, so we can optimize it further at `PackedFile` level.
        let _ = self.table.set_table_data(&entries);
        self.table.get_ref_table_data().is_empty()
    }

    /// This function returns the table/column/key from the provided key, if it exists in the current PackFile.
    ///
    /// We return the table without "_tables". Keep that in mind if you use this.
    pub fn get_source_location_of_loc_key(key: &str, dependencies: &Dependencies) -> Option<(String, String, String)> {
        if let Some(ref schema) = *SCHEMA.read().unwrap() {
            let key_split = key.split('_').map(|x| x.to_owned()).collect::<Vec<String>>();
            let mut table = String::new();

            // Loop to get the table.
            for (index, value) in key_split.iter().enumerate() {
                table.push_str(value);
                let temp_table = table.to_owned() + "_tables";
                if let Ok(definition) = schema.get_ref_last_definition_db(&temp_table, dependencies) {
                    let localised_fields = definition.get_localised_fields();
                    if !localised_fields.is_empty() && key_split.len() > index + 2 {
                        let mut field = String::new();

                        // Loop to get the column.
                        for (second_index, value) in key_split[index + 1..].iter().enumerate() {
                            field.push_str(value);
                            if localised_fields.iter().any(|x| x.get_name() == field) {

                                // If we reached this, the rest is the value.
                                let key_field = &key_split[index + second_index + 2..].join("_");
                                if let Some(field) = definition.get_fields_processed().iter().find(|x| (x.get_name() == "key" || x.get_name() == "id") && x.get_is_key()) {
                                    return Some((table, field.get_name().to_string(), key_field.to_owned()));
                                }
                            }
                            field.push('_');
                        }
                    }
                }
                table.push('_');
            }
        }

        None
    }

    /// This function imports a TSV file into a decoded table.
    pub fn import_tsv(
        schema: &Schema,
        path: &Path,
    ) -> Result<(Self, Option<Vec<String>>)> {
        let (table, file_path) = Table::import_tsv(schema, path)?;
        let loc = Loc::from(table);
        Ok((loc, file_path))
    }

    /// This function exports the provided data to a TSV file.
    pub fn export_tsv(
        &self,
        path: &Path,
        table_name: &str,
        file_path: &[String],
    ) -> Result<()> {
        self.table.export_tsv(path, table_name, file_path)
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
