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
Module with all the code to interact with any kind of table data.

This module contains the struct `Table`, used to manage the decoded data of a table. For internal use only.
!*/

use csv::{QuoteStyle, ReaderBuilder, WriterBuilder};
use serde_derive::{Serialize, Deserialize};

use std::{fmt, fmt::Display};
use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::PathBuf;

use rpfm_error::{Error, ErrorKind, Result};

use crate::assembly_kit::table_data::RawTable;
use crate::assembly_kit::table_definition::RawDefinition;
use crate::common::{decoder::Decoder, encoder::Encoder};
use crate::schema::*;

pub mod db;
pub mod loc;

//---------------------------------------------------------------------------//
//                              Enum & Structs
//---------------------------------------------------------------------------//

/// This struct contains the data of a Table-like PackedFile after being decoded.
///
/// This is for internal use. If you need to interact with this in any way, do it through the PackedFile that contains it, not directly.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Table {

    /// A copy of the `Definition` this table uses, so we don't have to check the schema everywhere.
    definition: Definition,

    /// The decoded entries of the table. This list is a Vec(rows) of a Vec(fields of a row) of DecodedData (decoded field).
    entries: Vec<Vec<DecodedData>>,
}

/// This enum is used to store different types of data in a unified way. Used, for example, to store the data from each field in a DB Table.
///
/// NOTE: `Sequence` it's a recursive type. A Sequence/List means you got a repeated sequence of fields
/// inside a single field. Used, for example, in certain model tables.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum DecodedData {
    Boolean(bool),
    Float(f32),
    Integer(i32),
    LongInteger(i64),
    StringU8(String),
    StringU16(String),
    OptionalStringU8(String),
    OptionalStringU16(String),
    Sequence(Table)
}

//----------------------------------------------------------------//
// Implementations for `DecodedData`.
//----------------------------------------------------------------//

/// Display implementation of `DecodedData`.
impl Display for DecodedData {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            DecodedData::Boolean(_) => write!(f, "Boolean"),
            DecodedData::Float(_) => write!(f, "Float"),
            DecodedData::Integer(_) => write!(f, "Integer"),
            DecodedData::LongInteger(_) => write!(f, "LongInteger"),
            DecodedData::StringU8(_) => write!(f, "StringU8"),
            DecodedData::StringU16(_) => write!(f, "StringU16"),
            DecodedData::OptionalStringU8(_) => write!(f, "OptionalStringU8"),
            DecodedData::OptionalStringU16(_) => write!(f, "OptionalStringU16"),
            DecodedData::Sequence(_) => write!(f, "Sequence"),
        }
    }
}

/// PartialEq implementation of `DecodedData`. We need this implementation due to the float comparison being... special.
impl PartialEq for DecodedData {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (DecodedData::Boolean(x), DecodedData::Boolean(y)) => x == y,
            (DecodedData::Float(x), DecodedData::Float(y)) => ((x * 1_000_000f32).round() / 1_000_000f32) == ((y * 1_000_000f32).round() / 1_000_000f32),
            (DecodedData::Integer(x), DecodedData::Integer(y)) => x == y,
            (DecodedData::LongInteger(x), DecodedData::LongInteger(y)) => x == y,
            (DecodedData::StringU8(x), DecodedData::StringU8(y)) => x == y,
            (DecodedData::StringU16(x), DecodedData::StringU16(y)) => x == y,
            (DecodedData::OptionalStringU8(x), DecodedData::OptionalStringU8(y)) => x == y,
            (DecodedData::OptionalStringU16(x), DecodedData::OptionalStringU16(y)) => x == y,
            (DecodedData::Sequence(x), DecodedData::Sequence(y)) => x == y,
            _ => false
        }
    }
}

/// Implementation of `DecodedData`.
impl DecodedData {

