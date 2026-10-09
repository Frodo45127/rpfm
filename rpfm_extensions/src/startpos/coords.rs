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
//! The scales were fitted from the game data (max error 7.5e-5 world units over every trade spline point).
//! Their ratio is `2 / sqrt(3)`, as expected from a hex grid.

use getset::CopyGetters;
use serde_derive::{Deserialize, Serialize};

#[cfg(test)] #[path = "coords_test.rs"] mod coords_test;

/// Raw units per world unit, horizontally.
pub const RAW_SCALE_X: f32 = 8.975_681;

/// Raw units per world unit, vertically.
pub const RAW_SCALE_Y: f32 = 10.364_225;

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
        let offsets = if self.x % 2 == 1 { &ODD_COLUMN_NEIGHBOURS } else { &EVEN_COLUMN_NEIGHBOURS };
        offsets.iter().filter_map(|(dx, dy)| {
            let x = u16::try_from(i32::from(self.x) + dx).ok()?;
            let y = u16::try_from(i32::from(self.y) + dy).ok()?;
            Some(HexCell::new(x, y))
        })
    }

    /// Returns the cell followed by its neighbours, in the order the game stores slot footprints.
    pub fn footprint(&self) -> Vec<HexCell> {
        std::iter::once(*self).chain(self.neighbours()).collect()
    }
}

/// Converts a world position to raw map units.
pub fn world_to_raw(x: f32, y: f32) -> (f32, f32) {
    (x * RAW_SCALE_X + RAW_OFFSET, y * RAW_SCALE_Y + RAW_OFFSET)
}

/// Converts raw map units to a world position.
pub fn raw_to_world(x: f32, y: f32) -> (f32, f32) {
    ((x - RAW_OFFSET) / RAW_SCALE_X, (y - RAW_OFFSET) / RAW_SCALE_Y)
}

/// Converts a raw offset (a vector, no translation) to world units.
///
/// Map files store offsets at 100 times the raw scale.
pub fn raw_offset_to_world(x: i16, y: i16) -> (f32, f32) {
    (f32::from(x) / (RAW_SCALE_X * 100.0), f32::from(y) / (RAW_SCALE_Y * 100.0))
}
