//---------------------------------------------------------------------------//
// Copyright (c) 2017-2024 Ismael Gutiérrez González. All rights reserved.
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
//                           Implementation of TileVariation
//---------------------------------------------------------------------------//

impl TileVariation {

    pub(crate) fn read_v2<R: ReadBytes>(&mut self, data: &mut R, _extra_data: &Option<DecodeableExtraData>) -> Result<()> {
        self.location = data.read_sized_string_u8()?;
        self.min_height = data.read_f32()?;
        self.scale = data.read_f32()?;
        self.normal_strength = data.read_f32()?;
        self.overlap_border_size = data.read_f32()?;
        self.raw_data_tri_density = data.read_u32()?;
        self.blend_common = data.read_sized_string_u8()?;
        self.index_common = data.read_sized_string_u8()?;
        self.normal_common = data.read_sized_string_u8()?;
        self.red = data.read_f32()?;
        self.green = data.read_f32()?;
        self.blue = data.read_f32()?;
        self.barbarian = data.read_bool()?;

        Ok(())
    }

    pub(crate) fn write_v2<W: WriteBytes>(&mut self, buffer: &mut W, _extra_data: &Option<EncodeableExtraData>) -> Result<()> {
        buffer.write_sized_string_u8(&self.location)?;
        buffer.write_f32(self.min_height)?;
        buffer.write_f32(self.scale)?;
        buffer.write_f32(self.normal_strength)?;
        buffer.write_f32(self.overlap_border_size)?;
        buffer.write_u32(self.raw_data_tri_density)?;
        buffer.write_sized_string_u8(&self.blend_common)?;
        buffer.write_sized_string_u8(&self.index_common)?;
        buffer.write_sized_string_u8(&self.normal_common)?;
        buffer.write_f32(self.red)?;
        buffer.write_f32(self.green)?;
        buffer.write_f32(self.blue)?;
        buffer.write_bool(self.barbarian)?;

        Ok(())
    }
}