    /// Default implementation of `DecodedData`.
    pub fn default(field_type: &FieldType) -> Self {
        match field_type {
            FieldType::Boolean => DecodedData::Boolean(false),
            FieldType::Float => DecodedData::Float(0.0),
            FieldType::Integer => DecodedData::Integer(0),
            FieldType::LongInteger => DecodedData::LongInteger(0),
            FieldType::StringU8 => DecodedData::StringU8("".to_owned()),
            FieldType::StringU16 => DecodedData::StringU16("".to_owned()),
            FieldType::OptionalStringU8 => DecodedData::OptionalStringU8("".to_owned()),
            FieldType::OptionalStringU16 => DecodedData::OptionalStringU16("".to_owned()),
            FieldType::Sequence(definition) => DecodedData::Sequence(Table::new(definition)),
        }
    }

    /// This functions checks if the type of an specific `DecodedData` is the one it should have, according to the provided `FieldType`.
    pub fn is_field_type_correct(&self, field_type: FieldType) -> bool {
        match self {
            DecodedData::Boolean(_) => field_type == FieldType::Boolean,
            DecodedData::Float(_) => field_type == FieldType::Float,
            DecodedData::Integer(_) => field_type == FieldType::Integer,
            DecodedData::LongInteger(_) => field_type == FieldType::LongInteger,
            DecodedData::StringU8(_) => field_type == FieldType::StringU8,
            DecodedData::StringU16(_) => field_type == FieldType::StringU16,
            DecodedData::OptionalStringU8(_) => field_type == FieldType::OptionalStringU8,
            DecodedData::OptionalStringU16(_) => field_type == FieldType::OptionalStringU16,
            DecodedData::Sequence(_) => if let FieldType::Sequence(_) = field_type { true } else { false },
        }
    }
}

//----------------------------------------------------------------//
// Implementations for `Table`.
//----------------------------------------------------------------//

/// Implementation of `Table`.
impl Table {

    /// This function creates a new Table from an existing definition.
    pub fn new(definition: &Definition) -> Self {
        Table {
            definition: definition.clone(),
            entries: vec![],
        }
    }

    /// This function returns a copy of the definition of this Table.
    pub fn get_definition(&self) -> Definition {
        self.definition.clone()
    }

    /// This function returns a reference to the definition of this Table.
    pub fn get_ref_definition(&self) -> &Definition {
        &self.definition
    }

    /// This function returns a copy of the entries of this Table.
    pub fn get_table_data(&self) -> Vec<Vec<DecodedData>> {
        self.entries.to_vec()
    }

    /// This function returns a reference to the entries of this Table.
    pub fn get_ref_table_data(&self) -> &[Vec<DecodedData>] {
        &self.entries
    }

    /// This function returns the amount of entries in this Table.
    pub fn get_entry_count(&self) -> usize {
        self.entries.len()
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
                    entry.push(DecodedData::default(&new_definition.fields[*new_pos as usize].field_type));
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
            if row.len() != self.definition.fields.len() { return Err(ErrorKind::TableRowWrongFieldCount(self.definition.fields.len() as u32, row.len() as u32).into()) }
            for (index, cell) in row.iter().enumerate() {

                // Next, we need to ensure each file is of the type we expected.
                if !DecodedData::is_field_type_correct(cell, self.definition.fields[index].field_type.clone()) {
                    return Err(ErrorKind::TableWrongFieldType(format!("{}", cell), format!("{}", self.definition.fields[index].field_type)).into())
                }
            }
        }

