//---------------------------------------------------------------------------//
// Copyright (c) 2017-2024 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

/*!
Module with all the code to interact with the Assembly Kit's DB Files.

This module contains all the code needed to parse Assembly Kit's DB files to a format we can understand.
!*/

use rayon::prelude::*;
use regex::Regex;
use serde_derive::Deserialize;
use serde_xml_rs::from_reader;

use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

use crate::error::{Result, RLibError};
use crate::files::{db::DB, table::{DecodedData, Table}};
use crate::schema::FieldType;

use super::table_definition::RawDefinition;

//---------------------------------------------------------------------------//
// Types for parsing the Assembly Kit DB Files into.
//---------------------------------------------------------------------------//

/// This is the raw equivalent to the `entries` field in a `DB` struct. In files, this is the equivalent to the `.xml` file with all the data in the table.
///
/// It contains a vector with all the rows of data in the `.xml` table file.
#[derive(Debug, Default, Deserialize)]
#[serde(rename = "dataroot")]
pub struct RawTable {
    pub definition: Option<RawDefinition>,

    pub rows: Vec<RawTableRow>,
}

/// This is the raw equivalent to a row of data from a DB file.
#[derive(Debug, Default, Deserialize)]
#[serde(rename = "datarow")]
pub struct RawTableRow {

    #[serde(rename = "datafield")]
    pub fields: Vec<RawTableField>,
}

/// This is the raw equivalent to a `DecodedData`.
#[derive(Debug, Default, Deserialize)]
#[serde(rename = "datafield")]
pub struct RawTableField {
    pub field_name: String,

    #[serde(rename = "$value")]
    pub field_data: String,
    pub state: Option<String>,
}

//---------------------------------------------------------------------------//
// Implementations
//---------------------------------------------------------------------------//

/// Implementation of `RawTable`.
impl RawTable {

    /// This function reads the provided folder and tries to parse all the Raw Assembly Kit Tables inside it.
    pub fn read_all(raw_tables_folder: &Path, version: i16, tables_to_skip: &[&str]) -> Result<Vec<Self>> {

        // First, we try to read all `RawDefinitions` from the same folder.
        let definitions = RawDefinition::read_all(raw_tables_folder, version, tables_to_skip)?;

        // Then, depending on the version, we have to use one logic or another.
        match version {

            // Version 2 is Rome 2+. Version 1 is Shogun 2. Almost the same format, but we have to
            // provide a different path for Shogun 2, so it has his own version.
            // Version 0 is Napoleon and Empire. These two don't have an assembly kit, but CA released years ago their table files.
            0..=2 => Ok(definitions.par_iter().filter_map(|definition| Self::read(definition, raw_tables_folder, version).ok()).collect()),
            _ => Err(RLibError::AssemblyKitUnsupportedVersion(version))
        }
    }

    /// This function tries to parse a Raw Assembly Kit Table to memory.
    pub fn read(raw_definition: &RawDefinition, raw_table_data_folder: &Path, version: i16) -> Result<Self> {
        match version {
            0..=2 => {
                let name_no_xml = raw_definition.name.as_ref().unwrap().split_at(raw_definition.name.as_ref().unwrap().len() - 4).0;

                // This file is present in Rome 2, Attila and Thrones. It's almost 400mb. And we don't need it.
                if raw_definition.name.as_ref().unwrap() == "translated_texts.xml" {
                    return Err(RLibError::AssemblyKitTableTableIgnored)
                }

                let raw_table_data_path = raw_table_data_folder.join(raw_definition.name.as_ref().unwrap());
                let mut raw_table_data_file = BufReader::new(File::open(raw_table_data_path)?);

                // Before deserializing the data, due to limitations of serde_xml_rs, we have to rename all rows, because unique names for
                // rows in each file is not supported for deserializing. Same for the fields, we have to change them to something more generic.
                let mut buffer = String::new();
                raw_table_data_file.read_to_string(&mut buffer)?;
                buffer = buffer.replace(&format!("<{name_no_xml} record_uuid"), "<rows record_uuid");
                buffer = buffer.replace(&format!("<{name_no_xml}>"), "<rows>");
                buffer = buffer.replace(&format!("</{name_no_xml}>"), "</rows>");
                for field in &raw_definition.fields {
                    let field_name_regex = Regex::new(&format!("\n<{}>", field.name)).unwrap();
                    let field_name_regex2 = Regex::new(&format!("\n<{} .+?\">", field.name)).unwrap();
                    buffer = field_name_regex.replace_all(&buffer, &*format!("\n<datafield field_name=\"{}\">", field.name)).to_string();
                    buffer = field_name_regex2.replace_all(&buffer, &*format!("\n<datafield field_name=\"{}\" state=\"1\">", field.name)).to_string();
                    buffer = buffer.replace(&format!("</{}>", field.name), "</datafield>");
                }

                // Serde shits itself if it sees an empty field, so we have to work around that.
                buffer = buffer.replace("\"></datafield>", "\">Frodo Best Waifu</datafield>");
                buffer = buffer.replace("\"> </datafield>", "\"> Frodo Best Waifu</datafield>");
                buffer = buffer.replace("\">  </datafield>", "\">  Frodo Best Waifu</datafield>");
                buffer = buffer.replace("\">   </datafield>", "\">   Frodo Best Waifu</datafield>");
                buffer = buffer.replace("\">    </datafield>", "\">    Frodo Best Waifu</datafield>");

                // Only if the table has data we deserialize it. If not, we just create an empty one.
                let mut raw_table = if buffer.contains("</rows>\r\n</dataroot>") {
                    from_reader(buffer.as_bytes())?
                } else {
                    Self::default()
                };

                raw_table.definition = Some(raw_definition.clone());
                Ok(raw_table)
            }
            _ => Err(RLibError::AssemblyKitUnsupportedVersion(version))
        }
    }
}

