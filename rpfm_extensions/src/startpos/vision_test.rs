//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

use rpfm_lib::files::esf::{NodeType, RecordNode};

use super::*;
use crate::startpos::esf_nav::child_record;
use crate::startpos::startpos_test::{field, test_startpos};

/// The vision grid of Attila's main campaign.
fn attila_grid() -> VisionGrid {
    VisionGrid::new((678.5, 556.143_9))
}

/// Decodes a quad tree (see the module docs) into an area.
pub(crate) fn decode(bytes: &[u8], grid: &VisionGrid) -> Vision {
    fn node(bytes: &[u8], position: &mut usize, (x, y, size): (u32, u32, u32), cells: &mut Vec<(u32, u32)>) {
        let byte = bytes[*position];
        *position += 1;
        if byte == 0 {
            for (index, row) in bytes[*position..*position + 8].iter().enumerate() {
                for column in 0..8 {
                    if row >> column & 1 == 1 {
                        cells.push((x + column, y + 7 - index as u32));
                    }
                }
            }
            *position += 8;
            return;
        }
        let half = size / 2;
        for (index, (child_x, child_y)) in [(x, y), (x + half, y), (x, y + half), (x + half, y + half)].into_iter().enumerate() {
            match byte >> (2 * index) & 3 {
                MIXED => node(bytes, position, (child_x, child_y, half), cells),
                FULL => cells.extend((0..half).flat_map(|dx| (0..half).map(move |dy| (child_x + dx, child_y + dy)))),
                _ => {},
            }
        }
    }

    let mut cells = vec![];
    let mut position = 0;
    node(bytes, &mut position, (0, 0, grid.tree_size), &mut cells);
    assert_eq!(position, bytes.len(), "trailing bytes in the quad tree");
    Vision { cells: cells.into_iter().map(|(x, y)| (x, grid.tree_size - 1 - y)).collect() }
}

#[test]
fn test_grid_of_attila() {
    assert_eq!(attila_grid().dimensions(), (1357, 1112, 2048));
}

#[test]
fn test_empty_area_is_an_empty_leaf() {
    assert_eq!(Vision::default().encode(&attila_grid()), vec![0; 9]);
}

#[test]
fn test_disk_shape() {
    let grid = attila_grid();
    let disk = grid.disk((50.0, 50.0), 1.0);

    // Radius 2 cells: rows of 3, 5, 5, 5, 3 cells.
    assert_eq!(disk.cells.len(), 21);
    assert_eq!(decode(&disk.encode(&grid), &grid), disk);
}

#[test]
fn test_bounds() {
    let grid = VisionGrid::new((100.0, 100.0));
    let vision = Vision { cells: [(2, 3), (4, 5)].into_iter().collect() };

    assert_eq!(vision.bounds(&grid), Some(((1.0, 1.5), (2.5, 3.0))));
    assert_eq!(Vision::default().bounds(&grid), None);
}

/// Returns every `QUAD_TREE_BIT_ARRAY` byte array under a record.
fn quad_trees(record: &RecordNode, out: &mut Vec<Vec<u8>>) {
    for node in record.children().iter().flatten() {
        if let NodeType::Record(child) = node {
            if child.name() == "QUAD_TREE_BIT_ARRAY" {
                if let NodeType::U8Array(bytes) = field(child, 3) {
                    out.push(bytes.to_owned());
                }
            } else {
                quad_trees(child, out);
            }
        }
    }
}

#[test]
#[ignore = "needs the game data, see startpos_test.rs"]
fn test_encoding_matches_the_game_startpos() {
    let grid = attila_grid();
    let NodeType::Record(root) = test_startpos().root_node() else { panic!("the root isn't a record") };
    let world = ["CAMPAIGN_ENV", "CAMPAIGN_MODEL", "WORLD"].iter().fold(&**root, |record, name| child_record(record, name).unwrap());

    let mut trees = vec![];
    quad_trees(world, &mut trees);
    let mismatches = trees.iter().filter(|bytes| decode(bytes, &grid).encode(&grid) != **bytes).count();

    assert!(trees.len() > 1000);
    assert_eq!(mismatches, 0, "{mismatches} of {} quad trees re-encode differently", trees.len());
}
