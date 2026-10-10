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
    // then the colonels of the four garrison forces.
    let (ids, by_entry) = family_ids(&index, &plan, &forces);
    assert_eq!(ids, vec![1, 2, 4, 5, 6, 7, 8]);
    assert_eq!(by_entry.get("32"), Some(&3));
}

#[test]
fn test_colonels_use_general_art_sets() {
    assert_eq!(art_set_agent(COLONEL_AGENT), GENERAL_AGENT);
    assert_eq!(art_set_agent("spy"), "spy");
}
