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

use self::river_node::RiverNode;

use super::*;

mod river_node;
mod v6;
mod v7;
mod v8;
mod v9;
mod v10;

//---------------------------------------------------------------------------//
//                              Enum & Structs
//---------------------------------------------------------------------------//

#[derive(Default, PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct SoundShape {
    serialise_version: u16,
    key: String,
    rtype: String,
    points: Vec<Point3d>,
    inner_radius: f32,
    outer_radius: f32,
    inner_cube: Cube,
    outer_cube: Cube,
    river_nodes: Vec<RiverNode>,
    clamp_to_surface: bool,
    height_mode: String,
    campaign_type_mask: u64,
    pdlc_mask: u64,
    direction: Point3d,
    up: Point3d,
    scope: String,
}

//---------------------------------------------------------------------------//
//                   Implementation of SoundShape
//---------------------------------------------------------------------------//

impl Decodeable for SoundShape {

    fn decode<R: ReadBytes>(data: &mut R, extra_data: &Option<DecodeableExtraData>) -> Result<Self> {
        let mut decoded = Self::default();
        decoded.serialise_version = data.read_u16()?;

        match decoded.serialise_version {
            6 => decoded.read_v6(data, extra_data)?,
            7 => decoded.read_v7(data, extra_data)?,
            8 => decoded.read_v8(data, extra_data)?,
            9 => decoded.read_v9(data, extra_data)?,
            10 => decoded.read_v10(data, extra_data)?,
            _ => return Err(RLibError::DecodingFastBinUnsupportedVersion(String::from("SoundShape"), decoded.serialise_version)),
        }

        Ok(decoded)
    }
}

impl Encodeable for SoundShape {

    fn encode<W: WriteBytes>(&mut self, buffer: &mut W, extra_data: &Option<EncodeableExtraData>) -> Result<()> {
        buffer.write_u16(self.serialise_version)?;

        match self.serialise_version {
            6 => self.write_v6(buffer, extra_data)?,
            7 => self.write_v7(buffer, extra_data)?,
            8 => self.write_v8(buffer, extra_data)?,
            9 => self.write_v9(buffer, extra_data)?,
            10 => self.write_v10(buffer, extra_data)?,
            _ => return Err(RLibError::EncodingFastBinUnsupportedVersion(String::from("SoundShape"), self.serialise_version)),
        }

        Ok(())
    }
}
