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
use crate::files::Decodeable;

use self::flags::Flags;
use self::transform::Transform;

use super::*;

//---------------------------------------------------------------------------//
//                           Implementation of Text
//---------------------------------------------------------------------------//

impl Prop {

    pub(crate) fn read_v25<R: ReadBytes>(&mut self, data: &mut R, extra_data: &Option<DecodeableExtraData>) -> Result<()> {
        self.key_index = data.read_u32()?;
        self.transform = Transform::decode(data, extra_data)?;
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
        self.pdlc_mask = data.read_f64()?;
        self.cast_shadows = data.read_bool()?;
        self.no_culling = data.read_bool()?;
        self.has_height_patch = data.read_bool()?;
        self.apply_height_patch = data.read_bool()?;
        self.include_in_fog = data.read_bool()?;
        self.visible_without_shroud = data.read_bool()?;
        self.use_dynamic_shadows = data.read_bool()?;
        self.uses_terrain_vertex_offset = data.read_bool()?;

        Ok(())
    }
}