        // If we passed all the checks, replace the data.
        self.entries = data.to_vec();
        Ok(())
    }

    /// This function decodes all the fields of a table from raw bytes.
    fn decode(&mut self,
        data: &[u8],
        entry_count: u32,
        mut index: &mut usize,
    ) -> Result<()> {
        self.entries = Vec::with_capacity(entry_count as usize);
        for row in 0..entry_count {
            let mut decoded_row = Vec::with_capacity(self.definition.fields.len());
            for column in 0..self.definition.fields.len() {
                let decoded_cell = match &self.definition.fields[column].field_type {
                    FieldType::Boolean => {
                        if let Ok(data) = data.decode_packedfile_bool(*index, &mut index) { DecodedData::Boolean(data) }
                        else { return Err(ErrorKind::HelperDecodingEncodingError(format!("<p>Error trying to decode the <i><b>Row {}, Cell {}</b></i> as a <b><i>Boolean</b></i> value: the value is not a boolean, or there are insufficient bytes left to decode it as a boolean value.</p>", row + 1, column + 1)).into()) }
                    }
                    FieldType::Float => {
                        if let Ok(data) = data.decode_packedfile_float_f32(*index, &mut index) { DecodedData::Float(data) }
                        else { return Err(ErrorKind::HelperDecodingEncodingError(format!("<p>Error trying to decode the <i><b>Row {}, Cell {}</b></i> as a <b><i>F32</b></i> value: the value is not a valid F32, or there are insufficient bytes left to decode it as a F32 value.</p>", row + 1, column + 1)).into()) }
                    }
                    FieldType::Integer => {
                        if let Ok(data) = data.decode_packedfile_integer_i32(*index, &mut index) { DecodedData::Integer(data) }
                        else { return Err(ErrorKind::HelperDecodingEncodingError(format!("<p>Error trying to decode the <i><b>Row {}, Cell {}</b></i> as a <b><i>I32</b></i> value: the value is not a valid I32, or there are insufficient bytes left to decode it as an I32 value.</p>", row + 1, column + 1)).into()) }
                    }
                    FieldType::LongInteger => {
                        if let Ok(data) = data.decode_packedfile_integer_i64(*index, &mut index) { DecodedData::LongInteger(data) }
                        else { return Err(ErrorKind::HelperDecodingEncodingError(format!("<p>Error trying to decode the <i><b>Row {}, Cell {}</b></i> as a <b><i>I64</b></i> value: either the value is not a valid I64, or there are insufficient bytes left to decode it as an I64 value.</p>", row + 1, column + 1)).into()) }
                    }
                    FieldType::StringU8 => {
                        if let Ok(data) = data.decode_packedfile_string_u8(*index, &mut index) { DecodedData::StringU8(data) }
                        else { return Err(ErrorKind::HelperDecodingEncodingError(format!("<p>Error trying to decode the <i><b>Row {}, Cell {}</b></i> as an <b><i>UTF-8 String</b></i> value: the value is not a valid UTF-8 String, or there are insufficient bytes left to decode it as an UTF-8 String.</p>", row + 1, column + 1)).into()) }
                    }
                    FieldType::StringU16 => {
                        if let Ok(data) = data.decode_packedfile_string_u16(*index, &mut index) { DecodedData::StringU16(data) }
                        else { return Err(ErrorKind::HelperDecodingEncodingError(format!("<p>Error trying to decode the <i><b>Row {}, Cell {}</b></i> as an <b><i>UTF-16 String</b></i> value: the value is not a valid UTF-16 String, or there are insufficient bytes left to decode it as an UTF-16 String.</p>", row + 1, column + 1)).into()) }
                    }
                    FieldType::OptionalStringU8 => {
                        if let Ok(data) = data.decode_packedfile_optional_string_u8(*index, &mut index) { DecodedData::OptionalStringU8(data) }
                        else { return Err(ErrorKind::HelperDecodingEncodingError(format!("<p>Error trying to decode the <i><b>Row {}, Cell {}</b></i> as an <b><i>Optional UTF-8 String</b></i> value: the value is not a valid Optional UTF-8 String, or there are insufficient bytes left to decode it as an Optional UTF-8 String.</p>", row + 1, column + 1)).into()) }
                    }
                    FieldType::OptionalStringU16 => {
                        if let Ok(data) = data.decode_packedfile_optional_string_u16(*index, &mut index) { DecodedData::OptionalStringU16(data) }
                        else { return Err(ErrorKind::HelperDecodingEncodingError(format!("<p>Error trying to decode the <i><b>Row {}, Cell {}</b></i> as an <b><i>Optional UTF-16 String</b></i> value: the value is not a valid Optional UTF-16 String, or there are insufficient bytes left to decode it as an Optional UTF-16 String.</p>", row + 1, column + 1)).into()) }
                    }

                    // This type is just a recursive type.
                    FieldType::Sequence(definition) => {
                        if let Ok(entry_count) = data.decode_packedfile_integer_u32(*index, &mut index) {
                            let mut sub_table = Table::new(definition);
                            sub_table.decode(&data, entry_count, index)?;
                            DecodedData::Sequence(sub_table) }
                        else { return Err(ErrorKind::HelperDecodingEncodingError(format!("<p>Error trying to get the Entry Count of<i><b>Row {}, Cell {}</b></i>: the value is not a valid U32, or there are insufficient bytes left to decode it as an U32 value.</p>", row + 1, column + 1)).into()) }
                    }
                };
                decoded_row.push(decoded_cell);
            }
            self.entries.push(decoded_row);
        }
        Ok(())
    }

    /// This function encodes all the fields of a table to raw bytes.
    fn encode(&self, mut packed_file: &mut Vec<u8>) -> Result<()> {
        let fields = &self.definition.fields;
        for row in &self.entries {

            // First, we need to make sure all rows we're going to encode are exactly what we expect.
            if row.len() != fields.len() { return Err(ErrorKind::TableRowWrongFieldCount(fields.len() as u32, row.len() as u32).into()) }
            for (index, cell) in row.iter().enumerate() {

                // Next, we need to ensure each file is of the type we expected.
                if !DecodedData::is_field_type_correct(cell, fields[index].field_type.clone()) {
                    return Err(ErrorKind::TableWrongFieldType(format!("{}", cell), format!("{}", fields[index].field_type)).into())
                }

                // If there are no problems, encode the data.
                match *cell {
                    DecodedData::Boolean(data) => packed_file.encode_bool(data),
                    DecodedData::Float(data) => packed_file.encode_float_f32(data),
                    DecodedData::Integer(data) => packed_file.encode_integer_i32(data),
                    DecodedData::LongInteger(data) => packed_file.encode_integer_i64(data),
                    DecodedData::StringU8(ref data) => packed_file.encode_packedfile_string_u8(data),
                    DecodedData::StringU16(ref data) => packed_file.encode_packedfile_string_u16(data),
                    DecodedData::OptionalStringU8(ref data) => packed_file.encode_packedfile_optional_string_u8(data),
                    DecodedData::OptionalStringU16(ref data) => packed_file.encode_packedfile_optional_string_u16(data),
                    DecodedData::Sequence(ref data) => {
                        if let FieldType::Sequence(_) = fields[index].field_type {
                            packed_file.encode_integer_u32(data.entries.len() as u32);
                            data.encode(&mut packed_file)?;
                        }
                    },
                }
            }
        }

        Ok(())
    }

    //----------------------------------------------------------------//
    // TSV Functions for PackedFiles.
    //----------------------------------------------------------------//

    /// This function imports a TSV file into a decoded table.
    fn import_tsv(
        definition: &Definition,
        path: &PathBuf,
        name: &str,
        version: i32,
    ) -> Result<Self> {

        // We want the reader to have no quotes, tab as delimiter and custom headers, because otherwise
        // Excel, Libreoffice and all the programs that edit this kind of files break them on save.
        let mut reader = ReaderBuilder::new()
            .delimiter(b'\t')
            .quoting(false)
            .has_headers(false)
            .flexible(true)
            .from_path(&path)?;

        // If we succesfully load the TSV file into a reader, check the first two lines to ensure
        // it's a valid TSV for our specific table.
        let mut entries = vec![];
        for (row, record) in reader.records().enumerate() {
            if let Ok(record) = record {

                // The first line should contain the "table_folder_name"/"Loc PackedFile/PackFile List", and the version (1 for Locs).
                // If it doesn't match with the name we provided, return an error.
                if row == 0 {
                    if record.get(0).unwrap_or("error") != name { return Err(ErrorKind::ImportTSVWrongTypeTable.into()); }
                    if record.get(1).unwrap_or("-1").parse::<i32>().map_err(|_| Error::from(ErrorKind::ImportTSVInvalidVersion))? != version {
                        return Err(ErrorKind::ImportTSVWrongVersion.into());
                    }
                }

                // The second line contains the column headers. Is just to help people in other programs, so we skip it.
                else if row == 1 { continue }

                // Then read the rest of the rows as a normal TSV.
                else if record.len() == definition.fields.len() {
                    let mut entry = vec![];
                    for (column, field) in record.iter().enumerate() {
                        match definition.fields[column].field_type {
                            FieldType::Boolean => {
                                let value = field.to_lowercase();
                                if value == "true" || value == "1" { entry.push(DecodedData::Boolean(true)); }
                                else if value == "false" || value == "0" { entry.push(DecodedData::Boolean(false)); }
                                else { return Err(ErrorKind::ImportTSVIncorrectRow(row, column).into()); }
                            }
                            FieldType::Float => entry.push(DecodedData::Float(field.parse::<f32>().map_err(|_| Error::from(ErrorKind::ImportTSVIncorrectRow(row, column)))?)),
                            FieldType::Integer => entry.push(DecodedData::Integer(field.parse::<i32>().map_err(|_| Error::from(ErrorKind::ImportTSVIncorrectRow(row, column)))?)),
                            FieldType::LongInteger => entry.push(DecodedData::LongInteger(field.parse::<i64>().map_err(|_| Error::from(ErrorKind::ImportTSVIncorrectRow(row, column)))?)),
                            FieldType::StringU8 => entry.push(DecodedData::StringU8(field.to_owned())),
                            FieldType::StringU16 => entry.push(DecodedData::StringU16(field.to_owned())),
                            FieldType::OptionalStringU8 => entry.push(DecodedData::OptionalStringU8(field.to_owned())),
                            FieldType::OptionalStringU16 => entry.push(DecodedData::OptionalStringU16(field.to_owned())),

                            // For now fail on Sequences. These are a bit special and I don't know if the're even possible in TSV.
                            FieldType::Sequence(_) => return Err(ErrorKind::ImportTSVIncorrectRow(row, column).into())
                        }
                    }
                    entries.push(entry);
                }

                // If it fails here, return an error with the len of the record instead a field.
                else { return Err(ErrorKind::ImportTSVIncorrectRow(row, record.len()).into()); }
            }
            else { return Err(ErrorKind::ImportTSVIncorrectRow(row, 0).into()); }
        }

        // If we reached this point without errors, we replace the old data with the new one and return success.
        let mut table = Table::new(definition);
        table.entries = entries;
        Ok(table)
    }

    /// This function imports a TSV file into a new Table File.
    fn import_tsv_to_binary_file(
        schema: &Schema,
        source_path: &PathBuf,
        destination_path: &PathBuf,
    ) -> Result<()> {

        // We want the reader to have no quotes, tab as delimiter and custom headers, because otherwise
        // Excel, Libreoffice and all the programs that edit this kind of files break them on save.
        let mut reader = ReaderBuilder::new()
            .delimiter(b'\t')
            .quoting(false)
            .has_headers(true)
            .flexible(true)
            .from_path(&source_path)?;

        // If we succesfully load the TSV file into a reader, check the first line to ensure it's a valid TSV file.
        let table_type;
        let table_version;
        {
            let headers = reader.headers()?;
            table_type = if let Some(table_type) = headers.get(0) { table_type.to_owned() } else { return Err(ErrorKind::ImportTSVWrongTypeTable.into()) };
            table_version = if let Some(table_version) = headers.get(1) { table_version.parse::<i32>().map_err(|_| Error::from(ErrorKind::ImportTSVInvalidVersion))? } else { return Err(ErrorKind::ImportTSVInvalidVersion.into()) };
        }

        // Get his definition depending on his first line's contents.
        let definition = if table_type == loc::TSV_NAME_LOC { schema.get_ref_versioned_file_loc()?.get_version(table_version)?.clone() }
        else { schema.get_ref_versioned_file_db(&table_type)?.get_version(table_version)?.clone() };

        // Try to import the entries of the file.
        let mut entries = vec![];
        for (row, record) in reader.records().enumerate() {
            if let Ok(record) = record {

                // The second line contains the column headers. Is just to help people in other programs, not needed to be check.
                if row == 0 { continue }

                // Then read the rest of the rows as a normal TSV.
                else if record.len() == definition.fields.len() {
                    let mut entry = vec![];
                    for (column, field) in record.iter().enumerate() {
                        match definition.fields[column].field_type {
                            FieldType::Boolean => {
                                let value = field.to_lowercase();
                                if value == "true" || value == "1" { entry.push(DecodedData::Boolean(true)); }
                                else if value == "false" || value == "0" { entry.push(DecodedData::Boolean(false)); }
                                else { return Err(ErrorKind::ImportTSVIncorrectRow(row, column).into()); }
                            }
                            FieldType::Float => entry.push(DecodedData::Float(field.parse::<f32>().map_err(|_| Error::from(ErrorKind::ImportTSVIncorrectRow(row, column)))?)),
                            FieldType::Integer => entry.push(DecodedData::Integer(field.parse::<i32>().map_err(|_| Error::from(ErrorKind::ImportTSVIncorrectRow(row, column)))?)),
                            FieldType::LongInteger => entry.push(DecodedData::LongInteger(field.parse::<i64>().map_err(|_| Error::from(ErrorKind::ImportTSVIncorrectRow(row, column)))?)),
                            FieldType::StringU8 => entry.push(DecodedData::StringU8(field.to_owned())),
                            FieldType::StringU16 => entry.push(DecodedData::StringU16(field.to_owned())),
                            FieldType::OptionalStringU8 => entry.push(DecodedData::OptionalStringU8(field.to_owned())),
                            FieldType::OptionalStringU16 => entry.push(DecodedData::OptionalStringU16(field.to_owned())),
                            FieldType::Sequence(_) => return Err(ErrorKind::ImportTSVIncorrectRow(row, column).into())
                        }
                    }
                    entries.push(entry);
                }

                // If it fails here, return an error with the len of the record instead a field.
                else { return Err(ErrorKind::ImportTSVIncorrectRow(row, record.len()).into()); }
            }

            else { return Err(ErrorKind::ImportTSVIncorrectRow(row, 0).into()); }
        }

        // If we reached this point without errors, we create the File in memory and add the entries to it.
        let data = if table_type == loc::TSV_NAME_LOC {
            let mut file = loc::Loc::new(&definition);
            file.set_table_data(&entries)?;
            file.save()
        }
        else {
            let mut file = db::DB::new(&table_type, &definition);
            file.set_table_data(&entries)?;
            file.save()
        }?;

        // Then, we try to write it on disk. If there is an error, report it.
        let mut file = BufWriter::new(File::create(&destination_path)?);
        file.write_all(&data)?;

        // If all worked, return success.
        Ok(())
    }

    /// This function exports the provided data to a TSV file.
    fn export_tsv(
        &self,
        path: &PathBuf,
        table_name: &str,
        table_version: i32
    ) -> Result<()> {

        // We want the writer to have no quotes, tab as delimiter and custom headers, because otherwise
        // Excel, Libreoffice and all the programs that edit this kind of files break them on save.
        let mut writer = WriterBuilder::new()
            .delimiter(b'\t')
            .quote_style(QuoteStyle::Never)
            .has_headers(false)
            .flexible(true)
            .from_writer(vec![]);

        // We serialize the info of the table (name and version) in the first line, and the column names in the second one.
        writer.serialize((table_name, table_version))?;
        writer.serialize(self.definition.fields.iter().map(|x| x.name.to_owned()).collect::<Vec<String>>())?;

        // Then we serialize each entry in the DB Table.
        for entry in &self.entries { writer.serialize(&entry)?; }

        // Then, we try to write it on disk. If there is an error, report it.
        let mut file = File::create(&path)?;
        file.write_all(String::from_utf8(writer.into_inner().unwrap())?.as_bytes())?;

        Ok(())
    }

    /// This function exports the provided file to a TSV file..
    fn export_tsv_from_binary_file(
        schema: &Schema,
        source_path: &PathBuf,
        destination_path: &PathBuf
    ) -> Result<()> {

        // We want the writer to have no quotes, tab as delimiter and custom headers, because otherwise
        // Excel, Libreoffice and all the programs that edit this kind of files break them on save.
        let mut writer = WriterBuilder::new()
            .delimiter(b'\t')
            .quote_style(QuoteStyle::Never)
            .has_headers(false)
            .flexible(true)
            .from_path(destination_path)?;

        // We don't know what type this file is, so we try to decode it as a Loc. If that fails, we try
        // to decode it as a DB using the name of his parent folder. If that fails too, run before it explodes!
        let mut file = BufReader::new(File::open(source_path)?);
        let mut data = vec![];
        file.read_to_end(&mut data)?;

        let (table_type, version, entries) = if let Ok(data) = loc::Loc::read(&data, schema) {
            (loc::TSV_NAME_LOC, data.get_definition().version, data.get_table_data())
        }
        else {
            let table_type = source_path.parent().unwrap().file_name().unwrap().to_str().unwrap();
            if let Ok(data) = db::DB::read(&data, table_type, schema) { (table_type, data.get_definition().version, data.get_table_data()) }
            else { return Err(ErrorKind::ImportTSVWrongTypeTable.into()) }
        };

        let definition = if table_type == loc::TSV_NAME_LOC { schema.get_ref_versioned_file_loc()?.get_version(version)?.clone() }
        else { schema.get_ref_versioned_file_db(&table_type)?.get_version(version)?.clone() };

        // We serialize the info of the table (name and version) in the first line, and the column names in the second one.
        writer.serialize((&table_type, version))?;
        writer.serialize(definition.fields.iter().map(|x| x.name.to_owned()).collect::<Vec<String>>())?;

        // Then we serialize each entry in the DB Table.
        for entry in entries { writer.serialize(&entry)?; }
        writer.flush().map_err(From::from)
    }
}

