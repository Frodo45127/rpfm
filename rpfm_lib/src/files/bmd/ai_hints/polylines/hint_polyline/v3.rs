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
//                 Implementation of HintPolyline
//---------------------------------------------------------------------------//

impl HintPolyline {

    pub(crate) fn read_v3<R: ReadBytes>(&mut self, data: &mut R, extra_data: &Option<DecodeableExtraData>) -> Result<()> {
        self.rtype = data.read_sized_string_u8()?;

        for _ in 0..data.read_u32()? {
            self.points.push(Point2d::decode(data, extra_data)?);
        }

        self.script_id = data.read_sized_string_u8()?;
        self.only_vanguard = data.read_bool()?;

        Ok(())
    }

    pub(crate) fn write_v3<W: WriteBytes>(&mut self, buffer: &mut W, extra_data: &Option<EncodeableExtraData>) -> Result<()> {
        buffer.write_sized_string_u8(&self.rtype)?;

        buffer.write_u32(self.points.len() as u32)?;
        for point in &mut self.points {
            point.encode(buffer, extra_data)?;
        }

        buffer.write_sized_string_u8(&self.script_id)?;
        buffer.write_bool(self.only_vanguard)?;

        Ok(())
    }
}
