//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Editing operations over the blocks of a formation.
//!
//! Vanilla files always define every block after the blocks it depends on. Operations that can break
//! that order sort the blocks afterwards. Block ids are never changed.

use std::collections::{BTreeSet, HashMap, HashSet};

use crate::error::{Result, RLibError};

use super::{Block, ContainerAbsolute, ContainerRelative, EntityArrangement, EntityPreference, GroupFormation, GroupFormationBlock, GroupFormationsFormat, Spanning};
use super::layout::{self, LayoutParams, LayoutRect};

//---------------------------------------------------------------------------//
//                             Implementations
//---------------------------------------------------------------------------//

impl GroupFormation {

    /// Adds a container with the defaults used by vanilla files at the end of the formation.
    ///
    /// # Arguments
    ///
    /// * `parent_id` - Block to position the container relative to, or `None` for an absolute container.
    /// * `position` - Offset from the parent for relative containers, or position for absolute ones.
    /// * `format` - Format of the file, to create a valid entity preference.
    ///
    /// # Returns
    ///
    /// The id of the new block.
    ///
    /// # Errors
    ///
    /// Returns an error if the parent doesn't exist.
    pub fn add_container(&mut self, parent_id: Option<u32>, position: (f32, f32), format: GroupFormationsFormat) -> Result<u32> {
        let container = ContainerAbsolute {
            block_priority: 1.0,
            entity_arrangement: EntityArrangement::Line,
            inter_entity_spacing: 2.0,
            crescent_y_offset: 0.0,
            position_x: position.0,
            position_y: position.1,
            minimum_entity_threshold: 0,
            maximum_entity_threshold: -1,
            entity_preferences: vec![EntityPreference::new(format)],
        };

        let block = match parent_id {
            Some(parent_id) => {
                self.position_of(parent_id)?;
                Block::ContainerRelative(to_relative(container, parent_id))
            },
            None => Block::ContainerAbsolute(container),
        };

        Ok(self.push_block(block))
    }

    /// Adds a span over the provided blocks at the end of the formation.
    ///
    /// # Arguments
    ///
    /// * `member_ids` - Blocks covered by the span.
    ///
    /// # Returns
    ///
    /// The id of the new block.
    ///
    /// # Errors
    ///
    /// Returns an error if any of the members doesn't exist.
    pub fn add_span(&mut self, member_ids: &[u32]) -> Result<u32> {
        for member_id in member_ids {
            self.position_of(*member_id)?;
        }

        Ok(self.push_block(Block::Spanning(Spanning { spanned_block_ids: member_ids.to_vec() })))
    }

    /// Replaces the members of a span.
    ///
    /// # Arguments
    ///
    /// * `span_id` - The span to edit.
    /// * `member_ids` - New blocks covered by the span.
    ///
    /// # Errors
    ///
    /// Returns an error if a block doesn't exist, the edited block isn't a span, or a member depends on the span.
    pub fn set_span_members(&mut self, span_id: u32, member_ids: &[u32]) -> Result<()> {
        let span_position = self.position_of(span_id)?;
        for member_id in member_ids {
            self.position_of(*member_id)?;
            if self.depends_on(*member_id, span_id) {
                return Err(RLibError::GroupFormationsCyclicReference(span_id, *member_id));
            }
        }

        match &mut self.group_formation_blocks[span_position].block {
            Block::Spanning(span) => span.spanned_block_ids = member_ids.to_vec(),
            Block::ContainerAbsolute(_) |
            Block::ContainerRelative(_) => return Err(RLibError::GroupFormationsBlockIsNotASpan(span_id)),
        }

        self.sort_blocks();
        Ok(())
    }

