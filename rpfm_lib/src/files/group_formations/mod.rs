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

use getset::*;
use serde_derive::{Serialize, Deserialize};

use std::fmt::Display;

use crate::binary::{ReadBytes, WriteBytes};
use crate::error::{Result, RLibError};
use crate::files::{Decodeable, EncodeableExtraData, Encodeable};
use crate::games::{GameInfo, supported_games::*};
use crate::utils::*;

use super::DecodeableExtraData;

/// Fixed path to the Group Formations file.
pub const PATH: &str = "groupformations.bin";

pub mod edit;
pub mod layout;
pub mod validation;
pub mod versions;

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
    ai_purpose: AIPurpose,

    /// Unknown field, present in Troy and later.
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
    category: UnitCategory,

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
    entity_arrangement: EntityArrangement,

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
    entity_arrangement: EntityArrangement,

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
    entity: Entity,

    /// Weight class of the unit (light, medium, heavy, etc.). Introduced in Rome 2.
    entity_weight: EntityWeight,

    /// Unknown field, present in Troy and later. Vanilla files almost always use 6.
    uk_1: u32,

    /// Entity class string identifier, present in Troy and later.
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

/// Binary layout of a Group Formations file, which depends on the game it belongs to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GroupFormationsFormat {

    /// Shogun 2.
    Shogun2,

    /// Rome 2, Attila and Thrones of Britannia.
    Rome2,

    /// Troy and Pharaoh.
    Troy,

    /// Three Kingdoms and Warhammer 3.
    Warhammer3,
}

/// AI purpose flags indicating when a formation should be used.
///
/// - V1: Shogun 2 flag layout (different bit assignments from Rome 2+).
/// - V2: Rome 2 and later flag layout.
#[derive(PartialEq, Clone, Debug, Serialize, Deserialize)]
pub enum AIPurpose {
    V1(versions::v1::AIPurposeFlags),
    V2(versions::v2::AIPurposeFlags),
}

/// How units should be arranged within a formation block (line, column, crescent, etc.).
///
/// Identical across all game versions.
#[derive(Default, Clone, Copy, PartialEq, Debug, Serialize, Deserialize)]
#[repr(u32)]
pub enum EntityArrangement {
    #[default] Line = 0,
    Column = 1,
    CrescentFront = 2,
    CrescentBack = 3,
}

/// Unit category classifications (cavalry, infantry melee, infantry ranged, naval, etc.).
///
/// Identical across all game versions.
#[derive(Default, Clone, Copy, PartialEq, Debug, Serialize, Deserialize)]
#[repr(u32)]
pub enum UnitCategory {
    #[default] Cavalry = 0,
    InfantryMelee = 13,
    InfantryRanged = 14,
    NavalHeavy = 15,
    NavalMedium = 16,
    NavalLight = 17,
}

/// Entity type classifications.
///
/// - V1: Shogun 2 entity types (65+ specific unit classes like CavalryHeavy, InfantryLine, etc.).
/// - V2: Rome 2 and later entity types (18 abstract classes like InfMel, CavShk, etc.).
#[derive(PartialEq, Clone, Debug, Serialize, Deserialize)]
pub enum Entity {
    V1(versions::v1::EntityType),
    V2(versions::v2::EntityType),
}

/// Entity weight classifications (light, medium, heavy, etc.).
///
/// Introduced in Rome 2. Identical across all post-Shogun 2 game versions.
/// Shogun 2 does not use entity weights.
#[derive(Default, Clone, Copy, PartialEq, Debug, Serialize, Deserialize)]
#[repr(u32)]
pub enum EntityWeight {
    VeryLight = 0,
    Light = 1,
    Medium = 2,
    Heavy = 3,
    VeryHeavy = 4,
    SuperHeavy = 5,
    #[default] Any = 6,
}

//---------------------------------------------------------------------------//
//                          Implementation of GroupFormations
//---------------------------------------------------------------------------//

impl GroupFormationsFormat {

