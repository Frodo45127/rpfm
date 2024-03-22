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

impl Event {

    pub(crate) fn read_v122<R: ReadBytes>(data: &mut R) -> Result<Self> {
        let element_count = data.read_u32()?;dbg!(element_count);
        let mut values = vec![];
        for _ in 0..element_count {
            let data = data.read_u32()?;
            values.push(data);
        }

        Ok(Self {
            values
        })
    }

    pub(crate) fn write_v122<W: WriteBytes>(&self, buffer: &mut W) -> Result<()> {
        buffer.write_u32(self.values.len() as u32)?;

        for value in self.values() {
            buffer.write_u32(*value)?;
        }

        Ok(())
    }
}
