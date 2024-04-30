//---------------------------------------------------------------------------//
// Copyright (c) 2017-2024 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! This is a module to read/write sound_bank_database files from Shogun 2, Napoleon and Empire.
//!
//! These files are NOT VERSIONED, so we only support the latest one per-game.

use getset::*;
use serde_derive::{Serialize, Deserialize};

use crate::binary::{ReadBytes, WriteBytes};
use crate::error::{Result, RLibError};
use crate::files::{DecodeableExtraData, Decodeable, EncodeableExtraData, Encodeable};
use crate::games::supported_games::KEY_SHOGUN_2;
use crate::utils::check_size_mismatch;

pub const PATH: &str = "sounds_packed/sound_bank_database";

mod games;

#[cfg(test)] mod sound_bank_database_test;

//---------------------------------------------------------------------------//
//                              Enum & Structs
//---------------------------------------------------------------------------//

#[derive(Default, PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct SoundBankDatabase {
}

//---------------------------------------------------------------------------//
//                              Implementations
//---------------------------------------------------------------------------//

impl Decodeable for SoundBankDatabase {

    fn decode<R: ReadBytes>(data: &mut R, extra_data: &Option<DecodeableExtraData>) -> Result<Self> {
        let extra_data = extra_data.as_ref().ok_or(RLibError::DecodingMissingExtraData)?;
        let game_key = extra_data.game_key.ok_or_else(|| RLibError::DecodingMissingExtraDataField("game_key".to_owned()))?;

        let mut sound_bank = Self::default();

        match game_key {
            KEY_SHOGUN_2 => sound_bank.read_sho2(data)?,
            //KEY_NAPOLEON => {},
            //KEY_EMPIRE => sound_bank.read_emp(data)?,
            _ => return Err(RLibError::DecodingSoundPackedUnsupportedGame(game_key.to_string())),
        }

        // If we are not in the last byte, it means we didn't parse the entire file, which means this file is corrupt.
        check_size_mismatch(data.stream_position()? as usize, data.len()? as usize)?;

        Ok(sound_bank)
    }
}

impl Encodeable for SoundBankDatabase {

    fn encode<W: WriteBytes>(&mut self, buffer: &mut W, extra_data: &Option<EncodeableExtraData>) -> Result<()> {
        let extra_data = extra_data.as_ref().ok_or(RLibError::EncodingMissingExtraData)?;
        let game_key = extra_data.game_key.ok_or_else(|| RLibError::DecodingMissingExtraDataField("game_key".to_owned()))?;

        match game_key {
            KEY_SHOGUN_2 => self.write_sho2(buffer),
            //KEY_NAPOLEON => {},
            //KEY_EMPIRE => self.write_emp(buffer),
            _ => Err(RLibError::EncodingSoundPackedUnsupportedGame(game_key.to_string())),
        }
    }
}
