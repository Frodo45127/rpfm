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

#[test]
fn test_from_world_matches_game_cells() {

    // Character positions of a game-built startpos (`LOCOMOTABLE`), with the cells stored next to them.
    assert_eq!(HexCell::from_world(201.21036, 380.1539), HexCell::new(301, 492));
    assert_eq!(HexCell::from_world(60.83104, 451.16742), HexCell::new(91, 584));
    assert_eq!(HexCell::from_world(48.13005, 460.0441), HexCell::new(72, 596));
}

#[test]
fn test_world_raw_roundtrip() {
    let (raw_x, raw_y) = world_to_raw(482.63748, 41.295902);
    let (x, y) = raw_to_world(raw_x, raw_y);

    assert!((x - 482.63748).abs() < 1e-3);
    assert!((y - 41.295902).abs() < 1e-3);
}

#[test]
fn test_footprint_order_odd_column() {
    let footprint = HexCell::new(533, 204).footprint();

    assert_eq!(footprint, vec![
        HexCell::new(533, 204),
        HexCell::new(533, 205),
        HexCell::new(534, 205),
        HexCell::new(534, 204),
        HexCell::new(533, 203),
        HexCell::new(532, 204),
        HexCell::new(532, 205),
    ]);
}

#[test]
fn test_footprint_order_even_column() {
    let footprint = HexCell::new(530, 204).footprint();

    assert_eq!(footprint, vec![
        HexCell::new(530, 204),
        HexCell::new(530, 205),
        HexCell::new(531, 204),
        HexCell::new(531, 203),
        HexCell::new(530, 203),
        HexCell::new(529, 203),
        HexCell::new(529, 204),
    ]);
}

#[test]
fn test_neighbours_skip_out_of_range() {
    let neighbours = HexCell::new(0, 0).neighbours().collect::<Vec<_>>();

    assert_eq!(neighbours, vec![HexCell::new(0, 1), HexCell::new(1, 0)]);
}

#[test]
fn test_world_position_matches_game_placements() {

    // Characters with startx/starty in the table, placed by the game (Attila, Roxolani generals).
    let (odd_x, odd_y) = HexCell::new(449, 543).world_position();
    let (even_x, even_y) = HexCell::new(564, 494).world_position();

    assert!((odd_x - 300.144_35).abs() < 1e-3 && (odd_y - 419.520_08).abs() < 1e-3);
    assert!((even_x - 377.018_74).abs() < 1e-3 && (even_y - 381.311_7).abs() < 1e-3);
}
