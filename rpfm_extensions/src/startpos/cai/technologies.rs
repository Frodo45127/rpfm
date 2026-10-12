//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! The technologies of each faction's tree, in the order the game builds it.

use std::collections::BTreeMap;

use rpfm_lib::error::Result;

use crate::startpos::tables::StartposTables;
use crate::startpos::world::FactionEntry;

#[cfg(test)] #[path = "technologies_test.rs"] mod technologies_test;

//---------------------------------------------------------------------------//
//                              Implementations
//---------------------------------------------------------------------------//

/// Returns the technologies of a faction's tree.
///
/// The tree uses the node sets of the faction if it has any, else the ones of its subculture (for no faction), else
/// the ones of its culture (for no faction nor subculture), looking first at the campaign's sets and then at the ones
/// without campaign. The sets go by technology category from the last (military before civil), then in reverse
/// table order. Each set lists its
/// nodes for the campaign and the faction by (tier, indent), the faction's own node replacing the generic one of its
/// slot, and technologies appear once.
///
/// # Arguments
///
/// * `tables` - Tables with `technology_node_sets` and `technology_nodes`.
/// * `campaign` - Campaign key.
/// * `faction` - The faction.
///
/// # Returns
///
/// The technology keys, in tree order.
///
/// # Errors
///
/// Returns an error if a table or column is missing.
pub fn faction_technologies(tables: &StartposTables, campaign: &str, faction: &FactionEntry) -> Result<Vec<String>> {
    let for_campaign = |key: &str| key.is_empty() || key == campaign;

    let mut campaign_sets = vec![];
    let mut generic_sets = vec![];
    for row in tables.table("technology_node_sets")?.rows() {
        let set_campaign = row.str("campaign_key")?;
        if for_campaign(&set_campaign) {
            let set = (row.str("key")?.into_owned(), row.str("faction_key")?.into_owned(), row.str("subculture")?.into_owned(), row.str("culture")?.into_owned(), row.str("technology_category")?.into_owned());
            if set_campaign.is_empty() { generic_sets.push(set) } else { campaign_sets.push(set) }
        }
    }
    let mut chosen = [&campaign_sets, &generic_sets].into_iter()
        .flat_map(|sets| [
            sets.iter().filter(|set| set.1 == *faction.key()).collect::<Vec<_>>(),
            sets.iter().filter(|set| set.1.is_empty() && set.2 == *faction.subculture()).collect(),
            sets.iter().filter(|set| set.1.is_empty() && set.2.is_empty() && set.3 == *faction.culture()).collect(),
        ])
        .find(|sets| !sets.is_empty())
        .unwrap_or_default();
    chosen.reverse();
    chosen.sort_by(|a, b| b.4.cmp(&a.4));

    let nodes = tables.table("technology_nodes")?;
    let mut technologies: Vec<String> = vec![];
    for (set, ..) in chosen {
        let mut slots = BTreeMap::new();
        for row in nodes.rows() {
            let node_faction = row.str("faction_key")?;
            if row.str("technology_node_set")? != *set || !for_campaign(&row.str("campaign_key")?) || !(node_faction.is_empty() || node_faction == *faction.key()) {
                continue;
            }

            // The faction's own node wins its slot over the generic one.
            let slot = (row.int("tier")?, row.int("indent")?);
            let own = !node_faction.is_empty();
            let technology = row.str("technology_key")?.into_owned();
            match slots.get(&slot) {
                Some((true, _)) if !own => {},
                _ => { slots.insert(slot, (own, technology)); },
            }
        }
        for (_, technology) in slots.into_values() {
            if !technologies.contains(&technology) {
                technologies.push(technology);
            }
        }
    }
    Ok(technologies)
}
