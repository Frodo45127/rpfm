//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Sound bank database for older Total War games.
//!
//! This module handles the `sound_bank_database` file found in Shogun 2 (and possibly
//! Napoleon and Empire). This file maps game events to sound bank event records,
//! allowing the game to trigger appropriate sounds based on gameplay actions.
//!
//! The database contains multiple categories of bank events, each linking event record
//! indices to various parameters that control when and how sounds are triggered.
//!
//! These files are not versioned, so only the latest format per game is supported.
//!
//! Note: Most of this format is not fully understood. Only `BankEventProjectileFire`
//! has partially identified fields.

use getset::*;
use serde_derive::{Serialize, Deserialize};

use crate::binary::{ReadBytes, WriteBytes};
use crate::error::{Result, RLibError};
use crate::files::{DecodeableExtraData, Decodeable, EncodeableExtraData, Encodeable};
use crate::games::supported_games::KEY_SHOGUN_2;
use crate::utils::check_size_mismatch;

/// Path to the sound bank database file within a pack.
pub const PATH: &str = "sounds_packed/sound_bank_database";

mod games;

#[cfg(test)] mod sound_bank_database_test;

//---------------------------------------------------------------------------//
//                              Enum & Structs
//---------------------------------------------------------------------------//

/// Sound bank database mapping game events to sound events.
///
/// Contains multiple categories of bank events that link event record indices
/// (referencing the sound_events file) to parameter sets that control sound triggering.
///
/// Note: Most fields in this format are not fully understood and are marked as `uk_*`
/// or have generic parameter names.
#[derive(Default, PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct SoundBankDatabase {
    /// Unknown float parameters.
    uk_1: Vec<f32>,

    /// Bank event category 0 (unknown purpose).
    bank_event_uk_0: Vec<BankEventUk0>,
    /// Bank events for projectile firing sounds.
    bank_event_projectile_fire: Vec<BankEventProjectileFire>,
    /// Bank event category 2 (unknown purpose).
    bank_event_uk_2: Vec<BankEventUk2>,
    /// Bank event category 3 (unknown purpose).
    bank_event_uk_3: Vec<BankEventUk3>,
    /// Bank event category 4 (unknown purpose).
    bank_event_uk_4: Vec<BankEventUk4>,
    /// Bank event category 5 (unknown purpose).
    bank_event_uk_5: Vec<BankEventUk5>,
    /// Bank event category 6 (unknown purpose).
    bank_event_uk_6: Vec<BankEventUk6>,
    /// Bank event category 7 (unknown purpose).
    bank_event_uk_7: Vec<BankEventUk7>,
    /// Bank event category 8 (unknown purpose).
    bank_event_uk_8: Vec<BankEventUk8>,
    /// Bank event category 9 (unknown purpose).
    bank_event_uk_9: Vec<BankEventUk9>,
    /// Bank event category 10 (unknown purpose).
    bank_event_uk_10: Vec<BankEventUk10>,
    /// Bank event category 11 (unknown purpose).
    bank_event_uk_11: Vec<BankEventUk11>,
    /// Bank event category 12 (unknown purpose).
    bank_event_uk_12: Vec<BankEventUk12>,
    /// Bank event category 13 (unknown purpose).
    bank_event_uk_13: Vec<BankEventUk13>,
    /// Bank event category 14 (unknown purpose).
    bank_event_uk_14: Vec<BankEventUk14>,
    /// Bank event category 15 (unknown purpose).
    bank_event_uk_15: Vec<BankEventUk15>,
    /// Bank event category 16 (unknown purpose).
    bank_event_uk_16: Vec<BankEventUk16>,
    /// Bank event category 17 (unknown purpose).
    bank_event_uk_17: Vec<BankEventUk17>,
    /// Bank event category 18 (unknown purpose).
    bank_event_uk_18: Vec<BankEventUk18>,
    /// Bank event category 19 (unknown purpose).
    bank_event_uk_19: Vec<BankEventUk19>,
    /// Bank event category 20 (unknown purpose).
    bank_event_uk_20: Vec<BankEventUk20>,
    /// Bank event category 21 (unknown purpose).
    bank_event_uk_21: Vec<BankEventUk21>,
    /// Bank event category 22 (unknown purpose).
    bank_event_uk_22: Vec<BankEventUk22>,
    /// Bank event category 23 (unknown purpose).
    bank_event_uk_23: Vec<BankEventUk23>,
    /// Bank event category 24 (unknown purpose).
    bank_event_uk_24: Vec<BankEventUk24>,

    /// Unknown data section.
    uk_2: Vec<Uk1>,
}

