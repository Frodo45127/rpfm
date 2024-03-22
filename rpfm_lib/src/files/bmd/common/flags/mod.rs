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
pub struct Flags {
    serialise_version: u16,
    allow_in_outfield: bool,
    clamp_to_surface: bool,
    clamp_to_water_surface: bool,
    spring: bool,
    summer: bool,
    autumn: bool,
    winter: bool,
    visible_in_tactical_view: bool,
    visible_in_tactical_view_only: bool,
}

//---------------------------------------------------------------------------//
//                           Implementation of Flags
//---------------------------------------------------------------------------//

impl Decodeable for Flags {

    fn decode<R: ReadBytes>(data: &mut R, extra_data: &Option<DecodeableExtraData>) -> Result<Self> {
        let mut flags = Self::default();
        flags.serialise_version = data.read_u16()?;

        match flags.serialise_version {
            1 => flags.read_v1(data, extra_data)?,
            2 => flags.read_v2(data, extra_data)?,
            3 => flags.read_v3(data, extra_data)?,
            4 => flags.read_v4(data, extra_data)?,
            _ => return Err(RLibError::DecodingFastBinUnsupportedVersion(String::from("Flags"), flags.serialise_version)),
        }

        Ok(flags)
    }
}

impl Encodeable for Flags {

    fn encode<W: WriteBytes>(&mut self, buffer: &mut W, extra_data: &Option<EncodeableExtraData>) -> Result<()> {
        buffer.write_u16(self.serialise_version)?;

        match self.serialise_version {
            1 => self.write_v1(buffer, extra_data)?,
            2 => self.write_v2(buffer, extra_data)?,
            3 => self.write_v3(buffer, extra_data)?,
            4 => self.write_v4(buffer, extra_data)?,
            _ => return Err(RLibError::EncodingFastBinUnsupportedVersion(String::from("Flags"), self.serialise_version)),
        }

        Ok(())
    }
}
