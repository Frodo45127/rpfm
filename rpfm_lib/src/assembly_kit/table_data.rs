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
Module with all the code to interact with the Assembly Kit's DB Files.

This module contains all the code needed to parse Assembly Kit's DB files to a format we can understand.
!*/

use rayon::iter::Either;
use rayon::prelude::*;
use regex::Regex;
use serde_derive::Deserialize;
use serde_xml_rs::from_reader;

use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

use rpfm_error::{Result, Error, ErrorKind};

use crate::assembly_kit::table_definition::RawDefinition;
use crate::dependencies::Dependencies;

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
    pub fn read_all(raw_tables_folder: &Path, version: i16, skip_ingame_tables: bool, dependencies: &Dependencies) -> Result<(Vec<Self>, Vec<Error>)> {

        // First, we try to read all `RawDefinitions` from the same folder.
        let (definitions, _) = RawDefinition::read_all(raw_tables_folder, version, skip_ingame_tables, dependencies)?;

        // Then, depending on the version, we have to use one logic or another.
        match version {

            // Version 2 is Rome 2+. Version 1 is Shogun 2. Almost the same format, but we have to
            // provide a different path for Shogun 2, so it has his own version.
            2 | 1 => {
                Ok(definitions.par_iter().partition_map(|definition|
                    match Self::read(definition, raw_tables_folder, version) {
                        Ok(y) => Either::Left(y),
                        Err(y) => Either::Right(y)
                    }
                ))
            }

            // Version 0 is Napoleon and Empire. These two don't have an assembly kit, but CA released years ago their table files.
            // So... these are kinda unique. The schemas are xsd files, and the data format is kinda different and it's not yet supported.
            _ => Err(ErrorKind::AssemblyKitUnsupportedVersion(version).into())
        }
    }

    /// This function tries to parse a Raw Assembly Kit Table to memory.
    pub fn read(raw_definition: &RawDefinition, raw_table_data_folder: &Path, version: i16) -> Result<Self> {
        match version {
            2 | 1 => {
                let name_no_xml = raw_definition.name.as_ref().unwrap().split_at(raw_definition.name.as_ref().unwrap().len() - 4).0;

                // This file is present in Rome 2, Attila and Thrones. It's almost 400mb. And we don't need it.
                if raw_definition.name.as_ref().unwrap() == "translated_texts.xml" { return Err(ErrorKind::AssemblyKitTableTableIgnored.into()) }

                let raw_table_data_path = raw_table_data_folder.join(&raw_definition.name.as_ref().unwrap());
                let mut raw_table_data_file = BufReader::new(File::open(&raw_table_data_path)?);

                // Before deserializing the data, due to limitations of serde_xml_rs, we have to rename all rows, because unique names for
                // rows in each file is not supported for deserializing. Same for the fields, we have to change them to something more generic.
                let mut buffer = String::new();
                raw_table_data_file.read_to_string(&mut buffer)?;
                buffer = buffer.replace(&format!("<{} record_uuid", name_no_xml), "<rows record_uuid");
                buffer = buffer.replace(&format!("<{}>", name_no_xml), "<rows>");
                buffer = buffer.replace(&format!("</{}>", name_no_xml), "</rows>");
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
                    from_reader(buffer.as_bytes()).map_err(Error::from)?
                } else {
                    Self::default()
                };

                raw_table.definition = Some(raw_definition.clone());
                Ok(raw_table)
            }
            _ => Err(ErrorKind::AssemblyKitUnsupportedVersion(version).into())
        }
    }
}
