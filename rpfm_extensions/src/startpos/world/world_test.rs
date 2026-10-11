//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Fixture shared by the world stage tests, and the tests of [`CampaignIndex`].

use rpfm_lib::files::table::DecodedData;

use super::*;
use crate::startpos::startpos_test::{test_data_tables, TEST_CAMPAIGN};

/// Campaign of the fixture.
pub(crate) const FIXTURE_CAMPAIGN: &str = "camp";

/// Parses a fixture value: booleans, integers (without leading zeros) and floats (with a dot) get their type, the rest
/// is text.
fn value(text: &str) -> DecodedData {
    match text {
        "true" => DecodedData::Boolean(true),
        "false" => DecodedData::Boolean(false),
        _ if text.parse::<i32>().is_ok() && (text == "0" || !text.starts_with('0')) => DecodedData::I32(text.parse().unwrap()),
        _ if text.contains('.') && text.parse::<f32>().is_ok() => DecodedData::F32(text.parse().unwrap()),
        _ => DecodedData::StringU8(text.to_owned()),
    }
}

/// Adds a table to a fixture, with its values parsed by [`value`].
pub(crate) fn add_table(tables: &mut StartposTables, name: &str, columns: &[&str], rows: &[&[&str]]) {
    let rows = rows.iter().map(|row| row.iter().map(|text| value(text)).collect()).collect::<Vec<_>>();
    tables.add_rows(name, columns, &rows).unwrap();
}

/// A small campaign: faction `fact_a` (playable, two regions, capital `reg_a1`), faction `fact_b` (a horde-capable
/// faction with one region), a rebel-owned region `reg_r`, and a faction of another campaign.
pub(crate) fn fixture_tables() -> StartposTables {
    let mut tables = StartposTables::new();
    add_table(&mut tables, "cultures_subcultures", &["subculture", "culture"], &[
        &["sub_a", "cult_a"],
        &["sub_b", "cult_b"],
    ]);
    add_table(&mut tables, "factions", &[
        "key", "subculture", "military_group", "senator_total", "can_be_horde", "flags_path", "republican_flag_path",
        "primary_colour_hex", "secondary_colour_hex", "uniform_colour_hex", "alt_primary_colour_hex", "alt_secondary_colour_hex", "alt_uniform_colour_hex",
    ], &[
        &["fact_a", "sub_a", "mg_a", "500", "false", "flags/a", "flags/a_rep", "FF0000", "00FF00", "0000FF", "101010", "202020", "303030"],
        &["fact_b", "sub_b", "mg_b", "1200", "true", "flags/b", "flags/b_rep", "5B5B5B", "000000", "000000", "000000", "000000", "000000"],
        &["fact_c", "sub_a", "mg_a", "150", "false", "flags/c", "flags/c_rep", "000000", "000000", "000000", "000000", "000000", "000000"],
        &["rebels_x", "sub_b", "mg_b", "0", "false", "flags/rebels", "flags/rebels", "5B5B5B", "000000", "000000", "000000", "000000", "000000"],
    ]);
    add_table(&mut tables, "start_pos_factions", &["ID", "faction", "campaign", "playable", "starting_order"], &[
        &["1", "fact_a", FIXTURE_CAMPAIGN, "true", "10"],
        &["2", "fact_b", FIXTURE_CAMPAIGN, "false", "5"],
        &["3", "fact_c", "other", "true", "0"],
    ]);
    add_table(&mut tables, "start_pos_regions", &["id", "region", "campaign", "owning_faction", "faction_capital", "alternative_rebel_faction"], &[
        &["10", "reg_a1", FIXTURE_CAMPAIGN, "1", "true", "rebels_x"],
        &["11", "reg_a2", FIXTURE_CAMPAIGN, "1", "true", "rebels_x"],
        &["12", "reg_r", FIXTURE_CAMPAIGN, "", "false", "rebels_x"],
        &["13", "reg_b1", FIXTURE_CAMPAIGN, "2", "false", "rebels_x"],
        &["14", "reg_c1", "other", "3", "true", "rebels_x"],
    ]);
    add_table(&mut tables, "start_pos_settlements", &["id", "region", "primary_building", "port_building", "building1", "building2", "building3", "building4", "building5", "startpos_slave_points"], &[
        &["20", "10", "bld_city_major_1", "bld_port_1", "bld_farm_1", "", "", "", "", "50"],
        &["21", "11", "bld_city_minor_1", "", "", "bld_farm_1", "", "", "", "50"],
        &["22", "12", "bld_city_minor_1", "", "", "", "", "", "", "40"],
        &["23", "13", "bld_city_minor_1", "", "", "", "", "", "", "50"],
    ]);
    add_table(&mut tables, "start_pos_characters", &["ID", "faction", "Type", "Age", "startx", "starty", "ministerial_position", "loyalty", "death_type", "progenitor", "override_general_unit"], &[
        &["30", "1", "general", "40", "0.0", "0.0", "", "0", "", "true", ""],
        &["31", "1", "general", "20", "5.0", "7.0", "faction_leader", "16", "", "false", ""],
        &["32", "1", "general", "60", "0.0", "0.0", "", "5", "EXECUTED", "false", ""],
        &["33", "2", "general", "30", "0.0", "0.0", "", "5", "", "false", ""],
        &["34", "3", "general", "30", "0.0", "0.0", "", "5", "", "false", ""],
    ]);
    tables
}

