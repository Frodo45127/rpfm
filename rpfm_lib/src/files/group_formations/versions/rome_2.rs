//---------------------------------------------------------------------------//
// Copyright (c) 2017-2023 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

use crate::binary::{ReadBytes, WriteBytes};
use crate::error::Result;

use super::*;

bitflags! {

    #[derive(Default, PartialEq, Clone, Debug, Serialize, Deserialize)]
    pub struct AIPurpose: u32 {
        const ATTACK                        = 0b0000_0000_0000_0000_0000_0000_0000_0001;
        const DEFEND                        = 0b0000_0000_0000_0000_0000_0000_0000_0010;
        const RIVER_ATTACK                  = 0b0000_0000_0000_0000_0000_0000_0000_0100;
        const RIVER_DEFEND                  = 0b0000_0000_0000_0000_0000_0000_0000_1000;
        const UK_4                          = 0b0000_0000_0000_0000_0000_0000_0001_0000;
        const AMBUSH_DEFENCE_BLOCK          = 0b0000_0000_0000_0000_0000_0000_0010_0000;
        const SETTLEMENT_ASSAULT            = 0b0000_0000_0000_0000_0000_0000_0100_0000;
        const SETTLEMENT_AREA_DEFEND_NARROW = 0b0000_0000_0000_0000_0000_0000_1000_0000;
        const SETTLEMENT_AREA_ATTACK_NARROW = 0b0000_0000_0000_0000_0000_0001_0000_0000;
        const UK_9                          = 0b0000_0000_0000_0000_0000_0010_0000_0000;
        const NAVAL_ATTACK                  = 0b0000_0000_0000_0000_0000_0100_0000_0000;
        const NAVAL_DEFEND                  = 0b0000_0000_0000_0000_0000_1000_0000_0000;
        const DEFAULT_DEPLOYMENT            = 0b0000_0000_0000_0000_0001_0000_0000_0000;
        const NAVAL_DEFAULT_DEPLOYMENT      = 0b0000_0000_0000_0000_0010_0000_0000_0000;
        const UK_14                         = 0b0000_0000_0000_0000_0100_0000_0000_0000;
        const UK_15                         = 0b0000_0000_0000_0000_1000_0000_0000_0000;
        const UK_16                         = 0b0000_0000_0000_0001_0000_0000_0000_0000;
        const UK_17                         = 0b0000_0000_0000_0010_0000_0000_0000_0000;
        const UK_18                         = 0b0000_0000_0000_0100_0000_0000_0000_0000;
        const UK_19                         = 0b0000_0000_0000_1000_0000_0000_0000_0000;
        const UK_20                         = 0b0000_0000_0001_0000_0000_0000_0000_0000;
        const UK_21                         = 0b0000_0000_0010_0000_0000_0000_0000_0000;
        const UK_22                         = 0b0000_0000_0100_0000_0000_0000_0000_0000;
        const UK_23                         = 0b0000_0000_1000_0000_0000_0000_0000_0000;
        const UK_24                         = 0b0000_0001_0000_0000_0000_0000_0000_0000;
        const UK_25                         = 0b0000_0010_0000_0000_0000_0000_0000_0000;
        const UK_26                         = 0b0000_0100_0000_0000_0000_0000_0000_0000;
        const UK_27                         = 0b0000_1000_0000_0000_0000_0000_0000_0000;
        const UK_28                         = 0b0001_0000_0000_0000_0000_0000_0000_0000;
        const UK_29                         = 0b0010_0000_0000_0000_0000_0000_0000_0000;
        const UK_30                         = 0b0100_0000_0000_0000_0000_0000_0000_0000;
        const UK_31                         = 0b1000_0000_0000_0000_0000_0000_0000_0000;
    }
}

//---------------------------------------------------------------------------//
//                              Enum & Structs
//---------------------------------------------------------------------------//

#[derive(Default, PartialEq, Clone, Debug, Serialize, Deserialize)]
pub enum EntityArrangement {
    #[default] Line, // 0
    Column, // 1
    CrescentFront, // 2, unused in rome 2 it seems.
    CrescentBack, // 3, unused in rome 2 it seems.
    Other(u32),
}

// Unused in rome 2 it seems. Copied from shogun 2, mapping may be incorrect or missing things.
#[derive(Default, PartialEq, Clone, Debug, Serialize, Deserialize)]
pub enum UnitCategory {
    #[default] Cavalry, // 0
    InvantryMelee, //13
    InfantryRanged, //14
    NavalHeavy, //15
    NavalMedium, //16
    NavalLight, //17
    Other(u32),
}

