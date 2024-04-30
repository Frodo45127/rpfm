//---------------------------------------------------------------------------//
// Copyright (c) 2017-2024 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! This is a module to read/write sound_events files from Shogun 2, Napoleon and Empire.
//!
//! These files are NOT VERSIONED, so we only support the latest one per-game.

use getset::*;
use serde_derive::{Serialize, Deserialize};

use crate::binary::{ReadBytes, WriteBytes};
use crate::error::{Result, RLibError};
use crate::files::{DecodeableExtraData, Decodeable, EncodeableExtraData, Encodeable};
use crate::games::supported_games::{KEY_SHOGUN_2, KEY_EMPIRE};
use crate::utils::check_size_mismatch;

pub const PATH: &str = "sounds_packed/sound_events";

mod games;

#[cfg(test)] mod sound_events_test;

//---------------------------------------------------------------------------//
//                              Enum & Structs
//---------------------------------------------------------------------------//

#[derive(Default, PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct SoundEvents {
    master_volume: f32,
    categories: Vec<Category>,
    uk_1: Vec<Uk1>,
    uk_4: Vec<Uk4>,
    uk_5: Vec<Uk5>,
    uk_6: u32,
    uk_7: u32,
    uk_8: Vec<Uk8>,
    event_data: Vec<EventData>,
    event_records: Vec<EventRecord>,
    ambience_map: Vec<AmbienceMap>,
    uk_3: Vec<Uk3>,
    movies: Vec<Movie>,
    uk_9: Vec<Uk9>,
}

#[derive(Default, PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct Category {
    name: String,
    uk_1: f32,
}

#[derive(Default, PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct Uk1 {
    uk_1: i32,
}

#[derive(Default, PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct Uk4 {
    uk_1: i32,
    uk_2: i32,
}

#[derive(Default, PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct Uk5 {
    uk_1: f32,
    uk_2: f32,
    uk_3: f32,
    uk_4: f32,
    uk_5: f32,
    uk_6: f32,
    uk_7: f32,
    uk_8: f32,
}

#[derive(Default, PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct Uk8 {
    uk_1: u32,
}

#[derive(Default, PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct EventData {
    uk_1: f32,
    uk_2: f32,
    uk_3: f32,
    uk_4: f32,
    uk_5: f32,
    uk_6: f32,
    uk_7: f32,
    uk_8: f32,
    uk_9: f32,
    uk_10: f32,
    uk_11: f32,
    uk_12: f32,
    uk_13: f32,
    uk_14: f32,
    uk_15: f32,
    uk_16: f32,
    uk_17: f32,
    uk_18: f32,
    uk_19: f32,
    uk_20: f32,
    uk_21: f32,
    uk_22: f32,
    uk_23: f32,
    uk_24: f32,
    uk_25: f32,
    uk_26: f32,
    uk_27: f32,
    uk_28: f32,
    uk_29: f32,
    uk_30: f32,
    uk_31: f32,
    uk_32: f32,
    uk_33: f32,
    uk_34: f32,
    uk_35: f32,
    uk_36: f32,
    uk_37: f32,
    uk_38: f32,
    uk_39: f32,
    uk_40: f32,
    uk_41: f32,
    uk_42: f32,
    uk_43: f32,
    uk_44: f32,
    uk_45: f32,
    uk_46: f32,
    uk_47: f32,
}

#[derive(Default, PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct EventRecord {
    category: u32,
    name: Option<String>,
    uk_1: u32,
    uk_2: i32,
    uk_3: i32,
    event_data_index: u32,
    uk_4: u8,
    sounds: Vec<String>,
}

#[derive(Default, PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct AmbienceMap {
    name: String,
    records: Vec<AmbienceRecord>,
}

#[derive(Default, PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct AmbienceRecord {
    uk_1: u32,
    event_index: u32,
    uk_3: f32,
    uk_4: f32,
    uk_5: f32,
    uk_6: f32,
    uk_7: f32,
    uk_8: f32,
}

#[derive(Default, PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct Uk3 {
    uk_1: i32,
}

#[derive(Default, PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct Movie {
    file: String,
    volume: f32,
}

#[derive(Default, PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct Uk9 {
    file: String,
    uk_1: i32,
}

//---------------------------------------------------------------------------//
//                              Implementations
//---------------------------------------------------------------------------//

impl Decodeable for SoundEvents {

    fn decode<R: ReadBytes>(data: &mut R, extra_data: &Option<DecodeableExtraData>) -> Result<Self> {
        let extra_data = extra_data.as_ref().ok_or(RLibError::DecodingMissingExtraData)?;
        let game_key = extra_data.game_key.ok_or_else(|| RLibError::DecodingMissingExtraDataField("game_key".to_owned()))?;

        let mut sound_events = Self::default();

        match game_key {
            KEY_SHOGUN_2 => sound_events.read_sho2(data)?,
            //KEY_NAPOLEON => {},
            KEY_EMPIRE => sound_events.read_emp(data)?,
            _ => return Err(RLibError::DecodingSoundPackedUnsupportedGame(game_key.to_string())),
        }

        // If we are not in the last byte, it means we didn't parse the entire file, which means this file is corrupt.
        check_size_mismatch(data.stream_position()? as usize, data.len()? as usize)?;

        Ok(sound_events)
    }
}

impl Encodeable for SoundEvents {

    fn encode<W: WriteBytes>(&mut self, buffer: &mut W, extra_data: &Option<EncodeableExtraData>) -> Result<()> {
        let extra_data = extra_data.as_ref().ok_or(RLibError::EncodingMissingExtraData)?;
        let game_key = extra_data.game_key.ok_or_else(|| RLibError::DecodingMissingExtraDataField("game_key".to_owned()))?;

        match game_key {
            KEY_SHOGUN_2 => self.write_sho2(buffer),
            //KEY_NAPOLEON => {},
            KEY_EMPIRE => self.write_emp(buffer),
            _ => Err(RLibError::EncodingSoundPackedUnsupportedGame(game_key.to_string())),
        }
    }
}
