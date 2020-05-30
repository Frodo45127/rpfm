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
Module with the code to support migration operations from Schema V0 onwards.

Schema V0 was the one used by RPFM from 0.X to 1.6.2. Was written in Json, unversioned, and worked only for DB Tables.
This module contains only the code needed for reading/writing Schemas V0 and for migrating them to Schemas V1.

In case it's not clear enough, this is for supporting legacy schemas, not intended to be used externally in ANY other way.
Also, when using this, remember that SchemasV0 where stored in the RPFM's Folder, under `schemas` folder.

The basic structure of an V0 `Schema` is:
```Json
{
  "tables_definitions": [
    {
      "name": "_kv_battle_ai_ability_usage_variables_tables",
      "versions": [
        {
          "version": 0,
          "fields": [
            {
              "field_name": "key",
              "field_type": "StringU8",
              "field_is_key": true,
              "field_is_reference": null,
              "field_description": ""
            },
            {
              "field_name": "value",
              "field_type": "Float",
              "field_is_key": false,
              "field_is_reference": null,
              "field_description": ""
            }
          ]
        }
      ]
    },
  ]
}
```
!*/

use rayon::prelude::*;
use serde_json::to_string_pretty;
use serde_derive::{Serialize, Deserialize};

use std::collections::BTreeMap;
use std::fs::File;
use std::io::{BufReader, Write};
use std::path::PathBuf;

use rpfm_error::Result;

use crate::config::get_config_path;
use crate::SUPPORTED_GAMES;
use crate::schema::Schema;
use crate::schema::SCHEMA_FOLDER;

use super::v1::*;

//---------------------------------------------------------------------------//
//                              Enum & Structs
//---------------------------------------------------------------------------//

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SchemaV0 {
    pub tables_definitions: Vec<TableDefinitionsV0>,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Serialize, Deserialize)]
pub struct TableDefinitionsV0 {
    pub name: String,
    pub versions: Vec<TableDefinitionV0>,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Serialize, Deserialize)]
pub struct TableDefinitionV0 {
    pub version: i32,
    pub fields: Vec<FieldV0>,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Serialize, Deserialize)]
pub struct FieldV0 {
    pub field_name: String,
    pub field_type: FieldTypeV0,
    pub field_is_key: bool,
    pub field_is_reference: Option<(String, String)>,
    pub field_description: String,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Serialize, Deserialize)]
pub enum FieldTypeV0 {
    Boolean,
    Float,
    Integer,
    LongInteger,
    StringU8,
    StringU16,
    OptionalStringU8,
    OptionalStringU16,
}

//---------------------------------------------------------------------------//
//                       Enum & Structs Implementations
//---------------------------------------------------------------------------//

impl SchemaV0 {

    pub fn load(schema_file: &str) -> Result<Self> {
        let mut file_path = PathBuf::from("./schemas/");
        file_path.push(schema_file.replace(".ron", ".json"));
        let file = BufReader::new(File::open(&file_path)?);
        serde_json::from_reader(file).map_err(From::from)
    }

    pub fn save(&mut self, schema_file: &str) -> Result<()> {
        let mut file_path = get_config_path()?.join(SCHEMA_FOLDER);
        file_path.push(schema_file);

        let mut file = File::create(&file_path)?;
        self.tables_definitions.sort();
        file.write_all(to_string_pretty(&self)?.as_bytes())?;
        Ok(())
    }

    pub fn update() {
        println!("Importing schemas from V0 to V1");
        let legacy_schemas = SUPPORTED_GAMES.par_iter().map(|(x, y)| ((*x).to_owned(), Self::load(&y.schema))).filter_map(|(x, y)| if let Ok(y) = y { Some((x, From::from(&y))) } else { None }).collect::<BTreeMap<String, SchemaV1>>();
        let mut schemas = SUPPORTED_GAMES.par_iter().map(|(x, y)| ((*x).to_owned(), SchemaV1::load(&y.schema))).filter_map(|(x, y)| if let Ok(y) = y { Some((x, y)) } else { None }).collect::<BTreeMap<String, SchemaV1>>();
        println!("Amount of Schemas V0: {:?}", legacy_schemas.len());
        println!("Amount of schemas V1: {:?}", schemas.len());
        if !schemas.is_empty() {
            schemas.par_iter_mut().for_each(|(game, schema)| {
                if let Some(legacy_schema) = legacy_schemas.get(game) {
                    legacy_schema.0.iter().for_each(|legacy_versioned_file| schema.add_versioned_file(legacy_versioned_file));
                    if let Some(file_name) = SUPPORTED_GAMES.iter().filter_map(|(x, y)| if x == game { Some(y.schema.to_owned()) } else { None }).find(|_| true) {
                        if schema.save(&file_name).is_ok() {
                            println!("SchemaV0 for game {} updated to SchemaV1.", game);
                        }
                    }
                }
            });
        }
        else {
            println!("No Schema V1 found. Trying updating to Schema V2 directly.");
            let mut legacy_schemas = legacy_schemas.par_iter().map(|(x, y)| ((*x).to_owned(), From::from(y))).collect::<BTreeMap<String, Schema>>();
            println!("Amount of SchemasV2: {:?}", legacy_schemas.len());
            legacy_schemas.par_iter_mut().for_each(|(game, legacy_schema)| {
                if let Some(file_name) = SUPPORTED_GAMES.iter().filter_map(|(x, y)| if x == game { Some(y.schema.to_owned()) } else { None }).find(|_| true) {
                    if legacy_schema.save(&file_name).is_ok() {
                        println!("SchemaV1 for game {} updated to Schema V2.", game);
                    }
                }
            });
        }
    }
}

impl From<&SchemaV0> for SchemaV1 {
    fn from(legacy_schema: &SchemaV0) -> Self {
        let mut schema = Self::default();
        legacy_schema.tables_definitions.iter().map(From::from).for_each(|x| schema.0.push(x));
        schema
    }
}

impl From<&TableDefinitionsV0> for VersionedFileV1 {
    fn from(legacy_table_definitions: &TableDefinitionsV0) -> Self {
        Self::DB(legacy_table_definitions.name.to_owned(), legacy_table_definitions.versions.iter().map(From::from).collect())
    }
}

impl From<&TableDefinitionV0> for DefinitionV1 {
    fn from(legacy_table_definition: &TableDefinitionV0) -> Self {
        let mut definition = Self::new(legacy_table_definition.version);
        legacy_table_definition.fields.iter().map(From::from).for_each(|x| definition.fields.push(x));
        definition
    }
}

impl From<&FieldV0> for FieldV1 {
    fn from(legacy_field: &FieldV0) -> Self {
        let mut field = Self::default();
        field.name = legacy_field.field_name.to_owned();
        field.field_type = From::from(&legacy_field.field_type);
        field.is_key = legacy_field.field_is_key;
        field.is_reference = legacy_field.field_is_reference.clone();
        field.description = legacy_field.field_description.to_owned();
        field
    }
}

impl From<&FieldTypeV0> for FieldTypeV1 {
    fn from(legacy_field_type: &FieldTypeV0) -> Self {
        match legacy_field_type {
            FieldTypeV0::Boolean => Self::Boolean,
            FieldTypeV0::Float => Self::Float,
            FieldTypeV0::Integer => Self::Integer,
            FieldTypeV0::LongInteger => Self::LongInteger,
            FieldTypeV0::StringU8 => Self::StringU8,
            FieldTypeV0::StringU16 => Self::StringU16,
            FieldTypeV0::OptionalStringU8 => Self::OptionalStringU8,
            FieldTypeV0::OptionalStringU16 => Self::OptionalStringU16,
        }
    }
}
