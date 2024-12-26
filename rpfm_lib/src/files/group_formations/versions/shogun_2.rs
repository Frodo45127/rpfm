//---------------------------------------------------------------------------//
// Copyright (c) 2017-2024 Ismael Gutiérrez González. All rights reserved.
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

    // Note: the UK ones may not even exist, but we left them there just in case a game uses them.
    #[derive(Default, PartialEq, Clone, Debug, Serialize, Deserialize)]
    pub struct AIPurpose: u32 {
        const ATTACK                    = 0b0000_0000_0000_0000_0000_0000_0000_0001;
        const DEFEND                    = 0b0000_0000_0000_0000_0000_0000_0000_0010;
        const RIVER_ATTACK              = 0b0000_0000_0000_0000_0000_0000_0000_0100;
        const UK_3                      = 0b0000_0000_0000_0000_0000_0000_0000_1000;
        const UK_4                      = 0b0000_0000_0000_0000_0000_0000_0001_0000;
        const NAVAL_ATTACK              = 0b0000_0000_0000_0000_0000_0000_0010_0000;
        const NAVAL_DEFEND              = 0b0000_0000_0000_0000_0000_0000_0100_0000;
        const DEFAULT_DEPLOYMENT        = 0b0000_0000_0000_0000_0000_0000_1000_0000;
        const NAVAL_DEFAULT_DEPLOYMENT  = 0b0000_0000_0000_0000_0000_0001_0000_0000;
        const LARGE_MAP_ONLY            = 0b0000_0000_0000_0000_0000_0010_0000_0000;
        const UK_10                     = 0b0000_0000_0000_0000_0000_0100_0000_0000;
        const UK_11                     = 0b0000_0000_0000_0000_0000_1000_0000_0000;
        const UK_12                     = 0b0000_0000_0000_0000_0001_0000_0000_0000;
        const UK_13                     = 0b0000_0000_0000_0000_0010_0000_0000_0000;
        const UK_14                     = 0b0000_0000_0000_0000_0100_0000_0000_0000;
        const UK_15                     = 0b0000_0000_0000_0000_1000_0000_0000_0000;
        const UK_16                     = 0b0000_0000_0000_0001_0000_0000_0000_0000;
        const UK_17                     = 0b0000_0000_0000_0010_0000_0000_0000_0000;
        const UK_18                     = 0b0000_0000_0000_0100_0000_0000_0000_0000;
        const UK_19                     = 0b0000_0000_0000_1000_0000_0000_0000_0000;
        const UK_20                     = 0b0000_0000_0001_0000_0000_0000_0000_0000;
        const UK_21                     = 0b0000_0000_0010_0000_0000_0000_0000_0000;
        const UK_22                     = 0b0000_0000_0100_0000_0000_0000_0000_0000;
        const UK_23                     = 0b0000_0000_1000_0000_0000_0000_0000_0000;
        const UK_24                     = 0b0000_0001_0000_0000_0000_0000_0000_0000;
        const UK_25                     = 0b0000_0010_0000_0000_0000_0000_0000_0000;
        const UK_26                     = 0b0000_0100_0000_0000_0000_0000_0000_0000;
        const UK_27                     = 0b0000_1000_0000_0000_0000_0000_0000_0000;
        const UK_28                     = 0b0001_0000_0000_0000_0000_0000_0000_0000;
        const UK_29                     = 0b0010_0000_0000_0000_0000_0000_0000_0000;
        const UK_30                     = 0b0100_0000_0000_0000_0000_0000_0000_0000;
        const UK_31                     = 0b1000_0000_0000_0000_0000_0000_0000_0000;
    }
}

//---------------------------------------------------------------------------//
//                              Enum & Structs
//---------------------------------------------------------------------------//

#[derive(Default, PartialEq, Clone, Debug, Serialize, Deserialize)]
pub enum EntityArrangement {
    #[default] Line, // 0
    Column, // 1
    CrescentFront, // 2
    CrescentBack, // 3
    Other(u32),
}

