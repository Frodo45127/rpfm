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
use crate::startpos::world::world_test::{add_table, fixture_tables};

fn estate_tables() -> StartposTables {
    let mut tables = fixture_tables();
    add_table(&mut tables, "effect_bonus_value_estate_junctions", &["bonus_value_id", "effect", "estate"], &[
        &["amount", "estates_farm", "estate_farm"],
        &["amount", "estates_church", "estate_church"],
        &["aversion", "aversion_farm", "estate_farm"],
    ]);
    add_table(&mut tables, "building_effects_junction", &["building", "effect", "value"], &[
        &["bld_farm_1", "estates_farm", "2"],
        &["bld_farm_1", "aversion_farm", "1"],
        &["bld_abbey_1", "estates_church", "1"],
        &["bld_abbey_1", "estates_farm", "1"],
    ]);
    add_table(&mut tables, "start_pos_character_estates", &["character", "region", "estate_type", "number"], &[
        &["31", "reg_a1", "estate_farm", "2"],
    ]);
    tables
}

#[test]
fn test_buildings_give_the_estates_of_their_amount_effects() {
    let rules = EstateRules::new(&estate_tables()).unwrap();

    assert_eq!(rules.estates_of("bld_farm_1"), vec!["estate_farm", "estate_farm"]);
    assert_eq!(rules.estates_of("bld_abbey_1"), vec!["estate_church", "estate_farm"]);
    assert!(rules.estates_of("bld_port_1").is_empty());
}

#[test]
fn test_the_table_gives_some_estates_their_owner() {
    let rules = EstateRules::new(&estate_tables()).unwrap();

    assert_eq!(rules.table_owner("reg_a1", "estate_farm", 2), Some("31"));
    assert_eq!(rules.table_owner("reg_a1", "estate_farm", 1), None);
}
