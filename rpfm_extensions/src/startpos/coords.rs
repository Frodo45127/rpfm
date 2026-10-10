//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Coordinate systems of the campaign map.
//!
//! The campaign map uses three related coordinate systems:
//!
//! - **World**: the `f32` positions stored in startpos `Coord2d` nodes.
//! - **Raw**: the `u16` units stored in the map files (`trade_routes.ptd`). `raw = world * scale + 4`.
//! - **Cell**: the logical hex grid (1016x720 cells in Attila, the size of `pathfinding.ppd`). A cell is
//!   6 raw units wide and 8 raw units high, and odd columns are shifted half a cell down.
//!
//! The scales were fitted from the game data: characters placed on cells land exactly where the game puts them
//! (518 of 519 in Attila's main campaign), and trade spline points within 7.5e-5 world units. Their ratio is
//! `2 / sqrt(3)`, as expected from a hex grid.

use getset::CopyGetters;
use serde_derive::{Deserialize, Serialize};

#[cfg(test)] #[path = "coords_test.rs"] mod coords_test;

/// Raw units per world unit, horizontally.
pub const RAW_SCALE_X: f64 = 8.975_681_124;

/// Raw units per world unit, vertically.
pub const RAW_SCALE_Y: f64 = 10.364_224_094;

/// Offset added to raw coordinates.
pub const RAW_OFFSET: f32 = 4.0;

/// Raw units per cell, horizontally.
pub const CELL_RAW_WIDTH: f32 = 6.0;

/// Raw units per cell, vertically.
pub const CELL_RAW_HEIGHT: f32 = 8.0;

/// Neighbour offsets of a cell in an odd column, in the order the game uses (south first, then clockwise).
const ODD_COLUMN_NEIGHBOURS: [(i32, i32); 6] = [(0, 1), (1, 1), (1, 0), (0, -1), (-1, 0), (-1, 1)];

/// Neighbour offsets of a cell in an even column, in the order the game uses (south first, then clockwise).
const EVEN_COLUMN_NEIGHBOURS: [(i32, i32); 6] = [(0, 1), (1, 0), (1, -1), (0, -1), (-1, -1), (-1, 0)];

/// Edges of a hex cell.
pub const HEX_EDGES: u8 = 6;

/// Edge mask with every edge set.
const ALL_EDGES: u8 = 0b11_1111;

/// Edge mask of the first two edges.
const FIRST_TWO_EDGES: u8 = 0b11;

//---------------------------------------------------------------------------//
//                              Types
//---------------------------------------------------------------------------//

/// A cell of the logical hex grid of the campaign map.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default, CopyGetters, Serialize, Deserialize)]
#[getset(get_copy = "pub")]
pub struct HexCell {

    /// Column.
    x: u16,

    /// Row.
    y: u16,
}

/// A piece covering some edges of a hex cell (a road or a coast), as a shape and a rotation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, CopyGetters)]
#[getset(get_copy = "pub")]
pub struct EdgePiece {

    /// Edge mask of the piece before rotating it.
    shape: u8,

    /// Edges the shape is rotated by, in the game's direction order.
    rotation: u8,
}

//---------------------------------------------------------------------------//
//                              Implementations
//---------------------------------------------------------------------------//

impl HexCell {

    /// Creates a cell from its column and row.
    pub fn new(x: u16, y: u16) -> Self {
        Self { x, y }
    }

    /// Returns the cell containing a world position.
    ///
    /// Odd columns are shifted half a cell down, so their rows start half a cell height later.
    pub fn from_world(x: f32, y: f32) -> Self {
        let (raw_x, raw_y) = world_to_raw(x, y);
        let column = (raw_x / CELL_RAW_WIDTH).floor() as u16;
        let row_offset = if column % 2 == 1 { CELL_RAW_HEIGHT / 2.0 } else { 0.0 };
        Self::new(column, ((raw_y - row_offset) / CELL_RAW_HEIGHT).floor() as u16)
    }

