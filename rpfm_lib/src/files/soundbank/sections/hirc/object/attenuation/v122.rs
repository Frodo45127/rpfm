//---------------------------------------------------------------------------//
// Copyright (c) 2017-2024 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

use crate::binary::{ReadBytes, WriteBytes};
use crate::error::Result;

use super::*;

//---------------------------------------------------------------------------//
//                              Implementation
//---------------------------------------------------------------------------//

impl Attenuation {

    pub(crate) fn read_v122<R: ReadBytes>(data: &mut R, version: u32) -> Result<Self> {
        let id = data.read_u32()?;

        let is_cone_enabled = if data.read_bool()? {
            Some((data.read_f32()?, data.read_f32()?, data.read_f32()?, data.read_f32()?, data.read_f32()?))
        } else { None };

        let mut curve_index = vec![];
        for _ in 0..NUM_CURVES {
            curve_index.push(data.read_u8()?);
        }

        let mut curves = vec![];
        for _ in 0..data.read_u8()? {
            curves.push(RTPCCurve::read(data, version)?);
        }

        let initial_rtpc = InitialRTPC::read(data, version)?;

        Ok(Self {
            id,
            is_cone_enabled,
            curve_index,
            curves,
            initial_rtpc,
        })
    }

    pub(crate) fn write_v122<W: WriteBytes>(&self, buffer: &mut W, version: u32) -> Result<()> {
        buffer.write_u32(self.id)?;

        buffer.write_bool(self.is_cone_enabled.is_some())?;

        if let Some((f_inside_degrees, f_outside_degrees, f_outside_volume, lo_pass, hi_pass)) = self.is_cone_enabled {
            buffer.write_f32(f_inside_degrees)?;
            buffer.write_f32(f_outside_degrees)?;
            buffer.write_f32(f_outside_volume)?;
            buffer.write_f32(lo_pass)?;
            buffer.write_f32(hi_pass)?;
        }

        for index in self.curve_index() {
            buffer.write_u8(*index)?;
        }

        buffer.write_u8(self.curve_index().len() as u8)?;
        for curve in self.curves() {
            curve.write(buffer, version)?;
        }

        self.initial_rtpc.write(buffer, version)?;

        Ok(())
    }
}
