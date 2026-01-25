//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! This module contains the implementation of the Group Formations file format for Total War games.
//!
//! Group formations define fixed formation templates that both the AI and player can use to
//! deploy their units on the battlefield. Each formation is designed for specific tactical
//! scenarios (attack, defend, naval, etc.) and specifies unit placement, spacing, arrangement
//! patterns, and which unit types should be placed in each position.
//!
//! # File Format
//!
//! The group formations file (`groupformations.bin`) is a binary file containing formation
//! definitions that can be used by both AI and players to deploy armies tactically. Each
//! formation can specify:
//!
//! - AI purpose flags (attack, defend, river crossing, naval, etc.)
//! - Priority and unit category requirements
//! - Supported factions and subcultures
//! - Formation blocks defining unit positions (absolute, relative, or spanning)
//! - Entity preferences specifying which unit types go where
//!
//! # Game Support
//!
//! This file format is not versioned in the traditional sense. Instead, different games have
//! different implementations in the `versions/` subdirectory:
//!
//! - Shogun 2: Basic formation system with entity types and arrangements
//! - Rome 2 (and later): Extended system with entity weights, subcultures, and more AI purposes
//!
//! # File Location
//!
//! Group formations files are typically found at:
//! - `groupformations.bin` in the root of a game's pack
//!
//! # Usage Example
//!
//! ```rust,ignore
//! use rpfm_lib::files::{Decodeable, group_formations::*};
//!
//! // Decode a group formations file
//! let mut data = std::io::Cursor::new(file_data);
//! let extra_data = Some(DecodeableExtraData {
//!     game_info: Some(game_info),
//!     ..Default::default()
//! });
//! let formations = GroupFormations::decode(&mut data, &extra_data)?;
//!
//! // Access formation data
//! for formation in formations.formations() {
//!     println!("Formation: {}", formation.name());
//!     println!("Priority: {}", formation.ai_priority());
//! }
//! ```

use bitflags::bitflags;
use getset::*;
use serde_derive::{Serialize, Deserialize};

use crate::binary::{ReadBytes, WriteBytes};
use crate::error::{Result, RLibError};
use crate::files::{Decodeable, EncodeableExtraData, Encodeable};
use crate::games::supported_games::*;
use crate::utils::*;

use super::DecodeableExtraData;

/// Fixed path to the Group Formations file.
pub const PATH: &str = "groupformations.bin";

mod versions;

#[cfg(test)] mod test_group_formations;

//---------------------------------------------------------------------------//
//                              Enum & Structs
//---------------------------------------------------------------------------//

/// Represents an entire Group Formations file decoded in memory.
///
/// Contains a list of formation templates that both AI and players can use to
/// deploy armies on the battlefield for various tactical scenarios.
#[derive(Default, PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct GroupFormations {
    /// List of all formation definitions in the file.
    formations: Vec<GroupFormation>,
}

/// A single formation definition specifying how units should be arranged.
///
/// Each formation includes AI usage criteria, unit requirements, and a set of
/// formation blocks that define where different unit types should be positioned.
#[derive(Default, PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct GroupFormation {
    /// Name identifier for this formation.
    name: String,

    /// AI priority value for selecting this formation (higher = more preferred).
    ai_priority: f32,

    /// Bitflags indicating when this formation should be used (attack, defend, naval, etc.).
    ai_purpose: AIPurposeCommon,

    /// Unknown field, present in Three Kingdoms.
    uk_2: u32,

    /// Minimum percentage requirements for unit categories in this formation.
    min_unit_category_percentage: Vec<MinUnitCategoryPercentage>,

    /// List of supported subcultures (introduced in Rome 2).
    ///
    /// If non-empty, this formation is only available to armies from these subcultures.
    ai_supported_subcultures: Vec<String>,

    /// List of supported factions (introduced in Rome 2).
    ///
    /// If non-empty, this formation is only available to armies from these factions.
    ai_supported_factions: Vec<String>,

    /// Formation blocks defining unit positions and arrangements.
    group_formation_blocks: Vec<GroupFormationBlock>,
}

/// Specifies a minimum percentage requirement for a unit category in a formation.
///
/// For example, a formation might require at least 30% cavalry units.
#[derive(Default, PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct MinUnitCategoryPercentage {
    /// The unit category (cavalry, infantry melee, infantry ranged, etc.).
    category: UnitCategoryCommon,

    /// Minimum percentage (0-100) of the army that must belong to this category.
    percentage: u32,
}

/// A formation block defining a specific position or region in the formation.
///
/// Each block has an ID and contains either absolute positioning, relative positioning
/// to another block, or spans multiple other blocks.
#[derive(Default, PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct GroupFormationBlock {
    /// Unique identifier for this block, used for relative positioning references.
    block_id: u32,

    /// The block type and its associated data.
    block: Block,
}

