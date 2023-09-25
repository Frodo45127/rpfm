//---------------------------------------------------------------------------//
// Copyright (c) 2017-2023 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Sound .dat file.

use getset::*;
use serde_derive::{Serialize, Deserialize};

use crate::binary::{ReadBytes, WriteBytes};
use crate::error::Result;
use crate::files::{Decodeable, EncodeableExtraData, Encodeable};
use crate::utils::*;

use super::DecodeableExtraData;

pub const EXTENSION: &str = ".dat";

//#[cfg(test)] mod test_dat;

//---------------------------------------------------------------------------//
//                              Enum & Structs
//---------------------------------------------------------------------------//

/// This holds an entire `Dat` file decoded in memory.
#[derive(Default, PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct Dat {
    block_0: Vec<(String, f32)>,
    block_1: Vec<(String, Vec<String>)>,
    block_2: Vec<(String, Vec<String>)>,
    block_3: Vec<(String, Vec<String>)>,
    block_4: Vec<String>,
    block_5: Vec<String>,
}

//---------------------------------------------------------------------------//
//                          Implementation of Dat
//---------------------------------------------------------------------------//

impl Decodeable for Dat {

    fn decode<R: ReadBytes>(data: &mut R, _extra_data: &Option<DecodeableExtraData>) -> Result<Self> {
        let mut decoded = Self::default();
        let data_len = data.len()?;

        for _ in 0..data.read_u32()? {
            let string_size = data.read_u32()?;
            let event_name = data.read_string_u8(string_size as usize)?;
            let event_value = data.read_f32()?;
            decoded.block_0.push((event_name, event_value));
        }

        for _ in 0..data.read_u32()? {
            let string_size = data.read_u32()?;
            let event_enum = data.read_string_u8(string_size as usize)?;

            let mut entries = vec![];
            for _ in 0..data.read_u32()? {
                let string_size = data.read_u32()?;
                let event_enum = data.read_string_u8(string_size as usize)?;
                entries.push(event_enum);
            }
            decoded.block_1.push((event_enum, entries));
        }

        for _ in 0..data.read_u32()? {
            let string_size = data.read_u32()?;
            let event_enum = data.read_string_u8(string_size as usize)?;

            let mut entries = vec![];
            for _ in 0..data.read_u32()? {
                let string_size = data.read_u32()?;
                let event_enum = data.read_string_u8(string_size as usize)?;
                entries.push(event_enum);
            }
            decoded.block_2.push((event_enum, entries));
        }

        for _ in 0..data.read_u32()? {
            let string_size = data.read_u32()?;
            let event_enum = data.read_string_u8(string_size as usize)?;

            let mut entries = vec![];
            for _ in 0..data.read_u32()? {
                let string_size = data.read_u32()?;
                let event_enum = data.read_string_u8(string_size as usize)?;
                entries.push(event_enum);
            }
            decoded.block_3.push((event_enum, entries));
        }

        for _ in 0..data.read_u32()? {
            let string_size = data.read_u32()?;
            let event_enum = data.read_string_u8(string_size as usize)?;
            decoded.block_4.push(event_enum);
        }

        for _ in 0..data.read_u32()? {
            let string_size = data.read_u32()?;
            let event_enum = data.read_string_u8(string_size as usize)?;
            decoded.block_5.push(event_enum);
        }

        check_size_mismatch(data.stream_position()? as usize, data_len as usize)?;
        Ok(decoded)
    }
}

impl Encodeable for Dat {

    fn encode<W: WriteBytes>(&mut self, buffer: &mut W, _extra_data: &Option<EncodeableExtraData>) -> Result<()> {
        buffer.write_u32(self.block_0.len() as u32)?;
        for entry in self.block_0() {
            buffer.write_u32(entry.0.len() as u32)?;
            buffer.write_string_u8(&entry.0)?;
            buffer.write_f32(entry.1)?;
        }

        buffer.write_u32(self.block_1.len() as u32)?;
        for entry in self.block_1() {
            buffer.write_u32(entry.0.len() as u32)?;
            buffer.write_string_u8(&entry.0)?;

            buffer.write_u32(entry.1.len() as u32)?;
            for subentry in &entry.1 {
                buffer.write_u32(subentry.len() as u32)?;
                buffer.write_string_u8(subentry)?;
            }
        }

        buffer.write_u32(self.block_2.len() as u32)?;
        for entry in self.block_2() {
            buffer.write_u32(entry.0.len() as u32)?;
            buffer.write_string_u8(&entry.0)?;

            buffer.write_u32(entry.1.len() as u32)?;
            for subentry in &entry.1 {
                buffer.write_u32(subentry.len() as u32)?;
                buffer.write_string_u8(subentry)?;
            }
        }

        buffer.write_u32(self.block_3.len() as u32)?;
        for entry in self.block_3() {
            buffer.write_u32(entry.0.len() as u32)?;
            buffer.write_string_u8(&entry.0)?;

            buffer.write_u32(entry.1.len() as u32)?;
            for subentry in &entry.1 {
                buffer.write_u32(subentry.len() as u32)?;
                buffer.write_string_u8(subentry)?;
            }
        }

        buffer.write_u32(self.block_4.len() as u32)?;
        for entry in self.block_4() {
            buffer.write_u32(entry.len() as u32)?;
            buffer.write_string_u8(entry)?;
        }

        buffer.write_u32(self.block_5.len() as u32)?;
        for entry in self.block_5() {
            buffer.write_u32(entry.len() as u32)?;
            buffer.write_string_u8(entry)?;
        }

        Ok(())
    }
}
