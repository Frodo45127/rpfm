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
//                           Implementation of Flags
//---------------------------------------------------------------------------//

impl Flags {

    pub(crate) fn read_v3<R: ReadBytes>(&mut self, data: &mut R, _extra_data: &Option<DecodeableExtraData>) -> Result<()> {
        self.allow_in_outfield = data.read_bool()?;
        self.clamp_to_water_surface = data.read_bool()?;
        self.spring = data.read_bool()?;
        self.summer = data.read_bool()?;
        self.autumn = data.read_bool()?;
        self.winter = data.read_bool()?;

        Ok(())
    }

    pub(crate) fn write_v3<W: WriteBytes>(&mut self, buffer: &mut W, _extra_data: &Option<EncodeableExtraData>) -> Result<()> {
        buffer.write_bool(self.allow_in_outfield)?;
        buffer.write_bool(self.clamp_to_water_surface)?;
        buffer.write_bool(self.spring)?;
        buffer.write_bool(self.summer)?;
        buffer.write_bool(self.autumn)?;
        buffer.write_bool(self.winter)?;

        Ok(())
    }
}
