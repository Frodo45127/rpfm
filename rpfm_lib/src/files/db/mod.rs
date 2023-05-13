//---------------------------------------------------------------------------//
// Copyright (c) 2017-2023 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! DB files are tables that contain a lot of data used by the game in a database-like format.
//!
//! Think of these files as tables of a database. Each table may be split in files (fragments) or be a single file.
//!
//! They're sequencial, which means to decode them we need to know their definition, as in "first column is
//! of this type, second column is of this type,...". Also, they're versioned through an optional value in their header.
//!
//! If you want to know more about specific types a table
//! can have, check the [`Table`](crate::files::table) module. If you want to know more about definitions and
//! how to make them, check the [`Schema`](crate::schema) module.
//!
//! # DB Structure
//!
//! ## Header
//!
//! | Bytes  | Type            | Data                                                         |
//! | ------ | --------------- | ------------------------------------------------------------ |
//! | 4      | &\[[u8]\]       | GUID Marker. Optional.                                       |
//! | 2 + 72 | Sized StringU16 | GUID. Only present if GUID Marker is present too.            |
//! | 4      | &\[[u8]\]       | Version Marker. Optional.                                    |
//! | 4      | [u32]           | Version of the table. Only present if Version Marker is too. |
//! | 1      | [bool]          | Unknown. Probably a bool because it's always either 0 or 1.  |
//! | 4      | [u32]           | Amount of entries on the table.                              |
//!
//! ## Data
//!
//! The data structure depends on the definition of the table.

use csv::{StringRecordsIter, Writer};
use getset::Getters;
#[cfg(feature = "integration_sqlite")] use r2d2::Pool;
#[cfg(feature = "integration_sqlite")] use r2d2_sqlite::SqliteConnectionManager;
use rayon::prelude::*;
use serde_derive::{Serialize, Deserialize};
use uuid::Uuid;

use std::borrow::Cow;
#[cfg(test)] use std::collections::BTreeMap;
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::SeekFrom;

use crate::binary::{ReadBytes, WriteBytes};
use crate::error::{RLibError, Result};
use crate::files::{Container, ContainerPath, DecodeableExtraData, Decodeable, EncodeableExtraData, Encodeable, FileType, table::{DecodedData, Table}, pack::Pack, RFileDecoded};
#[cfg(test)] use crate::schema::FieldType;
use crate::schema::{Definition, DefinitionPatch, Field, Schema};
use crate::utils::check_size_mismatch;

/// If this sequence is found, the DB Table has a GUID after it.
const GUID_MARKER: &[u8] = &[253, 254, 252, 255];

/// If this sequence is found, the DB Table has a version number after it.
const VERSION_MARKER: &[u8] = &[252, 253, 254, 255];

#[cfg(test)] mod db_test;

//---------------------------------------------------------------------------//
//                              Enum & Structs
//---------------------------------------------------------------------------//

/// This holds an entire DB Table decoded in memory.
#[derive(PartialEq, Clone, Debug, Getters, Serialize, Deserialize)]
#[getset(get = "pub")]
pub struct DB {

    /// Don't know his use, but it's in all the tables I've seen, always being `1` or `0`.
    /// NOTE: In Warhammer 2, a 0 here seems to crash the game when the tables are loaded.
    mysterious_byte: bool,

    /// GUID of this table.
    guid: String,

    /// The table's data, containing all the stuff needed to decode/encode it.
    table: Table,
}

//---------------------------------------------------------------------------//
//                           Implementation of DB
//---------------------------------------------------------------------------//

impl Decodeable for DB {

