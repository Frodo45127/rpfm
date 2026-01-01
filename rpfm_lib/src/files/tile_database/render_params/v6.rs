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
//                           Implementation of RenderParams
//---------------------------------------------------------------------------//

impl RenderParams {

    pub(crate) fn read_v6<R: ReadBytes>(&mut self, data: &mut R, _extra_data: &Option<DecodeableExtraData>) -> Result<()> {
        self.lf_height = data.read_f32()?;
        self.hf_height = data.read_f32()?;
        self.blend_pixel_scale = data.read_f32()?;
        self.unit_scale = data.read_f32()?;
        self.mid_distance_detail_scale = data.read_f32()?;
        self.mid_distance_detail_strength = data.read_f32()?;
        self.mid_distance_normal_strength = data.read_f32()?;
        self.mid_distance_detail_near = data.read_f32()?;
        self.mid_distance_detail_far = data.read_f32()?;
        self.mid_distance_detail_slope_low = data.read_f32()?;
        self.mid_distance_detail_slope_high = data.read_f32()?;
        self.vertical_offset = data.read_f32()?;
        self.normal_lf_scale = data.read_f32()?;
        self.normal_tile_scale = data.read_f32()?;
        self.normal_terrain_scale = data.read_f32()?;
        self.blend_contrast = data.read_f32()?;
        self.layer_exempt_0 = data.read_sized_string_u8()?;
        self.layer_exempt_1 = data.read_sized_string_u8()?;
        self.layer_exempt_2 = data.read_sized_string_u8()?;
        self.layer_exempt_3 = data.read_sized_string_u8()?;
        self.campaign_sea_transparency_scale = data.read_f32()?;
        self.campaign_sea_uv_scale = data.read_f32()?;
        self.campaign_lake_transparency_scale = data.read_f32()?;
        self.near_distance_detail_distance = data.read_f32()?;
        self.near_distance_detail_scale = data.read_f32()?;
        self.near_distance_detail_strength = data.read_f32()?;
        self.mid_distance_detail_near_lq = data.read_f32()?;

        Ok(())
    }

    pub(crate) fn write_v6<W: WriteBytes>(&mut self, buffer: &mut W, _extra_data: &Option<EncodeableExtraData>) -> Result<()> {
        buffer.write_f32(self.lf_height)?;
        buffer.write_f32(self.hf_height)?;
        buffer.write_f32(self.blend_pixel_scale)?;
        buffer.write_f32(self.unit_scale)?;
        buffer.write_f32(self.mid_distance_detail_scale)?;
        buffer.write_f32(self.mid_distance_detail_strength)?;
        buffer.write_f32(self.mid_distance_normal_strength)?;
        buffer.write_f32(self.mid_distance_detail_near)?;
        buffer.write_f32(self.mid_distance_detail_far)?;
        buffer.write_f32(self.mid_distance_detail_slope_low)?;
        buffer.write_f32(self.mid_distance_detail_slope_high)?;
        buffer.write_f32(self.vertical_offset)?;
        buffer.write_f32(self.normal_lf_scale)?;
        buffer.write_f32(self.normal_tile_scale)?;
        buffer.write_f32(self.normal_terrain_scale)?;
        buffer.write_f32(self.blend_contrast)?;
        buffer.write_sized_string_u8(&self.layer_exempt_0)?;
        buffer.write_sized_string_u8(&self.layer_exempt_1)?;
        buffer.write_sized_string_u8(&self.layer_exempt_2)?;
        buffer.write_sized_string_u8(&self.layer_exempt_3)?;
        buffer.write_f32(self.campaign_sea_transparency_scale)?;
        buffer.write_f32(self.campaign_sea_uv_scale)?;
        buffer.write_f32(self.campaign_lake_transparency_scale)?;
        buffer.write_f32(self.near_distance_detail_distance)?;
        buffer.write_f32(self.near_distance_detail_scale)?;
        buffer.write_f32(self.near_distance_detail_strength)?;
        buffer.write_f32(self.mid_distance_detail_near_lq)?;

        Ok(())
    }
}
