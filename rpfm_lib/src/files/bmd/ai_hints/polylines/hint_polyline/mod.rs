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

mod v1;
mod v2;
mod v3;
mod v4;

//---------------------------------------------------------------------------//
//                              Enum & Structs
//---------------------------------------------------------------------------//

#[derive(Default, PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct HintPolyline {
    serialise_version: u16,
    rtype: String,
    points: Vec<Point2d>,
    script_id: String,
    only_vanguard: bool,
    only_deploy_when_clear: bool,
    spawn_vfx: bool,
}

//---------------------------------------------------------------------------//
//                   Implementation of HintPolyline
//---------------------------------------------------------------------------//

impl Decodeable for HintPolyline {

    fn decode<R: ReadBytes>(data: &mut R, extra_data: &Option<DecodeableExtraData>) -> Result<Self> {
        let mut decoded = Self::default();
        decoded.serialise_version = data.read_u16()?;

        match decoded.serialise_version {
            1 => decoded.read_v1(data, extra_data)?,
            2 => decoded.read_v2(data, extra_data)?,
            3 => decoded.read_v3(data, extra_data)?,
            4 => decoded.read_v4(data, extra_data)?,
            _ => return Err(RLibError::DecodingFastBinUnsupportedVersion(String::from("HintPolyline (Point)"), decoded.serialise_version)),
        }

        Ok(decoded)
    }
}

impl Encodeable for HintPolyline {

    fn encode<W: WriteBytes>(&mut self, buffer: &mut W, extra_data: &Option<EncodeableExtraData>) -> Result<()> {
        buffer.write_u16(self.serialise_version)?;

        match self.serialise_version {
            1 => self.write_v1(buffer, extra_data)?,
            2 => self.write_v2(buffer, extra_data)?,
            3 => self.write_v3(buffer, extra_data)?,
            4 => self.write_v4(buffer, extra_data)?,
            _ => return Err(RLibError::EncodingFastBinUnsupportedVersion(String::from("HintPolyline (Point)"), self.serialise_version)),
        }

        Ok(())
    }
}
