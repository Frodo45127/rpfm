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
Module with all the code to interact with MatchedCombat.

Not really sure what it contains, but it may be useful.
!*/

use serde_json::to_string_pretty;
use serde_derive::{Serialize, Deserialize};

use rpfm_error::{ErrorKind, Result};

use crate::common::{decoder::Decoder, encoder::Encoder};
use super::DecodedData;
use super::Table;

use crate::schema::*;

/// Full path of a matched combat table. This is an special type of bin, so we identify it by his full path.
pub const PATH: [&str; 3] = [
    "animations",
    "matched_combat",
    "attila_generated.bin"
];

//---------------------------------------------------------------------------//
//                              Enum & Structs
//---------------------------------------------------------------------------//

/// This stores the data of a decoded MatchedCombat PackedFile in memory.
#[derive(PartialEq, Clone, Debug, Serialize, Deserialize)]
pub struct MatchedCombat {

    /// The table's data, containing all the stuff needed to decode/encode it.
    table: Table,
}

//---------------------------------------------------------------------------//
//                      Implementation of MatchedCombat
//---------------------------------------------------------------------------//

/// Implementation of `MatchedCombat`.
impl MatchedCombat {

    /// This function creates a new empty `MatchedCombat`.
    pub fn new(definition: &Definition) -> Self {
        Self {
            table: Table::new(definition),
        }
    }

    /// This function returns a copy of the definition of this MatchedCombat.
    pub fn get_definition(&self) -> Definition {
        self.table.get_definition()
    }

    /// This function returns a reference to the definition of this MatchedCombat Table.
    pub fn get_ref_definition(&self) -> &Definition {
        self.table.get_ref_definition()
    }

    /// This function returns a copy of the entries of this MatchedCombat Table.
    pub fn get_table_data(&self) -> Vec<Vec<DecodedData>> {
        self.table.get_table_data()
    }

    /// This function returns a reference to the entries of this MatchedCombat Table.
    pub fn get_ref_table_data(&self) -> &[Vec<DecodedData>] {
        self.table.get_ref_table_data()
    }

    /// This function returns the amount of entries in this MatchedCombat Table.
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

    /// This function creates a new `MatchedCombat` from a `Vec<u8>`.
    pub fn read(packed_file_data: &[u8], schema: &Schema, return_incomplete: bool) -> Result<Self> {

        let mut index = 0;
        let version = packed_file_data.decode_packedfile_integer_i32(index, &mut index)?;
        let entry_count = packed_file_data.decode_packedfile_integer_u32(index, &mut index)?;

        // Try to get the table_definition for this table, if exists.
        let versioned_file = schema.get_ref_versioned_file_matched_combat();
        if versioned_file.is_err() && entry_count == 0 { return Err(ErrorKind::TableEmptyWithNoDefinition.into()) }
        let definition = versioned_file?.get_version(version);
        if definition.is_err() && entry_count == 0 { return Err(ErrorKind::TableEmptyWithNoDefinition.into()) }
        let definition = definition?;

        // Then try to decode all the entries.
        let mut table = Table::new(&definition);
        table.decode(&packed_file_data, entry_count as u32, &mut index, return_incomplete)?;

        // If we are not in the last byte, it means we didn't parse the entire file, which means this file is corrupt.
        if index != packed_file_data.len() { return Err(ErrorKind::PackedFileSizeIsNotWhatWeExpect(packed_file_data.len(), index).into()) }

        // If we've reached this, we've succesfully decoded the table.
        Ok(Self {
            table,
        })
    }

    pub fn to_json(&self) -> String {
        to_string_pretty(&self).unwrap()
    }

    /// This function takes a `MatchedCombat` and encodes it to `Vec<u8>`.
    pub fn save(&self) -> Result<Vec<u8>> {

        // Create the vector to hold them all.
        let mut packed_file: Vec<u8> = vec![];
        packed_file.encode_integer_i32(self.table.definition.version);
        packed_file.encode_integer_u32(self.table.entries.len() as u32);
        self.table.encode(&mut packed_file)?;

        // Return the encoded `PackedFile`.
        Ok(packed_file)
    }
}
