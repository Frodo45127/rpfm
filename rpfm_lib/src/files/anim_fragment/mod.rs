//---------------------------------------------------------------------------//
// Copyright (c) 2017-2023 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

use getset::{Getters, Setters};
use serde_derive::{Serialize, Deserialize};

use crate::binary::{ReadBytes, WriteBytes};
use crate::error::{RLibError, Result};
use crate::files::{DecodeableExtraData, Decodeable, EncodeableExtraData, Encodeable};
use crate::utils::check_size_mismatch;

pub const BASE_PATH: &str = "animations";

pub const EXTENSIONS: [&str; 2] = [".frg", ".bin"];

mod versions;

#[cfg(test)] mod anim_fragment_test;

//---------------------------------------------------------------------------//
//                              Enum & Structs
//---------------------------------------------------------------------------//

#[derive(PartialEq, Clone, Debug, Default, Getters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", set = "pub")]
pub struct AnimFragment {
    version: u32,
    uk1: u32,
    skeleton_1: String,
    uk2: u32,
    skeleton_2: String,
    locomotion_graph: String,
    uk_string: String,

    entries: Vec<Entry>,
}

#[derive(PartialEq, Clone, Debug, Default, Getters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", set = "pub")]
pub struct Entry {
    uk1: u32,
    uk2: f32,
    uk3: f32,
    uk4: u32,
    uk5: bool,
    anim_data: Vec<AnimData>,
}

#[derive(PartialEq, Clone, Debug, Default, Getters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", set = "pub")]
pub struct AnimData {
    anim: String,
    anim_meta: String,
    anim_snd: String,
}

//---------------------------------------------------------------------------//
//                      Implementation of MatchedCombat
//---------------------------------------------------------------------------//

impl Decodeable for AnimFragment {

    fn decode<R: ReadBytes>(data: &mut R, _extra_data: &Option<DecodeableExtraData>) -> Result<Self> {
        let version = data.read_u32()?;

        let mut fragment = Self::default();
        fragment.version = version;

        match version {
            4 => fragment.read_v4(data)?,
            _ => Err(RLibError::DecodingAnimFragmentUnsupportedVersion(version as usize))?,
        }

        // If we are not in the last byte, it means we didn't parse the entire file, which means this file is corrupt.
        check_size_mismatch(data.stream_position()? as usize, data.len()? as usize)?;

        Ok(fragment)
    }
}

impl Encodeable for AnimFragment {

    fn encode<W: WriteBytes>(&mut self, buffer: &mut W, _extra_data: &Option<EncodeableExtraData>) -> Result<()> {
        buffer.write_u32(self.version)?;

        match self.version {
            4 => self.write_v4(buffer)?,
            _ => Err(RLibError::DecodingAnimFragmentUnsupportedVersion(self.version as usize))?,
        };

        Ok(())
    }
}