/// Bank event with unknown purpose (category 0).
#[derive(Default, PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct BankEventUk0 {
    /// Index into the sound_events event_records array.
    event_record_index: u32,
    /// Parameter set 1.
    params_1: Vec<u32>,
    /// Parameter set 2.
    params_2: Vec<u32>,
    /// Parameter set 3.
    params_3: Vec<u32>,
    /// Parameter set 4.
    params_4: Vec<u32>,
}

/// Bank event for projectile firing sounds.
///
/// This is the only partially understood bank event type. It maps projectile
/// firing actions to sound events based on weapon and projectile characteristics.
#[derive(Default, PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct BankEventProjectileFire {
    /// Index into the sound_events event_records array.
    event_record_index: u32,
    /// Gun type identifiers that trigger this sound.
    gun_types: Vec<u32>,
    /// Shot type identifiers.
    shot_types: Vec<u32>,
    /// Projectile size categories.
    projectile_sizes: Vec<u32>,
    /// Unknown parameter set 4.
    params_4: Vec<u32>,
    /// Unit indices that use this sound.
    unit_indexes: Vec<u32>,
}

/// Bank event with unknown purpose (category 2).
#[derive(Default, PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct BankEventUk2 {
    /// Index into the sound_events event_records array.
    event_record_index: u32,
    /// Parameter set 1.
    params_1: Vec<u32>,
    /// Parameter set 2.
    params_2: Vec<u32>,
    /// Parameter set 3.
    params_3: Vec<u32>,
    /// Parameter set 4.
    params_4: Vec<u32>,
}

/// Bank event with unknown purpose (category 3).
#[derive(Default, PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct BankEventUk3 {
    /// Index into the sound_events event_records array.
    event_record_index: u32,
    /// Parameter set 1.
    params_1: Vec<u32>,
    /// Parameter set 2.
    params_2: Vec<u32>,
    /// Parameter set 3.
    params_3: Vec<u32>,
}

/// Bank event with unknown purpose (category 4).
#[derive(Default, PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct BankEventUk4 {
    /// Index into the sound_events event_records array.
    event_record_index: u32,
    /// Parameter set 1.
    params_1: Vec<u32>,
    /// Parameter set 2.
    params_2: Vec<u32>,
}

/// Bank event with unknown purpose (category 5).
#[derive(Default, PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct BankEventUk5 {
    /// Index into the sound_events event_records array.
    event_record_index: u32,
    /// Parameter set 1.
    params_1: Vec<u32>,
    /// Parameter set 2.
    params_2: Vec<u32>,
    /// Parameter set 3.
    params_3: Vec<u32>,
    /// Parameter set 4.
    params_4: Vec<u32>,
}

/// Bank event with unknown purpose (category 6).
#[derive(Default, PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct BankEventUk6 {
    /// Index into the sound_events event_records array.
    event_record_index: u32,
    /// Parameter set 1.
    params_1: Vec<u32>,
    /// Parameter set 2.
    params_2: Vec<u32>,
    /// Parameter set 3.
    params_3: Vec<u32>,
    /// Parameter set 4.
    params_4: Vec<u32>,
    /// Parameter set 5 (byte values).
    params_5: Vec<u8>,
    /// Parameter set 6 (byte values).
    params_6: Vec<u8>,
    /// Parameter set 7.
    params_7: Vec<u32>,
}

/// Bank event with unknown purpose (category 7).
#[derive(Default, PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct BankEventUk7 {
    /// Index into the sound_events event_records array.
    event_record_index: u32,
    /// Parameter set 1.
    params_1: Vec<u32>,
    /// Parameter set 2.
    params_2: Vec<u32>,
    /// Parameter set 3.
    params_3: Vec<u32>,
    /// Parameter set 4.
    params_4: Vec<u32>,
    /// Parameter set 5.
    params_5: Vec<u32>,
}

