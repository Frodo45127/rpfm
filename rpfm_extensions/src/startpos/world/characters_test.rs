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
use crate::startpos::esf_nav::{as_record, child_record};
use crate::startpos::startpos_test::{array_items, field, field_str, field_u32, test_data_tables, test_startpos, world, world_factions, world_regions, TEST_CAMPAIGN};
use crate::startpos::world::world_test::{add_table, fixture_index, fixture_tables};

/// Start year of Attila's main campaign (`start_pos_calendars.start_year`).
const TEST_START_YEAR: i64 = 1147;

#[test]
fn test_birth_year_and_loyalty() {
    let tables = fixture_tables();
    let index = fixture_index(&tables);
    let characters = index.characters();

    assert_eq!(birth_year(1147, &characters[0]), 1107);
    assert_eq!(loyalty(&characters[0]), 3);
    assert_eq!(loyalty(&characters[1]), 10);
    assert_eq!(loyalty(&characters[2]), 5);
}

#[test]
fn test_placement_rules() {
    let mut tables = fixture_tables();
    add_table(&mut tables, "start_pos_character_to_settlements", &["character", "settlement"], &[
        &["30", "99"],
        &["30", "23"],
        &["30", "21"],
        &["33", "21"],
    ]);
    let index = fixture_index(&tables);
    let characters = index.characters();

    // Unknown settlement, then one of another faction, then an owned one.
    assert_eq!(placement(&tables, &index, &characters[0]).unwrap(), Placement::Settlement("reg_a2".to_owned()));
    assert_eq!(placement(&tables, &index, &characters[1]).unwrap(), Placement::Cell(HexCell::new(5, 7)));
    assert_eq!(placement(&tables, &index, &characters[2]).unwrap(), Placement::OffMap);

    // `fact_b` has no capital, and its only link is to another faction's settlement.
    assert_eq!(placement(&tables, &index, &characters[3]).unwrap(), Placement::FirstArmy);
}

/// Returns the position of a startpos character.
fn position(character: &RecordNode) -> (f32, f32) {
    match field(child_record(character, "LOCOMOTABLE").unwrap(), 0) {
        NodeType::Coord2d(position) => (*position.x(), *position.y()),
        other => panic!("{other:?}"),
    }
}

/// Returns the name ids of a startpos character.
fn name_ids(character: &RecordNode) -> BTreeSet<String> {
    let names = child_record(character, "CHARACTER_DETAILS")
        .and_then(|details| child_record(details, "CHARACTER_NAME"))
        .and_then(|name| child_record(name, "NAMES_BLOCK"))
        .unwrap();
    names.children().iter()
        .filter_map(|group| group.iter().filter_map(as_record).find(|record| record.name() == "CAMPAIGN_LOCALISATION"))
        .map(|localisation| field_str(localisation, 0).trim_start_matches("names_name_").to_owned())
        .filter(|id| !id.is_empty())
        .collect()
}

#[test]
#[ignore = "needs the game data, see startpos_test.rs"]
fn test_characters_real_data() {
    let tables = test_data_tables();
    let index = CampaignIndex::new(tables, TEST_CAMPAIGN).unwrap();
    let world = world(test_startpos());
    let factions = world_factions(world);

    let characters = factions.iter()
        .flat_map(|faction| child_record(faction, "CHARACTER_ARRAY").map(|array| array_items(array, "CHARACTER")).unwrap_or_default().into_iter().map(move |character| (field_str(faction, 1), character)))
        .collect::<Vec<_>>();
    let positions = characters.iter().map(|(_, character)| (field_u32(character, 1), position(character))).collect::<HashMap<_, _>>();
    let commanders = factions.iter()
        .flat_map(|faction| child_record(faction, "ARMY_ARRAY").map(|array| array_items(array, "MILITARY_FORCE")).unwrap_or_default())
        .map(|force| (field_u32(force, 0), field_u32(force, 1)))
        .collect::<HashMap<_, _>>();
    let settlement_positions = world_regions(world).iter().filter_map(|region| {
        let residence = child_record(region, "SETTLEMENT").and_then(|settlement| child_record(settlement, "GARRISON_RESIDENCE"))?;
        let commander = commanders.get(&field_u32(residence, 8))?;
        Some((field_str(region, 1).to_owned(), positions[commander]))
    }).collect::<HashMap<_, _>>();
    let first_army_positions = factions.iter().filter_map(|faction| {
        let general_army = child_record(faction, "ARMY_ARRAY").map(|array| array_items(array, "MILITARY_FORCE")).unwrap_or_default().into_iter()
            .map(|force| field_u32(force, 1))
            .find(|commander| characters.iter().any(|(_, character)| field_u32(character, 1) == *commander && field_str(character, 3) == "general"))?;
        Some((field_str(faction, 1).to_owned(), positions[&general_army]))
    }).collect::<HashMap<_, _>>();

    let names = tables.table("start_pos_characters").unwrap().rows()
        .map(|row| (row.str("ID").unwrap().into_owned(), ["Name", "Surname", "clan_name", "other_name"].iter().map(|column| row.str(column).unwrap().into_owned()).filter(|id| !id.is_empty()).collect::<BTreeSet<_>>()))
        .collect::<HashMap<_, _>>();

    let (mut linked, mut matches) = (0, 0);
    for (faction, character) in characters.iter().filter(|(_, character)| field_str(character, 3) == "general") {
        let ids = name_ids(character);
        let candidates = index.characters_of(faction).filter(|entry| entry.death_type().is_empty() && names[entry.id()] == ids).collect::<Vec<_>>();
        let [entry] = candidates.as_slice() else { continue };
        linked += 1;

        let details = child_record(character, "CHARACTER_DETAILS").unwrap();
        let birth = child_record(details, "DATE").map(|date| field_u32(date, 0)).unwrap();
        let stored_loyalty = match field(details, 32) { NodeType::I8(value) => i64::from(*value), other => panic!("{other:?}") };
        let expected_position = match placement(tables, &index, entry).unwrap() {
            Placement::Cell(cell) => Some(cell.world_position()),
            Placement::Settlement(region) => settlement_positions.get(&region).copied(),
            Placement::FirstArmy => first_army_positions.get(*faction).copied(),
            Placement::OffMap => Some((0.0, 0.0)),
        };
        let (x, y) = position(character);
        let position_ok = expected_position.is_some_and(|(ex, ey)| (ex - x).abs() < 1e-3 && (ey - y).abs() < 1e-3);

        matches += usize::from(i64::from(birth) == birth_year(TEST_START_YEAR, entry) && stored_loyalty == loyalty(entry) && position_ok);
    }

    // Two Sassanid generals are moved by the game to free spots around an occupied Ctesiphon, if they're linked.
    assert!(linked >= 330, "{linked}");
    assert!(linked - matches <= 2, "{matches} of {linked}");
}