// No clue from where are these ids retrieved from, as they're not in the db. At least in Shogun 2.
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
    ArtilleryFixed, // 0
    ArtilleryFoot, // 1
    ArtilleryHorse, // 2
    CavalryCamels, // 3
    CavalryHeavy, // 4
    CavalryIrregular, // 5
    CavalryLancers, // 6
    CavalryLight, // 7
    CavalryMissile, // 8
    CavalryStandard, // 9
    Dragoons, // 10
    Elephants, // 11
    General, // 12
    InfantryBerserker, // 13
    InfantryElite, // 14
    InfantryGrenadiers, // 15
    InfantryIrregulars, // 16
    InfantryLight, // 17
    InfantryLine, // 18
    InfantryMelee, // 19
    InfantryMilitia, // 20
    InfantryMob, // 21
    InfantrySkirmishers, // 22
    NavalAdmiral, // 23
    NavalBombKetch, // 24
    NavalBrig, // 25
    NavalDhow, // 26
    NavalFifthRate, // 27
    NavalFirstRate, // 28
    NavalFourthRate, // 29
    NavalHeavyGalley, // 30
    NavalIndiaman, // 31
    NavalLightGalley, // 32
    NavalLugger, // 33
    NavalMediumGalley, // 34
    NavalOverFirstRate, // 35
    NavalRazee, // 36
    NavalRocketShip, // 37
    NavalSecondRate, // 38
    NavalSixthRate, // 39
    NavalSloop, // 40
    NavalSteamShip, // 41
    NavalThirdRate, // 42
    NavalXebec, // 43
    InfantrySpearman, // 45
    InfantryHeavy, // 46
    InfantrySpecial, // 47
    InfantryBow, // 48
    InfantryMatchlock, // 49
    InfantrySword, // 50
    Siege, // 51
    CavalrySword, // 52
    NavalHeavyShip, // 54
    NavalMediumShip, // 55
    NavalLightShip, // 56
    NavalCannonShip, // 57
    NavalGalleon, // 58
    NavalIronclad, // 60
    NavalCorvette, // 61
    NavalFrigate, // 62
    NavalGunboat, // 63
    NavalTorpedoboat, // 64

    #[default] Any, // 65
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
            0 => Self::ArtilleryFixed,
            1 => Self::ArtilleryFoot,
            2 => Self::ArtilleryHorse,
            3 => Self::CavalryCamels,
            4 => Self::CavalryHeavy,
            5 => Self::CavalryIrregular,
            6 => Self::CavalryLancers,
            7 => Self::CavalryLight,
            8 => Self::CavalryMissile,
            9 => Self::CavalryStandard,
            10 => Self::Dragoons,
            11 => Self::Elephants,
            12 => Self::General,
            13 => Self::InfantryBerserker,
            14 => Self::InfantryElite,
            15 => Self::InfantryGrenadiers,
            16 => Self::InfantryIrregulars,
            17 => Self::InfantryLight,
            18 => Self::InfantryLine,
            19 => Self::InfantryMelee,
            20 => Self::InfantryMilitia,
            21 => Self::InfantryMob,
            22 => Self::InfantrySkirmishers,
            23 => Self::NavalAdmiral,
            24 => Self::NavalBombKetch,
            25 => Self::NavalBrig,
            26 => Self::NavalDhow,
            27 => Self::NavalFifthRate,
            28 => Self::NavalFirstRate,
            29 => Self::NavalFourthRate,
            30 => Self::NavalHeavyGalley,
            31 => Self::NavalIndiaman,
            32 => Self::NavalLightGalley,
            33 => Self::NavalLugger,
            34 => Self::NavalMediumGalley,
            35 => Self::NavalOverFirstRate,
            36 => Self::NavalRazee,
            37 => Self::NavalRocketShip,
            38 => Self::NavalSecondRate,
            39 => Self::NavalSixthRate,
            40 => Self::NavalSloop,
            41 => Self::NavalSteamShip,
            42 => Self::NavalThirdRate,
            43 => Self::NavalXebec,
            45 => Self::InfantrySpearman,
            46 => Self::InfantryHeavy,
            47 => Self::InfantrySpecial,
            48 => Self::InfantryBow,
            49 => Self::InfantryMatchlock,
            50 => Self::InfantrySword,
            51 => Self::Siege,
            52 => Self::CavalrySword,
            54 => Self::NavalHeavyShip,
            55 => Self::NavalMediumShip,
            56 => Self::NavalLightShip,
            57 => Self::NavalCannonShip,
            58 => Self::NavalGalleon,
            60 => Self::NavalIronclad,
            61 => Self::NavalCorvette,
            62 => Self::NavalFrigate,
            63 => Self::NavalGunboat,
            64 => Self::NavalTorpedoboat,
            65 => Self::Any,
            _ => Self::Other(value),
        }
    }
}