    /// Returns the format used by the provided game.
    ///
    /// # Arguments
    ///
    /// * `game_info` - The game the file belongs to.
    ///
    /// # Returns
    ///
    /// The format of the game's Group Formations file.
    ///
    /// # Errors
    ///
    /// Returns an error if the game doesn't support Group Formations files.
    pub fn from_game(game_info: &GameInfo) -> Result<Self> {
        match game_info.key() {
            KEY_PHARAOH_DYNASTIES |
            KEY_PHARAOH |
            KEY_TROY => Ok(Self::Troy),
            KEY_THREE_KINGDOMS |
            KEY_WARHAMMER_3 => Ok(Self::Warhammer3),
            KEY_THRONES_OF_BRITANNIA |
            KEY_ATTILA |
            KEY_ROME_2 => Ok(Self::Rome2),
            KEY_SHOGUN_2 => Ok(Self::Shogun2),

            // Warhammer, Warhammer 2, Napoleon and Empire formats are not yet researched.
            _ => Err(RLibError::DecodingUnsupportedGameSelected(game_info.key().to_string())),
        }
    }

    /// Returns if formations in this format have a list of supported factions.
    pub fn has_ai_supported_factions(&self) -> bool {
        matches!(self, Self::Shogun2 | Self::Rome2 | Self::Warhammer3)
    }

    /// Returns if formations in this format have a list of supported subcultures.
    pub fn has_ai_supported_subcultures(&self) -> bool {
        matches!(self, Self::Rome2 | Self::Troy | Self::Warhammer3)
    }

    /// Returns if formations in this format have the `uk_2` field.
    pub fn has_formation_uk_2(&self) -> bool {
        matches!(self, Self::Troy | Self::Warhammer3)
    }

    /// Returns if entity preferences in this format have an entity weight.
    pub fn has_entity_weight(&self) -> bool {
        matches!(self, Self::Rome2 | Self::Troy | Self::Warhammer3)
    }

    /// Returns if entity preferences in this format have the `uk_1` and `entity_class` fields.
    pub fn has_entity_class(&self) -> bool {
        matches!(self, Self::Troy | Self::Warhammer3)
    }

    /// Returns an empty set of AI purpose flags of the version used by this format.
    pub fn default_ai_purpose(&self) -> AIPurpose {
        match self {
            Self::Shogun2 => AIPurpose::V1(versions::v1::AIPurposeFlags::empty()),
            Self::Rome2 | Self::Troy | Self::Warhammer3 => AIPurpose::V2(versions::v2::AIPurposeFlags::empty()),
        }
    }

    /// Returns the default entity type of the version used by this format.
    pub fn default_entity(&self) -> Entity {
        match self {
            Self::Shogun2 => Entity::V1(versions::v1::EntityType::Any),
            Self::Rome2 => Entity::V2(versions::v2::EntityType::InfMel),

            // Vanilla files pair the "any" entity class with this value.
            Self::Troy | Self::Warhammer3 => Entity::V2(versions::v2::EntityType::Invalid),
        }
    }
}

impl GroupFormation {

    /// Creates an empty formation valid for the provided format.
    ///
    /// # Arguments
    ///
    /// * `name` - Name of the new formation.
    /// * `format` - Format of the file the formation will belong to.
    ///
    /// # Returns
    ///
    /// A formation with no blocks and the AI purpose version expected by the format.
    pub fn new(name: &str, format: GroupFormationsFormat) -> Self {
        Self {
            name: name.to_owned(),
            ai_purpose: format.default_ai_purpose(),
            ..Default::default()
        }
    }
}

impl EntityPreference {

    /// Creates an entity preference valid for the provided format, with the defaults used by vanilla files.
    ///
    /// # Arguments
    ///
    /// * `format` - Format of the file the preference will belong to.
    ///
    /// # Returns
    ///
    /// An entity preference with the entity version expected by the format.
    pub fn new(format: GroupFormationsFormat) -> Self {
        Self {
            priority: 1.0,
            entity: format.default_entity(),
            entity_weight: EntityWeight::Any,
            uk_1: if format.has_entity_class() { 6 } else { 0 },
            entity_class: if format.has_entity_class() { "any".to_owned() } else { String::new() },
        }
    }
}

impl AIPurpose {

    /// Returns the raw bits of these flags, if they're the V1 (Shogun 2) version.
    pub(crate) fn v1_bits(&self) -> Result<u32> {
        match self {
            Self::V1(flags) => Ok(flags.bits()),
            Self::V2(_) => Err(RLibError::EncodingGroupFormationsMismatchedVersion("AIPurpose".to_owned())),
        }
    }

