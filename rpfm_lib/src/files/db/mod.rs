//---------------------------------------------------------------------------//
// Copyright (c) 2017-2022 Ismael Gutiérrez González. All rights reserved.
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
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use serde_derive::{Serialize, Deserialize};
use uuid::Uuid;

use std::borrow::Cow;
use std::collections::{BTreeMap, HashMap};
use std::fs::File;
use std::io::SeekFrom;

use crate::binary::{ReadBytes, WriteBytes};
use crate::error::{RLibError, Result};
use crate::files::{DecodeableExtraData, Decodeable, EncodeableExtraData, Encodeable, table::{DecodedData, Table}};
use crate::schema::{Definition, DefinitionPatch, Field, FieldType, Schema};
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
/*
/// This holds all the data needed to trigger cascade editions.
///
/// We use a struct because Cascade Editions need a lot of different data, and it's a mess to deal with all of it independently.
#[derive(Clone, Debug, Default, Getters, MutGetters, Set)]
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
*/
//---------------------------------------------------------------------------//
//                           Implementation of DB
//---------------------------------------------------------------------------//

impl Decodeable for DB {

    fn decode<R: ReadBytes>(data: &mut R, extra_data: &Option<DecodeableExtraData>) -> Result<Self> {
        let extra_data = extra_data.as_ref().ok_or(RLibError::DecodingMissingExtraData)?;
        let schema = extra_data.schema.ok_or(RLibError::DecodingMissingExtraDataField("schema".to_owned()))?;
        let table_name = extra_data.table_name.ok_or(RLibError::DecodingMissingExtraDataField("table_name".to_owned()))?;
        let return_incomplete = extra_data.return_incomplete;
        let pool = extra_data.pool;

        let (version, mysterious_byte, guid, entry_count) = Self::read_header(data)?;

        // Try to get the table_definition for this table, if exists.
        let definitions = schema.definitions_by_table_name(table_name).ok_or_else(|| {
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
                if db.is_ok() {
                    if data.stream_position()? == len {
                        working_definition = Ok(definition);
                        break;
                    }
                }
            }

            let definition = working_definition?;
            let definition_patch = schema.patches_for_table(table_name).cloned().unwrap_or(HashMap::new());

            // Reset the index before the table, and now decode the table with proper backend support.
            data.seek(SeekFrom::Start(index_reset))?;
            let table = Table::decode(&pool, data, &definition, &definition_patch, Some(entry_count), return_incomplete, table_name)?;
            table
        }

        // For +0 versions, we expect unique definitions.
        else {

            let definition = definitions.iter()
                .find(|definition| *definition.version() == version)
                .ok_or_else(|| {
                    RLibError::DecodingDBNoDefinitionsFound
                })?;

            let definition_patch = schema.patches_for_table(table_name).cloned().unwrap_or(HashMap::new());
            let table = Table::decode(&pool, data, &definition, &definition_patch, Some(entry_count), return_incomplete, table_name)?;
            table
        };

        // If we are not in the last byte, it means we didn't parse the entire file, which means this file is corrupt, or the decoding failed and we bailed early.
        check_size_mismatch(data.stream_position()? as usize, len as usize)?;

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
            if regenerate_table_guid && !self.guid.is_empty() {
                buffer.write_sized_string_u16(&self.guid)?;
            } else {
                buffer.write_sized_string_u16(&Uuid::new_v4().to_string())?;
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
        let table = Table::new(&definition, definition_patch, table_name, use_sql_backend);

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

    /// This function returns the definition of a Loc table.
    pub fn test_definition() -> Definition {
        let mut definition = Definition::new(-100);
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
        fields.push(Field::new("sequenceu16".to_owned(), FieldType::SequenceU16(Box::new(Definition::new(-100))), false, None, false, None, None, None, String::new(), 0, 0, BTreeMap::new(), None));
        fields.push(Field::new("sequenceu32".to_owned(), FieldType::SequenceU32(Box::new(Definition::new(-100))), false, None, false, None, None, None, String::new(), 0, 0, BTreeMap::new(), None));

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

    /*

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
*/
    /// This function imports a TSV file into a decoded table.
    pub fn tsv_import(records: StringRecordsIter<File>, field_order: &HashMap<u32, String>, schema: &Schema, table_name: &str, table_version: i32) -> Result<Self> {
        let definition = schema.definition_by_name_and_version(table_name, table_version).ok_or(RLibError::DecodingDBNoDefinitionsFound)?;
        let definition_patch = schema.patches_for_table(table_name);
        let table = Table::tsv_import(records, &definition, field_order, table_name, definition_patch)?;
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
