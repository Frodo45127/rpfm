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
use std::io::{BufReader, Read};
use std::path::Path;

use crate::error::{Result, RLibError};

use super::*;
use super::get_raw_definition_paths;
use super::localisable_fields::RawLocalisableField;
use super::table_data::RawTableRow;

const IGNORABLE_FIELDS: [&str; 4] = ["s_ColLineage", "s_Generation", "s_GUID", "s_Lineage"];

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
    pub encyclopaedia_export: Option<String>,

    /// This one is custom. Is for marking fields of old games (napoleon and shogun 2) to use proper types.
    pub is_old_game: Option<bool>,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename = "xsd_schema")]
pub struct RawDefinitionV0 {
    pub xsd_element: Vec<Element>,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename = "xsd_element")]
pub struct Element {
    pub name: Option<String>,

    #[serde(rename = "od_jetType")]
    pub jet_type: Option<String>,

    #[serde(rename = "minOccurs")]
    pub min_occurs: Option<i32>,

    #[serde(rename = "xsd_annotation")]
    pub xsd_annotation: Option<Annotation>,

    #[serde(rename = "xsd_simpleType")]
    pub xsd_simple_type: Option<Vec<SimpleType>>,

    #[serde(rename = "xsd_complexType")]
    pub xsd_complex_type: Option<Vec<ComplexType>>,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename = "xsd_simpleType")]
pub struct SimpleType {
    pub xsd_restriction: Option<Restriction>,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename = "xsd_complexType")]
pub struct ComplexType {

    #[serde(rename = "xsd_sequence")]
    pub xsd_sequence: Sequence,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename = "xsd_sequence")]
pub struct Sequence {
    pub xsd_element: Vec<Element>,
}


#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename = "xsd_restriction")]
pub struct Restriction {
    pub base: String,

    #[serde(rename = "xsd_maxLength")]
    pub max_lenght: Option<MaxLength>
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename = "xsd_maxLength")]
pub struct MaxLength {
    pub value: i32
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename = "xsd_annotation")]
pub struct Annotation {

    #[serde(rename = "xsd_appinfo")]
    pub xsd_appinfo: Option<AppInfo>
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename = "xsd_appinfo")]
pub struct AppInfo {

    #[serde(rename = "od_index")]
    pub od_index: Option<Vec<Index>>
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename = "od_index")]
pub struct Index {

    #[serde(rename = "index-name")]
    pub name: String,

    #[serde(rename = "index-key")]
    pub key: String,

    #[serde(rename = "primary")]
    pub primary: String,

    #[serde(rename = "unique")]
    pub unique: String,

