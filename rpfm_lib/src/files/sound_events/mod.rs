//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Sound events configuration for older Total War games.
//!
//! This module handles the `sound_events` file found in Shogun 2, Napoleon, and Empire.
//! These files define sound event categories, event records, ambience maps, and movie
//! audio settings used by the game's audio system.
//!
//! These files are not versioned, so only the latest format per game is supported.

use getset::*;
use serde_derive::{Serialize, Deserialize};

use crate::binary::{ReadBytes, WriteBytes};
use crate::error::{Result, RLibError};
use crate::files::{DecodeableExtraData, Decodeable, EncodeableExtraData, Encodeable};
use crate::games::supported_games::{KEY_SHOGUN_2, KEY_EMPIRE};
use crate::utils::check_size_mismatch;

/// Path to the sound events file within a pack.
pub const PATH: &str = "sounds_packed/sound_events";

mod games;

#[cfg(test)] mod sound_events_test;

//---------------------------------------------------------------------------//
//                              Enum & Structs
//---------------------------------------------------------------------------//

/// Sound events configuration for a game.
///
/// Contains the complete audio event system configuration including categories,
/// event definitions, ambience mappings, and movie audio settings.
///
/// Note: Many fields in this format are not fully understood and are marked as `uk_*`.
#[derive(Default, PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct SoundEvents {
    /// Master volume level for all sounds.
    master_volume: f32,
    /// Sound categories (e.g., music, SFX, voice).
    categories: Vec<Category>,
    /// Unknown data section 1.
    uk_1: Vec<Uk1>,
    /// Unknown data section 4.
    uk_4: Vec<Uk4>,
    /// Unknown data section 5.
    uk_5: Vec<Uk5>,
    /// Unknown value 6.
    uk_6: u32,
    /// Unknown value 7.
    uk_7: u32,
    /// Unknown data section 8.
    uk_8: Vec<Uk8>,
    /// Event data parameters referenced by event records.
    event_data: Vec<EventData>,
    /// Sound event definitions.
    event_records: Vec<EventRecord>,
    /// Ambience sound mappings.
    ambience_map: Vec<AmbienceMap>,
    /// Unknown data section 3.
    uk_3: Vec<Uk3>,
    /// Movie audio configurations.
    movies: Vec<Movie>,
    /// Unknown data section 9.
    uk_9: Vec<Uk9>,
}

/// A sound category definition.
///
/// Categories group related sounds together for volume control and management.
#[derive(Default, PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct Category {
    /// Name of the category.
    name: String,
    /// Unknown parameter (possibly volume or priority).
    uk_1: f32,
}

/// Unknown data structure 1.
#[derive(Default, PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct Uk1 {
    /// Unknown value.
    uk_1: i32,
}

/// Unknown data structure 4.
#[derive(Default, PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct Uk4 {
    /// Unknown value 1.
    uk_1: i32,
    /// Unknown value 2.
    uk_2: i32,
}

/// Unknown data structure 5 with float parameters.
#[derive(Default, PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct Uk5 {
    /// Unknown float 1.
    uk_1: f32,
    /// Unknown float 2.
    uk_2: f32,
    /// Unknown float 3.
    uk_3: f32,
    /// Unknown float 4.
    uk_4: f32,
    /// Unknown float 5.
    uk_5: f32,
    /// Unknown float 6.
    uk_6: f32,
    /// Unknown float 7.
    uk_7: f32,
    /// Unknown float 8.
    uk_8: f32,
}

/// Unknown data structure 8.
#[derive(Default, PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct Uk8 {
    /// Unknown value.
    uk_1: u32,
}

/// Event data parameters.
///
/// Contains a large set of float parameters that define audio playback characteristics.
/// The exact meaning of each parameter is not fully understood.
#[derive(Default, PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct EventData {
    /// Unknown parameter 1.
    uk_1: f32,
    /// Unknown parameter 2.
    uk_2: f32,
    /// Unknown parameter 3.
    uk_3: f32,
    /// Unknown parameter 4.
    uk_4: f32,
    /// Unknown parameter 5.
    uk_5: f32,
    /// Unknown parameter 6.
    uk_6: f32,
    /// Unknown parameter 7.
    uk_7: f32,
    /// Unknown parameter 8.
    uk_8: f32,
    /// Unknown parameter 9.
    uk_9: f32,
    /// Unknown parameter 10.
    uk_10: f32,
    /// Unknown parameter 11.
    uk_11: f32,
    /// Unknown parameter 12.
    uk_12: f32,
    /// Unknown parameter 13.
    uk_13: f32,
    /// Unknown parameter 14.
    uk_14: f32,
    /// Unknown parameter 15.
    uk_15: f32,
    /// Unknown parameter 16.
    uk_16: f32,
    /// Unknown parameter 17.
    uk_17: f32,
    /// Unknown parameter 18.
    uk_18: f32,
    /// Unknown parameter 19.
    uk_19: f32,
    /// Unknown parameter 20.
    uk_20: f32,
    /// Unknown parameter 21.
    uk_21: f32,
    /// Unknown parameter 22.
    uk_22: f32,
    /// Unknown parameter 23.
    uk_23: f32,
    /// Unknown parameter 24.
    uk_24: f32,
    /// Unknown parameter 25.
    uk_25: f32,
    /// Unknown parameter 26.
    uk_26: f32,
    /// Unknown parameter 27.
    uk_27: f32,
    /// Unknown parameter 28.
    uk_28: f32,
    /// Unknown parameter 29.
    uk_29: f32,
    /// Unknown parameter 30.
    uk_30: f32,
    /// Unknown parameter 31.
    uk_31: f32,
    /// Unknown parameter 32.
    uk_32: f32,
    /// Unknown parameter 33.
    uk_33: f32,
    /// Unknown parameter 34.
    uk_34: f32,
    /// Unknown parameter 35.
    uk_35: f32,
    /// Unknown parameter 36.
    uk_36: f32,
    /// Unknown parameter 37.
    uk_37: f32,
    /// Unknown parameter 38.
    uk_38: f32,
    /// Unknown parameter 39.
    uk_39: f32,
    /// Unknown parameter 40.
    uk_40: f32,
    /// Unknown parameter 41.
    uk_41: f32,
    /// Unknown parameter 42.
    uk_42: f32,
    /// Unknown parameter 43.
    uk_43: f32,
    /// Unknown parameter 44.
    uk_44: f32,
    /// Unknown parameter 45.
    uk_45: f32,
    /// Unknown parameter 46.
    uk_46: f32,
    /// Unknown parameter 47.
    uk_47: f32,
}

