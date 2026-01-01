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
//                       Implementation of ConversionParams
//---------------------------------------------------------------------------//

impl ConversionParams {

    pub(crate) fn read_v1<R: ReadBytes>(&mut self, data: &mut R, _extra_data: &Option<DecodeableExtraData>) -> Result<()> {
        self.triangle_density = data.read_i32()?;
        self.max_lf_heights_per_pixel = data.read_i32()?;
        self.triangle_decimation_size_factors0 = data.read_f32()?;
        self.triangle_decimation_size_factors1 = data.read_f32()?;
        self.triangle_decimation_size_factors2 = data.read_f32()?;
        self.triangle_decimation_size_factors3 = data.read_f32()?;
        self.triangle_decimation_size_factors4 = data.read_f32()?;
        self.triangle_decimation_size_factors5 = data.read_f32()?;
        self.triangle_decimation_angle_factors0 = data.read_f32()?;
        self.triangle_decimation_angle_factors1 = data.read_f32()?;
        self.triangle_decimation_angle_factors2 = data.read_f32()?;
        self.triangle_decimation_angle_factors3 = data.read_f32()?;
        self.triangle_decimation_angle_factors4 = data.read_f32()?;
        self.triangle_decimation_angle_factors5 = data.read_f32()?;

        Ok(())
    }

    pub(crate) fn write_v1<W: WriteBytes>(&mut self, buffer: &mut W, _extra_data: &Option<EncodeableExtraData>) -> Result<()> {
        buffer.write_i32(self.triangle_density)?;
        buffer.write_i32(self.max_lf_heights_per_pixel)?;
        buffer.write_f32(self.triangle_decimation_size_factors0)?;
        buffer.write_f32(self.triangle_decimation_size_factors1)?;
        buffer.write_f32(self.triangle_decimation_size_factors2)?;
        buffer.write_f32(self.triangle_decimation_size_factors3)?;
        buffer.write_f32(self.triangle_decimation_size_factors4)?;
        buffer.write_f32(self.triangle_decimation_size_factors5)?;
        buffer.write_f32(self.triangle_decimation_angle_factors0)?;
        buffer.write_f32(self.triangle_decimation_angle_factors1)?;
        buffer.write_f32(self.triangle_decimation_angle_factors2)?;
        buffer.write_f32(self.triangle_decimation_angle_factors3)?;
        buffer.write_f32(self.triangle_decimation_angle_factors4)?;
        buffer.write_f32(self.triangle_decimation_angle_factors5)?;

        Ok(())
    }
}
