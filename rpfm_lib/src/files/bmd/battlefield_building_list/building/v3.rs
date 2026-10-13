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
use crate::files::Decodeable;

use super::*;

//---------------------------------------------------------------------------//
//                           Implementation of Building
//---------------------------------------------------------------------------//

/// Version 3, used by Attila: the placement is a position, an orientation and a scale, and the properties are only the damage and the flags.
impl Building {

    pub(crate) fn read_v3<R: ReadBytes>(&mut self, data: &mut R, extra_data: &Option<DecodeableExtraData>) -> Result<()> {
        self.building_key = data.read_sized_string_u8()?;
        self.position_type = data.read_sized_string_u8()?;
        self.position = Point3d::decode(data, extra_data)?;
        self.orientation = Quaternion::decode(data, extra_data)?;
        self.scale = Point3d::decode(data, extra_data)?;

        let properties = &mut self.properties;
        properties.set_starting_damage_unary(data.read_f32()?);
        properties.set_on_fire(data.read_bool()?);
        properties.set_start_disabled(data.read_bool()?);
        properties.set_weak_point(data.read_bool()?);
        properties.set_ai_breachable(data.read_bool()?);
        properties.set_indestructible(data.read_bool()?);
        properties.set_dockable(data.read_bool()?);
        properties.set_toggleable(data.read_bool()?);
        properties.set_lite(data.read_bool()?);

        Ok(())
    }

    pub(crate) fn write_v3<W: WriteBytes>(&mut self, buffer: &mut W, extra_data: &Option<EncodeableExtraData>) -> Result<()> {
        buffer.write_sized_string_u8(&self.building_key)?;
        buffer.write_sized_string_u8(&self.position_type)?;
        self.position.encode(buffer, extra_data)?;
        self.orientation.encode(buffer, extra_data)?;
        self.scale.encode(buffer, extra_data)?;

        let properties = &self.properties;
        buffer.write_f32(*properties.starting_damage_unary())?;
        buffer.write_bool(*properties.on_fire())?;
        buffer.write_bool(*properties.start_disabled())?;
        buffer.write_bool(*properties.weak_point())?;
        buffer.write_bool(*properties.ai_breachable())?;
        buffer.write_bool(*properties.indestructible())?;
        buffer.write_bool(*properties.dockable())?;
        buffer.write_bool(*properties.toggleable())?;
        buffer.write_bool(*properties.lite())?;

        Ok(())
    }
}
