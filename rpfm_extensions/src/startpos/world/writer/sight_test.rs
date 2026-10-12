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
use crate::startpos::esf_nav::{as_record, child_record};
use crate::startpos::startpos_test::{field_str, test_data_tables, test_startpos, world, world_factions};
use crate::startpos::vision::vision_test::decode;
use crate::startpos::world::writer::writer_test::{game_plan, world_test_data};

/// Path of the shroud of a faction.
const SHROUD_PATH: &str = "CAMPAIGN_STARTPOS/CAMPAIGN_ENV/CAMPAIGN_MODEL/WORLD/FACTION_ARRAY/FACTION/CAMPAIGN_SHROUD";

/// Filler sights (vision and bounds) that must match the game's, of 186.
const MIN_MATCHING_FILLER_SIGHTS: usize = 150;

/// Returns the settlement cells of each tile of a `CAMPAIGN_SHROUD_FILLERS` record, in order.
fn filler_cells(fillers: &RecordNode) -> Vec<Vec<(u16, u16)>> {
    let block = fillers.children().iter().flatten().find_map(as_record).unwrap();
    block.children().iter().map(|tile| {
        tile.iter().filter_map(as_record).flat_map(|los_block| los_block.children().iter()).map(|filler| match (&filler[0], &filler[1]) {
            (NodeType::U16(x), NodeType::U16(y)) => (*x, *y),
            _ => panic!("a filler doesn't start with its cell"),
        }).collect()
    }).collect()
}

#[test]
#[ignore = "needs the game data, see startpos_test.rs"]
fn test_shroud_fillers_match_the_game_startpos() {
    let (index, layout, settlements, region_map, legacy_overrides) = world_test_data();
    let inputs = WorldInputs { tables: test_data_tables(), index: &index, layout: &layout, settlements: &settlements, region_map: &region_map, legacy_overrides: &legacy_overrides };
    let plan = game_plan(&index);
    let sight = SightPlan::new(&inputs, &plan);

    // Fillers are written from the game's own vision of each faction, so only the filler rules are compared.
    let mut compared = 0;
    for faction in world_factions(world(test_startpos())) {
        let Some(shroud) = child_record(faction, "CAMPAIGN_SHROUD") else { continue };
        let theirs = filler_cells(child_record(shroud, "CAMPAIGN_SHROUD_FILLERS").unwrap());
        if theirs.is_empty() {
            continue;
        }
        let layer = shroud.children()[0].iter().filter_map(as_record).filter(|record| record.name() == "QUAD_TREE_BIT_ARRAY").nth(1).unwrap();
        let Some(NodeType::U8Array(bytes)) = layer.children()[0].get(3) else { panic!("the shroud layer has no quad tree") };
        let vision = decode(bytes, sight.grid());

        let ours = filler_cells(&write_fillers(&layout, SHROUD_PATH, &sight, &vision).unwrap());
        assert_eq!(ours, theirs, "fillers of {}", field_str(faction, 1));
        compared += 1;
    }
    assert!(compared > 0, "the game startpos has no shroud fillers");
}

/// Returns the `LINE_OF_SIGHT` record of each filler of a `CAMPAIGN_SHROUD_FILLERS` record, by settlement cell.
fn filler_sights(fillers: &RecordNode) -> HashMap<(u16, u16), RecordNode> {
    let block = fillers.children().iter().flatten().find_map(as_record).unwrap();
    block.children().iter()
        .flat_map(|tile| tile.iter().filter_map(as_record))
        .flat_map(|los_block| los_block.children().iter())
        .filter_map(|filler| match (&filler[0], &filler[1], filler.iter().find_map(as_record)) {
            (NodeType::U16(x), NodeType::U16(y), Some(sight)) => Some(((*x, *y), sight.clone())),
            _ => None,
        })
        .collect()
}

#[test]
#[ignore = "needs the game data, see startpos_test.rs"]
fn test_shroud_filler_vision_matches_the_game_startpos() {
    let (index, layout, settlements, region_map, legacy_overrides) = world_test_data();
    let inputs = WorldInputs { tables: test_data_tables(), index: &index, layout: &layout, settlements: &settlements, region_map: &region_map, legacy_overrides: &legacy_overrides };
    let plan = game_plan(&index);
    let sight = SightPlan::new(&inputs, &plan);

    // Without vision, a faction gets a filler for every settlement.
    let ours = filler_sights(&write_fillers(&layout, SHROUD_PATH, &sight, &Vision::default()).unwrap());
    let mut theirs = HashMap::new();
    for faction in world_factions(world(test_startpos())) {
        if let Some(fillers) = child_record(faction, "CAMPAIGN_SHROUD").and_then(|shroud| child_record(shroud, "CAMPAIGN_SHROUD_FILLERS")) {
            theirs.extend(filler_sights(fillers));
        }
    }

    let tree = |sight: &RecordNode| sight.children()[0].iter().find_map(as_record).cloned();
    let leaves = |sight: &RecordNode| sight.children()[0].iter().filter(|node| as_record(node).is_none()).cloned().collect::<Vec<_>>();
    let areas = theirs.iter().filter(|(cell, sight)| ours.get(cell).map(tree) == Some(tree(sight))).count();
    let bounds = theirs.iter().filter(|(cell, sight)| ours.get(cell).map(leaves) == Some(leaves(sight))).count();
    assert!(areas >= MIN_MATCHING_FILLER_SIGHTS, "{areas}/{} filler areas match ({bounds} bounds)", theirs.len());
}

#[test]
#[ignore = "needs the game data, see startpos_test.rs"]
fn test_lines_of_sight_without_vision_use_the_map_grid() {
    let layout = StartposLayout::from_esf(test_startpos()).unwrap();
    let grid = VisionGrid::new((100.0, 50.0));

    let record = write_line_of_sight(&layout, "CAMPAIGN_STARTPOS/CAMPAIGN_ENV/CAMPAIGN_MODEL/WORLD/FACTION_ARRAY/FACTION/CHARACTER_ARRAY/CHARACTER", &grid, None).unwrap();

    let quad_tree = child_record(&record, "QUAD_TREE_BIT_ARRAY").unwrap();
    let dimensions = quad_tree.children()[0][..3].iter().map(|leaf| match leaf {
        NodeType::U32(node) => *node.value(),
        _ => panic!("a quad tree dimension isn't a u32"),
    }).collect::<Vec<_>>();
    assert_eq!(dimensions, vec![200, 100, 256]);
    assert_eq!(quad_tree.children()[0][3], NodeType::U8Array(vec![0; 9]));
}
