//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Shared data types for v1 (Shogun 2) group formations.

use bitflags::bitflags;
use serde_derive::{Serialize, Deserialize};

use crate::error::{Result, RLibError};

bitflags! {

    // Note: the UK ones may not even exist, but we left them there just in case a game uses them.
    #[derive(Default, PartialEq, Clone, Debug, Serialize, Deserialize)]
    pub struct AIPurposeFlags: u32 {
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

/// Entity types for Shogun 2 formations.
///
/// These are specific unit class identifiers used in Shogun 2.
#[derive(Default, Clone, Copy, PartialEq, Debug, Serialize, Deserialize)]
#[repr(u32)]
pub enum EntityType {
    ArtilleryFixed = 0,
    ArtilleryFoot = 1,
    ArtilleryHorse = 2,
    CavalryCamels = 3,
    CavalryHeavy = 4,
    CavalryIrregular = 5,
    CavalryLancers = 6,
    CavalryLight = 7,
    CavalryMissile = 8,
    CavalryStandard = 9,
    Dragoons = 10,
    Elephants = 11,
    General = 12,
    InfantryBerserker = 13,
    InfantryElite = 14,
    InfantryGrenadiers = 15,
    InfantryIrregulars = 16,
    InfantryLight = 17,
    InfantryLine = 18,
    InfantryMelee = 19,
    InfantryMilitia = 20,
    InfantryMob = 21,
    InfantrySkirmishers = 22,
    NavalAdmiral = 23,
    NavalBombKetch = 24,
    NavalBrig = 25,
    NavalDhow = 26,
    NavalFifthRate = 27,
    NavalFirstRate = 28,
    NavalFourthRate = 29,
    NavalHeavyGalley = 30,
    NavalIndiaman = 31,
    NavalLightGalley = 32,
    NavalLugger = 33,
    NavalMediumGalley = 34,
    NavalOverFirstRate = 35,
    NavalRazee = 36,
    NavalRocketShip = 37,
    NavalSecondRate = 38,
    NavalSixthRate = 39,
    NavalSloop = 40,
    NavalSteamShip = 41,
    NavalThirdRate = 42,
    NavalXebec = 43,
    InfantrySpearman = 45,
    InfantryHeavy = 46,
    InfantrySpecial = 47,
    InfantryBow = 48,
    InfantryMatchlock = 49,
    InfantrySword = 50,
    Siege = 51,
    CavalrySword = 52,
    NavalHeavyShip = 54,
    NavalMediumShip = 55,
    NavalLightShip = 56,
    NavalCannonShip = 57,
    NavalGalleon = 58,
    NavalIronclad = 60,
    NavalCorvette = 61,
    NavalFrigate = 62,
    NavalGunboat = 63,
    NavalTorpedoboat = 64,
    #[default] Any = 65,
}

impl TryFrom<u32> for EntityType {
    type Error = RLibError;
    fn try_from(value: u32) -> Result<Self> {
        match value {
            _ if value == Self::ArtilleryFixed as u32 => Ok(Self::ArtilleryFixed),
            _ if value == Self::ArtilleryFoot as u32 => Ok(Self::ArtilleryFoot),
            _ if value == Self::ArtilleryHorse as u32 => Ok(Self::ArtilleryHorse),
            _ if value == Self::CavalryCamels as u32 => Ok(Self::CavalryCamels),
            _ if value == Self::CavalryHeavy as u32 => Ok(Self::CavalryHeavy),
            _ if value == Self::CavalryIrregular as u32 => Ok(Self::CavalryIrregular),
            _ if value == Self::CavalryLancers as u32 => Ok(Self::CavalryLancers),
            _ if value == Self::CavalryLight as u32 => Ok(Self::CavalryLight),
            _ if value == Self::CavalryMissile as u32 => Ok(Self::CavalryMissile),
            _ if value == Self::CavalryStandard as u32 => Ok(Self::CavalryStandard),
            _ if value == Self::Dragoons as u32 => Ok(Self::Dragoons),
            _ if value == Self::Elephants as u32 => Ok(Self::Elephants),
            _ if value == Self::General as u32 => Ok(Self::General),
            _ if value == Self::InfantryBerserker as u32 => Ok(Self::InfantryBerserker),
            _ if value == Self::InfantryElite as u32 => Ok(Self::InfantryElite),
            _ if value == Self::InfantryGrenadiers as u32 => Ok(Self::InfantryGrenadiers),
            _ if value == Self::InfantryIrregulars as u32 => Ok(Self::InfantryIrregulars),
            _ if value == Self::InfantryLight as u32 => Ok(Self::InfantryLight),
            _ if value == Self::InfantryLine as u32 => Ok(Self::InfantryLine),
            _ if value == Self::InfantryMelee as u32 => Ok(Self::InfantryMelee),
            _ if value == Self::InfantryMilitia as u32 => Ok(Self::InfantryMilitia),
            _ if value == Self::InfantryMob as u32 => Ok(Self::InfantryMob),
            _ if value == Self::InfantrySkirmishers as u32 => Ok(Self::InfantrySkirmishers),
            _ if value == Self::NavalAdmiral as u32 => Ok(Self::NavalAdmiral),
            _ if value == Self::NavalBombKetch as u32 => Ok(Self::NavalBombKetch),
            _ if value == Self::NavalBrig as u32 => Ok(Self::NavalBrig),
            _ if value == Self::NavalDhow as u32 => Ok(Self::NavalDhow),
            _ if value == Self::NavalFifthRate as u32 => Ok(Self::NavalFifthRate),
            _ if value == Self::NavalFirstRate as u32 => Ok(Self::NavalFirstRate),
            _ if value == Self::NavalFourthRate as u32 => Ok(Self::NavalFourthRate),
            _ if value == Self::NavalHeavyGalley as u32 => Ok(Self::NavalHeavyGalley),
            _ if value == Self::NavalIndiaman as u32 => Ok(Self::NavalIndiaman),
            _ if value == Self::NavalLightGalley as u32 => Ok(Self::NavalLightGalley),
            _ if value == Self::NavalLugger as u32 => Ok(Self::NavalLugger),
            _ if value == Self::NavalMediumGalley as u32 => Ok(Self::NavalMediumGalley),
            _ if value == Self::NavalOverFirstRate as u32 => Ok(Self::NavalOverFirstRate),
            _ if value == Self::NavalRazee as u32 => Ok(Self::NavalRazee),
            _ if value == Self::NavalRocketShip as u32 => Ok(Self::NavalRocketShip),
            _ if value == Self::NavalSecondRate as u32 => Ok(Self::NavalSecondRate),
            _ if value == Self::NavalSixthRate as u32 => Ok(Self::NavalSixthRate),
            _ if value == Self::NavalSloop as u32 => Ok(Self::NavalSloop),
            _ if value == Self::NavalSteamShip as u32 => Ok(Self::NavalSteamShip),
            _ if value == Self::NavalThirdRate as u32 => Ok(Self::NavalThirdRate),
            _ if value == Self::NavalXebec as u32 => Ok(Self::NavalXebec),
            _ if value == Self::InfantrySpearman as u32 => Ok(Self::InfantrySpearman),
            _ if value == Self::InfantryHeavy as u32 => Ok(Self::InfantryHeavy),
            _ if value == Self::InfantrySpecial as u32 => Ok(Self::InfantrySpecial),
            _ if value == Self::InfantryBow as u32 => Ok(Self::InfantryBow),
            _ if value == Self::InfantryMatchlock as u32 => Ok(Self::InfantryMatchlock),
            _ if value == Self::InfantrySword as u32 => Ok(Self::InfantrySword),
            _ if value == Self::Siege as u32 => Ok(Self::Siege),
            _ if value == Self::CavalrySword as u32 => Ok(Self::CavalrySword),
            _ if value == Self::NavalHeavyShip as u32 => Ok(Self::NavalHeavyShip),
            _ if value == Self::NavalMediumShip as u32 => Ok(Self::NavalMediumShip),
            _ if value == Self::NavalLightShip as u32 => Ok(Self::NavalLightShip),
            _ if value == Self::NavalCannonShip as u32 => Ok(Self::NavalCannonShip),
            _ if value == Self::NavalGalleon as u32 => Ok(Self::NavalGalleon),
            _ if value == Self::NavalIronclad as u32 => Ok(Self::NavalIronclad),
            _ if value == Self::NavalCorvette as u32 => Ok(Self::NavalCorvette),
            _ if value == Self::NavalFrigate as u32 => Ok(Self::NavalFrigate),
            _ if value == Self::NavalGunboat as u32 => Ok(Self::NavalGunboat),
            _ if value == Self::NavalTorpedoboat as u32 => Ok(Self::NavalTorpedoboat),
            _ if value == Self::Any as u32 => Ok(Self::Any),
            _ => Err(RLibError::DecodingGroupFormationsUnknownEnumValue("EntityType(v1)".to_string(), value)),
        }
    }
}

impl From<EntityType> for u32 {
    fn from(value: EntityType) -> u32 {
        value as u32
    }
}
