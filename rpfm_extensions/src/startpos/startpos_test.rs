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

use std::collections::{BTreeMap, HashMap};
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
use super::layout::{child_path, StartposLayout};
use super::tables::StartposTables;

/// Environment variable with the folder of the game data used by the ignored tests.
const TEST_DATA_VAR: &str = "RPFM_STARTPOS_TEST_DATA";

/// Relative difference under which two floats of a startpos count as equal.
const FLOAT_TOLERANCE: f32 = 1e-3;

/// Campaign of the test data.
pub(crate) const TEST_CAMPAIGN: &str = "main_attila";

/// Tables the world stage tests read.
const TEST_TABLES: &[&str] = &[
    "agent_culture_details", "agent_to_agent_abilities", "agent_to_agent_attributes", "agents",
    "armed_citizenry_units_to_unit_groups_junctions", "battlefield_engines", "building_level_armed_citizenry_junctions",
    "building_levels", "campaign_character_art_sets", "campaign_character_art_sets_group_junctions",
    "character_skill_node_sets", "character_trait_levels", "commander_unit_permissions", "commodities", "cultures_subcultures",
    "faction_political_parties_junctions", "faction_to_mercenary_set_junctions", "factions", "land_units", "main_units",
    "mercenary_pool_to_groups_junctions", "mercenary_unit_groups", "military_force_legacy_emblems",
    "military_force_legacy_names", "ministerial_positions", "ministerial_positions_culture_details", "naval_units",
    "political_parties", "province_to_mercenary_set_junctions", "region_to_province_junctions", "religions", "resources",
    "start_pos_calendars", "start_pos_character_to_settlements", "start_pos_character_traits", "start_pos_characters",
    "start_pos_diplomacy", "start_pos_factions", "start_pos_horde_details", "start_pos_land_units",
    "start_pos_naval_units", "start_pos_past_events", "start_pos_region_religions", "start_pos_region_slot_templates",
    "start_pos_regions", "start_pos_settlements", "start_pos_technologies", "start_pos_victory_conditions",
    "technology_unit_upgrades", "units_to_groupings_military_permissions",
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

/// Matches and totals of each field, by `path:field`.
#[derive(Debug, Default)]
pub(crate) struct Report {
    fields: BTreeMap<String, (usize, usize)>,
}

impl Report {

    /// Counts a comparison of a field.
    pub(crate) fn count(&mut self, field: String, matches: bool) {
        let entry = self.fields.entry(field).or_default();
        entry.0 += usize::from(matches);
        entry.1 += 1;
    }

    /// Returns the matches and the total over every field.
    pub(crate) fn totals(&self) -> (usize, usize) {
        self.fields.values().fold((0, 0), |(matches, total), (field_matches, field_total)| (matches + field_matches, total + field_total))
    }

    /// Returns the fields with mismatches, worst first, as text.
    pub(crate) fn worst(&self, count: usize) -> String {
        let mut fields = self.fields.iter().filter(|(_, (matches, total))| matches < total).collect::<Vec<_>>();
        fields.sort_by_key(|(_, (matches, total))| std::cmp::Reverse(total - matches));
        fields.iter().take(count).map(|(field, (matches, total))| format!("{matches}/{total} {field}")).collect::<Vec<_>>().join("\n")
    }
}

/// Kinds of identity keys of an array group, tried in order until one is unique on both sides.
#[derive(Debug, Clone, Copy)]
pub(crate) enum KeyKind {

    /// The group's first string, or its first record's first string.
    String,

    /// The group's first two leaves (ids, or the cell of cell lists, which the game stores in hash order).
    Leaves,
}

/// Returns an identity key of an array group.
pub(crate) fn group_key(group: &[NodeType], kind: KeyKind) -> Option<String> {
    match kind {
        KeyKind::String => group.iter().find_map(|node| match node {
            NodeType::Ascii(value) | NodeType::Utf16(value) => Some(value.to_owned()),
            _ => None,
        }).or_else(|| group.iter().find_map(|node| match node {
            NodeType::Record(record) => record.children().first().and_then(|first| group_key(first, KeyKind::String)),
            _ => None,
        })),
        KeyKind::Leaves => {
            let leaves = group.iter().filter(|node| !matches!(node, NodeType::Record(_))).take(2).collect::<Vec<_>>();
            match leaves.as_slice() {
                [] => group.iter().find_map(|node| match node {
                    NodeType::Record(record) => record.children().first().and_then(|first| group_key(first, KeyKind::Leaves)),
                    _ => None,
                }),
                leaves => Some(format!("{leaves:?}")),
            }
        },
    }
}

/// Pairs the groups of two records: single groups together, by the first kind of key unique on both sides, by
/// position otherwise.
pub(crate) fn pair_groups<'a>(ours: &'a [Vec<NodeType>], theirs: &'a [Vec<NodeType>]) -> Vec<(Option<&'a [NodeType]>, &'a [NodeType])> {
    if let ([our_group], [their_group]) = (ours, theirs) {
        return vec![(Some(our_group.as_slice()), their_group.as_slice())];
    }
    for kind in [KeyKind::String, KeyKind::Leaves] {
        let keys = |groups: &'a [Vec<NodeType>]| groups.iter().map(|group| group_key(group, kind)).collect::<Option<Vec<_>>>();
        if let (Some(our_keys), Some(their_keys)) = (keys(ours), keys(theirs)) {
            let ours_by_key = our_keys.iter().zip(ours).map(|(key, group)| (key.as_str(), group.as_slice())).collect::<HashMap<_, _>>();
            let unique = ours_by_key.len() == ours.len() && their_keys.iter().collect::<std::collections::HashSet<_>>().len() == theirs.len();
            if unique {
                return their_keys.iter().zip(theirs).map(|(key, group)| (ours_by_key.get(key.as_str()).copied(), group.as_slice())).collect();
            }
        }
    }
    theirs.iter().enumerate().map(|(index, group)| (ours.get(index).map(Vec::as_slice), group.as_slice())).collect()
}

