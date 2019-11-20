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
Module with all the code to interact with any kind of table data.

This module contains the trait `Table`, used to easely decode/encode all the entries of a table.
It also contains his implementation for `Vec<Vec<DecodedData>>`, the type used for that data in DB and LOC Tables.
!*/

use csv::{QuoteStyle, ReaderBuilder, WriterBuilder};

use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Write};
use std::marker::Sized;
use std::path::PathBuf;

use rpfm_error::{Error, ErrorKind, Result};

use super::DecodedData;
use crate::common::{decoder::Decoder, encoder::Encoder};
use crate::schema::*;

pub mod db;
pub mod loc;

//---------------------------------------------------------------------------//
//                              Trait Definition
//---------------------------------------------------------------------------//

/// This trait contains the functions to decode/encode data of a table from/to raw bytes.
pub trait Table {

    /// This function decodes all the fields of a table from raw bytes.
    fn decode(fields: &[Field], data: &[u8], entry_count: u32, index: &mut usize) -> Result<Self> where Self:Sized;

    /// This function encodes all the fields of a table to raw bytes.
    fn encode(&self, fields: &[Field], packed_file: &mut Vec<u8>) -> Result<()>;

    /// This function imports a TSV file into a decoded table.
    fn import_tsv(definition: &Definition, path: &PathBuf, name: &str, version: i32) -> Result<Self> where Self:Sized;

    /// This function imports a TSV file into a new Table File.
    fn import_tsv_to_binary_file(schema: &Schema, source_path: &PathBuf, destination_path: &PathBuf) -> Result<()>;

    /// This function exports the provided data to a TSV file.
    fn export_tsv(&self, path: &PathBuf, headers: &[String], table_name: &str, table_version:i32) -> Result<()>;

    /// This function exports the provided file to a TSV file..
    fn export_tsv_from_binary_file(schema: &Schema, source_path: &PathBuf, destination_path: &PathBuf) -> Result<()>;
}

//---------------------------------------------------------------------------//
//                              Trait Implementation
//---------------------------------------------------------------------------//

/// Implementation of the trait `Table` for `Vec<Vec<DecodedData>>`.
impl Table for Vec<Vec<DecodedData>> {

