//---------------------------------------------------------------------------//
// Copyright (c) 2017-2024 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

use crate::binary::ReadBytes;
use crate::error::Result;

use super::*;

//---------------------------------------------------------------------------//
//                    Implementation of CaptureLocationSet
//---------------------------------------------------------------------------//

// BIG NOTE: This decoder is incomplete until I find a file to check it against.

impl CaptureLocationSet {

    pub(crate) fn read_v8<R: ReadBytes>(&mut self, data: &mut R, extra_data: &Option<DecodeableExtraData>) -> Result<()> {
        for _ in 0..data.read_u32()? {
            let mut list = CaptureLocationList::default();
            for _ in 0..data.read_u32()? {
                let mut location = CaptureLocation::default();

                location.set_location(Point2d::decode(data, extra_data)?);
                location.set_radius(data.read_f32()?);
                location.set_valid_for_min_num_players(data.read_u32()?);
                location.set_valid_for_max_num_players(data.read_u32()?);
                location.set_capture_point_type(data.read_sized_string_u8()?);
                location.set_restore_type(data.read_sized_string_u8()?);

                for _ in 0..data.read_u32()? {
                    location.location_points_mut().push(Point2d::decode(data, extra_data)?);
                }

                location.set_database_key(data.read_sized_string_u8()?);

                location.set_flag_facing(Point2d::decode(data, extra_data)?);

                location.set_destroy_building_on_capture(data.read_bool()?);
                location.set_disable_building_abilities_when_no_original_owner(data.read_bool()?);
                location.set_abilities_affect_globally(data.read_bool()?);

                for _ in 0..data.read_u32()? {
                    location.building_links_mut().push(BuildingLink::decode(data, extra_data)?);
                }

                for _ in 0..data.read_u32()? {
                    location.toggle_slots_links_mut().push(data.read_u32()?);
                }

                for _ in 0..data.read_u32()? {
                    location.ai_hints_links_mut().push(data.read_u8()?);
                }

                location.set_script_id(data.read_sized_string_u8()?);
                location.set_is_time_based(data.read_bool()?);

                list.capture_locations.push(location);
            }

            self.capture_location_sets.push(list);
        }

        Ok(())
    }


    pub(crate) fn write_v8<W: WriteBytes>(&mut self, buffer: &mut W, extra_data: &Option<EncodeableExtraData>) -> Result<()> {
        buffer.write_u32(self.capture_location_sets.len() as u32)?;
        for list in &mut self.capture_location_sets {
            buffer.write_u32(list.capture_locations.len() as u32)?;
            for capture_location in list.capture_locations_mut() {
                capture_location.location_mut().encode(buffer, extra_data)?;

                buffer.write_f32(*capture_location.radius())?;
                buffer.write_u32(*capture_location.valid_for_min_num_players())?;
                buffer.write_u32(*capture_location.valid_for_max_num_players())?;
                buffer.write_sized_string_u8(capture_location.capture_point_type())?;
                buffer.write_sized_string_u8(capture_location.restore_type())?;

                buffer.write_u32(capture_location.location_points().len() as u32)?;
                for location_point in capture_location.location_points_mut() {
                    location_point.encode(buffer, extra_data)?;
                }

                buffer.write_sized_string_u8(capture_location.database_key())?;

                capture_location.flag_facing_mut().encode(buffer, extra_data)?;

                buffer.write_bool(*capture_location.destroy_building_on_capture())?;
                buffer.write_bool(*capture_location.disable_building_abilities_when_no_original_owner())?;
                buffer.write_bool(*capture_location.abilities_affect_globally())?;

                buffer.write_u32(capture_location.building_links().len() as u32)?;
                for building_link in capture_location.building_links_mut() {
                    building_link.encode(buffer, extra_data)?;
                }

                buffer.write_u32(capture_location.toggle_slots_links().len() as u32)?;
                for slot_link in capture_location.toggle_slots_links_mut() {
                    buffer.write_u32(*slot_link)?;
                }

                buffer.write_u32(capture_location.ai_hints_links().len() as u32)?;
                for hint_link in capture_location.ai_hints_links_mut() {
                    buffer.write_u8(*hint_link)?;
                }

                buffer.write_sized_string_u8(capture_location.script_id())?;
                buffer.write_bool(*capture_location.is_time_based())?;
            }
        }

        Ok(())
    }
}
