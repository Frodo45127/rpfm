//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
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
//                           Implementation of Properties
//---------------------------------------------------------------------------//

impl Properties {

    pub(crate) fn read_v11<R: ReadBytes>(&mut self, data: &mut R, _extra_data: &Option<DecodeableExtraData>) -> Result<()> {
        self.building_id = data.read_sized_string_u8()?;
        self.starting_damage_unary = data.read_f32()?;
        self.on_fire = data.read_bool()?;
        self.start_disabled = data.read_bool()?;
        self.weak_point = data.read_bool()?;
        self.ai_breachable = data.read_bool()?;
        self.indestructible = data.read_bool()?;
        self.dockable = data.read_bool()?;
        self.toggleable = data.read_bool()?;
        self.lite = data.read_bool()?;
        self.cast_shadows = data.read_bool()?;
        self.key_building = data.read_bool()?;
        self.key_building_use_fort = data.read_bool()?;
        self.is_prop_in_outfield = data.read_bool()?;
        self.settlement_level_configurable = data.read_bool()?;
        self.hide_tooltip = data.read_bool()?;
        self.include_in_fog = data.read_bool()?;

        Ok(())
    }

    pub(crate) fn write_v11<W: WriteBytes>(&mut self, buffer: &mut W, _extra_data: &Option<EncodeableExtraData>) -> Result<()> {
        buffer.write_sized_string_u8(&self.building_id)?;
        buffer.write_f32(self.starting_damage_unary)?;
        buffer.write_bool(self.on_fire)?;
        buffer.write_bool(self.start_disabled)?;
        buffer.write_bool(self.weak_point)?;
        buffer.write_bool(self.ai_breachable)?;
        buffer.write_bool(self.indestructible)?;
        buffer.write_bool(self.dockable)?;
        buffer.write_bool(self.toggleable)?;
        buffer.write_bool(self.lite)?;
        buffer.write_bool(self.cast_shadows)?;
        buffer.write_bool(self.key_building)?;
        buffer.write_bool(self.key_building_use_fort)?;
        buffer.write_bool(self.is_prop_in_outfield)?;
        buffer.write_bool(self.settlement_level_configurable)?;
        buffer.write_bool(self.hide_tooltip)?;
        buffer.write_bool(self.include_in_fog)?;

        Ok(())
    }
}
