//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Consistency checks for Group Formations files.

use std::collections::{HashMap, HashSet, hash_map::Entry};

use super::{Block, GroupFormation, GroupFormationBlock, GroupFormations};

//---------------------------------------------------------------------------//
//                              Enum & Structs
//---------------------------------------------------------------------------//

/// A problem found in a Group Formations file.
#[derive(Clone, Debug, PartialEq)]
pub enum ValidationIssue {

    /// Another formation earlier in the file has the same name.
    DuplicateFormationName,

    /// The formation has no absolute container, so nothing anchors its blocks.
    NoAbsoluteBlock,

    /// More than one block uses this id.
    DuplicateBlockId(u32),

    /// A block references an id that doesn't exist in the formation.
    MissingReference { block_id: u32, referenced_id: u32 },

    /// A block references a block defined after it. Vanilla files never do this.
    ForwardReference { block_id: u32, referenced_id: u32 },

    /// These blocks depend on each other's position in a loop.
    ReferenceCycle(Vec<u32>),

    /// A spanning block with no members.
    EmptySpan(u32),

    /// A container whose minimum entity threshold is above its maximum.
    InvalidThresholds(u32),

    /// A container with no entity preferences, so no unit can be placed in it.
    NoEntityPreferences(u32),
}

/// Visit state of a block while searching for reference cycles.
#[derive(Clone, Copy, PartialEq)]
enum VisitState {
    Unvisited,
    OnStack,
    Done,
}

/// Depth-first search over the block dependencies, collecting every cycle it finds.
struct CycleFinder<'a> {
    blocks: &'a [GroupFormationBlock],
    positions: &'a HashMap<u32, usize>,
    states: Vec<VisitState>,
    stack: Vec<usize>,
    cycles: Vec<Vec<u32>>,
}

//---------------------------------------------------------------------------//
//                             Implementations
//---------------------------------------------------------------------------//

impl ValidationIssue {

    /// Returns if this issue can break the formation in-game, as opposed to being just suspicious.
    pub fn is_error(&self) -> bool {
        match self {
            Self::NoAbsoluteBlock |
            Self::DuplicateBlockId(_) |
            Self::MissingReference { .. } |
            Self::ReferenceCycle(_) => true,
            Self::DuplicateFormationName |
            Self::ForwardReference { .. } |
            Self::EmptySpan(_) |
            Self::InvalidThresholds(_) |
            Self::NoEntityPreferences(_) => false,
        }
    }
}

impl GroupFormations {

    /// Checks all the formations of the file for consistency problems.
    ///
    /// # Returns
    ///
    /// A list of issues, each one paired with the index of the formation it was found in.
    pub fn validate(&self) -> Vec<(usize, ValidationIssue)> {
        let mut issues = vec![];
        let mut names = HashSet::with_capacity(self.formations.len());

        for (index, formation) in self.formations.iter().enumerate() {
            if !names.insert(formation.name.as_str()) {
                issues.push((index, ValidationIssue::DuplicateFormationName));
            }

            issues.extend(formation.validate().into_iter().map(|issue| (index, issue)));
        }

        issues
    }
}

impl GroupFormation {

    /// Checks the blocks of this formation for consistency problems.
    ///
    /// # Returns
    ///
    /// A list of the issues found, in block order.
    pub fn validate(&self) -> Vec<ValidationIssue> {
        let mut issues = vec![];
        let blocks = &self.group_formation_blocks;

        if !blocks.iter().any(|block| matches!(block.block, Block::ContainerAbsolute(_))) {
            issues.push(ValidationIssue::NoAbsoluteBlock);
        }

        // Duplicated ids keep the position of their first appearance, as that's the one references will find.
        let mut positions = HashMap::with_capacity(blocks.len());
        for (position, block) in blocks.iter().enumerate() {
            match positions.entry(block.block_id) {
                Entry::Occupied(_) => issues.push(ValidationIssue::DuplicateBlockId(block.block_id)),
                Entry::Vacant(entry) => { entry.insert(position); },
            }
        }

        for (position, block) in blocks.iter().enumerate() {
            let block_id = block.block_id;
            for referenced_id in block.block.dependencies() {
                match positions.get(referenced_id) {
                    None => issues.push(ValidationIssue::MissingReference { block_id, referenced_id: *referenced_id }),
                    Some(referenced_position) if *referenced_position > position => issues.push(ValidationIssue::ForwardReference { block_id, referenced_id: *referenced_id }),
                    Some(_) => {},
                }
            }

            let (minimum, maximum, has_preferences) = match &block.block {
                Block::ContainerAbsolute(container) => (*container.minimum_entity_threshold(), *container.maximum_entity_threshold(), !container.entity_preferences().is_empty()),
                Block::ContainerRelative(container) => (*container.minimum_entity_threshold(), *container.maximum_entity_threshold(), !container.entity_preferences().is_empty()),
                Block::Spanning(span) => {
                    if span.spanned_block_ids().is_empty() {
                        issues.push(ValidationIssue::EmptySpan(block_id));
                    }
                    continue;
                },
            };

            // A negative maximum means there's no limit.
            if maximum >= 0 && minimum > maximum {
                issues.push(ValidationIssue::InvalidThresholds(block_id));
            }

            if !has_preferences {
                issues.push(ValidationIssue::NoEntityPreferences(block_id));
            }
        }

        let mut finder = CycleFinder {
            blocks,
            positions: &positions,
            states: vec![VisitState::Unvisited; blocks.len()],
            stack: vec![],
            cycles: vec![],
        };

        for position in 0..blocks.len() {
            if finder.states[position] == VisitState::Unvisited {
                finder.visit(position);
            }
        }

        issues.extend(finder.cycles.into_iter().map(ValidationIssue::ReferenceCycle));
        issues
    }
}

impl CycleFinder<'_> {

    /// Visits a block and all its dependencies, recording a cycle whenever a dependency is still on the stack.
    fn visit(&mut self, position: usize) {
        self.states[position] = VisitState::OnStack;
        self.stack.push(position);

        for referenced_id in self.blocks[position].block.dependencies() {
            let Some(&referenced_position) = self.positions.get(referenced_id) else { continue };

            match self.states[referenced_position] {
                VisitState::Unvisited => self.visit(referenced_position),
                VisitState::OnStack => {
                    let start = self.stack.iter().position(|stacked| *stacked == referenced_position).expect("a block marked as on stack must be in the stack");
                    self.cycles.push(self.stack[start..].iter().map(|stacked| self.blocks[*stacked].block_id).collect());
                },
                VisitState::Done => {},
            }
        }

        self.stack.pop();
        self.states[position] = VisitState::Done;
    }
}
