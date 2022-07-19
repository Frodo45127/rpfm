//---------------------------------------------------------------------------//
// Copyright (c) 2017-2022 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Loc files are key/value (kinda) tables that contain localised Strings.
//!
//! They're used for string translation in all Total War games since Empire. One thing to take into account
//! when you're using a language other than english is that in all games up to Troy, the game will only load
//! the main `localisation.loc` file. It'll not load individual loc files.
//!
//! # Loc Structure
//!
//! ## Header
//!
//! | Bytes | Type     | Data                                           |
//! | ----- | -------- | ---------------------------------------------- |
//! | 2     | [u16]    | Byteorder mark. Always 0xFF0xFE.               |
//! | 3     | StringU8 | FileType String. Always LOC.                   |
//! | 1     | [u8]     | Unknown, always 0. Maybe part of the fileType? |
//! | 4     | [u32]    | Version of the table. Always 1.                |
//! | 4     | [u32]    | Amount of entries on the table.                |
//!
//! ## Data
//!
//! | Bytes | Type            | Data              |
//! | ----- | --------------- | ----------------- |
//! | *     | Sized StringU16 | Localisation key. |
//! | *     | Sized StringU16 | Localised string. |
//! | 1     | [bool]          | Unknown.          |

use getset::{Getters, Setters};
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use serde_derive::{Serialize, Deserialize};

use std::borrow::Cow;
use std::collections::BTreeMap;

use crate::binary::{ReadBytes, WriteBytes};
use crate::error::{RLibError, Result};
use crate::files::{DecodeableExtraData, Decodeable, EncodeableExtraData, Encodeable, table::{DecodedData, Table}};
use crate::schema::*;
use crate::utils::check_size_mismatch;

/// This represents the value that every Loc file has in their first 2 bytes.
const BYTEORDER_MARK: u16 = 65279; // FF FE

/// This represents the value that every Loc file has in their 2-5 bytes. The sixth byte is always a 0.
const FILE_TYPE: &str = "LOC";

/// Size of the header of a Loc file.
const HEADER_SIZE: usize = 14;

/// This is the name used in TSV-exported Loc files to identify them as Loc files.
const TSV_NAME_LOC: &str = "Loc";

/// Extension used by Loc files.
pub const EXTENSION: &str = ".loc";

/// Version used by Loc files. We've only seen version 1 so far, so we stick with that one.
const VERSION: i32 = 1;

/// Name of the internal table name, in case we use the SQL Backend.
const SQL_TABLE_NAME: &str = "localisation";

#[cfg(test)] mod loc_test;

//---------------------------------------------------------------------------//
//                              Enum & Structs
//---------------------------------------------------------------------------//

/// This stores the data of a decoded Localisation file in memory.
#[derive(PartialEq, Clone, Debug, Getters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", set = "pub")]
pub struct Loc {

    /// The table's data, containing all the stuff needed to decode/encode it.
    table: Table,
}

//---------------------------------------------------------------------------//
//                           Implementation of Loc
//---------------------------------------------------------------------------//

/// Implementation of `Loc`.
impl Loc {

    /// This function creates a new empty `Loc`.
    pub fn new(use_sql_backend: bool) -> Self {
        let definition = Self::new_definition();

        Self {
            table: Table::new(&definition, SQL_TABLE_NAME, use_sql_backend),
        }
    }

    /// This function returns the definition of a Loc table.
    pub(crate) fn new_definition() -> Definition {
        let mut definition = Definition::new(VERSION);
        let mut fields = Vec::with_capacity(3);
        fields.push(Field::new("key".to_owned(), FieldType::StringU16, true, Some("PLACEHOLDER".to_owned()), false, None, None, None, String::new(), 0, 0, BTreeMap::new(), None));
        fields.push(Field::new("text".to_owned(), FieldType::StringU16, false, Some("PLACEHOLDER".to_owned()), false, None, None, None, String::new(), 0, 0, BTreeMap::new(), None));
        fields.push(Field::new("tooltip".to_owned(), FieldType::Boolean, false, Some("PLACEHOLDER".to_owned()), false, None, None, None, String::new(), 0, 0, BTreeMap::new(), None));
        definition.set_fields(fields);
        definition
    }

    /// This function returns a reference of the definition used by the Loc table.
    pub fn definition(&self) -> &Definition {
        self.table.definition()
    }

    /// This function returns a reference to the entries of this Loc table.
    pub fn data(&self, pool: &Option<&Pool<SqliteConnectionManager>>) -> Result<Cow<[Vec<DecodedData>]>> {
        self.table.data(pool)
    }

    /*
    /// This function returns if the provided data corresponds to a LOC Table or not.
    pub fn is_loc(data: &[u8]) -> bool {
        if data.len() < HEADER_SIZE { return false }
        if BYTEORDER_MARK != data.decode_integer_u16(0).unwrap() { return false }
        if PACKED_FILE_TYPE != data.decode_string_u8(2, 3).unwrap() { return false }
        true
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

    */
    /// This function tries to read the header of a Loc file from a reader.
    pub fn read_header<R: ReadBytes>(data: &mut R) -> Result<(i32, u32)> {

        // A valid Loc PackedFile has at least 14 bytes. This ensures they exists before anything else.
        if data.len()? < HEADER_SIZE as u64 {
            return Err(RLibError::DecodingLocNotALocTable)
        }

        // More checks to ensure this is a valid Loc file.
        if BYTEORDER_MARK != data.read_u16()? {
            return Err(RLibError::DecodingLocNotALocTable)
        }

        if FILE_TYPE != data.read_string_u8(3)? {
            return Err(RLibError::DecodingLocNotALocTable)
        }

        let _ = data.read_u8()?;
        let version = data.read_i32()?;
        let entry_count = data.read_u32()?;

        Ok((version, entry_count))
    }
/*

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
    }*/
}


impl Decodeable for Loc {

    fn decode<R: ReadBytes>(data: &mut R, extra_data: &Option<DecodeableExtraData>) -> Result<Self> {
        let extra_data = extra_data.as_ref().ok_or(RLibError::DecodingMissingExtraData)?;
        let file_name = extra_data.file_name.ok_or(RLibError::DecodingMissingExtraDataField("file_name".to_string()))?;
        let pool = extra_data.pool;

        // Version is always 1, so we ignore it.
        let (_version, entry_count) = Self::read_header(data)?;

        let definition = Self::new_definition();
        let table = Table::decode(&pool, data, &definition, Some(entry_count), false, file_name)?;

        // If we are not in the last byte, it means we didn't parse the entire file, which means this file is corrupt.
        check_size_mismatch(data.stream_position()? as usize, data.len()? as usize)?;

        Ok(Self {
            table,
        })
    }
}

impl Encodeable for Loc {

    fn encode<W: WriteBytes>(&mut self, buffer: &mut W, extra_data: &Option<EncodeableExtraData>) -> Result<()> {
        let pool = if let Some (extra_data) = extra_data { extra_data.pool } else { None };

        buffer.write_u16(BYTEORDER_MARK)?;
        buffer.write_string_u8(FILE_TYPE)?;
        buffer.write_u8(0)?;
        buffer.write_i32(*self.table.definition().version())?;
        buffer.write_u32(self.table.len(pool)? as u32)?;

        self.table.encode(buffer, &None, &None, &pool)
    }
}

/// Implementation to create a `Loc` from a `Table` directly.
impl From<Table> for Loc {
    fn from(table: Table) -> Self {
        Self {
            table,
        }
    }
}
