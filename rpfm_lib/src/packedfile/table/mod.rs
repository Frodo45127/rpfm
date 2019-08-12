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

use rpfm_error::{ErrorKind, Result};

use super::DecodedData;
use crate::common::{decoder::Decoder, encoder::Encoder};
use crate::schema::*;

use std::marker::Sized;

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
}

//---------------------------------------------------------------------------//
//                              Trait Implementation
//---------------------------------------------------------------------------//

/// Implementation of the trait `Table` for `Vec<Vec<DecodedData>>`.
impl Table for Vec<Vec<DecodedData>> {

    /// This function gets the fields data for the provided definition.
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

    /// This function returns the encoded version of the provided entry list.
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
}