//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Building chains a faction can have (`building_chain_availabilities`). The game drops the starting buildings of a
//! settlement whose chain its owner can't have.

use std::collections::HashSet;

use rpfm_lib::error::Result;

use crate::startpos::tables::StartposTables;
use crate::startpos::world::FactionEntry;

#[cfg(test)] #[path = "availability_test.rs"] mod availability_test;

/// Returns the building chains available to a faction.
///
/// The chains are the ones of the availability sets of the faction's rows if it has any, else of its subculture's
/// (rows without faction), else of its culture's (rows without faction nor subculture), looking first at the
/// campaign's rows and then at the ones without campaign. The rows without culture, subculture nor faction add their
/// sets to every faction.
///
/// # Arguments
///
/// * `tables` - Tables with `building_chain_availabilities` and `building_chain_availability_sets`.
/// * `campaign` - Campaign key.
/// * `faction` - The faction.
///
/// # Returns
///
/// The chain keys, or `None` if the tables don't restrict chains.
///
/// # Errors
///
/// Returns an error if a column is missing.
pub fn available_chains(tables: &StartposTables, campaign: &str, faction: &FactionEntry) -> Result<Option<HashSet<String>>> {
    let (Ok(availabilities), Ok(sets)) = (tables.table("building_chain_availabilities"), tables.table("building_chain_availability_sets")) else { return Ok(None) };

    let mut campaign_rows = vec![];
    let mut generic_rows = vec![];
    let mut set_ids = HashSet::new();
    for row in availabilities.rows() {
        let row_campaign = row.str("campaign")?;
        if !row_campaign.is_empty() && row_campaign != campaign {
            continue;
        }

        let (row_faction, row_subculture, row_culture) = (row.str("faction")?, row.str("sub_culture")?, row.str("culture")?);
        let set_id = row.str("set_id")?.into_owned();
        if row_faction.is_empty() && row_subculture.is_empty() && row_culture.is_empty() {
            set_ids.insert(set_id);
            continue;
        }

        let level = if row_faction == *faction.key() {
            Some(0)
        } else if row_faction.is_empty() && !row_subculture.is_empty() && row_subculture == *faction.subculture() {
            Some(1)
        } else if row_faction.is_empty() && row_subculture.is_empty() && row_culture == *faction.culture() {
            Some(2)
        } else {
            None
        };
        if let Some(level) = level {
            let rows = if row_campaign.is_empty() { &mut generic_rows } else { &mut campaign_rows };
            rows.push((level, set_id));
        }
    }

    let chosen = [campaign_rows, generic_rows].into_iter()
        .flat_map(|rows| (0..3).map(move |level| rows.iter().filter(|(row_level, _)| *row_level == level).map(|(_, set)| set.to_owned()).collect::<Vec<_>>()))
        .find(|level_sets| !level_sets.is_empty())
        .unwrap_or_default();
    set_ids.extend(chosen);

    let mut chains = HashSet::new();
    for row in sets.rows() {
        if set_ids.contains(row.str("id")?.as_ref()) {
            chains.insert(row.str("building_chain")?.into_owned());
        }
    }
    Ok(Some(chains))
}
