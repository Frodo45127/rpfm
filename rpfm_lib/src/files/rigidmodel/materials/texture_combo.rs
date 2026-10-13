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

/// Texture combo materials: a name and a texture directory.
impl Material {
    pub fn read_texture_combo<R: ReadBytes>(data: &mut R) -> Result<Self> {
        Ok(Self {
            name: data.read_string_u8_0padded(PADDED_SIZE_32)?,
            texture_directory: data.read_string_u8_0padded(PADDED_SIZE_256)?,
            ..Default::default()
        })
    }

    pub fn write_texture_combo<W: WriteBytes>(&self, data: &mut W) -> Result<()> {
        data.write_string_u8_0padded(&self.name, PADDED_SIZE_32, true)?;
        data.write_string_u8_0padded(&self.texture_directory, PADDED_SIZE_256, true)
    }
}
