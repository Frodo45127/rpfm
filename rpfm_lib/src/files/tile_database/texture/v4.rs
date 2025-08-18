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
//                           Implementation of Texture
//---------------------------------------------------------------------------//

impl Texture {

    pub(crate) fn read_v4<R: ReadBytes>(&mut self, data: &mut R, _extra_data: &Option<DecodeableExtraData>) -> Result<()> {
        self.name = data.read_sized_string_u8()?;
        self.mid_distance_strength = data.read_f32()?;
        self.lerp_reflectivity0 = data.read_f32()?;
        self.lerp_smoothness0 = data.read_f32()?;
        self.lerp_reflectivity1 = data.read_f32()?;
        self.lerp_smoothness1 = data.read_f32()?;
        self.lerp_reflectivity2 = data.read_f32()?;
        self.lerp_smoothness2 = data.read_f32()?;
        self.lerp_reflectivity3 = data.read_f32()?;
        self.lerp_smoothness3 = data.read_f32()?;
        self.blend_pixel_scale = data.read_f32()?;
        self.outfield_blend_pixel_scale = data.read_f32()?;

        Ok(())
    }

    pub(crate) fn write_v4<W: WriteBytes>(&mut self, buffer: &mut W, _extra_data: &Option<EncodeableExtraData>) -> Result<()> {
        buffer.write_sized_string_u8(&self.name)?;
        buffer.write_f32(self.mid_distance_strength)?;
        buffer.write_f32(self.lerp_reflectivity0)?;
        buffer.write_f32(self.lerp_smoothness0)?;
        buffer.write_f32(self.lerp_reflectivity1)?;
        buffer.write_f32(self.lerp_smoothness1)?;
        buffer.write_f32(self.lerp_reflectivity2)?;
        buffer.write_f32(self.lerp_smoothness2)?;
        buffer.write_f32(self.lerp_reflectivity3)?;
        buffer.write_f32(self.lerp_smoothness3)?;
        buffer.write_f32(self.blend_pixel_scale)?;
        buffer.write_f32(self.outfield_blend_pixel_scale)?;

        Ok(())
    }
}