#[derive(Default, PartialEq, Clone, Debug, Serialize, Deserialize)]
pub enum Entity {
    InfMel, // 0
    InfSpr, // 1
    InfPik, // 2
    InfMis, // 3
    Com, // 4
    CavShk, // 5
    CavMel, // 6
    CavMis, // 7
    Chariot, // 8
    Elph, // 9
    Spcl, // 10
    ArtFld, // 11
    ArtFix, // 12

    ShpMel, // 14
    ShpMis, // 15
    ShpArt, // 16
    ShpTrn, // 17

    #[default] Any, // 18
    Other(u32),
}

#[derive(Default, PartialEq, Clone, Debug, Serialize, Deserialize)]
pub enum EntityWeight {
    VeryLight, // 0
    Light, // 1
    Medium, // 2
    Heavy, // 3
    VeyHeavy, // 4,
    SuperHeavy, // 5,
    #[default] Any, // 6
    Other(u32),
}

//---------------------------------------------------------------------------//
//                          Implementation of GroupFormations
//---------------------------------------------------------------------------//

impl From<u32> for EntityArrangement {
    fn from(value: u32) -> Self {
        match value {
            0 => Self::Line,
            1 => Self::Column,
            2 => Self::CrescentFront,
            3 => Self::CrescentBack,
            _ => Self::Other(value),
        }
    }
}

impl From<EntityArrangement> for u32 {
    fn from(value: EntityArrangement) -> u32 {
        match value {
            EntityArrangement::Line => 0,
            EntityArrangement::Column => 1,
            EntityArrangement::CrescentFront => 2,
            EntityArrangement::CrescentBack => 3,
            EntityArrangement::Other(value) => value,
        }
    }
}

impl From<u32> for UnitCategory {
    fn from(value: u32) -> Self {
        match value {
            0 => Self::Cavalry,
            13 => Self::InvantryMelee,
            14 => Self::InfantryRanged,
            15 => Self::NavalHeavy,
            16 => Self::NavalMedium,
            17 => Self::NavalLight,
            _ => Self::Other(value),
        }
    }
}

impl From<UnitCategory> for u32 {
    fn from(value: UnitCategory) -> u32 {
        match value {
            UnitCategory::Cavalry => 0,
            UnitCategory::InvantryMelee => 13,
            UnitCategory::InfantryRanged => 14,
            UnitCategory::NavalHeavy => 15,
            UnitCategory::NavalMedium => 16,
            UnitCategory::NavalLight => 17,
            UnitCategory::Other(value) => value,
        }
    }
}

impl From<u32> for Entity {
    fn from(value: u32) -> Self {
        match value {
            0 => Self::InfMel,
            1 => Self::InfSpr,
            2 => Self::InfPik,
            3 => Self::InfMis,
            4 => Self::Com,
            5 => Self::CavShk,
            6 => Self::CavMel,
            7 => Self::CavMis,
            8 => Self::Chariot,
            9 => Self::Elph,
            10 => Self::Spcl,
            11 => Self::ArtFld,
            12 => Self::ArtFix,
            14 => Self::ShpMel,
            15 => Self::ShpMis,
            16 => Self::ShpArt,
            17 => Self::ShpTrn,
            18 => Self::Any,
            _ => Self::Other(value),
        }
    }
}

impl From<Entity> for u32 {
    fn from(value: Entity) -> u32 {
        match value {
            Entity::InfMel => 0,
            Entity::InfSpr => 1,
            Entity::InfPik => 2,
            Entity::InfMis => 3,
            Entity::Com => 4,
            Entity::CavShk => 5,
            Entity::CavMel => 6,
            Entity::CavMis => 7,
            Entity::Chariot => 8,
            Entity::Elph => 9,
            Entity::Spcl => 10,
            Entity::ArtFld => 11,
            Entity::ArtFix => 12,
            Entity::ShpMel => 14,
            Entity::ShpMis => 15,
            Entity::ShpArt => 16,
            Entity::ShpTrn => 17,
            Entity::Any => 18,
            Entity::Other(value) => value,
        }
    }
}

impl From<u32> for EntityWeight {
    fn from(value: u32) -> Self {
        match value {
            0 => Self::VeryLight,
            1 => Self::Light,
            2 => Self::Medium,
            3 => Self::Heavy,
            4 => Self::VeyHeavy,
            5 => Self::SuperHeavy,
            6 => Self::Any,
            _ => Self::Other(value),
        }
    }
}

