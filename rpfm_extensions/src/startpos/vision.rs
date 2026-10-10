//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Vision areas of a startpos (`LINE_OF_SIGHT` and `CAMPAIGN_SHROUD` records) and their quad tree encoding.
//!
//! Vision lives on a square grid of half world units over the campaign map (1357x1112 cells in Attila; the height
//! is stretched to fit the map exactly). The startpos stores an area as a quad tree over a power-of-two square that
//! covers the grid, with its rows counted from the top:
//!
//! - A node with children is a byte with the code of each child in two bits, the first child in the lowest bits
//!   (1: mixed, 2: empty, 3: full), followed by its mixed children. Children are the top-left, top-right,
//!   bottom-left and bottom-right quarters.
//! - A node without children (the 8x8 cells at the bottom of the tree) is a 0 byte followed by 8 bytes, one per row
//!   from the bottom, with the leftmost cell in the lowest bit. An empty area is a single such node.

use std::collections::HashSet;

#[cfg(test)] #[path = "vision_test.rs"] mod vision_test;

/// World units per vision cell, horizontally.
const CELL_SIZE: f32 = 0.5;

/// Cells per side of a quad tree leaf.
const LEAF_SIZE: u32 = 8;

/// Child codes of a quad tree node.
const MIXED: u8 = 1;
const EMPTY: u8 = 2;
const FULL: u8 = 3;

//---------------------------------------------------------------------------//
//                              Types
//---------------------------------------------------------------------------//

/// The vision grid of a campaign map.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VisionGrid {

    /// Columns.
    width: u32,

    /// Rows.
    height: u32,

    /// World units per cell, horizontally and vertically.
    cell_size: (f32, f32),

    /// Size of the map in world units.
    world_size: (f32, f32),

    /// Side of the quad tree square.
    tree_size: u32,
}

/// An area of the vision grid: a set of cells (column, row from the bottom).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Vision {
    cells: HashSet<(u32, u32)>,
}

//---------------------------------------------------------------------------//
//                              Implementations
//---------------------------------------------------------------------------//

impl VisionGrid {

    /// Creates the vision grid of a map of the given size in world units.
    pub fn new(world_size: (f32, f32)) -> Self {
        let width = (world_size.0 / CELL_SIZE).floor().max(1.0) as u32;
        let height = (world_size.1 / CELL_SIZE).floor().max(1.0) as u32;
        Self {
            width,
            height,
            cell_size: (world_size.0 / width as f32, world_size.1 / height as f32),
            world_size,
            tree_size: width.max(height).max(LEAF_SIZE).next_power_of_two(),
        }
    }

    /// Returns the columns, rows and quad tree side, as the `QUAD_TREE_BIT_ARRAY` records store them.
    pub fn dimensions(&self) -> (u32, u32, u32) {
        (self.width, self.height, self.tree_size)
    }

    /// Returns the cell containing a world position.
    pub fn cell_at(&self, position: (f32, f32)) -> (u32, u32) {
        ((position.0 / self.cell_size.0).max(0.0) as u32, (position.1 / self.cell_size.1).max(0.0) as u32)
    }

    /// Returns the world position of a cell's corner closest to the origin: the cell times the map size, over the
    /// cells of the grid. This gives the game's floats horizontally, and vertically in 3 of 4 cases.
    pub fn cell_position(&self, cell: (u32, u32)) -> (f32, f32) {
        ((cell.0 as f32 * self.world_size.0) / self.width as f32, (cell.1 as f32 * self.world_size.1) / self.height as f32)
    }

    /// Returns what a character sees: the cells within its sight radius around its cell, counting the radius in
    /// horizontal cells. A cell `(dx, dy)` away is in if `dx² + dy² <= r * (r + 1)`.
    pub fn disk(&self, position: (f32, f32), radius: f32) -> Vision {
        let (cx, cy) = self.cell_at(position);
        let r = (radius / self.cell_size.0).round() as i64;
        let limit = r * (r + 1);
        let mut cells = HashSet::new();
        for dx in -r..=r {
            for dy in -r..=r {
                let (x, y) = (i64::from(cx) + dx, i64::from(cy) + dy);
                if dx * dx + dy * dy <= limit && self.contains(x, y) {
                    cells.insert((x as u32, y as u32));
                }
            }
        }
        Vision { cells }
    }

