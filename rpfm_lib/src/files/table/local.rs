//---------------------------------------------------------------------------//
// Copyright (c) 2017-2024 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Module to hold all table functions specific of the local backend.

use base64::{Engine, engine::general_purpose::STANDARD};
use csv::{StringRecordsIter, Writer};
use getset::*;
#[cfg(feature = "integration_sqlite")]use r2d2::Pool;
#[cfg(feature = "integration_sqlite")]use r2d2_sqlite::SqliteConnectionManager;
#[cfg(feature = "integration_sqlite")]use rusqlite::params_from_iter;
use serde_derive::{Serialize, Deserialize};

use std::borrow::Cow;
use std::collections::HashMap;
use std::fs::File;

use crate::binary::{ReadBytes, WriteBytes};
use crate::error::{Result, RLibError};
use crate::files::table::DecodedData;
//#[cfg(feature = "integration_log")] use crate::integrations::log::{info, warn};
use crate::schema::{Definition, DefinitionPatch, FieldType};
use crate::utils::parse_str_as_bool;

use super::{Table, decode_table, encode_table};

//---------------------------------------------------------------------------//
//                              Enum & Structs
//---------------------------------------------------------------------------//

/// This struct contains the data of a Table-like PackedFile after being decoded.
///
/// This is for internal use. If you need to interact with this in any way, do it through the PackedFile that contains it, not directly.
#[derive(Clone, Debug, PartialEq, Getters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", set = "pub")]
pub struct TableInMemory {

    /// A copy of the `Definition` this table uses, so we don't have to check the schema everywhere.
    table_name: String,

    #[getset(skip)]
    definition: Definition,

    #[getset(skip)]
    definition_patch: DefinitionPatch,

    #[getset(skip)]
    table_data: Vec<Vec<DecodedData>>,

    #[serde(skip)]
    #[cfg(feature = "integration_sqlite")]table_unique_id: u64,
}

//----------------------------------------------------------------//
// Implementations for `Table`.
//----------------------------------------------------------------//

impl TableInMemory {

    /// This function creates a new Table from an existing definition.
    pub fn new(definition: &Definition, definition_patch: Option<&DefinitionPatch>, table_name: &str) -> Self {
        let table_data = vec![];
        let definition_patch = if let Some(patch) = definition_patch { patch.clone() } else { HashMap::new() };

        Self {
            definition: definition.clone(),
            definition_patch,
            table_name: table_name.to_owned(),
            table_data,
            #[cfg(feature = "integration_sqlite")]table_unique_id: rand::random::<u64>(),
        }
    }

    pub fn decode<R: ReadBytes>(
        data: &mut R,
        definition: &Definition,
        definition_patch: &DefinitionPatch,
        entry_count: Option<u32>,
        return_incomplete: bool,
        table_name: &str,
    ) -> Result<Self> {

        let table_data = decode_table(data, definition, entry_count, return_incomplete)?;
        let table = Self {
            definition: definition.clone(),
            definition_patch: definition_patch.clone(),
            table_name: table_name.to_owned(),
            table_data,
            #[cfg(feature = "integration_sqlite")]table_unique_id: rand::random::<u64>(),
        };

        Ok(table)
    }

    pub fn encode<W: WriteBytes>(&self, data: &mut W) -> Result<()> {
        encode_table(&self.data(), data, self.definition(), &Some(self.patches()))
    }

    //----------------------------------------------------------------//
    // TSV Functions for tables.
    //----------------------------------------------------------------//
    // TODO: Make tsv trait.


