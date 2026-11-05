//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Garrisons of the settlements (`REGION/SETTLEMENT/GARRISON_RESIDENCE` #8 army, #9 navy).
//!
//! Each owned settlement gets a garrison army if any land unit comes out and a garrison navy if any ship does, each led by a
//! generated `colonel`. Their units come from the armed citizenry of the settlement's buildings. Verified on
//! Attila's main campaign: 173 of 175 settlements match. The other two (Ascaucalis and Eildon) differ by one unit
//! for reasons not found in the tables.

use getset::Getters;

use std::collections::{HashMap, HashSet};

use rpfm_lib::error::Result;

use super::CampaignIndex;
use crate::startpos::tables::StartposTables;

#[cfg(test)] #[path = "garrisons_test.rs"] pub(crate) mod garrisons_test;

//---------------------------------------------------------------------------//
//                              Types
//---------------------------------------------------------------------------//

/// The garrison of a settlement.
#[derive(Debug, Clone, PartialEq, Eq, Default, Getters)]
#[getset(get = "pub")]
pub struct Garrison {

    /// Region key.
    region: String,

    /// Owner faction key.
    faction: String,

    /// Land units, in building order.
    army: Vec<String>,

    /// Ships, in building order.
    navy: Vec<String>,
}

/// Unit data needed to build garrisons.
struct UnitRules {

    /// Building level and armed citizenry group of each `building_level_armed_citizenry_junctions` row, in the
    /// table's key order (see [`crate::startpos::tables::Table::rows_in_key_order`]).
    building_groups: Vec<(String, String)>,

    /// Members of each armed citizenry group, sorted by priority and then row id.
    group_members: HashMap<String, Vec<String>>,

    /// Units allowed for each military group.
    permissions: HashMap<String, HashSet<String>>,

    /// Units that are ships.
    naval: HashSet<String>,

    /// Every unit key in `main_units`.
    units: HashSet<String>,

    /// Land units that can't move (fixed emplacements like the onager bastions), which never join a garrison.
    static_units: HashSet<String>,
}

//---------------------------------------------------------------------------//
//                              Implementations
//---------------------------------------------------------------------------//

/// Builds the garrisons of every settlement owned by a faction (rebel-owned settlements get none).
///
/// Every `building_level_armed_citizenry_junctions` row, in the table's key order, gives one unit per building of its
/// level in the settlement: the group's own unit if the owner's military group allows it, otherwise the first allowed
/// member of the group. Groups that aren't units, groups without allowed members and units that can't move are
/// skipped. The game creates a garrison's units grouped by key in an order not reproduced yet.
///
/// # Arguments
///
/// * `tables` - Tables with `building_level_armed_citizenry_junctions`, `armed_citizenry_units_to_unit_groups_junctions`,
///   `units_to_groupings_military_permissions`, `main_units`, `land_units` and `battlefield_engines`.
/// * `index` - The campaign's startpos rows.
///
/// # Returns
///
/// One garrison per owned settlement, in settlement order.
///
/// # Errors
///
/// Returns an error if a table or column is missing.
pub fn garrisons(tables: &StartposTables, index: &CampaignIndex) -> Result<Vec<Garrison>> {
    let rules = UnitRules::new(tables)?;
    let mut garrisons = vec![];

    for settlement in index.settlements() {
        let Some(faction) = index.region(settlement.region()).and_then(|region| region.owner().as_deref()).and_then(|key| index.faction(key)) else { continue };
        let allowed = rules.permissions.get(faction.military_group());
        let mut garrison = Garrison {
            region: settlement.region().to_owned(),
            faction: faction.key().to_owned(),
            ..Default::default()
        };

        let buildings = settlement.all_buildings().collect::<Vec<_>>();
        for (level, group) in &rules.building_groups {
            for _ in buildings.iter().filter(|building| **building == level) {
                if let Some(unit) = rules.unit_for_group(group, allowed) {
                    if rules.naval.contains(unit) {
                        garrison.navy.push(unit.to_owned());
                    } else {
                        garrison.army.push(unit.to_owned());
                    }
                }
            }
        }

        garrisons.push(garrison);
    }

    Ok(garrisons)
}

impl UnitRules {

    /// Collects the unit data from the tables.
    fn new(tables: &StartposTables) -> Result<Self> {
        let mut building_groups = vec![];
        for row in tables.table("building_level_armed_citizenry_junctions")?.rows_in_key_order("id")? {
            building_groups.push((row.str("building_level")?.into_owned(), row.str("unit_group")?.into_owned()));
        }

        let mut members = HashMap::<String, Vec<(i64, i64, String)>>::new();
        for row in tables.table("armed_citizenry_units_to_unit_groups_junctions")?.rows() {
            members.entry(row.str("unit_group")?.into_owned()).or_default().push((row.int("priority")?, row.int("id")?, row.str("unit")?.into_owned()));
        }
        let group_members = members.into_iter().map(|(group, mut list)| {
            list.sort();
            (group, list.into_iter().map(|(_, _, unit)| unit).collect())
        }).collect();

        let mut permissions = HashMap::<String, HashSet<String>>::new();
        for row in tables.table("units_to_groupings_military_permissions")?.rows() {
            permissions.entry(row.str("military_group")?.into_owned()).or_default().insert(row.str("unit")?.into_owned());
        }

        let mut fixed_engines = HashSet::new();
        for row in tables.table("battlefield_engines")?.rows() {
            if !row.bool("can_move")? {
                fixed_engines.insert(row.str("key")?.into_owned());
            }
        }

        let mut static_land_units = HashSet::new();
        for row in tables.table("land_units")?.rows() {
            if fixed_engines.contains(row.str("engine")?.as_ref()) {
                static_land_units.insert(row.str("key")?.into_owned());
            }
        }

        let mut naval = HashSet::new();
        let mut units = HashSet::new();
        let mut static_units = HashSet::new();
        for row in tables.table("main_units")?.rows() {
            let unit = row.str("unit")?.into_owned();
            if row.bool("is_naval")? {
                naval.insert(unit.to_owned());
            } else if static_land_units.contains(row.str("land_unit")?.as_ref()) {
                static_units.insert(unit.to_owned());
            }
            units.insert(unit);
        }

        Ok(Self { building_groups, group_members, permissions, naval, units, static_units })
    }

    /// Returns the unit an armed citizenry group gives to a faction with the given allowed units, if any.
    fn unit_for_group<'a>(&'a self, group: &'a str, allowed: Option<&HashSet<String>>) -> Option<&'a str> {
        if !self.units.contains(group) {
            return None;
        }

        let is_allowed = |unit: &str| allowed.is_some_and(|allowed| allowed.contains(unit));
        let unit = if is_allowed(group) {
            group
        } else {
            self.group_members.get(group)?.iter().map(String::as_str).find(|unit| is_allowed(unit))?
        };

        (!self.static_units.contains(unit)).then_some(unit)
    }
}
