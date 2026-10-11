//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

use std::collections::HashMap;

use super::*;
use crate::startpos::cai::cai_test::{game_cai_record, id_translation, test_inputs, translate};
use crate::startpos::cai::CAI_WORLD_PATH;
use crate::startpos::esf_nav::as_u32;
use crate::startpos::startpos_test::{compare, test_data_esf, test_data_tables, test_startpos, world, Report, TEST_CAMPAIGN};
use crate::startpos::world::CampaignIndex;

/// Share of the central pool's fields the writer must match, leaving out its beliefs and their links (left empty).
const MIN_CENTRAL_MATCH_RATIO: f64 = 0.95;

/// Lists left empty on purpose: the game rebuilds the beliefs, and the components' links to them.
const EMPTY_LISTS: [&str; 2] = ["CAI_BDI_POOL_BELIEFS", "CAI_BDI_COMPONENT_BLOCK_OWNS"];

/// Share of the mobiles' influence maps at the game's cell. The rest are garrison fleets (their port cell is
/// approximate) and agents; the game recomputes the maps anyway.
const MIN_INFLUENCE_CELL_RATIO: f64 = 0.8;

/// Children of each mobile's entry in the influence map manager.
const INFLUENCE_ENTRY_LEN: usize = 16;

/// Game ids of the director pool's objects, in creation order.
const GAME_DIRECTOR_IDS: [u32; 6] = [19506, 19507, 19508, 19509, 19510, 19511];

/// Returns a child record of a written `CAI_INTERFACE`.
fn written_child<'a>(record: &'a RecordNode, name: &str) -> &'a RecordNode {
    child_record(record, name).unwrap()
}

/// Returns a copy of a record with the [`EMPTY_LISTS`] emptied.
fn without_empty_lists(record: &RecordNode) -> RecordNode {
    let mut record = record.clone();
    if EMPTY_LISTS.contains(&record.name().as_str()) {
        record.set_children(vec![]);
    }
    for node in record.children_mut().iter_mut().flatten() {
        if let NodeType::Record(child) = node {
            **child = without_empty_lists(child);
        }
    }
    record
}

/// Compares a written record with the game's, returning the share of fields that match.
fn match_ratio(layout: &StartposLayout, ours: &RecordNode, theirs: &RecordNode, path: &str) -> f64 {
    let mut report = Report::default();
    compare(layout, ours, theirs, path, &mut report);
    let (matches, total) = report.totals();
    println!("{path}: {matches}/{total} fields match. Worst:\n{}", report.worst(15));
    matches as f64 / total as f64
}

/// Returns the (mobile, map B cell) pairs of an influence map manager.
fn influence_cells(record: &RecordNode) -> HashMap<u32, (u16, u16)> {
    let children = &record.children()[0];
    let count = as_u32(&children[0]).unwrap() as usize;
    (0..count).map(|entry| {
        let base = 1 + entry * INFLUENCE_ENTRY_LEN;
        let cell = |offset: usize| match &children[base + offset] { NodeType::U16(value) => *value, node => panic!("not a cell: {node:?}") };
        (as_u32(&children[base]).unwrap(), (cell(11), cell(12)))
    }).collect()
}

#[test]
#[ignore = "needs the game data, see startpos_test.rs"]
fn test_interface_matches_the_game_startpos() {
    let layout = StartposLayout::from_esf(test_startpos()).unwrap();
    let index = CampaignIndex::new(test_data_tables(), TEST_CAMPAIGN).unwrap();
    let (map_data, hlp_data) = (test_data_esf("map_data.esf"), test_data_esf("hlp_data.esf"));
    let inputs = CaiInputs { tables: test_data_tables(), index: &index, layout: &layout, world: world(test_startpos()), map_data: &map_data, hlp_data: &hlp_data, trade_splines: 2486, campaign: TEST_CAMPAIGN };
    let context = CaiContext::new(&inputs).unwrap();
    let written = context.write_interface(&inputs).unwrap();
    assert_eq!(context.plan().director().len(), GAME_DIRECTOR_IDS.len());

    let central = without_empty_lists(written_child(&written, "CAI_BDI_POOL"));
    let central = match_ratio(&layout, &central, &without_empty_lists(game_cai_record("CAI_BDI_POOL")), &child_path(CAI_PATH, "CAI_BDI_POOL"));
    assert!(central >= MIN_CENTRAL_MATCH_RATIO);

    let translation = context.plan().director().iter().copied().zip(GAME_DIRECTOR_IDS).collect();
    let director = translate(written_child(&written, "CAI_INTERFACE_DIRECTOR_BDI_POOL"), &translation);
    let director_ratio = match_ratio(&layout, &director, game_cai_record("CAI_INTERFACE_DIRECTOR_BDI_POOL"), &child_path(CAI_PATH, "CAI_INTERFACE_DIRECTOR_BDI_POOL"));
    assert_eq!(director_ratio, 1.0);

    // Influence maps: an entry per mobile at its cell, then the factions' maps.
    let mobiles = id_translation(&test_inputs());
    let ours = influence_cells(written_child(&written, "CAI_INFLUENCE_MAP_MANAGER")).into_iter()
        .map(|(mobile, cell)| (mobiles.get(&mobile).copied().unwrap_or(mobile), cell))
        .collect::<HashMap<_, _>>();
    let theirs = influence_cells(game_cai_record("CAI_INFLUENCE_MAP_MANAGER"));
    let same_cells = theirs.iter().filter(|(mobile, cell)| ours.get(mobile) == Some(cell)).count();
    println!("influence maps: {} entries (game {}), {same_cells} at the game's cell", ours.len(), theirs.len());
    assert_eq!(ours.len(), theirs.len());
    assert!(same_cells as f64 >= MIN_INFLUENCE_CELL_RATIO * theirs.len() as f64);

    // The factions' maps: count and faction components (the game's maps have tiles, ours start empty).
    let faction_keys = |record: &RecordNode, entries: usize| record.children()[0][1 + INFLUENCE_ENTRY_LEN * entries..].iter().filter_map(as_u32).collect::<Vec<_>>();
    assert_eq!(faction_keys(written_child(&written, "CAI_INFLUENCE_MAP_MANAGER"), ours.len()), faction_keys(game_cai_record("CAI_INFLUENCE_MAP_MANAGER"), theirs.len()));

    let world_record = translate(written_child(&written, "CAI_WORLD"), &id_translation(&test_inputs()));
    assert!(match_ratio(&layout, &world_record, game_cai_record("CAI_WORLD"), CAI_WORLD_PATH) >= 0.99);
}
