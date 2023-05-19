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

use super::*;

//---------------------------------------------------------------------------//
//                           Implementation of PointLight
//---------------------------------------------------------------------------//

impl PointLight {

    pub(crate) fn read_v5<R: ReadBytes>(&mut self, data: &mut R, extra_data: &Option<DecodeableExtraData>) -> Result<()> {
        self.position = Point3d::decode(data, extra_data)?;
        self.radius = data.read_f32()?;
        self.colour = ColourRGB::decode(data, extra_data)?;
        self.colour_scale = data.read_f32()?;
        self.animation_type = data.read_u8()?;
        self.params = Point2d::decode(data, extra_data)?;
        self.colour_min = data.read_f32()?;
        self.random_offset = data.read_f32()?;
        self.falloff_type = data.read_sized_string_u8()?;

        // TODO: How the fuck do we get a 194 here?!!! It's supposed to be a boolean.
        self.lf_relative = data.read_u8()?;
        self.height_mode = data.read_sized_string_u8()?;
        self.light_probes_only = data.read_bool()?;
        self.pdlc_mask = data.read_u32()? as u64;

        Ok(())
    }

    pub(crate) fn write_v5<W: WriteBytes>(&mut self, buffer: &mut W, extra_data: &Option<EncodeableExtraData>) -> Result<()> {
        self.position.encode(buffer, extra_data)?;
        buffer.write_f32(self.radius)?;
        self.colour.encode(buffer, extra_data)?;
        buffer.write_f32(self.colour_scale)?;
        buffer.write_u8(self.animation_type)?;
        self.params.encode(buffer, extra_data)?;
        buffer.write_f32(self.colour_min)?;
        buffer.write_f32(self.random_offset)?;

        buffer.write_sized_string_u8(&self.falloff_type)?;
        buffer.write_u8(self.lf_relative)?;
        buffer.write_sized_string_u8(&self.height_mode)?;
        buffer.write_bool(self.light_probes_only)?;
        buffer.write_u32(self.pdlc_mask as u32)?;

        Ok(())
    }
}