/// Bank event with unknown purpose (category 8).
#[derive(Default, PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct BankEventUk8 {
    /// Index into the sound_events event_records array.
    event_record_index: u32,
    /// Parameter set 1.
    params_1: Vec<u32>,
    /// Parameter set 2.
    params_2: Vec<u32>,
    /// Parameter set 3.
    params_3: Vec<u32>,
    /// Parameter set 4.
    params_4: Vec<u32>,
}

/// Bank event with unknown purpose (category 9).
#[derive(Default, PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct BankEventUk9 {
    /// Index into the sound_events event_records array.
    event_record_index: u32,
    /// Parameter set 1.
    params_1: Vec<u32>,
    /// Parameter set 2.
    params_2: Vec<u32>,
}

/// Bank event with unknown purpose (category 10).
#[derive(Default, PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct BankEventUk10 {
    /// Index into the sound_events event_records array.
    event_record_index: u32,
    /// Parameter set 1.
    params_1: Vec<u32>,
    /// Parameter set 2.
    params_2: Vec<u32>,
}

/// Bank event with unknown purpose (category 11).
#[derive(Default, PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct BankEventUk11 {
    /// Index into the sound_events event_records array.
    event_record_index: u32,
    /// Parameter set 1.
    params_1: Vec<u32>,
}

/// Bank event with unknown purpose (category 12).
#[derive(Default, PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct BankEventUk12 {
    /// Index into the sound_events event_records array.
    event_record_index: u32,
    /// Parameter set 1.
    params_1: Vec<u32>,
    /// Parameter set 2.
    params_2: Vec<u32>,
    /// Parameter set 3.
    params_3: Vec<u32>,
    /// Parameter set 4.
    params_4: Vec<u32>,
    /// Parameter set 5.
    params_5: Vec<u32>,
    /// Parameter set 6.
    params_6: Vec<u32>,
}

/// Bank event with unknown purpose (category 13).
#[derive(Default, PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct BankEventUk13 {
    /// Index into the sound_events event_records array.
    event_record_index: u32,
    /// Parameter set 1.
    params_1: Vec<u32>,
    /// Parameter set 2.
    params_2: Vec<u32>,
    /// Parameter set 3.
    params_3: Vec<u32>,
    /// Parameter set 4.
    params_4: Vec<u32>,
}

/// Bank event with unknown purpose (category 14).
#[derive(Default, PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct BankEventUk14 {
    /// Index into the sound_events event_records array.
    event_record_index: u32,
    /// Parameter set 1.
    params_1: Vec<u32>,
    /// Parameter set 2.
    params_2: Vec<u32>,
    /// Parameter set 3.
    params_3: Vec<u32>,
    /// Parameter set 4.
    params_4: Vec<u32>,
}

/// Bank event with unknown purpose (category 15).
#[derive(Default, PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct BankEventUk15 {
    /// Index into the sound_events event_records array.
    event_record_index: u32,
    /// Parameter set 1.
    params_1: Vec<u32>,
    /// Parameter set 2.
    params_2: Vec<u32>,
    /// Parameter set 3.
    params_3: Vec<u32>,
    /// Parameter set 4.
    params_4: Vec<u32>,
    /// Parameter set 5.
    params_5: Vec<u32>,
    /// Parameter set 6.
    params_6: Vec<u32>,
}

/// Bank event with unknown purpose (category 16).
#[derive(Default, PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct BankEventUk16 {
    /// Index into the sound_events event_records array.
    event_record_index: u32,
    /// Parameter set 1.
    params_1: Vec<u32>,
    /// Parameter set 2.
    params_2: Vec<u32>,
}

/// Bank event with unknown purpose (category 17).
#[derive(Default, PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct BankEventUk17 {
    /// Index into the sound_events event_records array.
    event_record_index: u32,
    /// Parameter set 1.
    params_1: Vec<u32>,
    /// Parameter set 2.
    params_2: Vec<u32>,
    /// Parameter set 3.
    params_3: Vec<u32>,
}

