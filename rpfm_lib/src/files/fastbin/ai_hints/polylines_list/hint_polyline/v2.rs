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

    pub(crate) fn read_v2<R: ReadBytes>(&mut self, data: &mut R, _extra_data: &Option<DecodeableExtraData>) -> Result<()> {
        self.rtype = data.read_sized_string_u8()?;
        self.district = data.read_u32()?;

        for _ in 0..data.read_u32()? {
            let mut polygon = Polygon::default();

            for _ in 0..data.read_u32()? {
                polygon.points.push(Point {
                    x: data.read_f32()?,
                    y: data.read_f32()?,
                });
            }

            self.polygons.push(polygon);
        }

        Ok(())
    }

    pub(crate) fn write_v2<W: WriteBytes>(&mut self, buffer: &mut W, _extra_data: &Option<EncodeableExtraData>) -> Result<()> {
        buffer.write_sized_string_u8(&self.rtype)?;
        buffer.write_u32(self.district)?;

        buffer.write_u32(self.polygons.len() as u32)?;
        for polygon in &self.polygons {
            buffer.write_u32(polygon.points.len() as u32)?;
            for point in &polygon.points {
                buffer.write_f32(point.x)?;
                buffer.write_f32(point.y)?;
            }
        }

        Ok(())
    }
}
