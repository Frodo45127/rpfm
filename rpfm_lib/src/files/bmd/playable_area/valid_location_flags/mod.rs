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

//---------------------------------------------------------------------------//
//                              Enum & Structs
//---------------------------------------------------------------------------//

#[derive(Default, PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct ValidLocationFlags {
    #[serde(rename = "@serialise_version")]
    serialise_version: u16,
    #[serde(rename = "@valid_north")]
    valid_north: bool,
    #[serde(rename = "@valid_south")]
    valid_south: bool,
    #[serde(rename = "@valid_east")]
    valid_east: bool,
    #[serde(rename = "@valid_west")]
    valid_west: bool,
}

//---------------------------------------------------------------------------//
//                           Implementation of Text
//---------------------------------------------------------------------------//

impl Decodeable for ValidLocationFlags {

    fn decode<R: ReadBytes>(data: &mut R, extra_data: &Option<DecodeableExtraData>) -> Result<Self> {
        let mut valid_location_flags = Self::default();
        valid_location_flags.serialise_version = data.read_u16()?;

        match valid_location_flags.serialise_version {
            1 => valid_location_flags.read_v1(data, extra_data)?,
            _ => return Err(RLibError::DecodingFastBinUnsupportedVersion(String::from("ValidLocationFlags"), valid_location_flags.serialise_version)),
        }

        Ok(valid_location_flags)
    }
}

impl Encodeable for ValidLocationFlags {

    fn encode<W: WriteBytes>(&mut self, buffer: &mut W, extra_data: &Option<EncodeableExtraData>) -> Result<()> {
        buffer.write_u16(self.serialise_version)?;

        match self.serialise_version {
            1 => self.write_v1(buffer, extra_data)?,
            _ => return Err(RLibError::EncodingFastBinUnsupportedVersion(String::from("ValidLocationFlags"), self.serialise_version)),
        }

        Ok(())
    }
}