impl From<EntityWeight> for u32 {
    fn from(value: EntityWeight) -> u32 {
        match value {
            EntityWeight::VeryLight => 0,
            EntityWeight::Light => 1,
            EntityWeight::Medium => 2,
            EntityWeight::Heavy => 3,
            EntityWeight::VeyHeavy => 4,
            EntityWeight::SuperHeavy => 5,
            EntityWeight::Any => 6,
            EntityWeight::Other(value) => value,
        }
    }
}

impl GroupFormations {

    pub(crate) fn decode_rom_2<R: ReadBytes>(&mut self, data: &mut R) -> Result<()> {

        //GroupFormation
        for _ in 0..data.read_u32()? {
            let mut formation = GroupFormation::default();
            formation.name = data.read_sized_string_u8()?;
            formation.ai_priority = data.read_f32()?;
            formation.ai_purpose = AIPurposeCommon::Rome2(AIPurpose::from_bits_truncate(data.read_u32()?));

            // MinUnitCategoryPercentage is one of these
            for _ in 0..data.read_u32()? {
                let mut min_unit_category_percentage = MinUnitCategoryPercentage::default();

                min_unit_category_percentage.category = UnitCategoryCommon::Rome2(UnitCategory::from(data.read_u32()?));
                min_unit_category_percentage.percentage = data.read_u32()?;

                formation.min_unit_category_percentage.push(min_unit_category_percentage);
            }

            for _ in 0..data.read_u32()? {
                formation.ai_supported_subcultures.push(data.read_sized_string_u8()?);
            }

            for _ in 0..data.read_u32()? {
                formation.ai_supported_factions.push(data.read_sized_string_u8()?);
            }

            // GroupFormationBlock
            for _ in 0..data.read_u32()? {
                let mut block = GroupFormationBlock::default();
                block.block_id = data.read_u32()?;

                // Possible enum: 0 absolute, 1 relative, 3 spanning
                let block_type = data.read_u32()?;
                match block_type {

                    // ContainerAbsolute
                    0 => {
                        let mut container = ContainerAbsolute::default();
                        container.block_priority = data.read_f32()?;
                        container.entity_arrangement = EntityArrangementCommon::Rome2(EntityArrangement::from(data.read_u32()?));
                        container.inter_entity_spacing = data.read_f32()?;
                        container.crescent_y_offset = data.read_f32()?;
                        container.position_x = data.read_f32()?;
                        container.position_y = data.read_f32()?;
                        container.minimum_entity_threshold = data.read_i32()?;
                        container.maximum_entity_threshold = data.read_i32()?;

                        // EntityPreference
                        for _ in 0..data.read_u32()? {
                            let mut entity_pref = EntityPreference::default();
                            entity_pref.priority = data.read_f32()?;
                            entity_pref.entity = EntityCommon::Rome2(Entity::from(data.read_u32()?));
                            entity_pref.entity_weight = EntityWeightCommon::Rome2(EntityWeight::from(data.read_u32()?));

                            container.entity_preferences.push(entity_pref);
                        }

                        block.block = Block::ContainerAbsolute(container);
                    },

                    // ContainerRelative
                    1 => {
                        let mut container = ContainerRelative::default();
                        container.block_priority = data.read_f32()?;
                        container.relative_block_id = data.read_u32()?;
                        container.entity_arrangement = EntityArrangementCommon::Rome2(EntityArrangement::from(data.read_u32()?));
                        container.inter_entity_spacing = data.read_f32()?;
                        container.crescent_y_offset = data.read_f32()?;
                        container.position_x = data.read_f32()?;
                        container.position_y = data.read_f32()?;
                        container.minimum_entity_threshold = data.read_i32()?;
                        container.maximum_entity_threshold = data.read_i32()?;

                        // EntityPreference
                        for _ in 0..data.read_u32()? {
                            let mut entity_pref = EntityPreference::default();
                            entity_pref.priority = data.read_f32()?;
                            entity_pref.entity = EntityCommon::Rome2(Entity::from(data.read_u32()?));
                            entity_pref.entity_weight = EntityWeightCommon::Rome2(EntityWeight::from(data.read_u32()?));

                            container.entity_preferences.push(entity_pref);
                        }

                        block.block = Block::ContainerRelative(container);
                    },

                    // Spanning
                    3 => {
                        let mut container = Spanning::default();
                        for _ in 0..data.read_u32()? {
                            container.spanned_block_ids.push(data.read_u32()?);
                        }
                        block.block = Block::Spanning(container);
                    },
                    _ => todo!("unknown block type {}.", block_type),
                }

                formation.group_formation_blocks.push(block);
            }

            self.formations.push(formation);
        }

        Ok(())
    }

