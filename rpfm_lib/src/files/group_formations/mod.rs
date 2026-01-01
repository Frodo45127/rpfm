//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! NOTE: This file type is not versioned. Meaning we have logic for each game supported in different files.

use bitflags::bitflags;
use getset::*;
use serde_derive::{Serialize, Deserialize};

use crate::binary::{ReadBytes, WriteBytes};
use crate::error::{Result, RLibError};
use crate::files::{Decodeable, EncodeableExtraData, Encodeable};
use crate::games::supported_games::*;
use crate::utils::*;

use super::DecodeableExtraData;

pub const PATH: &str = "groupformations.bin";

mod versions;

#[cfg(test)] mod test_group_formations;

//---------------------------------------------------------------------------//
//                              Enum & Structs
//---------------------------------------------------------------------------//

/// This holds an entire `GroupFormations` file decoded in memory.
#[derive(Default, PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct GroupFormations {
    formations: Vec<GroupFormation>,
}

#[derive(Default, PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct GroupFormation {
    name: String,

    ai_priority: f32,
    ai_purpose: AIPurposeCommon,

    // These two are in 3k.
    uk_2: u32,

    min_unit_category_percentage: Vec<MinUnitCategoryPercentage>,

    // Introduced in rome 2.
    ai_supported_subcultures: Vec<String>,
    ai_supported_factions: Vec<String>,

    group_formation_blocks: Vec<GroupFormationBlock>,
}

#[derive(Default, PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct MinUnitCategoryPercentage {
    category: UnitCategoryCommon,
    percentage: u32,
}

#[derive(Default, PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct GroupFormationBlock {
    block_id: u32,
    block: Block,
}

#[derive(PartialEq, Clone, Debug, Serialize, Deserialize)]
pub enum Block {
    ContainerAbsolute(ContainerAbsolute),
    ContainerRelative(ContainerRelative),
    Spanning(Spanning)
}

#[derive(Default, PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct ContainerAbsolute {
    block_priority: f32,
    entity_arrangement: EntityArrangementCommon,
    inter_entity_spacing: f32,
    crescent_y_offset: f32,
    position_x: f32,
    position_y: f32,
    minimum_entity_threshold: i32,
    maximum_entity_threshold: i32,
    entity_preferences: Vec<EntityPreference>,
}

#[derive(Default, PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct ContainerRelative {
    block_priority: f32,
    relative_block_id: u32,
    entity_arrangement: EntityArrangementCommon,
    inter_entity_spacing: f32,
    crescent_y_offset: f32,
    position_x: f32,
    position_y: f32,
    minimum_entity_threshold: i32,
    maximum_entity_threshold: i32,
    entity_preferences: Vec<EntityPreference>,
}

#[derive(Default, PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct EntityPreference {
    priority: f32,

    // NOTE: This is called EntityClass in Rome 2, EntityDescription in Shogun 2, but seems to be the same thing.
    entity: EntityCommon,

    // This is from Rome 2.
    entity_weight: EntityWeightCommon,

    // This is from 3k.
    uk_1: u32,
    uk_2: u32,
    uk_3: u32,
    entity_class: String,
}

#[derive(Default, PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct Spanning {
    spanned_block_ids: Vec<u32>,
}

#[derive(PartialEq, Clone, Debug, Serialize, Deserialize)]
pub enum AIPurposeCommon {
    Rome2(versions::rome_2::AIPurpose),
    Shogun2(versions::shogun_2::AIPurpose),
}

#[derive(PartialEq, Clone, Debug, Serialize, Deserialize)]
pub enum EntityArrangementCommon {
    Rome2(versions::rome_2::EntityArrangement),
    Shogun2(versions::shogun_2::EntityArrangement),
}

#[derive(PartialEq, Clone, Debug, Serialize, Deserialize)]
pub enum UnitCategoryCommon {
    Rome2(versions::rome_2::UnitCategory),
    Shogun2(versions::shogun_2::UnitCategory),
}

#[derive(PartialEq, Clone, Debug, Serialize, Deserialize)]
pub enum EntityCommon {
    Rome2(versions::rome_2::Entity),
    Shogun2(versions::shogun_2::Entity),
}

#[derive(PartialEq, Clone, Debug, Serialize, Deserialize)]
pub enum EntityWeightCommon {
    Rome2(versions::rome_2::EntityWeight),
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
