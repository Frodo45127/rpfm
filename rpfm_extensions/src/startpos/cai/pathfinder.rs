//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! The AI's high-level pathfinder graph (`CAI_HIGH_LEVEL_PATHFINDER`, from the map's `hlp_data.esf`), and the
//! distances between regions the AI derives from it.
//!
//! The record is a flat list of leaves: the region count, then for each region its index and its nodes. A node is
//! its id (area index << 9 | region index), its cell, an unknown U32, its portals and the cost of crossing the node
//! from each portal to each other one. A portal is the segment it crosses, the cost of crossing it, the node it leads
//! to, its slot in the node's cost table and two flags. The cost table lists, slot by slot, the cost to every other
//! slot. The region-to-region tables after the regions are filled in by the game.

use std::cmp::Reverse;
use std::collections::{BTreeSet, BinaryHeap, HashMap};

use rpfm_lib::error::{RLibError, Result};
use rpfm_lib::files::esf::{NodeType, RecordNode};

use crate::startpos::esf_nav::leaves;

#[cfg(test)] #[path = "pathfinder_test.rs"] mod pathfinder_test;

//---------------------------------------------------------------------------//
//                              Types
//---------------------------------------------------------------------------//

/// The high-level pathfinder graph of a map.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Pathfinder {
    nodes: Vec<PathNode>,
}

/// A node of the graph: an area of a region.
#[derive(Debug, Clone, PartialEq, Default)]
struct PathNode {

    /// Index of the node's region in the map.
    region: u32,

    /// Portals to neighbour nodes.
    portals: Vec<Portal>,

    /// Cost of crossing the node between two portals, by portal slot (see the module docs).
    crossing_costs: Vec<u32>,
}

/// A portal from a node to a neighbour one.
#[derive(Debug, Clone, PartialEq, Default)]
struct Portal {

    /// Ends of the segment the portal crosses.
    segment: [u32; 4],

    /// Cost of crossing the portal.
    cost: u32,

    /// Position of the neighbour node in the graph.
    target: usize,

    /// Position in the neighbour node of the portal leading back.
    back: usize,

    /// Slot of the portal in the node's cost table.
    slot: usize,
}

//---------------------------------------------------------------------------//
//                              Implementations
//---------------------------------------------------------------------------//

impl Pathfinder {

    /// Reads the graph of a `CAI_HIGH_LEVEL_PATHFINDER` record.
    ///
    /// # Errors
    ///
    /// Returns an error if the record doesn't have the expected leaves.
    pub fn read(record: &RecordNode) -> Result<Self> {
        let invalid = || RLibError::CampaignMapDataError("unexpected CAI_HIGH_LEVEL_PATHFINDER layout".to_owned());
        let group = record.children().first().ok_or_else(invalid)?;
        let mut values = leaves(group).map(|leaf| match leaf {
            NodeType::U8(value) => Some(u32::from(*value)),
            NodeType::U16(value) => Some(u32::from(*value)),
            NodeType::U32(value) => Some(*value.value()),
            NodeType::Bool(value) => Some(u32::from(*value.value())),
            _ => None,
        });
        let mut next = || values.next().flatten().ok_or_else(invalid);

        let mut ids = HashMap::new();
        let mut node_targets = vec![];
        let mut nodes = vec![];
        for _ in 0..next()? {
            let region = next()?;
            for _ in 0..next()? {
                ids.insert(next()?, nodes.len());
                let _cell = (next()?, next()?);
                let _unknown = next()?;

                let mut portals = vec![];
                let mut targets = vec![];
                for _ in 0..next()? {
                    let segment = [next()?, next()?, next()?, next()?];
                    let cost = next()?;
                    targets.push(next()?);
                    let slot = next()? as usize;
                    let _flags = (next()?, next()?);
                    portals.push(Portal { segment, cost, slot, ..Default::default() });
                }
                let crossings = portals.len() * portals.len().saturating_sub(1);
                let crossing_costs = (0..crossings).map(|_| next()).collect::<Result<Vec<_>>>()?;
                node_targets.push(targets);
                nodes.push(PathNode { region, portals, crossing_costs });
            }
        }

        // Link the portals: each leads to a node, through the portal there that crosses the same segment back.
        for (position, targets) in node_targets.into_iter().enumerate() {
            for (portal, target_id) in targets.into_iter().enumerate() {
                nodes[position].portals[portal].target = *ids.get(&target_id).ok_or_else(invalid)?;
            }
        }
        for position in 0..nodes.len() {
            for portal in 0..nodes[position].portals.len() {
                let Portal { segment: [x1, y1, x2, y2], target, .. } = nodes[position].portals[portal];
                let back = nodes[target].portals.iter()
                    .position(|other| other.target == position && other.segment == [x2, y2, x1, y1])
                    .ok_or_else(invalid)?;
                nodes[position].portals[portal].back = back;
            }
        }
        Ok(Self { nodes })
    }

    /// Returns the largest cost of a path between two regions, as the game stores it in
    /// `CAI_INTERFACE_MAX_REGION_DISTANCE`. Regions without a path between them don't count.
    pub fn max_region_distance(&self) -> u32 {
        let regions = self.nodes.iter().map(|node| node.region).collect::<BTreeSet<_>>();
        regions.iter()
            .flat_map(|&region| {
                let costs = self.region_costs(region);
                costs.into_iter().filter(move |(other, _)| *other > region).map(|(_, cost)| cost)
            })
            .max()
            .unwrap_or_default()
    }

    /// Returns the cost of the cheapest path from a region to every region it reaches.
    ///
    /// Paths leave the region through any portal of its nodes, then cross each node from the portal they entered by
    /// to the one they leave by, paying the crossing cost of the node and of each portal.
    fn region_costs(&self, region: u32) -> HashMap<u32, u32> {
        let mut costs = HashMap::<(usize, usize), u32>::new();
        let mut queue = BinaryHeap::new();
        let push = |costs: &mut HashMap<(usize, usize), u32>, queue: &mut BinaryHeap<_>, state: (usize, usize), cost: u32| {
            if costs.get(&state).is_none_or(|known| cost < *known) {
                costs.insert(state, cost);
                queue.push(Reverse((cost, state)));
            }
        };

        for node in self.nodes.iter().filter(|node| node.region == region) {
            for portal in &node.portals {
                push(&mut costs, &mut queue, (portal.target, portal.back), portal.cost);
            }
        }

        let mut regions = HashMap::new();
        while let Some(Reverse((cost, (position, entry)))) = queue.pop() {
            if costs.get(&(position, entry)).is_some_and(|known| cost > *known) {
                continue;
            }
            let node = &self.nodes[position];
            regions.entry(node.region).and_modify(|best: &mut u32| *best = (*best).min(cost)).or_insert(cost);
            for (exit, portal) in node.portals.iter().enumerate().filter(|(exit, _)| *exit != entry) {
                let total = cost.saturating_add(node.crossing_cost(entry, exit)).saturating_add(portal.cost);
                push(&mut costs, &mut queue, (portal.target, portal.back), total);
            }
        }
        regions.remove(&region);
        regions
    }
}

impl PathNode {

    /// Returns the cost of crossing the node from a portal to another one.
    fn crossing_cost(&self, from: usize, to: usize) -> u32 {
        let (from, to) = (self.portals[from].slot, self.portals[to].slot);
        let width = self.portals.len() - 1;
        let column = if to < from { to } else { to - 1 };
        self.crossing_costs.get(from * width + column).copied().unwrap_or(u32::MAX)
    }
}
