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
use crate::files::{bmd::common::flags::Flags, Decodeable, EncodeableExtraData, Encodeable};

use super::*;

mod v2;
mod v3;

//---------------------------------------------------------------------------//
//                              Enum & Structs
//---------------------------------------------------------------------------//

#[derive(Default, PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct TerrainStencilTriangle {
    serialise_version: u16,
    position_0: Point3d,
    position_1: Point3d,
    position_2: Point3d,
    height_mode: String,
    flags: Flags,
}

//---------------------------------------------------------------------------//
//                   Implementation of TerrainStencilTriangle
//---------------------------------------------------------------------------//

impl Decodeable for TerrainStencilTriangle {

    fn decode<R: ReadBytes>(data: &mut R, extra_data: &Option<DecodeableExtraData>) -> Result<Self> {
        let mut decoded = Self::default();
        decoded.serialise_version = data.read_u16()?;

        match decoded.serialise_version {
            2 => decoded.read_v2(data, extra_data)?,
            3 => decoded.read_v3(data, extra_data)?,
            _ => return Err(RLibError::DecodingFastBinUnsupportedVersion(String::from("TerrainStencilTriangle"), decoded.serialise_version)),
        }

        Ok(decoded)
    }
}

impl Encodeable for TerrainStencilTriangle {

    fn encode<W: WriteBytes>(&mut self, buffer: &mut W, extra_data: &Option<EncodeableExtraData>) -> Result<()> {
        buffer.write_u16(self.serialise_version)?;

        match self.serialise_version {
            2 => self.write_v2(buffer, extra_data)?,
            3 => self.write_v3(buffer, extra_data)?,
            _ => return Err(RLibError::EncodingFastBinUnsupportedVersion(String::from("TerrainStencilTriangle"), self.serialise_version)),
        }

        Ok(())
    }
}
