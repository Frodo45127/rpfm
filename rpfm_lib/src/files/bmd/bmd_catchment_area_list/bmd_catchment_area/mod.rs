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
use serde::{Serializer};

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
#[serde(rename = "BMD_CATCHMENT_AREA")]
pub struct BmdCatchmentArea {
    #[serde(rename = "@serialise_version")]
    serialise_version: u16,
    #[serde(rename = "@name")]
    name: String,
    #[serde(rename = "aabb", serialize_with = "as_aabb")]
    area: Rectangle,
    #[serde(rename = "@battle_type")]
    battle_type: String,
    #[serde(rename = "@defending_faction_restriction")]
    defending_faction_restriction: String,
    #[serde(rename = "valid_location_flags")]
    valid_location_flags: ValidLocationFlags,
}

//---------------------------------------------------------------------------//
//                Implementation of BmdCatchmentArea
//---------------------------------------------------------------------------//

fn as_aabb<S>(rect: &Rectangle, serializer: S) -> std::result::Result<S::Ok, S::Error>
where
    S: Serializer,
{
    use serde::ser::SerializeStruct;

    let mut state = serializer.serialize_struct("area", 4)?;
    state.serialize_field("@min_x", &format!("{:.6}", rect.min_x()))?;
    state.serialize_field("@min_y", &format!("{:.6}", rect.min_y()))?;
    state.serialize_field("@max_x", &format!("{:.6}", rect.max_x()))?;
    state.serialize_field("@max_y", &format!("{:.6}", rect.max_y()))?;
    state.end()
}

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
