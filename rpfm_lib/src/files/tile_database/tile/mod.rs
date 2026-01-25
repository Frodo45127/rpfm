//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Individual terrain tile definitions.
//!
//! A tile is a square or rectangular battle map that gets placed at a specific
//! position in the tile map. When a battle occurs at that position on the campaign
//! map, the corresponding tile is loaded as the battle terrain.

use getset::*;
use serde_derive::{Serialize, Deserialize};

use crate::binary::{ReadBytes, WriteBytes};
use crate::error::{Result, RLibError};
use crate::files::{Decodeable, EncodeableExtraData, Encodeable};

use super::{*, tile_link::TileLink, tile_link_target::TileLinkTarget, tile_variation::TileVariation};

mod v3;

//---------------------------------------------------------------------------//
//                              Enum & Structs
//---------------------------------------------------------------------------//

/// A single terrain tile definition.
///
/// Contains all properties for a terrain tile including dimensions, textures,
/// rendering flags, and connection rules.
#[derive(Default, PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct Tile {
    /// Serialisation format version.
    serialise_version: u16,

    /// File path or location identifier.
    location: String,
    /// Parent tile set name.
    tile_set: String,
    /// Mask texture path.
    mask: String,
    /// Tile width in units.
    width: u32,
    /// Tile height in units.
    height: u32,
    /// Red colour component (0.0-1.0).
    red: f32,
    /// Green colour component (0.0-1.0).
    green: f32,
    /// Blue colour component (0.0-1.0).
    blue: f32,
    /// Whether tile requires in-field LOD processing.
    requires_infield_lodding: bool,
    /// Whether tile can be randomly rotated during placement.
    random_rotatable: bool,
    /// Custom alpha blend texture path.
    custom_alpha_blend_texture: String,
    /// Whether tile can be scaled.
    scalable: bool,
    /// Whether units can encamp on this tile.
    encampable: bool,
    /// Custom blend tile reference.
    custom_blend_tile: String,

    /// Red channel texture path.
    texture_red: String,
    /// Green channel texture path.
    texture_green: String,
    /// Blue channel texture path.
    texture_blue: String,
    /// Alpha channel texture path.
    texture_alpha: String,

    /// Visual variations of this tile.
    variations: Vec<TileVariation>,
    /// Targets for tile linking.
    link_targets: Vec<TileLinkTarget>,
    /// Links to other tiles.
    links: Vec<TileLink>,
}

//---------------------------------------------------------------------------//
//                      Implementation of TileSet
//---------------------------------------------------------------------------//

impl Decodeable for Tile {

    fn decode<R: ReadBytes>(data: &mut R, extra_data: &Option<DecodeableExtraData>) -> Result<Self> {
        let mut decoded = Self::default();
        decoded.serialise_version = data.read_u16()?;

        match decoded.serialise_version {
            3 => decoded.read_v3(data, extra_data)?,
            _ => return Err(RLibError::DecodingFastBinUnsupportedVersion(String::from("Tile"), decoded.serialise_version)),
        }

        Ok(decoded)
    }
}

impl Encodeable for Tile {

    fn encode<W: WriteBytes>(&mut self, buffer: &mut W, extra_data: &Option<EncodeableExtraData>) -> Result<()> {
        buffer.write_u16(self.serialise_version)?;

        match self.serialise_version {
            3 => self.write_v3(buffer, extra_data)?,
            _ => return Err(RLibError::EncodingFastBinUnsupportedVersion(String::from("Tile"), self.serialise_version)),
        }

        Ok(())
    }
}