/// Types of formation blocks that can be used to define unit positions.
///
/// - `ContainerAbsolute`: Positioned at fixed coordinates.
/// - `ContainerRelative`: Positioned relative to another block.
/// - `Spanning`: Encompasses multiple other blocks.
#[derive(PartialEq, Clone, Debug, Serialize, Deserialize)]
pub enum Block {
    /// A container positioned at absolute coordinates.
    ContainerAbsolute(ContainerAbsolute),

    /// A container positioned relative to another block.
    ContainerRelative(ContainerRelative),

    /// A spanning block that encompasses multiple other blocks.
    Spanning(Spanning)
}

/// A container block positioned at absolute coordinates on the battlefield.
///
/// Defines how units should be arranged at a specific location, including their
/// spacing, arrangement pattern, and which types of units should occupy this position.
#[derive(Default, PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct ContainerAbsolute {
    /// Priority for filling this block (higher priority blocks are filled first).
    block_priority: f32,

    /// How units should be arranged (line, column, crescent, etc.).
    entity_arrangement: EntityArrangementCommon,

    /// Spacing between units in this block.
    inter_entity_spacing: f32,

    /// Y-axis offset for crescent formations.
    crescent_y_offset: f32,

    /// X coordinate of this block's position.
    position_x: f32,

    /// Y coordinate of this block's position.
    position_y: f32,

    /// Minimum number of units required to use this block.
    minimum_entity_threshold: i32,

    /// Maximum number of units that can be placed in this block.
    maximum_entity_threshold: i32,

    /// Ordered list of preferred unit types for this block.
    entity_preferences: Vec<EntityPreference>,
}

/// A container block positioned relative to another block.
///
/// Similar to `ContainerAbsolute` but positioned at an offset from a reference block,
/// allowing formations to be built up from interconnected positioned blocks.
#[derive(Default, PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct ContainerRelative {
    /// Priority for filling this block (higher priority blocks are filled first).
    block_priority: f32,

    /// ID of the block this is positioned relative to.
    relative_block_id: u32,

    /// How units should be arranged (line, column, crescent, etc.).
    entity_arrangement: EntityArrangementCommon,

    /// Spacing between units in this block.
    inter_entity_spacing: f32,

    /// Y-axis offset for crescent formations.
    crescent_y_offset: f32,

    /// X offset relative to the reference block.
    position_x: f32,

    /// Y offset relative to the reference block.
    position_y: f32,

    /// Minimum number of units required to use this block.
    minimum_entity_threshold: i32,

    /// Maximum number of units that can be placed in this block.
    maximum_entity_threshold: i32,

    /// Ordered list of preferred unit types for this block.
    entity_preferences: Vec<EntityPreference>,
}

/// Defines a preference for a specific type of unit to occupy a formation block.
///
/// Multiple preferences can be defined in priority order, so the AI will try to place
/// the highest priority matching units first.
#[derive(Default, PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct EntityPreference {
    /// Priority for this entity type (higher = more preferred).
    priority: f32,

    /// The type of unit entity (infantry, cavalry, artillery, etc.).
    ///
    /// Note: This is called EntityClass in Rome 2 and EntityDescription in Shogun 2,
    /// but represents the same concept.
    entity: EntityCommon,

    /// Weight class of the unit (light, medium, heavy, etc.). Introduced in Rome 2.
    entity_weight: EntityWeightCommon,

    /// Unknown fields present in Three Kingdoms.
    uk_1: u32,
    /// Unknown field present in Three Kingdoms.
    uk_2: u32,
    /// Unknown field present in Three Kingdoms.
    uk_3: u32,

    /// Entity class string identifier (used in Three Kingdoms).
    entity_class: String,
}

/// A spanning block that encompasses multiple other blocks.
///
/// Used to group related blocks together for organization or special handling.
#[derive(Default, PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct Spanning {
    /// IDs of the blocks that this spanning block encompasses.
    spanned_block_ids: Vec<u32>,
}

/// Game-specific AI purpose flags indicating when a formation should be used.
///
/// Different games have different sets of available purposes. These are bitflags,
/// so a formation can have multiple purposes (e.g., both attack and naval).
#[derive(PartialEq, Clone, Debug, Serialize, Deserialize)]
pub enum AIPurposeCommon {
    /// Rome 2 (and later) AI purpose flags.
    Rome2(versions::rome_2::AIPurpose),

    /// Shogun 2 AI purpose flags.
    Shogun2(versions::shogun_2::AIPurpose),
}

/// Game-specific entity arrangement patterns.
///
/// Defines how units should be laid out within a formation block (line, column, crescent, etc.).
#[derive(PartialEq, Clone, Debug, Serialize, Deserialize)]
pub enum EntityArrangementCommon {
    /// Rome 2 entity arrangement.
    Rome2(versions::rome_2::EntityArrangement),

    /// Shogun 2 entity arrangement.
    Shogun2(versions::shogun_2::EntityArrangement),
}

