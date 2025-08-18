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

mod v2;

//---------------------------------------------------------------------------//
//                              Enum & Structs
//---------------------------------------------------------------------------//

#[derive(Default, PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct TileVariation {
    serialise_version: u16,

    location: String,
    min_height: f32,
    scale: f32,
    normal_strength: f32,
    overlap_border_size: f32,
    raw_data_tri_density: u32,
    blend_common: String,
    index_common: String,
    normal_common: String,
    red: f32,
    green: f32,
    blue: f32,
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