/// Bank event with unknown purpose (category 18).
#[derive(Default, PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct BankEventUk18 {
    /// Index into the sound_events event_records array.
    event_record_index: u32,
    /// Parameter set 1.
    params_1: Vec<u32>,
    /// Parameter set 2.
    params_2: Vec<u32>,
    /// Parameter set 3.
    params_3: Vec<u32>,
}

/// Bank event with unknown purpose (category 19).
#[derive(Default, PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct BankEventUk19 {
    /// Index into the sound_events event_records array.
    event_record_index: u32,
    /// Parameter set 1.
    params_1: Vec<u32>,
    /// Parameter set 2.
    params_2: Vec<u32>,
}

/// Bank event with unknown purpose (category 20).
#[derive(Default, PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct BankEventUk20 {
    /// Index into the sound_events event_records array.
    event_record_index: u32,
    /// Parameter set 1.
    params_1: Vec<u32>,
    /// Parameter set 2.
    params_2: Vec<u32>,
}

/// Bank event with unknown purpose (category 21).
#[derive(Default, PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct BankEventUk21 {
    /// Index into the sound_events event_records array.
    event_record_index: u32,
    /// Parameter set 1.
    params_1: Vec<u32>,
    /// Parameter set 2.
    params_2: Vec<u32>,
}

/// Bank event with unknown purpose (category 22).
#[derive(Default, PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct BankEventUk22 {
    /// Index into the sound_events event_records array.
    event_record_index: u32,
    /// Parameter set 1.
    params_1: Vec<u32>,
    /// Parameter set 2.
    params_2: Vec<u32>,
}

/// Bank event with unknown purpose (category 23).
#[derive(Default, PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct BankEventUk23 {
    /// Index into the sound_events event_records array.
    event_record_index: u32,
    /// Parameter set 1.
    params_1: Vec<u32>,
    /// Parameter set 2.
    params_2: Vec<u32>,
}

/// Bank event with unknown purpose (category 24).
#[derive(Default, PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct BankEventUk24 {
    /// Index into the sound_events event_records array.
    event_record_index: u32,
    /// Parameter set 1.
    params_1: Vec<u32>,
    /// Parameter set 2.
    params_2: Vec<u32>,
}

/// Unknown data structure.
#[derive(Default, PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct Uk1 {
    /// Unknown parameter set.
    uk_1: Vec<u32>,
}

//---------------------------------------------------------------------------//
//                              Implementations
//---------------------------------------------------------------------------//

impl Decodeable for SoundBankDatabase {

    fn decode<R: ReadBytes>(data: &mut R, extra_data: &Option<DecodeableExtraData>) -> Result<Self> {
        let extra_data = extra_data.as_ref().ok_or(RLibError::DecodingMissingExtraData)?;
        let game_info = extra_data.game_info.ok_or_else(|| RLibError::DecodingMissingExtraDataField("game_info".to_owned()))?;

        let mut sound_bank = Self::default();

        match game_info.key() {
            KEY_SHOGUN_2 => sound_bank.read_sho2(data)?,
            //KEY_NAPOLEON => {},
            //KEY_EMPIRE => sound_bank.read_emp(data)?,
            _ => return Err(RLibError::DecodingSoundPackedUnsupportedGame(game_info.key().to_string())),
        }

        // If we are not in the last byte, it means we didn't parse the entire file, which means this file is corrupt.
        check_size_mismatch(data.stream_position()? as usize, data.len()? as usize)?;

        Ok(sound_bank)
    }
}

impl Encodeable for SoundBankDatabase {

    fn encode<W: WriteBytes>(&mut self, buffer: &mut W, extra_data: &Option<EncodeableExtraData>) -> Result<()> {
        let extra_data = extra_data.as_ref().ok_or(RLibError::EncodingMissingExtraData)?;
        let game_info = extra_data.game_info.ok_or_else(|| RLibError::DecodingMissingExtraDataField("game_info".to_owned()))?;

        match game_info.key() {
            KEY_SHOGUN_2 => self.write_sho2(buffer),
            //KEY_NAPOLEON => {},
            //KEY_EMPIRE => self.write_emp(buffer),
            _ => Err(RLibError::EncodingSoundPackedUnsupportedGame(game_info.key().to_string())),
        }
    }
}