    /// This function tries to imports a TSV file on the path provided into a binary db table.
    pub(crate) fn tsv_import(records: StringRecordsIter<File>, definition: &Definition, field_order: &HashMap<u32, String>, table_name: &str, schema_patches: Option<&DefinitionPatch>) -> Result<Self> {
        let mut table = Self::new(definition, schema_patches, table_name);
        let mut entries = vec![];

        let fields_processed = definition.fields_processed();

        for (row, record) in records.enumerate() {
            match record {
                Ok(record) => {
                    let mut entry = table.new_row();
                    for (column, field) in record.iter().enumerate() {

                        // Get the column name from the header, and try to map it to a column in the table's.
                        if let Some(column_name) = field_order.get(&(column as u32)) {
                            if let Some(column_number) = fields_processed.iter().position(|x| x.name() == column_name) {

                                entry[column_number] = match fields_processed[column_number].field_type() {
                                    FieldType::Boolean => parse_str_as_bool(field).map(DecodedData::Boolean).map_err(|_| RLibError::ImportTSVIncorrectRow(row, column))?,
                                    FieldType::F32 => DecodedData::F32(field.parse::<f32>().map_err(|_| RLibError::ImportTSVIncorrectRow(row, column))?),
                                    FieldType::F64 => DecodedData::F64(field.parse::<f64>().map_err(|_| RLibError::ImportTSVIncorrectRow(row, column))?),
                                    FieldType::I16 => DecodedData::I16(field.parse::<i16>().map_err(|_| RLibError::ImportTSVIncorrectRow(row, column))?),
                                    FieldType::I32 => DecodedData::I32(field.parse::<i32>().map_err(|_| RLibError::ImportTSVIncorrectRow(row, column))?),
                                    FieldType::I64 => DecodedData::I64(field.parse::<i64>().map_err(|_| RLibError::ImportTSVIncorrectRow(row, column))?),
                                    FieldType::OptionalI16 => DecodedData::OptionalI16(field.parse::<i16>().map_err(|_| RLibError::ImportTSVIncorrectRow(row, column))?),
                                    FieldType::OptionalI32 => DecodedData::OptionalI32(field.parse::<i32>().map_err(|_| RLibError::ImportTSVIncorrectRow(row, column))?),
                                    FieldType::OptionalI64 => DecodedData::OptionalI64(field.parse::<i64>().map_err(|_| RLibError::ImportTSVIncorrectRow(row, column))?),
                                    FieldType::ColourRGB => DecodedData::ColourRGB(if u32::from_str_radix(field, 16).is_ok() {
                                        field.to_owned()
                                    } else {
                                        Err(RLibError::ImportTSVIncorrectRow(row, column))?
                                    }),
                                    FieldType::StringU8 => DecodedData::StringU8(field.to_owned()),
                                    FieldType::StringU16 => DecodedData::StringU16(field.to_owned()),
                                    FieldType::OptionalStringU8 => DecodedData::OptionalStringU8(field.to_owned()),
                                    FieldType::OptionalStringU16 => DecodedData::OptionalStringU16(field.to_owned()),

                                    // For now fail on Sequences. These are a bit special and I don't know if the're even possible in TSV.
                                    FieldType::SequenceU16(_) => DecodedData::SequenceU16(STANDARD.decode(field).map_err(|_| RLibError::ImportTSVIncorrectRow(row, column))?),
                                    FieldType::SequenceU32(_) => DecodedData::SequenceU32(STANDARD.decode(field).map_err(|_| RLibError::ImportTSVIncorrectRow(row, column))?),
                                }
                            }
                        }
                    }
                    entries.push(entry);
                }
                Err(_) => return Err(RLibError::ImportTSVIncorrectRow(row, 0)),
            }
        }

        // If we reached this point without errors, we replace the old data with the new one and return success.
        table.set_data(&entries)?;
        Ok(table)
    }

    /// This function exports the provided data to a TSV file.
    pub(crate) fn tsv_export(&self, writer: &mut Writer<File>, table_path: &str, keys_first: bool) -> Result<()> {

        let fields_processed = self.definition().fields_processed();
        let fields_sorted = self.definition().fields_processed_sorted(keys_first);
        let fields_sorted_properly = fields_sorted.iter()
            .map(|field_sorted| (fields_processed.iter().position(|field| field == field_sorted).unwrap(), field_sorted))
            .collect::<Vec<(_,_)>>();

        // We serialize the info of the table (name and version) in the first line, and the column names in the second one.
        let metadata = (format!("#{};{};{}", self.table_name(), self.definition().version(), table_path), vec![String::new(); fields_sorted_properly.len() - 1]);
        writer.serialize(fields_sorted_properly.iter().map(|(_, field)| field.name()).collect::<Vec<&str>>())?;
        writer.serialize(metadata)?;

        // Then we serialize each entry in the DB Table.
        let entries = self.data();
        for entry in &*entries {
            let sorted_entry = fields_sorted_properly.iter()
                .map(|(index, _)| entry[*index].data_to_string())
                .collect::<Vec<Cow<str>>>();
            writer.serialize(sorted_entry)?;
        }

        writer.flush().map_err(From::from)
    }