/// Compares a written record with a game-built one, adding every field to the report.
pub(crate) fn compare(layout: &StartposLayout, ours: &RecordNode, theirs: &RecordNode, path: &str, report: &mut Report) {
    let Some(record_layout) = layout.record(path) else {
        report.count(format!("{path}:<no layout>"), false);
        return;
    };

    for (our_group, their_group) in pair_groups(ours.children(), theirs.children()) {
        let Some(our_group) = our_group else {
            for _ in 0..leaf_count(their_group) {
                report.count(format!("{path}:<missing group>"), false);
            }
            continue;
        };

        if record_layout.free_form() {
            report.count(format!("{path}:<children>"), our_group == their_group);
            continue;
        }

        let fields = |group: &'_ [NodeType]| -> HashMap<String, NodeType> {
            record_layout.match_children(group).map(|slots| slots.into_iter().zip(group).map(|(slot, node)| (slot.id().to_owned(), node.clone())).collect()).unwrap_or_default()
        };
        let (our_fields, their_fields) = (fields(our_group), fields(their_group));

        for slot in record_layout.slots() {
            let id = slot.id().to_owned();
            match (our_fields.get(&id), their_fields.get(&id)) {
                (Some(NodeType::Record(our_child)), Some(NodeType::Record(their_child))) => compare(layout, our_child, their_child, &child_path(path, &id), report),
                (None, Some(NodeType::Record(their_child))) => {
                    for _ in 0..their_child.children().iter().map(|group| leaf_count(group)).sum::<usize>().max(1) {
                        report.count(format!("{path}:{id}<missing>"), false);
                    }
                },
                (ours, Some(theirs)) => report.count(format!("{path}:{id}"), ours.is_some_and(|ours| same_leaf(ours, theirs))),
                (Some(_), None) => report.count(format!("{path}:{id}<extra>"), false),
                (None, None) => {},
            }
        }
    }
}

/// Returns if two leaves are equal. Floats only need to be close: the game's float rounding isn't always
/// reproducible (trade spline points are off by one ulp in 10% of the cases).
pub(crate) fn same_leaf(ours: &NodeType, theirs: &NodeType) -> bool {
    let close = |ours: f32, theirs: f32| (ours - theirs).abs() <= FLOAT_TOLERANCE * theirs.abs().max(1.0);
    match (ours, theirs) {
        (NodeType::F32(ours), NodeType::F32(theirs)) => close(*ours.value(), *theirs.value()) && ours.optimized() == theirs.optimized(),
        (NodeType::Coord2d(ours), NodeType::Coord2d(theirs)) => close(*ours.x(), *theirs.x()) && close(*ours.y(), *theirs.y()),
        (NodeType::F32Array(ours), NodeType::F32Array(theirs)) => ours.len() == theirs.len() && ours.iter().zip(theirs).all(|(ours, theirs)| close(*ours, *theirs)),
        _ => ours == theirs,
    }
}

/// Returns the number of leaves in a group and everything under it.
pub(crate) fn leaf_count(group: &[NodeType]) -> usize {
    group.iter().map(|node| match node {
        NodeType::Record(record) => record.children().iter().map(|group| leaf_count(group)).sum(),
        _ => 1,
    }).sum()
}

