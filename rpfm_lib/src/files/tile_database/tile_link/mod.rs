//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Tile link definitions for the tile database.
//!
//! This module contains the [`TileLink`] struct, which defines connections between
//! tiles within the terrain system. Links allow tiles to reference and blend with
//! other tiles, creating seamless transitions across terrain boundaries.

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

/// A link definition connecting tiles within the terrain system.
///
/// Tile links define how tiles connect to and blend with other tiles in link sets.
/// They specify position, blending parameters, and entry points for tile transitions.
#[derive(Default, PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct TileLink {
    /// Serialization version for this structure.
    serialise_version: u16,

    /// Name of the link set this tile link belongs to.
    link_set: String,
    /// X coordinate of this link.
    x: i32,
    /// Y coordinate of this link.
    y: i32,
    /// Base X coordinate for the link origin.
    base_x: i32,
    /// Base Y coordinate for the link origin.
    base_y: i32,
    /// Whether this link serves as an entry point.
    is_entry: bool,
    /// Blend quad indices for texture blending.
    blend_quads: Vec<u32>,
    /// Size of the blend area.
    blend_size: u32,
    /// If true, disables offline blending for this link.
    no_offline_blend: bool,
    /// Test string field (purpose unclear).
    test: String,
}

//---------------------------------------------------------------------------//
//                      Implementation of TileLink
//---------------------------------------------------------------------------//

impl Decodeable for TileLink {

    fn decode<R: ReadBytes>(data: &mut R, extra_data: &Option<DecodeableExtraData>) -> Result<Self> {
        let mut decoded = Self::default();
        decoded.serialise_version = data.read_u16()?;

        match decoded.serialise_version {
            1 => decoded.read_v1(data, extra_data)?,
            _ => return Err(RLibError::DecodingFastBinUnsupportedVersion(String::from("TileLink"), decoded.serialise_version)),
        }

        Ok(decoded)
    }
}

impl Encodeable for TileLink {

    fn encode<W: WriteBytes>(&mut self, buffer: &mut W, extra_data: &Option<EncodeableExtraData>) -> Result<()> {
        buffer.write_u16(self.serialise_version)?;

        match self.serialise_version {
            1 => self.write_v1(buffer, extra_data)?,
            _ => return Err(RLibError::EncodingFastBinUnsupportedVersion(String::from("TileLink"), self.serialise_version)),
        }

        Ok(())
    }
}
