//---------------------------------------------------------------------------//
// Copyright (c) 2017-2023 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Matched Combat files are tables containing data about matched animations between units.

use getset::{Getters, Setters};
use serde_derive::{Serialize, Deserialize};

use crate::binary::{ReadBytes, WriteBytes};
use crate::error::{RLibError, Result};
use crate::files::{DecodeableExtraData, Decodeable, EncodeableExtraData, Encodeable};
use crate::utils::check_size_mismatch;

/// Matched combat files go under these folders.
pub const BASE_PATHS: [&str; 2] = ["animations/matched_combat", "animations/database/matched"];

/// Extension of MatchedCombat files.
pub const EXTENSION: &str = ".bin";

mod versions;

#[cfg(test)] mod matched_combat_test;

//---------------------------------------------------------------------------//
//                              Enum & Structs
//---------------------------------------------------------------------------//

/// This stores the data of a decoded matched combat file in memory.
#[derive(PartialEq, Clone, Debug, Default, Getters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", set = "pub")]
pub struct MatchedCombat {
    version: u32,
    entries: Vec<Entry>,
}

#[derive(PartialEq, Clone, Debug, Default, Getters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", set = "pub")]
pub struct Entry {
    uk1: u32,
    uk2: u32,
    uk3: u32,
    uk4: u32,
    uk5: u32,
    uk6: u32,
    str1: String,
    str2: String,
    uk21: u32,
    uk22: u32,
    uk23: u32,
    uk24: u32,
    uk25: u32,
    str21: String,
    str22: String,
}

//---------------------------------------------------------------------------//
//                      Implementation of MatchedCombat
//---------------------------------------------------------------------------//

impl Decodeable for MatchedCombat {

    fn decode<R: ReadBytes>(data: &mut R, _extra_data: &Option<DecodeableExtraData>) -> Result<Self> {
        let version = data.read_u32()?;

        let mut matched = Self::default();
        matched.version = version;

        match version {
            1 => matched.read_v1(data)?,
            _ => Err(RLibError::DecodingMatchedCombatUnsupportedVersion(version as usize))?,
        }

        // If we are not in the last byte, it means we didn't parse the entire file, which means this file is corrupt.
        check_size_mismatch(data.stream_position()? as usize, data.len()? as usize)?;

        Ok(matched)
    }
}

impl Encodeable for MatchedCombat {

    fn encode<W: WriteBytes>(&mut self, buffer: &mut W, _extra_data: &Option<EncodeableExtraData>) -> Result<()> {
        buffer.write_u32(self.version)?;

        match self.version {
            1 => self.write_v1(buffer)?,
            _ => Err(RLibError::DecodingMatchedCombatUnsupportedVersion(self.version as usize))?,
        };

        Ok(())
    }
}
