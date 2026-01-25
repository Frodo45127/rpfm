//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Tile variation definitions for the tile database.
//!
//! This module contains the [`TileVariation`] struct, which defines visual and rendering
//! variations for tiles. Variations allow the terrain system to add diversity to terrain
//! appearance through different textures, colors, and rendering parameters.

use getset::*;
use serde_derive::{Serialize, Deserialize};

use crate::binary::{ReadBytes, WriteBytes};
use crate::error::{Result, RLibError};
use crate::files::{Decodeable, EncodeableExtraData, Encodeable};

use super::*;

mod v2;

//---------------------------------------------------------------------------//
//                              Enum & Structs
//---------------------------------------------------------------------------//

/// A visual variation for terrain tiles.
///
/// Tile variations define different appearances for tiles, including height,
/// scale, normal mapping, blending, and color tinting. This allows for more
/// diverse and natural-looking terrain.
#[derive(Default, PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct TileVariation {
    /// Serialization version for this structure.
    serialise_version: u16,

    /// Path or identifier for this variation's location.
    location: String,
    /// Minimum height at which this variation appears.
    min_height: f32,
    /// Scale factor for this variation.
    scale: f32,
    /// Strength of normal mapping for surface detail.
    normal_strength: f32,
    /// Size of the overlap border for blending with adjacent tiles.
    overlap_border_size: f32,
    /// Triangle density for raw terrain data.
    raw_data_tri_density: u32,
    /// Path to the common blend texture.
    blend_common: String,
    /// Path to the common index texture.
    index_common: String,
    /// Path to the common normal map texture.
    normal_common: String,
    /// Red component of the color tint (0.0-1.0).
    red: f32,
    /// Green component of the color tint (0.0-1.0).
    green: f32,
    /// Blue component of the color tint (0.0-1.0).
    blue: f32,
    /// Whether this variation uses the "barbarian" terrain style.
    barbarian: bool,
}

//---------------------------------------------------------------------------//
//                      Implementation of TileVariation
//---------------------------------------------------------------------------//

impl Decodeable for TileVariation {

    fn decode<R: ReadBytes>(data: &mut R, extra_data: &Option<DecodeableExtraData>) -> Result<Self> {
        let mut decoded = Self::default();
        decoded.serialise_version = data.read_u16()?;

        match decoded.serialise_version {
            2 => decoded.read_v2(data, extra_data)?,
            _ => return Err(RLibError::DecodingFastBinUnsupportedVersion(String::from("TileVariation"), decoded.serialise_version)),
        }

        Ok(decoded)
    }
}

impl Encodeable for TileVariation {

    fn encode<W: WriteBytes>(&mut self, buffer: &mut W, extra_data: &Option<EncodeableExtraData>) -> Result<()> {
        buffer.write_u16(self.serialise_version)?;

        match self.serialise_version {
            2 => self.write_v2(buffer, extra_data)?,
            _ => return Err(RLibError::EncodingFastBinUnsupportedVersion(String::from("TileVariation"), self.serialise_version)),
        }

        Ok(())
    }
}
