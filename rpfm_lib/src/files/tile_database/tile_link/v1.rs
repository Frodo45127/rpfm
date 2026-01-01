//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
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
//                           Implementation of TileLink
//---------------------------------------------------------------------------//

impl TileLink {

    pub(crate) fn read_v1<R: ReadBytes>(&mut self, data: &mut R, _extra_data: &Option<DecodeableExtraData>) -> Result<()> {
        self.link_set = data.read_sized_string_u8()?;
        self.x = data.read_i32()?;
        self.y = data.read_i32()?;
        self.base_x = data.read_i32()?;
        self.base_y = data.read_i32()?;
        self.is_entry = data.read_bool()?;

        for _ in 0..data.read_u32()? {
            self.blend_quads.push(data.read_u32()?);
        }

        self.blend_size = data.read_u32()?;
        self.no_offline_blend = data.read_bool()?;
        self.test = data.read_sized_string_u8()?;

        Ok(())
    }

    pub(crate) fn write_v1<W: WriteBytes>(&mut self, buffer: &mut W, _extra_data: &Option<EncodeableExtraData>) -> Result<()> {
        buffer.write_sized_string_u8(&self.link_set)?;
        buffer.write_i32(self.x)?;
        buffer.write_i32(self.y)?;
        buffer.write_i32(self.base_x)?;
        buffer.write_i32(self.base_y)?;
        buffer.write_bool(self.is_entry)?;

        buffer.write_u32(self.blend_quads.len() as u32)?;
        for quad in self.blend_quads() {
            buffer.write_u32(*quad)?;
        }

        buffer.write_u32(self.blend_size)?;
        buffer.write_bool(self.no_offline_blend)?;
        buffer.write_sized_string_u8(&self.test)?;

        Ok(())
    }
}
