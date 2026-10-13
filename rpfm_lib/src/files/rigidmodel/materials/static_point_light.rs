//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

use super::*;

/// Size in bytes of the name of a static point light.
const STATIC_POINT_LIGHT_NAME_SIZE: usize = 512;

//---------------------------------------------------------------------------//
//                            Implementation
//---------------------------------------------------------------------------//

/// Static point light materials: the name of the light, and its position in `v_pivot`.
impl Material {
    pub fn read_static_point_light<R: ReadBytes>(data: &mut R) -> Result<Self> {
        Ok(Self {
            name: data.read_string_u8_0padded(STATIC_POINT_LIGHT_NAME_SIZE)?,
            v_pivot: Vector3::new(data.read_f32()?, data.read_f32()?, data.read_f32()?),
            ..Default::default()
        })
    }

    pub fn write_static_point_light<W: WriteBytes>(&self, data: &mut W) -> Result<()> {
        data.write_string_u8_0padded(&self.name, STATIC_POINT_LIGHT_NAME_SIZE, true)?;
        data.write_f32(self.v_pivot.x)?;
        data.write_f32(self.v_pivot.y)?;
        data.write_f32(self.v_pivot.z)
    }
}