    //----------------------------------------------------------------//
    // SQL functions for tables.
    //----------------------------------------------------------------//

    /// This function insert the table in memory into a sql database.
    #[cfg(feature = "integration_sqlite")]
    pub fn db_to_sql(&self, pool: &Pool<SqliteConnectionManager>) -> Result<()> {

        // Try to create the table, in case it doesn't exist yet. Ignore a failure here, as it'll mean the table already exists.
        let params: Vec<String> = vec![];
        let create_table = self.definition().map_to_sql_create_table_string(self.table_name());
        match pool.get()?.execute(&create_table, params_from_iter(params)) {
            Ok(_) => {
                //#[cfg(feature = "integration_log")] {
                //    info!("Table {} created succesfully.", self.table_name());
                //}
            },

            Err(error) => {
                //#[cfg(feature = "integration_log")] {
                //    warn!("Table {} failed to be created: {error}", self.table_name());
                //}
            },
        }

        self.insert_all_to_sql(pool)?;
        Ok(())
    }

    #[cfg(feature = "integration_sqlite")]
    pub fn sql_to_db(&mut self, pool: &Pool<SqliteConnectionManager>) -> Result<()> {
        self.table_data = self.select_all_from_sql(pool)?;
        Ok(())
    }

    /// This function inserts the provided rows of data into a database.
    #[cfg(feature = "integration_sqlite")]
    fn insert_all_to_sql(&self, pool: &Pool<SqliteConnectionManager>) -> Result<()> {
        let mut params = vec![];
        let values = self.table_data.iter().map(|row| {
            format!("({}, {})", self.table_unique_id, row.iter().map(|field| {
                match field {
                    DecodedData::Boolean(data) => if *data { "1".to_owned() } else { "0".to_owned() },
                    DecodedData::F32(data) => format!("{data:.4}"),
                    DecodedData::F64(data) => format!("{data:.4}"),
                    DecodedData::I16(data) => format!("\"{data}\""),
                    DecodedData::I32(data) => format!("\"{data}\""),
                    DecodedData::I64(data) => format!("\"{data}\""),
                    DecodedData::ColourRGB(data) => format!("\"{}\"", data.replace('\"', "\"\"")),
                    DecodedData::StringU8(data) => format!("\"{}\"", data.replace('\"', "\"\"")),
                    DecodedData::StringU16(data) => format!("\"{}\"", data.replace('\"', "\"\"")),
                    DecodedData::OptionalI16(data) => format!("\"{data}\""),
                    DecodedData::OptionalI32(data) => format!("\"{data}\""),
                    DecodedData::OptionalI64(data) => format!("\"{data}\""),
                    DecodedData::OptionalStringU8(data) => format!("\"{}\"", data.replace('\"', "\"\"")),
                    DecodedData::OptionalStringU16(data) => format!("\"{}\"", data.replace('\"', "\"\"")),
                    DecodedData::SequenceU16(data) => {
                        params.push(data.to_vec());
                        "?".to_owned()
                    },
                    DecodedData::SequenceU32(data) => {
                        params.push(data.to_vec());
                        "?".to_owned()
                    },
                }
            }).collect::<Vec<_>>().join(","))
        }).collect::<Vec<_>>().join(",");

        let query = format!("INSERT OR REPLACE INTO \"{}_v{}\" {} VALUES {}",
            self.table_name().replace('\"', "'"),
            self.definition().version(),
            self.definition().map_to_sql_insert_into_string(),
            values
        );

        pool.get()?.execute(&query, params_from_iter(params.iter()))
            .map(|_| ())
            .map_err(From::from)
    }

