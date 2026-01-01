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
//                    Implementation of SpotLight
//---------------------------------------------------------------------------//

impl SpotLight {

    pub(crate) fn read_v7<R: ReadBytes>(&mut self, data: &mut R, extra_data: &Option<DecodeableExtraData>) -> Result<()> {
        self.position = Point3d::decode(data, extra_data)?;
        self.end = Quaternion::decode(data, extra_data)?;
        self.length = data.read_f32()?;
        self.inner_angle = data.read_f32()?;
        self.outer_angle = data.read_f32()?;
        self.colour = ColourRGB::decode(data, extra_data)?;
        self.falloff = data.read_f32()?;
        self.gobo = data.read_sized_string_u8()?;
        self.volumetric = data.read_bool()?;
        self.height_mode = data.read_sized_string_u8()?;
        self.pdlc_version = data.read_u64()?;

        Ok(())
    }

    pub(crate) fn write_v7<W: WriteBytes>(&mut self, buffer: &mut W, extra_data: &Option<EncodeableExtraData>) -> Result<()> {
        self.position.encode(buffer, extra_data)?;
        self.end.encode(buffer, extra_data)?;
        buffer.write_f32(self.length)?;
        buffer.write_f32(self.inner_angle)?;
        buffer.write_f32(self.outer_angle)?;
        self.colour.encode(buffer, extra_data)?;
        buffer.write_f32(self.falloff)?;
        buffer.write_sized_string_u8(&self.gobo)?;
        buffer.write_bool(self.volumetric)?;
        buffer.write_sized_string_u8(&self.height_mode)?;
        buffer.write_u64(self.pdlc_version)?;

        Ok(())
    }
}
