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
//               Implementation of TerrainStencilTriangle
//---------------------------------------------------------------------------//

impl TerrainStencilTriangle {

    pub(crate) fn read_v3<R: ReadBytes>(&mut self, data: &mut R, extra_data: &Option<DecodeableExtraData>) -> Result<()> {
        self.position_0 = Position {
            x: data.read_f32()?,
            y: data.read_f32()?,
            z: data.read_f32()?,
        };
        self.position_1 = Position {
            x: data.read_f32()?,
            y: data.read_f32()?,
            z: data.read_f32()?,
        };
        self.position_2 = Position {
            x: data.read_f32()?,
            y: data.read_f32()?,
            z: data.read_f32()?,
        };
        self.height_mode = data.read_sized_string_u8()?;
        self.flags = Flags::decode(data, extra_data)?;

        Ok(())
    }

    pub(crate) fn write_v3<W: WriteBytes>(&mut self, buffer: &mut W, extra_data: &Option<EncodeableExtraData>) -> Result<()> {
        buffer.write_f32(self.position_0.x)?;
        buffer.write_f32(self.position_0.y)?;
        buffer.write_f32(self.position_0.z)?;
        buffer.write_f32(self.position_1.x)?;
        buffer.write_f32(self.position_1.y)?;
        buffer.write_f32(self.position_1.z)?;
        buffer.write_f32(self.position_2.x)?;
        buffer.write_f32(self.position_2.y)?;
        buffer.write_f32(self.position_2.z)?;

        buffer.write_sized_string_u8(&self.height_mode)?;

        self.flags.encode(buffer, extra_data)?;

        Ok(())
    }
}

