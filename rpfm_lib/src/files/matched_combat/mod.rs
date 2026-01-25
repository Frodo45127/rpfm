//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! This module contains the implementation of the Matched Combat file format for Total War games.
//!
//! Matched combat files define synchronized combat animations between units, specifying which
//! animation files to use, what conditions must be met, and how units transition during the
//! animation sequence. These files control cinematic melee combat interactions where units
//! perform matched animations together (e.g., sword duels, executions, grapples).
//!
//! # File Format
//!
//! Matched combat files (`.bin`) contain entries that define combat animation sequences.
//! Each entry specifies:
//!
//! - Animation participants (multiple units involved in the sequence)
//! - Entity-specific animation files and metadata
//! - Selection weights for animation variety
//! - Filters to determine when animations can be used (unit types, equipment, etc.)
//! - State transitions (alive to dead, alive to alive, etc.)
//! - Team assignments for participants
//!
//! # Versions
//!
//! The format has multiple versions with game-specific variations:
//!
//! - Version 1 (Three Kingdoms): Basic matched combat system
//! - Version 1 (Warhammer 3): Extended with mount animations
//! - Version 3: Further expanded format
//!
//! # File Locations
//!
//! Matched combat files are typically found in:
//! - `animations/matched_combat/*.bin`
//! - `animations/database/matched/*.bin`
//! - `animations/database/trigger/*.bin`
//!
//! # Usage Example
//!
//! ```rust,ignore
//! use rpfm_lib::files::{Decodeable, matched_combat::*};
//!
//! // Decode a matched combat file
//! let mut data = std::io::Cursor::new(file_data);
//! let extra_data = Some(DecodeableExtraData {
//!     game_info: Some(game_info),
//!     ..Default::default()
//! });
//! let matched = MatchedCombat::decode(&mut data, &extra_data)?;
//!
//! // Access combat entries
//! for entry in matched.entries() {
//!     println!("Combat ID: {}", entry.id());
//!     for participant in entry.participants() {
//!         println!("Team: {}", participant.team());
//!     }
//! }
//! ```

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

/// Represents a matched combat file decoded in memory.
///
/// Contains the format version and a list of combat animation entries.
#[derive(PartialEq, Clone, Debug, Default, Getters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", set = "pub")]
pub struct MatchedCombat {
    /// File format version (1 or 3).
    version: u32,

    /// List of matched combat animation entries.
    entries: Vec<MatchedEntry>,
}

/// A single matched combat animation entry.
///
/// Defines a synchronized combat sequence involving one or more participants,
/// each with their own animations and state transitions.
#[derive(PartialEq, Clone, Debug, Default, Getters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", set = "pub")]
pub struct MatchedEntry {
    /// Unique identifier for this matched combat entry.
    id: String,

    /// List of participants involved in this combat sequence.
    ///
    /// Typically 2 participants for duels, but can be more for group animations.
    participants: Vec<Participant>,
}

/// A participant in a matched combat animation sequence.
///
/// Represents one unit/entity involved in the combat, including which animations
/// it should play, what conditions must be met, and how its state changes.
#[derive(PartialEq, Clone, Debug, Default, Getters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", set = "pub")]
pub struct Participant {
    /// Team identifier for this participant (e.g., 0 for attacker, 1 for defender).
    team: u32,

    /// Bundles of animation entities with selection weights.
    ///
    /// Multiple bundles allow for animation variety - the game randomly selects
    /// one bundle based on weights.
    entity_info: Vec<EntityBundle>,

    /// State transition for this participant (e.g., alive to dead).
    state: State,

    /// Unknown field from Three Kingdoms files.
    uk1: u32,

    /// Unknown field from Three Kingdoms files.
    uk2: u32,

    /// Unknown field from Warhammer 3 files.
    uk3: u32,

    /// Unknown field from Warhammer 3 files.
    uk4: u32,
}

