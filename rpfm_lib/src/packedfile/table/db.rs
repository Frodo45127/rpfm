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
Module with all the code to interact with DB Tables.

DB Tables are the files which controls a lot of the parameters used in game, like units data,
effects data, projectile parameters.... It's what modders use the most.
!*/

use bincode::deserialize;
use rayon::prelude::*;
use serde_derive::{Serialize, Deserialize};
use uuid::Uuid;

use std::cmp::Ordering;
use std::collections::{HashSet, BTreeMap};
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::{Path, PathBuf};

use rpfm_error::{ErrorKind, Result};
use rpfm_macros::*;

use crate::assembly_kit::table_data::RawTable;
use crate::common::{decoder::Decoder, encoder::Encoder};
use crate::GAME_SELECTED;
use crate::packedfile::DecodedPackedFile;
use crate::packedfile::Dependencies;
use crate::packedfile::PackedFileType;
use crate::packfile::packedfile::PackedFile;
use crate::packfile::PackFile;
use crate::schema::*;
use crate::SETTINGS;
use crate::SCHEMA;
use super::{DecodedData, Table, DependencyData};

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
    /// NOTE: In Warhammer 2, a 0 here seems to crash the game when the tables are loaded.
    pub mysterious_byte: bool,

    /// UUID of this table.
    pub uuid: String,

    /// The table's data, containing all the stuff needed to decode/encode it.
    table: Table,
}

/// This holds all the data needed to trigger cascade editions.
///
/// We use a struct because Cascade Editions need a lot of different data, and it's a mess to deal with all of it independently.
#[derive(Clone, Debug, Default, GetRef, GetRefMut, Set)]
pub struct CascadeEdition {

    /// Name of the edited table.
    edited_table_name: String,

    /// Definition of the edited table.
    edited_table_definition: Definition,

    /// Which columns of which tables point to the column used as key.
    referenced_tables: BTreeMap<u32, (BTreeMap<String, Vec<String>>, bool)>,

    /// Change we have to do, with the column as key.
    data_changes: BTreeMap<u32, Vec<(String, String)>>,
}

//---------------------------------------------------------------------------//
//                           Implementation of DB
//---------------------------------------------------------------------------//

/// Implementation of `DB`.
impl DB {

    /// This function creates a new empty `DB` from a definition and his name.
    pub fn new(
        name: &str,
        uuid: Option<&str>,
        definition: &Definition
    ) -> Self {
        Self{
            name: name.to_owned(),
            mysterious_byte: true,
            uuid: if let Some(uuid) = uuid { uuid.to_owned() } else { Uuid::new_v4().to_string() },
            table: Table::new(definition),
        }
    }

    /// This function returns a copy of the name of this DB Table.
    pub fn get_table_name(&self) -> String {
        self.name.to_owned()
    }

    /// This function returns a copy of the name of this DB Table, without the "_tables" suffix.
    pub fn get_table_name_without_tables(&self) -> String {
        self.name.to_owned().drain(..self.name.len() - 7).collect()
    }

    /// This function returns a reference of the name of this DB Table.
    pub fn get_ref_table_name(&self) -> &str {
        &self.name
    }

    /// This function returns a copy of the UUID of this DB Table.
    pub fn get_uuid(&self) -> String {
        self.uuid.to_owned()
    }

    /// This function returns a copy of the definition of this DB Table.
    pub fn get_definition(&self) -> Definition {
        self.table.get_definition()
    }

    /// This function returns a reference to the definition of this DB Table.
    pub fn get_ref_definition(&self) -> &Definition {
        self.table.get_ref_definition()
    }

    /// This function returns a reference to the underlying table.
    pub fn get_ref_table(&self) -> &Table {
        &self.table
    }

    /// This function returns a copy of the entries of this DB Table.
    pub fn get_table_data(&self) -> Vec<Vec<DecodedData>> {
        self.table.get_table_data()
    }

    /// This function returns a reference to the entries of this DB Table.
    pub fn get_ref_table_data(&self) -> &[Vec<DecodedData>] {
        self.table.get_ref_table_data()
    }

