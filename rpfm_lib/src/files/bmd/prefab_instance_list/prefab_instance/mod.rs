//---------------------------------------------------------------------------//
// Copyright (c) 2017-2023 Ismael Gutiérrez González. All rights reserved.
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

use self::property_override::PropertyOverride;

use super::*;

mod property_override;
mod v6;
mod v8;
mod v9;

//---------------------------------------------------------------------------//
//                              Enum & Structs
//---------------------------------------------------------------------------//

#[derive(Default, PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct PrefabInstance {
    serialise_version: u16,
    key: String,
    transform: Transform4x4,
    property_overrides: Vec<PropertyOverride>,
    campaign_type_mask: u64,
    campaign_region_key: String,
    clamp_to_surface: bool,
    height_mode: String,
    uid: u64,
}

//---------------------------------------------------------------------------//
//                Implementation of PrefabInstance
//---------------------------------------------------------------------------//

impl Decodeable for PrefabInstance {

    fn decode<R: ReadBytes>(data: &mut R, extra_data: &Option<DecodeableExtraData>) -> Result<Self> {
        let mut decoded = Self::default();
        decoded.serialise_version = data.read_u16()?;

        match decoded.serialise_version {
            6 => decoded.read_v6(data, extra_data)?,
            8 => decoded.read_v8(data, extra_data)?,
            9 => decoded.read_v9(data, extra_data)?,
            _ => return Err(RLibError::DecodingFastBinUnsupportedVersion(String::from("PrefabInstance"), decoded.serialise_version)),
        }

        Ok(decoded)
    }
}

impl Encodeable for PrefabInstance {

    fn encode<W: WriteBytes>(&mut self, buffer: &mut W, extra_data: &Option<EncodeableExtraData>) -> Result<()> {
        buffer.write_u16(self.serialise_version)?;

        match self.serialise_version {
            6 => self.write_v6(buffer, extra_data)?,
            8 => self.write_v8(buffer, extra_data)?,
            9 => self.write_v9(buffer, extra_data)?,
            _ => return Err(RLibError::EncodingFastBinUnsupportedVersion(String::from("PrefabInstance"), self.serialise_version)),
        }

        Ok(())
    }
}

 