    #[serde(rename = "clustered")]
    pub clustered: String,
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
                definitions.iter()
                    .filter(|x| !BLACKLISTED_TABLES.contains(&x.file_name().unwrap().to_str().unwrap()))
                    .filter(|x| {
                        let table_name = x.file_stem().unwrap().to_str().unwrap().split_at(5).1;
                        !tables_to_skip.par_iter().any(|vanilla_name| vanilla_name == &table_name)
                    })
                    .map(|x| Self::read(x, version))
                    .collect::<Result<Vec<Self>>>()
            }
            0 => {
                let v0s = definitions.iter()
                    .filter(|x| !BLACKLISTED_TABLES.contains(&x.file_name().unwrap().to_str().unwrap()))
                    .filter(|x| {
                        let table_name = x.file_stem().unwrap().to_str().unwrap();
                        !tables_to_skip.par_iter().any(|vanilla_name| vanilla_name == &table_name)
                    })
                    .filter_map(|x| RawDefinitionV0::read(x).transpose())
                    .collect::<Result<Vec<RawDefinitionV0>>>()?;

                // We need to do a second pass because without the entire set available we cannot figure out the references.
                Ok(v0s.iter()
                    .map(|def_v0| {
                        let mut new_def = Self::from(def_v0);
                        if let Some(elements) = def_v0.xsd_element.get(1) {
                            if let Some(ref table_name) = elements.name {
                                if let Some(ref ann) = elements.xsd_annotation {
                                    if let Some(ref app) = ann.xsd_appinfo {
                                        if let Some(ref od_index) = app.od_index {
                                            for index in od_index {

                                                // Ignore indexes of unused fields, the primary key, and field-specific indexes.
                                                if IGNORABLE_FIELDS.contains(&&*index.name) || index.name == "PrimaryKey" || index.name == index.key.trim() {
                                                    continue;
                                                }

                                                // Indexes follow the format "remotetablelocaltable", with a 61 char limit. To find the remote table,
                                                // we need to remove the local one, and to do so, we need to find what part of the local one is actually in the index name.
                                                let remote_table_name = if index.name.chars().count() == 61 {
                                                    let mut table_name = table_name.clone();
                                                    let mut remote_table_name = String::new();
                                                    loop {
                                                        if index.name.ends_with(&*table_name) {
                                                            remote_table_name = index.name.clone();
                                                            if let Some(sub) = index.name.len().checked_sub(table_name.len()) {
                                                                remote_table_name.truncate(sub);
                                                            } else {
                                                                remote_table_name = String::new();
                                                            }
                                                            break;
                                                        } else {
                                                            if table_name.is_empty() {
                                                                break;
                                                            }

                                                            table_name.pop();
                                                        }
                                                    }

                                                    remote_table_name
                                                } else {
                                                    let mut remote_table_name = index.name.clone();
                                                    if let Some(sub) = index.name.len().checked_sub(table_name.len()) {
                                                        remote_table_name.truncate(sub);
                                                    } else {
                                                        remote_table_name = String::new();
                                                    }
                                                    remote_table_name
                                                };

                                                // Now we need to find the primary key of the remote table, if any.
                                                if !remote_table_name.is_empty() {
                                                    if let Some(remote_def) = v0s.par_iter().find_map_first(|def_v0| {
                                                        if let Some(elements) = def_v0.xsd_element.get(1) {
                                                            if let Some(ref table_name) = elements.name {
                                                                if table_name == &remote_table_name {
                                                                    Some(def_v0)
                                                                } else { None }
                                                            } else { None }
                                                        } else { None }
                                                    }) {

                                                        if let Some(elements) = remote_def.xsd_element.get(1) {
                                                            let primary_keys = if let Some(ref ann) = elements.xsd_annotation {
                                                                if let Some(ref app) = ann.xsd_appinfo {
                                                                    if let Some(ref od_index) = app.od_index {
                                                                        od_index.iter().find_map(|index| {
                                                                            if index.name == "PrimaryKey" {

                                                                                // Always trim to remove the final space, then split by space to find all the keys of the table.
                                                                                let keys = index.key.trim().split(' ').collect::<Vec<_>>();
                                                                                if keys.len() == 1 && IGNORABLE_FIELDS.contains(&keys[0]) {
                                                                                    None
                                                                                } else {
                                                                                    Some(keys)
                                                                                }
                                                                            } else {
                                                                                None
                                                                            }
                                                                        }).unwrap_or(vec![])
                                                                    } else { vec![] }
                                                                } else { vec![] }
                                                            } else { vec![] };

                                                            if !primary_keys.is_empty() {

                                                                // No fucking clue if ANY reference is to a multikey table, but if is, we'll use the first key as ref key, and the rest as lookups.
                                                                for field in &mut new_def.fields {
                                                                    if field.name == index.key.trim() {
                                                                        field.column_source_table = Some(remote_table_name.to_string());
                                                                        field.column_source_column = Some(primary_keys.iter().map(|x| x.to_string()).collect());
                                                                    }
                                                                }
                                                            }

                                                            // Check if our remote table has a "key" column.
                                                            else {

                                                                // No fucking clue if ANY reference is to a multikey table, but if is, we'll use the first key as ref key, and the rest as lookups.
                                                                for field in &mut new_def.fields {
                                                                    if field.name == "key" {
                                                                        field.column_source_table = Some(remote_table_name.to_string());
                                                                        field.column_source_column = Some(vec!["key".to_owned()]);
                                                                    }
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        new_def
                    })
                    .collect())
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

        let is_old_game = raw_field.is_old_game.unwrap_or(false);

        let field_type = match &*raw_field.field_type {
            "yesno" => FieldType::Boolean,
            "single" => FieldType::F32,
            "double" => FieldType::F64,
            "integer" => FieldType::I32,
            "autonumber" | "card64" => FieldType::I64,
            "colour" => FieldType::ColourRGB,
            "expression" | "text" => {
                if raw_field.required == "1" {
                    if is_old_game {
                        FieldType::StringU16
                    } else {
                        FieldType::StringU8
                    }
                }
                else if is_old_game {
                    FieldType::OptionalStringU16
                } else {
                    FieldType::OptionalStringU8
                }
            },
            _ => if is_old_game {
                FieldType::StringU16
            } else {
                FieldType::StringU8
            },
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

impl RawDefinitionV0 {

    /// This function tries to parse a Raw Assembly Kit Definition to memory.
    pub fn read(raw_definition_path: &Path) -> Result<Option<Self>> {
        let mut definition_file = BufReader::new(File::open(raw_definition_path).map_err(|_| RLibError::AssemblyKitNotFound)?);

        // Before deserializing the data, due to limitations of serde_xml_rs, we have to rename all rows, because unique names for
        // rows in each file is not supported for deserializing. Same for the fields, we have to change them to something more generic.
        let mut buffer = String::new();
        definition_file.read_to_string(&mut buffer)?;

        if buffer.is_empty() {
            return Ok(None)
        }

        // Rust doesn't like : in variable names when deserializing.
        buffer = buffer.replace("xsd:schema", "xsd_schema");
        buffer = buffer.replace("xsd:element", "xsd_element");
        buffer = buffer.replace("xsd:complexType", "xsd_complexType");
        buffer = buffer.replace("xsd:sequence", "xsd_sequence");
        buffer = buffer.replace("xsd:attribute", "xsd_attribute");
        buffer = buffer.replace("xsd:annotation", "xsd_annotation");
        buffer = buffer.replace("xsd:appinfo", "xsd_appinfo");
        buffer = buffer.replace("od:index", "od_index");
        buffer = buffer.replace("xsd:sequence", "xsd_sequence");
        buffer = buffer.replace("xsd:simpleType", "xsd_simpleType");
        buffer = buffer.replace("xsd:restriction", "xsd_restriction");
        buffer = buffer.replace("xsd:maxLength", "xsd_maxLength");
        buffer = buffer.replace("od:jetType", "od_jetType");

        buffer = buffer.replace("xs:schema", "xsd_schema");
        buffer = buffer.replace("xs:element", "xsd_element");
        buffer = buffer.replace("xs:complexType", "xsd_complexType");
        buffer = buffer.replace("xs:sequence", "xsd_sequence");
        buffer = buffer.replace("xs:attribute", "xsd_attribute");
        buffer = buffer.replace("xs:annotation", "xsd_annotation");
        buffer = buffer.replace("xs:appinfo", "xsd_appinfo");
        buffer = buffer.replace("xs:sequence", "xsd_sequence");
        buffer = buffer.replace("xs:simpleType", "xsd_simpleType");
        buffer = buffer.replace("xs:restriction", "xsd_restriction");
        buffer = buffer.replace("xs:maxLength", "xsd_maxLength");

        // Only if the table has data we deserialize it. If not, we just create an empty one.
        let definition: RawDefinitionV0 = from_reader(buffer.as_bytes())?;

        //dbg!(&definition);
        Ok(Some(definition))
    }
}

/// Old games don't use references, but rather indexes like a database. This means we're unable to find
/// the referenced column without having the reference definition. So ref data needs to be calculated after this.
impl From<&RawDefinitionV0> for RawDefinition {
    fn from(value: &RawDefinitionV0) -> Self {
        let mut definition = Self::default();

        // Second element has the fields.
        if let Some(elements) = value.xsd_element.get(1) {
            definition.name = elements.name.clone().map(|x| format!("{}.xml", x));

            // Try to get the indexes to check what do we need to mark as key.
            let primary_keys = if let Some(ref ann) = elements.xsd_annotation {
                if let Some(ref app) = ann.xsd_appinfo {
                    if let Some(ref od_index) = app.od_index {
                        od_index.iter().find_map(|index| {
                            if index.name == "PrimaryKey" {

                                // Always trim to remove the final space, then split by space to find all the keys of the table.
                                let keys = index.key.trim().split(' ').collect::<Vec<_>>();
                                if keys.len() == 1 && IGNORABLE_FIELDS.contains(&keys[0]) {
                                    None
                                } else {
                                    Some(keys)
                                }
                            } else {
                                None
                            }
                        }).unwrap_or(vec![])
                    } else { vec![] }
                } else { vec![] }
            } else { vec![] };

            if let Some(complex) = &elements.xsd_complex_type {
                if let Some(elements) = complex.get(0) {
                    for element in &elements.xsd_sequence.xsd_element {

                        // For a field to be valid we need name and type.
                        if let Some(ref name) = element.name {
                            if let Some(ref jet_type) = element.jet_type {

                                // Ignore fields that will not end up in a table.
                                if IGNORABLE_FIELDS.contains(&&**name) {
                                    continue;
                                }

                                let mut field = RawField::default();
                                field.name = name.to_owned();

                                field.field_type = match &**jet_type {
                                    "decimal" | "single" => "single".to_owned(),
                                    "double" => "double".to_owned(),
                                    "yesno" => "yesno".to_owned(),

                                    // No fucking clue, but they're stuff not in the tables.
                                    "oleobject" | "replicationid" => continue,
                                    "integer" => "integer".to_owned(),
                                    "longinteger" | "autonumber" => "autonumber".to_owned(),
                                    "text" => "text".to_owned(),

                                    // These are dates as in a DateTime format. Treat them as text for now.
                                    "datetime" => "text".to_owned(),

                                    // Loc files?
                                    "memo" => continue,
                                    _ => todo!("{}", jet_type),
                                };

                                if primary_keys.contains(&&*field.name) {
                                    field.primary_key = "1".to_owned();
                                }

                                field.is_old_game = Some(true);

                                definition.fields.push(field);
                            }
                        }
                    }
                }
            }
        }

        definition
    }
}
