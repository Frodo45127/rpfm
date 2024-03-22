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

mod v11;

//---------------------------------------------------------------------------//
//                              Enum & Structs
//---------------------------------------------------------------------------//

#[derive(Default, PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct PropertyOverride {
    serialise_version: u16,
    building_id: String,
    starting_damage_unary: f32,
    on_fire: bool,
    start_disabled: bool,
    weak_point: bool,
    ai_breachable: bool,
    indestructible: bool,
    dockable: bool,
    toggleable: bool,
    lite: bool,
    cast_shadows: bool,
    key_building: bool,
    key_building_use_fort: bool,
    is_prop_in_outfield: bool,
    settlement_level_configurable: bool,
    hide_tooltip: bool,
    include_in_fog: bool,
}

//---------------------------------------------------------------------------//
//                Implementation of PropertyOverride
//---------------------------------------------------------------------------//

impl Decodeable for PropertyOverride {

    fn decode<R: ReadBytes>(data: &mut R, extra_data: &Option<DecodeableExtraData>) -> Result<Self> {
        let mut decoded = Self::default();
        decoded.serialise_version = data.read_u16()?;

        match decoded.serialise_version {
            11 => decoded.read_v11(data, extra_data)?,
            _ => return Err(RLibError::DecodingFastBinUnsupportedVersion(String::from("PropertyOverride"), decoded.serialise_version)),
        }

        Ok(decoded)
    }
}

impl Encodeable for PropertyOverride {

    fn encode<W: WriteBytes>(&mut self, buffer: &mut W, extra_data: &Option<EncodeableExtraData>) -> Result<()> {
        buffer.write_u16(self.serialise_version)?;

        match self.serialise_version {
            11 => self.write_v11(buffer, extra_data)?,
            _ => return Err(RLibError::EncodingFastBinUnsupportedVersion(String::from("PropertyOverride"), self.serialise_version)),
        }

        Ok(())
    }
}


 
