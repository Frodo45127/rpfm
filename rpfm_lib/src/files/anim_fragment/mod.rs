//---------------------------------------------------------------------------//
// Copyright (c) 2017-2023 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

use bitflags::bitflags;
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

    // Common stuff.
    version: u32,
    entries: Vec<Entry>,
    skeleton_name: String,
    uk_3: String,

    // Wh3 stuff.
    subversion: u32,
    name: String,
    mount_bin: String,
    uk_string_1: String,
    locomotion_graph: String,
    uk_string_2: String,

    // Wh2 stuff.
    min_id: u32,
    max_id: u32,
}

#[derive(PartialEq, Clone, Debug, Default, Getters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", set = "pub")]
pub struct Entry {

    // Common stuff.
    animation_id: u32,
    blend_in_time: f32,
    selection_weight: f32,
    weapon_bone: WeaponBone,

    // Wh3 stuff
    uk_1: bool,
    anim_refs: Vec<AnimRef>,

    // Wh2 stuff.
    slot_id: u32,
    filename: String,
    metadata: String,
    metadata_sound: String,
    skeleton_type: String,
    uk_3: u32,
    uk_4: String,
    single_frame_variant: bool,
}

#[derive(PartialEq, Clone, Debug, Default, Getters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", set = "pub")]
pub struct AnimRef {
    file_path: String,
    meta_file_path: String,
    snd_file_path: String,
}

bitflags! {

    /// This represents the bitmasks of weapon_bone values.
    #[derive(Default, Getters, Setters, Serialize, Deserialize)]
    pub struct WeaponBone: u32 {
        const WEAPON_BONE_1 = 0b0000_0000_0000_0001;
        const WEAPON_BONE_2 = 0b0000_0000_0000_0010;
        const WEAPON_BONE_3 = 0b0000_0000_0000_0100;
        const WEAPON_BONE_4 = 0b0000_0000_0000_1000;
        const WEAPON_BONE_5 = 0b0000_0000_0001_0000;
        const WEAPON_BONE_6 = 0b0000_0000_0010_0000;
    }
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
            2 => fragment.read_v2(data)?,
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
            2 => self.write_v2(buffer)?,
            4 => self.write_v4(buffer)?,
            _ => Err(RLibError::DecodingAnimFragmentUnsupportedVersion(self.version as usize))?,
        };

        Ok(())
    }
}

