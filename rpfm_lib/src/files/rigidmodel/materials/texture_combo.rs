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
    pub fn read_texture_combo<R: ReadBytes>(data: &mut R, version: u32) -> Result<Self> {
        Ok(Self {
            name: read_padded_string(data, version, PADDED_SIZE_32)?,
            texture_directory: read_padded_string(data, version, PADDED_SIZE_256)?,
            ..Default::default()
        })
    }

    pub fn write_texture_combo<W: WriteBytes>(&self, data: &mut W, version: u32) -> Result<()> {
        write_padded_string(data, version, &self.name, PADDED_SIZE_32)?;
        write_padded_string(data, version, &self.texture_directory, PADDED_SIZE_256)
    }
}
