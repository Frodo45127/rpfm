//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
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

use self::valid_location_flags::ValidLocationFlags;

use super::*;

mod valid_location_flags;
mod v6;

//---------------------------------------------------------------------------//
//                              Enum & Structs
//---------------------------------------------------------------------------//

#[derive(Default, PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct BmdCatchmentArea {
    serialise_version: u16,
    name: String,
    area: Rectangle,
    battle_type: String,
    defending_faction_restriction: String,
    valid_location_flags: ValidLocationFlags,
}

//---------------------------------------------------------------------------//
//                Implementation of BmdCatchmentArea
//---------------------------------------------------------------------------//

impl Decodeable for BmdCatchmentArea {

    fn decode<R: ReadBytes>(data: &mut R, extra_data: &Option<DecodeableExtraData>) -> Result<Self> {
        let mut decoded = Self::default();
        decoded.serialise_version = data.read_u16()?;

        match decoded.serialise_version {
            6 => decoded.read_v6(data, extra_data)?,
            _ => return Err(RLibError::DecodingFastBinUnsupportedVersion(String::from("BmdCatchmentArea"), decoded.serialise_version)),
        }

        Ok(decoded)
    }
}

impl Encodeable for BmdCatchmentArea {

    fn encode<W: WriteBytes>(&mut self, buffer: &mut W, extra_data: &Option<EncodeableExtraData>) -> Result<()> {
        buffer.write_u16(self.serialise_version)?;

        match self.serialise_version {
            6 => self.write_v6(buffer, extra_data)?,
            _ => return Err(RLibError::EncodingFastBinUnsupportedVersion(String::from("BmdCatchmentArea"), self.serialise_version)),
        }

        Ok(())
    }
}
