//---------------------------------------------------------------------------//
// Copyright (c) 2017-2023 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

use getset::{Getters, Setters};
use serde_derive::{Serialize, Deserialize};

use crate::binary::{ReadBytes, WriteBytes};
use crate::error::{RLibError, Result};
use crate::files::{DecodeableExtraData, Decodeable, EncodeableExtraData, Encodeable};
use crate::utils::check_size_mismatch;

pub const BASE_PATH: &str = "animations/";

/// To differentiate them from other bin tables, this lib only recognizes files ending in _tables.bin as AnimsTable.
/// This is only for this lib, not a limitation of the game.
pub const EXTENSION: &str = "_tables.bin";

mod versions;

#[cfg(test)] mod anims_table_test;

//---------------------------------------------------------------------------//
//                              Enum & Structs
//---------------------------------------------------------------------------//

#[derive(PartialEq, Clone, Debug, Default, Getters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", set = "pub")]
pub struct AnimsTable {
    version: u32,
    entries: Vec<Entry>,
}

#[derive(PartialEq, Clone, Debug, Default, Getters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", set = "pub")]
pub struct Entry {
    table_name: String,
    skeleton_type: String,
    mount_table_name: String,
    fragments: Vec<Fragment>,
    uk_6: bool,
    uk_7: bool,
}

#[derive(PartialEq, Clone, Debug, Default, Getters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", set = "pub")]
pub struct Fragment {
    name: String,
    uk_5: u32,
}

//---------------------------------------------------------------------------//
//                      Implementation of AnimsTable
//---------------------------------------------------------------------------//

impl Decodeable for AnimsTable {

    fn decode<R: ReadBytes>(data: &mut R, _extra_data: &Option<DecodeableExtraData>) -> Result<Self> {
        let mut table = Self::default();
        table.version = data.read_u32()?;

        match table.version {
            2 => table.read_v2(data)?,
            _ => Err(RLibError::DecodingMatchedCombatUnsupportedVersion(table.version as usize))?,
        }

        // If we are not in the last byte, it means we didn't parse the entire file, which means this file is corrupt.
        check_size_mismatch(data.stream_position()? as usize, data.len()? as usize)?;

        Ok(table)
    }
}

impl Encodeable for AnimsTable {

    fn encode<W: WriteBytes>(&mut self, buffer: &mut W, _extra_data: &Option<EncodeableExtraData>) -> Result<()> {
        buffer.write_u32(self.version)?;

        match self.version {
            2 => self.write_v2(buffer)?,
            _ => Err(RLibError::DecodingAnimFragmentUnsupportedVersion(self.version as usize))?,
        };

        Ok(())
    }
}