    fn decode<R: ReadBytes>(data: &mut R, extra_data: &Option<DecodeableExtraData>) -> Result<Self> {
        let extra_data = extra_data.as_ref().ok_or(RLibError::DecodingMissingExtraData)?;
        let schema = extra_data.schema.ok_or_else(|| RLibError::DecodingMissingExtraDataField("schema".to_owned()))?;
        let table_name = extra_data.table_name.ok_or_else(|| RLibError::DecodingMissingExtraDataField("table_name".to_owned()))?;
        let return_incomplete = extra_data.return_incomplete;
        let pool = extra_data.pool;

        let (version, mysterious_byte, guid, entry_count) = Self::read_header(data)?;

        // Try to get the table_definition for this table, if exists.
        let definitions = schema.definitions_by_table_name(table_name).ok_or({
            if entry_count == 0 {
                RLibError::DecodingDBNoDefinitionsFoundAndEmptyFile
            } else {
                RLibError::DecodingDBNoDefinitionsFound
            }
        })?;

        // Try to decode the table.
        let len = data.len()?;
        let table = if version == 0 {
            let index_reset = data.stream_position()?;

            // For version 0 tables, get all definitions between 0 and -99, and get the first one that works.
            let mut working_definition = Err(RLibError::DecodingDBNoDefinitionsFound);
            for definition in definitions.iter().filter(|definition| *definition.version() < 1) {

                // First, reset the index in case it was changed in a previous iteration.
                // Then, check if the definition works.
                data.seek(SeekFrom::Start(index_reset))?;
                let db = Table::decode_table(data, definition, Some(entry_count), return_incomplete);
                if db.is_ok() && data.stream_position()? == len {
                    working_definition = Ok(definition);
                    break;
                }
            }

            let definition = working_definition?;
            let definition_patch = schema.patches_for_table(table_name).cloned().unwrap_or_default();

            // Reset the index before the table, and now decode the table with proper backend support.
            data.seek(SeekFrom::Start(index_reset))?;
            Table::decode(&pool, data, definition, &definition_patch, Some(entry_count), return_incomplete, table_name)?
        }

        // For +0 versions, we expect unique definitions.
        else {

            let definition = definitions.iter()
                .find(|definition| *definition.version() == version)
                .ok_or(RLibError::DecodingDBNoDefinitionsFound)?;

            let definition_patch = schema.patches_for_table(table_name).cloned().unwrap_or_default();
            Table::decode(&pool, data, definition, &definition_patch, Some(entry_count), return_incomplete, table_name)?
        };

        // If we are not in the last byte, it means we didn't parse the entire file, which means this file is corrupt, or the decoding failed and we bailed early.
        //
        // If we have return_incomplete enabled, we pass whatever we got decoded into this error.
        check_size_mismatch(data.stream_position()? as usize, len as usize).map_err(|error| {
            RLibError::DecodingTableIncomplete(error.to_string(), table.clone())
        })?;

        // If we've reached this, we've successfully decoded the table.
        Ok(Self {
            mysterious_byte,
            guid,
            table,
        })
    }
}

impl Encodeable for DB {

    fn encode<W: WriteBytes>(&mut self, buffer: &mut W, extra_data: &Option<EncodeableExtraData>) -> Result<()> {
        let pool = if let Some (ref extra_data) = extra_data { extra_data.pool } else { None };
        let table_has_guid = if let Some (ref extra_data) = extra_data { extra_data.table_has_guid } else { false };
        let regenerate_table_guid = if let Some (ref extra_data) = extra_data { extra_data.regenerate_table_guid } else { false };

        // Napoleon and Empire do not have GUID, and adding it to their tables crash both games.
        // So for those two games, remember that you have to ignore the GUID_MARKER and the GUID itself.
        if table_has_guid {
            buffer.write_all(GUID_MARKER)?;
            if regenerate_table_guid || self.guid.is_empty() {
                buffer.write_sized_string_u16(&Uuid::new_v4().to_string())?;
            } else {
                buffer.write_sized_string_u16(&self.guid)?;
            }
        }

        // Only put version numbers on tables with an actual version.
        if *self.table.definition().version() > 0 {
            buffer.write_all(VERSION_MARKER)?;
            buffer.write_i32(*self.table.definition().version())?;
        }

        buffer.write_bool(self.mysterious_byte)?;
        buffer.write_u32(self.table.len(pool)? as u32)?;

        self.table.encode(buffer, &None, &pool)
    }
}

impl DB {

    /// This function creates a new empty [DB] table.
    pub fn new(definition: &Definition, definition_patch: Option<&DefinitionPatch>, table_name: &str, use_sql_backend: bool) -> Self {
        let table = Table::new(definition, definition_patch, table_name, use_sql_backend);

        Self {
            mysterious_byte: true,
            guid: String::new(),
            table,
        }
    }

