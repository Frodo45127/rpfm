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
use crate::startpos::esf_nav::child_record;
use crate::startpos::startpos_test::{array_items, field_str, force_units, field_u32, test_data_tables, test_startpos, world, world_factions, world_regions, TEST_CAMPAIGN};
use crate::startpos::world::garrisons::garrisons_test::garrison_tables;
use crate::startpos::world::world_test::{add_table, fixture_index};

/// Adds the unit tables of the generals: the leader of `fact_a` has an army, the general of `fact_b` a navy.
pub(crate) fn force_tables() -> StartposTables {
    let mut tables = garrison_tables();
    add_table(&mut tables, "start_pos_land_units", &["id", "unit_type", "general", "soldiers"], &[
        &["1", "levy", "31", "100"],
    ]);
    add_table(&mut tables, "start_pos_naval_units", &["ID", "unit_type", "admiral"], &[
        &["1", "boat", "33"],
        &["2", "boat", "33"],
    ]);
    add_table(&mut tables, "commander_unit_permissions", &["culture_key", "faction_key", "subculture_key", "unit_key"], &[
        &["", "fact_a", "", "band"],
    ]);
    add_table(&mut tables, "technology_unit_upgrades", &["unit", "target_unit"], &[]);
    add_table(&mut tables, "start_pos_character_to_settlements", &["character", "settlement"], &[]);
    tables
}

#[test]
fn test_plan_numbers_generals_then_garrisons() {
    let tables = force_tables();
    let index = fixture_index(&tables);
    let plan = WorldPlan::new(&index);

    let force_plan = ForcePlan::new(&tables, &index, &plan).unwrap();
    let characters = force_plan.characters().iter()
        .map(|character| (character.id(), character.entry().map(|entry| entry.id().as_str()), character.force()))
        .collect::<Vec<_>>();
    let forces = force_plan.forces().iter()
        .map(|force| (force.id(), force.kind(), force.region().as_deref(), force.commander()))
        .collect::<Vec<_>>();

    assert_eq!(characters, vec![
        (1, Some("31"), Some(0)),
        (2, Some("30"), None),
        (3, Some("33"), Some(1)),
        (4, None, Some(2)),
        (5, None, Some(3)),
        (6, None, Some(4)),
        (7, None, Some(5)),
    ]);
    assert_eq!(forces, vec![
        (1, ForceKind::Army, None, 0),
        (2, ForceKind::Navy, None, 2),
        (3, ForceKind::GarrisonArmy, Some("reg_a1"), 3),
        (4, ForceKind::GarrisonNavy, Some("reg_a1"), 4),
        (5, ForceKind::GarrisonArmy, Some("reg_a2"), 5),
        (6, ForceKind::GarrisonArmy, Some("reg_b1"), 6),
    ]);
    assert_eq!(force_plan.garrison_of("reg_a1", ForceKind::GarrisonNavy).map(|force| force.id()), Some(4));
}

#[test]
fn test_plan_numbers_units_by_force() {
    let tables = force_tables();
    let index = fixture_index(&tables);
    let plan = WorldPlan::new(&index);

    let force_plan = ForcePlan::new(&tables, &index, &plan).unwrap();
    let units = |position: usize| {
        let force = &force_plan.forces()[position];
        force.units().iter().zip(force.unit_ids()).map(|(unit, id)| (unit.unit().as_str(), unit.commands(), id)).collect::<Vec<_>>()
    };

    // The general's bodyguard comes first but takes the last id; the navy has no flagship candidate; colonels
    // command their first unit.
    assert_eq!(units(0), vec![("band", true, 2), ("levy", false, 1)]);
    assert_eq!(units(1), vec![("boat", false, 3), ("boat", false, 4)]);
    assert_eq!(units(2), vec![("levy", true, 5), ("band", false, 6), ("levy", false, 7)]);
    assert_eq!(force_plan.forces()[0].commander_unit_id(), Some(2));
    assert_eq!(force_plan.forces()[1].commander_unit_id(), None);
}

#[test]
#[ignore = "needs the game data, see startpos_test.rs"]
fn test_plan_matches_the_game_startpos() {
    let tables = test_data_tables();
    let index = CampaignIndex::new(tables, TEST_CAMPAIGN).unwrap();
    let game_world = world(test_startpos());
    let factions = world_factions(game_world).iter().map(|faction| index.faction(field_str(faction, 1)).unwrap()).collect();
    let regions = world_regions(game_world).iter().map(|region| index.region(field_str(region, 1)).unwrap()).collect();
    let plan = WorldPlan::with_orders(factions, regions);

    let force_plan = ForcePlan::new(tables, &index, &plan).unwrap();

    // Garrison force ids only depend on the faction and region orders, and on the number of generals' forces.
    for region in world_regions(game_world) {
        let residence = child_record(region, "SETTLEMENT").and_then(|settlement| child_record(settlement, "GARRISON_RESIDENCE")).unwrap();
        let id = |kind| force_plan.garrison_of(field_str(region, 1), kind).map(|force| force.id()).unwrap_or_default();
        assert_eq!((id(ForceKind::GarrisonArmy), id(ForceKind::GarrisonNavy)), (field_u32(residence, 8), field_u32(residence, 9)), "{}", field_str(region, 1));
    }

    for faction in world_factions(game_world) {
        let count = |name: &str, item: &str| child_record(faction, name).map(|array| array_items(array, item).len()).unwrap_or_default();
        assert_eq!(force_plan.characters_of(field_str(faction, 1)).count(), count("CHARACTER_ARRAY", "CHARACTER"), "{}", field_str(faction, 1));
        assert_eq!(force_plan.forces_of(field_str(faction, 1)).count(), count("ARMY_ARRAY", "MILITARY_FORCE"), "{}", field_str(faction, 1));
    }
    assert_eq!((force_plan.characters().len(), force_plan.forces().len()), (578, 343));

    // Each faction's forces as sets of units: the game numbers generals' forces in a hash order of characters.
    let game_units = force_units(game_world);
    let sorted = |mut units: Vec<String>| { units.sort(); units };
    let mut mismatches = vec![];
    for faction in world_factions(game_world) {
        let key = field_str(faction, 1);
        let mut ours = force_plan.forces_of(key).map(|force| sorted(force.units().iter().map(|unit| unit.unit().to_owned()).collect())).collect::<Vec<_>>();
        let mut theirs = force_plan.forces_of(key).map(|force| sorted(game_units.get(&force.id()).cloned().unwrap_or_default())).collect::<Vec<_>>();
        ours.sort();
        theirs.sort();
        if ours != theirs {
            mismatches.push(key.to_owned());
        }
    }

    // The two known garrison exceptions (see garrisons_test.rs).
    assert_eq!(mismatches, vec!["att_fact_votadini".to_owned(), "att_fact_burgundii".to_owned()]);
}
