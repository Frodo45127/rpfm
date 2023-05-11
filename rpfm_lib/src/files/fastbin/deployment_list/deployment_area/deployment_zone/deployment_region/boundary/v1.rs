//---------------------------------------------------------------------------//
// Copyright (c) 2017-2023 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

use crate::binary::ReadBytes;
use crate::error::Result;

use super::*;

//---------------------------------------------------------------------------//
//                    Implementation of Boundary
//---------------------------------------------------------------------------//

impl Boundary {

    pub(crate) fn read_v1<R: ReadBytes>(&mut self, data: &mut R, _extra_data: &Option<DecodeableExtraData>) -> Result<()> {
        self.deployment_area_boundary_type = data.read_sized_string_u8()?;

        for _ in 0..data.read_u32()? {
            self.boundary.push(Position {
                x: data.read_f32()?,
                y: data.read_f32()?,
            });
        }

        Ok(())
    }

    pub(crate) fn write_v1<W: WriteBytes>(&mut self, buffer: &mut W, _extra_data: &Option<EncodeableExtraData>) -> Result<()> {
        buffer.write_sized_string_u8(&self.deployment_area_boundary_type)?;
        buffer.write_u32(self.boundary.len() as u32)?;

        for boundary in &mut self.boundary {
            buffer.write_f32(boundary.x)?;
            buffer.write_f32(boundary.y)?;
        }

        Ok(())
    }
}
