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

use super::*;

mod v4;
mod v6;
mod v7;
mod v11;

//---------------------------------------------------------------------------//
//                              Enum & Structs
//---------------------------------------------------------------------------//

#[derive(Default, PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct Properties {
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
    clamp_to_surface: bool,
    cast_shadows: bool,
    dont_merge_building: bool,
    key_building: bool,
    key_building_use_fort: bool,
    is_prop_in_outfield: bool,
    settlement_level_configurable: bool,
    hide_tooltip: bool,
    include_in_fog: bool,
    tint_inherit_from_parent: bool,
}

//---------------------------------------------------------------------------//
//                           Implementation of Properties
//---------------------------------------------------------------------------//

impl Decodeable for Properties {

    fn decode<R: ReadBytes>(data: &mut R, extra_data: &Option<DecodeableExtraData>) -> Result<Self> {
        let mut flags = Self::default();
        flags.serialise_version = data.read_u16()?;

        match flags.serialise_version {
            4 => flags.read_v4(data, extra_data)?,
            6 => flags.read_v6(data, extra_data)?,
            7 => flags.read_v7(data, extra_data)?,
            11 => flags.read_v11(data, extra_data)?,
            _ => return Err(RLibError::DecodingFastBinUnsupportedVersion(String::from("Properties"), flags.serialise_version)),
        }

        Ok(flags)
    }
}

impl Encodeable for Properties {

    fn encode<W: WriteBytes>(&mut self, buffer: &mut W, extra_data: &Option<EncodeableExtraData>) -> Result<()> {
        buffer.write_u16(self.serialise_version)?;

        match self.serialise_version {
            4 => self.write_v4(buffer, extra_data)?,
            6 => self.write_v6(buffer, extra_data)?,
            7 => self.write_v7(buffer, extra_data)?,
            11 => self.write_v11(buffer, extra_data)?,
            _ => return Err(RLibError::EncodingFastBinUnsupportedVersion(String::from("Properties"), self.serialise_version)),
        }

        Ok(())
    }
}