    /// Changes the parent of a container, keeping its simulated position when possible.
    ///
    /// # Arguments
    ///
    /// * `block_id` - The container to edit.
    /// * `parent_id` - The new parent, or `None` to turn it into an absolute container.
    /// * `params` - The simulated deployment used to keep the container in place.
    ///
    /// # Errors
    ///
    /// Returns an error if a block doesn't exist, the edited block is a span, or the new parent depends on the edited block.
    pub fn reparent(&mut self, block_id: u32, parent_id: Option<u32>, params: &LayoutParams) -> Result<()> {
        let position = self.position_of(block_id)?;
        if let Some(parent_id) = parent_id {
            self.position_of(parent_id)?;
            if self.depends_on(parent_id, block_id) {
                return Err(RLibError::GroupFormationsCyclicReference(block_id, parent_id));
            }
        }

        if let Block::Spanning(_) = self.group_formation_blocks[position].block {
            return Err(RLibError::GroupFormationsBlockIsNotAContainer(block_id));
        }

        let rects = self.layout(params);
        self.attach(position, parent_id, &rects);
        self.sort_blocks();
        Ok(())
    }

    /// Moves a container so its simulated center lands on the target, snapping its offset or position to a grid.
    ///
    /// # Arguments
    ///
    /// * `block_id` - The container to move.
    /// * `center` - Target position of the container's center.
    /// * `grid_step` - Size of the grid to snap to, or 0 to not snap.
    /// * `params` - The simulated deployment used to compute the offset.
    ///
    /// # Errors
    ///
    /// Returns an error if the block doesn't exist, is a span, or it or its parent can't be positioned.
    pub fn move_block(&mut self, block_id: u32, center: (f32, f32), grid_step: f32, params: &LayoutParams) -> Result<()> {
        let position = self.position_of(block_id)?;
        let rects = self.layout(params);
        let rect = rects.get(&block_id).ok_or(RLibError::GroupFormationsBlockNotPlaceable(block_id))?;

        match &mut self.group_formation_blocks[position].block {
            Block::ContainerAbsolute(container) => {
                container.position_x = snap(center.0, grid_step);
                container.position_y = snap(center.1, grid_step);
            },
            Block::ContainerRelative(container) => {
                let parent = rects.get(&container.relative_block_id).ok_or(RLibError::GroupFormationsBlockNotPlaceable(container.relative_block_id))?;
                let (offset_x, offset_y) = layout::offset_for_center(parent, rect.width(), rect.height(), center.0, center.1);
                container.position_x = snap(offset_x, grid_step);
                container.position_y = snap(offset_y, grid_step);
            },
            Block::Spanning(_) => return Err(RLibError::GroupFormationsBlockIsNotAContainer(block_id)),
        }

        Ok(())
    }

    /// Deletes blocks, re-attaching the containers positioned relative to them to their closest surviving ancestor.
    ///
    /// Spans left without members are deleted too. Containers left without ancestors become absolute.
    ///
    /// # Arguments
    ///
    /// * `block_ids` - The blocks to delete.
    /// * `params` - The simulated deployment used to keep re-attached containers in place.
    ///
    /// # Errors
    ///
    /// Returns an error if any of the blocks doesn't exist.
    pub fn delete_blocks(&mut self, block_ids: &[u32], params: &LayoutParams) -> Result<()> {
        for block_id in block_ids {
            self.position_of(*block_id)?;
        }

        let rects = self.layout(params);
        let deleted = self.with_emptied_spans(block_ids.iter().copied().collect());

        let parents = self.group_formation_blocks.iter()
            .filter_map(|block| match &block.block {
                Block::ContainerRelative(container) => Some((block.block_id, container.relative_block_id)),
                Block::ContainerAbsolute(_) |
                Block::Spanning(_) => None,
            })
            .collect::<HashMap<_, _>>();

        for position in 0..self.group_formation_blocks.len() {
            let block_id = self.group_formation_blocks[position].block_id;
            let Some(parent_id) = parents.get(&block_id) else { continue };
            if deleted.contains(&block_id) || !deleted.contains(parent_id) {
                continue;
            }

            // The step limit protects against cycles between deleted blocks.
            let mut ancestor = Some(*parent_id);
            for _ in 0..=self.group_formation_blocks.len() {
                match ancestor {
                    Some(ancestor_id) if deleted.contains(&ancestor_id) => ancestor = parents.get(&ancestor_id).copied(),
                    _ => break,
                }
            }

            let ancestor = ancestor.filter(|ancestor_id| !deleted.contains(ancestor_id));
            self.attach(position, ancestor, &rects);
        }

        self.group_formation_blocks.retain(|block| !deleted.contains(&block.block_id));
        for block in &mut self.group_formation_blocks {
            if let Block::Spanning(span) = &mut block.block {
                span.spanned_block_ids.retain(|member_id| !deleted.contains(member_id));
            }
        }

        self.sort_blocks();
        Ok(())
    }