/// Game-specific unit category classifications.
///
/// Categorizes units by their role (cavalry, infantry melee, infantry ranged, naval, etc.).
#[derive(PartialEq, Clone, Debug, Serialize, Deserialize)]
pub enum UnitCategoryCommon {
    /// Rome 2 unit category.
    Rome2(versions::rome_2::UnitCategory),

    /// Shogun 2 unit category.
    Shogun2(versions::shogun_2::UnitCategory),
}

/// Game-specific entity type classifications.
///
/// More specific than unit categories, identifying exact unit types
/// (e.g., infantry spearman, cavalry lancer, artillery field).
#[derive(PartialEq, Clone, Debug, Serialize, Deserialize)]
pub enum EntityCommon {
    /// Rome 2 entity type.
    Rome2(versions::rome_2::Entity),

    /// Shogun 2 entity type.
    Shogun2(versions::shogun_2::Entity),
}

/// Game-specific entity weight classifications.
///
/// Categorizes units by their weight class (very light, light, medium, heavy, very heavy, super heavy).
#[derive(PartialEq, Clone, Debug, Serialize, Deserialize)]
pub enum EntityWeightCommon {
    /// Rome 2 entity weight.
    Rome2(versions::rome_2::EntityWeight),

    /// Attila entity weight (uses same format as Rome 2).
    Attila(versions::rome_2::EntityWeight),
}

//---------------------------------------------------------------------------//
//                          Implementation of GroupFormations
//---------------------------------------------------------------------------//

impl Default for Block {
    fn default() -> Self {
        Self::ContainerAbsolute(ContainerAbsolute::default())
    }
}

impl Default for AIPurposeCommon {
    fn default() -> Self {
        Self::Shogun2(versions::shogun_2::AIPurpose::default())
    }
}

impl Default for EntityArrangementCommon {
    fn default() -> Self {
        Self::Shogun2(versions::shogun_2::EntityArrangement::default())
    }
}

impl Default for UnitCategoryCommon {
    fn default() -> Self {
        Self::Shogun2(versions::shogun_2::UnitCategory::default())
    }
}

impl Default for EntityCommon {
    fn default() -> Self {
        Self::Shogun2(versions::shogun_2::Entity::default())
    }
}

impl Default for EntityWeightCommon {
    fn default() -> Self {
        Self::Rome2(versions::rome_2::EntityWeight::default())
    }
}

impl Decodeable for GroupFormations {

    fn decode<R: ReadBytes>(data: &mut R, extra_data: &Option<DecodeableExtraData>) -> Result<Self> {
        let extra_data = extra_data.as_ref().ok_or(RLibError::DecodingMissingExtraData)?;
        let game_info = extra_data.game_info.ok_or_else(|| RLibError::DecodingMissingExtraDataField("game_info".to_owned()))?;

        let mut decoded = Self::default();
        let data_len = data.len()?;

        match game_info.key() {
            //KEY_WARHAMMER_3 |
            //KEY_TROY |
            //KEY_THREE_KINGDOMS => decoded.decode_3k(data)?,
            //KEY_WARHAMMER_2 |
            //KEY_WARHAMMER |
            //KEY_THRONES_OF_BRITANNIA |
            //KEY_ATTILA |
            KEY_ROME_2 => decoded.decode_rom_2(data)?,
            KEY_SHOGUN_2 => decoded.decode_sho_2(data)?,
            //KEY_NAPOLEON |
            //KEY_EMPIRE => data.read_sized_string_u16()?,
            _ => return Err(RLibError::DecodingUnsupportedGameSelected(game_info.key().to_string())),
        }

        check_size_mismatch(data.stream_position()? as usize, data_len as usize)?;

        Ok(decoded)
    }
}

impl Encodeable for GroupFormations {

    fn encode<W: WriteBytes>(&mut self, buffer: &mut W, extra_data: &Option<EncodeableExtraData>) -> Result<()> {
        let extra_data = extra_data.as_ref().ok_or(RLibError::EncodingMissingExtraData)?;
        let game_info = extra_data.game_info.ok_or_else(|| RLibError::DecodingMissingExtraDataField("game_info".to_owned()))?;

        match game_info.key() {
            //KEY_WARHAMMER_3 |
            //KEY_TROY |
            //KEY_THREE_KINGDOMS => self.encode_3k(buffer)?,
            //KEY_WARHAMMER_2 |
            //KEY_WARHAMMER |
            //KEY_THRONES_OF_BRITANNIA |
            //KEY_ATTILA |
            KEY_ROME_2 => self.encode_rom_2(buffer)?,
            KEY_SHOGUN_2 => self.encode_sho_2(buffer)?,
            //KEY_NAPOLEON |
            //KEY_EMPIRE => buffer.write_sized_string_u16(formation.name())?,
            _ => return Err(RLibError::DecodingUnsupportedGameSelected(game_info.key().to_string())),
        };

        Ok(())
    }
}
