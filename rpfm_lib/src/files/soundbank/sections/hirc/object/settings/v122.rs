//---------------------------------------------------------------------------//
// Copyright (c) 2017-2023 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

use std::collections::HashMap;

use crate::binary::{ReadBytes, WriteBytes};
use crate::error::Result;

use super::*;

//---------------------------------------------------------------------------//
//                              Implementation
//---------------------------------------------------------------------------//

impl Settings {

    pub(crate) fn read_v122<R: ReadBytes>(data: &mut R) -> Result<Self> {
        let element_count = data.read_u8()?;dbg!(element_count);
        let mut settings = HashMap::new();
        for _ in 0..element_count {
            let index = data.read_u8()?;
            let data = data.read_f32()?;
            settings.insert(index, data);
        }

        Ok(Self {
            settings
        })
    }

    pub(crate) fn write_v122<W: WriteBytes>(&self, buffer: &mut W) -> Result<()> {
        buffer.write_u8(self.settings.len() as u8)?;

        for (index, value) in &self.settings {
            buffer.write_u8(*index)?;
            buffer.write_f32(*value)?;
        }

        Ok(())
    }
}
