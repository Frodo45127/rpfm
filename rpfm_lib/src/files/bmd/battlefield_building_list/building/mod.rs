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
use crate::files::bmd::common::properties::Properties;

use super::*;

mod v8;
mod v11;

//---------------------------------------------------------------------------//
//                              Enum & Structs
//---------------------------------------------------------------------------//

#[derive(Default, PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct Building {
    serialise_version: u16,
    building_id: String,
    parent_id: i32,
    building_key: String,
    position_type: String,
    transform: Transform3x4,
    properties: Properties,
    height_mode: String,
    uid: u64,
}

//---------------------------------------------------------------------------//
//                           Implementation of Building
//---------------------------------------------------------------------------//

impl Decodeable for Building {

    fn decode<R: ReadBytes>(data: &mut R, extra_data: &Option<DecodeableExtraData>) -> Result<Self> {
        let mut prop = Self::default();

        prop.serialise_version = data.read_u16()?;

        match prop.serialise_version {
            8 => prop.read_v8(data, extra_data)?,
            11 => prop.read_v11(data, extra_data)?,
            _ => return Err(RLibError::DecodingFastBinUnsupportedVersion(String::from("Building"), prop.serialise_version)),
        }

        Ok(prop)
    }
}

impl Encodeable for Building {

    fn encode<W: WriteBytes>(&mut self, buffer: &mut W, extra_data: &Option<EncodeableExtraData>) -> Result<()> {
        buffer.write_u16(self.serialise_version)?;

        match self.serialise_version {
            8 => self.write_v8(buffer, extra_data)?,
            11 => self.write_v11(buffer, extra_data)?,
            _ => return Err(RLibError::EncodingFastBinUnsupportedVersion(String::from("Building"), self.serialise_version)),
        }

        Ok(())
    }
}
