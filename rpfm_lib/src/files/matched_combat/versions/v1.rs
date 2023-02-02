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
            let entry_count = data.read_u32()?;
            let mut matched_entries = vec![];

            for _ in 0..entry_count {
                let entry = Entry {
                    uk1: data.read_u32()?,      // Team?
                    uk2: data.read_u32()?,      // Start status (0: alive, 1: dead)?
                    uk3: data.read_u32()?,      // Ends status (0: alive, 1: dead)?
                    uk4: data.read_u32()?,
                    uk5: data.read_u32()?,
                    file_path: data.read_sized_string_u8()?,
                    mount_file_path: data.read_sized_string_u8()?,
                };

                matched_entries.push(entry);
            }
            self.entries.push(matched_entries);
        }

        Ok(())
    }

    pub fn write_v1<W: WriteBytes>(&self, buffer: &mut W) -> Result<()> {
        buffer.write_u32(self.entries.len() as u32)?;
        for entry in &self.entries {
            buffer.write_u32(entry.len() as u32)?;
            for matched_entry in entry {
                buffer.write_u32(matched_entry.uk1)?;
                buffer.write_u32(matched_entry.uk2)?;
                buffer.write_u32(matched_entry.uk3)?;
                buffer.write_u32(matched_entry.uk4)?;
                buffer.write_u32(matched_entry.uk5)?;
                buffer.write_sized_string_u8(&matched_entry.file_path)?;
                buffer.write_sized_string_u8(&matched_entry.mount_file_path)?;
            }
        }

        Ok(())
    }
}