/// Implementation of `From<&RawTable>` for `Table`.
impl From<&RawTable> for Table {
    fn from(raw_table: &RawTable) -> Self {
        if let Some(ref raw_definition) = raw_table.definition {
            let mut table = Self::new(&From::from(raw_definition));
            for row in &raw_table.rows {
                let mut entry = vec![];

                // Some games (Thrones, Attila, Rome 2 and Shogun 2) may have missing fields when said field is empty.
                // To compensate it, if we don't find a field from the definition in the table, we add it empty.
                for field_def in &table.definition.fields {
                    let mut exists = false;
                    for field in &row.fields {
                        if field_def.name == field.field_name {
                            exists = true;
                            entry.push(match field_def.field_type {
                                FieldType::Boolean => DecodedData::Boolean(field.field_data == "true" || field.field_data == "1"),
                                FieldType::Float => DecodedData::Float(if let Ok(data) = field.field_data.parse::<f32>() { data } else { 0.0 }),
                                FieldType::Integer => DecodedData::Integer(if let Ok(data) = field.field_data.parse::<i32>() { data } else { 0 }),
                                FieldType::LongInteger => DecodedData::LongInteger(if let Ok(data) = field.field_data.parse::<i64>() { data } else { 0 }),
                                FieldType::StringU8 => DecodedData::StringU8(if field.field_data == "Frodo Best Waifu" { String::new() } else { field.field_data.to_string() }),
                                FieldType::StringU16 => DecodedData::StringU16(if field.field_data == "Frodo Best Waifu" { String::new() } else { field.field_data.to_string() }),
                                FieldType::OptionalStringU8 => DecodedData::OptionalStringU8(if field.field_data == "Frodo Best Waifu" { String::new() } else { field.field_data.to_string() }),
                                FieldType::OptionalStringU16 => DecodedData::OptionalStringU16(if field.field_data == "Frodo Best Waifu" { String::new() } else { field.field_data.to_string() }),

                                // This type is not used in the raw tables so, if we find it, we skip it.
                                FieldType::Sequence(_) => continue,
                            });
                            break;
                        }
                    }

                    // If the field doesn't exist, we create it empty.
                    if !exists {
                        entry.push(DecodedData::OptionalStringU8(String::new()));
                    }
                }
                table.entries.push(entry);
            }
            table
        }
        else {
            Self::new(&Definition::new(-1))
        }
    }
}
