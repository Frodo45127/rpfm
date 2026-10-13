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

/// Materials of types 54, 56 and 57: the strings of the default material, its pivot and its matrix index, with no vertex format.
/// Types 56 and 57 have a second name, after the first one.
impl Material {
    pub fn read_uk_54<R: ReadBytes>(data: &mut R, version: u32, mtype: MaterialType) -> Result<Self> {
        Ok(Self {
            name: read_padded_string(data, version, PADDED_SIZE_32)?,
            name2: if mtype != MaterialType::Uk54 { read_padded_string(data, version, PADDED_SIZE_32)? } else { String::new() },
            texture_directory: read_padded_string(data, version, PADDED_SIZE_256)?,
            filters: read_padded_string(data, version, PADDED_SIZE_256)?,
            v_pivot: Vector3::new(data.read_f32()?, data.read_f32()?, data.read_f32()?),
            i_matrix_index: data.read_i32()?,
            ..Default::default()
        })
    }

    pub fn write_uk_54<W: WriteBytes>(&self, data: &mut W, version: u32, mtype: MaterialType) -> Result<()> {
        write_padded_string(data, version, &self.name, PADDED_SIZE_32)?;
        if mtype != MaterialType::Uk54 {
            write_padded_string(data, version, &self.name2, PADDED_SIZE_32)?;
        }
        write_padded_string(data, version, &self.texture_directory, PADDED_SIZE_256)?;
        write_padded_string(data, version, &self.filters, PADDED_SIZE_256)?;
        data.write_f32(self.v_pivot.x)?;
        data.write_f32(self.v_pivot.y)?;
        data.write_f32(self.v_pivot.z)?;
        data.write_i32(self.i_matrix_index)
    }
}