/// A sound event record.
///
/// Defines a triggerable sound event that references sound files and event data parameters.
#[derive(Default, PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct EventRecord {
    /// Index of the category this event belongs to.
    category: u32,
    /// Optional name of the event.
    name: Option<String>,
    /// Unknown value 1.
    uk_1: u32,
    /// Unknown value 2.
    uk_2: i32,
    /// Unknown value 3.
    uk_3: i32,
    /// Index into the event_data array for this event's parameters.
    event_data_index: u32,
    /// Unknown value 4.
    uk_4: u8,
    /// List of sound file paths associated with this event.
    sounds: Vec<String>,
}

/// An ambience map definition.
///
/// Maps ambience names to collections of ambience records for environmental audio.
#[derive(Default, PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct AmbienceMap {
    /// Name of the ambience map.
    name: String,
    /// Ambience records within this map.
    records: Vec<AmbienceRecord>,
}

/// An ambience record within an ambience map.
///
/// Links to an event and contains spatial/volume parameters for ambient sounds.
#[derive(Default, PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct AmbienceRecord {
    /// Unknown value 1.
    uk_1: u32,
    /// Index into the event_records array.
    event_index: u32,
    /// Unknown float 3.
    uk_3: f32,
    /// Unknown float 4.
    uk_4: f32,
    /// Unknown float 5.
    uk_5: f32,
    /// Unknown float 6.
    uk_6: f32,
    /// Unknown float 7.
    uk_7: f32,
    /// Unknown float 8.
    uk_8: f32,
}

/// Unknown data structure 3.
#[derive(Default, PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct Uk3 {
    /// Unknown value.
    uk_1: i32,
}

/// Movie audio configuration.
///
/// Associates a movie file with its audio volume setting.
#[derive(Default, PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct Movie {
    /// Path to the movie file.
    file: String,
    /// Volume level for the movie's audio.
    volume: f32,
}

/// Unknown data structure 9 with file reference.
#[derive(Default, PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct Uk9 {
    /// Path to a file.
    file: String,
    /// Unknown value.
    uk_1: i32,
}

//---------------------------------------------------------------------------//
//                              Implementations
//---------------------------------------------------------------------------//

impl Decodeable for SoundEvents {

    fn decode<R: ReadBytes>(data: &mut R, extra_data: &Option<DecodeableExtraData>) -> Result<Self> {
        let extra_data = extra_data.as_ref().ok_or(RLibError::DecodingMissingExtraData)?;
        let game_info = extra_data.game_info.ok_or_else(|| RLibError::DecodingMissingExtraDataField("game_info".to_owned()))?;

        let mut sound_events = Self::default();

        match game_info.key() {
            KEY_SHOGUN_2 => sound_events.read_sho2(data)?,
            //KEY_NAPOLEON => {},
            KEY_EMPIRE => sound_events.read_emp(data)?,
            _ => return Err(RLibError::DecodingSoundPackedUnsupportedGame(game_info.key().to_string())),
        }

        // If we are not in the last byte, it means we didn't parse the entire file, which means this file is corrupt.
        check_size_mismatch(data.stream_position()? as usize, data.len()? as usize)?;

        Ok(sound_events)
    }
}

impl Encodeable for SoundEvents {

    fn encode<W: WriteBytes>(&mut self, buffer: &mut W, extra_data: &Option<EncodeableExtraData>) -> Result<()> {
        let extra_data = extra_data.as_ref().ok_or(RLibError::EncodingMissingExtraData)?;
        let game_info = extra_data.game_info.ok_or_else(|| RLibError::DecodingMissingExtraDataField("game_info".to_owned()))?;

        match game_info.key() {
            KEY_SHOGUN_2 => self.write_sho2(buffer),
            //KEY_NAPOLEON => {},
            KEY_EMPIRE => self.write_emp(buffer),
            _ => Err(RLibError::EncodingSoundPackedUnsupportedGame(game_info.key().to_string())),
        }
    }
}
