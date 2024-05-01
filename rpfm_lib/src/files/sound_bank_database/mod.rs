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
    uk_1: Vec<f32>,

    bank_event_uk_0: Vec<BankEventUk0>,
    bank_event_projectile_fire: Vec<BankEventProjectileFire>,
    bank_event_uk_2: Vec<BankEventUk2>,
    bank_event_uk_3: Vec<BankEventUk3>,
    bank_event_uk_4: Vec<BankEventUk4>,
    bank_event_uk_5: Vec<BankEventUk5>,
    bank_event_uk_6: Vec<BankEventUk6>,
    bank_event_uk_7: Vec<BankEventUk7>,
    bank_event_uk_8: Vec<BankEventUk8>,
    bank_event_uk_9: Vec<BankEventUk9>,
    bank_event_uk_10: Vec<BankEventUk10>,
    bank_event_uk_11: Vec<BankEventUk11>,
    bank_event_uk_12: Vec<BankEventUk12>,
    bank_event_uk_13: Vec<BankEventUk13>,
    bank_event_uk_14: Vec<BankEventUk14>,
    bank_event_uk_15: Vec<BankEventUk15>,
    bank_event_uk_16: Vec<BankEventUk16>,
    bank_event_uk_17: Vec<BankEventUk17>,
    bank_event_uk_18: Vec<BankEventUk18>,
    bank_event_uk_19: Vec<BankEventUk19>,
    bank_event_uk_20: Vec<BankEventUk20>,
    bank_event_uk_21: Vec<BankEventUk21>,
    bank_event_uk_22: Vec<BankEventUk22>,
    bank_event_uk_23: Vec<BankEventUk23>,
    bank_event_uk_24: Vec<BankEventUk24>,

    uk_2: Vec<Uk1>,
}

#[derive(Default, PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct BankEventUk0 {
    event_record_index: u32,
    params_1: Vec<u32>,
    params_2: Vec<u32>,
    params_3: Vec<u32>,
    params_4: Vec<u32>,
}

#[derive(Default, PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct BankEventProjectileFire {
    event_record_index: u32,
    gun_types: Vec<u32>,
    shot_types: Vec<u32>,
    projectile_sizes: Vec<u32>,
    params_4: Vec<u32>,
    unit_indexes: Vec<u32>,
}

#[derive(Default, PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct BankEventUk2 {
    event_record_index: u32,
    params_1: Vec<u32>,
    params_2: Vec<u32>,
    params_3: Vec<u32>,
    params_4: Vec<u32>,
}

#[derive(Default, PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct BankEventUk3 {
    event_record_index: u32,
    params_1: Vec<u32>,
    params_2: Vec<u32>,
    params_3: Vec<u32>,
}

#[derive(Default, PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct BankEventUk4 {
    event_record_index: u32,
    params_1: Vec<u32>,
    params_2: Vec<u32>,
}

#[derive(Default, PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct BankEventUk5 {
    event_record_index: u32,
    params_1: Vec<u32>,
    params_2: Vec<u32>,
    params_3: Vec<u32>,
    params_4: Vec<u32>,
}

#[derive(Default, PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct BankEventUk6 {
    event_record_index: u32,
    params_1: Vec<u32>,
    params_2: Vec<u32>,
    params_3: Vec<u32>,
    params_4: Vec<u32>,
    params_5: Vec<u8>,
    params_6: Vec<u8>,
    params_7: Vec<u32>,
}

#[derive(Default, PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct BankEventUk7 {
    event_record_index: u32,
    params_1: Vec<u32>,
    params_2: Vec<u32>,
    params_3: Vec<u32>,
    params_4: Vec<u32>,
    params_5: Vec<u32>,
}

#[derive(Default, PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct BankEventUk8 {
    event_record_index: u32,
    params_1: Vec<u32>,
    params_2: Vec<u32>,
    params_3: Vec<u32>,
    params_4: Vec<u32>,
}

#[derive(Default, PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct BankEventUk9 {
    event_record_index: u32,
    params_1: Vec<u32>,
    params_2: Vec<u32>,
}

#[derive(Default, PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct BankEventUk10 {
    event_record_index: u32,
    params_1: Vec<u32>,
    params_2: Vec<u32>,
}

#[derive(Default, PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct BankEventUk11 {
    event_record_index: u32,
    params_1: Vec<u32>,
}

#[derive(Default, PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct BankEventUk12 {
    event_record_index: u32,
    params_1: Vec<u32>,
    params_2: Vec<u32>,
    params_3: Vec<u32>,
    params_4: Vec<u32>,
    params_5: Vec<u32>,
    params_6: Vec<u32>,
}

#[derive(Default, PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct BankEventUk13 {
    event_record_index: u32,
    params_1: Vec<u32>,
    params_2: Vec<u32>,
    params_3: Vec<u32>,
    params_4: Vec<u32>,
}

#[derive(Default, PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct BankEventUk14 {
    event_record_index: u32,
    params_1: Vec<u32>,
    params_2: Vec<u32>,
    params_3: Vec<u32>,
    params_4: Vec<u32>,
}
#[derive(Default, PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct BankEventUk15 {
    event_record_index: u32,
    params_1: Vec<u32>,
    params_2: Vec<u32>,
    params_3: Vec<u32>,
    params_4: Vec<u32>,
    params_5: Vec<u32>,
    params_6: Vec<u32>,
}

#[derive(Default, PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct BankEventUk16 {
    event_record_index: u32,
    params_1: Vec<u32>,
    params_2: Vec<u32>,
}

#[derive(Default, PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct BankEventUk17 {
    event_record_index: u32,
    params_1: Vec<u32>,
    params_2: Vec<u32>,
    params_3: Vec<u32>,
}

#[derive(Default, PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct BankEventUk18 {
    event_record_index: u32,
    params_1: Vec<u32>,
    params_2: Vec<u32>,
    params_3: Vec<u32>,
}

#[derive(Default, PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct BankEventUk19 {
    event_record_index: u32,
    params_1: Vec<u32>,
    params_2: Vec<u32>,
}

#[derive(Default, PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct BankEventUk20 {
    event_record_index: u32,
    params_1: Vec<u32>,
    params_2: Vec<u32>,
}

#[derive(Default, PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct BankEventUk21 {
    event_record_index: u32,
    params_1: Vec<u32>,
    params_2: Vec<u32>,
}

#[derive(Default, PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct BankEventUk22 {
    event_record_index: u32,
    params_1: Vec<u32>,
    params_2: Vec<u32>,
}

#[derive(Default, PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct BankEventUk23 {
    event_record_index: u32,
    params_1: Vec<u32>,
    params_2: Vec<u32>,
}

#[derive(Default, PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct BankEventUk24 {
    event_record_index: u32,
    params_1: Vec<u32>,
    params_2: Vec<u32>,
}

#[derive(Default, PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct Uk1 {
    uk_1: Vec<u32>,
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