    fn decode(
        fields: &[Field],
        data: &[u8],
        entry_count: u32,
        mut index: &mut usize,
    ) -> Result<Self> {
        let mut entries = Vec::with_capacity(entry_count as usize);
        for row in 0..entry_count {
            let mut decoded_row = Vec::with_capacity(fields.len());
            for column in 0..fields.len() {
                let decoded_cell = match &fields[column].field_type {
                    FieldType::Boolean => {
                        if let Ok(data) = data.decode_packedfile_bool(*index, &mut index) { DecodedData::Boolean(data) }
                        else { return Err(ErrorKind::HelperDecodingEncodingError(format!("<p>Error trying to decode the <i><b>Row {}, Cell {}</b></i> as a <b><i>Boolean</b></i> value: the value is not a boolean, or there are insufficient bytes left to decode it as a boolean value.</p>", row + 1, column + 1)))? }
                    }
                    FieldType::Float => {
                        if let Ok(data) = data.decode_packedfile_float_f32(*index, &mut index) { DecodedData::Float(data) }
                        else { return Err(ErrorKind::HelperDecodingEncodingError(format!("<p>Error trying to decode the <i><b>Row {}, Cell {}</b></i> as a <b><i>F32</b></i> value: the value is not a valid F32, or there are insufficient bytes left to decode it as a F32 value.</p>", row + 1, column + 1)))? }
                    }
                    FieldType::Integer => {
                        if let Ok(data) = data.decode_packedfile_integer_i32(*index, &mut index) { DecodedData::Integer(data) }
                        else { return Err(ErrorKind::HelperDecodingEncodingError(format!("<p>Error trying to decode the <i><b>Row {}, Cell {}</b></i> as a <b><i>I32</b></i> value: the value is not a valid I32, or there are insufficient bytes left to decode it as an I32 value.</p>", row + 1, column + 1)))? }
                    }
                    FieldType::LongInteger => {
                        if let Ok(data) = data.decode_packedfile_integer_i64(*index, &mut index) { DecodedData::LongInteger(data) }
                        else { return Err(ErrorKind::HelperDecodingEncodingError(format!("<p>Error trying to decode the <i><b>Row {}, Cell {}</b></i> as a <b><i>I64</b></i> value: either the value is not a valid I64, or there are insufficient bytes left to decode it as an I64 value.</p>", row + 1, column + 1)))? }
                    }
                    FieldType::StringU8 => {
                        if let Ok(data) = data.decode_packedfile_string_u8(*index, &mut index) { DecodedData::StringU8(data) }
                        else { return Err(ErrorKind::HelperDecodingEncodingError(format!("<p>Error trying to decode the <i><b>Row {}, Cell {}</b></i> as an <b><i>UTF-8 String</b></i> value: the value is not a valid UTF-8 String, or there are insufficient bytes left to decode it as an UTF-8 String.</p>", row + 1, column + 1)))? }
                    }
                    FieldType::StringU16 => {
                        if let Ok(data) = data.decode_packedfile_string_u16(*index, &mut index) { DecodedData::StringU16(data) }
                        else { return Err(ErrorKind::HelperDecodingEncodingError(format!("<p>Error trying to decode the <i><b>Row {}, Cell {}</b></i> as an <b><i>UTF-16 String</b></i> value: the value is not a valid UTF-16 String, or there are insufficient bytes left to decode it as an UTF-16 String.</p>", row + 1, column + 1)))? }
                    }
                    FieldType::OptionalStringU8 => {
                        if let Ok(data) = data.decode_packedfile_optional_string_u8(*index, &mut index) { DecodedData::OptionalStringU8(data) }
                        else { return Err(ErrorKind::HelperDecodingEncodingError(format!("<p>Error trying to decode the <i><b>Row {}, Cell {}</b></i> as an <b><i>Optional UTF-8 String</b></i> value: the value is not a valid Optional UTF-8 String, or there are insufficient bytes left to decode it as an Optional UTF-8 String.</p>", row + 1, column + 1)))? }
                    }
                    FieldType::OptionalStringU16 => {
                        if let Ok(data) = data.decode_packedfile_optional_string_u16(*index, &mut index) { DecodedData::OptionalStringU16(data) }
                        else { return Err(ErrorKind::HelperDecodingEncodingError(format!("<p>Error trying to decode the <i><b>Row {}, Cell {}</b></i> as an <b><i>Optional UTF-16 String</b></i> value: the value is not a valid Optional UTF-16 String, or there are insufficient bytes left to decode it as an Optional UTF-16 String.</p>", row + 1, column + 1)))? }
                    }

                    // This type is just a recursive type.
                    FieldType::Sequence(fields) => {
                        if let Ok(entry_count) = data.decode_packedfile_integer_u32(*index, &mut index) { DecodedData::Sequence(Self::decode(&*fields, &data, entry_count, index)?) }
                        else { return Err(ErrorKind::HelperDecodingEncodingError(format!("<p>Error trying to get the Entry Count of<i><b>Row {}, Cell {}</b></i>: the value is not a valid U32, or there are insufficient bytes left to decode it as an U32 value.</p>", row + 1, column + 1)))? }
                    }
                };
                decoded_row.push(decoded_cell);
            }
            entries.push(decoded_row);
        }
        Ok(entries)
    }

    fn encode(&self, fields: &[Field], mut packed_file: &mut Vec<u8>) -> Result<()> {
        for row in self {

            // First, we need to make sure all rows we're going to encode are exactly what we expect.
            if row.len() != fields.len() { Err(ErrorKind::TableRowWrongFieldCount(fields.len() as u32, row.len() as u32))? }
            for (index, cell) in row.iter().enumerate() {

                // Next, we need to ensure each file is of the type we expected.
                if !DecodedData::is_field_type_correct(cell, fields[index].field_type.clone()) {
                    Err(ErrorKind::TableWrongFieldType(format!("{}", cell), format!("{}", fields[index].field_type)))?
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
                        packed_file.encode_integer_u32(data.len() as u32);
                        Self::encode(&data, fields, &mut packed_file)?;
                    },
                }
            }
        }

