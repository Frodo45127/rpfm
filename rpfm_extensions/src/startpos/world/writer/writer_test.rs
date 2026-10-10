//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Comparison of written records against game-built ones, field by field.

use rpfm_lib::files::esf::{NodeType, RecordNode};

use std::collections::{BTreeMap, HashMap};

use super::*;
use crate::startpos::layout::Slot;
use crate::startpos::startpos_test::{field_str, test_data_esf, test_data_tables, test_startpos, world, world_factions, world_regions, TEST_CAMPAIGN};

/// Share of the game's `WORLD` fields the writer must match. Raised as the writer covers more records.
const MIN_MATCH_RATIO: f64 = 0.48;

/// Matches and totals of each field, by `path:field`.
#[derive(Debug, Default)]
struct Report {
    fields: BTreeMap<String, (usize, usize)>,
}

impl Report {

    /// Counts a comparison of a field.
    fn count(&mut self, field: String, matches: bool) {
        let entry = self.fields.entry(field).or_default();
        entry.0 += usize::from(matches);
        entry.1 += 1;
    }

    /// Returns the matches and the total over every field.
    fn totals(&self) -> (usize, usize) {
        self.fields.values().fold((0, 0), |(matches, total), (field_matches, field_total)| (matches + field_matches, total + field_total))
    }

    /// Returns the fields with mismatches, worst first, as text.
    fn worst(&self, count: usize) -> String {
        let mut fields = self.fields.iter().filter(|(_, (matches, total))| matches < total).collect::<Vec<_>>();
        fields.sort_by_key(|(_, (matches, total))| std::cmp::Reverse(total - matches));
        fields.iter().take(count).map(|(field, (matches, total))| format!("{matches}/{total} {field}")).collect::<Vec<_>>().join("\n")
    }
}

/// Returns the identity key of an array group: its first string, its first record's first string, or its first two
/// leaves (the cell of cell lists, which the game stores in hash order).
fn group_key(group: &[NodeType]) -> Option<String> {
    group.iter().find_map(|node| match node {
        NodeType::Ascii(value) | NodeType::Utf16(value) => Some(value.to_owned()),
        _ => None,
    }).or_else(|| group.iter().find_map(|node| match node {
        NodeType::Record(record) => record.children().first().and_then(|first| group_key(first)),
        _ => None,
    })).or_else(|| {
        let leaves = group.iter().filter(|node| !matches!(node, NodeType::Record(_))).take(2).collect::<Vec<_>>();
        (leaves.len() == 2).then(|| format!("{leaves:?}"))
    })
}

/// Pairs the groups of two records: by key when every key is unique on both sides, by position otherwise.
fn pair_groups<'a>(ours: &'a [Vec<NodeType>], theirs: &'a [Vec<NodeType>]) -> Vec<(Option<&'a [NodeType]>, &'a [NodeType])> {
    let keys = |groups: &'a [Vec<NodeType>]| groups.iter().map(|group| group_key(group)).collect::<Option<Vec<_>>>();
    if let (Some(our_keys), Some(their_keys)) = (keys(ours), keys(theirs)) {
        let ours_by_key = our_keys.iter().zip(ours).map(|(key, group)| (key.as_str(), group.as_slice())).collect::<HashMap<_, _>>();
        let unique = ours_by_key.len() == ours.len() && their_keys.iter().collect::<std::collections::HashSet<_>>().len() == theirs.len();
        if unique {
            return their_keys.iter().zip(theirs).map(|(key, group)| (ours_by_key.get(key.as_str()).copied(), group.as_slice())).collect();
        }
    }
    theirs.iter().enumerate().map(|(index, group)| (ours.get(index).map(Vec::as_slice), group.as_slice())).collect()
}

/// Compares a written record with a game-built one, adding every field to the report.
fn compare(layout: &StartposLayout, ours: &RecordNode, theirs: &RecordNode, path: &str, report: &mut Report) {
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
            record_layout.match_children(group).map(|slots| slots.into_iter().zip(group).map(|(slot, node)| (slot_id(slot), node.clone())).collect()).unwrap_or_default()
        };
        let (our_fields, their_fields) = (fields(our_group), fields(their_group));

        for slot in record_layout.slots() {
            let id = slot_id(slot);
            match (our_fields.get(&id), their_fields.get(&id)) {
                (Some(NodeType::Record(our_child)), Some(NodeType::Record(their_child))) => compare(layout, our_child, their_child, &child_path(path, &id), report),
                (None, Some(NodeType::Record(their_child))) => {
                    for _ in 0..their_child.children().iter().map(|group| leaf_count(group)).sum::<usize>().max(1) {
                        report.count(format!("{path}:{id}<missing>"), false);
                    }
                },
                (ours, Some(theirs)) => report.count(format!("{path}:{id}"), ours == Some(theirs)),
                (Some(_), None) => report.count(format!("{path}:{id}<extra>"), false),
                (None, None) => {},
            }
        }
    }
}

/// Returns the number of leaves in a group and everything under it.
fn leaf_count(group: &[NodeType]) -> usize {
    group.iter().map(|node| match node {
        NodeType::Record(record) => record.children().iter().map(|group| leaf_count(group)).sum(),
        _ => 1,
    }).sum()
}

/// Returns a slot's name (leaves) or key (records).
fn slot_id(slot: &Slot) -> String {
    match slot {
        Slot::Leaf { name, .. } => name.to_owned(),
        Slot::Record { key, .. } => key.to_owned(),
    }
}

#[test]
#[ignore = "needs the game data, see startpos_test.rs"]
fn test_world_matches_the_game_startpos() {
    let tables = test_data_tables();
    let index = CampaignIndex::new(tables, TEST_CAMPAIGN).unwrap();
    let layout = StartposLayout::from_esf(test_startpos()).unwrap();
    let map_data = test_data_esf("map_data.esf");
    let settlements = SettlementLayout::all_from_map_data(&map_data, |region| index.settlement_of(region).is_some_and(|settlement| settlement.is_major())).unwrap();
    let region_map = RegionMap::from_map_data(&map_data).unwrap();
    let inputs = WorldInputs { tables, index: &index, layout: &layout, settlements: &settlements, region_map: &region_map };

    // The game's object orders come from hash tables; using them here checks the rules instead of the orders.
    let game_world = world(test_startpos());
    let factions = world_factions(game_world).iter().map(|faction| index.faction(field_str(faction, 1)).unwrap()).collect();
    let regions = world_regions(game_world).iter().map(|region| index.region(field_str(region, 1)).unwrap()).collect();
    let plan = WorldPlan::with_orders(factions, regions);

    let written = write_world_with_plan(&inputs, &plan).unwrap();
    let mut report = Report::default();
    compare(&layout, &written, game_world, WORLD_PATH, &mut report);

    let (matches, total) = report.totals();
    let ratio = matches as f64 / total as f64;
    assert!(ratio >= MIN_MATCH_RATIO, "{matches}/{total} ({ratio:.4}) fields match. Worst:\n{}", report.worst(60));
}
