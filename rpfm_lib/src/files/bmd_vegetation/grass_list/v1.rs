//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

use half::f16;

use crate::binary::ReadBytes;
use crate::error::Result;

use super::*;

//---------------------------------------------------------------------------//
//                           Implementation of GrassList
//---------------------------------------------------------------------------//

impl GrassList {

    pub(crate) fn read_v1<R: ReadBytes>(&mut self, data: &mut R, _extra_data: &Option<DecodeableExtraData>) -> Result<()> {
        for _ in 0..data.read_u32()? {
            let mut vector = GrassItemVector {
                key: data.read_sized_string_u8()?,
                ..Default::default()
            };

            for _ in 0..data.read_u32()? {
                vector.value.push(GrassItem {
                    x: data.read_f16()?.to_f32(),
                    y: data.read_f16()?.to_f32(),
                    z: data.read_f16()?.to_f32(),
                });
            }

            self.grass_list.push(vector);
        }

        Ok(())
    }

    pub(crate) fn write_v1<W: WriteBytes>(&mut self, buffer: &mut W, _extra_data: &Option<EncodeableExtraData>) -> Result<()> {
        buffer.write_u32(self.grass_list.len() as u32)?;

        for vector in &self.grass_list {
            buffer.write_sized_string_u8(&vector.key)?;
            buffer.write_u32(vector.value.len() as u32)?;
            for item in &vector.value {
                buffer.write_f16(f16::from_f32(item.x))?;
                buffer.write_f16(f16::from_f32(item.y))?;
                buffer.write_f16(f16::from_f32(item.z))?;
            }
        }

        Ok(())
    }
}