/// A bundle of animation entities that can be selected together.
///
/// Allows the game to randomly choose between different animation variations
/// based on selection weights, providing visual variety in combat.
#[derive(PartialEq, Clone, Debug, Default, Getters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", set = "pub")]
pub struct EntityBundle {
    /// Animation entities in this bundle.
    entities: Vec<Entity>,

    /// Weight for random selection of this bundle (higher = more likely to be chosen).
    selection_weight: f32,
}

/// An entity's animation data for matched combat.
///
/// Specifies the animation files, metadata, timing, equipment visibility,
/// and conditions (filters) for when this animation can be used.
#[derive(PartialEq, Clone, Debug, Default, Getters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", set = "pub")]
pub struct Entity {
    /// Path to the animation file for this entity.
    animation_filename: String,

    /// Paths to metadata files containing additional animation data.
    metadata_filenames: Vec<String>,

    /// Time in seconds for blending into this animation from the previous state.
    blend_in_time: f32,

    /// Equipment display flags controlling weapon/shield visibility during animation.
    equipment_display: u32,

    /// Filters that must match for this animation to be eligible for selection.
    ///
    /// Filters check conditions like unit type, equipment, size, etc.
    filters: Vec<Filter>,

    /// Unknown field.
    uk: u32,

    /// Animation filename for the mount (only in Warhammer 3 files).
    ///
    /// Used when the participant is mounted on a creature or horse.
    mount_filename: String,
}

/// A filter condition for determining when an animation can be used.
///
/// Filters check properties of the participating units (e.g., unit type, equipment,
/// size class) to ensure animations only play when appropriate.
#[derive(PartialEq, Clone, Debug, Default, Getters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", set = "pub")]
pub struct Filter {
    /// If true, check for equality; if false, check for inequality.
    equals: bool,

    /// If true, combine with previous filter using OR; if false, use AND.
    or: bool,

    /// Type of filter (e.g., unit type, equipment slot, size class).
    filter_type: u32,

    /// Value to compare against (interpretation depends on filter_type).
    value: String,
}

/// State transition for a participant during the matched combat sequence.
///
/// Defines how the participant's state changes from the start to the end of the animation
/// (e.g., alive to dead for an execution animation, alive to alive for a non-lethal clash).
#[derive(PartialEq, Clone, Debug, Default, Getters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", set = "pub")]
pub struct State {
    /// State at the beginning of the animation.
    start: StateParticipant,

    /// State at the end of the animation.
    end: StateParticipant,
}

/// Possible states for a participant in a matched combat animation.
///
/// Indicates whether the participant is alive, dead, or in some other state
/// at the beginning or end of the animation sequence.
#[derive(PartialEq, Clone, Copy, Debug, Default, Serialize, Deserialize)]
#[repr(u32)]
pub enum StateParticipant {
    /// Participant is alive and active.
    #[default] Alive,

    /// Participant is dead or dying.
    Dead = 1,

    /// Unknown state variant.
    NoIdea1 = 2,

    /// Unknown state variant.
    NoIdea2 = 3,

    /// Unknown state variant.
    NoIdea3 = 4,

    /// Unknown state variant.
    NoIdea4 = 5,

    /// Unknown state variant.
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
            _ if value == Self::Alive as u32 => Ok(Self::Alive),
            _ if value == Self::Dead as u32 => Ok(Self::Dead),
            _ if value == Self::NoIdea1 as u32 => Ok(Self::NoIdea1),
            _ if value == Self::NoIdea2 as u32 => Ok(Self::NoIdea2),
            _ if value == Self::NoIdea3 as u32 => Ok(Self::NoIdea3),
            _ if value == Self::NoIdea4 as u32 => Ok(Self::NoIdea4),
            _ if value == Self::NoIdea5 as u32 => Ok(Self::NoIdea5),
            _ => Err(RLibError::InvalidStateParticipantValue(value)),
        }
    }
}
