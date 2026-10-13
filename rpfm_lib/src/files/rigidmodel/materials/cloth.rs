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
    pub fn read_cloth<R: ReadBytes>(data: &mut R, version: u32) -> Result<Self> {
        let mut mat = Self::read_default(data, version)?;

        // These have a ton of extra data here.
        let uk_7_count = data.read_u32()?;
        mat.uk_7 = Vec::with_capacity(uk_7_count as usize);

        let uk_8_count = data.read_u32()?;
        mat.uk_8 = Vec::with_capacity(uk_8_count as usize);

        let uk_9_count = data.read_u32()?;
        mat.uk_9 = Vec::with_capacity(uk_9_count as usize);

        for _ in 0..mat.uk_7.capacity() {
            let mut uk = Uk7::default();

            uk.uk1 = data.read_i32()?;
            uk.uk2 = data.read_i32()?;
            uk.uk3 = data.read_f32()?;

            mat.uk_7.push(uk);
        }

        for _ in 0..mat.uk_8.capacity() {
            let mut uk = Uk8::default();

            uk.uk1 = data.read_i32()?;

            mat.uk_8.push(uk);
        }

        for _ in 0..mat.uk_9.capacity() {
            let mut uk = Uk9::default();

            uk.uk1 = data.read_i32()?;
            uk.uk2 = data.read_i32()?;
            uk.uk3 = data.read_i32()?;

            mat.uk_9.push(uk);
        }

        Ok(mat)
    }
    pub fn write_cloth<W: WriteBytes>(&self, buffer: &mut W, version: u32) -> Result<()> {
        self.write_default(buffer, version)?;

        buffer.write_i32(self.uk_7.len() as i32)?;
        buffer.write_i32(self.uk_8.len() as i32)?;
        buffer.write_i32(self.uk_9.len() as i32)?;

        for uk in self.uk_7() {
            buffer.write_i32(uk.uk1)?;
            buffer.write_i32(uk.uk2)?;
            buffer.write_f32(uk.uk3)?;
        }

        for uk in self.uk_8() {
            buffer.write_i32(uk.uk1)?;
        }

        for uk in self.uk_9() {
            buffer.write_i32(uk.uk1)?;
            buffer.write_i32(uk.uk2)?;
            buffer.write_i32(uk.uk3)?;
        }

        Ok(())
    }
}
