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
use crate::startpos::world::world_test::fixture_index;
use crate::startpos::world::writer::forces::forces_test::force_tables;

#[test]
fn test_family_ids_leave_room_for_the_dead() {
    let tables = force_tables();
    let index = fixture_index(&tables);
    let plan = WorldPlan::new(&index);
    let forces = ForcePlan::new(&tables, &index, &plan).unwrap();

    // `fact_a`: its leader and another living character, then its dead character takes the next id; then `fact_b`;
    // then the colonels of the three garrison forces (`reg_b1` has no land unit to garrison).
    let (ids, by_entry) = family_ids(&index, &plan, &forces);
    assert_eq!(ids, vec![1, 2, 4, 5, 6, 7]);
    assert_eq!(by_entry.get("32"), Some(&3));
}

#[test]
fn test_colonels_use_general_art_sets() {
    assert_eq!(art_set_agent(COLONEL_AGENT), GENERAL_AGENT);
    assert_eq!(art_set_agent("spy"), "spy");
}

#[test]
fn test_art_sets_of_portraits_then_the_faction_without_custom_ones() {
    let tables = force_tables();
    let index = fixture_index(&tables);
    let faction = index.faction("fact_a").unwrap();
    let art_set = |id: &str, faction: &str, is_custom: bool| ArtSet {
        id: id.to_owned(),
        agent: GENERAL_AGENT.to_owned(),
        is_male: true,
        faction: faction.to_owned(),
        subculture: String::new(),
        culture: String::new(),
        is_custom,
    };
    let art_sets = [art_set("hero", "fact_a", true), art_set("generic_1", "fact_a", false), art_set("generic_2", "fact_a", false)];

    assert_eq!(pick_art_set(&art_sets, faction, GENERAL_AGENT, true, "hero", 0), "hero");
    assert_eq!(pick_art_set(&art_sets, faction, GENERAL_AGENT, true, "", 0), "generic_1");
    assert_eq!(pick_art_set(&art_sets, faction, GENERAL_AGENT, true, "", 1), "generic_2");
    assert_eq!(pick_art_set(&art_sets, faction, GENERAL_AGENT, true, "other_campaign_set", 1), "generic_2");
}