    /// This function returns the position of a column in a definition, or an error if the column is not found.
    pub fn get_column_position_by_name(&self, column_name: &str) -> Result<usize> {
        self.table.get_column_position_by_name(column_name)
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

    /// This function returns a valid empty row for this table.
    pub fn get_new_row(&self) -> Vec<DecodedData> {
        Table::get_new_row(self.get_ref_definition(), Some(&self.get_table_name()))
    }

    /// This function creates a `DB` from a `Vec<u8>`.
    pub fn read(
        packed_file_data: &[u8],
        name: &str,
        schema: &Schema,
        return_incomplete: bool
    ) -> Result<Self> {

        // Get the header of the `DB`.
        let (version, mysterious_byte, uuid, entry_count, mut index) = Self::read_header(packed_file_data)?;

        // Try to get the table_definition for this table, if exists.
        let versioned_file = schema.get_ref_versioned_file_db(name);
        if versioned_file.is_err() && entry_count == 0 { return Err(ErrorKind::TableEmptyWithNoDefinition.into()) }

        // For version 0 tables, get all definitions between 0 and -99, and get the first one that works.
        let index_reset = index;
        let table = if version == 0 {
            let definitions = versioned_file?.get_version_alternatives();
            let table: Option<Result<Table>> = definitions.iter().find_map(|definition| {
                let mut table = Table::new(definition);
                index = index_reset;
                let decoded_table = table.decode(packed_file_data, entry_count, &mut index, return_incomplete);
                if decoded_table.is_ok() {
                    Some(Ok(table))
                } else if return_incomplete {
                    if let Err(error) = decoded_table {
                        if let ErrorKind::TableIncompleteError(_, _) = error.kind() {
                            Some(Err(error))
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                } else {
                    None
                }
            });

            match table {
                Some(table) => table?,
                None => return Err(ErrorKind::SchemaDefinitionNotFound.into()),
            }
        }

        // For +0 versions, we expect unique definitions.
        else {
            let definition = versioned_file?.get_version(version);
            if definition.is_err() && entry_count == 0 { return Err(ErrorKind::TableEmptyWithNoDefinition.into()) }

            // Then try to decode all the entries.
            let mut table = Table::new(definition?);
            table.decode(packed_file_data, entry_count, &mut index, return_incomplete)?;
            table
        };

        // If we are not in the last byte, it means we didn't parse the entire file, which means this file is corrupt.
        if index != packed_file_data.len() { return Err(ErrorKind::PackedFileSizeIsNotWhatWeExpect(packed_file_data.len(), index).into()) }

        // If we've reached this, we've successfully decoded the table.
        Ok(Self {
            name: name.to_owned(),
            mysterious_byte,
            uuid,
            table,
        })
    }

    /// This function creates a `DB` from a `Vec<u8>` using only a field list instead of a full definition.
    pub fn read_with_fields(
        packed_file_data: &[u8],
        name: &str,
        fields: &[Field],
        return_incomplete: bool
    ) -> Result<Self> {

        // Get the header of the `DB`.
        let (version, mysterious_byte, uuid, entry_count, mut index) = Self::read_header(packed_file_data)?;

        // Then try to decode all the entries.
        let mut definition = Definition::new(version);
        *definition.get_ref_mut_fields() = fields.to_vec();

        let mut table = Table::new(&definition);
        table.decode(packed_file_data, entry_count, &mut index, return_incomplete)?;

        // If we are not in the last byte, it means we didn't parse the entire file, which means this file is corrupt.
        if index != packed_file_data.len() { return Err(ErrorKind::PackedFileSizeIsNotWhatWeExpect(packed_file_data.len(), index).into()) }

        // If we've reached this, we've successfully decoded the table.
        Ok(Self {
            name: name.to_owned(),
            mysterious_byte,
            uuid,
            table,
        })
    }

    /// This function takes a `DB` and encodes it to `Vec<u8>`.
    pub fn save(&self) -> Result<Vec<u8>> {
        let mut packed_file: Vec<u8> = vec![];

        // Napoleon and Empire do not have GUID, and adding it to their tables crash both games.
        // So for those two games, we ignore the GUID_MARKER and the GUID itself.
        if GAME_SELECTED.read().unwrap().get_db_tables_have_guid() {
            packed_file.extend_from_slice(GUID_MARKER);
            if SETTINGS.read().unwrap().settings_bool["disable_uuid_regeneration_on_db_tables"] && !self.uuid.is_empty() {
                packed_file.encode_packedfile_string_u16(&self.uuid);
            }
            else {
                packed_file.encode_packedfile_string_u16(&format!("{}", Uuid::new_v4()));
            }
        }

        // Only put version numbers on tables with an actual version.
        if self.table.definition.get_version() > 0 {
            packed_file.extend_from_slice(VERSION_MARKER);
            packed_file.encode_integer_i32(self.table.definition.get_version());
        }

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
    /// - `uuid`: the UUID of this table.
    /// - `entry_count`: amount of entries this `DB` has.
    /// - `index`: position where the header ends. Useful if you want to decode the data of the `DB` after this.
    pub fn read_header(packed_file_data: &[u8]) -> Result<(i32, bool, String, u32, usize)> {

        // 5 is the minimum amount of bytes a valid DB Table can have. If there is less, either the table is broken,
        // or the data is not from a DB Table.
        if packed_file_data.len() < 5 { return Err(ErrorKind::DBTableIsNotADBTable.into()) }

        // Create the index that we'll use to decode the entire table.
        let mut index = 0;

        // If there is a GUID_MARKER, skip it together with the GUID itself (4 bytes for the marker, 74 for the GUID).
        // About this GUID, it's something that gets randomly generated every time you export a table with DAVE. Not useful.
        let uuid = if packed_file_data.get_bytes_checked(0, 4)? == GUID_MARKER {
            index += 4;
            packed_file_data.decode_packedfile_string_u16(index, &mut index)?
        }
        else { String::new() };

        // If there is a VERSION_MARKER, we get the version (4 bytes for the marker, 4 for the version). Otherwise, we default to 0.
        let version = if packed_file_data.get_bytes_checked(index, 4)? == VERSION_MARKER {
            index += 4;
            packed_file_data.decode_packedfile_integer_i32(index, &mut index)?
        } else { 0 };

        // We get the rest of the data from the header.
        let mysterious_byte = packed_file_data.decode_packedfile_bool(index, &mut index)?;
        let entry_count = packed_file_data.decode_packedfile_integer_u32(index, &mut index)?;
        Ok((version, mysterious_byte, uuid, entry_count, index))
    }

    /// This function loads the PAK file of the game selected (if exists) into memory.
    ///
    /// This is useful to help resolving dependencies.
    pub fn read_pak_file() -> Vec<Self> {

        // Create the empty list.
        let mut db_files = vec![];

        // Get all the paths we need.
        if let Ok(pak_file) = GAME_SELECTED.read().unwrap().get_dependencies_cache_file() {
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

        // For each vanilla table, if it's the same table/version as our own, we check it
        let mut entries = self.get_ref_table_data().to_vec();
        let definition = self.get_ref_definition();
        let first_key = definition.get_fields_sorted().iter().position(|x| x.get_is_key()).unwrap_or(0);

        // To do it faster, make a freaking big table with all the vanilla entries together.
        let vanilla_table = vanilla_tables.iter()
            .filter(|x| x.name == self.name && x.get_ref_definition().get_version() == definition.get_version())
            .map(|x| x.get_ref_table_data().to_vec())
            .flatten()
            .map(|x| {

                // We map all floats here to string representations of floats, so we can actually compare them reliably.
                let json = x.iter().map(|data|
                    if let DecodedData::F32(value) = data {
                        DecodedData::StringU8(format!("{:.4}", value))
                    } else {
                        data.to_owned()
                    }
                ).collect::<Vec<DecodedData>>();
                serde_json::to_string(&json).unwrap()
            })
            .collect::<HashSet<String>>();

        // Remove ITM and ITNR entries, sort the remaining ones by keys, and dedup them.
        let new_row = self.get_new_row().iter().map(|data|
            if let DecodedData::F32(value) = data {
                DecodedData::StringU8(format!("{:.4}", value))
            } else {
                data.to_owned()
            }
        ).collect::<Vec<DecodedData>>();

        entries.retain(|entry| {
            let entry_json = entry.iter().map(|data|
                if let DecodedData::F32(value) = data {
                    DecodedData::StringU8(format!("{:.4}", value))
                } else {
                    data.to_owned()
                }
            ).collect::<Vec<DecodedData>>();
            !vanilla_table.contains(&serde_json::to_string(&entry_json).unwrap()) && entry != &new_row
        });

        // Sort the table so it can be dedupd. Sorting floats is a pain in the ass.
        entries.par_sort_by(|a, b| {
            let ordering = if let DecodedData::F32(x) = a[first_key] {
                if let DecodedData::F32(y) = b[first_key] {
                    if float_eq::float_eq!(x, y, abs <= 0.0001) {
                        Some(Ordering::Equal)
                    } else { None }
                } else { None }
            } else { None };

            match ordering {
                Some(ordering) => ordering,
                None => a[first_key].partial_cmp(&b[first_key]).unwrap_or(Ordering::Equal)
            }
        });

        entries.dedup();

        // Then we overwrite the entries and return if the table is empty or now, so we can optimize it further at `PackedFile` level.
        let _ = self.table.set_table_data(&entries);
        self.table.get_ref_table_data().is_empty()
    }

    /// This function returns the dependency/lookup data of a column from the dependency database.
    ///
    /// Returns true if anything was found. Otherwise returns false.
    fn get_dependency_data_from_vanilla_and_modded_tables(
        references: &mut DependencyData,
        reference_info: (&str, &str, &[String]),
        real_dep_db: &[PackedFile],
    ) -> bool {

        // Scan the dependency data for references. The process is simple: keep finding referenced tables,
        // Then open them and get the column we need. Here, we do it on the real dependencies (vanilla + mod).
        let mut data_found = false;
        let ref_table_tables = format!("{}_tables", reference_info.0);
        let ref_column = reference_info.1;
        let ref_lookup_columns = reference_info.2;

        real_dep_db.iter()
            .filter(|x| x.get_path()[1] == ref_table_tables)
            .filter_map(|packed_file| if let Ok(DecodedPackedFile::DB(db)) = packed_file.get_decoded_from_memory() { Some(db) } else { None })
            .for_each(|db| {
            for row in db.get_ref_table_data() {
                let mut reference_data = String::new();
                let mut lookup_data = vec![];

                // First, we get the reference data.
                if let Some(index) = db.get_definition().get_fields_processed().iter().position(|x| x.get_name() == ref_column) {
                    match row[index] {
                        DecodedData::Boolean(ref entry) => reference_data = format!("{}", entry),
                        DecodedData::F32(ref entry) => reference_data = format!("{}", entry),
                        DecodedData::I16(ref entry) => reference_data = format!("{}", entry),
                        DecodedData::I32(ref entry) => reference_data = format!("{}", entry),
                        DecodedData::I64(ref entry) => reference_data = format!("{}", entry),
                        DecodedData::StringU8(ref entry) |
                        DecodedData::StringU16(ref entry) |
                        DecodedData::OptionalStringU8(ref entry) |
                        DecodedData::OptionalStringU16(ref entry) => reference_data = entry.to_owned(),
                        _ => {}
                    }
                }

                // Then, we get the lookup data.
                for column in ref_lookup_columns {
                    if let Some(index) = db.get_definition().get_fields_processed().iter().position(|x| x.get_name() == column) {
                        match row[index] {
                            DecodedData::Boolean(ref entry) => lookup_data.push(format!("{}", entry)),
                            DecodedData::F32(ref entry) => lookup_data.push(format!("{}", entry)),
                            DecodedData::I16(ref entry) => lookup_data.push(format!("{}", entry)),
                            DecodedData::I32(ref entry) => lookup_data.push(format!("{}", entry)),
                            DecodedData::I64(ref entry) => lookup_data.push(format!("{}", entry)),
                            DecodedData::StringU8(ref entry) |
                            DecodedData::StringU16(ref entry) |
                            DecodedData::OptionalStringU8(ref entry) |
                            DecodedData::OptionalStringU16(ref entry) => lookup_data.push(entry.to_owned()),
                            _ => {}
                        }
                    }
                }
                references.data.insert(reference_data, lookup_data.join(" "));

                if !data_found {
                    data_found = true;
                }
            }
        });
        data_found
    }

    /// This function returns the dependency/lookup data of a column from the fake dependency database.
    fn get_dependency_data_from_asskit_only_tables(
        references: &mut DependencyData,
        reference_info: (&str, &str, &[String]),
        fake_dep_db: &[DB],
    ) -> bool {
        let mut data_found = false;
        let ref_table = reference_info.0;
        let ref_column = reference_info.1;
        let ref_lookup_columns = reference_info.2;
        fake_dep_db.iter().filter(|x| x.name == format!("{}_tables", ref_table)).for_each(|table| {
            for row in &table.get_table_data() {
                let mut reference_data = String::new();
                let mut lookup_data = vec![];

                // First, we get the reference data.
                if let Some(index) = table.get_definition().get_fields_processed().iter().position(|x| x.get_name() == ref_column) {
                    match row[index] {
                        DecodedData::Boolean(ref entry) => reference_data = format!("{}", entry),
                        DecodedData::F32(ref entry) => reference_data = format!("{}", entry),
                        DecodedData::I16(ref entry) => reference_data = format!("{}", entry),
                        DecodedData::I32(ref entry) => reference_data = format!("{}", entry),
                        DecodedData::I64(ref entry) => reference_data = format!("{}", entry),
                        DecodedData::StringU8(ref entry) |
                        DecodedData::StringU16(ref entry) |
                        DecodedData::OptionalStringU8(ref entry) |
                        DecodedData::OptionalStringU16(ref entry) => reference_data = entry.to_owned(),
                        _ => {}
                    }
                }

                // Then, we get the lookup data.
                for column in ref_lookup_columns {
                    if let Some(index) = table.get_definition().get_fields_processed().iter().position(|x| x.get_name() == column) {
                        match row[index] {
                            DecodedData::Boolean(ref entry) => lookup_data.push(format!("{}", entry)),
                            DecodedData::F32(ref entry) => lookup_data.push(format!("{}", entry)),
                            DecodedData::I16(ref entry) => lookup_data.push(format!("{}", entry)),
                            DecodedData::I32(ref entry) => lookup_data.push(format!("{}", entry)),
                            DecodedData::I64(ref entry) => lookup_data.push(format!("{}", entry)),
                            DecodedData::StringU8(ref entry) |
                            DecodedData::StringU16(ref entry) |
                            DecodedData::OptionalStringU8(ref entry) |
                            DecodedData::OptionalStringU16(ref entry) => lookup_data.push(entry.to_owned()),
                            _ => {}
                        }
                    }
                }

                references.data.insert(reference_data, lookup_data.join(" "));

                if !data_found {
                    data_found = true;
                }
            }
        });
        data_found
    }

    /// This function returns the dependency/lookup data of a column from our own `PackFile`.
    fn get_dependency_data_from_packfile(
        references: &mut DependencyData,
        reference_info: (&str, &str, &[String]),
        packfile: &PackFile,
        files_to_ignore: &[Vec<String>]
    ) -> bool {

        // Scan our own packedfiles data for references. The process is simple: keep finding referenced tables,
        // Then open them and get the column we need. Here, we do it on the real dependencies (vanilla + mod).
        let mut data_found = false;
        let ref_table = reference_info.0;
        let ref_column = reference_info.1;
        let ref_lookup_columns = reference_info.2;
        packfile.get_ref_packed_files_by_path_start(&["db".to_owned(), format!("{}_tables", ref_table)]).iter()
            .filter(|x| !files_to_ignore.contains(&x.get_path().to_vec()))
            .for_each(|packed_file| {
            if let Ok(DecodedPackedFile::DB(db)) = packed_file.get_decoded_from_memory() {
                for row in db.get_ref_table_data() {
                    let mut reference_data = String::new();
                    let mut lookup_data = vec![];

                    // First, we get the reference data.
                    if let Some(index) = db.get_definition().get_fields_processed().iter().position(|x| x.get_name() == ref_column) {
                        match row[index] {
                            DecodedData::Boolean(ref entry) => reference_data = format!("{}", entry),
                            DecodedData::F32(ref entry) => reference_data = format!("{}", entry),
                            DecodedData::I16(ref entry) => reference_data = format!("{}", entry),
                            DecodedData::I32(ref entry) => reference_data = format!("{}", entry),
                            DecodedData::I64(ref entry) => reference_data = format!("{}", entry),
                            DecodedData::StringU8(ref entry) |
                            DecodedData::StringU16(ref entry) |
                            DecodedData::OptionalStringU8(ref entry) |
                            DecodedData::OptionalStringU16(ref entry) => reference_data = entry.to_owned(),
                            _ => {}
                        }
                    }

                    // Then, we get the lookup data.
                    for column in ref_lookup_columns {
                        if let Some(index) = db.get_definition().get_fields_processed().iter().position(|x| x.get_name() == column) {
                            match row[index] {
                                DecodedData::Boolean(ref entry) => lookup_data.push(format!("{}", entry)),
                                DecodedData::F32(ref entry) => lookup_data.push(format!("{}", entry)),
                                DecodedData::I16(ref entry) => lookup_data.push(format!("{}", entry)),
                                DecodedData::I32(ref entry) => lookup_data.push(format!("{}", entry)),
                                DecodedData::I64(ref entry) => lookup_data.push(format!("{}", entry)),
                                DecodedData::StringU8(ref entry) |
                                DecodedData::StringU16(ref entry) |
                                DecodedData::OptionalStringU8(ref entry) |
                                DecodedData::OptionalStringU16(ref entry) => lookup_data.push(entry.to_owned()),
                                _ => {}
                            }
                        }
                    }

                    references.data.insert(reference_data, lookup_data.join(" "));

                    if !data_found {
                        data_found = true;
                    }
                }
            }
        });
        data_found
    }

    /// This function returns the dependency/lookup data of each column of a DB Table.
    ///
    /// The returned references are in the following format:
    /// ```BTreeMap<column_index, Vec<(referenced_value, lookup_value, only_present_in_ak)>```.
    pub fn get_dependency_data(
        pack_file: &PackFile,
        table_name: &str,
        table_definition: &Definition,
        vanilla_dependencies: &[PackedFile],
        asskit_dependencies: &[DB],
        dependencies: &Dependencies,
        files_to_ignore: &[Vec<String>]
    ) -> BTreeMap<i32, DependencyData> {
        let schema = SCHEMA.read().unwrap();

        // First check if the data is already cached, to speed up things.
        let mut vanilla_references = if !table_name.is_empty() {

            // Wait to have the lock, because this can trigger crashes due to locks.
            let cache = loop {
                if let Ok(ref cache) = dependencies.get_ref_cached_data().read() {
                    break cache.get(table_name).cloned();
                }
            };

            match cache {
                Some(cached_data) => cached_data,
                None => {
                    let cached_data = table_definition.get_fields_processed().into_iter().enumerate().filter_map(|(column, field)| {
                        if let Some((ref ref_table, ref ref_column)) = field.get_is_reference() {
                            if !ref_table.is_empty() && !ref_column.is_empty() {

                                // Get his lookup data if it has it.
                                let lookup_data = if let Some(ref data) = field.get_lookup() { data.to_vec() } else { Vec::with_capacity(0) };
                                let mut references = DependencyData::default();

                                let fake_found = if asskit_dependencies.is_empty() {
                                    Self::get_dependency_data_from_asskit_only_tables(&mut references, (ref_table, ref_column, &lookup_data), dependencies.get_ref_asskit_only_db_tables())
                                } else { Self::get_dependency_data_from_asskit_only_tables(&mut references, (ref_table, ref_column, &lookup_data), asskit_dependencies) };

                                let real_found = if vanilla_dependencies.is_empty() {
                                    if let Ok(dependencies) = dependencies.get_db_and_loc_tables_from_cache(true, false, true, true) {
                                        Self::get_dependency_data_from_vanilla_and_modded_tables(&mut references, (ref_table, ref_column, &lookup_data), &dependencies)
                                    } else { false }
                                } else { Self::get_dependency_data_from_vanilla_and_modded_tables(&mut references, (ref_table, ref_column, &lookup_data), vanilla_dependencies) };

                                if fake_found && !real_found {
                                    references.referenced_table_is_ak_only = true;
                                }

                                if let Some(ref schema) = *schema {
                                    if let Ok(ref_definition) = schema.get_ref_last_definition_db(ref_table, dependencies) {
                                        if ref_definition.get_localised_fields().iter().any(|x| x.get_name() == ref_column) {
                                            references.referenced_column_is_localised = true;
                                        }
                                    }
                                }

                                Some((column as i32, references))
                            } else { None }
                        } else { None }
                    }).collect::<BTreeMap<i32, DependencyData>>();

                    // Wait to have the lock, because this can trigger crashes due to multiple threads trying to write to the cache at the same time.
                    loop {
                        if let Ok(ref mut cache) = dependencies.get_ref_cached_data().write() {
                            cache.insert(table_name.to_owned(), cached_data.clone());
                            break;
                        }
                    }
                    cached_data
                }
            }
        } else { BTreeMap::new() };

        let local_references = table_definition.get_fields_processed().into_par_iter().enumerate().filter_map(|(column, field)| {
            if let Some((ref ref_table, ref ref_column)) = field.get_is_reference() {
                if !ref_table.is_empty() && !ref_column.is_empty() {

                    // Get his lookup data if it has it.
                    let lookup_data = if let Some(ref data) = field.get_lookup() { data.to_vec() } else { Vec::with_capacity(0) };
                    let mut references = DependencyData::default();

                    let _local_found = Self::get_dependency_data_from_packfile(&mut references, (ref_table, ref_column, &lookup_data), pack_file, files_to_ignore);

                    Some((column as i32, references))
                } else { None }
            } else { None }
        }).collect::<BTreeMap<i32, DependencyData>>();

        vanilla_references.par_iter_mut().for_each(|(key, value)|
            if let Some(local_value) = local_references.get(key) {
                value.data.extend(local_value.data.iter().map(|(k, v)| (k.clone(), v.clone())));
            }
        );

        vanilla_references
    }

    /// This function is used to check if a table is outdated or not.
    pub fn is_outdated(&self, dependencies: &Dependencies) -> bool {
        if let Ok(vanilla_dbs) = dependencies.get_db_tables_from_cache(self.get_ref_table_name(), true, false) {
            if let Some(vanilla_db) = vanilla_dbs.iter()
                .max_by(|x, y| x.get_ref_definition().get_version().cmp(&y.get_ref_definition().get_version())) {
                if vanilla_db.get_ref_definition().get_version() != self.get_ref_definition().get_version() {
                    return true;
                }
            }
        }

        false
    }

    /// This mess of a function performs a recursive/cascade editing over the related files on a PackFile.
    ///
    /// It edits:
    /// - References to the cell we're editing (recursive).
    /// - Loc references to the table we're editing.
    ///
    /// It returns the list of edited paths.
    pub fn cascade_edition(editions: &CascadeEdition, pack_file: &mut PackFile) -> Vec<Vec<String>> {
        let mut edited_paths = vec![];

        for (column, data_changes) in editions.get_ref_data_changes() {

            // This little boy is the one that contains the list of precalculated tables to edit.
            // Get the ones for the column we're currently editing, then get all the tables we need to edit.
            if let Some((ref_table_data, _)) = editions.get_ref_referenced_tables().get(column) {
                for (ref_table_name, ref_column_names) in ref_table_data {
                    for packed_file in pack_file.get_ref_mut_packed_files_by_path_start(&["db".to_owned(), ref_table_name.to_string()]) {
                        let path = packed_file.get_path().to_vec();
                        if let DecodedPackedFile::DB(table) = packed_file.get_ref_mut_decoded() {
                            let mut table_data = table.get_table_data();

                            // Find the column to edit within the table.
                            for ref_column_name in ref_column_names {
                                if let Some(column) = table.get_definition().get_fields_processed().iter().position(|x| x.get_name() == ref_column_name) {

                                    // Then, only if the data has changed, go through all the rows and perform the edits.
                                    for (old_data, new_data) in data_changes {
                                        if old_data != new_data {
                                            for row in &mut table_data {
                                                if let Some(field_data) = row.get_mut(column) {
                                                    match field_data {
                                                        DecodedData::StringU8(field_data) |
                                                        DecodedData::StringU16(field_data) |
                                                        DecodedData::OptionalStringU8(field_data) |
                                                        DecodedData::OptionalStringU16(field_data) => {

                                                            // Only edit exact matches.
                                                            if field_data == old_data {
                                                                *field_data = new_data.to_owned();

                                                                if !edited_paths.contains(&path) {
                                                                    edited_paths.push(path.to_vec());
                                                                }
                                                            }
                                                        }
                                                        _ => continue
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }

                            // Set the table's data. Let's hope nothing wrong happened.
                            let _ = table.set_table_data(&table_data);
                        }
                    }
                }
            }

            // Now, with locs. First, this is done only if our definition has localised fields and we edited the "key" field of our table.
            if let Some(field) = editions.get_ref_edited_table_definition().get_fields_processed().get(*column as usize) {
                if field.get_is_key() || field.get_name().to_lowercase() == "key" {
                    if !editions.get_ref_edited_table_definition().get_localised_fields().is_empty() {

                        for packed_file in pack_file.get_ref_mut_packed_files_by_type(PackedFileType::Loc, false) {
                            let path = packed_file.get_path().to_vec();
                            if let DecodedPackedFile::Loc(table) = packed_file.get_ref_mut_decoded() {
                                let mut table_data = table.get_table_data();
                                for row in &mut table_data {
                                    for (old_data, new_data) in data_changes {

                                        // Same as with the tables, but here the column is always 0 and the entry structure is:
                                        // "tablenamewithout_tables"_"localisedcolumnname"_"editedkey".
                                        for loc_field in editions.get_ref_edited_table_definition().get_localised_fields() {
                                            let short_table_name = if editions.get_ref_edited_table_name().ends_with("_tables") {
                                                editions.get_ref_edited_table_name().split_at(editions.get_ref_edited_table_name().len() - 7).0
                                            } else { editions.get_ref_edited_table_name() };

                                            let old_localised_key = format!("{}_{}_{}", short_table_name, loc_field.get_name(), &old_data);
                                            let new_localised_key = format!("{}_{}_{}", short_table_name, loc_field.get_name(), &new_data);

                                            if old_localised_key != new_localised_key {
                                                match &mut row[0] {
                                                    DecodedData::StringU8(field_data) |
                                                    DecodedData::StringU16(field_data) |
                                                    DecodedData::OptionalStringU8(field_data) |
                                                    DecodedData::OptionalStringU16(field_data) => {
                                                        if *field_data == old_localised_key {
                                                            *field_data = new_localised_key;

                                                            if !edited_paths.contains(&path) {
                                                                edited_paths.push(path.to_vec());
                                                            }
                                                        }
                                                    }
                                                    _ => continue
                                                }
                                            }
                                        }
                                    }
                                }

                                let _ = table.set_table_data(&table_data);

                            }
                        }
                    }
                }
            }
        }

        edited_paths
    }

    /// This function imports a TSV file into a decoded table.
    pub fn import_tsv(
        schema: &Schema,
        path: &Path,
    ) -> Result<(Self, Option<Vec<String>>)> {
        let (table, file_path) = Table::import_tsv(schema, path)?;
        let db = DB::from(table);
        Ok((db, file_path))
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

    /// This function imports a TSV file into a binary file on disk.
    pub fn import_tsv_to_binary_file(
        schema: &Schema,
        source_paths: &[PathBuf],
    ) -> Result<()> {
        for path in source_paths {
            let mut destination = path.clone();
            destination.set_extension("");
            Table::import_tsv_to_binary_file(schema, path, &destination)?;
        }

        Ok(())
    }

    /// This function exports to TSV a binary file on disk.
    pub fn export_tsv_from_binary_file(
        schema: &Schema,
        source_paths: &[PathBuf],
    ) -> Result<()> {
        for path in source_paths {
            let mut destination = path.clone();
            destination.set_extension("tsv");
            Table::export_tsv_from_binary_file(schema, path, &destination)?;
        }

        Ok(())
    }
}

/// Implementation to create a `DB` from a `Table`.
impl From<Table> for DB {
    fn from(table: Table) -> Self {
        Self {
            name: String::new(),
            mysterious_byte: true,
            uuid: Uuid::new_v4().to_string(),
            table,
        }
    }
}

/// Implementation to create a `DB` from a `RawTable`.
impl From<&RawTable> for DB {
    fn from(raw_table: &RawTable) -> Self {
        let name_table = if let Some(ref x) = raw_table.definition {
            if let Some(ref y) = x.name {

                // Remove the .xml of the name in the most awesome way there is.
                let mut x = y.to_owned();
                x.pop();
                x.pop();
                x.pop();
                x.pop();

                format!("{}_tables", x)
            }
            else { String::new() }
        } else { String::new() };

        Self {
            name: name_table,
            mysterious_byte: true,
            uuid: Uuid::new_v4().to_string(),
            table: From::from(raw_table),
        }
    }
}