        Ok(())
    }

    //----------------------------------------------------------------//
    // TSV Functions for PackedFiles.
    //----------------------------------------------------------------//

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
                    if record.get(0).unwrap_or("error") != name { return Err(ErrorKind::ImportTSVWrongTypeTable)?; }
                    if record.get(1).unwrap_or("-1").parse::<i32>().map_err(|_| Error::from(ErrorKind::ImportTSVInvalidVersion))? != version {
                        return Err(ErrorKind::ImportTSVWrongVersion)?;
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
                                else { return Err(ErrorKind::ImportTSVIncorrectRow(row, column))?; }
                            }
                            FieldType::Float => entry.push(DecodedData::Float(field.parse::<f32>().map_err(|_| Error::from(ErrorKind::ImportTSVIncorrectRow(row, column)))?)),
                            FieldType::Integer => entry.push(DecodedData::Integer(field.parse::<i32>().map_err(|_| Error::from(ErrorKind::ImportTSVIncorrectRow(row, column)))?)),
                            FieldType::LongInteger => entry.push(DecodedData::LongInteger(field.parse::<i64>().map_err(|_| Error::from(ErrorKind::ImportTSVIncorrectRow(row, column)))?)),
                            FieldType::StringU8 => entry.push(DecodedData::StringU8(field.to_owned())),
                            FieldType::StringU16 => entry.push(DecodedData::StringU16(field.to_owned())),
                            FieldType::OptionalStringU8 => entry.push(DecodedData::OptionalStringU8(field.to_owned())),
                            FieldType::OptionalStringU16 => entry.push(DecodedData::OptionalStringU16(field.to_owned())),

                            // For now fail on Sequences. These are a bit special and I don't know if the're even possible in TSV.
                            FieldType::Sequence(_) => return Err(ErrorKind::ImportTSVIncorrectRow(row, column))?
                        }
                    }
                    entries.push(entry);
                }

                // If it fails here, return an error with the len of the record instead a field.
                else { return Err(ErrorKind::ImportTSVIncorrectRow(row, record.len()))?; }
            }
            else { return Err(ErrorKind::ImportTSVIncorrectRow(row, 0))?; }
        }

        // If we reached this point without errors, we replace the old data with the new one and return success.
        Ok(entries)
    }

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
            table_type = if let Some(table_type) = headers.get(0) { table_type.to_owned() } else { return Err(ErrorKind::ImportTSVWrongTypeTable)? };
            table_version = if let Some(table_version) = headers.get(1) { table_version.parse::<i32>().map_err(|_| Error::from(ErrorKind::ImportTSVInvalidVersion))? } else { return Err(ErrorKind::ImportTSVInvalidVersion)? };
        }

        // Get his definition depending on his first line's contents.
        let definition = if table_type == loc::TSV_NAME { schema.get_versioned_file_loc()?.get_version(table_version)?.clone() }
        else { schema.get_versioned_file_db(&table_type)?.get_version(table_version)?.clone() };

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
                                else { return Err(ErrorKind::ImportTSVIncorrectRow(row, column))?; }
                            }
                            FieldType::Float => entry.push(DecodedData::Float(field.parse::<f32>().map_err(|_| Error::from(ErrorKind::ImportTSVIncorrectRow(row, column)))?)),
                            FieldType::Integer => entry.push(DecodedData::Integer(field.parse::<i32>().map_err(|_| Error::from(ErrorKind::ImportTSVIncorrectRow(row, column)))?)),
                            FieldType::LongInteger => entry.push(DecodedData::LongInteger(field.parse::<i64>().map_err(|_| Error::from(ErrorKind::ImportTSVIncorrectRow(row, column)))?)),
                            FieldType::StringU8 => entry.push(DecodedData::StringU8(field.to_owned())),
                            FieldType::StringU16 => entry.push(DecodedData::StringU16(field.to_owned())),
                            FieldType::OptionalStringU8 => entry.push(DecodedData::OptionalStringU8(field.to_owned())),
                            FieldType::OptionalStringU16 => entry.push(DecodedData::OptionalStringU16(field.to_owned())),
                            FieldType::Sequence(_) => return Err(ErrorKind::ImportTSVIncorrectRow(row, column))?
                        }
                    }
                    entries.push(entry);
                }

                // If it fails here, return an error with the len of the record instead a field.
                else { return Err(ErrorKind::ImportTSVIncorrectRow(row, record.len()))?; }
            }

            else { return Err(ErrorKind::ImportTSVIncorrectRow(row, 0))?; }
        }

        // If we reached this point without errors, we create the File in memory and add the entries to it.
        let data = if table_type == loc::TSV_NAME {
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

    fn export_tsv(
        &self,
        path: &PathBuf,
        headers: &[String],
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
        writer.serialize(headers)?;

        // Then we serialize each entry in the DB Table.
        for entry in self { writer.serialize(&entry)?; }

        // Then, we try to write it on disk. If there is an error, report it.
        let mut file = File::create(&path)?;
        file.write_all(String::from_utf8(writer.into_inner().unwrap())?.as_bytes())?;

        Ok(())
    }

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
            (loc::TSV_NAME, data.get_definition().version, data.get_table_data())
        }
        else {
            let table_type = source_path.parent().unwrap().file_name().unwrap().to_str().unwrap();
            if let Ok(data) = db::DB::read(&data, table_type, schema) { (table_type, data.get_definition().version, data.get_table_data()) }
            else { return Err(ErrorKind::ImportTSVWrongTypeTable)? }
        };

        let definition = if table_type == loc::TSV_NAME { schema.get_versioned_file_loc()?.get_version(version)?.clone() }
        else { schema.get_versioned_file_db(&table_type)?.get_version(version)?.clone() };

        // We serialize the info of the table (name and version) in the first line, and the column names in the second one.
        writer.serialize((&table_type, version))?;
        writer.serialize(definition.fields.iter().map(|x| x.name.to_owned()).collect::<Vec<String>>())?;

        // Then we serialize each entry in the DB Table.
        for entry in entries { writer.serialize(&entry)?; }
        writer.flush().map_err(From::from)
    }
}
