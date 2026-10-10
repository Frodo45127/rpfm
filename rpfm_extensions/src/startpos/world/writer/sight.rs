//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Vision of the world: what regions, settlements and forces see (`LINE_OF_SIGHT`), and what each playable faction
//! has seen (`CAMPAIGN_SHROUD`).
//!
//! Characters see a disk of their agent's sight radius, and settlements one of a colonel's, as the game does. The
//! vision of regions, sea regions and the shroud are approximations: the game's exact rules aren't known, and it
//! recomputes vision while playing.

use std::collections::HashMap;

use rpfm_lib::error::Result;
use rpfm_lib::files::esf::{Coordinates2DNode, NodeType, RecordNode};

use super::{child, WorldInputs, WorldPlan};
use crate::startpos::layout::StartposLayout;
use crate::startpos::vision::{Vision, VisionGrid};

/// Sight radius of settlements, in world units (a colonel's `agents.line_of_sight`).
const SETTLEMENT_SIGHT: f32 = 5.0;

/// How far a region sees past its own cells, in world units. Fitted to the game's region vision (about 1% of the
/// cells differ).
const REGION_SIGHT: f32 = 1.7;

/// Quad trees of a faction's shroud: what it has seen, what it sees, and an empty one.
const SHROUD_LAYERS: [&str; 3] = ["QUAD_TREE_BIT_ARRAY", "QUAD_TREE_BIT_ARRAY@2", "QUAD_TREE_BIT_ARRAY@3"];

//---------------------------------------------------------------------------//
//                              Types
//---------------------------------------------------------------------------//

/// Vision of the regions and settlements of the world.
#[derive(Debug, Clone)]
pub struct SightPlan {

    /// The vision grid of the map.
    grid: VisionGrid,

    /// What each region sees, by region key.
    regions: HashMap<String, Vision>,

    /// What each settlement sees, by region key.
    settlements: HashMap<String, Vision>,
}

//---------------------------------------------------------------------------//
//                              Implementations
//---------------------------------------------------------------------------//

impl SightPlan {

    /// Computes the vision of the regions and settlements.
    pub fn new(inputs: &WorldInputs, plan: &WorldPlan) -> Self {
        let grid = VisionGrid::new(inputs.region_map.world_size());
        let cells = inputs.region_map.cells_by_region();

        let mut regions = HashMap::new();
        let mut settlements = HashMap::new();
        for region in plan.regions() {
            let corners = cells.get(region.key().as_str()).map(|cells| cells.iter().map(|cell| cell.world_position()).collect::<Vec<_>>()).unwrap_or_default();
            regions.insert(region.key().to_owned(), grid.around(&corners, REGION_SIGHT));

            let position = inputs.settlements.iter()
                .find(|settlement| settlement.region() == region.key())
                .and_then(|settlement| settlement.slots().first())
                .and_then(|slot| slot.position());
            if let Some(position) = position {
                settlements.insert(region.key().to_owned(), grid.disk(position.world_position(), SETTLEMENT_SIGHT));
            }
        }
        Self { grid, regions, settlements }
    }

    /// Returns the vision grid of the map.
    pub fn grid(&self) -> &VisionGrid {
        &self.grid
    }

    /// Returns what a region sees.
    pub fn region(&self, region: &str) -> Option<&Vision> {
        self.regions.get(region)
    }

    /// Returns what a region's settlement sees.
    pub fn settlement(&self, region: &str) -> Option<&Vision> {
        self.settlements.get(region)
    }
}

/// Writes a `LINE_OF_SIGHT` record under a parent record: whether it sees, the world bounds of what it sees and the
/// area itself. Without an area, it's the default record.
///
/// # Errors
///
/// Returns an error if the layout lacks the records.
pub fn write_line_of_sight(layout: &StartposLayout, parent: &str, grid: &VisionGrid, vision: Option<&Vision>) -> Result<RecordNode> {
    let path = format!("{parent}/LINE_OF_SIGHT");
    let mut record = layout.builder(&path)?;
    let Some(vision) = vision else {
        record.group()?
            .set_bool("^+1", false)?
            .unset("^+2")?
            .unset("^+3")?
            .set_record("QUAD_TREE_BIT_ARRAY", None)?;
        return record.build();
    };

    let ((min_x, min_y), (max_x, max_y)) = vision.bounds(grid).unwrap_or_default();
    record.group()?
        .set_bool("^+1", true)?
        .set("^+2", coordinates(min_x, min_y))?
        .set("^+3", coordinates(max_x, max_y))?
        .set_record("QUAD_TREE_BIT_ARRAY", Some(write_quad_tree(layout, &path, grid, vision)?))?;
    record.build()
}

/// Writes the shroud of a playable faction (`CAMPAIGN_SHROUD`): everything it sees is also what it has seen.
///
/// # Errors
///
/// Returns an error if the layout lacks the records.
pub fn write_shroud(layout: &StartposLayout, faction_path: &str, grid: &VisionGrid, vision: &Vision) -> Result<RecordNode> {
    let path = format!("{faction_path}/CAMPAIGN_SHROUD");
    let layers = [vision, vision, &Vision::default()];
    let mut record = layout.builder(&path)?;
    let group = record.group()?;
    for (key, layer) in SHROUD_LAYERS.iter().zip(layers) {
        group.set_record(key, Some(write_quad_tree(layout, &path, grid, layer)?))?;
    }
    record.build()
}

/// Writes a `QUAD_TREE_BIT_ARRAY` record under a parent record.
fn write_quad_tree(layout: &StartposLayout, parent: &str, grid: &VisionGrid, vision: &Vision) -> Result<RecordNode> {
    let (width, height, tree_size) = grid.dimensions();
    let mut record = child(layout, parent, "QUAD_TREE_BIT_ARRAY")?;
    record.group()?
        .set_u32("^+1", width)?
        .set_u32("^+2", height)?
        .set_u32("^+3", tree_size)?
        .set_bytes("^+4", vision.encode(grid))?;
    record.build()
}

/// Returns a `Coord2d` leaf.
fn coordinates(x: f32, y: f32) -> NodeType {
    let mut node = Coordinates2DNode::default();
    node.set_x(x);
    node.set_y(y);
    NodeType::Coord2d(node)
}
