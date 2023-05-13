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
//                    Implementation of LightProbe
//---------------------------------------------------------------------------//

impl LightProbe {

    pub(crate) fn read_v3<R: ReadBytes>(&mut self, data: &mut R, extra_data: &Option<DecodeableExtraData>) -> Result<()> {
        self.position = Point3d::decode(data, extra_data)?;
        self.outer_radius = data.read_f32()?;
        self.inner_radius = data.read_f32()?;
        self.is_cylinder = data.read_bool()?;
        self.is_primary = data.read_bool()?;
        self.height_mode = data.read_sized_string_u8()?;

        Ok(())
    }

    pub(crate) fn write_v3<W: WriteBytes>(&mut self, buffer: &mut W, extra_data: &Option<EncodeableExtraData>) -> Result<()> {
        self.position.encode(buffer, extra_data)?;
        buffer.write_f32(self.outer_radius)?;
        buffer.write_f32(self.inner_radius)?;
        buffer.write_bool(self.is_cylinder)?;
        buffer.write_bool(self.is_primary)?;
        buffer.write_sized_string_u8(&self.height_mode)?;

        Ok(())
    }
}
