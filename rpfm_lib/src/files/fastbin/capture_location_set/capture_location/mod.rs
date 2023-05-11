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
use crate::error::Result;
use crate::files::{Decodeable, EncodeableExtraData, Encodeable};

use self::building_link::BuildingLink;

use super::*;

mod building_link;

//---------------------------------------------------------------------------//
//                              Enum & Structs
//---------------------------------------------------------------------------//

#[derive(Default, PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct CaptureLocation {
    location: Location,
    radius: f32,
    valid_for_min_num_players: u32,
    valid_for_max_num_players: u32,
    capture_point_type: String,
    restore_type: String,
    location_points: Vec<Point>,
    database_key: String,
    flag_facing: FlagFacing,
    destroy_building_on_capture: bool,
    disable_building_abilities_when_no_original_owner: bool,
    abilities_affect_globally: bool,
    building_links: Vec<BuildingLink>,
    toggle_slots_links: Vec<u32>,
    ai_hints_links: Vec<u8>,
    script_id: String,
    is_time_based: bool,
}

#[derive(Default, PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct Location {
    x: f32,
    y: f32
}

#[derive(Default, PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct Point {
    x: f32,
    y: f32
}

#[derive(Default, PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct FlagFacing {
    x: f32,
    y: f32
}

//---------------------------------------------------------------------------//
//                Implementation of CaptureLocation
//---------------------------------------------------------------------------//

impl Decodeable for CaptureLocation {

    fn decode<R: ReadBytes>(data: &mut R, extra_data: &Option<DecodeableExtraData>) -> Result<Self> {
        let mut decoded = Self::default();

        decoded.location = Location {
            x: data.read_f32()?,
            y: data.read_f32()?,
        };

        decoded.radius = data.read_f32()?;
        decoded.valid_for_min_num_players = data.read_u32()?;
        decoded.valid_for_max_num_players = data.read_u32()?;
        decoded.capture_point_type = data.read_sized_string_u8()?;
        decoded.restore_type = data.read_sized_string_u8()?;

        for _ in 0..data.read_u32()? {
            decoded.location_points.push(Point {
                x: data.read_f32()?,
                y: data.read_f32()?,
            });
        }

        decoded.database_key = data.read_sized_string_u8()?;

        decoded.flag_facing = FlagFacing {
            x: data.read_f32()?,
            y: data.read_f32()?,
        };

        decoded.destroy_building_on_capture = data.read_bool()?;
        decoded.disable_building_abilities_when_no_original_owner = data.read_bool()?;
        decoded.abilities_affect_globally = data.read_bool()?;

        for _ in 0..data.read_u32()? {
            decoded.building_links.push(BuildingLink::decode(data, extra_data)?);
        }

        for _ in 0..data.read_u32()? {
            decoded.toggle_slots_links.push(data.read_u32()?);
        }

        for _ in 0..data.read_u32()? {
            decoded.ai_hints_links.push(data.read_u8()?);
        }

        decoded.script_id = data.read_sized_string_u8()?;
        decoded.is_time_based = data.read_bool()?;

        Ok(decoded)
    }
}

impl Encodeable for CaptureLocation {

    fn encode<W: WriteBytes>(&mut self, buffer: &mut W, extra_data: &Option<EncodeableExtraData>) -> Result<()> {
        buffer.write_f32(self.location.x)?;
        buffer.write_f32(self.location.y)?;

        buffer.write_f32(self.radius)?;
        buffer.write_u32(self.valid_for_min_num_players)?;
        buffer.write_u32(self.valid_for_max_num_players)?;
        buffer.write_sized_string_u8(&self.capture_point_type)?;
        buffer.write_sized_string_u8(&self.restore_type)?;

        buffer.write_u32(self.location_points.len() as u32)?;
        for location_point in &self.location_points {
            buffer.write_f32(location_point.x)?;
            buffer.write_f32(location_point.y)?;
        }

        buffer.write_sized_string_u8(&self.database_key)?;

        buffer.write_f32(self.flag_facing.x)?;
        buffer.write_f32(self.flag_facing.y)?;

        buffer.write_bool(self.destroy_building_on_capture)?;
        buffer.write_bool(self.disable_building_abilities_when_no_original_owner)?;
        buffer.write_bool(self.abilities_affect_globally)?;

        buffer.write_u32(self.building_links.len() as u32)?;
        for building_link in &mut self.building_links {
            building_link.encode(buffer, extra_data)?;
        }

        buffer.write_u32(self.toggle_slots_links.len() as u32)?;
        for slot_link in &self.toggle_slots_links {
            buffer.write_u32(*slot_link)?;
        }

        buffer.write_u32(self.ai_hints_links.len() as u32)?;
        for hint_link in &self.ai_hints_links {
            buffer.write_u8(*hint_link)?;
        }

        buffer.write_sized_string_u8(&self.script_id)?;
        buffer.write_bool(self.is_time_based)?;

        Ok(())
    }
}

