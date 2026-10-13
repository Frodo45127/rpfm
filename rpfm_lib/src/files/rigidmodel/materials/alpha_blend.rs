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

impl Material {
    pub fn read_alpha_blend<R: ReadBytes>(data: &mut R, version: u32) -> Result<Self> {
        Ok(Self {
            name: read_padded_string(data, version, PADDED_SIZE_256)?,
            ..Default::default()
        })
    }

    pub fn write_alpha_blend<W: WriteBytes>(&self, data: &mut W, version: u32) -> Result<()> {
        write_padded_string(data, version, self.name(), PADDED_SIZE_256)?;

        Ok(())
    }
}