    /// Returns the raw bits of these flags, if they're the V2 (Rome 2 and later) version.
    pub(crate) fn v2_bits(&self) -> Result<u32> {
        match self {
            Self::V2(flags) => Ok(flags.bits()),
            Self::V1(_) => Err(RLibError::EncodingGroupFormationsMismatchedVersion("AIPurpose".to_owned())),
        }
    }
}

impl Entity {

    /// Returns the raw value of this entity, if it's the V1 (Shogun 2) version.
    pub(crate) fn v1_value(&self) -> Result<u32> {
        match self {
            Self::V1(entity) => Ok((*entity).into()),
            Self::V2(_) => Err(RLibError::EncodingGroupFormationsMismatchedVersion("Entity".to_owned())),
        }
    }

    /// Returns the raw value of this entity, if it's the V2 (Rome 2 and later) version.
    pub(crate) fn v2_value(&self) -> Result<u32> {
        match self {
            Self::V2(entity) => Ok((*entity).into()),
            Self::V1(_) => Err(RLibError::EncodingGroupFormationsMismatchedVersion("Entity".to_owned())),
        }
    }
}

impl EntityArrangement {

    /// All the possible values, in their binary order.
    pub const ALL: [Self; 4] = [Self::Line, Self::Column, Self::CrescentFront, Self::CrescentBack];
}

impl UnitCategory {

    /// All the possible values, in their binary order.
    pub const ALL: [Self; 6] = [Self::Cavalry, Self::InfantryMelee, Self::InfantryRanged, Self::NavalHeavy, Self::NavalMedium, Self::NavalLight];
}

impl EntityWeight {

    /// All the possible values, in their binary order.
    pub const ALL: [Self; 7] = [Self::VeryLight, Self::Light, Self::Medium, Self::Heavy, Self::VeryHeavy, Self::SuperHeavy, Self::Any];
}

impl Display for EntityArrangement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Line => write!(f, "Line"),
            Self::Column => write!(f, "Column"),
            Self::CrescentFront => write!(f, "Crescent Front"),
            Self::CrescentBack => write!(f, "Crescent Back"),
        }
    }
}

impl Display for UnitCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Cavalry => write!(f, "Cavalry"),
            Self::InfantryMelee => write!(f, "Melee Infantry"),
            Self::InfantryRanged => write!(f, "Ranged Infantry"),
            Self::NavalHeavy => write!(f, "Heavy Naval"),
            Self::NavalMedium => write!(f, "Medium Naval"),
            Self::NavalLight => write!(f, "Light Naval"),
        }
    }
}

impl Display for EntityWeight {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::VeryLight => write!(f, "Very Light"),
            Self::Light => write!(f, "Light"),
            Self::Medium => write!(f, "Medium"),
            Self::Heavy => write!(f, "Heavy"),
            Self::VeryHeavy => write!(f, "Very Heavy"),
            Self::SuperHeavy => write!(f, "Super Heavy"),
            Self::Any => write!(f, "Any"),
        }
    }
}

impl Block {

    /// Returns the ids of the blocks this block needs to be positioned first.
    ///
    /// # Returns
    ///
    /// The parent block for relative containers, the spanned blocks for spans, and nothing for absolute containers.
    pub fn dependencies(&self) -> &[u32] {
        match self {
            Self::ContainerAbsolute(_) => &[],
            Self::ContainerRelative(container) => std::slice::from_ref(&container.relative_block_id),
            Self::Spanning(span) => &span.spanned_block_ids,
        }
    }
}

impl Default for Block {
    fn default() -> Self {
        Self::ContainerAbsolute(ContainerAbsolute::default())
    }
}

impl Default for AIPurpose {
    fn default() -> Self {
        Self::V1(versions::v1::AIPurposeFlags::default())
    }
}

impl Default for Entity {
    fn default() -> Self {
        Self::V2(versions::v2::EntityType::default())
    }
}

