//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

use rpfm_lib::files::esf::NodeType;

use std::collections::HashMap;

use super::*;
use crate::startpos::esf_nav::child_record;
use crate::startpos::startpos_test::{array_items, field, field_str, field_u32, test_data_esf, test_startpos, world, world_factions, world_regions};

#[test]
fn test_region_at_reads_the_first_byte_of_each_cell() {
    let mut cells = vec![0; 3 * 2 * BYTES_PER_CELL];
    cells[(3 + 2) * BYTES_PER_CELL] = 1;
    cells[(3 + 2) * BYTES_PER_CELL + 1] = 5;
    let map = RegionMap { keys: vec!["land".to_owned(), "sea".to_owned()], width: 3, height: 2, cells };

    assert_eq!(map.region_at(HexCell::new(2, 1)), Some("sea"));
    assert_eq!(map.region_at(HexCell::new(0, 0)), Some("land"));
    assert_eq!(map.region_at(HexCell::new(3, 0)), None);
    assert_eq!(map.region_at(HexCell::new(0, 2)), None);
}

#[test]
fn test_edge_masks_read_the_last_two_bytes_of_each_cell() {
    let mut cells = vec![0; 2 * BYTES_PER_CELL];
    cells[BYTES_PER_CELL + 5] = 9;
    cells[BYTES_PER_CELL + 6] = 36;
    let map = RegionMap { keys: vec!["land".to_owned()], width: 2, height: 1, cells };

    assert_eq!(map.edge_masks(HexCell::new(1, 0)), [9, 36]);
    assert_eq!(map.edge_masks(HexCell::new(0, 0)), [0, 0]);
    assert_eq!(map.edge_masks(HexCell::new(2, 0)), [0, 0]);
}

#[test]
#[ignore = "needs the game data, see startpos_test.rs"]
fn test_garrisons_stand_in_their_region() {
    let map = RegionMap::from_map_data(&test_data_esf("map_data.esf")).unwrap();
    let world = world(test_startpos());

    // Garrison armies are led by colonels standing on their settlement.
    let mut positions = HashMap::new();
    for faction in world_factions(world) {
        for character in child_record(faction, "CHARACTER_ARRAY").map(|array| array_items(array, "CHARACTER")).unwrap_or_default() {
            if let NodeType::Coord2d(position) = field(child_record(character, "LOCOMOTABLE").unwrap(), 0) {
                positions.insert(field_u32(character, 4), HexCell::from_world(*position.x(), *position.y()));
            }
        }
    }

    let mut checked = 0;
    for region in world_regions(world) {
        let residence = child_record(region, "SETTLEMENT").and_then(|settlement| child_record(settlement, "GARRISON_RESIDENCE")).unwrap();
        let force = field_u32(residence, 8);
        if let Some(cell) = positions.get(&force).filter(|_| force != 0) {
            assert_eq!(map.region_at(*cell), Some(field_str(region, 1)));
            checked += 1;
        }
    }

    assert_eq!(map.keys().len(), 224);
    assert_eq!(checked, 175);
}
