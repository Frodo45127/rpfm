//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

use rpfm_lib::files::pack::Pack;
use rpfm_lib::files::table::DecodedData;
use rpfm_lib::games::supported_games::{SupportedGames, KEY_ATTILA};

use super::*;
use crate::dependencies::Dependencies;
use crate::startpos::generator::load_tables;
use crate::startpos::startpos_test::{test_data_path, TEST_CAMPAIGN};
use crate::startpos::tables::tables_test::text;

/// Returns the schema of the test data.
fn test_schema() -> Schema {
    Schema::load(&test_data_path("schema.ron"), None).unwrap()
}

/// Returns tables with a faction, characters (one with a broken name, one in the generals pool leading an army, a
/// child adopted by a progenitor and one adopted by another character), and a unit of a missing character.
fn broken_tables() -> StartposTables {
    let mut tables = StartposTables::new();
    tables.add_rows("campaigns", &["campaign_name"], &[vec![text("main_attila")]]).unwrap();
    tables.add_rows("start_pos_factions", &["ID"], &[vec![text("1")]]).unwrap();
    tables.add_rows("names", &["id"], &[vec![text("10")]]).unwrap();
    tables.add_rows("start_pos_characters", &["ID", "faction", "Name", "is_in_generals_pool", "Age", "progenitor"], &[
        vec![text("100"), text("1"), text("99"), DecodedData::Boolean(false), DecodedData::I32(40), DecodedData::Boolean(true)],
        vec![text("101"), text("1"), text("10"), DecodedData::Boolean(true), DecodedData::I32(30), DecodedData::Boolean(false)],
        vec![text("103"), text("1"), text("10"), DecodedData::Boolean(false), DecodedData::I32(4), DecodedData::Boolean(false)],
        vec![text("104"), text("1"), text("10"), DecodedData::Boolean(false), DecodedData::I32(2), DecodedData::Boolean(false)],
    ]).unwrap();
    tables.add_rows("start_pos_family_relationships", &["character", "related_to", "relationship", "adopted"], &[
        vec![text("103"), text("100"), text("child"), DecodedData::Boolean(true)],
        vec![text("104"), text("101"), text("child"), DecodedData::Boolean(true)],
        vec![text("104"), text("100"), text("child"), DecodedData::Boolean(false)],
    ]).unwrap();
    tables.add_rows("start_pos_character_to_settlements", &["character"], &[vec![text("101")]]).unwrap();
    tables.add_rows("start_pos_land_units", &["id", "general"], &[vec![text("1"), text("101")], vec![text("2"), text("102")]]).unwrap();
    tables
}

#[test]
#[ignore = "needs the game data, see startpos_test.rs"]
fn test_broken_data_is_reported() {
    let issues = validate(&broken_tables(), &test_schema(), "missing_campaign");
    let summary = issues.iter().map(|issue| (issue.severity, issue.table.as_str(), issue.row.as_str(), issue.column.as_deref())).collect::<Vec<_>>();

    assert_eq!(summary, vec![
        (Severity::Error, "campaigns", "missing_campaign", None),
        (Severity::Error, "start_pos_characters", "100", Some("Name")),
        (Severity::Error, "start_pos_characters", "101", Some("is_in_generals_pool")),
        (Severity::Error, "start_pos_family_relationships", "103|100", Some("adopted")),
        (Severity::Warning, "start_pos_land_units", "2", Some("general")),
    ]);
}

#[test]
#[ignore = "needs the game data, see startpos_test.rs"]
fn test_game_data_has_no_errors() {
    let games = SupportedGames::default();
    let game = games.game(KEY_ATTILA).unwrap();
    let schema = test_schema();
    let pack = Pack::read_and_merge(&["data.pack", "main_attila.pack"].map(test_data_path), game, true, false, false).unwrap();
    let tables = load_tables(&Dependencies::default(), &pack, &schema).unwrap();

    let issues = validate(&tables, &schema, TEST_CAMPAIGN);
    let errors = issues.iter().filter(|issue| issue.severity == Severity::Error).map(ToString::to_string).collect::<Vec<_>>();
    assert!(errors.is_empty(), "{errors:#?}");
}

#[test]
fn test_data_the_game_drops_is_reported() {
    let mut tables = StartposTables::new();
    tables.add_rows("names", &["id", "type"], &[vec![text("10"), DecodedData::I32(0)], vec![text("11"), DecodedData::I32(1)]]).unwrap();
    tables.add_rows("start_pos_characters", &["ID", "Name", "Surname", "clan_name", "other_name"], &[
        vec![text("100"), text("10"), text("11"), text(""), text("")],
        vec![text("101"), text("10"), text("10"), text(""), text("")],
    ]).unwrap();
    let flag = DecodedData::Boolean;
    tables.add_rows("start_pos_diplomacy", &["key", "faction1", "faction2", "stance", "non_aggression_pact", "grants_military_access"], &[
        vec![text("1"), text("2"), text("1"), text("vassal"), flag(false), flag(false)],
        vec![text("2"), text("2"), text("3"), text("neutral"), flag(true), flag(false)],
        vec![text("3"), text("1"), text("3"), text("neutral"), flag(true), flag(true)],
    ]).unwrap();

    let mut issues = vec![];
    validate_name_kinds(&tables, &mut issues);
    validate_vassal_pacts(&tables, &mut issues);
    let summary = issues.iter().map(|issue| (issue.table.as_str(), issue.row.as_str(), issue.column.as_deref())).collect::<Vec<_>>();

    assert_eq!(summary, vec![
        ("start_pos_characters", "101", Some("Surname")),
        ("start_pos_diplomacy", "2", Some("non_aggression_pact")),
    ]);
}

#[test]
#[ignore = "needs the game data, see startpos_test.rs"]
fn test_game_data_drops_are_reported() {
    let games = SupportedGames::default();
    let game = games.game(KEY_ATTILA).unwrap();
    let schema = test_schema();
    let pack = Pack::read_and_merge(&["data.pack", "main_attila.pack"].map(test_data_path), game, true, false, false).unwrap();
    let tables = load_tables(&Dependencies::default(), &pack, &schema).unwrap();

    let issues = validate(&tables, &schema, TEST_CAMPAIGN);
    let count = |table: &str| issues.iter().filter(|issue| issue.table == table && issue.severity == Severity::Warning).count();

    // The names, technologies and pacts the game drops from Attila's main campaign.
    assert_eq!(count("start_pos_technologies"), 2);
    assert_eq!(count("start_pos_diplomacy"), 2);
    assert_eq!(issues.iter().filter(|issue| issue.table == "start_pos_characters" && issue.message.contains("isn't of type")).count(), 5);
}
