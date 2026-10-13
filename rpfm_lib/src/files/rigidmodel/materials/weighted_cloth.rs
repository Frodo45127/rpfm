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

/// Size in bytes of the name of a cloth bone.
const CLOTH_BONE_NAME_SIZE: usize = 32;

//---------------------------------------------------------------------------//
//                            Implementation
//---------------------------------------------------------------------------//

/// Weighted cloth materials of version 6 RigidModels: the default material, followed by the bones of the cloth simulation.
impl Material {
    pub fn read_weighted_cloth<R: ReadBytes>(data: &mut R) -> Result<Self> {
        let mut material = Self::read_default(data)?;

        let bone_count = data.read_u32()?;
        material.uk_1 = data.read_u32()?;
        material.cloth_bones = Vec::with_capacity(bone_count as usize);

        for _ in 0..bone_count {
            material.cloth_bones.push(ClothBone {
                name: data.read_string_u16_0padded(CLOTH_BONE_NAME_SIZE)?,
                uk_1: Vector3::new(data.read_f32()?, data.read_f32()?, data.read_f32()?),
                uk_2: data.read_u32()?,
                uk_3: data.read_u32()?,
                uk_4: data.read_u32()?,
                uk_5: data.read_u32()?,
                uk_6: data.read_u32()?,
                uk_7: data.read_f32()?,
                matrix: Matrix3x4::new(
                    data.read_f32()?, data.read_f32()?, data.read_f32()?, data.read_f32()?,
                    data.read_f32()?, data.read_f32()?, data.read_f32()?, data.read_f32()?,
                    data.read_f32()?, data.read_f32()?, data.read_f32()?, data.read_f32()?
                ),
                parent_index: data.read_i32()?,
                vertex_index: data.read_u32()?,
            });
        }

        Ok(material)
    }

    pub fn write_weighted_cloth<W: WriteBytes>(&self, buffer: &mut W) -> Result<()> {
        self.write_default(buffer)?;

        buffer.write_u32(self.cloth_bones.len() as u32)?;
        buffer.write_u32(self.uk_1)?;

        for bone in &self.cloth_bones {
            buffer.write_string_u16_0padded(&bone.name, CLOTH_BONE_NAME_SIZE, false)?;
            buffer.write_f32(bone.uk_1.x)?;
            buffer.write_f32(bone.uk_1.y)?;
            buffer.write_f32(bone.uk_1.z)?;
            buffer.write_u32(bone.uk_2)?;
            buffer.write_u32(bone.uk_3)?;
            buffer.write_u32(bone.uk_4)?;
            buffer.write_u32(bone.uk_5)?;
            buffer.write_u32(bone.uk_6)?;
            buffer.write_f32(bone.uk_7)?;

            // The matrix is stored row by row, while nalgebra iterates matrices column by column.
            for row in bone.matrix.row_iter() {
                for value in row.iter() {
                    buffer.write_f32(*value)?;
                }
            }

            buffer.write_i32(bone.parent_index)?;
            buffer.write_u32(bone.vertex_index)?;
        }

        Ok(())
    }
}
