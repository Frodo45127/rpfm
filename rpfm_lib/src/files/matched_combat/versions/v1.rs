//---------------------------------------------------------------------------//
// Copyright (c) 2017-2023 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! This is a module to read/write binary Matched Combat files, v1.
//!
//! For internal use only.

use crate::error::Result;
use crate::binary::{ReadBytes, WriteBytes};
use crate::files::matched_combat::*;

//---------------------------------------------------------------------------//
//                            Implementation
//---------------------------------------------------------------------------//

impl MatchedCombat {

    pub fn read_v1<R: ReadBytes>(&mut self, data: &mut R) -> Result<()> {
        let count = data.read_u32()?;

        for _ in 0..count {
            let entry = Entry {
                uk1: data.read_u32()?,
                uk2: data.read_u32()?,
                uk3: data.read_u32()?,
                uk4: data.read_u32()?,
                uk5: data.read_u32()?,
                uk6: data.read_u32()?,
                str1: data.read_sized_string_u8()?,
                str2: data.read_sized_string_u8()?,
                uk21: data.read_u32()?,
                uk22: data.read_u32()?,
                uk23: data.read_u32()?,
                uk24: data.read_u32()?,
                uk25: data.read_u32()?,
                str21: data.read_sized_string_u8()?,
                str22: data.read_sized_string_u8()?,
            };

            self.entries.push(entry);
        }

        Ok(())
    }

    pub fn write_v1<W: WriteBytes>(&self, buffer: &mut W) -> Result<()> {
        buffer.write_u32(self.entries.len() as u32)?;
        for entry in &self.entries {
            buffer.write_u32(entry.uk1)?;
            buffer.write_u32(entry.uk2)?;
            buffer.write_u32(entry.uk3)?;
            buffer.write_u32(entry.uk4)?;
            buffer.write_u32(entry.uk5)?;
            buffer.write_u32(entry.uk6)?;
            buffer.write_sized_string_u8(&entry.str1)?;
            buffer.write_sized_string_u8(&entry.str2)?;
            buffer.write_u32(entry.uk21)?;
            buffer.write_u32(entry.uk22)?;
            buffer.write_u32(entry.uk23)?;
            buffer.write_u32(entry.uk24)?;
            buffer.write_u32(entry.uk25)?;
            buffer.write_sized_string_u8(&entry.str21)?;
            buffer.write_sized_string_u8(&entry.str22)?;
        }

        Ok(())
    }
}

