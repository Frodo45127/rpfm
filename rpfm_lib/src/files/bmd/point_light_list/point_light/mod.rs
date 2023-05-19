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
use crate::files::{bmd::common::flags::Flags, Decodeable, EncodeableExtraData, Encodeable};

use super::*;

mod v5;
mod v6;
mod v7;

//---------------------------------------------------------------------------//
//                              Enum & Structs
//---------------------------------------------------------------------------//

#[derive(Default, PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct PointLight {
    serialise_version: u16,
    position: Point3d,
    radius: f32,
    colour: ColourRGB,
    colour_scale: f32,
    animation_type: u8,
    params: Point2d,
    colour_min: f32,
    random_offset: f32,
    falloff_type: String,
    lf_relative: u8,
    height_mode: String,
    light_probes_only: bool,
    pdlc_mask: u64,
    flags: Flags,
}

//---------------------------------------------------------------------------//
//                Implementation of PointLight
//---------------------------------------------------------------------------//

impl Decodeable for PointLight {

    fn decode<R: ReadBytes>(data: &mut R, extra_data: &Option<DecodeableExtraData>) -> Result<Self> {
        let mut decoded = Self::default();
        decoded.serialise_version = data.read_u16()?;

        match decoded.serialise_version {
            5 => decoded.read_v5(data, extra_data)?,
            6 => decoded.read_v6(data, extra_data)?,
            7 => decoded.read_v7(data, extra_data)?,
            _ => return Err(RLibError::DecodingFastBinUnsupportedVersion(String::from("PointLight"), decoded.serialise_version)),
        }

        Ok(decoded)
    }
}

impl Encodeable for PointLight {

    fn encode<W: WriteBytes>(&mut self, buffer: &mut W, extra_data: &Option<EncodeableExtraData>) -> Result<()> {
        buffer.write_u16(self.serialise_version)?;

        match self.serialise_version {
            5 => self.write_v5(buffer, extra_data)?,
            6 => self.write_v6(buffer, extra_data)?,
            7 => self.write_v7(buffer, extra_data)?,
            _ => return Err(RLibError::EncodingFastBinUnsupportedVersion(String::from("PointLight"), self.serialise_version)),
        }

        Ok(())
    }
}


