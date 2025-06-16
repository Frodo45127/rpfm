//---------------------------------------------------------------------------//
// Copyright (c) 2017-2024 Ismael Gutiérrez González. All rights reserved.
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
use crate::games::supported_games::{KEY_THREE_KINGDOMS, KEY_WARHAMMER_3};
use crate::utils::check_size_mismatch;

/// Matched combat files go under these folders.
pub const BASE_PATHS: [&str; 3] = ["animations/matched_combat", "animations/database/matched", "animations/database/trigger"];

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
    entries: Vec<MatchedEntry>,
}

#[derive(PartialEq, Clone, Debug, Default, Getters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", set = "pub")]
pub struct MatchedEntry {
    id: String,
    participants: Vec<Participant>,
}

#[derive(PartialEq, Clone, Debug, Default, Getters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", set = "pub")]
pub struct Participant {
    team: u32,
    entity_info: Vec<EntityBundle>,
    state: State,

    // Unknown values from the Three Kingdoms files.
    uk1: u32,
    uk2: u32,

    // Unknown values from the Warhammer 3 files.
    uk3: u32,
    uk4: u32,
}

#[derive(PartialEq, Clone, Debug, Default, Getters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", set = "pub")]
pub struct EntityBundle {
    entities: Vec<Entity>,
    selection_weight: f32,
}

#[derive(PartialEq, Clone, Debug, Default, Getters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", set = "pub")]
pub struct Entity {
    animation_filename: String,
    metadata_filenames: Vec<String>,
    blend_in_time: f32,
    equipment_display: u32,
    filters: Vec<Filter>,
    uk: u32,

    // Only in Warhammer 3 files.
    mount_filename: String,

}

#[derive(PartialEq, Clone, Debug, Default, Getters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", set = "pub")]
pub struct Filter {
    equals: bool,
    or: bool,
    filter_type: u32,
    value: String,
}

#[derive(PartialEq, Clone, Debug, Default, Getters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", set = "pub")]
pub struct State {
    start: StateParticipant,
    end: StateParticipant,
}

#[derive(PartialEq, Clone, Copy, Debug, Default, Serialize, Deserialize)]
pub enum StateParticipant {
    #[default] Alive,
    Dead = 1,
    NoIdea1 = 2,
    NoIdea2 = 3,
    NoIdea3 = 4,
    NoIdea4 = 5,
    NoIdea5 = 6,
}

//---------------------------------------------------------------------------//
//                      Implementation of MatchedCombat
//---------------------------------------------------------------------------//

impl Decodeable for MatchedCombat {

    fn decode<R: ReadBytes>(data: &mut R, extra_data: &Option<DecodeableExtraData>) -> Result<Self> {
        let extra_data = extra_data.as_ref().ok_or(RLibError::DecodingMissingExtraData)?;
        let game_info = extra_data.game_info.ok_or_else(|| RLibError::DecodingMissingExtraDataField("game_info".to_owned()))?;

        let mut matched = Self::default();
        matched.version = data.read_u32()?;

        match matched.version {
            1 => match game_info.key() {
                KEY_WARHAMMER_3 => matched.read_v1_wh3(data)?,
                KEY_THREE_KINGDOMS => matched.read_v1_3k(data)?,
                _ => Err(RLibError::DecodingMatchedCombatUnsupportedVersion(matched.version as usize))?,
            }
            3 => matched.read_v3(data)?,
            _ => Err(RLibError::DecodingMatchedCombatUnsupportedVersion(matched.version as usize))?,
        }

        // If we are not in the last byte, it means we didn't parse the entire file, which means this file is corrupt.
        check_size_mismatch(data.stream_position()? as usize, data.len()? as usize)?;

        Ok(matched)
    }
}

impl Encodeable for MatchedCombat {

    fn encode<W: WriteBytes>(&mut self, buffer: &mut W, extra_data: &Option<EncodeableExtraData>) -> Result<()> {
        let extra_data = extra_data.as_ref().ok_or(RLibError::EncodingMissingExtraData)?;
        let game_info = extra_data.game_info.ok_or_else(|| RLibError::DecodingMissingExtraDataField("game_info".to_owned()))?;

        buffer.write_u32(self.version)?;

        match self.version {
            1 => match game_info.key() {
                KEY_WARHAMMER_3 => self.write_v1_wh3(buffer)?,
                KEY_THREE_KINGDOMS => self.write_v1_3k(buffer)?,
                _ => Err(RLibError::DecodingMatchedCombatUnsupportedVersion(self.version as usize))?,
            }
            3 => self.write_v3(buffer)?,
            _ => Err(RLibError::DecodingMatchedCombatUnsupportedVersion(self.version as usize))?,
        };

        Ok(())
    }
}

impl TryFrom<u32> for StateParticipant {
    type Error = RLibError;

    fn try_from(value: u32) -> std::result::Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Alive),
            1 => Ok(Self::Dead),
            2 => Ok(Self::NoIdea1),
            3 => Ok(Self::NoIdea2),
            4 => Ok(Self::NoIdea3),
            5 => Ok(Self::NoIdea4),
            6 => Ok(Self::NoIdea5),
            _ => Err(RLibError::InvalidStateParticipantValue(value)),
        }
    }
}
