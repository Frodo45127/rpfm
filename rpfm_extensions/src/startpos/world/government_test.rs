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

use std::collections::BTreeSet;

use super::*;
use crate::startpos::esf_nav::{as_record, child_record};
use crate::startpos::startpos_test::{array_items, field, field_str, field_u32, test_data_tables, test_startpos, world, world_factions, TEST_CAMPAIGN};
use crate::startpos::world::world_test::{add_table, fixture_index, fixture_tables};

/// Adds the government tables to the fixture.
fn government_tables() -> StartposTables {
    let mut tables = fixture_tables();
    add_table(&mut tables, "ministerial_positions", &["minister_key", "maximum_concurrent_ministers"], &[
        &["faction_leader", "1"],
        &["general", "3"],
    ]);
    add_table(&mut tables, "ministerial_positions_culture_details", &["culture_key", "faction_key", "subculture_key", "ministerial_position_key", "unique_id", "campaign"], &[
        &["cult_a", "", "", "faction_leader", "100", ""],
        &["", "", "sub_a", "general", "101", ""],
        &["", "fact_a", "", "general", "102", "camp"],
        &["", "fact_a", "", "general", "103", "other"],
        &["cult_a", "", "sub_b", "general", "104", ""],
    ]);
    add_table(&mut tables, "political_parties", &["key", "initial_power"], &[
        &["party_ruler", "60"],
        &["party_council", "40"],
        &["party_zero_1", "0"],
        &["party_zero_2", "0"],
    ]);
    add_table(&mut tables, "faction_political_parties_junctions", &["faction_key", "political_party_key"], &[
        &["fact_a", "party_ruler"],
        &["fact_a", "party_council"],
        &["fact_b", "party_zero_1"],
        &["fact_b", "party_zero_2"],
    ]);
    tables
}

#[test]
fn test_posts_of_culture_subculture_and_faction() {
    let tables = government_tables();
    let index = fixture_index(&tables);

    let posts = government_posts(&tables, &index, index.faction("fact_a").unwrap()).unwrap();

    assert_eq!(posts.iter().map(|group| (group.position().as_str(), group.unique_id(), group.count())).collect::<Vec<_>>(), vec![
        ("faction_leader", 100, 1),
        ("general", 101, 3),
        ("general", 102, 3),
    ]);
}

#[test]
fn test_party_power_split() {
    let tables = government_tables();

    let powers = |faction| political_parties(&tables, faction).unwrap().iter().map(|party| party.power()).collect::<Vec<_>>();

    assert_eq!(powers("fact_a"), vec![300, 200]);
    assert_eq!(powers("fact_b"), vec![600, 600]);
}

#[test]
fn test_leader_by_position_or_first_living() {
    let tables = government_tables();
    let index = fixture_index(&tables);

    assert_eq!(faction_leader(&index, "fact_a").unwrap().id(), "31");
    assert_eq!(faction_leader(&index, "fact_b").unwrap().id(), "33");
}

#[test]
#[ignore = "needs the game data, see startpos_test.rs"]
fn test_government_real_data() {
    let tables = test_data_tables();
    let index = CampaignIndex::new(tables, TEST_CAMPAIGN).unwrap();
    let (mut post_matches, mut party_matches, mut leader_matches, mut leaders) = (0, 0, 0, 0);
    let mut leader_mismatches = vec![];

    for faction_record in world_factions(world(test_startpos())) {
        let faction = index.faction(field_str(faction_record, 1)).unwrap();
        let government = child_record(faction_record, "GOVERNMENT").unwrap();

        // Post groups: a count, then that many CHARACTER_POST records (#0 id, #1 position, #4 unique id).
        let mut leader_post = None;
        let mut startpos_posts = vec![];
        for group in child_record(government, "CHARACTER_POSTS").map(|posts| posts.children().to_vec()).unwrap_or_default() {
            let posts = group.iter().filter_map(as_record).collect::<Vec<_>>();
            let unique_id = match field(posts[0], 4) { NodeType::I64(value) => *value, other => panic!("{other:?}") };
            if field_str(posts[0], 1) == FACTION_LEADER_POSITION {
                leader_post = Some(field_u32(posts[0], 0));
            }
            startpos_posts.push((field_str(posts[0], 1).to_owned(), unique_id, u32::try_from(posts.len()).unwrap()));
        }
        let mut generated = government_posts(tables, &index, faction).unwrap().iter().map(|group| (group.position().to_owned(), group.unique_id(), group.count())).collect::<Vec<_>>();
        startpos_posts.sort();
        generated.sort();
        post_matches += usize::from(startpos_posts == generated);

        let mut startpos_parties = child_record(faction_record, "CAMPAIGN_POLITICS")
            .and_then(|politics| child_record(politics, "POLITICAL_PARTIES"))
            .map(|parties| array_items(parties, "CAMPAIGN_POLITICAL_PARTY").iter().map(|party| (field_str(party, 0).to_owned(), field_u32(party, 1))).collect::<Vec<_>>())
            .unwrap_or_default();
        let mut generated = political_parties(tables, faction.key()).unwrap().iter().map(|party| (party.key().to_owned(), party.power())).collect::<Vec<_>>();
        startpos_parties.sort();
        generated.sort();
        party_matches += usize::from(startpos_parties == generated);

        // The leader holds the leader post; it's matched to its table row by its first name (the startpos drops
        // names it can't resolve, like the surname of Armenia's leader).
        let characters = child_record(faction_record, "CHARACTER_ARRAY").map(|array| array_items(array, "CHARACTER")).unwrap_or_default();
        if let Some(character) = characters.iter().find(|character| leader_post.is_some_and(|post| field_u32(character, 8) == post)) {
            leaders += 1;
            let leader = faction_leader(&index, faction.key()).unwrap();
            if startpos_name_ids(character).contains(&table_name(tables, leader.id())) {
                leader_matches += 1;
            } else {
                leader_mismatches.push(faction.key().to_owned());
            }
        }
    }

    assert_eq!(post_matches, 118);
    assert_eq!(party_matches, 118);
    assert_eq!(leaders, 60);
    assert_eq!(leader_matches, 60, "{leader_mismatches:?}");
}

/// Returns the name ids of a startpos character (its `names_name_<id>` keys).
fn startpos_name_ids(character: &RecordNode) -> BTreeSet<String> {
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

/// Returns the first name id of a `start_pos_characters` row.
fn table_name(tables: &StartposTables, id: &str) -> String {
    tables.table("start_pos_characters").unwrap().rows().find(|row| row.str("ID").unwrap() == id).unwrap().str("Name").unwrap().into_owned()
}
