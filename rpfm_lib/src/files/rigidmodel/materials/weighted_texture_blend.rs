//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

use crate::files::rigidmodel::PADDED_SIZE_64;

use super::*;

//---------------------------------------------------------------------------//
//                            Implementation
//---------------------------------------------------------------------------//

impl Material {
    pub fn read_weighted_texture_blend<R: ReadBytes>(data: &mut R, version: u32) -> Result<Self> {
        Ok(Self {
            name: read_padded_string(data, version, PADDED_SIZE_64)?,
            uk_1: data.read_u32()?,
            uk_2: data.read_u32()?,
            uk_3: data.read_u32()?,
            uk_4: data.read_u32()?,
            uk_5: data.read_u32()?,
            uk_6: data.read_u32()?,
            ..Default::default()
        })
    }
    pub fn write_weighted_texture_blend<W: WriteBytes>(&self, data: &mut W, version: u32) -> Result<()> {
        write_padded_string(data, version, self.name(), PADDED_SIZE_64)?;
        data.write_u32(*self.uk_1())?;
        data.write_u32(*self.uk_2())?;
        data.write_u32(*self.uk_3())?;
        data.write_u32(*self.uk_4())?;
        data.write_u32(*self.uk_5())?;
        data.write_u32(*self.uk_6())?;

        Ok(())
    }
}
