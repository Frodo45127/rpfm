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

/// Custom terrain materials (cliffs and other hand-made terrain pieces): just the path of their texture.
impl Material {
    pub fn read_custom_terrain<R: ReadBytes>(data: &mut R) -> Result<Self> {
        let (name, name_padding) = read_string_u8_0padded_with_tail(data, PADDED_SIZE_256)?;
        Ok(Self {
            name,
            name_padding,
            ..Default::default()
        })
    }

    pub fn write_custom_terrain<W: WriteBytes>(&self, data: &mut W) -> Result<()> {
        write_string_u8_0padded_with_tail(data, &self.name, &self.name_padding, PADDED_SIZE_256)
    }
}