    /// This functions decodes the header part of a `DB` from a reader.
    ///
    /// The data returned is:
    /// - `version`: the version of this table.
    /// - `mysterious_byte`: don't know what this is.
    /// - `guid`: the GUID of this table.
    /// - `entry_count`: amount of entries this `DB` has.
    pub fn read_header<R: ReadBytes>(data: &mut R) -> Result<(i32, bool, String, u32)> {

        // 5 is the minimum amount of bytes a valid DB Table can have. If there is less, either the table is broken,
        // or the data is not from a DB Table.
        if data.len()? < 5 {
            return Err(RLibError::DecodingDBNotADBTable);
        }

        // If there is a GUID_MARKER, get the GUID and store it. If not, just store an empty string.
        let guid = if data.read_slice(4, false)? == GUID_MARKER {
            data.read_sized_string_u16()?
        } else {
            data.seek(SeekFrom::Current(-4))?;
            String::new()
        };

        // If there is a VERSION_MARKER, we get the version (4 bytes for the marker, 4 for the version).
        // Otherwise, we default to version 0.
        let version = if data.read_slice(4, false)? == VERSION_MARKER {
            data.read_i32()?
        } else {
            data.seek(SeekFrom::Current(-4))?;
            0
        };

        // We get the rest of the data from the header.
        let mysterious_byte = data.read_bool()?;
        let entry_count = data.read_u32()?;
        Ok((version, mysterious_byte, guid, entry_count))
    }

    /// This function returns a reference of the definition of this DB Table.
    pub fn definition(&self) -> &Definition {
        self.table.definition()
    }

    /// This function returns a reference of the definition patches of this DB Table.
    pub fn patches(&self) -> &DefinitionPatch {
        self.table.patches()
    }

    /// This function returns a reference of the name of this DB Table.
    pub fn table_name(&self) -> &str {
        self.table.table_name()
    }

    /// This function returns the name of this DB Table, without the "_tables" suffix.
    pub fn table_name_without_tables(&self) -> String {

        // Note: it needs this check because this explodes if instead of "_tables" we have non-ascii characters.
        if self.table_name().ends_with("_tables") {
            self.table_name().to_owned().drain(..self.table_name().len() - 7).collect()
        } else {
            panic!("Either the code is broken, or someone with a few loose screws has renamed the fucking table folder. Crash for now, may return an error in the future.")
        }
    }

    /// This function returns a reference to the entries of this DB table.
    pub fn data(&self, pool: &Option<&Pool<SqliteConnectionManager>>) -> Result<Cow<[Vec<DecodedData>]>> {
        self.table.data(pool)
    }

    /// This function returns a reference to the entries of this DB table.
    ///
    /// Make sure to keep the table structure valid for the table definition.
    pub fn data_mut(&mut self) -> Result<&mut Vec<Vec<DecodedData>>> {
        self.table.data_mut()
    }

    /// This function replaces the data of this table with the one provided.
    ///
    /// This can (and will) fail if the data is not in the format defined by the definition of the table.
    pub fn set_data(&mut self, pool: Option<&Pool<SqliteConnectionManager>>, data: &[Vec<DecodedData>]) -> Result<()> {
        self.table.set_data(pool, data)
    }

    /// This function returns a valid empty (with default values if any) row for this table.
    pub fn new_row(&self) -> Vec<DecodedData> {
        Table::new_row(self.definition(), Some(self.patches()))
    }

