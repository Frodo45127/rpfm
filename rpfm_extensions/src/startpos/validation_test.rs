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

/// Returns tables with a faction, two characters (one with a broken name, one in the generals pool leading an army)
/// and a unit of a missing character.
fn broken_tables() -> StartposTables {
    let mut tables = StartposTables::new();
    tables.add_rows("campaigns", &["campaign_name"], &[vec![text("main_attila")]]).unwrap();
    tables.add_rows("start_pos_factions", &["ID"], &[vec![text("1")]]).unwrap();
    tables.add_rows("names", &["id"], &[vec![text("10")]]).unwrap();
    tables.add_rows("start_pos_characters", &["ID", "faction", "Name", "is_in_generals_pool"], &[
        vec![text("100"), text("1"), text("99"), DecodedData::Boolean(false)],
        vec![text("101"), text("1"), text("10"), DecodedData::Boolean(true)],
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