/// Indexes the fixture campaign.
pub(crate) fn fixture_index(tables: &StartposTables) -> CampaignIndex {
    CampaignIndex::new(tables, FIXTURE_CAMPAIGN).unwrap()
}

#[test]
fn test_index_keeps_only_the_campaign() {
    let index = fixture_index(&fixture_tables());

    assert_eq!(index.factions().iter().map(|faction| faction.key().as_str()).collect::<Vec<_>>(), vec!["fact_a", "fact_b"]);
    assert_eq!(index.regions().len(), 4);
    assert_eq!(index.settlements().len(), 4);
    assert_eq!(index.characters().len(), 4);
}

#[test]
fn test_index_resolves_references() {
    let index = fixture_index(&fixture_tables());

    assert_eq!(index.faction("fact_b").unwrap().culture(), "cult_b");
    assert_eq!(index.region("reg_a2").unwrap().owner().as_deref(), Some("fact_a"));
    assert_eq!(index.region("reg_r").unwrap().owner(), &None);
    assert_eq!(index.settlement_of("reg_a1").unwrap().all_buildings().collect::<Vec<_>>(), vec!["bld_city_major_1", "bld_port_1", "bld_farm_1"]);
    assert_eq!(index.characters_of("fact_a").count(), 3);
    assert_eq!((index.characters()[1].start_x(), index.characters()[1].start_y()), (5, 7));
}

#[test]
fn test_capital_is_the_first_flagged_region() {
    let index = fixture_index(&fixture_tables());

    assert_eq!(index.capital_of("fact_a").unwrap().key(), "reg_a1");
    assert!(index.capital_of("fact_b").is_none());
}

#[test]
fn test_startpos_order_puts_playable_factions_first() {
    let index = fixture_index(&fixture_tables());

    let order = index.factions_in_startpos_order().iter().map(|faction| faction.key().as_str()).collect::<Vec<_>>();
    assert_eq!(order, vec!["fact_a", "fact_b"]);
}

#[test]
#[ignore = "needs the game data, see startpos_test.rs"]
fn test_index_real_data() {
    let index = CampaignIndex::new(test_data_tables(), TEST_CAMPAIGN).unwrap();

    assert_eq!(index.factions().len(), 118);
    assert_eq!(index.regions().len(), 186);
    assert_eq!(index.regions().iter().filter(|region| region.owner().is_none()).count(), 11);
    assert_eq!(index.capital_of("att_fact_sassanid_empire").unwrap().key(), "att_reg_asorstan_ctesiphon");
}
