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
use crate::startpos::startpos_test::test_data_esf;

/// A region with areas in groups 4, 7, 4 and 9, the biggest one (size 50) in group 7.
fn region() -> MapRegion {
    MapRegion {
        key: "reg".to_owned(),
        area_groups: vec![4, 7, 4, 9],
        area_sizes: vec![10, 50, 20, 50],
        connections: vec![],
        connects_to_other_kind: false,
    }
}

#[test]
fn test_hlcis_group_areas_in_first_seen_order() {
    assert_eq!(region().hlcis(), vec![(4, vec![0, 2]), (7, vec![1]), (9, vec![3])]);
}

#[test]
fn test_largest_area_and_its_hlci() {
    assert_eq!(region().largest_area(), Some(1));
    assert_eq!(region().hlci_of_area(3), Some(2));
}

#[test]
#[ignore = "needs the game data, see startpos_test.rs"]
fn test_pathfinder_nodes_real_data() {
    let nodes = pathfinder_nodes(&test_data_esf("hlp_data.esf")).unwrap();

    assert_eq!(nodes.len(), 224);
    assert_eq!(nodes[&0].iter().map(|node| (node.area(), node.x(), node.y())).collect::<Vec<_>>(), vec![(0, 533, 204), (3, 570, 199)]);
}