impl From<Entity> for u32 {
    fn from(value: Entity) -> u32 {
        match value {
            Entity::ArtilleryFixed => 0,
            Entity::ArtilleryFoot => 1,
            Entity::ArtilleryHorse => 2,
            Entity::CavalryCamels => 3,
            Entity::CavalryHeavy => 4,
            Entity::CavalryIrregular => 5,
            Entity::CavalryLancers => 6,
            Entity::CavalryLight => 7,
            Entity::CavalryMissile => 8,
            Entity::CavalryStandard => 9,
            Entity::Dragoons => 10,
            Entity::Elephants => 11,
            Entity::General => 12,
            Entity::InfantryBerserker => 13,
            Entity::InfantryElite => 14,
            Entity::InfantryGrenadiers => 15,
            Entity::InfantryIrregulars => 16,
            Entity::InfantryLight => 17,
            Entity::InfantryLine => 18,
            Entity::InfantryMelee => 19,
            Entity::InfantryMilitia => 20,
            Entity::InfantryMob => 21,
            Entity::InfantrySkirmishers => 22,
            Entity::NavalAdmiral => 23,
            Entity::NavalBombKetch => 24,
            Entity::NavalBrig => 25,
            Entity::NavalDhow => 26,
            Entity::NavalFifthRate => 27,
            Entity::NavalFirstRate => 28,
            Entity::NavalFourthRate => 29,
            Entity::NavalHeavyGalley => 30,
            Entity::NavalIndiaman => 31,
            Entity::NavalLightGalley => 32,
            Entity::NavalLugger => 33,
            Entity::NavalMediumGalley => 34,
            Entity::NavalOverFirstRate => 35,
            Entity::NavalRazee => 36,
            Entity::NavalRocketShip => 37,
            Entity::NavalSecondRate => 38,
            Entity::NavalSixthRate => 39,
            Entity::NavalSloop => 40,
            Entity::NavalSteamShip => 41,
            Entity::NavalThirdRate => 42,
            Entity::NavalXebec => 43,
            Entity::InfantrySpearman => 45,
            Entity::InfantryHeavy => 46,
            Entity::InfantrySpecial => 47,
            Entity::InfantryBow => 48,
            Entity::InfantryMatchlock => 49,
            Entity::InfantrySword => 50,
            Entity::Siege => 51,
            Entity::CavalrySword => 52,
            Entity::NavalHeavyShip => 54,
            Entity::NavalMediumShip => 55,
            Entity::NavalLightShip => 56,
            Entity::NavalCannonShip => 57,
            Entity::NavalGalleon => 58,
            Entity::NavalIronclad => 60,
            Entity::NavalCorvette => 61,
            Entity::NavalFrigate => 62,
            Entity::NavalGunboat => 63,
            Entity::NavalTorpedoboat => 64,
            Entity::Any => 65,
            Entity::Other(value) => value,
        }
    }
}

impl GroupFormations {

