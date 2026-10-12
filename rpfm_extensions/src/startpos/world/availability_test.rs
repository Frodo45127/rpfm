//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

use std::collections::BTreeSet;

use super::*;
use crate::startpos::world::world_test::{add_table, fixture_index, fixture_tables, FIXTURE_CAMPAIGN};

/// Adds the chain availabilities to the fixture: `fact_a` has its own rows, `fact_b` only its culture's, and the
/// campaign has its own rows for `cult_b`'s subculture.
fn availability_tables() -> StartposTables {
    let mut tables = fixture_tables();
    add_table(&mut tables, "building_chain_availabilities", &["campaign", "culture", "sub_culture", "faction", "set_id"], &[
        &["", "", "", "", "global"],
        &["", "cult_a", "", "", "culture_a"],
        &["", "cult_a", "sub_a", "fact_a", "faction_a"],
        &["", "cult_b", "", "", "culture_b"],
        &["other", "cult_b", "", "", "other_campaign"],
    ]);
    add_table(&mut tables, "building_chain_availability_sets", &["id", "building_chain"], &[
        &["global", "chain_slum"],
        &["culture_a", "chain_farm"],
        &["faction_a", "chain_own_farm"],
        &["culture_b", "chain_farm"],
        &["culture_b", "chain_port"],
        &["other_campaign", "chain_other"],
    ]);
    tables
}

#[test]
fn test_faction_rows_replace_culture_rows() {
    let tables = availability_tables();
    let index = fixture_index(&tables);

    let chains = |faction| available_chains(&tables, FIXTURE_CAMPAIGN, index.faction(faction).unwrap()).unwrap().unwrap().into_iter().collect::<BTreeSet<_>>();

    assert_eq!(chains("fact_a"), ["chain_own_farm", "chain_slum"].map(str::to_owned).into());
    assert_eq!(chains("fact_b"), ["chain_farm", "chain_port", "chain_slum"].map(str::to_owned).into());
}

#[test]
fn test_no_restriction_without_the_tables() {
    let tables = fixture_tables();
    let index = fixture_index(&tables);

    assert_eq!(available_chains(&tables, FIXTURE_CAMPAIGN, index.faction("fact_a").unwrap()).unwrap(), None);
}

#[test]
fn test_settlements_drop_the_buildings_their_owner_cant_have() {
    let tables = availability_tables();

    let index = fixture_index(&tables);

    // fact_a can't have farms nor ports, only its own farm chain; cities aren't restricted.
    assert_eq!(index.settlement_of("reg_a1").unwrap().all_buildings().collect::<Vec<_>>(), vec!["bld_city_major_1"]);
}
