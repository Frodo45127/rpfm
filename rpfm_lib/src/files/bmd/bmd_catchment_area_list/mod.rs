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
use serde::{Serializer, Deserializer};
// use crate::compression::_::_serde::Serialize;
// use crate::compression::_::_serde::Deserialize;

use crate::binary::{ReadBytes, WriteBytes};
use crate::error::{Result, RLibError};
use crate::files::{Decodeable, EncodeableExtraData, Encodeable};

use self::bmd_catchment_area::BmdCatchmentArea;
use super::*;

mod bmd_catchment_area;
mod v1;

//---------------------------------------------------------------------------//
//                              Enum & Structs
//---------------------------------------------------------------------------//

#[derive(Default, PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct BmdCatchmentAreaList {
    #[serde(rename = "@serialise_version")]
    serialise_version: u16,
    #[serde(
        rename = "BMD_CATCHMENT_AREAS",
        serialize_with = "serialize_nested_areas",
        deserialize_with = "deserialize_nested_areas"
    )] // custom serializer and deserializer to implement nesting the BMD_CATCHMENT_AREA tag.
    bmd_catchment_areas: Vec<BmdCatchmentArea>,
}

//---------------------------------------------------------------------------//
//                Implementation of BmdCatchmentAreaList
//---------------------------------------------------------------------------//
fn serialize_nested_areas<S>(
    areas: &Vec<BmdCatchmentArea>,
    serializer: S,
) -> std::result::Result<S::Ok, S::Error>
where
    S: Serializer,
{
    use serde::Serialize;

    #[derive(Serialize)]
    struct Wrapper<'a> {
        #[serde(rename = "BMD_CATCHMENT_AREA")]
        items: &'a Vec<BmdCatchmentArea>,
    }

    let wrapper = Wrapper { items: areas };
    wrapper.serialize(serializer)
}

fn deserialize_nested_areas<'de, D>(
    deserializer: D,
) -> std::result::Result<Vec<BmdCatchmentArea>, D::Error>
where
    D: Deserializer<'de>,
{
    use serde::Deserialize;
    #[derive(Deserialize)]
    struct Wrapper {
        #[serde(rename = "BMD_CATCHMENT_AREA", default)]
        items: Vec<BmdCatchmentArea>,
    }

    let wrapper = Wrapper::deserialize(deserializer)?;
    Ok(wrapper.items)
}

impl Decodeable for BmdCatchmentAreaList {

    fn decode<R: ReadBytes>(data: &mut R, extra_data: &Option<DecodeableExtraData>) -> Result<Self> {
        let mut decoded = Self::default();
        decoded.serialise_version = data.read_u16()?;

        match decoded.serialise_version {
            1 => decoded.read_v1(data, extra_data)?,
            _ => return Err(RLibError::DecodingFastBinUnsupportedVersion(String::from("BmdCatchmentAreaList"), decoded.serialise_version)),
        }

        Ok(decoded)
    }
}

impl Encodeable for BmdCatchmentAreaList {

    fn encode<W: WriteBytes>(&mut self, buffer: &mut W, extra_data: &Option<EncodeableExtraData>) -> Result<()> {
        buffer.write_u16(self.serialise_version)?;

        match self.serialise_version {
            1 => self.write_v1(buffer, extra_data)?,
            _ => return Err(RLibError::EncodingFastBinUnsupportedVersion(String::from("BmdCatchmentAreaList"), self.serialise_version)),
        }

        Ok(())
    }
}
