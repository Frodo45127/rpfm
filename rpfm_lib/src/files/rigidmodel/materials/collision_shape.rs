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

//---------------------------------------------------------------------------//
//                            Implementation
//---------------------------------------------------------------------------//

/// Collision shape materials: the default material, followed by the sphere around the shape.
impl Material {
    pub fn read_collision_shape<R: ReadBytes>(data: &mut R) -> Result<Self> {
        let mut material = Self::read_default(data)?;
        material.uk_1 = data.read_u32()?;
        material.sphere_center = Vector3::new(data.read_f32()?, data.read_f32()?, data.read_f32()?);
        material.sphere_radius = data.read_f32()?;

        Ok(material)
    }

    pub fn write_collision_shape<W: WriteBytes>(&self, data: &mut W) -> Result<()> {
        self.write_default(data)?;
        data.write_u32(self.uk_1)?;
        data.write_f32(self.sphere_center.x)?;
        data.write_f32(self.sphere_center.y)?;
        data.write_f32(self.sphere_center.z)?;
        data.write_f32(self.sphere_radius)?;

        Ok(())
    }
}
