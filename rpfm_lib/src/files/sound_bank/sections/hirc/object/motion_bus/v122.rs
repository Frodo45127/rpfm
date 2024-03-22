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

impl MotionBus {

    pub(crate) fn read_v122<R: ReadBytes>(data: &mut R, size: usize) -> Result<Self> {
        Ok(Self {
            data: data.read_slice(size, false)?,
        })
    }

    pub(crate) fn write_v122<W: WriteBytes>(&self, buffer: &mut W) -> Result<()> {
        buffer.write_all(&self.data)?;

        Ok(())
    }
}
