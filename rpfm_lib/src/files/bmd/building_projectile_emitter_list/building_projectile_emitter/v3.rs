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
//              Implementation of BuildingProjectileEmitter
//---------------------------------------------------------------------------//

impl BuildingProjectileEmitter {

    pub(crate) fn read_v3<R: ReadBytes>(&mut self, data: &mut R, extra_data: &Option<DecodeableExtraData>) -> Result<()> {
        self.position = Point3d::decode(data, extra_data)?;
        self.direction = Point3d::decode(data, extra_data)?;
        self.building_index = data.read_u32()?;
        self.height_mode = data.read_sized_string_u8()?;
        self.specialized_building_projectile_emitter_key = data.read_sized_string_u8()?;

        Ok(())
    }

    pub(crate) fn write_v3<W: WriteBytes>(&mut self, buffer: &mut W, extra_data: &Option<EncodeableExtraData>) -> Result<()> {
        self.position.encode(buffer, extra_data)?;
        self.direction.encode(buffer, extra_data)?;
        buffer.write_u32(self.building_index)?;
        buffer.write_sized_string_u8(&self.height_mode)?;
        buffer.write_sized_string_u8(&self.specialized_building_projectile_emitter_key)?;

        Ok(())
    }
}
