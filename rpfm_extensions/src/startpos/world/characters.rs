//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Per-character values of the `CHARACTER` record that come from the tables: birth year, loyalty and where the
//! character starts on the map.
//!
//! Verified on Attila's main campaign: birth years and loyalty match for 332 of 334 linked generals, placements for
//! 332 of 334 (two Sassanid generals linked to an occupied Ctesiphon are moved by the game to a free spot nearby).

use std::collections::HashSet;
use std::ops::RangeInclusive;

use rpfm_lib::error::Result;

use super::{CampaignIndex, CharacterEntry};
use crate::startpos::coords::HexCell;
use crate::startpos::tables::StartposTables;

#[cfg(test)] #[path = "characters_test.rs"] mod characters_test;

/// Range the startpos clamps the table loyalty to.
pub const LOYALTY_RANGE: RangeInclusive<i64> = 3..=10;

//---------------------------------------------------------------------------//
//                              Types
//---------------------------------------------------------------------------//

/// Where a character starts on the campaign map.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Placement {

    /// At the cell given by `startx`/`starty` (see [`HexCell::world_position`]).
    Cell(HexCell),

    /// At the settlement of a region (the first `start_pos_character_to_settlements` link owned by the faction).
    Settlement(String),

    /// With the faction's first general's army (factions without capital).
    FirstArmy,

    /// Off the map, at world position (0, 0).
    OffMap,
}

//---------------------------------------------------------------------------//
//                              Implementations
//---------------------------------------------------------------------------//

/// Returns a character's birth year: the campaign's start year minus its age.
pub fn birth_year(start_year: i64, character: &CharacterEntry) -> i64 {
    start_year - character.age()
}

/// Returns the loyalty the startpos stores: the table value clamped to [`LOYALTY_RANGE`].
pub fn loyalty(character: &CharacterEntry) -> i64 {
    character.loyalty().clamp(*LOYALTY_RANGE.start(), *LOYALTY_RANGE.end())
}

/// Returns where a character starts on the campaign map. An army stands where its general starts.
///
/// # Arguments
///
/// * `tables` - Tables with `start_pos_character_to_settlements`.
/// * `index` - The campaign's startpos rows.
/// * `character` - The character.
///
/// # Returns
///
/// The cell given in the table if any; otherwise the first linked settlement owned by the character's faction
/// (links to unknown settlements are ignored); otherwise the faction's first army for factions without capital;
/// otherwise off the map.
///
/// # Errors
///
/// Returns an error if a table or column is missing.
pub fn placement(tables: &StartposTables, index: &CampaignIndex, character: &CharacterEntry) -> Result<Placement> {
    if character.start_x() != 0 || character.start_y() != 0 {
        return Ok(Placement::Cell(HexCell::new(character.start_x(), character.start_y())));
    }

    let owned = index.regions_of(character.faction()).map(|region| region.key().as_str()).collect::<HashSet<_>>();
    for row in tables.table("start_pos_character_to_settlements")?.rows() {
        if row.str("character")? != *character.id() {
            continue;
        }
        let settlement_id = row.str("settlement")?;
        if let Some(settlement) = index.settlements().iter().find(|settlement| *settlement.id() == settlement_id) {
            if owned.contains(settlement.region().as_str()) {
                return Ok(Placement::Settlement(settlement.region().to_owned()));
            }
        }
    }

    if index.capital_of(character.faction()).is_none() {
        return Ok(Placement::FirstArmy);
    }

    Ok(Placement::OffMap)
}
