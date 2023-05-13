//---------------------------------------------------------------------------//
// Copyright (c) 2017-2023 Ismael Gutiérrez González. All rights reserved.
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
use crate::files::{bmd::flags::Flags, Decodeable, EncodeableExtraData, Encodeable};

use super::*;

mod v8;

//---------------------------------------------------------------------------//
//                              Enum & Structs
//---------------------------------------------------------------------------//

#[derive(Default, PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct SpotLight {
    serialise_version: u16,
    position: Point3d,
    end: Quaternion,
    length: f32,
    inner_angle: f32,
    outer_angle: f32,
    colour: Colour,
    falloff: f32,
    gobo: String,
    volumetric: bool,
    height_mode: String,
    pdlc_version: u64,
    flags: Flags,
}

//---------------------------------------------------------------------------//
//                      Implementation of SpotLight
//---------------------------------------------------------------------------//

impl Decodeable for SpotLight {

    fn decode<R: ReadBytes>(data: &mut R, extra_data: &Option<DecodeableExtraData>) -> Result<Self> {
        let mut decoded = Self::default();
        decoded.serialise_version = data.read_u16()?;

        match decoded.serialise_version {
            8 => decoded.read_v8(data, extra_data)?,
            _ => return Err(RLibError::DecodingFastBinUnsupportedVersion(String::from("SpotLight"), decoded.serialise_version)),
        }

        Ok(decoded)
    }
}

impl Encodeable for SpotLight {

    fn encode<W: WriteBytes>(&mut self, buffer: &mut W, extra_data: &Option<EncodeableExtraData>) -> Result<()> {
        buffer.write_u16(self.serialise_version)?;

        match self.serialise_version {
            8 => self.write_v8(buffer, extra_data)?,
            _ => return Err(RLibError::EncodingFastBinUnsupportedVersion(String::from("SpotLight"), self.serialise_version)),
        }

        Ok(())
    }
}

