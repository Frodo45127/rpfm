//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Simulated layout of the blocks of a formation.
//!
//! The real size of a block depends on the units the game assigns to it, so this module simulates
//! a deployment with a configurable number of units per block. Distances are in meters, and +Y is
//! the front of the formation.
//!
//! NOTE: how the game applies relative offsets hasn't been confirmed in-game. This module assumes an
//! offset is a gap measured from the parent's edge, in the offset's direction, and that a zero offset
//! on an axis centers the block on its parent. If it turns out offsets are center-to-center, only
//! [`place_relative`] and [`offset_for_center`] need to change.

use getset::*;

use std::collections::{HashMap, HashSet};

use super::{Block, EntityArrangement, GroupFormation, GroupFormationBlock};

//---------------------------------------------------------------------------//
//                              Enum & Structs
//---------------------------------------------------------------------------//

/// Area covered by a block in the simulated layout.
#[derive(Clone, Copy, Debug, Default, PartialEq, CopyGetters)]
#[getset(get_copy = "pub")]
pub struct LayoutRect {

    /// X coordinate of the center of the block.
    center_x: f32,

    /// Y coordinate of the center of the block.
    center_y: f32,

    /// Size of the block across the formation's front.
    width: f32,

    /// Size of the block along the formation's depth.
    height: f32,
}

/// Parameters of the simulated deployment used to compute a layout.
#[derive(Clone, Debug, PartialEq, Getters, MutGetters, Setters)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct LayoutParams {

    /// Frontage of a single unit.
    unit_width: f32,

    /// Depth of a single unit.
    unit_depth: f32,

    /// Units assumed on each block without an explicit count, before applying the block's thresholds.
    default_unit_count: u32,

    /// Explicit unit counts for specific blocks, by block id.
    unit_counts: HashMap<u32, u32>,
}

/// Places blocks recursively, caching the results.
struct LayoutBuilder<'a> {
    blocks: HashMap<u32, &'a GroupFormationBlock>,
    params: &'a LayoutParams,
    rects: HashMap<u32, LayoutRect>,
    visiting: HashSet<u32>,
    unplaceable: HashSet<u32>,
}

//---------------------------------------------------------------------------//
//                             Implementations
//---------------------------------------------------------------------------//

impl Default for LayoutParams {
    fn default() -> Self {
        Self {
            unit_width: 20.0,
            unit_depth: 8.0,
            default_unit_count: 2,
            unit_counts: HashMap::new(),
        }
    }
}

impl LayoutRect {

    /// Creates a rect from its center and size.
    pub fn new(center_x: f32, center_y: f32, width: f32, height: f32) -> Self {
        Self { center_x, center_y, width, height }
    }

    /// X coordinate of the left edge.
    pub fn left(&self) -> f32 {
        self.center_x - self.width / 2.0
    }

    /// X coordinate of the right edge.
    pub fn right(&self) -> f32 {
        self.center_x + self.width / 2.0
    }

    /// Y coordinate of the front edge.
    pub fn front(&self) -> f32 {
        self.center_y + self.height / 2.0
    }

    /// Y coordinate of the back edge.
    pub fn back(&self) -> f32 {
        self.center_y - self.height / 2.0
    }

    /// Returns the smallest rect containing both rects.
    pub fn union(&self, other: &Self) -> Self {
        let left = self.left().min(other.left());
        let right = self.right().max(other.right());
        let back = self.back().min(other.back());
        let front = self.front().max(other.front());
        Self::new((left + right) / 2.0, (back + front) / 2.0, right - left, front - back)
    }
}

impl GroupFormation {

    /// Computes the simulated position and size of every block of the formation.
    ///
    /// # Arguments
    ///
    /// * `params` - The simulated deployment to use.
    ///
    /// # Returns
    ///
    /// The rect of each block, by block id. Blocks that can't be placed due to missing references,
    /// reference cycles or empty spans are left out.
    pub fn layout(&self, params: &LayoutParams) -> HashMap<u32, LayoutRect> {
        let mut blocks = HashMap::with_capacity(self.group_formation_blocks.len());
        for block in &self.group_formation_blocks {

            // On duplicated ids, references find the first block, as in validation.
            blocks.entry(block.block_id).or_insert(block);
        }

        let mut builder = LayoutBuilder {
            blocks,
            params,
            rects: HashMap::with_capacity(self.group_formation_blocks.len()),
            visiting: HashSet::new(),
            unplaceable: HashSet::new(),
        };

        for block in &self.group_formation_blocks {
            builder.place(block.block_id);
        }

        builder.rects
    }
}