    /// Deletes a block, all the containers positioned relative to it, recursively, and the spans left without members.
    ///
    /// # Arguments
    ///
    /// * `block_id` - The root of the subtree to delete.
    /// * `params` - The simulated deployment used to keep re-attached containers in place.
    ///
    /// # Errors
    ///
    /// Returns an error if the block doesn't exist.
    pub fn delete_subtree(&mut self, block_id: u32, params: &LayoutParams) -> Result<()> {
        self.position_of(block_id)?;

        let mut subtree = HashSet::from([block_id]);
        loop {
            let children = self.group_formation_blocks.iter()
                .filter(|block| !subtree.contains(&block.block_id))
                .filter(|block| match &block.block {
                    Block::ContainerRelative(container) => subtree.contains(&container.relative_block_id),
                    Block::ContainerAbsolute(_) => false,
                    Block::Spanning(span) => !span.spanned_block_ids.is_empty() && span.spanned_block_ids.iter().all(|member_id| subtree.contains(member_id)),
                })
                .map(|block| block.block_id)
                .collect::<Vec<_>>();

            if children.is_empty() {
                break;
            }

            subtree.extend(children);
        }

        self.delete_blocks(&subtree.into_iter().collect::<Vec<_>>(), params)
    }

    /// Reorders the blocks so each one comes after the blocks it depends on.
    ///
    /// Blocks keep their current order when possible, and blocks in reference cycles are moved to the end.
    pub fn sort_blocks(&mut self) {
        let mut positions = HashMap::with_capacity(self.group_formation_blocks.len());
        for (position, block) in self.group_formation_blocks.iter().enumerate() {
            positions.entry(block.block_id).or_insert(position);
        }

        // Kahn's algorithm, always taking the earliest ready block so the current order is kept when possible.
        let mut pending_dependencies = vec![0; self.group_formation_blocks.len()];
        let mut dependents = vec![vec![]; self.group_formation_blocks.len()];
        for (position, block) in self.group_formation_blocks.iter().enumerate() {
            for dependency in block.block.dependencies() {
                if let Some(dependency_position) = positions.get(dependency) {
                    pending_dependencies[position] += 1;
                    dependents[*dependency_position].push(position);
                }
            }
        }

        let mut ready = pending_dependencies.iter()
            .enumerate()
            .filter(|(_, pending)| **pending == 0)
            .map(|(position, _)| position)
            .collect::<BTreeSet<_>>();

        let mut order = Vec::with_capacity(self.group_formation_blocks.len());
        while let Some(position) = ready.pop_first() {
            order.push(position);
            for dependent in &dependents[position] {
                pending_dependencies[*dependent] -= 1;
                if pending_dependencies[*dependent] == 0 {
                    ready.insert(*dependent);
                }
            }
        }

        let sorted = order.iter().copied().collect::<HashSet<_>>();
        order.extend((0..self.group_formation_blocks.len()).filter(|position| !sorted.contains(position)));

        let mut blocks = std::mem::take(&mut self.group_formation_blocks).into_iter().map(Some).collect::<Vec<_>>();
        self.group_formation_blocks = order.iter()
            .map(|position| blocks[*position].take().expect("every position appears once in the order"))
            .collect();
    }

    /// Returns the position in the block list of the first block with the provided id.
    fn position_of(&self, block_id: u32) -> Result<usize> {
        self.group_formation_blocks.iter()
            .position(|block| block.block_id == block_id)
            .ok_or(RLibError::GroupFormationsBlockNotFound(block_id))
    }

    /// Returns if a block is the target, or depends on it directly or through other blocks.
    ///
    /// # Arguments
    ///
    /// * `block_id` - The block to check.
    /// * `target_id` - The block that may be depended on.
    ///
    /// # Returns
    ///
    /// `true` if making the target depend on the block would create a reference cycle.
    pub fn depends_on(&self, block_id: u32, target_id: u32) -> bool {
        let mut visited = HashSet::new();
        let mut pending = vec![block_id];
        while let Some(current_id) = pending.pop() {
            if current_id == target_id {
                return true;
            }

            if visited.insert(current_id) {
                if let Ok(position) = self.position_of(current_id) {
                    pending.extend_from_slice(self.group_formation_blocks[position].block.dependencies());
                }
            }
        }

        false
    }

