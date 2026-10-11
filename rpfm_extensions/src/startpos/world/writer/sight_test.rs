//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

use std::collections::BTreeSet;

use super::*;
use crate::startpos::esf_nav::{as_record, child_record};
use crate::startpos::startpos_test::{field_str, test_data_tables, test_startpos, world, world_factions};
use crate::startpos::vision::vision_test::decode;
use crate::startpos::world::writer::writer_test::{game_plan, world_test_data};

/// Path of the shroud of a faction.
const SHROUD_PATH: &str = "CAMPAIGN_STARTPOS/CAMPAIGN_ENV/CAMPAIGN_MODEL/WORLD/FACTION_ARRAY/FACTION/CAMPAIGN_SHROUD";

/// Returns the settlement cells of each tile of a `CAMPAIGN_SHROUD_FILLERS` record, in tile order.
fn filler_cells(fillers: &RecordNode) -> Vec<BTreeSet<(u16, u16)>> {
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
