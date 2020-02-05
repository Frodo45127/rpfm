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
Module with all the code to interact with DB Tables.

DB Tables are the files which controls a lot of the parameters used ingame, like units data,
effects data, projectile parameters.... It's what modders use the most.
!*/

use bincode::deserialize;
use serde_derive::{Serialize, Deserialize};
use uuid::Uuid;

use std::collections::BTreeMap;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::PathBuf;

use rpfm_error::{ErrorKind, Result};

use crate::assembly_kit::table_data::RawTable;
use crate::common::{decoder::Decoder, encoder::Encoder};
use crate::common::get_game_selected_pak_file;
use crate::GAME_SELECTED;
use crate::games::*;
use crate::packedfile::DecodedPackedFile;
use crate::packfile::PackFile;
use crate::packfile::packedfile::PackedFile;
use crate::schema::*;
use super::DecodedData;
use super::Table;

/// If this sequence is found, the DB Table has a GUID after it.
const GUID_MARKER: &[u8] = &[253, 254, 252, 255];

/// If this sequence is found, the DB Table has a version number after it.
const VERSION_MARKER: &[u8] = &[252, 253, 254, 255];

//---------------------------------------------------------------------------//
//                              Enum & Structs
//---------------------------------------------------------------------------//

/// This holds an entire DB Table decoded in memory.
#[derive(PartialEq, Clone, Debug, Serialize, Deserialize)]
pub struct DB {

    /// The name of the table. Not his literal file name, but the name of the table it represents, usually, db/"this_name"/yourtable. Needed to get his `Definition`.
    pub name: String,

    /// Don't know his use, but it's in all the tables I've seen, always being `1` or `0`.
    pub mysterious_byte: bool,

