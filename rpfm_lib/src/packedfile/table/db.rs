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

use bincode::deserialize;
use serde_derive::{Serialize, Deserialize};
use uuid::Uuid;

use std::io::{BufReader, Read};
use std::fs::File;

use rpfm_error::{ErrorKind, Result};

use super::DecodedData;
use crate::common::{decoder::Decoder, encoder::Encoder};
use crate::common::get_game_selected_pak_file;
use crate::GAME_SELECTED;
use crate::schema::*;
use super::Table;

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
    definition: Definition,
    
    /// The decoded entries of the table. This list is a Vec(rows) of a Vec(fields of a row) of DecodedData (decoded field).
    entries: Vec<Vec<DecodedData>>,
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
        schema: &Schema
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
        let versioned_file = schema.get_versioned_file_db(&name);
        if versioned_file.is_err() && entry_count == 0 { Err(ErrorKind::TableEmptyWithNoDefinition)? }
        let definition = versioned_file?.get_version(version);
        if definition.is_err() && entry_count == 0 { Err(ErrorKind::TableEmptyWithNoDefinition)? }
        let definition = definition?;

        // Then try to decode all the entries.
        let entries = Table::decode(&definition.fields, &packed_file_data, entry_count, &mut index)?;

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
    pub fn save(&self) -> Result<Vec<u8>> {
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

        Table::encode(&self.entries, &self.definition.fields, &mut packed_file)?;

        // Return the encoded PackedFile.
        Ok(packed_file)
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
        if packed_file_data.get_bytes_checked(0, 4)? == GUID_MARKER { index += 78; }

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

    /// This function loads the PAK file of the game selected (if exists) into memory.
    ///
    /// This is useful to help resolving dependencies.
    pub fn read_pak_file() -> Vec<Self> {

        // Create the empty list.
        let mut db_files = vec![];

        // Get all the paths we need.
        if let Ok(pak_file) = get_game_selected_pak_file(&*GAME_SELECTED.lock().unwrap()) {
            if let Ok(pak_file) = File::open(pak_file) {
                let mut pak_file = BufReader::new(pak_file);
                let mut data = vec![];
                if pak_file.read_to_end(&mut data).is_ok() {
                    if let Ok(pak_file) = deserialize(&data) {
                        db_files = pak_file;
                    }
                }
            }
        }

        // Return the fake DB Table list.
        db_files
    }


    /// This function returns a reference to the definition of this DB Table.
    pub fn get_ref_definition(&self) -> &Definition {
        &self.definition
    }

    /// This function returns a reference to the entries of this DB Table.
    pub fn get_ref_table_data(&self) -> &Vec<Vec<DecodedData>> {
        &self.entries
    }

    /// This function returns a copy of the definition of this DB Table.
    pub fn get_definition(&self) -> Definition {
        self.definition.clone()
    }

    /// This function returns a copy of the entries of this DB Table.
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
}
