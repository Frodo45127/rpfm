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

use self::building_link::BuildingLink;

use super::*;

mod building_link;

//---------------------------------------------------------------------------//
//                              Enum & Structs
//---------------------------------------------------------------------------//

#[derive(Default, PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct ToggleableBuildingsSlot {
    toggleable_slot_type: String,
    building_links: Vec<BuildingLink>,
    composite_scenes: Vec<u8>,
    outlines: Vec<u8>,
    script_id: String,
    map_barrier_record_key: String,
    ground_melee_attack_allowed: bool,
}

//---------------------------------------------------------------------------//
//                Implementation of ToggleableBuildingsSlot
//---------------------------------------------------------------------------//

impl Decodeable for ToggleableBuildingsSlot {

    fn decode<R: ReadBytes>(data: &mut R, extra_data: &Option<DecodeableExtraData>) -> Result<Self> {
        let mut decoded = Self::default();
        decoded.toggleable_slot_type = data.read_sized_string_u8()?;

        for _ in 0..data.read_u32()? {
            decoded.building_links.push(BuildingLink::decode(data, extra_data)?);
        }

        for _ in 0..data.read_u32()? {

        }

        for _ in 0..data.read_u32()? {

        }

        decoded.script_id = data.read_sized_string_u8()?;
        decoded.map_barrier_record_key = data.read_sized_string_u8()?;
        decoded.ground_melee_attack_allowed = data.read_bool()?;

        Ok(decoded)
    }
}

impl Encodeable for ToggleableBuildingsSlot {

    fn encode<W: WriteBytes>(&mut self, buffer: &mut W, extra_data: &Option<EncodeableExtraData>) -> Result<()> {
        buffer.write_sized_string_u8(&self.toggleable_slot_type)?;

        buffer.write_u32(self.building_links.len() as u32)?;
        for link in &mut self.building_links {
            link.encode(buffer, extra_data)?;
        }

        buffer.write_u32(self.composite_scenes.len() as u32)?;
        for _ in &mut self.composite_scenes {

        }

        buffer.write_u32(self.outlines.len() as u32)?;
        for _ in &mut self.outlines {

        }

        buffer.write_sized_string_u8(&self.script_id)?;
        buffer.write_sized_string_u8(&self.map_barrier_record_key)?;
        buffer.write_bool(self.ground_melee_attack_allowed)?;

        Ok(())
    }
}
