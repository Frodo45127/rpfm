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
use crate::files::{bmd::building_link::BuildingLink, Decodeable, EncodeableExtraData, Encodeable};

use super::*;

mod v2;
mod v7;
mod v8;
mod v10;
mod v11;

//---------------------------------------------------------------------------//
//                              Enum & Structs
//---------------------------------------------------------------------------//

#[derive(Default, PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct CaptureLocationSet {
    serialise_version: u16,
    capture_location_sets: Vec<CaptureLocationList>,
}

#[derive(Default, PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct CaptureLocationList {
    capture_locations: Vec<CaptureLocation>,
}

#[derive(Default, PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct CaptureLocation {
    location: Point2d,
    radius: f32,
    valid_for_min_num_players: u32,
    valid_for_max_num_players: u32,
    capture_point_type: String,
    restore_type: String,
    location_points: Vec<Point2d>,
    database_key: String,
    flag_facing: Point2d,
    destroy_building_on_capture: bool,
    disable_building_abilities_when_no_original_owner: bool,
    abilities_affect_globally: bool,
    building_links: Vec<BuildingLink>,
    toggle_slots_links: Vec<u32>,
    ai_hints_links: Vec<u8>,
    script_id: String,
    is_time_based: bool,
}

//---------------------------------------------------------------------------//
//                Implementation of CaptureLocationSet
//---------------------------------------------------------------------------//

impl Decodeable for CaptureLocationSet {

    fn decode<R: ReadBytes>(data: &mut R, extra_data: &Option<DecodeableExtraData>) -> Result<Self> {
        let mut decoded = Self::default();
        decoded.serialise_version = data.read_u16()?;

        match decoded.serialise_version {
            2 => decoded.read_v2(data, extra_data)?,
            7 => decoded.read_v7(data, extra_data)?,
            8 => decoded.read_v8(data, extra_data)?,
            10 => decoded.read_v10(data, extra_data)?,
            11 => decoded.read_v11(data, extra_data)?,
            _ => return Err(RLibError::DecodingFastBinUnsupportedVersion(String::from("CaptureLocationSet"), decoded.serialise_version)),
        }

        Ok(decoded)
    }
}

impl Encodeable for CaptureLocationSet {

    fn encode<W: WriteBytes>(&mut self, buffer: &mut W, extra_data: &Option<EncodeableExtraData>) -> Result<()> {
        buffer.write_u16(self.serialise_version)?;

        match self.serialise_version {
            2 => self.write_v2(buffer, extra_data)?,
            7 => self.write_v7(buffer, extra_data)?,
            8 => self.write_v8(buffer, extra_data)?,
            10 => self.write_v10(buffer, extra_data)?,
            11 => self.write_v11(buffer, extra_data)?,
            _ => return Err(RLibError::EncodingFastBinUnsupportedVersion(String::from("CaptureLocationSet"), self.serialise_version)),
        }

        Ok(())
    }
}
 
