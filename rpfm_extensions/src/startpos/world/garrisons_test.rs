//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

use std::collections::HashMap;

use super::*;
use crate::startpos::esf_nav::child_record;
use crate::startpos::startpos_test::{field_str, field_u32, force_units, test_data_tables, test_startpos, world, world_regions, TEST_CAMPAIGN};
use crate::startpos::world::world_test::{add_table, fixture_index, fixture_tables};

/// Adds the unit tables to the fixture: the city gives a levy, a group the faction can't use (substituted by its
/// first allowed member), a bastion and a group that isn't a unit; the port gives a ship.
pub(crate) fn garrison_tables() -> StartposTables {
    let mut tables = fixture_tables();
    add_table(&mut tables, "building_level_armed_citizenry_junctions", &["building_level", "id", "unit_group"], &[
        &["bld_city_major_1", "1", "levy"],
        &["bld_city_major_1", "2", "elite_band"],
        &["bld_city_major_1", "3", "bastion"],
        &["bld_city_major_1", "4", "not_a_unit"],
        &["bld_port_1", "5", "boat"],
        &["bld_farm_1", "6", "levy"],
    ]);
    add_table(&mut tables, "armed_citizenry_units_to_unit_groups_junctions", &["id", "priority", "unit", "unit_group"], &[
        &["1", "0", "elite_band", "elite_band"],
        &["3", "0", "other_band", "elite_band"],
        &["2", "0", "band", "elite_band"],
    ]);
    add_table(&mut tables, "units_to_groupings_military_permissions", &["unit", "military_group"], &[
        &["levy", "mg_a"],
        &["band", "mg_a"],
        &["other_band", "mg_a"],
        &["bastion", "mg_a"],
        &["boat", "mg_a"],
    ]);
    add_table(&mut tables, "main_units", &["unit", "is_naval", "land_unit", "num_men"], &[
        &["levy", "false", "levy_land", "160"],
        &["elite_band", "false", "elite_band_land", "120"],
        &["band", "false", "band_land", "120"],
        &["other_band", "false", "other_band_land", "120"],
        &["bastion", "false", "bastion_land", "10"],
        &["boat", "true", "", "40"],
    ]);
    add_table(&mut tables, "land_units", &["key", "engine"], &[
        &["levy_land", ""],
        &["bastion_land", "onager_bastion"],
    ]);
    add_table(&mut tables, "battlefield_engines", &["key", "can_move"], &[
        &["onager_bastion", "false"],
        &["onager", "true"],
    ]);
    tables
}

#[test]
fn test_garrison_rule() {
    let tables = garrison_tables();
    let index = fixture_index(&tables);

    let garrisons = garrisons(&tables, &index).unwrap();
    let capital = garrisons.iter().find(|garrison| garrison.region() == "reg_a1").unwrap();

    assert_eq!(capital.faction(), "fact_a");
    assert_eq!(capital.army(), &vec!["levy".to_owned(), "band".to_owned(), "levy".to_owned()]);
    assert_eq!(capital.navy(), &vec!["boat".to_owned()]);
}

#[test]
fn test_rebel_settlements_have_no_garrison() {
    let tables = garrison_tables();
    let index = fixture_index(&tables);

    let garrisons = garrisons(&tables, &index).unwrap();

    assert_eq!(garrisons.len(), 3);
    assert!(garrisons.iter().all(|garrison| garrison.region() != "reg_r"));
}

#[test]
#[ignore = "needs the game data, see startpos_test.rs"]
fn test_garrisons_real_data() {
    let tables = test_data_tables();
    let index = CampaignIndex::new(tables, TEST_CAMPAIGN).unwrap();
    let world = world(test_startpos());
    let forces = force_units(world);

    let mut expected = HashMap::new();
    for region in world_regions(world) {
        let residence = child_record(region, "SETTLEMENT").and_then(|settlement| child_record(settlement, "GARRISON_RESIDENCE")).unwrap();
        let units = |position| forces.get(&field_u32(residence, position)).cloned().unwrap_or_default();
        expected.insert(field_str(region, 1).to_owned(), (units(8), units(9)));
    }

    let garrisons = garrisons(tables, &index).unwrap();
    let sorted = |units: &[String]| { let mut units = units.to_vec(); units.sort(); units };
    let mismatches = garrisons.iter()
        .filter(|garrison| !expected.get(garrison.region()).is_some_and(|(army, navy)| sorted(army) == sorted(garrison.army()) && sorted(navy) == sorted(garrison.navy())))
        .map(|garrison| (garrison.region().to_owned(), garrison.army().to_owned(), garrison.navy().to_owned(), expected.get(garrison.region()).cloned()))
        .collect::<Vec<_>>();

    assert_eq!(garrisons.len(), 175);
    assert_eq!(mismatches.len(), 2, "{mismatches:#?}");
}
