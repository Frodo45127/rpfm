//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

use getset::*;
use serde_derive::{Serialize, Deserialize};

use crate::binary::{ReadBytes, WriteBytes};
use crate::error::{Result, RLibError};
use crate::files::{bmd::common::flags::Flags, Decodeable, EncodeableExtraData, Encodeable};

use super::*;

mod v5;
mod v6;
mod v7;
mod v8;
mod v9;
mod v10;

//---------------------------------------------------------------------------//
//                              Enum & Structs
//---------------------------------------------------------------------------//

#[derive(Default, PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct ParticleEmitter {
    serialise_version: u16,
    key: String,
    transform: Transform3x4,
    emission_rate: f32,
    instance_name: String,
    flags: Flags,
    height_mode: String,
    pdlc_mask: u64,
    autoplay: bool,
    visible_in_shroud: bool,
    parent_id: i32,
    visible_without_shroud: bool,
    uk1: i16
}

//---------------------------------------------------------------------------//
//                   Implementation of ParticleEmitter
//---------------------------------------------------------------------------//

impl Decodeable for ParticleEmitter {

    fn decode<R: ReadBytes>(data: &mut R, extra_data: &Option<DecodeableExtraData>) -> Result<Self> {
        let mut decoded = Self::default();
        decoded.serialise_version = data.read_u16()?;

        match decoded.serialise_version {
            5 => decoded.read_v5(data, extra_data)?,
            6 => decoded.read_v6(data, extra_data)?,
            7 => decoded.read_v7(data, extra_data)?,
            8 => decoded.read_v8(data, extra_data)?,
            9 => decoded.read_v9(data, extra_data)?,
            10 => decoded.read_v10(data, extra_data)?,
            _ => return Err(RLibError::DecodingFastBinUnsupportedVersion(String::from("ParticleEmitter"), decoded.serialise_version)),
        }

        Ok(decoded)
    }
}

impl Encodeable for ParticleEmitter {

    fn encode<W: WriteBytes>(&mut self, buffer: &mut W, extra_data: &Option<EncodeableExtraData>) -> Result<()> {
        buffer.write_u16(self.serialise_version)?;

        match self.serialise_version {
            5 => self.write_v5(buffer, extra_data)?,
            6 => self.write_v6(buffer, extra_data)?,
            7 => self.write_v7(buffer, extra_data)?,
            8 => self.write_v8(buffer, extra_data)?,
            9 => self.write_v9(buffer, extra_data)?,
            10 => self.write_v10(buffer, extra_data)?,
            _ => return Err(RLibError::EncodingFastBinUnsupportedVersion(String::from("ParticleEmitter"), self.serialise_version)),
        }

        Ok(())
    }
}