    /// This function inserts the provided rows of data into a database.
    #[cfg(feature = "integration_sqlite")]
    fn select_all_from_sql(&self, pool: &Pool<SqliteConnectionManager>) -> Result<Vec<Vec<DecodedData>>> {
        let definition = self.definition();
        let fields_processed = definition.fields_processed();

        let field_names = fields_processed.iter().map(|field| field.name()).collect::<Vec<&str>>().join(",");
        let query = format!("SELECT {} FROM \"{}_v{}\" WHERE table_unique_id = {} order by ROWID",
            field_names,
            self.table_name().replace('\"', "'"),
            definition.version(),
            self.table_unique_id()
        );

        let conn = pool.get()?;
        let mut stmt = conn.prepare(&query)?;
        let rows = stmt.query_map([], |row| {
            let mut data = Vec::with_capacity(fields_processed.len());
            for (i, field) in fields_processed.iter().enumerate() {
                data.push(match field.field_type() {
                    FieldType::Boolean => DecodedData::Boolean(row.get(i)?),
                    FieldType::F32 => DecodedData::F32(row.get(i)?),
                    FieldType::F64 => DecodedData::F64(row.get(i)?),
                    FieldType::I16 => DecodedData::I16(row.get(i)?),
                    FieldType::I32 => DecodedData::I32(row.get(i)?),
                    FieldType::I64 => DecodedData::I64(row.get(i)?),
                    FieldType::ColourRGB => DecodedData::ColourRGB(row.get(i)?),
                    FieldType::StringU8 => DecodedData::StringU8(row.get(i)?),
                    FieldType::StringU16 => DecodedData::StringU16(row.get(i)?),
                    FieldType::OptionalI16 => DecodedData::OptionalI16(row.get(i)?),
                    FieldType::OptionalI32 => DecodedData::OptionalI32(row.get(i)?),
                    FieldType::OptionalI64 => DecodedData::OptionalI64(row.get(i)?),
                    FieldType::OptionalStringU8 => DecodedData::OptionalStringU8(row.get(i)?),
                    FieldType::OptionalStringU16 => DecodedData::OptionalStringU16(row.get(i)?),
                    FieldType::SequenceU16(_) => DecodedData::SequenceU16(row.get(i)?),
                    FieldType::SequenceU32(_) => DecodedData::SequenceU32(row.get(i)?),
                });
            }

            Ok(data)
        })?;

        let mut data = vec![];
        for row in rows {
            data.push(row?);
        }

        Ok(data)
    }

    /// This function inserts the provided rows of data into a database.
    #[cfg(feature = "integration_sqlite")]
    pub fn count_table(
        pool: &Pool<SqliteConnectionManager>,
        table_name: &str,
        table_version: i32,
        table_unique_id: u64,
    ) -> Result<u64> {
        let query = format!("SELECT COUNT(*) FROM \"{}_v{}\" WHERE table_unique_id = {}",
            table_name.replace('\"', "'"),
            table_version,
            table_unique_id
        );

        let conn = pool.get()?;
        let mut stmt = conn.prepare(&query)?;
        let mut rows = stmt.query([])?;
        let mut count = 0;
        if let Some(row) = rows.next()? {
            count = row.get(0)?;
        }

        Ok(count)
    }
}

impl Table for TableInMemory {
    fn name(&self) -> &str {
        &self.table_name
    }

    fn definition(&self) -> &Definition {
        &self.definition
    }

    fn patches(&self) -> &DefinitionPatch {
        &self.definition_patch
    }

    fn data(&self) -> Cow<[Vec<DecodedData>]> {
        Cow::from(&self.table_data)
    }

    fn data_mut(&mut self) -> &mut Vec<Vec<DecodedData>> {
        &mut self.table_data
    }

    fn set_name(&mut self, val: String) {
        self.table_name = val;
    }