    pub(crate) fn decode_sho_2<R: ReadBytes>(&mut self, data: &mut R) -> Result<()> {

        //GroupFormation
        for _ in 0..data.read_u32()? {
            let mut formation = GroupFormation::default();
            formation.name = data.read_sized_string_u16()?;
            formation.ai_priority = data.read_f32()?;
            formation.ai_purpose = AIPurposeCommon::Shogun2(AIPurpose::from_bits_truncate(data.read_u32()?));

            // MinUnitCategoryPercentage is one of these
            for _ in 0..data.read_u32()? {
                let mut min_unit_category_percentage = MinUnitCategoryPercentage::default();

                min_unit_category_percentage.category = UnitCategoryCommon::Shogun2(UnitCategory::from(data.read_u32()?));
                min_unit_category_percentage.percentage = data.read_u32()?;

                formation.min_unit_category_percentage.push(min_unit_category_percentage);
            }

            // AiSupportedFaction is the other one.
            for _ in 0..data.read_u32()? {
                formation.ai_supported_factions.push(data.read_sized_string_u16()?);
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
                        container.entity_arrangement = EntityArrangementCommon::Shogun2(EntityArrangement::from(data.read_u32()?));
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
                            entity_pref.entity = EntityCommon::Shogun2(Entity::from(data.read_u32()?));

                            container.entity_preferences.push(entity_pref);
                        }

                        block.block = Block::ContainerAbsolute(container);
                    },

                    // ContainerRelative
                    1 => {
                        let mut container = ContainerRelative::default();
                        container.block_priority = data.read_f32()?;
                        container.relative_block_id = data.read_u32()?;
                        container.entity_arrangement = EntityArrangementCommon::Shogun2(EntityArrangement::from(data.read_u32()?));
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
                            entity_pref.entity = EntityCommon::Shogun2(Entity::from(data.read_u32()?));

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
                    _ => return Err(RLibError::GroupFormationUnknownBlockType(block_type)),
                }

                formation.group_formation_blocks.push(block);
            }

            self.formations.push(formation);
        }

        Ok(())
    }

    pub(crate) fn encode_sho_2<W: WriteBytes>(&mut self, buffer: &mut W) -> Result<()> {
        buffer.write_u32(self.formations.len() as u32)?;
        for formation in self.formations() {
            buffer.write_sized_string_u16(formation.name())?;

            buffer.write_f32(formation.ai_priority)?;
            if let AIPurposeCommon::Shogun2(data) = &formation.ai_purpose {
                buffer.write_u32(data.bits())?;
            }

            buffer.write_u32(formation.min_unit_category_percentage.len() as u32)?;
            for min_unit_category_percentage in formation.min_unit_category_percentage() {
                if let UnitCategoryCommon::Shogun2(data) = &min_unit_category_percentage.category {
                    buffer.write_u32(data.clone().into())?;
                }
                buffer.write_u32(min_unit_category_percentage.percentage)?;
            }

            buffer.write_u32(formation.ai_supported_factions.len() as u32)?;
            for ai_supported_faction in formation.ai_supported_factions() {
                buffer.write_sized_string_u16(ai_supported_faction)?;
            }

            buffer.write_u32(formation.group_formation_blocks.len() as u32)?;
            for block in formation.group_formation_blocks() {
                buffer.write_u32(block.block_id)?;

                match block.block {
                    Block::ContainerAbsolute(ref block) => {
                        buffer.write_u32(0)?;

                        buffer.write_f32(block.block_priority)?;
                        if let EntityArrangementCommon::Shogun2(data) = &block.entity_arrangement {
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
                            if let EntityCommon::Shogun2(data) = &ent_pref.entity {
                                buffer.write_u32(data.clone().into())?;
                            }
                        }
                    },

                    Block::ContainerRelative(ref block) => {
                        buffer.write_u32(1)?;

                        buffer.write_f32(block.block_priority)?;
                        buffer.write_u32(block.relative_block_id)?;
                        if let EntityArrangementCommon::Shogun2(data) = &block.entity_arrangement {
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
                            if let EntityCommon::Shogun2(data) = &ent_pref.entity {
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
