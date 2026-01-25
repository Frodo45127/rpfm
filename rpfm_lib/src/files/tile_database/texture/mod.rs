//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Texture definitions for terrain rendering.
//!
//! Textures define the visual properties of terrain surfaces including
//! reflectivity, smoothness, and blending parameters.

use getset::*;
use serde_derive::{Serialize, Deserialize};

use crate::binary::{ReadBytes, WriteBytes};
use crate::error::{Result, RLibError};
use crate::files::{Decodeable, EncodeableExtraData, Encodeable};

use super::*;

mod v4;

//---------------------------------------------------------------------------//
//                              Enum & Structs
//---------------------------------------------------------------------------//

/// A terrain texture with material properties.
///
/// Defines surface rendering properties including reflectivity and smoothness
/// values at different interpolation points.
#[derive(Default, PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct Texture {
    /// Serialisation format version.
    serialise_version: u16,

    /// Texture identifier name.
    name: String,
    /// Mid-distance detail strength.
    mid_distance_strength: f32,
    /// Reflectivity at lerp point 0.
    lerp_reflectivity0: f32,
    /// Smoothness at lerp point 0.
    lerp_smoothness0: f32,
    /// Reflectivity at lerp point 1.
    lerp_reflectivity1: f32,
    /// Smoothness at lerp point 1.
    lerp_smoothness1: f32,
    /// Reflectivity at lerp point 2.
    lerp_reflectivity2: f32,
    /// Smoothness at lerp point 2.
    lerp_smoothness2: f32,
    /// Reflectivity at lerp point 3.
    lerp_reflectivity3: f32,
    /// Smoothness at lerp point 3.
    lerp_smoothness3: f32,
    /// Pixel scale for texture blending.
    blend_pixel_scale: f32,
    /// Pixel scale for outfield blending.
    outfield_blend_pixel_scale: f32,
}

//---------------------------------------------------------------------------//
//                      Implementation of Texture
//---------------------------------------------------------------------------//

impl Decodeable for Texture {

    fn decode<R: ReadBytes>(data: &mut R, extra_data: &Option<DecodeableExtraData>) -> Result<Self> {
        let mut decoded = Self::default();
        decoded.serialise_version = data.read_u16()?;

        match decoded.serialise_version {
            4 => decoded.read_v4(data, extra_data)?,
            _ => return Err(RLibError::DecodingFastBinUnsupportedVersion(String::from("Texture"), decoded.serialise_version)),
        }

        Ok(decoded)
    }
}

impl Encodeable for Texture {

    fn encode<W: WriteBytes>(&mut self, buffer: &mut W, extra_data: &Option<EncodeableExtraData>) -> Result<()> {
        buffer.write_u16(self.serialise_version)?;

        match self.serialise_version {
            4 => self.write_v4(buffer, extra_data)?,
            _ => return Err(RLibError::EncodingFastBinUnsupportedVersion(String::from("Texture"), self.serialise_version)),
        }

        Ok(())
    }
}
