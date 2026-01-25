//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Tile link target definitions for the tile database.
//!
//! This module contains the [`TileLinkTarget`] struct, which specifies the destination
//! of a tile link. It identifies which tile set and position a link points to.

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

/// Target destination for a tile link.
///
/// Identifies the tile set and coordinates that a [`TileLink`](super::tile_link::TileLink)
/// connects to. This allows the terrain system to resolve link references to actual tiles.
#[derive(Default, PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct TileLinkTarget {
    /// Serialization version for this structure.
    serialise_version: u16,

    /// Name of the tile set containing the target tile.
    tile_set: String,
    /// X coordinate of the target tile within the tile set.
    x: i32,
    /// Y coordinate of the target tile within the tile set.
    y: i32,
}

//---------------------------------------------------------------------------//
//                      Implementation of TileLinkTarget
//---------------------------------------------------------------------------//

impl Decodeable for TileLinkTarget {

    fn decode<R: ReadBytes>(data: &mut R, extra_data: &Option<DecodeableExtraData>) -> Result<Self> {
        let mut decoded = Self::default();
        decoded.serialise_version = data.read_u16()?;

        match decoded.serialise_version {
            1 => decoded.read_v1(data, extra_data)?,
            _ => return Err(RLibError::DecodingFastBinUnsupportedVersion(String::from("TileLinkTarget"), decoded.serialise_version)),
        }

        Ok(decoded)
    }
}

impl Encodeable for TileLinkTarget {

    fn encode<W: WriteBytes>(&mut self, buffer: &mut W, extra_data: &Option<EncodeableExtraData>) -> Result<()> {
        buffer.write_u16(self.serialise_version)?;

        match self.serialise_version {
            1 => self.write_v1(buffer, extra_data)?,
            _ => return Err(RLibError::EncodingFastBinUnsupportedVersion(String::from("TileLinkTarget"), self.serialise_version)),
        }

        Ok(())
    }
}