    /// Returns the provided blocks plus the spans that would be left without members if they were deleted.
    fn with_emptied_spans(&self, mut deleted: HashSet<u32>) -> HashSet<u32> {
        loop {
            let emptied = self.group_formation_blocks.iter()
                .filter(|block| !deleted.contains(&block.block_id))
                .filter(|block| match &block.block {
                    Block::Spanning(span) => !span.spanned_block_ids.is_empty() && span.spanned_block_ids.iter().all(|member_id| deleted.contains(member_id)),
                    Block::ContainerAbsolute(_) |
                    Block::ContainerRelative(_) => false,
                })
                .map(|block| block.block_id)
                .collect::<Vec<_>>();

            if emptied.is_empty() {
                return deleted;
            }

            deleted.extend(emptied);
        }
    }

    /// Makes the container at the provided position absolute or relative to a new parent, keeping its simulated position if it has one.
    fn attach(&mut self, position: usize, parent_id: Option<u32>, rects: &HashMap<u32, LayoutRect>) {
        let block = &mut self.group_formation_blocks[position];
        let rect = rects.get(&block.block_id);
        let mut container = match std::mem::take(&mut block.block) {
            Block::ContainerAbsolute(container) => container,
            Block::ContainerRelative(container) => to_absolute(container),
            span @ Block::Spanning(_) => {
                block.block = span;
                return;
            },
        };

        block.block = match parent_id {
            Some(parent_id) => {
                if let (Some(rect), Some(parent)) = (rect, rects.get(&parent_id)) {
                    (container.position_x, container.position_y) = layout::offset_for_center(parent, rect.width(), rect.height(), rect.center_x(), rect.center_y());
                }
                Block::ContainerRelative(to_relative(container, parent_id))
            },
            None => {
                if let Some(rect) = rect {
                    (container.position_x, container.position_y) = (rect.center_x(), rect.center_y());
                }
                Block::ContainerAbsolute(container)
            },
        };
    }

    /// Adds a block at the end of the formation, with an id after all the existing ones.
    fn push_block(&mut self, block: Block) -> u32 {
        let block_id = self.group_formation_blocks.iter().map(|block| block.block_id.saturating_add(1)).max().unwrap_or(0);
        self.group_formation_blocks.push(GroupFormationBlock { block_id, block });
        block_id
    }
}

/// Converts a relative container into an absolute one, keeping its offset as position.
fn to_absolute(container: ContainerRelative) -> ContainerAbsolute {
    ContainerAbsolute {
        block_priority: container.block_priority,
        entity_arrangement: container.entity_arrangement,
        inter_entity_spacing: container.inter_entity_spacing,
        crescent_y_offset: container.crescent_y_offset,
        position_x: container.position_x,
        position_y: container.position_y,
        minimum_entity_threshold: container.minimum_entity_threshold,
        maximum_entity_threshold: container.maximum_entity_threshold,
        entity_preferences: container.entity_preferences,
    }
}

/// Converts an absolute container into one relative to the provided parent, keeping its position as offset.
fn to_relative(container: ContainerAbsolute, relative_block_id: u32) -> ContainerRelative {
    ContainerRelative {
        block_priority: container.block_priority,
        relative_block_id,
        entity_arrangement: container.entity_arrangement,
        inter_entity_spacing: container.inter_entity_spacing,
        crescent_y_offset: container.crescent_y_offset,
        position_x: container.position_x,
        position_y: container.position_y,
        minimum_entity_threshold: container.minimum_entity_threshold,
        maximum_entity_threshold: container.maximum_entity_threshold,
        entity_preferences: container.entity_preferences,
    }
}

/// Rounds a value to the closest multiple of the grid step, if the step is positive.
fn snap(value: f32, grid_step: f32) -> f32 {
    if grid_step > 0.0 {
        (value / grid_step).round() * grid_step
    } else {
        value
    }
}