    fn set_definition(&mut self, new_definition: &Definition) {

        // It's simple: we compare both schemas, and get the original and final positions of each column.
        // If a column is new, his original position is -1. If has been removed, his final position is -1.
        let mut positions: Vec<(i32, i32)> = vec![];
        let new_fields_processed = new_definition.fields_processed();
        let old_fields_processed = self.definition.fields_processed();

        for (new_pos, new_field) in new_fields_processed.iter().enumerate() {
            if let Some(old_pos) = old_fields_processed.iter().position(|x| x.name() == new_field.name()) {
                positions.push((old_pos as i32, new_pos as i32))
            } else { positions.push((-1, new_pos as i32)); }
        }

        // Then, for each field in the old definition, check if exists in the new one.
        for (old_pos, old_field) in old_fields_processed.iter().enumerate() {
            if !new_fields_processed.iter().any(|x| x.name() == old_field.name()) { positions.push((old_pos as i32, -1)); }
        }

        // We sort the columns by their destination.
        positions.sort_by_key(|x| x.1);

        // Then, we create the new data using the old one and the column changes.
        let mut new_entries: Vec<Vec<DecodedData>> = Vec::with_capacity(self.table_data.len());
        for row in self.table_data.iter() {
            let mut entry = vec![];
            for (old_pos, new_pos) in &positions {

                // If the new position is -1, it means the column got removed. We skip it.
                if *new_pos == -1 { continue; }

                // If the old position is -1, it means we got a new column. We need to get his type and create a `Default` field with it.
                else if *old_pos == -1 {
                    let field_type = new_fields_processed[*new_pos as usize].field_type();
                    let default_value = new_fields_processed[*new_pos as usize].default_value(Some(&self.definition_patch));
                    entry.push(DecodedData::new_from_type_and_value(field_type, &default_value));
                }

                // Otherwise, we got a moved column. Check here if it needs type conversion.
                else if new_fields_processed[*new_pos as usize].field_type() != old_fields_processed[*old_pos as usize].field_type() {
                    let converted_data = match row[*old_pos as usize].convert_between_types(new_fields_processed[*new_pos as usize].field_type()) {
                        Ok(data) => data,
                        Err(_) => {
                            let field_type = new_fields_processed[*new_pos as usize].field_type();
                            let default_value = new_fields_processed[*new_pos as usize].default_value(Some(&self.definition_patch));
                            DecodedData::new_from_type_and_value(field_type, &default_value)
                        }
                    };
                    entry.push(converted_data);
                }

                // If we reach this, we just got a moved column without any extra change.
                else {
                    entry.push(row[*old_pos as usize].clone());
                }
            }
            new_entries.push(entry);
        }

        self.table_data = new_entries;

        // Then, we finally replace our definition and our data.
        self.definition = new_definition.clone();
    }

    fn set_data(&mut self, data: &[Vec<DecodedData>]) -> Result<()> {
        let fields_processed = self.definition.fields_processed();
        for row in data {

            // First, we need to make sure all rows we have are exactly what we expect.
            if row.len() != fields_processed.len() {
                return Err(RLibError::TableRowWrongFieldCount(fields_processed.len(), row.len()))
            }

            for (index, cell) in row.iter().enumerate() {

                // Next, we need to ensure each file is of the type we expected.
                let field = fields_processed.get(index).unwrap();
                if !cell.is_field_type_correct(field.field_type()) {
                    return Err(RLibError::EncodingTableWrongFieldType(FieldType::from(cell).to_string(), field.field_type().to_string()))
                }
            }
        }

        // If we passed all the checks, replace the data.
        self.table_data = data.to_vec();
        Ok(())
    }

    fn column_position_by_name(&self, column_name: &str) -> Option<usize> {
        self.definition().column_position_by_name(column_name)
    }

    fn is_empty(&self) -> bool {
        self.data().is_empty()
    }

    fn len(&self) -> usize {
        self.data().len()
    }

    fn rows_containing_data(&self, column_name: &str, data: &str) -> Option<(usize, Vec<usize>)> {
        let mut row_indexes = vec![];

        let column_index = self.column_position_by_name(column_name)?;
        for (row_index, row) in self.data().iter().enumerate() {
            if let Some(cell_data) = row.get(column_index) {
                if cell_data.data_to_string() == data {
                    row_indexes.push(row_index);
                }
            }
        }

        if row_indexes.is_empty() {
            None
        } else {
            Some((column_index, row_indexes))
        }
    }
}