    pub(crate) fn encode_rom_2<W: WriteBytes>(&mut self, buffer: &mut W) -> Result<()> {
        buffer.write_u32(self.formations.len() as u32)?;
        for formation in self.formations() {
            buffer.write_sized_string_u8(formation.name())?;

            buffer.write_f32(formation.ai_priority)?;
            if let AIPurposeCommon::Rome2(data) = &formation.ai_purpose {
                buffer.write_u32(data.bits())?;
            }

            buffer.write_u32(formation.min_unit_category_percentage.len() as u32)?;
            for min_unit_category_percentage in formation.min_unit_category_percentage() {
                if let UnitCategoryCommon::Rome2(data) = &min_unit_category_percentage.category {
                    buffer.write_u32(data.clone().into())?;
                }
                buffer.write_u32(min_unit_category_percentage.percentage)?;
            }

            buffer.write_u32(formation.ai_supported_subcultures.len() as u32)?;
            for ai_supported_subculture in formation.ai_supported_subcultures() {
                buffer.write_sized_string_u8(ai_supported_subculture)?;
            }

            buffer.write_u32(formation.ai_supported_factions.len() as u32)?;
            for ai_supported_faction in formation.ai_supported_factions() {
                buffer.write_sized_string_u8(ai_supported_faction)?;
            }

            buffer.write_u32(formation.group_formation_blocks.len() as u32)?;
            for block in formation.group_formation_blocks() {
                buffer.write_u32(block.block_id)?;

                match block.block {
                    Block::ContainerAbsolute(ref block) => {
                        buffer.write_u32(0)?;

                        buffer.write_f32(block.block_priority)?;
                        if let EntityArrangementCommon::Rome2(data) = &block.entity_arrangement {
                            buffer.write_u32(data.clone().into())?;
                        }
                        buffer.write_f32(block.inter_entity_spacing)?;
                        buffer.write_f32(block.crescent_y_offset)?;
                        buffer.write_f32(block.position_x)?;
                        buffer.write_f32(block.position_y)?;
                        buffer.write_i32(block.minimum_entity_threshold)?;
                        buffer.write_i32(block.maximum_entity_threshold)?;

                        buffer.write_u32(block.entity_preferences.len() as u32)?;
                        for ent_pref in block.entity_preferences() {
                            buffer.write_f32(ent_pref.priority)?;
                            if let EntityCommon::Rome2(data) = &ent_pref.entity {
                                buffer.write_u32(data.clone().into())?;
                            }
                            if let EntityWeightCommon::Rome2(data) = &ent_pref.entity_weight {
                                buffer.write_u32(data.clone().into())?;
                            }
                        }
                    },

                    Block::ContainerRelative(ref block) => {
                        buffer.write_u32(1)?;

                        buffer.write_f32(block.block_priority)?;
                        buffer.write_u32(block.relative_block_id)?;
                        if let EntityArrangementCommon::Rome2(data) = &block.entity_arrangement {
                            buffer.write_u32(data.clone().into())?;
                        }
                        buffer.write_f32(block.inter_entity_spacing)?;
                        buffer.write_f32(block.crescent_y_offset)?;
                        buffer.write_f32(block.position_x)?;
                        buffer.write_f32(block.position_y)?;
                        buffer.write_i32(block.minimum_entity_threshold)?;
                        buffer.write_i32(block.maximum_entity_threshold)?;

                        buffer.write_u32(block.entity_preferences.len() as u32)?;
                        for ent_pref in block.entity_preferences() {
                            buffer.write_f32(ent_pref.priority)?;
                            if let EntityCommon::Rome2(data) = &ent_pref.entity {
                                buffer.write_u32(data.clone().into())?;
                            }
                            if let EntityWeightCommon::Rome2(data) = &ent_pref.entity_weight {
                                buffer.write_u32(data.clone().into())?;
                            }
                        }
                    },

                    Block::Spanning(ref block) => {
                        buffer.write_u32(3)?;
                        buffer.write_u32(block.spanned_block_ids.len() as u32)?;
                        for id in block.spanned_block_ids() {
                            buffer.write_u32(*id)?;
                        }
                    },
                }
            }
        }

        Ok(())
    }
}
