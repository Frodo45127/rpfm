//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Shared helpers of the startpos tests.
//!
//! Tests that compare against game data are `#[ignore]`d, because the game files can't be distributed. To run
//! them, point `RPFM_STARTPOS_TEST_DATA` to a folder with Attila's main campaign files (`map_data.esf`,
//! `trade_routes.ptd`), a game-built `startpos.esf`, the game's `data.pack`, the campaign's `main_attila.pack` (as
//! built by the Assembly Kit) and the game's schema as `schema.ron`, then run `cargo test -- --ignored`.

use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
use std::sync::OnceLock;

use rpfm_lib::files::{Container, Decodeable, DecodeableExtraData, RFileDecoded};
use rpfm_lib::files::esf::{ESF, NodeType, RecordNode};
use rpfm_lib::files::pack::Pack;
use rpfm_lib::games::supported_games::{SupportedGames, KEY_ATTILA};
use rpfm_lib::schema::Schema;

use super::esf_nav::{as_record, as_u32, child_record};
use super::tables::StartposTables;

/// Environment variable with the folder of the game data used by the ignored tests.
const TEST_DATA_VAR: &str = "RPFM_STARTPOS_TEST_DATA";

/// Campaign of the test data.
pub(crate) const TEST_CAMPAIGN: &str = "main_attila";

/// Tables the world stage tests read.
const TEST_TABLES: &[&str] = &[
    "agent_culture_details", "agent_to_agent_abilities", "agent_to_agent_attributes", "agents",
    "armed_citizenry_units_to_unit_groups_junctions", "battlefield_engines", "building_level_armed_citizenry_junctions",
    "building_levels", "campaign_character_art_sets", "campaign_character_art_sets_group_junctions",
    "character_skill_node_sets", "character_trait_levels", "commander_unit_permissions", "cultures_subcultures",
    "faction_political_parties_junctions", "faction_to_mercenary_set_junctions", "factions", "land_units", "main_units",
    "mercenary_pool_to_groups_junctions", "mercenary_unit_groups", "military_force_legacy_emblems",
    "military_force_legacy_names", "ministerial_positions", "ministerial_positions_culture_details", "naval_units",
    "political_parties", "province_to_mercenary_set_junctions", "region_to_province_junctions", "religions",
    "start_pos_calendars", "start_pos_character_to_settlements", "start_pos_character_traits", "start_pos_characters",
    "start_pos_diplomacy", "start_pos_factions", "start_pos_horde_details", "start_pos_land_units",
    "start_pos_naval_units", "start_pos_region_religions", "start_pos_region_slot_templates", "start_pos_regions",
    "start_pos_settlements", "start_pos_technologies", "start_pos_victory_conditions", "technology_unit_upgrades",
    "units_to_groupings_military_permissions",
];

/// Returns the path of a file of the startpos test data folder.
pub(crate) fn test_data_path(file_name: &str) -> PathBuf {
    let folder = std::env::var(TEST_DATA_VAR).unwrap_or_else(|_| panic!("set {TEST_DATA_VAR} to run this test"));
    PathBuf::from(folder).join(file_name)
}

/// Decodes an ESF file of the startpos test data folder.
pub(crate) fn test_data_esf(file_name: &str) -> ESF {
    let mut reader = BufReader::new(File::open(test_data_path(file_name)).unwrap());
    ESF::decode(&mut reader, &None).unwrap()
}

/// Returns the game-built startpos of the test data, decoded once for every test.
pub(crate) fn test_startpos() -> &'static ESF {
    static STARTPOS: OnceLock<ESF> = OnceLock::new();
    STARTPOS.get_or_init(|| test_data_esf("startpos.esf"))
}