    /// This function returns the definition of a table.
    #[cfg(test)]
    pub fn test_definition() -> Definition {
        let mut definition = Definition::new(-100, None);
        let mut fields = vec![];

        fields.push(Field::new("bool".to_owned(), FieldType::Boolean, false, Some("true".to_string()), false, None, None, None, String::new(), 0, 0, BTreeMap::new(), None));
        fields.push(Field::new("f32".to_owned(), FieldType::F32, false, Some("1.0".to_string()), false, None, None, None, String::new(), 0, 0, BTreeMap::new(), None));
        fields.push(Field::new("f64".to_owned(), FieldType::F64, false, Some("2.0".to_string()), false, None, None, None, String::new(), 0, 0, BTreeMap::new(), None));
        fields.push(Field::new("i16".to_owned(), FieldType::I16, false, Some("3".to_string()), false, None, None, None, String::new(), 0, 0, BTreeMap::new(), None));
        fields.push(Field::new("i32".to_owned(), FieldType::I32, false, Some("4".to_string()), false, None, None, None, String::new(), 0, 0, BTreeMap::new(), None));
        fields.push(Field::new("i64".to_owned(), FieldType::I64, false, Some("5".to_string()), false, None, None, None, String::new(), 0, 0, BTreeMap::new(), None));
        fields.push(Field::new("colour".to_owned(), FieldType::ColourRGB, false, Some("ABCDEF".to_string()), false, None, None, None, String::new(), 0, 0, BTreeMap::new(), None));
        fields.push(Field::new("stringu8".to_owned(), FieldType::StringU8, false, Some("AAAA".to_string()), false, None, None, None, String::new(), 0, 0, BTreeMap::new(), None));
        fields.push(Field::new("stringu16".to_owned(), FieldType::StringU16, false, Some("BBBB".to_string()), false, None, None, None, String::new(), 0, 0, BTreeMap::new(), None));
        fields.push(Field::new("optionali16".to_owned(), FieldType::OptionalI16, false, Some("3".to_string()), false, None, None, None, String::new(), 0, 0, BTreeMap::new(), None));
        fields.push(Field::new("optionali32".to_owned(), FieldType::OptionalI32, false, Some("4".to_string()), false, None, None, None, String::new(), 0, 0, BTreeMap::new(), None));
        fields.push(Field::new("optionali64".to_owned(), FieldType::OptionalI64, false, Some("5".to_string()), false, None, None, None, String::new(), 0, 0, BTreeMap::new(), None));
        fields.push(Field::new("optionalstringu8".to_owned(), FieldType::OptionalStringU8, false, Some("Opt".to_string()), false, None, None, None, String::new(), 0, 0, BTreeMap::new(), None));
        fields.push(Field::new("optionalstringu16".to_owned(), FieldType::OptionalStringU16, false, Some("Opt".to_string()), false, None, None, None, String::new(), 0, 0, BTreeMap::new(), None));
        fields.push(Field::new("sequenceu16".to_owned(), FieldType::SequenceU16(Box::new(Definition::new(-100, None))), false, None, false, None, None, None, String::new(), 0, 0, BTreeMap::new(), None));
        fields.push(Field::new("sequenceu32".to_owned(), FieldType::SequenceU32(Box::new(Definition::new(-100, None))), false, None, false, None, None, None, String::new(), 0, 0, BTreeMap::new(), None));

        // Special fields that use postprocessing.
        fields.push(Field::new("merged_colours_1_r".to_owned(), FieldType::I32, false, Some("AB".to_string()), false, None, None, None, String::new(), 0, 0, BTreeMap::new(), Some(0)));
        fields.push(Field::new("merged_colours_1_g".to_owned(), FieldType::I32, false, Some("CD".to_string()), false, None, None, None, String::new(), 0, 0, BTreeMap::new(), Some(0)));
        fields.push(Field::new("merged_colours_1_b".to_owned(), FieldType::I32, false, Some("EF".to_string()), false, None, None, None, String::new(), 0, 0, BTreeMap::new(), Some(0)));
        fields.push(Field::new("bitwise_values".to_owned(), FieldType::I32, false, Some("4".to_string()), false, None, None, None, String::new(), 0, 5, BTreeMap::new(), None));
        fields.push(Field::new("enum_values".to_owned(), FieldType::I32, false, Some("8".to_string()), false, None, None, None, String::new(), 0, 0, {
            let mut bt = BTreeMap::new();
            bt.insert(0, "test0".to_owned());
            bt.insert(1, "test1".to_owned());
            bt.insert(2, "test2".to_owned());
            bt.insert(3, "test3".to_owned());
            bt
        }, None));

        fields.push(Field::new("merged_colours_2_r".to_owned(), FieldType::I32, false, Some("AB".to_string()), false, None, None, None, String::new(), 0, 0, BTreeMap::new(), Some(1)));
        fields.push(Field::new("merged_colours_2_g".to_owned(), FieldType::I32, false, Some("CD".to_string()), false, None, None, None, String::new(), 0, 0, BTreeMap::new(), Some(1)));
        fields.push(Field::new("merged_colours_2_b".to_owned(), FieldType::I32, false, Some("EF".to_string()), false, None, None, None, String::new(), 0, 0, BTreeMap::new(), Some(1)));

        // TODO: add combined colour columns for testing.

        definition.set_fields(fields);
        definition
    }

    /// This function returns the position of a column in a definition, or None if the column is not found.
    pub fn column_position_by_name(&self, column_name: &str) -> Option<usize> {
        self.table.column_position_by_name(column_name)
    }

    /// This function returns the amount of entries in this DB Table.
    pub fn len(&self, pool: Option<&Pool<SqliteConnectionManager>>) -> Result<usize> {
        self.table.len(pool)
    }

    /// This function replaces the definition of this table with the one provided.
    ///
    /// This updates the table's data to follow the format marked by the new definition, so you can use it to *update* the version of your table.
    pub fn set_definition(&mut self, new_definition: &Definition) {
        self.table.set_definition(new_definition);
    }