impl TryFrom<u32> for EntityArrangement {
    type Error = RLibError;
    fn try_from(value: u32) -> Result<Self> {
        match value {
            _ if value == Self::Line as u32 => Ok(Self::Line),
            _ if value == Self::Column as u32 => Ok(Self::Column),
            _ if value == Self::CrescentFront as u32 => Ok(Self::CrescentFront),
            _ if value == Self::CrescentBack as u32 => Ok(Self::CrescentBack),
            _ => Err(RLibError::DecodingGroupFormationsUnknownEnumValue("EntityArrangement".to_string(), value)),
        }
    }
}

impl From<EntityArrangement> for u32 {
    fn from(value: EntityArrangement) -> u32 {
        value as u32
    }
}

impl TryFrom<u32> for UnitCategory {
    type Error = RLibError;
    fn try_from(value: u32) -> Result<Self> {
        match value {
            _ if value == Self::Cavalry as u32 => Ok(Self::Cavalry),
            _ if value == Self::InfantryMelee as u32 => Ok(Self::InfantryMelee),
            _ if value == Self::InfantryRanged as u32 => Ok(Self::InfantryRanged),
            _ if value == Self::NavalHeavy as u32 => Ok(Self::NavalHeavy),
            _ if value == Self::NavalMedium as u32 => Ok(Self::NavalMedium),
            _ if value == Self::NavalLight as u32 => Ok(Self::NavalLight),
            _ => Err(RLibError::DecodingGroupFormationsUnknownEnumValue("UnitCategory".to_string(), value)),
        }
    }
}

impl From<UnitCategory> for u32 {
    fn from(value: UnitCategory) -> u32 {
        value as u32
    }
}

impl TryFrom<u32> for EntityWeight {
    type Error = RLibError;
    fn try_from(value: u32) -> Result<Self> {
        match value {
            _ if value == Self::VeryLight as u32 => Ok(Self::VeryLight),
            _ if value == Self::Light as u32 => Ok(Self::Light),
            _ if value == Self::Medium as u32 => Ok(Self::Medium),
            _ if value == Self::Heavy as u32 => Ok(Self::Heavy),
            _ if value == Self::VeryHeavy as u32 => Ok(Self::VeryHeavy),
            _ if value == Self::SuperHeavy as u32 => Ok(Self::SuperHeavy),
            _ if value == Self::Any as u32 => Ok(Self::Any),
            _ => Err(RLibError::DecodingGroupFormationsUnknownEnumValue("EntityWeight".to_string(), value)),
        }
    }
}

impl From<EntityWeight> for u32 {
    fn from(value: EntityWeight) -> u32 {
        value as u32
    }
}

impl Decodeable for GroupFormations {

    fn decode<R: ReadBytes>(data: &mut R, extra_data: &Option<DecodeableExtraData>) -> Result<Self> {
        let extra_data = extra_data.as_ref().ok_or(RLibError::DecodingMissingExtraData)?;
        let game_info = extra_data.game_info.ok_or_else(|| RLibError::DecodingMissingExtraDataField("game_info".to_owned()))?;

        let mut decoded = Self::default();
        let data_len = data.len()?;

        match GroupFormationsFormat::from_game(game_info)? {
            GroupFormationsFormat::Troy => decoded.decode_troy(data)?,
            GroupFormationsFormat::Warhammer3 => decoded.decode_wh3(data)?,
            GroupFormationsFormat::Rome2 => decoded.decode_rom_2(data)?,
            GroupFormationsFormat::Shogun2 => decoded.decode_sho_2(data)?,
        }

        check_size_mismatch(data.stream_position()? as usize, data_len as usize)?;

        Ok(decoded)
    }
}

impl Encodeable for GroupFormations {

    fn encode<W: WriteBytes>(&mut self, buffer: &mut W, extra_data: &Option<EncodeableExtraData>) -> Result<()> {
        let extra_data = extra_data.as_ref().ok_or(RLibError::EncodingMissingExtraData)?;
        let game_info = extra_data.game_info.ok_or_else(|| RLibError::DecodingMissingExtraDataField("game_info".to_owned()))?;

        match GroupFormationsFormat::from_game(game_info)? {
            GroupFormationsFormat::Troy => self.encode_troy(buffer)?,
            GroupFormationsFormat::Warhammer3 => self.encode_wh3(buffer)?,
            GroupFormationsFormat::Rome2 => self.encode_rom_2(buffer)?,
            GroupFormationsFormat::Shogun2 => self.encode_sho_2(buffer)?,
        };

        Ok(())
    }
}
