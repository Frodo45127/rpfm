//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

use super::*;

#[test]
fn test_religion_shares_are_normalised() {
    assert_eq!(religion_shares(&[30.0, 30.0, 0.0]), vec![0.5, 0.5, 0.0]);
}

#[test]
fn test_regions_without_percentages_share_every_religion() {
    assert_eq!(religion_shares(&[0.0; 28]), vec![1.0 / 28.0; 28]);
    assert_eq!(religion_shares(&[0.0; 28])[0], 0.035714287);
}

#[test]
fn test_buildings_give_their_base_gdp() {
    let mut tables = crate::startpos::world::world_test::fixture_tables();
    crate::startpos::world::world_test::add_table(&mut tables, "effect_bonus_value_basic_junction", &["bonus_value_id", "effect"], &[
        &["gdp_farming", "gdp_farms"],
        &["gdp_mod_farming", "gdp_farms_mod"],
        &["gdp_manufacture", "gdp_industry"],
    ]);
    crate::startpos::world::world_test::add_table(&mut tables, "building_effects_junction", &["building", "effect", "value"], &[
        &["bld_farm_1", "gdp_farms", "80"],
        &["bld_farm_1", "gdp_farms_mod", "10"],
        &["bld_iron_1", "gdp_industry", "150"],
    ]);

    let gdp = building_gdp(&tables).unwrap();

    assert_eq!(gdp["bld_farm_1"], 80);
    assert_eq!(gdp["bld_iron_1"], 150);
    assert!(!gdp.contains_key("bld_port_1"));
}