    /// Returns the existing neighbours of the cell, in the game's direction order.
    ///
    /// # Returns
    ///
    /// Up to six cells; neighbours that would fall outside the `u16` range are skipped.
    pub fn neighbours(&self) -> impl Iterator<Item = HexCell> + '_ {
        (0..HEX_EDGES).filter_map(|direction| self.neighbour(direction))
    }

    /// Returns the neighbour in a direction (0 to 5, in the game's direction order).
    ///
    /// # Returns
    ///
    /// `None` if the neighbour would fall outside the `u16` range, or the direction is over 5.
    pub fn neighbour(&self, direction: u8) -> Option<HexCell> {
        let offsets = if self.x % 2 == 1 { &ODD_COLUMN_NEIGHBOURS } else { &EVEN_COLUMN_NEIGHBOURS };
        let (dx, dy) = offsets.get(usize::from(direction))?;
        let x = u16::try_from(i32::from(self.x) + dx).ok()?;
        let y = u16::try_from(i32::from(self.y) + dy).ok()?;
        Some(HexCell::new(x, y))
    }

    /// Returns the edge mask of the neighbours matching a predicate: bit `n` stands for the `n`-th neighbour in the
    /// game's direction order. Neighbours outside the `u16` range never match.
    pub fn neighbour_mask(&self, mut predicate: impl FnMut(HexCell) -> bool) -> u8 {
        (0..HEX_EDGES)
            .filter(|&direction| self.neighbour(direction).is_some_and(&mut predicate))
            .fold(0, |mask, direction| mask | 1 << direction)
    }

    /// Returns the cell followed by its neighbours, in the order the game stores slot footprints.
    pub fn footprint(&self) -> Vec<HexCell> {
        std::iter::once(*self).chain(self.neighbours()).collect()
    }

    /// Returns the world position of the cell's top-left corner, where the game places characters given by cell.
    ///
    /// The game converts in double precision and adds the half-cell shift of odd columns as a separate single
    /// precision term, so the position is computed the same way to get the same floats.
    pub fn world_position(&self) -> (f32, f32) {
        let x = (f64::from(self.x) * f64::from(CELL_RAW_WIDTH) / RAW_SCALE_X) as f32;
        let y = (f64::from(self.y) * f64::from(CELL_RAW_HEIGHT) / RAW_SCALE_Y) as f32;
        let row_offset = if self.x % 2 == 1 { (f64::from(CELL_RAW_HEIGHT / 2.0) / RAW_SCALE_Y) as f32 } else { 0.0 };
        (x, y + row_offset)
    }
}

impl EdgePiece {

    /// Splits an edge mask into a piece shape and a rotation.
    ///
    /// The shape is the smallest rotation of the mask with its first two edges set, or the smallest rotation if
    /// none has them (so a straight road is `0b1001` and a bend is `0b11`). The rotation is the smallest one turning
    /// the shape into the mask.
    pub fn from_mask(mask: u8) -> Self {
        let mask = mask & ALL_EDGES;
        let rotations = (0..HEX_EDGES).map(|rotation| rotate_edges(mask, rotation));
        let shape = rotations.clone().filter(|rotated| rotated & FIRST_TWO_EDGES == FIRST_TWO_EDGES).min()
            .or_else(|| rotations.min())
            .unwrap_or_default();
        let rotation = (0..HEX_EDGES).find(|&rotation| rotate_edges(shape, rotation) == mask).unwrap_or_default();
        Self { shape, rotation }
    }
}

/// Rotates an edge mask by some edges, in the game's direction order.
fn rotate_edges(mask: u8, edges: u8) -> u8 {
    ((mask << edges) | (mask >> ((HEX_EDGES - edges) % HEX_EDGES))) & ALL_EDGES
}

/// Converts a world position to raw map units.
pub fn world_to_raw(x: f32, y: f32) -> (f32, f32) {
    (x * RAW_SCALE_X as f32 + RAW_OFFSET, y * RAW_SCALE_Y as f32 + RAW_OFFSET)
}

/// Converts raw map units to a world position.
pub fn raw_to_world(x: f32, y: f32) -> (f32, f32) {
    ((x - RAW_OFFSET) / RAW_SCALE_X as f32, (y - RAW_OFFSET) / RAW_SCALE_Y as f32)
}

/// Converts a raw offset (a vector, no translation) to world units.
///
/// Map files store offsets at 100 times the raw scale.
pub fn raw_offset_to_world(x: i16, y: i16) -> (f32, f32) {
    (f32::from(x) / (RAW_SCALE_X as f32 * 100.0), f32::from(y) / (RAW_SCALE_Y as f32 * 100.0))
}