    /// The table's data, containing all the stuff needed to decode/encode it.
    table: Table,
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
            table: Table::new(&definition),
        }
    }

    /// This function returns a copy of the name of this DB Table.
    pub fn get_table_name(&self) -> String {
        self.name.to_owned()
    }

    /// This function returns a reference of the name of this DB Table.
    pub fn get_ref_table_name(&self) -> &str {
        &self.name
    }

    /// This function returns a copy of the definition of this DB Table.
    pub fn get_definition(&self) -> Definition {
        self.table.get_definition()
    }

    /// This function returns a reference to the definition of this DB Table.
    pub fn get_ref_definition(&self) -> &Definition {
        self.table.get_ref_definition()
    }

    /// This function returns a copy of the entries of this DB Table.
    pub fn get_table_data(&self) -> Vec<Vec<DecodedData>> {
        self.table.get_table_data()
    }

    /// This function returns a reference to the entries of this DB Table.
    pub fn get_ref_table_data(&self) -> &[Vec<DecodedData>] {
        self.table.get_ref_table_data()
    }

    /// This function returns the amount of entries in this DB Table.
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
        { return Err(ErrorKind::DBTableContainsListField.into()) }

        // Try to get the table_definition for this table, if exists.
        let versioned_file = schema.get_ref_versioned_file_db(&name);
        if versioned_file.is_err() && entry_count == 0 { return Err(ErrorKind::TableEmptyWithNoDefinition.into()) }
        let definition = versioned_file?.get_version(version);
        if definition.is_err() && entry_count == 0 { return Err(ErrorKind::TableEmptyWithNoDefinition.into()) }

        // Then try to decode all the entries.
        let mut table = Table::new(definition?);
        table.decode(&packed_file_data, entry_count, &mut index)?;

        // If we are not in the last byte, it means we didn't parse the entire file, which means this file is corrupt.
        if index != packed_file_data.len() { return Err(ErrorKind::PackedFileSizeIsNotWhatWeExpect(packed_file_data.len(), index).into()) }

        // If we've reached this, we've succesfully decoded the table.
        Ok(Self {
            name: name.to_owned(),
            mysterious_byte,
            table,
        })
    }

    /// This function takes a `DB` and encodes it to `Vec<u8>`.
    pub fn save(&self) -> Result<Vec<u8>> {
        let mut packed_file: Vec<u8> = vec![];

        // Napoleon and Empire do not have GUID, and adding it to their tables crash both games.
        // So for those two games, we ignore the GUID_MARKER and the GUID itself.
        let game_selected = GAME_SELECTED.read().unwrap().to_owned();
        if game_selected != KEY_EMPIRE && game_selected != KEY_NAPOLEON {
            packed_file.extend_from_slice(GUID_MARKER);
            packed_file.encode_packedfile_string_u16(&format!("{}", Uuid::new_v4()));
        }
        packed_file.extend_from_slice(VERSION_MARKER);
        packed_file.encode_integer_i32(self.table.definition.version);
        packed_file.encode_bool(self.mysterious_byte);
        packed_file.encode_integer_u32(self.table.entries.len() as u32);

        self.table.encode(&mut packed_file)?;

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
    pub fn get_header(packed_file_data: &[u8]) -> Result<(i32, bool, u32, usize)> {

        // 5 is the minimum amount of bytes a valid DB Table can have. If there is less, either the table is broken,
        // or the data is not from a DB Table.
        if packed_file_data.len() < 5 { return Err(ErrorKind::DBTableIsNotADBTable.into()) }

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
        if let Ok(pak_file) = get_game_selected_pak_file() {
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



    /// This function is used to optimize the size of a DB Table.
    ///
    /// It scans every line to check if it's a vanilla line, and remove it in that case. Also, if the entire
    /// file is composed of only vanilla lines, it marks the entire PackedFile for removal.
    pub fn optimize_table(&mut self, vanilla_tables: &[&Self]) -> bool {

        // For each vanilla table, if it's the same table/version as our own, we check
        let mut new_entries = Vec::with_capacity(self.table.get_entry_count());
        let entries = self.get_ref_table_data();
        let definition = self.get_ref_definition();
        for entry in entries {
            for vanilla_entries in vanilla_tables.iter().filter(|x| x.name == self.name && x.get_ref_definition().version == definition.version).map(|x| x.get_ref_table_data()) {
                if vanilla_entries.contains(entry) {
                    new_entries.push(entry.to_vec());
                    continue;
                }
            }
        }

        // Then we overwrite the entries and return if the table is empty or now, so we can optimize it further at `PackedFile` level.
        let _ = self.table.set_table_data(&new_entries);
        self.table.get_ref_table_data().is_empty()
    }

    /// This function returns the dependency/lookup data of a column from the dependency database.
    fn get_dependency_data_from_real_dependencies(
        references: &mut Vec<(String, String)>,
        reference_info: (&str, &str, &[String]),
        real_dep_db: &mut Vec<PackedFile>,
        schema: &Schema,
    ) {

        // Scan the dependency data for references. The process is simple: keep finding referenced tables,
        // Then open them and get the column we need. Here, we do it on the real dependencies (vanilla + mod).
        let ref_table = reference_info.0;
        let ref_column = reference_info.1;
        let ref_lookup_columns = reference_info.2;
        let mut iter = real_dep_db.iter_mut();
        while let Some(packed_file) = iter.find(|x| x.get_path().starts_with(&["db".to_owned(), format!("{}_tables", ref_table)])) {
            if let Ok(table) = packed_file.decode_return_ref_no_locks(schema) {
                if let DecodedPackedFile::DB(db) = table {
                    for row in &db.get_table_data() {
                        let mut reference_data = String::new();
                        let mut lookup_data = vec![];

                        // First, we get the reference data.
                        if let Some(index) = db.get_definition().fields.iter().position(|x| x.name == ref_column) {
                            match row[index] {
                                DecodedData::Boolean(ref entry) => reference_data = format!("{}", entry),
                                DecodedData::Float(ref entry) => reference_data = format!("{}", entry),
                                DecodedData::Integer(ref entry) => reference_data = format!("{}", entry),
                                DecodedData::LongInteger(ref entry) => reference_data = format!("{}", entry),
                                DecodedData::StringU8(ref entry) |
                                DecodedData::StringU16(ref entry) |
                                DecodedData::OptionalStringU8(ref entry) |
                                DecodedData::OptionalStringU16(ref entry) => reference_data = entry.to_owned(),
                                _ => {}
                            }
                        }

                        // Then, we get the lookup data.
                        for column in ref_lookup_columns {
                            if let Some(index) = db.get_definition().fields.iter().position(|x| &x.name == column) {
                                match row[index] {
                                    DecodedData::Boolean(ref entry) => lookup_data.push(format!("{}", entry)),
                                    DecodedData::Float(ref entry) => lookup_data.push(format!("{}", entry)),
                                    DecodedData::Integer(ref entry) => lookup_data.push(format!("{}", entry)),
                                    DecodedData::LongInteger(ref entry) => lookup_data.push(format!("{}", entry)),
                                    DecodedData::StringU8(ref entry) |
                                    DecodedData::StringU16(ref entry) |
                                    DecodedData::OptionalStringU8(ref entry) |
                                    DecodedData::OptionalStringU16(ref entry) => lookup_data.push(entry.to_owned()),
                                    _ => {}
                                }
                            }
                        }

                        references.push((reference_data, lookup_data.join(" ")));
                    }
                }
            }
        }
    }

    /// This function returns the dependency/lookup data of a column from the fake dependency database.
    fn get_dependency_data_from_fake_dependencies(
        references: &mut Vec<(String, String)>,
        reference_info: (&str, &str, &[String]),
        fake_dep_db: &[DB],
    ) {

        // Scan the dependency data for references. The process is simple: keep finding referenced tables,
        // Then open them and get the column we need. Here, we do it on the real dependencies (vanilla + mod).
        let ref_table = reference_info.0;
        let ref_column = reference_info.1;
        let ref_lookup_columns = reference_info.2;
        let mut iter = fake_dep_db.iter();
        if let Some(table) = iter.find(|x| x.name == format!("{}_tables", ref_table)) {
            for row in &table.get_table_data() {
                let mut reference_data = String::new();
                let mut lookup_data = vec![];

                // First, we get the reference data.
                if let Some(index) = table.get_definition().fields.iter().position(|x| x.name == ref_column) {
                    match row[index] {
                        DecodedData::Boolean(ref entry) => reference_data = format!("{}", entry),
                        DecodedData::Float(ref entry) => reference_data = format!("{}", entry),
                        DecodedData::Integer(ref entry) => reference_data = format!("{}", entry),
                        DecodedData::LongInteger(ref entry) => reference_data = format!("{}", entry),
                        DecodedData::StringU8(ref entry) |
                        DecodedData::StringU16(ref entry) |
                        DecodedData::OptionalStringU8(ref entry) |
                        DecodedData::OptionalStringU16(ref entry) => reference_data = entry.to_owned(),
                        _ => {}
                    }
                }

                // Then, we get the lookup data.
                for column in ref_lookup_columns {
                    if let Some(index) = table.get_definition().fields.iter().position(|x| &x.name == column) {
                        match row[index] {
                            DecodedData::Boolean(ref entry) => lookup_data.push(format!("{}", entry)),
                            DecodedData::Float(ref entry) => lookup_data.push(format!("{}", entry)),
                            DecodedData::Integer(ref entry) => lookup_data.push(format!("{}", entry)),
                            DecodedData::LongInteger(ref entry) => lookup_data.push(format!("{}", entry)),
                            DecodedData::StringU8(ref entry) |
                            DecodedData::StringU16(ref entry) |
                            DecodedData::OptionalStringU8(ref entry) |
                            DecodedData::OptionalStringU16(ref entry) => lookup_data.push(entry.to_owned()),
                            _ => {}
                        }
                    }
                }

                references.push((reference_data, lookup_data.join(" ")));
            }
        }
    }

    /// This function returns the dependency/lookup data of a column from our own `PackFile`.
    fn get_dependency_data_from_packfile(
        references: &mut Vec<(String, String)>,
        reference_info: (&str, &str, &[String]),
        packfile: &mut PackFile,
        schema: &Schema,
    ) {

        // Scan our own packedfiles data for references. The process is simple: keep finding referenced tables,
        // Then open them and get the column we need. Here, we do it on the real dependencies (vanilla + mod).
        let ref_table = reference_info.0;
        let ref_column = reference_info.1;
        let ref_lookup_columns = reference_info.2;
        for packed_file in packfile.get_ref_mut_packed_files_by_path_start(&["db".to_owned(), format!("{}_tables", ref_table)]) {
            if let Ok(table) = packed_file.decode_return_ref_no_locks(schema) {
                if let DecodedPackedFile::DB(db) = table {
                    for row in &db.get_table_data() {
                        let mut reference_data = String::new();
                        let mut lookup_data = vec![];

                        // First, we get the reference data.
                        if let Some(index) = db.get_definition().fields.iter().position(|x| x.name == ref_column) {
                            match row[index] {
                                DecodedData::Boolean(ref entry) => reference_data = format!("{}", entry),
                                DecodedData::Float(ref entry) => reference_data = format!("{}", entry),
                                DecodedData::Integer(ref entry) => reference_data = format!("{}", entry),
                                DecodedData::LongInteger(ref entry) => reference_data = format!("{}", entry),
                                DecodedData::StringU8(ref entry) |
                                DecodedData::StringU16(ref entry) |
                                DecodedData::OptionalStringU8(ref entry) |
                                DecodedData::OptionalStringU16(ref entry) => reference_data = entry.to_owned(),
                                _ => {}
                            }
                        }

                        // Then, we get the lookup data.
                        for column in ref_lookup_columns {
                            if let Some(index) = db.get_definition().fields.iter().position(|x| &x.name == column) {
                                match row[index] {
                                    DecodedData::Boolean(ref entry) => lookup_data.push(format!("{}", entry)),
                                    DecodedData::Float(ref entry) => lookup_data.push(format!("{}", entry)),
                                    DecodedData::Integer(ref entry) => lookup_data.push(format!("{}", entry)),
                                    DecodedData::LongInteger(ref entry) => lookup_data.push(format!("{}", entry)),
                                    DecodedData::StringU8(ref entry) |
                                    DecodedData::StringU16(ref entry) |
                                    DecodedData::OptionalStringU8(ref entry) |
                                    DecodedData::OptionalStringU16(ref entry) => lookup_data.push(entry.to_owned()),
                                    _ => {}
                                }
                            }
                        }

                        references.push((reference_data, lookup_data.join(" ")));
                    }
                }
            }
        }
    }

    /// This function returns the dependency/lookup data of each column of a DB Table.
    ///
    /// The returned references are in the following format:
    /// ```BTreeMap<column_index, Vec<(referenced_value, lookup_value)>```.
    pub fn get_dependency_data(
        pack_file: &mut PackFile,
        schema: &Schema,
        table_definition: &Definition,
        real_dep_db: &mut Vec<PackedFile>,
        fake_dep_db: &[DB],
    ) -> BTreeMap<i32, Vec<(String, String)>> {
        let mut data = BTreeMap::new();
        for (column, field) in table_definition.fields.iter().enumerate() {
            if let Some((ref ref_table, ref ref_column)) = field.is_reference {
                if !ref_table.is_empty() && !ref_column.is_empty() {

                    // Get his lookup data if it has it.
                    let lookup_data = if let Some(ref data) = field.lookup { data.to_vec() } else { Vec::with_capacity(0) };
                    let mut references = vec![];

                    Self::get_dependency_data_from_real_dependencies(&mut references, (&ref_table, &ref_column, &lookup_data), real_dep_db, schema);
                    Self::get_dependency_data_from_fake_dependencies(&mut references, (&ref_table, &ref_column, &lookup_data), fake_dep_db);
                    Self::get_dependency_data_from_packfile(&mut references, (&ref_table, &ref_column, &lookup_data), pack_file, schema);

                    // Sort and dedup the data found.
                    references.sort_unstable_by(|a, b| a.0.cmp(&b.0));
                    references.dedup();

                    data.insert(column as i32, references);
                }
            }
        }

        data
    }

    /// This function imports a TSV file into a decoded table.
    pub fn import_tsv(
        definition: &Definition,
        path: &PathBuf,
        name: &str,
        version: i32,
    ) -> Result<Self> {
        let table = Table::import_tsv(definition, path, name, version)?;
        let mut db = DB::from(table);
        db.name = name.to_owned();
        Ok(db)
    }

    /// This function exports the provided data to a TSV file.
    pub fn export_tsv(
        &self,
        path: &PathBuf,
        table_name: &str,
        table_version: i32
    ) -> Result<()> {
        self.table.export_tsv(path, table_name, table_version)
    }

    /// This function imports a TSV file into a binary file on disk.
    pub fn import_tsv_to_binary_file(
        schema: &Schema,
        source_paths: &[PathBuf],
    ) -> Result<()> {
        let destination = PathBuf::from(".");
        for path in source_paths {
            Table::import_tsv_to_binary_file(&schema, &path, &destination)?;
        }

        Ok(())
    }

    /// This function exports to TSV a binary file on disk.
    pub fn export_tsv_from_binary_file(
        schema: &Schema,
        source_paths: &[PathBuf],
    ) -> Result<()> {
        let destination = PathBuf::from(".");
        for path in source_paths {
            Table::export_tsv_from_binary_file(&schema, &path, &destination)?;
        }

        Ok(())
    }
}

/// Implementation to create a `DB` from a `Table`.
impl From<Table> for DB {
    fn from(table: Table) -> Self {
        Self {
            name: String::new(),
            mysterious_byte: false,
            table,
        }
    }
}

/// Implementation to create a `DB` from a `RawTable`.
impl From<&RawTable> for DB {
    fn from(raw_table: &RawTable) -> Self {
        let name_table = if let Some(ref x) = raw_table.definition {
            if let Some(ref y) = x.name {
                format!("{}_tables", y)
            }
            else { String::new() }
        } else { String::new() };

        Self {
            name: name_table,
            mysterious_byte: false,
            table: From::from(raw_table),
        }
    }
}
