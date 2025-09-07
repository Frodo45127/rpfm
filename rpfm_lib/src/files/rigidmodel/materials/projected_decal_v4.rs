//---------------------------------------------------------------------------//
// Copyright (c) 2017-2024 Ismael Gutiérrez González. All rights reserved.
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
    pub fn read_projected_decal_v4<R: ReadBytes>(data: &mut R) -> Result<Self> {
        Ok(Self {
            name: data.read_string_u8_0padded(PADDED_SIZE_64)?,
            uk_1: data.read_u32()?,
            uk_2: data.read_u32()?,
            uk_3: data.read_u32()?,
            uk_4: data.read_u32()?,
            uk_5: data.read_u32()?,
            uk_6: data.read_u32()?,
            ..Default::default()
        })
    }

    pub fn write_projected_decal_v4<W: WriteBytes>(&self, data: &mut W) -> Result<()> {
        data.write_string_u8_0padded(self.name(), PADDED_SIZE_64, true)?;
        data.write_u32(*self.uk_1())?;
        data.write_u32(*self.uk_2())?;
        data.write_u32(*self.uk_3())?;
        data.write_u32(*self.uk_4())?;
        data.write_u32(*self.uk_5())?;
        data.write_u32(*self.uk_6())?;

        Ok(())
    }
}
