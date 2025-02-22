//---------------------------------------------------------------------------//
// Copyright (c) 2017-2024 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

use getset::*;
use serde_derive::{Serialize, Deserialize};

use std::collections::BTreeMap;

use crate::error::Result;
use crate::binary::{ReadBytes, WriteBytes};
use crate::files::{DecodeableExtraData, Decodeable, EncodeableExtraData, Encodeable, table::{DecodedData, local::TableInMemory, Table}};
use crate::schema::{Definition, Field, FieldType};
use crate::utils::check_size_mismatch;

pub const EXTENSION: &str = ".atlas";

const IMAGE_SIZE: u32 = 4096;

const VERSION: i32 = 1;

#[cfg(test)] mod atlas_test;

//---------------------------------------------------------------------------//
//                              Enum & Structs
//---------------------------------------------------------------------------//

#[derive(PartialEq, Clone, Debug, Default, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct Atlas {
    version: u32,
    unknown: u32,
    entries: Vec<AtlasEntry>,
}

#[derive(PartialEq, Clone, Debug, Default, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct AtlasEntry {
    string1: String,
    string2: String,
    x_1: f32,
    y_1: f32,
    x_2: f32,
    y_2: f32,
    width: f32,
    height: f32,
}

//---------------------------------------------------------------------------//
//                           Implementation
//---------------------------------------------------------------------------//

impl From<TableInMemory> for Atlas {
    fn from(value: TableInMemory) -> Self {
        let entries = value.data()
            .iter()
            .map(|row| AtlasEntry {
                string1: if let DecodedData::StringU8(data) = &row[0] { data.to_string() } else { panic!("WTF?!")},
                string2: if let DecodedData::StringU8(data) = &row[1] { data.to_string() } else { panic!("WTF?!")},
                x_1: if let DecodedData::F32(data) = row[2] { data } else { panic!("WTF?!")},
                y_1: if let DecodedData::F32(data) = row[3] { data } else { panic!("WTF?!")},
                x_2: if let DecodedData::F32(data) = row[4] { data } else { panic!("WTF?!")},
                y_2: if let DecodedData::F32(data) = row[5] { data } else { panic!("WTF?!")},
                width: if let DecodedData::F32(data) = row[6] { data } else { panic!("WTF?!")},
                height: if let DecodedData::F32(data) = row[7] { data } else { panic!("WTF?!")},
            })
            .collect();

        Self {
            version: VERSION as u32,
            unknown: 0,
            entries,
        }
    }
}

impl From<Atlas> for TableInMemory {
    fn from(value: Atlas) -> Self {
        let mut table = Self::new(&Atlas::definition(), None, "");
        let data = value.entries.iter()
            .map(|entry| {
                 vec![
                    DecodedData::StringU8(entry.string1.to_owned()),
                    DecodedData::StringU8(entry.string2.to_owned()),
                    DecodedData::F32(entry.x_1),
                    DecodedData::F32(entry.y_1),
                    DecodedData::F32(entry.x_2),
                    DecodedData::F32(entry.y_2),
                    DecodedData::F32(entry.width),
                    DecodedData::F32(entry.height),
                ]
            })
            .collect::<Vec<_>>();
        let _ = table.set_data(&data);
        table
    }
}

impl Atlas {

    /// This function returns the definition of a Atlas table.
    pub fn definition() -> Definition {
        let mut definition = Definition::new(VERSION, None);
        let fields = vec![
            Field::new("string1".to_owned(), FieldType::StringU8, true, Some("PLACEHOLDER".to_owned()), false, None, None, None, String::new(), 0, 0, BTreeMap::new(), None),
            Field::new("string2".to_owned(), FieldType::StringU8, true, Some("PLACEHOLDER".to_owned()), false, None, None, None, String::new(), 0, 0, BTreeMap::new(), None),
            Field::new("x_1".to_owned(), FieldType::F32, false, Some("0".to_owned()), false, None, None, None, String::new(), 0, 0, BTreeMap::new(), None),
            Field::new("y_1".to_owned(), FieldType::F32, false, Some("0".to_owned()), false, None, None, None, String::new(), 0, 0, BTreeMap::new(), None),
            Field::new("x_2".to_owned(), FieldType::F32, false, Some("0".to_owned()), false, None, None, None, String::new(), 0, 0, BTreeMap::new(), None),
            Field::new("y_2".to_owned(), FieldType::F32, false, Some("0".to_owned()), false, None, None, None, String::new(), 0, 0, BTreeMap::new(), None),
            Field::new("width".to_owned(), FieldType::F32, false, Some("0".to_owned()), false, None, None, None, String::new(), 0, 0, BTreeMap::new(), None),
            Field::new("height".to_owned(), FieldType::F32, false, Some("0".to_owned()), false, None, None, None, String::new(), 0, 0, BTreeMap::new(), None),
        ];
        definition.set_fields(fields);
        definition
    }
}

impl Decodeable for Atlas {

    fn decode<R: ReadBytes>(data: &mut R, _extra_data: &Option<DecodeableExtraData>) -> Result<Self> {
        let version = data.read_u32()?;
        let unknown = data.read_u32()?;

        let mut entries = vec![];

        for _ in 0..data.read_u32()? {

            // The coordinates are stored in percentage of size.
            entries.push(AtlasEntry {
                string1: data.read_string_u16_0padded(512)?,
                string2: data.read_string_u16_0padded(512)?,
                x_1: data.read_f32()? * IMAGE_SIZE as f32,
                y_1: data.read_f32()? * IMAGE_SIZE as f32,
                x_2: data.read_f32()? * IMAGE_SIZE as f32,
                y_2: data.read_f32()? * IMAGE_SIZE as f32,
                width: data.read_f32()?,
                height: data.read_f32()?,
            })
        }

        // Trigger an error if there's left data on the source.
        check_size_mismatch(data.stream_position()? as usize, data.len()? as usize)?;

        Ok(Self {
            version,
            unknown,
            entries
        })
    }
}

impl Encodeable for Atlas {

    fn encode<W: WriteBytes>(&mut self, buffer: &mut W, _extra_data: &Option<EncodeableExtraData>) -> Result<()> {
        buffer.write_u32(self.version)?;
        buffer.write_u32(self.unknown)?;
        buffer.write_u32(self.entries.len() as u32)?;

        for entry in &self.entries {
            buffer.write_string_u16_0padded(&entry.string1, 512, true)?;
            buffer.write_string_u16_0padded(&entry.string2, 512, true)?;
            buffer.write_f32(entry.x_1 / IMAGE_SIZE as f32)?;
            buffer.write_f32(entry.y_1 / IMAGE_SIZE as f32)?;
            buffer.write_f32(entry.x_2 / IMAGE_SIZE as f32)?;
            buffer.write_f32(entry.y_2 / IMAGE_SIZE as f32)?;
            buffer.write_f32(entry.width)?;
            buffer.write_f32(entry.height)?;
        }

        Ok(())
    }
}

