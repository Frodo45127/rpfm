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

/// Returns the scale of Attila's main campaign map.
pub(crate) fn attila_scale() -> MapScale {
    MapScale::new((1016, 720), (678.5, 556.143_9))
}

#[test]
fn test_cell_at_matches_game_cells() {
    let scale = attila_scale();

    // Character positions of a game-built startpos (`LOCOMOTABLE`), with the cells stored next to them.
    assert_eq!(scale.cell_at(201.21036, 380.1539), HexCell::new(301, 492));
    assert_eq!(scale.cell_at(60.83104, 451.16742), HexCell::new(91, 584));
    assert_eq!(scale.cell_at(48.13005, 460.0441), HexCell::new(72, 596));
}

#[test]
fn test_cell_at_follows_slanted_edges() {
    let scale = attila_scale();

    // Near the left vertex of cell (3, 10), raw x 18.2: in the middle of the row it's still the cell; near its top
    // and bottom it's past the slanted edges, in the previous column's cells.
    let at = |raw_x: f32, raw_y: f32| {
        let (x, y) = scale.raw_to_world(raw_x, raw_y);
        scale.cell_at(x, y)
    };
    assert_eq!(at(18.2, 88.0), HexCell::new(3, 10));
    assert_eq!(at(18.2, 91.2), HexCell::new(2, 11));
    assert_eq!(at(18.2, 84.8), HexCell::new(2, 10));
}

#[test]
fn test_world_raw_roundtrip() {
    let scale = attila_scale();
    let (raw_x, raw_y) = scale.world_to_raw(482.63748, 41.295902);
    let (x, y) = scale.raw_to_world(raw_x, raw_y);

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

    // Characters with startx/starty in the table, placed by the game: Roxolani generals in the main campaign, and
    // Charlemagne's generals on its own map (920x800 cells; CA's build has x one ulp lower for the first one).
    let scale = attila_scale();
    assert_eq!(scale.world_position(HexCell::new(449, 543)), (300.144_35, 419.520_08));
    assert_eq!(scale.world_position(HexCell::new(564, 494)), (377.018_74, 381.311_7));

    let charlemagne = MapScale::new((920, 800), (613.8, 617.365_05));
    assert_eq!(charlemagne.world_position(HexCell::new(338, 412)), (225.750_17, 317.744_42));
    assert_eq!(charlemagne.world_position(HexCell::new(377, 370)), (251.798_26, 285.738_62));
}

#[test]
fn test_neighbour_mask_follows_the_direction_order() {
    let cell = HexCell::new(803, 181);
    let south = HexCell::new(803, 182);
    let north = HexCell::new(803, 180);

    assert_eq!(cell.neighbour_mask(|neighbour| neighbour == south || neighbour == north), 0b1001);
    assert_eq!(HexCell::new(0, 0).neighbour_mask(|_| true), 0b11);
}

#[test]
fn test_edge_pieces_match_the_game() {

    // Masks of sprawl cells of a game-built startpos (`SPRAWL_HEX_MAP_DATA`), with their shape and rotation.
    for (mask, shape, rotation) in [(0, 0, 0), (8, 1, 3), (18, 9, 1), (36, 9, 2), (13, 19, 2), (33, 3, 5), (50, 11, 4), (51, 15, 4), (53, 23, 4), (45, 27, 2)] {
        let piece = EdgePiece::from_mask(mask);
        assert_eq!((piece.shape(), piece.rotation()), (shape, rotation), "mask {mask}");
    }
}
