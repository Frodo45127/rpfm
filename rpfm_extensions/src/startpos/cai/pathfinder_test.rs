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
use crate::startpos::esf_nav::{as_record, as_u32, child_record};
use crate::startpos::startpos_test::{test_data_esf, test_startpos};
use crate::startpos::writer::u32_node;

/// Returns a pathfinder record with the given leaves.
fn record(values: &[NodeType]) -> RecordNode {
    let mut record = RecordNode::default();
    record.set_children(vec![values.to_vec()]);
    record
}

/// Returns the leaves of a node with portals (segment, cost, target id, slot) and its crossing costs.
fn node(id: u16, portals: &[([u16; 4], u32, u16, u8)], crossings: &[u32]) -> Vec<NodeType> {
    let mut leaves = vec![NodeType::U16(id), NodeType::U16(0), NodeType::U16(0), u32_node(0), u32_node(portals.len() as u32)];
    for (segment, cost, target, slot) in portals {
        leaves.extend(segment.iter().map(|value| NodeType::U16(*value)));
        leaves.extend([u32_node(*cost), NodeType::U16(*target), NodeType::U8(*slot), NodeType::Bool(Default::default()), NodeType::Bool(Default::default())]);
    }
    leaves.extend(crossings.iter().map(|cost| u32_node(*cost)));
    leaves
}

#[test]
fn test_region_distances_add_portals_and_crossings() {
    // Regions 0 - 1 - 2 in a row, one node each. Crossing region 1 from its first portal to its second costs 1000.
    let mut leaves = vec![u32_node(3)];
    for (region, nodes) in [
        (0, node(0, &[([1, 0, 1, 1], 80, 1, 0)], &[])),
        (1, node(1, &[([1, 1, 1, 0], 80, 0, 0), ([2, 0, 2, 1], 112, 2, 1)], &[1000, 1000])),
        (2, node(2, &[([2, 1, 2, 0], 112, 1, 0)], &[])),
    ] {
        leaves.extend([NodeType::U16(region), u32_node(1)]);
        leaves.extend(nodes);
    }

    let pathfinder = Pathfinder::read(&record(&leaves)).unwrap();

    assert_eq!(pathfinder.region_costs(0), HashMap::from([(1, 80), (2, 80 + 1000 + 112)]));
    assert_eq!(pathfinder.max_region_distance(), 1192);
}

#[test]
#[ignore = "needs the game data, see startpos_test.rs"]
fn test_max_region_distance_matches_the_game_startpos() {
    let hlp_data = test_data_esf("hlp_data.esf");
    let pathfinder = child_record(as_record(hlp_data.root_node()).unwrap(), "CAI_HIGH_LEVEL_PATHFINDER").unwrap();

    let cai = ["CAMPAIGN_ENV", "CAMPAIGN_MODEL", "CAI_INTERFACE"].iter()
        .try_fold(as_record(test_startpos().root_node()).unwrap(), |record, name| child_record(record, name))
        .unwrap();
    let game = child_record(cai, "CAI_INTERFACE_MAX_REGION_DISTANCE").and_then(|record| record.children()[0].first().and_then(as_u32)).unwrap();

    assert_eq!(Pathfinder::read(pathfinder).unwrap().max_region_distance(), game);
}
