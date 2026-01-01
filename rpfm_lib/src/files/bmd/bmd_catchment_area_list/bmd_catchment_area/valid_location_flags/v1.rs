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
//                           Implementation of Text
//---------------------------------------------------------------------------//

impl ValidLocationFlags {

    pub(crate) fn read_v1<R: ReadBytes>(&mut self, data: &mut R, _extra_data: &Option<DecodeableExtraData>) -> Result<()> {
        self.valid_north = data.read_bool()?;
        self.valid_south = data.read_bool()?;
        self.valid_east = data.read_bool()?;
        self.valid_west = data.read_bool()?;

        Ok(())
    }

    pub(crate) fn write_v1<W: WriteBytes>(&mut self, buffer: &mut W, _extra_data: &Option<EncodeableExtraData>) -> Result<()> {
        buffer.write_bool(self.valid_north)?;
        buffer.write_bool(self.valid_south)?;
        buffer.write_bool(self.valid_east)?;
        buffer.write_bool(self.valid_west)?;

        Ok(())
    }
}
