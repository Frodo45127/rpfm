//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Tile database files for Total War terrain systems.
//!
//! The tile database (`tile_database.bin`) contains definitions for terrain tiles,
//! including their rendering parameters, textures, climate variations, and linking rules.
//! This data is used by the game's terrain system to generate and render battle maps.
//!
//! # File Format
//!
//! Tile database files use the FASTBIN0 binary format with versioned serialisation.
//! Currently only version 1 is supported.
//!
//! # Structure
//!
//! The database contains several interconnected components:
//! - **Render Parameters**: Global rendering settings for terrain.
//! - **Conversion Parameters**: Parameters for tile format conversion.
//! - **Climates**: Climate-specific tile variations (e.g., temperate, desert).
//! - **Tile Sets**: Groups of related tiles.
//! - **Tiles**: Individual terrain tile definitions with textures and properties.

use getset::*;
use serde_derive::{Serialize, Deserialize};

use crate::binary::{ReadBytes, WriteBytes};
use crate::error::{Result, RLibError};
use crate::files::{Decodeable, EncodeableExtraData, Encodeable};
use crate::utils::check_size_mismatch;

use self::climate::Climate;
use self::conversion_params::ConversionParams;
use self::render_params::RenderParams;
use self::tile_set::TileSet;
use self::tile::Tile;

use super::DecodeableExtraData;

/// Name of the file used by Tile Databases.
pub const NAME: &str = "tile_database.bin";

/// FASTBIN0
pub const SIGNATURE: &[u8; 8] = &[0x46, 0x41, 0x53, 0x54, 0x42, 0x49, 0x4E, 0x30];

#[cfg(test)] mod tile_database_test;

mod climate;
mod conversion_params;
mod render_params;
mod texture;
mod tile_set;
mod tile;
mod tile_link;
mod tile_link_target;
mod tile_variation;

mod v1;

//---------------------------------------------------------------------------//
//                              Enum & Structs
//---------------------------------------------------------------------------//

/// In-memory representation of a decoded tile database file.
///
/// Contains all terrain tile definitions and related configuration data
/// used by the game's terrain system.
///
/// # Fields
///
/// * `serialise_version` - Format version (currently only 1 is supported).
/// * `render_params` - Global rendering parameters for terrain tiles.
/// * `conversion_params` - Parameters for tile format conversion.
/// * `climates` - Climate-specific tile configurations.
/// * `tile_sets` - Groups of related tiles.
/// * `tiles` - Individual tile definitions.
#[derive(Default, PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct TileDatabase {
    /// Serialisation format version.
    serialise_version: u16,

    /// Global rendering parameters.
    render_params: RenderParams,
    /// Tile conversion parameters.
    conversion_params: ConversionParams,
    /// Climate definitions with tile variations.
    climates: Vec<Climate>,
    /// Tile set groupings.
    tile_sets: Vec<TileSet>,
    /// Individual tile definitions.
    tiles: Vec<Tile>,
}


//---------------------------------------------------------------------------//
//                           Implementation of TileDatabase
//---------------------------------------------------------------------------//


impl Decodeable for TileDatabase {

    fn decode<R: ReadBytes>(data: &mut R, extra_data: &Option<DecodeableExtraData>) -> Result<Self> {
        let signature_bytes = data.read_slice(8, false)?;
        if signature_bytes.as_slice() != SIGNATURE {
            return Err(RLibError::DecodingFastBinUnsupportedSignature(signature_bytes));
        }

        let mut fastbin = Self::default();
        fastbin.serialise_version = data.read_u16()?;

        match fastbin.serialise_version {
            1 => fastbin.read_v1(data, extra_data)?,
            _ => return Err(RLibError::DecodingFastBinUnsupportedVersion(String::from("TileDatabase"), fastbin.serialise_version)),
        }

        // If we are not in the last byte, it means we didn't parse the entire file, which means this file is corrupt.
        check_size_mismatch(data.stream_position()? as usize, data.len()? as usize)?;

        Ok(fastbin)
    }
}

impl Encodeable for TileDatabase {

    fn encode<W: WriteBytes>(&mut self, buffer: &mut W, extra_data: &Option<EncodeableExtraData>) -> Result<()> {
        buffer.write_all(SIGNATURE)?;
        buffer.write_u16(self.serialise_version)?;

        match self.serialise_version {
            1 => self.write_v1(buffer, extra_data)?,
            _ => return Err(RLibError::EncodingFastBinUnsupportedVersion(String::from("TileDatabase"), self.serialise_version)),
        }

        Ok(())    }
}