    /// Returns the cells whose centre is within a radius of any of the given world positions.
    pub fn around(&self, positions: &[(f32, f32)], radius: f32) -> Vision {
        let mut cells = HashSet::new();
        for &(px, py) in positions {
            let (x0, y0) = self.cell_at((px - radius, py - radius));
            let (x1, y1) = self.cell_at((px + radius, py + radius));
            for x in x0..=x1 {
                for y in y0..=y1 {
                    let (cx, cy) = ((x as f32 + 0.5) * self.cell_size.0, (y as f32 + 0.5) * self.cell_size.1);
                    if (cx - px).powi(2) + (cy - py).powi(2) <= radius * radius && self.contains(i64::from(x), i64::from(y)) {
                        cells.insert((x, y));
                    }
                }
            }
        }
        Vision { cells }
    }

    /// Returns if a cell is inside the grid.
    fn contains(&self, x: i64, y: i64) -> bool {
        (0..i64::from(self.width)).contains(&x) && (0..i64::from(self.height)).contains(&y)
    }
}

impl Vision {

    /// Returns if the area has no cells.
    pub fn is_empty(&self) -> bool {
        self.cells.is_empty()
    }

    /// Adds the cells of another area to this one.
    pub fn extend(&mut self, other: &Vision) {
        self.cells.extend(other.cells.iter().copied());
    }

    /// Returns the world bounds of the area: the corner of its first cell and the far corner of its last one.
    pub fn bounds(&self, grid: &VisionGrid) -> Option<((f32, f32), (f32, f32))> {
        let min_x = self.cells.iter().map(|cell| cell.0).min()?;
        let min_y = self.cells.iter().map(|cell| cell.1).min()?;
        let max_x = self.cells.iter().map(|cell| cell.0).max()?;
        let max_y = self.cells.iter().map(|cell| cell.1).max()?;
        Some((grid.cell_position((min_x, min_y)), grid.cell_position((max_x + 1, max_y + 1))))
    }

    /// Encodes the area as the quad tree of a `QUAD_TREE_BIT_ARRAY` record (see the module docs).
    pub fn encode(&self, grid: &VisionGrid) -> Vec<u8> {
        let mut bytes = vec![];
        let cells = self.cells.iter().map(|&(x, y)| (x, grid.tree_size - 1 - y)).collect::<Vec<_>>();
        if cells.is_empty() {
            bytes.resize(1 + LEAF_SIZE as usize, 0);
        } else {
            encode_node((0, 0, grid.tree_size), cells, &mut bytes);
        }
        bytes
    }
}

/// Encodes a mixed node of the quad tree: (column, row from the top, side), with its cells (also counted from the
/// top). Each node splits its cells among its children, so the whole tree takes one pass per level.
fn encode_node((x, y, size): (u32, u32, u32), cells: Vec<(u32, u32)>, bytes: &mut Vec<u8>) {
    if size == LEAF_SIZE {
        let mut rows = [0u8; LEAF_SIZE as usize];
        for (cell_x, cell_y) in cells {
            rows[(cell_y - y) as usize] |= 1 << (cell_x - x);
        }
        bytes.push(0);
        bytes.extend(rows.iter().rev());
        return;
    }

    let half = size / 2;
    let mut children = [(x, y, vec![]), (x + half, y, vec![]), (x, y + half, vec![]), (x + half, y + half, vec![])];
    for cell in cells {
        let index = usize::from(cell.0 >= x + half) + 2 * usize::from(cell.1 >= y + half);
        children[index].2.push(cell);
    }

    let code = |count: usize| match count {
        0 => EMPTY,
        count if count == (half * half) as usize => FULL,
        _ => MIXED,
    };
    bytes.push(children.iter().enumerate().fold(0, |byte, (index, child)| byte | code(child.2.len()) << (2 * index)));
    for (child_x, child_y, child_cells) in children {
        if code(child_cells.len()) == MIXED {
            encode_node((child_x, child_y, half), child_cells, bytes);
        }
    }
}
