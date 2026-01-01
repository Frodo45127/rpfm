//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! This is a module to read/write anim files.
//!
//! Support is limited because:
//! - We only read the header. Data is kept on binary.

use getset::*;
use serde_derive::{Serialize, Deserialize};

use crate::binary::{ReadBytes, WriteBytes};
use crate::error::Result;
use crate::files::{DecodeableExtraData, Decodeable, EncodeableExtraData, Encodeable};
use crate::utils::check_size_mismatch;

pub const EXTENSION: &str = ".anim";

//mod versions;

#[cfg(test)] mod anim_test;

//---------------------------------------------------------------------------//
//                              Enum & Structs
//---------------------------------------------------------------------------//

#[derive(PartialEq, Clone, Debug, Default, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct Anim {
    version: u32,
    uk_1: u32,
    frame_rate: f32,
    skeleton_name: String,
    end_time: f32,
    bone_count: u32,
    data: Vec<u8>,
}

//---------------------------------------------------------------------------//
//                          Implementation of Anim
//---------------------------------------------------------------------------//

impl Decodeable for Anim {

    fn decode<R: ReadBytes>(data: &mut R, _extra_data: &Option<DecodeableExtraData>) -> Result<Self> {
        let mut anim = Self::default();
        anim.version = data.read_u32()?;
        anim.uk_1 = data.read_u32()?;
        anim.frame_rate = data.read_f32()?;
        anim.skeleton_name = data.read_sized_string_u8()?;
        anim.end_time = data.read_f32()?;
        anim.bone_count = data.read_u32()?;

        let data_left = data.len()?.checked_sub(data.stream_position()?);
        if let Some(data_left) = data_left {
            anim.data = data.read_slice(data_left as usize, false)?;
        }

        // If we are not in the last byte, it means we didn't parse the entire file, which means this file is corrupt.
        check_size_mismatch(data.stream_position()? as usize, data.len()? as usize)?;

        Ok(anim)
    }
}

impl Encodeable for Anim {

    fn encode<W: WriteBytes>(&mut self, buffer: &mut W, _extra_data: &Option<EncodeableExtraData>) -> Result<()> {
        buffer.write_u32(self.version)?;
        buffer.write_u32(self.uk_1)?;
        buffer.write_f32(self.frame_rate)?;
        buffer.write_sized_string_u8(self.skeleton_name())?;
        buffer.write_f32(self.end_time)?;
        buffer.write_u32(self.bone_count)?;
        buffer.write_all(self.data())?;

        Ok(())
    }
}
