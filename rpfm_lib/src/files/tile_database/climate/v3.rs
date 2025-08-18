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
//                       Implementation of Climate
//---------------------------------------------------------------------------//

impl Climate {

    pub(crate) fn read_v3<R: ReadBytes>(&mut self, data: &mut R, extra_data: &Option<DecodeableExtraData>) -> Result<()> {
        self.name = data.read_sized_string_u8()?;
        self.texture_set = data.read_sized_string_u8()?;
        self.red = data.read_f32()?;
        self.green = data.read_f32()?;
        self.blue = data.read_f32()?;

        for _ in 0..data.read_u32()? {
            self.textures.push(Texture::decode(data, extra_data)?);
        }

        self.grass_alpha_add = data.read_f32()?;
        self.grass_alpha_mul = data.read_f32()?;
        self.destruction_climate = data.read_sized_string_u8()?;

        Ok(())
    }

    pub(crate) fn write_v3<W: WriteBytes>(&mut self, buffer: &mut W, extra_data: &Option<EncodeableExtraData>) -> Result<()> {
        buffer.write_sized_string_u8(&self.name)?;
        buffer.write_sized_string_u8(&self.texture_set)?;
        buffer.write_f32(self.red)?;
        buffer.write_f32(self.green)?;
        buffer.write_f32(self.blue)?;

        buffer.write_u32(self.textures.len() as u32)?;
        for texture in self.textures_mut() {
            texture.encode(buffer, extra_data)?;
        }

        buffer.write_f32(self.grass_alpha_add)?;
        buffer.write_f32(self.grass_alpha_mul)?;
        buffer.write_sized_string_u8(&self.destruction_climate)?;

        Ok(())
    }
}
