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
//                 Implementation of HintPolyline
//---------------------------------------------------------------------------//

impl HintPolyline {

    pub(crate) fn read_v4<R: ReadBytes>(&mut self, data: &mut R, _extra_data: &Option<DecodeableExtraData>) -> Result<()> {
        self.rtype = data.read_sized_string_u8()?;

        for _ in 0..data.read_u32()? {
            self.points.push(Point {
                x: data.read_f32()?,
                y: data.read_f32()?,
            });
        }

        self.script_id = data.read_sized_string_u8()?;
        self.only_vanguard = data.read_bool()?;
        self.only_deploy_when_clear = data.read_bool()?;
        self.spawn_vfx = data.read_bool()?;

        Ok(())
    }

    pub(crate) fn write_v4<W: WriteBytes>(&mut self, buffer: &mut W, _extra_data: &Option<EncodeableExtraData>) -> Result<()> {
        buffer.write_sized_string_u8(&self.rtype)?;

        buffer.write_u32(self.points.len() as u32)?;
        for point in &self.points {
            buffer.write_f32(point.x)?;
            buffer.write_f32(point.y)?;
        }

        buffer.write_sized_string_u8(&self.script_id)?;
        buffer.write_bool(self.only_vanguard)?;
        buffer.write_bool(self.only_deploy_when_clear)?;
        buffer.write_bool(self.spawn_vfx)?;

        Ok(())
    }
}
