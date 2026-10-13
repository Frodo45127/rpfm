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

/// Projected decal materials of version 6 RigidModels: the decal's texture path, a flag, and since `ProjectedDecalV2`,
/// the box the decal is projected in.
impl Material {
    pub fn read_projected_decal<R: ReadBytes>(data: &mut R, mtype: MaterialType) -> Result<Self> {
        let mut material = Self {
            name: data.read_string_u8_0padded(PADDED_SIZE_256)?,
            ..Default::default()
        };

        if mtype != MaterialType::ProjectedDecal {
            material.decal_center = Vector3::new(data.read_f32()?, data.read_f32()?, data.read_f32()?);
            material.decal_size = Vector3::new(data.read_f32()?, data.read_f32()?, data.read_f32()?);
        }

        material.uk_1 = data.read_u32()?;

        if mtype != MaterialType::ProjectedDecal {
            material.uk_2 = data.read_u32()?;
            material.uk_3 = data.read_u32()?;
        }

        if mtype == MaterialType::ProjectedDecalV3 {
            material.uk_4 = data.read_u32()?;
        }

        Ok(material)
    }

    pub fn write_projected_decal<W: WriteBytes>(&self, data: &mut W, mtype: MaterialType) -> Result<()> {
        data.write_string_u8_0padded(self.name(), PADDED_SIZE_256, true)?;

        if mtype != MaterialType::ProjectedDecal {
            data.write_f32(self.decal_center.x)?;
            data.write_f32(self.decal_center.y)?;
            data.write_f32(self.decal_center.z)?;
            data.write_f32(self.decal_size.x)?;
            data.write_f32(self.decal_size.y)?;
            data.write_f32(self.decal_size.z)?;
        }

        data.write_u32(self.uk_1)?;

        if mtype != MaterialType::ProjectedDecal {
            data.write_u32(self.uk_2)?;
            data.write_u32(self.uk_3)?;
        }

        if mtype == MaterialType::ProjectedDecalV3 {
            data.write_u32(self.uk_4)?;
        }

        Ok(())
    }
}
