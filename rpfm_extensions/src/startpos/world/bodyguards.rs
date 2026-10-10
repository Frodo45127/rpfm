//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Default bodyguard unit of each faction (`CHARACTER_DETAILS/GENERAL_BODYGUARD_DETAILS`).
//!
//! Generals without an army store their bodyguard unit and its size; generals leading an army store an empty
//! record (their bodyguard is the army's first unit). Verified on Attila's main campaign: 43 of 43 factions with
//! army-less generals match.

use getset::{CopyGetters, Getters};

use std::collections::{HashMap, HashSet};

use rpfm_lib::error::Result;

use super::FactionEntry;
use crate::startpos::tables::StartposTables;

#[cfg(test)] #[path = "bodyguards_test.rs"] mod bodyguards_test;

//---------------------------------------------------------------------------//
//                              Types
//---------------------------------------------------------------------------//

/// The default bodyguard of a faction's generals.
#[derive(Debug, Clone, PartialEq, Eq, Default, Getters, CopyGetters)]
pub struct Bodyguard {

    /// Unit key.
    #[getset(get = "pub")]
    unit: String,

    /// Men in the unit (`main_units.num_men`), stored twice in the startpos (current and maximum).
    #[getset(get_copy = "pub")]
    men: u32,
}

//---------------------------------------------------------------------------//
//                              Implementations
//---------------------------------------------------------------------------//

/// Returns the default bodyguard of a faction's generals, or the flagship of its admirals.
///
/// The candidates are the `commander_unit_permissions` units of the faction, of its subculture (rows without
/// faction) and of its culture (rows without faction and subculture), allowed for the faction's military group
/// and naval only for flagships. The bodyguard is the candidate no other candidate upgrades into
/// (`technology_unit_upgrades`), that is, the start of the upgrade chain; the first one in table order if there are
/// several.
///
/// # Arguments
///
/// * `tables` - Tables with `commander_unit_permissions`, `units_to_groupings_military_permissions`,
///   `technology_unit_upgrades` and `main_units`.
/// * `faction` - The faction.
/// * `naval` - Look for a flagship instead of a land bodyguard.
///
/// # Returns
///
/// The bodyguard, or `None` if the faction has no candidate.
///
/// # Errors
///
/// Returns an error if a table or column is missing.
pub fn bodyguard(tables: &StartposTables, faction: &FactionEntry, naval: bool) -> Result<Option<Bodyguard>> {
    let mut allowed = HashSet::new();
    for row in tables.table("units_to_groupings_military_permissions")?.rows() {
        if row.str("military_group")? == *faction.military_group() {
            allowed.insert(row.str("unit")?.into_owned());
        }
    }

    let mut naval_units = HashSet::new();
    let mut men = HashMap::new();
    for row in tables.table("main_units")?.rows() {
        let unit = row.str("unit")?.into_owned();
        if row.bool("is_naval")? {
            naval_units.insert(unit.to_owned());
        }
        men.insert(unit, row.int("num_men")?);
    }

    let mut candidates = vec![];
    for row in tables.table("commander_unit_permissions")?.rows() {
        let row_faction = row.str("faction_key")?;
        let row_subculture = row.str("subculture_key")?;
        let matches = row_faction == *faction.key()
            || (row_faction.is_empty() && row_subculture == *faction.subculture())
            || (row_faction.is_empty() && row_subculture.is_empty() && row.str("culture_key")? == *faction.culture());

        let unit = row.str("unit_key")?.into_owned();
        if matches && allowed.contains(&unit) && naval_units.contains(&unit) == naval && !candidates.contains(&unit) {
            candidates.push(unit);
        }
    }

    let mut upgraded = HashSet::new();
    for row in tables.table("technology_unit_upgrades")?.rows() {
        if candidates.contains(&row.str("unit")?.into_owned()) {
            upgraded.insert(row.str("target_unit")?.into_owned());
        }
    }

    Ok(candidates.into_iter()
        .find(|unit| !upgraded.contains(unit))
        .map(|unit| Bodyguard {
            men: men.get(&unit).and_then(|&men| u32::try_from(men).ok()).unwrap_or_default(),
            unit,
        }))
}
