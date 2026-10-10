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

use std::collections::{BTreeSet, HashMap};

use super::*;
use crate::startpos::esf_nav::{as_record, as_u32, child_record};
use crate::startpos::startpos_test::{array_items, field, field_str, field_u32, test_data_esf, test_data_tables, test_startpos, world, world_factions, world_regions, TEST_CAMPAIGN};
use crate::startpos::world::world_test::{add_table, fixture_index, fixture_tables};

/// Adds the provinces to the fixture: `prov_a` has `fact_a`'s two regions, `prov_b` the rebel region and `fact_b`'s.
fn province_tables() -> StartposTables {
    let mut tables = fixture_tables();
    add_table(&mut tables, "region_to_province_junctions", &["province", "region"], &[
        &["prov_a", "reg_a2"],
        &["prov_a", "reg_a1"],
        &["prov_b", "reg_r"],
        &["prov_b", "reg_b1"],
        &["prov_c", "reg_c1"],
    ]);
    tables
}

#[test]
fn test_province_managers() {
    let tables = province_tables();
    let index = fixture_index(&tables);

    let managers = province_managers(&tables, &index).unwrap();
    let summary = managers.iter().map(|manager| (manager.province().as_str(), manager.faction().as_deref(), manager.regions().to_vec(), manager.main_region().as_str())).collect::<Vec<_>>();

    assert_eq!(summary, vec![
        ("prov_a", Some("fact_a"), vec!["reg_a2".to_owned(), "reg_a1".to_owned()], "reg_a1"),
        ("prov_b", None, vec!["reg_r".to_owned()], "reg_r"),
        ("prov_b", Some("fact_b"), vec!["reg_b1".to_owned()], "reg_b1"),
    ]);
}

/// Returns the k-th leaf (1-based, records skipped) after the first record with a name among a record's children.
fn leaf_after<'a>(record: &'a RecordNode, anchor: &str, k: usize) -> &'a NodeType {
    let group = &record.children()[0];
    let start = group.iter().position(|node| as_record(node).is_some_and(|child| child.name() == anchor)).unwrap();
    group[start + 1..].iter().filter(|node| as_record(node).is_none()).nth(k - 1).unwrap()
}

/// Returns the values of an `U32Array` leaf.
fn u32_array(node: &NodeType) -> Vec<u32> {
    match node {
        NodeType::U32Array(values) => values.value().to_vec(),
        other => panic!("not an U32 array: {other:?}"),
    }
}

#[test]
#[ignore = "needs the game data, see startpos_test.rs"]
fn test_provinces_and_seas_real_data() {
    let tables = test_data_tables();
    let index = CampaignIndex::new(tables, TEST_CAMPAIGN).unwrap();
    let world = world(test_startpos());

    let region_keys = world_regions(world).iter().map(|region| (field_u32(region, 0), field_str(region, 1).to_owned())).collect::<HashMap<_, _>>();
    let faction_keys = world_factions(world).iter().map(|faction| (field_u32(faction, 0), field_str(faction, 1).to_owned())).collect::<HashMap<_, _>>();
    let owner = |id: u32| faction_keys.get(&id).cloned();
    let regions = |node: &NodeType| u32_array(node).iter().map(|id| region_keys[id].to_owned()).collect::<BTreeSet<_>>();

    // Province managers, by province key and owner.
    let provinces = child_record(world, "PROVINCE_MANAGER").and_then(|manager| child_record(manager, "PROVINCE_ARRAY")).unwrap();
    let mut startpos_managers = BTreeSet::new();
    for group in provinces.children() {
        let key = match &group[0] { NodeType::Ascii(key) => key.to_owned(), other => panic!("{other:?}") };
        let province = group.iter().filter_map(as_record).next().unwrap();
        for manager in child_record(province, "FACTION_PROVINCE_MANAGER_ARRAY").map(|array| array_items(array, "FACTION_PROVINCE_MANAGER")).unwrap_or_default() {
            let main_region = region_keys[&as_u32(leaf_after(manager, "CAMPAIGN_EFFECT_BUNDLES", 4)).unwrap()].to_owned();
            startpos_managers.insert((key.to_owned(), owner(field_u32(manager, 1)), regions(field(manager, 2)), main_region));
        }
    }
    let generated = province_managers(tables, &index).unwrap().into_iter()
        .map(|manager| (manager.province().to_owned(), manager.faction().to_owned(), manager.regions().iter().cloned().collect(), manager.main_region().to_owned()))
        .collect::<BTreeSet<_>>();

    // Sea regions, in order, with their managers.
    let seas = child_record(world, "SEA_REGION_MANAGER").and_then(|manager| child_record(manager, "SEA_REGIONS_BLOCK")).map(|block| array_items(block, "SEA_REGION")).unwrap();
    let startpos_seas = seas.iter().map(|sea| {
        child_record(sea, "FACTION_SEA_REGION_MANAGER_BLOCK").map(|block| array_items(block, "FACTION_SEA_REGION_MANAGER")).unwrap_or_default().iter()
            .map(|manager| (owner(field_u32(manager, 1)), regions(field(manager, 2))))
            .collect::<BTreeSet<_>>()
    }).collect::<Vec<_>>();
    let map = RegionMap::from_map_data(&test_data_esf("map_data.esf")).unwrap();
    let layouts = SettlementLayout::all_from_map_data(&test_data_esf("map_data.esf"), |_| false).unwrap();
    let generated_seas = sea_regions(&map, &layouts, &index).into_iter()
        .map(|sea| sea.ports().iter().map(|(faction, regions)| (faction.to_owned(), regions.iter().cloned().collect())).collect::<BTreeSet<_>>())
        .collect::<Vec<_>>();

    // Without capital or major settlement, the game takes the first region in its hash-ordered region array, which
    // these functions replace with table order: one rebel province (Sarmatia Asiatica) differs.
    let without_main = |managers: &BTreeSet<(String, Option<String>, BTreeSet<String>, String)>| managers.iter().map(|(province, faction, regions, _)| (province.to_owned(), faction.to_owned(), regions.to_owned())).collect::<BTreeSet<_>>();
    assert_eq!(startpos_managers.len(), 94);
    assert_eq!(without_main(&startpos_managers), without_main(&generated));
    assert_eq!(startpos_managers.difference(&generated).count(), 1);
    assert_eq!(startpos_seas.len(), 25);
    assert_eq!(startpos_seas, generated_seas);
}