/// Returns the tables of the test data packs (`data.pack`, then `main_attila.pack`), decoded once for every test.
pub(crate) fn test_data_tables() -> &'static StartposTables {
    static TABLES: OnceLock<StartposTables> = OnceLock::new();
    TABLES.get_or_init(|| {
        let schema = Schema::load(&test_data_path("schema.ron"), None).unwrap();
        let games = SupportedGames::default();
        let game = games.game(KEY_ATTILA).unwrap();
        let mut extra = DecodeableExtraData::default();
        extra.set_schema(Some(&schema));
        let extra = Some(extra);

        let mut tables = StartposTables::new();
        for pack_name in ["data.pack", "main_attila.pack"] {
            let mut pack = Pack::read_and_merge(&[test_data_path(pack_name)], game, true, false, false).unwrap();
            let paths = pack.files().keys()
                .filter(|path| TEST_TABLES.iter().any(|table| path.starts_with(&format!("db/{table}_tables/"))))
                .cloned()
                .collect::<Vec<_>>();
            for path in paths {
                let file = pack.file_mut(&path, false).unwrap();
                file.decode(&extra, true, false).unwrap();
                if let Ok(RFileDecoded::DB(db)) = file.decoded() {
                    tables.add_db(db).unwrap();
                }
            }
        }
        tables
    })
}

/// Returns the `WORLD` record of a startpos.
pub(crate) fn world(startpos: &ESF) -> &RecordNode {
    ["CAMPAIGN_ENV", "CAMPAIGN_MODEL", "WORLD"].iter()
        .try_fold(as_record(startpos.root_node()).unwrap(), |record, name| child_record(record, name))
        .unwrap()
}

/// Returns the records of an array record (one per group, the group's first record), in order.
pub(crate) fn array_items<'a>(array: &'a RecordNode, item: &'a str) -> Vec<&'a RecordNode> {
    array.children().iter()
        .filter_map(|group| group.iter().filter_map(as_record).find(|record| record.name() == item))
        .collect()
}

/// Returns the factions of a startpos `WORLD`, in order.
pub(crate) fn world_factions(world: &RecordNode) -> Vec<&RecordNode> {
    child_record(world, "FACTION_ARRAY").map(|array| array_items(array, "FACTION")).unwrap_or_default()
}

/// Returns the child at a position of a record's first group (`#n` in the research notes).
pub(crate) fn field(record: &RecordNode, position: usize) -> &NodeType {
    &record.children()[0][position]
}

/// Returns the string at a position of a record's first group.
pub(crate) fn field_str(record: &RecordNode, position: usize) -> &str {
    match field(record, position) {
        NodeType::Ascii(value) | NodeType::Utf16(value) => value,
        other => panic!("not a string: {other:?}"),
    }
}

/// Returns the regions of a startpos `WORLD`, in order.
pub(crate) fn world_regions(world: &RecordNode) -> Vec<&RecordNode> {
    child_record(world, "REGION_MANAGER")
        .and_then(|manager| child_record(manager, "REGIONS_ARRAY"))
        .map(|array| array_items(array, "REGION"))
        .unwrap_or_default()
}

/// Returns the unsigned number at a position of a record's first group.
pub(crate) fn field_u32(record: &RecordNode, position: usize) -> u32 {
    as_u32(field(record, position)).unwrap_or_else(|| panic!("not a number: {:?}", field(record, position)))
}

/// Returns the unit keys of every force of a startpos `WORLD`, by force id.
pub(crate) fn force_units(world: &RecordNode) -> HashMap<u32, Vec<String>> {
    let mut forces = HashMap::new();
    for faction in world_factions(world) {
        let Some(armies) = child_record(faction, "ARMY_ARRAY") else { continue };
        for force in array_items(armies, "MILITARY_FORCE") {
            let units = child_record(force, "UNIT_CONTAINER")
                .and_then(|container| child_record(container, "UNITS_ARRAY"))
                .map(|array| array_items(array, "UNIT"))
                .unwrap_or_default()
                .into_iter()
                .filter_map(|unit| child_record(unit, "UNIT_RECORD_KEY").map(|key| field_str(key, 0).to_owned()))
                .collect();
            forces.insert(field_u32(force, 0), units);
        }
    }
    forces
}
