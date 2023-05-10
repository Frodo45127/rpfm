//---------------------------------------------------------------------------//
// Copyright (c) 2017-2023 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

use getset::*;
use serde_derive::{Serialize, Deserialize};

use crate::binary::{ReadBytes, WriteBytes};
use crate::error::{Result, RLibError};
use crate::files::{Decodeable, EncodeableExtraData, Encodeable};

use self::flags::Flags;
use self::transform::Transform;

use super::*;

mod flags;
mod transform;
mod v25;

//---------------------------------------------------------------------------//
//                              Enum & Structs
//---------------------------------------------------------------------------//

#[derive(Default, PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct Prop {
    serialise_version: u16,
    key_index: u32,

    transform: Transform,

    decal: bool,
    logic_decal: bool,
    is_fauna: bool,
    snow_inside: bool,
    snow_outside: bool,
    destruction_inside: bool,
    destruction_outside: bool,
    animated: bool,
    decal_parallax_scale: f32,
    decal_tiling: f32,
    decal_override_gbuffer_normal: bool,

    flags: Flags,

    visible_in_shroud: bool,
    decal_apply_to_terrain: bool,
    decal_apply_to_gbuffer_objects: bool,
    decal_render_above_snow: bool,
    height_mode: String,
    pdlc_mask: f64,                     // FIx this nan.
    cast_shadows: bool,
    no_culling: bool,
    has_height_patch: bool,
    apply_height_patch: bool,
    include_in_fog: bool,
    visible_without_shroud: bool,
    use_dynamic_shadows: bool,
    uses_terrain_vertex_offset: bool,
}

//---------------------------------------------------------------------------//
//                           Implementation of Text
//---------------------------------------------------------------------------//

impl Decodeable for Prop {

    fn decode<R: ReadBytes>(data: &mut R, extra_data: &Option<DecodeableExtraData>) -> Result<Self> {
        let mut prop = Self::default();

        prop.serialise_version = data.read_u16()?;

        match prop.serialise_version {
            25 => prop.read_v25(data, extra_data)?,
            _ => return Err(RLibError::DecodingFastBinUnsupportedVersion(String::from("Prop"), prop.serialise_version)),
        }

        Ok(prop)
    }
}

impl Encodeable for Prop {

    fn encode<W: WriteBytes>(&mut self, buffer: &mut W, extra_data: &Option<EncodeableExtraData>) -> Result<()> {
        buffer.write_u16(self.serialise_version)?;

        match self.serialise_version {
            25 => self.write_v25(buffer, extra_data)?,
            _ => return Err(RLibError::EncodingFastBinUnsupportedVersion(String::from("Prop"), self.serialise_version)),
        }

        Ok(())
    }
}
