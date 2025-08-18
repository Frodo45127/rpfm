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
//                           Implementation of Tile
//---------------------------------------------------------------------------//

impl Tile {

    pub(crate) fn read_v3<R: ReadBytes>(&mut self, data: &mut R, extra_data: &Option<DecodeableExtraData>) -> Result<()> {
        self.location = data.read_sized_string_u8()?;
        self.tile_set = data.read_sized_string_u8()?;
        self.mask = data.read_sized_string_u8()?;
        self.width = data.read_u32()?;
        self.height = data.read_u32()?;
        self.red = data.read_f32()?;
        self.green = data.read_f32()?;
        self.blue = data.read_f32()?;
        self.requires_infield_lodding = data.read_bool()?;
        self.random_rotatable = data.read_bool()?;
        self.custom_alpha_blend_texture = data.read_sized_string_u8()?;
        self.scalable = data.read_bool()?;
        self.encampable = data.read_bool()?;
        self.custom_blend_tile = data.read_sized_string_u8()?;

        self.texture_red = data.read_sized_string_u8()?;
        self.texture_green = data.read_sized_string_u8()?;
        self.texture_blue = data.read_sized_string_u8()?;
        self.texture_alpha = data.read_sized_string_u8()?;

        for _ in 0..data.read_u32()? {
            self.variations.push(TileVariation::decode(data, extra_data)?);
        }

        for _ in 0..data.read_u32()? {
            self.link_targets.push(TileLinkTarget::decode(data, extra_data)?);
        }

        for _ in 0..data.read_u32()? {
            self.links.push(TileLink::decode(data, extra_data)?);
        }

        Ok(())
    }

    pub(crate) fn write_v3<W: WriteBytes>(&mut self, buffer: &mut W, extra_data: &Option<EncodeableExtraData>) -> Result<()> {
        buffer.write_sized_string_u8(&self.location)?;
        buffer.write_sized_string_u8(&self.tile_set)?;
        buffer.write_sized_string_u8(&self.mask)?;
        buffer.write_u32(self.width)?;
        buffer.write_u32(self.height)?;
        buffer.write_f32(self.red)?;
        buffer.write_f32(self.green)?;
        buffer.write_f32(self.blue)?;
        buffer.write_bool(self.requires_infield_lodding)?;
        buffer.write_bool(self.random_rotatable)?;
        buffer.write_sized_string_u8(&self.custom_alpha_blend_texture)?;
        buffer.write_bool(self.scalable)?;
        buffer.write_bool(self.encampable)?;
        buffer.write_sized_string_u8(&self.custom_blend_tile)?;

        buffer.write_sized_string_u8(&self.texture_red)?;
        buffer.write_sized_string_u8(&self.texture_green)?;
        buffer.write_sized_string_u8(&self.texture_blue)?;
        buffer.write_sized_string_u8(&self.texture_alpha)?;

        buffer.write_u32(self.variations.len() as u32)?;
        for var in self.variations_mut() {
            var.encode(buffer, extra_data)?;
        }

        buffer.write_u32(self.link_targets.len() as u32)?;
        for lt in self.link_targets_mut() {
            lt.encode(buffer, extra_data)?;
        }

        buffer.write_u32(self.links.len() as u32)?;
        for link in self.links_mut() {
            link.encode(buffer, extra_data)?;
        }

        Ok(())
    }
}
