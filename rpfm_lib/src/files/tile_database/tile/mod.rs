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

use super::{*, tile_link::TileLink, tile_link_target::TileLinkTarget, tile_variation::TileVariation};

mod v3;

//---------------------------------------------------------------------------//
//                              Enum & Structs
//---------------------------------------------------------------------------//

#[derive(Default, PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct Tile {
    serialise_version: u16,

    location: String,
    tile_set: String,
    mask: String,
    width: u32,
    height: u32,
    red: f32,
    green: f32,
    blue: f32,
    requires_infield_lodding: bool,
    random_rotatable: bool,
    custom_alpha_blend_texture: String,
    scalable: bool,
    encampable: bool,
    custom_blend_tile: String,

    texture_red: String,
    texture_green: String,
    texture_blue: String,
    texture_alpha: String,

    variations: Vec<TileVariation>,
    link_targets: Vec<TileLinkTarget>,
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
