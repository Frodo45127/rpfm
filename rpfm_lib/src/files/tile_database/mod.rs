//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! This is a module to read/write tile_database files.

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

/// This holds an entire `TileDatabase` file decoded in memory.
#[derive(Default, PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct TileDatabase {
    serialise_version: u16,

    render_params: RenderParams,
    conversion_params: ConversionParams,
    climates: Vec<Climate>,
    tile_sets: Vec<TileSet>,
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
