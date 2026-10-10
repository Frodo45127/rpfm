//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Rebel setup of each region (`REGION/REGION_REBEL_MANAGER/REBEL_SETUP`).
//!
//! Regions owned by the rebels get a setup for their alternative rebel faction. Every other region stores the same default
//! setup (in Attila: Persis's flag with the Arian religion and an empire government, apparently a leftover).
//! Verified on Attila's main campaign: 186 of 186 regions match.

use getset::Getters;

use rpfm_lib::error::{RLibError, Result};

use super::RegionEntry;
use super::factions::{FactionFlag, faction_row, flag};
use crate::startpos::tables::StartposTables;

#[cfg(test)] #[path = "rebels_test.rs"] mod rebels_test;

//---------------------------------------------------------------------------//
//                              Types
//---------------------------------------------------------------------------//

/// The rebel setup of a region owned by the rebels.
#[derive(Debug, Clone, PartialEq, Eq, Default, Getters)]
#[getset(get = "pub")]
pub struct RebelSetup {

    /// Rebel faction key (`start_pos_regions.alternative_rebel_faction`).
    faction: String,

    /// The region's majority religion.
    religion: String,

    /// The rebel faction's flag, with its primary, secondary and uniform colours.
    flag: FactionFlag,
}

//---------------------------------------------------------------------------//
//                              Implementations
//---------------------------------------------------------------------------//

/// Returns the rebel setup of a region.
///
/// # Arguments
///
/// * `tables` - Tables with `start_pos_regions`, `start_pos_region_religions` and `factions`.
/// * `region` - The region.
///
/// # Returns
///
/// The setup for regions owned by the rebels, `None` for the others (which store the default setup).
///
/// # Errors
///
/// Returns an error if a table or column is missing, or the rebel faction isn't in `factions`.
pub fn rebel_setup(tables: &StartposTables, region: &RegionEntry) -> Result<Option<RebelSetup>> {
    if region.owner().is_some() {
        return Ok(None);
    }

    let mut faction = None;
    for row in tables.table("start_pos_regions")?.rows() {
        if row.str("id")? == *region.id() {
            faction = Some(row.str("alternative_rebel_faction")?.into_owned());
        }
    }
    let faction = faction.ok_or_else(|| RLibError::StartposInvalidValue("start_pos_regions".to_owned(), "id".to_owned(), region.id().to_owned()))?;

    let flag = flag(&faction_row(tables, &faction)?, "flags_path", ["primary_colour_hex", "secondary_colour_hex", "uniform_colour_hex"])?;
    Ok(Some(RebelSetup {
        religion: majority_religion(tables, region)?.unwrap_or_default(),
        faction,
        flag,
    }))
}

/// Returns the religion with the largest share of a region, the first in table order on ties.
///
/// # Arguments
///
/// * `tables` - Tables with `start_pos_region_religions`.
/// * `region` - The region.
///
/// # Returns
///
/// The religion key, or `None` if the region has no religion rows.
///
/// # Errors
///
/// Returns an error if a table or column is missing.
pub fn majority_religion(tables: &StartposTables, region: &RegionEntry) -> Result<Option<String>> {
    let mut best: Option<(f64, String)> = None;
    for row in tables.table("start_pos_region_religions")?.rows() {
        if row.str("region")? != *region.id() {
            continue;
        }
        let share = row.number("percentage")?;
        if best.as_ref().is_none_or(|(best_share, _)| share > *best_share) {
            best = Some((share, row.str("religion")?.into_owned()));
        }
    }
    Ok(best.map(|(_, religion)| religion))
}