impl TryFrom<&RawTable> for DB {
    type Error = RLibError;

    fn try_from(raw_table: &RawTable) -> Result<Self> {
        let table = Table::try_from(raw_table)?;
        Ok(Self::from(table))
    }
}

impl TryFrom<&RawTable> for Table {
    type Error = RLibError;

    fn try_from(raw_table: &RawTable) -> Result<Self> {
        let raw_definition = raw_table.definition.as_ref().ok_or(RLibError::RawTableMissingDefinition)?;
        let table_name = if let Some(ref raw_definition) = raw_definition.name {

            // Remove the .xml of the name in the most awesome way there is.
            let mut x = raw_definition.to_owned();
            x.pop();
            x.pop();
            x.pop();
            x.pop();

            format!("{x}_tables")
        } else { String::new() };

        let mut table = Self::new(&From::from(raw_definition), None, &table_name);
        let mut entries = vec![];
        for row in &raw_table.rows {
            let mut entry = vec![];

            // Some games (Thrones, Attila, Rome 2 and Shogun 2) may have missing fields when said field is empty.
            // To compensate it, if we don't find a field from the definition in the table, we add it empty.
            for field_def in table.definition().fields() {
                let mut exists = false;
                for field in &row.fields {
                    if field_def.name() == field.field_name {
                        exists = true;

                        entry.push(match field_def.field_type() {
                            FieldType::Boolean => DecodedData::Boolean(field.field_data == "true" || field.field_data == "1"),
                            FieldType::F32 => DecodedData::F32(if let Ok(data) = field.field_data.parse::<f32>() { data } else { 0.0 }),
                            FieldType::F64 => DecodedData::F64(if let Ok(data) = field.field_data.parse::<f64>() { data } else { 0.0 }),
                            FieldType::I16 => DecodedData::I16(if let Ok(data) = field.field_data.parse::<i16>() { data } else { 0 }),
                            FieldType::I32 => DecodedData::I32(if let Ok(data) = field.field_data.parse::<i32>() { data } else { 0 }),
                            FieldType::I64 => DecodedData::I64(if let Ok(data) = field.field_data.parse::<i64>() { data } else { 0 }),
                            FieldType::OptionalI16 => DecodedData::OptionalI16(if let Ok(data) = field.field_data.parse::<i16>() { data } else { 0 }),
                            FieldType::OptionalI32 => DecodedData::OptionalI32(if let Ok(data) = field.field_data.parse::<i32>() { data } else { 0 }),
                            FieldType::OptionalI64 => DecodedData::OptionalI64(if let Ok(data) = field.field_data.parse::<i64>() { data } else { 0 }),
                            FieldType::ColourRGB => DecodedData::ColourRGB(field.field_data.to_string()),
                            FieldType::StringU8 => DecodedData::StringU8(if field.field_data == "Frodo Best Waifu" { String::new() } else { field.field_data.to_string() }),
                            FieldType::StringU16 => DecodedData::StringU16(if field.field_data == "Frodo Best Waifu" { String::new() } else { field.field_data.to_string() }),
                            FieldType::OptionalStringU8 => DecodedData::OptionalStringU8(if field.field_data == "Frodo Best Waifu" { String::new() } else { field.field_data.to_string() }),
                            FieldType::OptionalStringU16 => DecodedData::OptionalStringU16(if field.field_data == "Frodo Best Waifu" { String::new() } else { field.field_data.to_string() }),

                            // This type is not used in the raw tables so, if we find it, we skip it.
                            FieldType::SequenceU16(_) | FieldType::SequenceU32(_) => continue,
                        });
                        break;
                    }
                }

                // If the field doesn't exist, we create it empty.
                if !exists {
                    entry.push(match field_def.field_type() {
                        FieldType::Boolean => DecodedData::Boolean(false),
                        FieldType::F32 => DecodedData::F32(0.0),
                        FieldType::F64 => DecodedData::F64(0.0),
                        FieldType::I16 => DecodedData::I16(0),
                        FieldType::I32 => DecodedData::I32(0),
                        FieldType::I64 => DecodedData::I64(0),
                        FieldType::OptionalI16 => DecodedData::OptionalI16(0),
                        FieldType::OptionalI32 => DecodedData::OptionalI32(0),
                        FieldType::OptionalI64 => DecodedData::OptionalI64(0),
                        FieldType::ColourRGB => DecodedData::ColourRGB(String::new()),
                        FieldType::StringU8 => DecodedData::StringU8(String::new()),
                        FieldType::StringU16 => DecodedData::StringU16(String::new()),
                        FieldType::OptionalStringU8 => DecodedData::OptionalStringU8(String::new()),
                        FieldType::OptionalStringU16 => DecodedData::OptionalStringU16(String::new()),

                        // This type is not used in the raw tables so, if we find it, we skip it.
                        FieldType::SequenceU16(_) | FieldType::SequenceU32(_) => unimplemented!("Does this ever happen?"),
                    });
                }
            }
            entries.push(entry);
        }

        table.set_data(&entries)?;
        Ok(table)
    }
}
