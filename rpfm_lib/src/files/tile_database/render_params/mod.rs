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

mod v6;

//---------------------------------------------------------------------------//
//                              Enum & Structs
//---------------------------------------------------------------------------//

#[derive(Default, PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct RenderParams {
    serialise_version: u16,

    lf_height: f32,
    hf_height: f32,
    blend_pixel_scale: f32,
    unit_scale: f32,
    mid_distance_detail_scale: f32,
    mid_distance_detail_strength: f32,
    mid_distance_normal_strength: f32,
    mid_distance_detail_near: f32,
    mid_distance_detail_far: f32,
    mid_distance_detail_slope_low: f32,
    mid_distance_detail_slope_high: f32,
    vertical_offset: f32,
    normal_lf_scale: f32,
    normal_tile_scale: f32,
    normal_terrain_scale: f32,
    blend_contrast: f32,
    layer_exempt_0: String,
    layer_exempt_1: String,
    layer_exempt_2: String,
    layer_exempt_3: String,
    campaign_sea_transparency_scale: f32,
    campaign_sea_uv_scale: f32,
    campaign_lake_transparency_scale: f32,
    near_distance_detail_distance: f32,
    near_distance_detail_scale: f32,
    near_distance_detail_strength: f32,
    mid_distance_detail_near_lq: f32,
}

//---------------------------------------------------------------------------//
//                      Implementation of RenderParams
//---------------------------------------------------------------------------//

impl Decodeable for RenderParams {

    fn decode<R: ReadBytes>(data: &mut R, extra_data: &Option<DecodeableExtraData>) -> Result<Self> {
        let mut decoded = Self::default();
        decoded.serialise_version = data.read_u16()?;

        match decoded.serialise_version {
            6 => decoded.read_v6(data, extra_data)?,
            _ => return Err(RLibError::DecodingFastBinUnsupportedVersion(String::from("RenderParams"), decoded.serialise_version)),
        }

        Ok(decoded)
    }
}

impl Encodeable for RenderParams {

    fn encode<W: WriteBytes>(&mut self, buffer: &mut W, extra_data: &Option<EncodeableExtraData>) -> Result<()> {
        buffer.write_u16(self.serialise_version)?;

        match self.serialise_version {
            6 => self.write_v6(buffer, extra_data)?,
            _ => return Err(RLibError::EncodingFastBinUnsupportedVersion(String::from("RenderParams"), self.serialise_version)),
        }

        Ok(())
    }
}
