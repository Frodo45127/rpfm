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
//                           Implementation of TileSet
//---------------------------------------------------------------------------//

impl TileSet {

    pub(crate) fn read_v1<R: ReadBytes>(&mut self, data: &mut R, _extra_data: &Option<DecodeableExtraData>) -> Result<()> {
        self.name = data.read_sized_string_u8()?;
        self.linking_tile = data.read_sized_string_u8()?;
        self.shared_geometry = data.read_sized_string_u8()?;
        self.also_place_tile_set = data.read_sized_string_u8()?;
        self.link_as_set = data.read_sized_string_u8()?;
        self.red = data.read_f32()?;
        self.green = data.read_f32()?;
        self.blue = data.read_f32()?;

        Ok(())
    }

    pub(crate) fn write_v1<W: WriteBytes>(&mut self, buffer: &mut W, _extra_data: &Option<EncodeableExtraData>) -> Result<()> {
        buffer.write_sized_string_u8(&self.name)?;
        buffer.write_sized_string_u8(&self.linking_tile)?;
        buffer.write_sized_string_u8(&self.shared_geometry)?;
        buffer.write_sized_string_u8(&self.also_place_tile_set)?;
        buffer.write_sized_string_u8(&self.link_as_set)?;

        buffer.write_f32(self.red)?;
        buffer.write_f32(self.green)?;
        buffer.write_f32(self.blue)?;

        Ok(())
    }
}