impl LayoutBuilder<'_> {

    /// Returns the rect of a block, placing it and its dependencies if they weren't placed yet.
    fn place(&mut self, block_id: u32) -> Option<LayoutRect> {
        if let Some(rect) = self.rects.get(&block_id) {
            return Some(*rect);
        }

        // A block being visited again means a reference cycle.
        if self.unplaceable.contains(&block_id) || !self.visiting.insert(block_id) {
            return None;
        }

        let block = self.blocks.get(&block_id).copied();
        let rect = block.and_then(|block| match &block.block {
            Block::ContainerAbsolute(container) => {
                let (width, height) = self.footprint(block)?;
                Some(LayoutRect::new(*container.position_x(), *container.position_y(), width, height))
            },
            Block::ContainerRelative(container) => {
                let parent = self.place(*container.relative_block_id())?;
                let (width, height) = self.footprint(block)?;
                Some(place_relative(&parent, width, height, *container.position_x(), *container.position_y()))
            },
            Block::Spanning(span) => span.spanned_block_ids()
                .iter()
                .map(|member_id| self.place(*member_id))
                .collect::<Option<Vec<_>>>()?
                .into_iter()
                .reduce(|total, rect| total.union(&rect)),
        });

        self.visiting.remove(&block_id);
        match rect {
            Some(rect) => { self.rects.insert(block_id, rect); },
            None => { self.unplaceable.insert(block_id); },
        }

        rect
    }

    /// Returns the width and height of a container with its simulated unit count, or `None` for spans.
    fn footprint(&self, block: &GroupFormationBlock) -> Option<(f32, f32)> {
        let (arrangement, spacing, crescent_y_offset, minimum, maximum) = match &block.block {
            Block::ContainerAbsolute(container) => (container.entity_arrangement, container.inter_entity_spacing, container.crescent_y_offset, container.minimum_entity_threshold, container.maximum_entity_threshold),
            Block::ContainerRelative(container) => (container.entity_arrangement, container.inter_entity_spacing, container.crescent_y_offset, container.minimum_entity_threshold, container.maximum_entity_threshold),
            Block::Spanning(_) => return None,
        };

        let count = self.params.unit_counts.get(&block.block_id).copied().unwrap_or(self.params.default_unit_count) as i32;

        // A negative maximum means there's no limit. Empty blocks still get one unit so they're visible.
        let count = if maximum >= 0 { count.min(maximum) } else { count };
        let count = count.max(minimum).max(1) as f32;

        let width = count * self.params.unit_width + (count - 1.0) * spacing;
        let depth = count * self.params.unit_depth + (count - 1.0) * spacing;
        match arrangement {
            EntityArrangement::Line => Some((width, self.params.unit_depth)),

            // Crescents are lines bent by their offset, so they're deeper by it.
            EntityArrangement::CrescentFront |
            EntityArrangement::CrescentBack => Some((width, self.params.unit_depth + crescent_y_offset.abs())),
            EntityArrangement::Column => Some((self.params.unit_width, depth)),
        }
    }
}

/// Places a block relative to its parent.
///
/// # Arguments
///
/// * `parent` - Rect of the parent block.
/// * `width` - Width of the block being placed.
/// * `height` - Height of the block being placed.
/// * `offset_x` - Gap to the parent's right edge if positive, to its left edge if negative, or 0 to center it.
/// * `offset_y` - Gap to the parent's front edge if positive, to its back edge if negative, or 0 to center it.
///
/// # Returns
///
/// The rect of the placed block.
pub fn place_relative(parent: &LayoutRect, width: f32, height: f32, offset_x: f32, offset_y: f32) -> LayoutRect {
    let center_x = if offset_x > 0.0 {
        parent.right() + offset_x + width / 2.0
    } else if offset_x < 0.0 {
        parent.left() + offset_x - width / 2.0
    } else {
        parent.center_x
    };

    let center_y = if offset_y > 0.0 {
        parent.front() + offset_y + height / 2.0
    } else if offset_y < 0.0 {
        parent.back() + offset_y - height / 2.0
    } else {
        parent.center_y
    };

    LayoutRect::new(center_x, center_y, width, height)
}

/// Computes the offset that places a block closest to a target position, as the inverse of [`place_relative`].
///
/// # Arguments
///
/// * `parent` - Rect of the parent block.
/// * `width` - Width of the block being placed.
/// * `height` - Height of the block being placed.
/// * `center_x` - Target X coordinate of the block's center.
/// * `center_y` - Target Y coordinate of the block's center.
///
/// # Returns
///
/// The X and Y offsets. Axes on which the target overlaps the parent get a 0 offset, centering the block on the parent.
pub fn offset_for_center(parent: &LayoutRect, width: f32, height: f32, center_x: f32, center_y: f32) -> (f32, f32) {
    let left = center_x - width / 2.0;
    let right = center_x + width / 2.0;
    let offset_x = if left > parent.right() {
        left - parent.right()
    } else if right < parent.left() {
        right - parent.left()
    } else {
        0.0
    };

    let back = center_y - height / 2.0;
    let front = center_y + height / 2.0;
    let offset_y = if back > parent.front() {
        back - parent.front()
    } else if front < parent.back() {
        front - parent.back()
    } else {
        0.0
    };

    (offset_x, offset_y)
}
