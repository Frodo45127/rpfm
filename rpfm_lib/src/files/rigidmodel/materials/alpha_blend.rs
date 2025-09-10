//---------------------------------------------------------------------------//
// Copyright (c) 2017-2024 Ismael Gutiérrez González. All rights reserved.
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

impl Material {
    pub fn read_alpha_blend<R: ReadBytes>(data: &mut R) -> Result<Self> {
        Ok(Self {
            name: data.read_string_u8_0padded(PADDED_SIZE_256)?,
            ..Default::default()
        })
    }

    pub fn write_alpha_blend<W: WriteBytes>(&self, data: &mut W) -> Result<()> {
        data.write_string_u8_0padded(self.name(), PADDED_SIZE_256, true)?;

        Ok(())
    }
}
