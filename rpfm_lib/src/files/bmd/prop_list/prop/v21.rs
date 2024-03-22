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
use crate::files::Decodeable;

use super::*;

//---------------------------------------------------------------------------//
//                           Implementation of Prop
//---------------------------------------------------------------------------//

impl Prop {

    pub(crate) fn read_v21<R: ReadBytes>(&mut self, data: &mut R, extra_data: &Option<DecodeableExtraData>) -> Result<()> {
        self.key_index = data.read_u32()?;
        self.transform = Transform3x4::decode(data, extra_data)?;
        self.decal = data.read_bool()?;
        self.logic_decal = data.read_bool()?;
        self.is_fauna = data.read_bool()?;
        self.snow_inside = data.read_bool()?;
        self.snow_outside = data.read_bool()?;
        self.destruction_inside = data.read_bool()?;
        self.destruction_outside = data.read_bool()?;
        self.animated = data.read_bool()?;
        self.decal_parallax_scale = data.read_f32()?;
        self.decal_tiling = data.read_f32()?;
        self.decal_override_gbuffer_normal = data.read_bool()?;
        self.flags = Flags::decode(data, extra_data)?;
        self.visible_in_shroud = data.read_bool()?;
        self.decal_apply_to_terrain = data.read_bool()?;
        self.decal_apply_to_gbuffer_objects = data.read_bool()?;
        self.decal_render_above_snow = data.read_bool()?;
        self.height_mode = data.read_sized_string_u8()?;
        self.pdlc_mask = data.read_u64()?;
        self.cast_shadows = data.read_bool()?;
        self.no_culling = data.read_bool()?;
        self.has_height_patch = data.read_bool()?;
        self.apply_height_patch = data.read_bool()?;
        self.include_in_fog = data.read_bool()?;
        self.visible_without_shroud = data.read_bool()?;
        self.use_dynamic_shadows = data.read_bool()?;

        Ok(())
    }

    pub(crate) fn write_v21<W: WriteBytes>(&mut self, buffer: &mut W, extra_data: &Option<EncodeableExtraData>) -> Result<()> {
        buffer.write_u32(self.key_index)?;

        self.transform.encode(buffer, extra_data)?;

        buffer.write_bool(self.decal)?;
        buffer.write_bool(self.logic_decal)?;
        buffer.write_bool(self.is_fauna)?;
        buffer.write_bool(self.snow_inside)?;
        buffer.write_bool(self.snow_outside)?;
        buffer.write_bool(self.destruction_inside)?;
        buffer.write_bool(self.destruction_outside)?;
        buffer.write_bool(self.animated)?;
        buffer.write_f32(self.decal_parallax_scale)?;
        buffer.write_f32(self.decal_tiling)?;
        buffer.write_bool(self.decal_override_gbuffer_normal)?;

        self.flags.encode(buffer, extra_data)?;

        buffer.write_bool(self.visible_in_shroud)?;
        buffer.write_bool(self.decal_apply_to_terrain)?;
        buffer.write_bool(self.decal_apply_to_gbuffer_objects)?;
        buffer.write_bool(self.decal_render_above_snow)?;
        buffer.write_sized_string_u8(&self.height_mode)?;
        buffer.write_u64(self.pdlc_mask)?;
        buffer.write_bool(self.cast_shadows)?;
        buffer.write_bool(self.no_culling)?;
        buffer.write_bool(self.has_height_patch)?;
        buffer.write_bool(self.apply_height_patch)?;
        buffer.write_bool(self.include_in_fog)?;
        buffer.write_bool(self.visible_without_shroud)?;
        buffer.write_bool(self.use_dynamic_shadows)?;

        Ok(())
    }
}
