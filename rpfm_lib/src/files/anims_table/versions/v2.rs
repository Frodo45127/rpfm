//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! This is a module to read/write binary Anims Table files, v2.
//!
//! For internal use only.

use crate::error::Result;
use crate::binary::{ReadBytes, WriteBytes};
use crate::files::anims_table::*;

//---------------------------------------------------------------------------//
//                            Implementation
//---------------------------------------------------------------------------//

impl AnimsTable {

    pub fn read_v2<R: ReadBytes>(&mut self, data: &mut R) -> Result<()> {
        let entry_count = data.read_u32()?;
        for _ in 0..entry_count {
            let table_name = data.read_sized_string_u8()?;
            let skeleton_type = data.read_sized_string_u8()?;
            let mount_table_name = data.read_sized_string_u8()?;

            let mut fragments = vec![];
            let entry_count = data.read_u32()?;
            for _ in 0..entry_count {
                let name = data.read_sized_string_u8()?;
                let uk_5 = data.read_u32()?;

                fragments.push(Fragment {
                    name,
                    uk_5
                });
            }
            let uk_6 = data.read_bool()?;
            let uk_7 = data.read_bool()?;

            self.entries.push(Entry {
                table_name,
                skeleton_type,
                mount_table_name,
                fragments,
                uk_6,
                uk_7,
            });
        }

        Ok(())
    }

    pub fn write_v2<W: WriteBytes>(&self, buffer: &mut W) -> Result<()> {
        buffer.write_u32(self.entries.len() as u32)?;
        for entry in &self.entries {
            buffer.write_sized_string_u8(&entry.table_name)?;
            buffer.write_sized_string_u8(&entry.skeleton_type)?;
            buffer.write_sized_string_u8(&entry.mount_table_name)?;

            buffer.write_u32(entry.fragments.len() as u32)?;
            for entry in &entry.fragments {
                buffer.write_sized_string_u8(&entry.name)?;
                buffer.write_u32(entry.uk_5)?;
            }

            buffer.write_bool(entry.uk_6)?;
            buffer.write_bool(entry.uk_7)?;
        }

        Ok(())
    }
}
