//---------------------------------------------------------------------------//
// Copyright (c) 2017-2023 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

/*!
Module with all the code to interact with the Assembly Kit's DB Files and Schemas.

This module contains all the code related with the *schema integration* with the Assembly Kit.
And by *integration* I mean the code that parses Assembly Kit tables and schemas to a format we can actually read.

Also, here is the code responsible for the creation of fake schemas from Assembly Kit files, and for putting them into PAK (Processed Assembly Kit) files.
For more information about PAK files, check the `generate_pak_file()` function. There are multiple types of Assembly Kit table files due to CA changing their format:
- `0`: Empire and Napoleon.
- `1`: Shogun 2.
- `2`: Anything since Rome 2.

Currently, due to the complexity of parsing the table type `0`, we don't have support for PAK files in Empire and Napoleon.
!*/

use rayon::prelude::*;
use serde_derive::Deserialize;
use serde_xml_rs::from_reader;

use std::collections::BTreeMap;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use crate::error::{Result, RLibError};

use super::*;
use super::get_raw_definition_paths;
use super::localisable_fields::RawLocalisableField;
use super::table_data::RawTableRow;

//---------------------------------------------------------------------------//
// Types for parsing the Assembly Kit Schema Files into.
//---------------------------------------------------------------------------//

/// This is the raw equivalent to a `Definition` struct. In files, this is the equivalent to a `TWaD_` file.
///
/// It contains a vector with all the fields that forms it.
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename = "root")]
pub struct RawDefinition {
    pub name: Option<String>,

    #[serde(rename = "field")]
    pub fields: Vec<RawField>,
}

/// This is the raw equivalent to a `Field` struct.
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename = "field")]
pub struct RawField {

    /// Ìf the field is primary key. `1` for `true`, `0` for false.
    pub primary_key: String,

    /// The name of the field.
    pub name: String,

    /// The type of the field in the Assembly Kit.
    pub field_type: String,

    /// If the field is required or can be blank.
    pub required: String,

    /// The default value of the field.
    pub default_value: Option<String>,

    /// The max allowed length for the data in the field.
    pub max_length: Option<String>,

    /// If the field's data corresponds to a filename.
    pub is_filename: Option<String>,

    /// Path where the file in the data of the field can be, if it's restricted to one path.
    pub filename_relative_path: Option<String>,

    /// No idea, but for what I saw, it's not useful for modders.
    pub fragment_path: Option<String>,

    /// Reference source column. First one is the referenced column, the rest, if exists, are the lookup columns concatenated.
    pub column_source_column: Option<Vec<String>>,

    /// Reference source table.
    pub column_source_table: Option<String>,

    /// Description of what the field does.
    pub field_description: Option<String>,

    /// If it has to be exported for the encyclopaedia? No idea really. `1` for `true`, `0` for false.
    pub encyclopaedia_export: Option<String>
}

//---------------------------------------------------------------------------//
// Implementations
//---------------------------------------------------------------------------//

/// Implementation of `RawDefinition`.
impl RawDefinition {

    /// This function reads the provided folder and tries to parse all the Raw Assembly Kit Definitions inside it.
    ///
    /// This function returns two vectors: one with the read files, and another with the errors during parsing.
    pub fn read_all(raw_definitions_folder: &Path, version: i16, tables_to_skip: &[&str]) -> Result<Vec<Self>> {
        let definitions = get_raw_definition_paths(raw_definitions_folder, version)?;
        match version {
            2 | 1 => {
                definitions.par_iter()
                    .filter(|x| !BLACKLISTED_TABLES.contains(&x.file_name().unwrap().to_str().unwrap()))
                    .filter(|x| {
                        let table_name = x.file_stem().unwrap().to_str().unwrap().split_at(5).1;
                        !tables_to_skip.par_iter().any(|vanilla_name| vanilla_name == &table_name)
                    })
                    .map(|x| Self::read(x, version))
                    .collect::<Result<Vec<Self>>>()
            }
            _ => Err(RLibError::AssemblyKitUnsupportedVersion(version))
        }
    }

    /// This function tries to parse a Raw Assembly Kit Definition to memory.
    pub fn read(raw_definition_path: &Path, version: i16) -> Result<Self> {
        match version {
            2 | 1 => {
                let definition_file = BufReader::new(File::open(raw_definition_path).map_err(|_| RLibError::AssemblyKitNotFound)?);
                let mut definition: Self = from_reader(definition_file)?;
                definition.name = Some(raw_definition_path.file_name().unwrap().to_str().unwrap().split_at(5).1.to_string());
                Ok(definition)
            }
            _ => Err(RLibError::AssemblyKitUnsupportedVersion(version))
        }
    }

    /// This function returns the fields without the localisable ones.
    pub fn get_non_localisable_fields(&self, raw_localisable_fields: &[RawLocalisableField], test_row: &RawTableRow) -> Vec<Field> {
        let raw_table_name = &self.name.as_ref().unwrap()[..self.name.as_ref().unwrap().len() - 4];
        let localisable_fields_names = raw_localisable_fields.iter()
            .filter(|x| x.table_name == raw_table_name)
            .map(|x| &*x.field)
            .collect::<Vec<&str>>();

        self.fields.iter()
            .filter(|x| test_row.fields.iter().find(|y| x.name == y.field_name).unwrap().state.is_none())
            .filter(|x| !localisable_fields_names.contains(&&*x.name))
            .map(From::from)
            .collect::<Vec<Field>>()
    }
}

impl From<&RawDefinition> for Definition {
    fn from(raw_definition: &RawDefinition) -> Self {
        let fields = raw_definition.fields.iter().map(From::from).collect::<Vec<_>>();
        Self::new_with_fields(-100, &fields, &[], None)
    }
}


impl From<&RawField> for Field {
    fn from(raw_field: &RawField) -> Self {
        let field_type = match &*raw_field.field_type {
            "yesno" => FieldType::Boolean,
            "single" => FieldType::F32,
            "double" => FieldType::F64,
            "integer" => FieldType::I32,
            "autonumber" | "card64" => FieldType::I64,
            "colour" => FieldType::ColourRGB,
            "expression" | "text" => {
                if raw_field.required == "1" {
                    FieldType::StringU8
                }
                else {
                    FieldType::OptionalStringU8
                }
            },
            _ => FieldType::StringU8,
        };

        let (is_reference, lookup) = if let Some(x) = &raw_field.column_source_table {
            if let Some(y) = &raw_field.column_source_column {
                if y.len() > 1 { (Some((x.to_owned(), y[0].to_owned())), Some(y[1..].to_vec()))}
                else { (Some((x.to_owned(), y[0].to_owned())), None) }
            } else { (None, None) }
        }
        else { (None, None) };

        Self::new(
            raw_field.name.to_owned(),
            field_type,
            raw_field.primary_key == "1",
            raw_field.default_value.clone(),
            raw_field.is_filename.is_some(),
            raw_field.filename_relative_path.clone(),
            is_reference,
            lookup,
            if let Some(x) = &raw_field.field_description { x.to_owned() } else { String::new() },
            0,
            0,
            BTreeMap::new(),
            None
        )
    }
}
