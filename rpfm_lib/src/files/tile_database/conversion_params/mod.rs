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

use crate::binary::{ReadBytes, WriteBytes};
use crate::error::{Result, RLibError};
use crate::files::{Decodeable, EncodeableExtraData, Encodeable};

use super::*;

mod v1;

//---------------------------------------------------------------------------//
//                              Enum & Structs
//---------------------------------------------------------------------------//

#[derive(Default, PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct ConversionParams {
    serialise_version: u16,

    triangle_density: i32,
    max_lf_heights_per_pixel: i32,
    triangle_decimation_size_factors0: f32,
    triangle_decimation_size_factors1: f32,
    triangle_decimation_size_factors2: f32,
    triangle_decimation_size_factors3: f32,
    triangle_decimation_size_factors4: f32,
    triangle_decimation_size_factors5: f32,
    triangle_decimation_angle_factors0: f32,
    triangle_decimation_angle_factors1: f32,
    triangle_decimation_angle_factors2: f32,
    triangle_decimation_angle_factors3: f32,
    triangle_decimation_angle_factors4: f32,
    triangle_decimation_angle_factors5: f32,
}

//---------------------------------------------------------------------------//
//                      Implementation of ConversionParams
//---------------------------------------------------------------------------//

impl Decodeable for ConversionParams {

    fn decode<R: ReadBytes>(data: &mut R, extra_data: &Option<DecodeableExtraData>) -> Result<Self> {
        let mut decoded = Self::default();
        decoded.serialise_version = data.read_u16()?;

        match decoded.serialise_version {
            1 => decoded.read_v1(data, extra_data)?,
            _ => return Err(RLibError::DecodingFastBinUnsupportedVersion(String::from("ConversionParams"), decoded.serialise_version)),
        }

        Ok(decoded)
    }
}

impl Encodeable for ConversionParams {

    fn encode<W: WriteBytes>(&mut self, buffer: &mut W, extra_data: &Option<EncodeableExtraData>) -> Result<()> {
        buffer.write_u16(self.serialise_version)?;

        match self.serialise_version {
            1 => self.write_v1(buffer, extra_data)?,
            _ => return Err(RLibError::EncodingFastBinUnsupportedVersion(String::from("ConversionParams"), self.serialise_version)),
        }

        Ok(())
    }
}
