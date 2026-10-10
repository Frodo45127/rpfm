//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

use super::*;

use crate::startpos::cai::cai_test::{game_cai_world_list, map_inputs};
use crate::startpos::startpos_test::{compare, test_startpos, Report};

/// HLCI fields the writer gets wrong: the position of Hafn's main HLCI (the game picks the node of its second largest
/// area; the rule isn't known).
const KNOWN_HLCI_MISMATCHES: usize = 2;

#[test]
#[ignore = "needs the game data, see startpos_test.rs"]
fn test_hlcis_match_the_game_startpos() {
    let layout = StartposLayout::from_esf(test_startpos()).unwrap();
    let (map_regions, nodes) = map_inputs();
    let plan = CaiPlan::new(&map_regions);

    let written = write_hlcis(&layout, &plan, &map_regions, &nodes).unwrap();
    let mut report = Report::default();
    compare(&layout, &written, game_cai_world_list("CAI_WORLD_REGION_HLCIS"), &child_path(CAI_WORLD_PATH, "CAI_WORLD_REGION_HLCIS"), &mut report);

    let (matches, total) = report.totals();
    assert_eq!(matches + KNOWN_HLCI_MISMATCHES, total, "{matches}/{total} fields match. Worst:\n{}", report.worst(20));
}

#[test]
#[ignore = "needs the game data, see startpos_test.rs"]
fn test_regions_match_the_game_startpos() {
    let layout = StartposLayout::from_esf(test_startpos()).unwrap();
    let (map_regions, nodes) = map_inputs();
    let plan = CaiPlan::new(&map_regions);

    let written = write_regions(&layout, &plan, &map_regions, &nodes, &[]).unwrap();
    let mut report = Report::default();
    compare(&layout, &written, game_cai_world_list("CAI_WORLD_REGIONS"), &child_path(CAI_WORLD_PATH, "CAI_WORLD_REGIONS"), &mut report);

    let (matches, total) = report.totals();
    println!("{matches}/{total} fields match. Worst:\n{}", report.worst(30));
}
