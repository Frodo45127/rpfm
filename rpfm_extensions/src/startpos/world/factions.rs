//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Per-faction values of the `FACTION` record that come from the tables: flags and colours
//! (`FACTION_FLAG_AND_COLOURS`), capital, owned regions, victory regions, overlord and subjects, horde state and
//! progenitor.
//!
//! Verified on Attila's main campaign: 118 of 118 factions match. Victory regions match as sets (the game stores
//! 6 of them in another order).

use getset::{CopyGetters, Getters};

use rpfm_lib::error::{RLibError, Result};

use super::{CampaignIndex, FactionEntry};
use crate::startpos::tables::{Row, StartposTables};

#[cfg(test)] #[path = "factions_test.rs"] mod factions_test;

//---------------------------------------------------------------------------//
//                              Types
//---------------------------------------------------------------------------//

/// A flag with its colours (`FACTION_FLAG_AND_COLOURS`).
#[derive(Debug, Clone, PartialEq, Eq, Default, Getters, CopyGetters)]
pub struct FactionFlag {

    /// Flag folder path.
    #[getset(get = "pub")]
    path: String,

    /// Primary, secondary and uniform colours, as RGB.
    #[getset(get_copy = "pub")]
    colours: [[u8; 3]; 3],
}

/// Per-faction values derived from the tables.
#[derive(Debug, Clone, PartialEq, Eq, Default, Getters, CopyGetters)]
pub struct FactionSummary {

    /// Faction key.
    #[getset(get = "pub")]
    key: String,

    /// Capital region key.
    #[getset(get = "pub")]
    capital: Option<String>,

    /// Number of owned regions.
    #[getset(get_copy = "pub")]
    region_count: usize,

    /// `start_pos_victory_conditions` regions of the faction, in table order.
    #[getset(get = "pub")]
    victory_regions: Vec<String>,

    /// Faction this one is a client state of.
    #[getset(get = "pub")]
    client_state_of: Option<String>,

    /// Faction this one is a vassal of.
    #[getset(get = "pub")]
    vassal_of: Option<String>,

    /// Client states and vassals of the faction, in startpos faction order.
    #[getset(get = "pub")]
    subjects: Vec<String>,

    /// The faction is at war, or in a vassal relation (either side).
    #[getset(get_copy = "pub")]
    has_war_or_vassal: bool,

    /// For factions that can be hordes (`factions.can_be_horde`), whether they start as a horde (own no regions).
    #[getset(get_copy = "pub")]
    horde: Option<bool>,

    /// The faction has no living characters.
    #[getset(get_copy = "pub")]
    without_characters: bool,

    /// `start_pos_characters` id of the faction's progenitor, if any.
    #[getset(get = "pub")]
    progenitor: Option<String>,
}

//---------------------------------------------------------------------------//
//                              Implementations
//---------------------------------------------------------------------------//

/// Returns the two flags of a faction: the normal one and the republican one.
///
/// # Arguments
///
/// * `tables` - Tables with `factions`.
/// * `faction` - Faction key.
///
/// # Returns
///
/// The flag from `flags_path` with the primary, secondary and uniform colours, and the flag from
/// `republican_flag_path` with the alternative colours.
///
/// # Errors
///
/// Returns an error if a table or column is missing, the faction isn't in `factions` or a colour isn't valid hex.
pub fn faction_flags(tables: &StartposTables, faction: &str) -> Result<[FactionFlag; 2]> {
    let row = faction_row(tables, faction)?;
    Ok([
        flag(&row, "flags_path", ["primary_colour_hex", "secondary_colour_hex", "uniform_colour_hex"])?,
        flag(&row, "republican_flag_path", ["alt_primary_colour_hex", "alt_secondary_colour_hex", "alt_uniform_colour_hex"])?,
    ])
}

/// Returns the per-faction values of a faction.
///
/// # Arguments
///
/// * `tables` - Tables with `start_pos_victory_conditions`, `start_pos_diplomacy` and `factions`.
/// * `index` - The campaign's startpos rows.
/// * `faction` - The faction.
///
/// # Errors
///
/// Returns an error if a table or column is missing.
pub fn faction_summary(tables: &StartposTables, index: &CampaignIndex, faction: &FactionEntry) -> Result<FactionSummary> {
    let key = faction.key();
    let mut summary = FactionSummary {
        key: key.to_owned(),
        capital: index.capital_of(key).map(|region| region.key().to_owned()),
        region_count: index.regions_of(key).count(),
        without_characters: index.characters_of(key).all(|character| !character.death_type().is_empty()),
        progenitor: index.characters_of(key).find(|character| character.progenitor()).map(|character| character.id().to_owned()),
        ..Default::default()
    };

    for row in tables.table("start_pos_victory_conditions")?.rows() {
        if row.str("start_pos_faction")? == *faction.id() {
            summary.victory_regions.push(row.str("region")?.into_owned());
        }
    }

    for row in tables.table("start_pos_diplomacy")?.rows() {
        let (Some(first), Some(second)) = (index.faction_by_id(&row.str("faction1")?), index.faction_by_id(&row.str("faction2")?)) else { continue };
        if first.key() != key && second.key() != key {
            continue;
        }

        let stance = row.str("stance")?;
        summary.has_war_or_vassal |= stance == "war" || stance == "vassal";
        match stance.as_ref() {
            "client_state" | "vassal" if second.key() == key => summary.subjects.push(first.key().to_owned()),
            "client_state" => summary.client_state_of = Some(second.key().to_owned()),
            "vassal" => summary.vassal_of = Some(second.key().to_owned()),
            _ => {},
        }
    }

    let order = index.factions_in_startpos_order();
    summary.subjects.sort_by_key(|subject| order.iter().position(|faction| faction.key() == subject));

    if faction_row(tables, key)?.bool("can_be_horde")? {
        summary.horde = Some(summary.region_count == 0);
    }

    Ok(summary)
}

/// Returns the `factions` row of a faction.
pub(crate) fn faction_row<'a>(tables: &'a StartposTables, faction: &str) -> Result<Row<'a>> {
    for row in tables.table("factions")?.rows() {
        if row.str("key")? == faction {
            return Ok(row);
        }
    }
    Err(RLibError::StartposInvalidValue("factions".to_owned(), "key".to_owned(), faction.to_owned()))
}

/// Builds a flag from a flag path column and three colour columns of a `factions` row.
pub(crate) fn flag(row: &Row, path_column: &str, colour_columns: [&str; 3]) -> Result<FactionFlag> {
    let mut colours = [[0; 3]; 3];
    for (colour, column) in colours.iter_mut().zip(colour_columns) {
        let hex = row.str(column)?;
        let invalid = || RLibError::StartposInvalidValue("factions".to_owned(), column.to_owned(), hex.to_string());
        if hex.len() != 6 {
            return Err(invalid());
        }
        for (channel, component) in colour.iter_mut().enumerate() {
            *component = u8::from_str_radix(&hex[channel * 2..channel * 2 + 2], 16).map_err(|_| invalid())?;
        }
    }
    Ok(FactionFlag { path: row.str(path_column)?.into_owned(), colours })
}
