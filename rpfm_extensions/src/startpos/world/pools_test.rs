//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

use rpfm_lib::files::esf::{NodeType, RecordNode};

use std::collections::HashMap;

use super::*;
use crate::startpos::esf_nav::{as_record, as_str, as_u32, child_record, leaves};
use crate::startpos::startpos_test::{array_items, field_str, field_u32, test_data_tables, test_startpos, world, world_factions, TEST_CAMPAIGN};
use crate::startpos::world::world_test::{add_table, fixture_index, fixture_tables};

/// A pool unit as compared in the tests: unit, initial count, chance bits, maximum and replenishment per turn.
type UnitSummary = (String, u32, u32, u32, u32);

/// Adds the mercenary and diplomacy tables to the fixture: `fact_a` has one set with two groups, the province of
/// `reg_b1` has another; `fact_b` is a vassal of `fact_a`.
fn pool_tables() -> StartposTables {
    let mut tables = fixture_tables();
    add_table(&mut tables, "faction_to_mercenary_set_junctions", &["faction", "mercenary_set"], &[&["fact_a", "set_a"]]);
    add_table(&mut tables, "province_to_mercenary_set_junctions", &["mercenary_set", "province"], &[&["set_p", "prov_b"]]);
    add_table(&mut tables, "mercenary_pool_to_groups_junctions", &["group", "initial_unit_count", "key", "pool"], &[
        &["grp_archers", "1", "1", "set_a"],
        &["grp_ships", "0", "2", "set_p"],
        &["grp_horse", "2", "3", "set_a"],
    ]);
    add_table(&mut tables, "mercenary_unit_groups", &["chance_to_replenish", "key", "max_count", "max_replenish_per_turn", "unit_record"], &[
        &["0.1", "grp_archers", "3", "1", "archers"],
        &["0.05", "grp_horse", "2", "1", "horse"],
        &["0.2", "grp_ships", "1", "1", "ship"],
    ]);
    add_table(&mut tables, "region_to_province_junctions", &["province", "region"], &[
        &["prov_a", "reg_a1"],
        &["prov_a", "reg_a2"],
        &["prov_b", "reg_b1"],
        &["prov_b", "reg_r"],
    ]);
    add_table(&mut tables, "start_pos_diplomacy", &["faction1", "faction2", "stance"], &[
        &["2", "1", "vassal"],
        &["1", "3", "client_state"],
        &["1", "2", "war"],
    ]);
    tables
}

#[test]
fn test_faction_and_province_pools() {
    let tables = pool_tables();

    let faction_pool = faction_mercenary_pool(&tables, "fact_a").unwrap();
    let province_pool = province_mercenary_pool(&tables, "prov_b").unwrap();

    assert_eq!(faction_pool.iter().map(|unit| unit.unit().as_str()).collect::<Vec<_>>(), vec!["archers", "horse"]);
    assert_eq!((faction_pool[1].initial_count(), faction_pool[1].replenish_chance(), faction_pool[1].max_count()), (2, 0.05, 2));
    assert_eq!(province_pool.iter().map(|unit| unit.unit().as_str()).collect::<Vec<_>>(), vec!["ship"]);
    assert!(faction_mercenary_pool(&tables, "fact_b").unwrap().is_empty());
}

#[test]
fn test_levies_of_subject_provinces() {
    let tables = pool_tables();
    let index = fixture_index(&tables);

    let pools = levy_pools(&tables, &index).unwrap();

    // The client state row points to a faction of another campaign, so it's ignored.
    assert_eq!(pools, vec![LevyPool { overlord: "fact_a".to_owned(), subject: "fact_b".to_owned(), province: "prov_b".to_owned() }]);
}

/// Reads the units of a `MERCENARY_POOL` record.
fn startpos_pool(pool: &RecordNode) -> Vec<UnitSummary> {
    let Some(array) = child_record(pool, "MERCENARY_POOL_UNIT_ARRAY") else { return vec![] };
    let mut units = array.children().iter().map(|group| {
        let values = leaves(group).collect::<Vec<_>>();
        let chance = match values[2] { NodeType::F32(value) => value.value().to_bits(), _ => 0 };
        (as_str(values[0]).unwrap().to_owned(), as_u32(values[1]).unwrap(), chance, as_u32(values[3]).unwrap(), as_u32(values[4]).unwrap())
    }).collect::<Vec<_>>();
    units.sort();
    units
}

/// Converts generated units to sorted summaries.
fn summaries(units: &[MercenaryUnit]) -> Vec<UnitSummary> {
    let mut units = units.iter().map(|unit| (unit.unit().to_owned(), unit.initial_count(), unit.replenish_chance().to_bits(), unit.max_count(), unit.max_replenish_per_turn())).collect::<Vec<_>>();
    units.sort();
    units
}

#[test]
#[ignore = "needs the game data, see startpos_test.rs"]
fn test_pools_real_data() {
    let tables = test_data_tables();
    let index = CampaignIndex::new(tables, TEST_CAMPAIGN).unwrap();
    let world = world(test_startpos());

    let factions = world_factions(world);
    let faction_matches = factions.iter()
        .filter(|faction| {
            let pool = child_record(faction, "FACTION_MERCENARY_POOL").and_then(|pool| child_record(pool, "MERCENARY_POOL")).unwrap();
            startpos_pool(pool) == summaries(&faction_mercenary_pool(tables, field_str(faction, 1)).unwrap())
        })
        .count();

    let provinces = child_record(world, "PROVINCE_MANAGER").and_then(|manager| child_record(manager, "PROVINCE_ARRAY")).unwrap();
    let mut province_keys = HashMap::new();
    let mut province_matches = 0;
    for group in provinces.children() {
        let key = leaves(group).find_map(as_str).unwrap();
        let province = group.iter().filter_map(as_record).next().unwrap();
        province_keys.insert(field_u32(province, 0), key.to_owned());
        let pool = child_record(province, "PROVINCE_MERCENARY_POOL").and_then(|pool| child_record(pool, "MERCENARY_POOL")).unwrap();
        if startpos_pool(pool) == summaries(&province_mercenary_pool(tables, key).unwrap()) {
            province_matches += 1;
        }
    }

    let faction_keys = factions.iter().map(|faction| (field_u32(faction, 0), field_str(faction, 1).to_owned())).collect::<HashMap<_, _>>();
    let mut levies = vec![];
    for faction in &factions {
        let Some(blocks) = child_record(faction, "LEVY_MERCENARY_MANAGER").and_then(|manager| child_record(manager, "LEVY_POOLS_BLOCK")) else { continue };
        for pool in array_items(blocks, "LEVY_MERCENARY_POOL") {
            levies.push(LevyPool {
                overlord: field_str(faction, 1).to_owned(),
                subject: faction_keys[&field_u32(pool, 2)].to_owned(),
                province: province_keys[&field_u32(pool, 3)].to_owned(),
            });
        }
    }
    levies.sort();

    assert_eq!(faction_matches, 118);
    assert_eq!(province_matches, 62);
    assert_eq!(levies, levy_pools(tables, &index).unwrap());
}
