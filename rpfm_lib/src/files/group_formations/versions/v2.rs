//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Shared data types for v2 (Rome 2 and later) group formations.
//!
//! Used by: Rome 2, Attila, Warhammer, Warhammer 2, Thrones of Britannia,
//! Three Kingdoms, Troy, Pharaoh, Warhammer 3.

use std::fmt::Display;

use bitflags::bitflags;
use serde_derive::{Serialize, Deserialize};

use crate::error::{Result, RLibError};

bitflags! {

    #[derive(Default, PartialEq, Clone, Debug, Serialize, Deserialize)]
    pub struct AIPurposeFlags: u32 {
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

/// Entity types for Rome 2 and later formations.
///
/// These are abstract unit class identifiers shared across Rome 2, Attila,
/// Warhammer series, Three Kingdoms, Troy, and Pharaoh.
#[derive(Default, Clone, Copy, PartialEq, Debug, Serialize, Deserialize)]
#[repr(u32)]
pub enum EntityType {
    InfMel = 0,
    InfSpr = 1,
    InfPik = 2,
    InfMis = 3,
    #[default] Com = 4,
    CavShk = 5,
    CavMel = 6,
    CavMis = 7,
    Chariot = 8,
    Elph = 9,
    Spcl = 10,
    ArtFld = 11,
    ArtFix = 12,
    ArtSiege = 13,
    ShpMel = 14,
    ShpMis = 15,
    ShpArt = 16,
    ShpTrn = 17,
    ShpStk = 18,
    ShpFir = 19,
    Invalid = 20,
    Uk21 = 21,
    Uk22 = 22,
    Uk23 = 23,
    Uk24 = 24,
}

impl TryFrom<u32> for EntityType {
    type Error = RLibError;
    fn try_from(value: u32) -> Result<Self> {
        match value {
            _ if value == Self::InfMel as u32 => Ok(Self::InfMel),
            _ if value == Self::InfSpr as u32 => Ok(Self::InfSpr),
            _ if value == Self::InfPik as u32 => Ok(Self::InfPik),
            _ if value == Self::InfMis as u32 => Ok(Self::InfMis),
            _ if value == Self::Com as u32 => Ok(Self::Com),
            _ if value == Self::CavShk as u32 => Ok(Self::CavShk),
            _ if value == Self::CavMel as u32 => Ok(Self::CavMel),
            _ if value == Self::CavMis as u32 => Ok(Self::CavMis),
            _ if value == Self::Chariot as u32 => Ok(Self::Chariot),
            _ if value == Self::Elph as u32 => Ok(Self::Elph),
            _ if value == Self::Spcl as u32 => Ok(Self::Spcl),
            _ if value == Self::ArtFld as u32 => Ok(Self::ArtFld),
            _ if value == Self::ArtFix as u32 => Ok(Self::ArtFix),
            _ if value == Self::ArtSiege as u32 => Ok(Self::ArtSiege),
            _ if value == Self::ShpMel as u32 => Ok(Self::ShpMel),
            _ if value == Self::ShpMis as u32 => Ok(Self::ShpMis),
            _ if value == Self::ShpArt as u32 => Ok(Self::ShpArt),
            _ if value == Self::ShpTrn as u32 => Ok(Self::ShpTrn),
            _ if value == Self::ShpStk as u32 => Ok(Self::ShpStk),
            _ if value == Self::ShpFir as u32 => Ok(Self::ShpFir),
            _ if value == Self::Invalid as u32 => Ok(Self::Invalid),
            _ if value == Self::Uk21 as u32 => Ok(Self::Uk21),
            _ if value == Self::Uk22 as u32 => Ok(Self::Uk22),
            _ if value == Self::Uk23 as u32 => Ok(Self::Uk23),
            _ if value == Self::Uk24 as u32 => Ok(Self::Uk24),
            _ => Err(RLibError::DecodingGroupFormationsUnknownEnumValue("EntityType(v2)".to_string(), value)),
        }
    }
}

impl From<EntityType> for u32 {
    fn from(value: EntityType) -> u32 {
        value as u32
    }
}

impl Display for EntityType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InfMel => write!(f, "Missile Infantry"),
            Self::InfSpr => write!(f, "Spear Infantry"),
            Self::InfPik => write!(f, "Pike Infantry"),
            Self::InfMis => write!(f, "Missile Infantry"),
            Self::Com => write!(f, "Command"),
            Self::CavShk => write!(f, "Shock Cavalry"),
            Self::CavMel => write!(f, "Melee Cavalry"),
            Self::CavMis => write!(f, "Missile Cavalry"),
            Self::Chariot => write!(f, "Chariot"),
            Self::Elph => write!(f, "Elephant"),
            Self::Spcl => write!(f, "Special"),
            Self::ArtFld => write!(f, "Field Artillery"),
            Self::ArtFix => write!(f, "Fixed Artillery"),
            Self::ArtSiege => write!(f, "Siege Artillery"),
            Self::ShpMel => write!(f, "Melee Ship"),
            Self::ShpMis => write!(f, "Missile Ship"),
            Self::ShpArt => write!(f, "Artillery Ship"),
            Self::ShpTrn => write!(f, "Transport Ship"),
            Self::ShpStk => write!(f, "Ramming Ship"),
            Self::ShpFir => write!(f, "Fire Ship"),
            Self::Invalid => write!(f, "Invalid"),
            Self::Uk21 => write!(f, "Uk21"),
            Self::Uk22 => write!(f, "Uk22"),
            Self::Uk23 => write!(f, "Uk23"),
            Self::Uk24 => write!(f, "Uk24"),
        }
    }
}
