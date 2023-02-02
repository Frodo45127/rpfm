//---------------------------------------------------------------------------//
// Copyright (c) 2017-2023 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Anims tables are tables containing data about unit animations.

use getset::{Getters, Setters};
use serde_derive::{Serialize, Deserialize};

use std::borrow::Cow;
use std::collections::{BTreeMap, HashMap};

use crate::binary::{ReadBytes, WriteBytes};
use crate::error::{RLibError, Result};
use crate::files::{DecodeableExtraData, Decodeable, EncodeableExtraData, Encodeable};

use crate::utils::check_size_mismatch;

/// Base path of an animation table. This is an special type of bin, stored only in this folder.
pub const BASE_PATH: &str = "animations";

/// To differentiate them from other bin tables, this lib only recognizes files ending in _tables.bin as AnimsTable.
/// This is only for this lib, not a limitation of the game.
pub const EXTENSION: &str = "_tables.bin";

/// Size of the header of a MatchedCombat PackedFile.
pub const HEADER_SIZE: usize = 8;

#[cfg(test)] mod anims_table_test;

//---------------------------------------------------------------------------//
//                              Enum & Structs
//---------------------------------------------------------------------------//

#[derive(PartialEq, Clone, Debug, Default, Getters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", set = "pub")]
pub struct AnimsTable {

}

//---------------------------------------------------------------------------//
//                      Implementation of MatchedCombat
//---------------------------------------------------------------------------//

impl Decodeable for AnimsTable {

    fn decode<R: ReadBytes>(data: &mut R, _extra_data: &Option<DecodeableExtraData>) -> Result<Self> {
        let version = data.read_i32()?;
        let entry_count = data.read_u32()?;


        // If we are not in the last byte, it means we didn't parse the entire file, which means this file is corrupt.
        check_size_mismatch(data.stream_position()? as usize, data.len()? as usize)?;

        Ok(Self {})
    }
}

impl Encodeable for AnimsTable {

    fn encode<W: WriteBytes>(&mut self, buffer: &mut W, _extra_data: &Option<EncodeableExtraData>) -> Result<()> {
        //buffer.write_i32(*self.table.definition().version())?;
        //buffer.write_u32(self.table.len(None)? as u32)?;
        //self.table.encode(buffer, &None, &None)
        Ok(())
    }
}
