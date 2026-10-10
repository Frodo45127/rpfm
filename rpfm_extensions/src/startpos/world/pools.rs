//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Mercenary pools (`FACTION/FACTION_MERCENARY_POOL`, `PROVINCE/PROVINCE_MERCENARY_POOL`) and levy pools
//! (`FACTION/LEVY_MERCENARY_MANAGER`).
//!
//! Verified on Attila's main campaign: 118 of 118 faction pools, 62 of 62 province pools and 3 of 3 levy managers
//! match. The game stores the units of a pool in hash order; these functions return them in table order.

use getset::{CopyGetters, Getters};

use std::collections::{BTreeSet, HashMap};

use rpfm_lib::error::Result;

use super::CampaignIndex;
use crate::startpos::tables::StartposTables;

#[cfg(test)] #[path = "pools_test.rs"] mod pools_test;

/// Diplomatic stances that make a faction raise levies for its overlord.
const SUBJECT_STANCES: [&str; 2] = ["vassal", "client_state"];

//---------------------------------------------------------------------------//
//                              Types
//---------------------------------------------------------------------------//

/// A unit of a mercenary pool (`MERCENARY_POOL_UNIT_ARRAY` entry).
#[derive(Debug, Clone, PartialEq, Default, Getters, CopyGetters)]
pub struct MercenaryUnit {

    /// Unit key (`mercenary_unit_groups.unit_record`).
    #[getset(get = "pub")]
    unit: String,

    /// Units available at the start (`initial_unit_count`).
    #[getset(get_copy = "pub")]
    initial_count: u32,

    /// Chance to get a unit back each turn, as stored in the table.
    #[getset(get_copy = "pub")]
    replenish_chance: f32,

    /// Maximum units in the pool. The startpos stores it twice.
    #[getset(get_copy = "pub")]
    max_count: u32,

    /// Maximum units replenished per turn.
    #[getset(get_copy = "pub")]
    max_replenish_per_turn: u32,
}

/// A levy pool: an empty mercenary pool an overlord gets for each province where a subject owns regions.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Default, Getters)]
#[getset(get = "pub")]
pub struct LevyPool {

    /// Overlord faction key.
    overlord: String,

    /// Subject faction key.
    subject: String,

    /// Province key.
    province: String,
}

//---------------------------------------------------------------------------//
//                              Implementations
//---------------------------------------------------------------------------//

/// Returns the mercenary pool of a faction.
///
/// # Arguments
///
/// * `tables` - Tables with `faction_to_mercenary_set_junctions`, `mercenary_pool_to_groups_junctions` and
///   `mercenary_unit_groups`.
/// * `faction` - Faction key.
///
/// # Returns
///
/// The units of every set of the faction, in table order. Empty for factions without sets.
///
/// # Errors
///
/// Returns an error if a table or column is missing.
pub fn faction_mercenary_pool(tables: &StartposTables, faction: &str) -> Result<Vec<MercenaryUnit>> {
    mercenary_pool(tables, "faction_to_mercenary_set_junctions", "faction", faction)
}

/// Returns the mercenary pool of a province.
///
/// # Arguments
///
/// * `tables` - Tables with `province_to_mercenary_set_junctions`, `mercenary_pool_to_groups_junctions` and
///   `mercenary_unit_groups`.
/// * `province` - Province key.
///
/// # Returns
///
/// The units of every set of the province, in table order. Empty for provinces without sets.
///
/// # Errors
///
/// Returns an error if a table or column is missing.
pub fn province_mercenary_pool(tables: &StartposTables, province: &str) -> Result<Vec<MercenaryUnit>> {
    mercenary_pool(tables, "province_to_mercenary_set_junctions", "province", province)
}

/// Returns the units of the mercenary sets assigned to an owner by a junction table.
///
/// The requirement columns of `mercenary_pool_to_groups_junctions` are blank in vanilla Attila and aren't applied.
fn mercenary_pool(tables: &StartposTables, junctions: &str, owner_column: &str, owner: &str) -> Result<Vec<MercenaryUnit>> {
    let mut groups = HashMap::new();
    for row in tables.table("mercenary_unit_groups")?.rows() {
        groups.insert(row.str("key")?.into_owned(), row);
    }

    let pool_rows = tables.table("mercenary_pool_to_groups_junctions")?;
    let mut units = vec![];
    for set_row in tables.table(junctions)?.rows() {
        if set_row.str(owner_column)? != owner {
            continue;
        }
        let set = set_row.str("mercenary_set")?;
        for row in pool_rows.rows() {
            if row.str("pool")? != set {
                continue;
            }
            let Some(group) = groups.get(row.str("group")?.as_ref()) else { continue };
            units.push(MercenaryUnit {
                unit: group.str("unit_record")?.into_owned(),
                initial_count: count(row.int("initial_unit_count")?),
                replenish_chance: group.f32("chance_to_replenish")?,
                max_count: count(group.int("max_count")?),
                max_replenish_per_turn: count(group.int("max_replenish_per_turn")?),
            });
        }
    }

    Ok(units)
}

/// Returns the levy pools of every overlord.
///
/// Each `start_pos_diplomacy` row with stance `vassal` or `client_state` (`faction1` the subject, `faction2` the
/// overlord) gives the overlord one pool per province where the subject owns regions.
///
/// # Arguments
///
/// * `tables` - Tables with `start_pos_diplomacy` and `region_to_province_junctions`.
/// * `index` - The campaign's startpos rows.
///
/// # Returns
///
/// The pools, sorted by overlord, subject and province key. The game sorts them by the startpos ids of the subject
/// and the province, which aren't known at this stage.
///
/// # Errors
///
/// Returns an error if a table or column is missing.
pub fn levy_pools(tables: &StartposTables, index: &CampaignIndex) -> Result<Vec<LevyPool>> {
    let mut province_of = HashMap::new();
    for row in tables.table("region_to_province_junctions")?.rows() {
        province_of.insert(row.str("region")?.into_owned(), row.str("province")?.into_owned());
    }

    let mut pools = BTreeSet::new();
    for row in tables.table("start_pos_diplomacy")?.rows() {
        if !SUBJECT_STANCES.contains(&row.str("stance")?.as_ref()) {
            continue;
        }
        let (Some(subject), Some(overlord)) = (index.faction_by_id(&row.str("faction1")?), index.faction_by_id(&row.str("faction2")?)) else { continue };
        for region in index.regions_of(subject.key()) {
            if let Some(province) = province_of.get(region.key()) {
                pools.insert(LevyPool {
                    overlord: overlord.key().to_owned(),
                    subject: subject.key().to_owned(),
                    province: province.to_owned(),
                });
            }
        }
    }

    Ok(pools.into_iter().collect())
}

/// Converts a table count to the unsigned value the startpos stores, clamping negative values to 0.
fn count(value: i64) -> u32 {
    u32::try_from(value.max(0)).unwrap_or(u32::MAX)
}
