//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Version 2 implementation of animation table binary format.
//!
//! This module handles reading and writing animation table files in version 2 format,
//! used in modern Total War games (Warhammer 2, Three Kingdoms, Warhammer 3).
//!
//! # Binary Format
//!
//! ```text
//! [u32] entry_count
//! For each entry:
//!   [u8 + string] table_name
//!   [u8 + string] skeleton_type
//!   [u8 + string] mount_table_name
//!   [u32] fragment_count
//!   For each fragment:
//!     [u8 + string] name
//!     [u32] uk_5 (unknown)
//!   [bool] uk_6 (unknown)
//!   [bool] uk_7 (unknown)
//! ```
//!
//! # String Encoding
//!
//! Strings are stored with a 1-byte length prefix followed by UTF-8 data
//! (`read_sized_string_u8`/`write_sized_string_u8`).
//!
//! # Usage
//!
//! This module is used internally by [`AnimsTable`] and should not be accessed
//! directly. Use [`Decodeable::decode()`] and [`Encodeable::encode()`] instead.

use crate::error::Result;
use crate::binary::{ReadBytes, WriteBytes};
use crate::files::anims_table::*;

//---------------------------------------------------------------------------//
//                            Implementation
//---------------------------------------------------------------------------//

impl AnimsTable {

    /// Reads version 2 animation table data from a binary stream.
    ///
    /// # Parameters
    ///
    /// - `data`: Binary reader implementing [`ReadBytes`]
    ///
    /// # Returns
    ///
    /// - `Ok(())`: Successfully read all entries and fragments
    /// - `Err(_)`: I/O error or malformed data
    ///
    /// # Binary Structure
    ///
    /// Reads:
    /// 1. Entry count (u32)
    /// 2. For each entry: table name, skeleton type, mount table, fragments, flags
    /// 3. For each fragment: name and metadata
    ///
    /// See module documentation for detailed binary format.
    pub(crate) fn read_v2<R: ReadBytes>(&mut self, data: &mut R) -> Result<()> {
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

    /// Writes version 2 animation table data to a binary stream.
    ///
    /// # Parameters
    ///
    /// - `buffer`: Binary writer implementing [`WriteBytes`]
    ///
    /// # Returns
    ///
    /// - `Ok(())`: Successfully wrote all entries and fragments
    /// - `Err(_)`: I/O error during writing
    ///
    /// # Binary Structure
    ///
    /// Writes:
    /// 1. Entry count (u32)
    /// 2. For each entry: table name, skeleton type, mount table, fragment list, flags
    /// 3. For each fragment: name and metadata
    ///
    /// See module documentation for detailed binary format.
    pub(crate) fn write_v2<W: WriteBytes>(&self, buffer: &mut W) -> Result<()> {
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