    /// This function updates the current table to a new definition.
    pub fn update(&mut self, new_definition: &Definition) {
        self.set_definition(new_definition)
    }

    /// This function performs a cascade update of DB/Loc values across an entire Pack, making sure
    /// all references to the edited value are updated accordingly.
    ///
    /// It returns the list of ContainerPath where said reference has been found and updated.
    pub fn cascade_edition(pack: &mut Pack, schema: &Option<Schema>, table_name: &str, field: &Field, definition: &Definition, value_before: &str, value_after: &str) -> Vec<ContainerPath> {

        // So, how does this work:
        // - First, we need to calculate all related tables/columns. This includes the parent columns if this is a reference, and all references to this field.
        // - Second, we need to calculate all related loc fields corresponding to the edited fiels.
        // - Third, we edit the table entries.
        // - Fourth, we edit the loc entries.
        let mut edited_paths = vec![];

        // If we're not changing anything, don't bother performing an edition.
        if value_before == value_after {
            return vec![];
        }

        // Just in case we're in a reference field, find the source, and trigger the edition from there.
        let mut definition = definition.clone();
        let mut field = field.clone();
        let mut table_name = table_name.to_owned();
        while let Some((ref_table, ref_column)) = field.is_reference() {
            let ref_table_name = format!("{ref_table}_tables");
            let table_folder = format!("db/{ref_table_name}");
            let parent_files = pack.files_by_type_and_paths_mut(&[FileType::DB], &[ContainerPath::Folder(table_folder.to_owned())], true);
            if !parent_files.is_empty() {
                if let Ok(RFileDecoded::DB(table)) = parent_files[0].decoded() {
                    if let Some(index) = table.definition().column_position_by_name(ref_column) {
                        definition = table.definition().clone();
                        field = definition.fields_processed()[index].clone();
                        table_name = table.table_name().to_owned();
                        continue;
                    }
                }
            }

            break;
        }

        // Get the tables/rows that need to be edited.
        let fields_processed = definition.fields_processed();
        let fields_localised = definition.localised_fields();
        let referenced_tables = Table::tables_and_columns_referencing_our_own(schema, &table_name, field.name(), &fields_processed, fields_localised);
        if let Some((mut ref_table_data, _)) = referenced_tables {

            // Add the source table and column to the list to edit.
            ref_table_data.insert(table_name, vec![field.name().to_owned()]);

            let container_paths = ref_table_data.keys().map(|ref_table_name| ContainerPath::Folder("db/".to_owned() + ref_table_name)).collect::<Vec<_>>();
            let mut files = pack.files_by_paths_mut(&container_paths, true);
            let mut loc_keys: Vec<(String, String)> = vec![];

            for file in files.iter_mut() {
                let path = file.path_in_container();
                if let Ok(RFileDecoded::DB(table)) = file.decoded_mut() {
                    let fields_processed = table.definition().fields_processed();
                    let fields_localised = table.definition().localised_fields().to_vec();
                    let localised_order = table.definition().localised_key_order().to_vec();
                    let patches = table.definition().patches().clone();
                    let table_name = table.table_name().to_owned();
                    let table_name_no_tables = table.table_name_without_tables();
                    let table_data = table.data_mut().unwrap();

                    let mut keys_edited = vec![];

                    // Find the column to edit within the table.
                    let column_indexes = fields_processed.iter()
                        .enumerate()
                        .filter_map(|(index, field)| if ref_table_data[&table_name].iter().any(|name| name == field.name()) { Some(index) } else { None })
                        .collect::<Vec<usize>>();

                    // Then, go through all the rows and perform the edits.
                    for row in table_data.iter_mut() {
                        for column in &column_indexes {

                            // TODO: FIX THIS SHIT. It duplicates ALL DATA IN ALL TABLES CHECK.
                            let row_copy = row.to_vec();

                            if let Some(field_data) = row.get_mut(*column) {
                                match field_data {
                                    DecodedData::StringU8(field_data) |
                                    DecodedData::StringU16(field_data) |
                                    DecodedData::OptionalStringU8(field_data) |
                                    DecodedData::OptionalStringU16(field_data) => {

                                        // Only edit exact matches.
                                        if field_data == value_before {
                                            let mut locs_edited = vec![];

                                            // If it's a key, calculate the relevant before and after loc keys.
                                            let is_key = fields_processed[*column].is_key(Some(&patches));
                                            if is_key {
                                                for loc_field in &fields_localised {
                                                    let loc_key = localised_order.iter().map(|pos| row_copy[*pos as usize].data_to_string()).collect::<Vec<_>>().join("");
                                                    locs_edited.push(format!("{}_{}_{}", table_name_no_tables, loc_field.name(), loc_key));
                                                }
                                            }

                                            *field_data = value_after.to_owned();

                                            if !locs_edited.is_empty() {
                                                for (index, loc_field) in fields_localised.iter().enumerate() {
                                                    if let Some(key_old) = locs_edited.get(index) {
                                                        let loc_key = localised_order.iter().map(|pos| row[*pos as usize].data_to_string()).collect::<Vec<_>>().join("");
                                                        let key_new = format!("{}_{}_{}", table_name_no_tables, loc_field.name(), loc_key);
                                                        keys_edited.push((key_old.to_owned(), key_new.to_owned()))
                                                    }
                                                }
                                            }

                                            if !edited_paths.contains(&path) {
                                                edited_paths.push(path.clone());
                                            }
                                        }
                                    }
                                    _ => continue
                                }
                            }
                        }
                    }

                    // If we edited a key field in the table, check if we need to edit any relevant loc field.
                    if !keys_edited.is_empty() {
                        loc_keys.append(&mut keys_edited);
                    }
                }
            }

            // Now, we find and replace all the loc keys we have to change.
            let mut loc_files = pack.files_by_type_mut(&[FileType::Loc]);
            for file in &mut loc_files {
                let path = file.path_in_container();
                if let Ok(RFileDecoded::Loc(data)) = file.decoded_mut() {
                    let data = data.data_mut().unwrap();
                    for row in data.iter_mut() {
                        if let Some(field_data) = row.get_mut(0) {
                            match field_data {
                                DecodedData::StringU8(field_data) |
                                DecodedData::StringU16(field_data) |
                                DecodedData::OptionalStringU8(field_data) |
                                DecodedData::OptionalStringU16(field_data) => {
                                    for (key_old, key_new) in &loc_keys {
                                        if field_data == key_old {
                                            *field_data = key_new.to_owned();

                                            if !edited_paths.contains(&path) {
                                                edited_paths.push(path.clone());
                                            }
                                        }
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
        }

        edited_paths
    }

    /// This function merges the data of a few DB tables into a new DB table.
    ///
    /// The metadata used (definition, patches) is taken from the first table on the list.
    ///
    /// May fail if the tables do not have the same table name.
    pub(crate) fn merge(sources: &[&Self]) -> Result<Self> {

        let table_names = sources.iter().map(|file| file.table_name()).collect::<HashSet<_>>();
        if table_names.len() > 1 {
            return Err(RLibError::RFileMergeTablesDifferentNames);
        }

        let mut new_table = Self::new(sources[0].definition(), Some(sources[0].patches()), sources[0].table_name(), false);
        let sources = sources.par_iter()
            .map(|table| {
                let mut table = table.table().clone();
                table.set_definition(new_table.definition());
                table
            })
            .collect::<Vec<_>>();

        let new_data = sources.par_iter()
            .filter_map(|table| table.data(&None).ok())
            .map(|data| data.to_vec())
            .flatten()
            .collect::<Vec<_>>();
        new_table.set_data(None, &new_data)?;

        Ok(new_table)
    }

    /// This function imports a TSV file into a decoded table.
    pub fn tsv_import(records: StringRecordsIter<File>, field_order: &HashMap<u32, String>, schema: &Schema, table_name: &str, table_version: i32) -> Result<Self> {
        let definition = schema.definition_by_name_and_version(table_name, table_version).ok_or(RLibError::DecodingDBNoDefinitionsFound)?;
        let definition_patch = schema.patches_for_table(table_name);
        let table = Table::tsv_import(records, definition, field_order, table_name, definition_patch)?;
        let db = DB::from(table);
        Ok(db)
    }

    /// This function imports a TSV file into a decoded table.
    pub fn tsv_export(&self, writer: &mut Writer<File>, table_path: &str) -> Result<()> {
        self.table.tsv_export(writer, table_path)
    }
}

/// Implementation to create a `DB` from a `Table`.
impl From<Table> for DB {
    fn from(table: Table) -> Self {
        Self {
            mysterious_byte: true,
            guid: Uuid::new_v4().to_string(),
            table,
        }
    }
}
