//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Global rendering parameters for terrain.
//!
//! Contains settings that affect how terrain is rendered including
//! height scales, detail levels, normal mapping, and water properties.

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

/// Global terrain rendering parameters.
///
/// Controls various aspects of terrain rendering including height mapping,
/// detail levels at different distances, normal mapping, and water effects.
#[derive(Default, PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct RenderParams {
    /// Serialisation format version.
    serialise_version: u16,

    /// Low-frequency height scale.
    lf_height: f32,
    /// High-frequency height scale.
    hf_height: f32,
    /// Pixel scale for blending.
    blend_pixel_scale: f32,
    /// World unit scale factor.
    unit_scale: f32,
    /// Mid-distance detail texture scale.
    mid_distance_detail_scale: f32,
    /// Mid-distance detail intensity.
    mid_distance_detail_strength: f32,
    /// Mid-distance normal map intensity.
    mid_distance_normal_strength: f32,
    /// Near threshold for mid-distance detail.
    mid_distance_detail_near: f32,
    /// Far threshold for mid-distance detail.
    mid_distance_detail_far: f32,
    /// Low slope threshold for mid-distance detail.
    mid_distance_detail_slope_low: f32,
    /// High slope threshold for mid-distance detail.
    mid_distance_detail_slope_high: f32,
    /// Vertical offset for terrain placement.
    vertical_offset: f32,
    /// Low-frequency normal scale.
    normal_lf_scale: f32,
    /// Tile-level normal scale.
    normal_tile_scale: f32,
    /// Overall terrain normal scale.
    normal_terrain_scale: f32,
    /// Contrast for blend transitions.
    blend_contrast: f32,
    /// Layer exempt from processing (slot 0).
    layer_exempt_0: String,
    /// Layer exempt from processing (slot 1).
    layer_exempt_1: String,
    /// Layer exempt from processing (slot 2).
    layer_exempt_2: String,
    /// Layer exempt from processing (slot 3).
    layer_exempt_3: String,
    /// Campaign map sea transparency.
    campaign_sea_transparency_scale: f32,
    /// Campaign map sea UV tiling.
    campaign_sea_uv_scale: f32,
    /// Campaign map lake transparency.
    campaign_lake_transparency_scale: f32,
    /// Distance for near-detail rendering.
    near_distance_detail_distance: f32,
    /// Near-distance detail texture scale.
    near_distance_detail_scale: f32,
    /// Near-distance detail intensity.
    near_distance_detail_strength: f32,
    /// Low-quality near threshold for mid-distance.
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
