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
use crate::startpos::world::world_test::{add_table, fixture_index, fixture_tables, FIXTURE_CAMPAIGN};

/// The fixture campaign plus `fact_c`, with `fact_b` a trading vassal of `fact_a` and `fact_a` at war with `fact_c`.
fn diplomacy_tables() -> StartposTables {
    let mut tables = fixture_tables();
    add_table(&mut tables, "start_pos_factions", &["ID", "faction", "campaign", "playable", "starting_order"], &[
        &["4", "fact_c", FIXTURE_CAMPAIGN, "false", "20"],
    ]);
    add_table(&mut tables, "start_pos_diplomacy", &["key", "faction1", "faction2", "stance", "grants_military_access", "grants_trade_agreement", "non_aggression_pact"], &[
        &["1", "2", "1", "vassal", "false", "true", "false"],
        &["2", "1", "4", "war", "false", "false", "false"],
    ]);
    add_table(&mut tables, "start_pos_past_events", &["event", "source", "target", "turns_ago"], &[]);
    tables
}

#[test]
fn test_rows_give_both_sides_their_stance() {
    let tables = diplomacy_tables();
    let index = fixture_index(&tables);
    let map = RegionMap::from_cells(vec![], 0, 0, vec![]);

    let diplomacy = Diplomacy::new(&tables, &index, &map).unwrap();
    let vassal = diplomacy.relationship("fact_b", "fact_a");
    let master = diplomacy.relationship("fact_a", "fact_b");

    assert_eq!((vassal.stance().as_str(), vassal.trade(), vassal.vassal_values()), ("vassal", true, (VASSAL_VALUE, 0)));
    assert_eq!((master.stance().as_str(), master.trade(), master.vassal_values()), ("master", true, (0, VASSAL_VALUE)));
    assert!(vassal.known() && master.known());
}

#[test]
fn test_vassals_join_their_masters_wars() {
    let tables = diplomacy_tables();
    let index = fixture_index(&tables);
    let map = RegionMap::from_cells(vec![], 0, 0, vec![]);

    let diplomacy = Diplomacy::new(&tables, &index, &map).unwrap();

    assert_eq!(diplomacy.relationship("fact_b", "fact_c").stance(), "war");
    assert_eq!(diplomacy.relationship("fact_c", "fact_b").stance(), "war");
    assert_eq!(diplomacy.relationship("fact_c", "fact_a").stance(), "war");
}

#[test]
fn test_neighbours_know_each_other() {
    let tables = diplomacy_tables();
    let index = fixture_index(&tables);

    // `reg_a1` (fact_a) next to `reg_b1` (fact_b), next to the rebels' `reg_r`.
    let keys = ["reg_a1", "reg_b1", "reg_r"].iter().map(|key| key.to_string()).collect();
    let mut cells = vec![0; 3 * 7];
    cells[7] = 1;
    cells[14] = 2;
    let map = RegionMap::from_cells(keys, 3, 1, cells);

    let diplomacy = Diplomacy::new(&tables, &index, &map).unwrap();

    assert!(diplomacy.relationship("fact_a", "fact_b").known());
    assert_eq!(diplomacy.relationship("fact_a", "fact_c"), Relationship { stance: "war".to_owned(), known: true, ..Default::default() });
}

#[test]
fn test_past_events_make_factions_known() {
    let mut tables = fixture_tables();
    add_table(&mut tables, "start_pos_diplomacy", &["key", "faction1", "faction2", "stance", "grants_military_access", "grants_trade_agreement", "non_aggression_pact"], &[]);
    add_table(&mut tables, "start_pos_past_events", &["event", "source", "target", "turns_ago"], &[
        &["RAIDING", "1", "2", "5"],
    ]);
    let index = fixture_index(&tables);
    let map = RegionMap::from_cells(vec![], 0, 0, vec![]);

    let diplomacy = Diplomacy::new(&tables, &index, &map).unwrap();

    assert_eq!(diplomacy.relationship("fact_a", "fact_b"), Relationship { known: true, ..Default::default() });
    assert!(diplomacy.relationship("fact_b", "fact_a").known());
}
