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
use crate::startpos::cai::cai_test::game_cai_world_list;
use crate::startpos::cai::CAI_WORLD_PATH;
use crate::startpos::layout::{child_path, StartposLayout};
use crate::startpos::startpos_test::{test_data_esf, test_startpos};

/// A region with areas in groups 4, 7, 4 and 9, the biggest one (size 50) in group 7.
fn region() -> MapRegion {
    MapRegion {
        key: "reg".to_owned(),
        area_groups: vec![4, 7, 4, 9],
        area_sizes: vec![10, 50, 20, 50],
        connections: vec![],
        connects_to_other_kind: false,
        has_settlement: true,
        port: None,
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

#[test]
#[ignore = "needs the game data, see startpos_test.rs"]
fn test_area_groups_match_the_game_situated_records() {
    let map_data = test_data_esf("map_data.esf");
    let regions = MapRegion::all_from_map_data(&map_data).unwrap();
    let region_map = RegionMap::from_map_data(&map_data).unwrap();
    let groups = AreaGroups::new(&region_map, &regions);
    let layout = StartposLayout::from_esf(test_startpos()).unwrap();

    let mut total = 0;
    let mut matches = 0;
    for (list, situated) in [("CAI_WORLD_SETTLEMENTS", "CAI_SITUATED"), ("CAI_WORLD_SETTLEMENTS", "PORT"), ("CAI_WORLD_RESOURCE_MOBILES", "CAI_SITUATED")] {
        let path = child_path(CAI_WORLD_PATH, list);
        for group in layout.read(&path, game_cai_world_list(list)).unwrap() {
            let (situated_path, record) = match situated {
                "PORT" => {
                    let threatened_path = child_path(&path, "CAI_THREATENED_GARRISONABLE");
                    let threatened = layout.read(&threatened_path, group.record("CAI_THREATENED_GARRISONABLE").unwrap()).unwrap();
                    let Some(port) = threatened[0].record("CAI_SITUATED") else { continue };
                    (child_path(&threatened_path, "CAI_SITUATED"), port)
                },
                _ => (child_path(&path, situated), group.record(situated).unwrap()),
            };
            let record = layout.read(&situated_path, record).unwrap();
            let cell = HexCell::new(record[0].u32("^+1").unwrap() as u16, record[0].u32("^+2").unwrap() as u16);
            total += 1;
            matches += usize::from(groups.group_at(cell).unwrap_or_default() == record[0].u32("^+5").unwrap());
        }
    }
    assert_eq!((matches, total), (597, 597));
}
