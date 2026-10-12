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
//!
//! The shroud also lists the settlements a faction doesn't see yet (`CAMPAIGN_SHROUD_FILLERS`), each with what it
//! would reveal, grouped by the tile of a 16x16 split of the map that holds them. The AI needs them to deal with
//! other factions. A settlement reveals the cells within two hexes of the zones of its primary slots, inside the
//! bounds of those zones widened by [`FILLER_MARGIN`]. Verified on Attila's main campaign: 150 of 186 settlements
//! reveal exactly what the game's do (the rest differ by a few cells at the edges of the bounds, whose vertical
//! extent the game takes from elsewhere).

use std::cmp::Reverse;
use std::collections::{BTreeMap, HashMap, HashSet};

use rpfm_lib::error::Result;
use rpfm_lib::files::esf::{Coordinates2DNode, NodeType, RecordNode};

use super::{child, WorldInputs, WorldPlan};
use crate::startpos::coords::{HexCell, RAW_SCALE_X};
use crate::startpos::layout::{child_path, StartposLayout};
use crate::startpos::settlements::SettlementLayout;
use crate::startpos::vision::{Vision, VisionGrid, WorldBounds};

#[cfg(test)] #[path = "sight_test.rs"] mod sight_test;

/// Sight radius of settlements, in world units (a colonel's `agents.line_of_sight`).
const SETTLEMENT_SIGHT: f32 = 5.0;

/// Hex steps a shroud filler reveals around the zones of its settlement's primary slots.
const FILLER_STEPS: usize = 2;

/// Slots whose zones a shroud filler reveals (`SprawlCell::zone_slots` bits): the main settlement and the port.
const FILLER_SLOTS: u32 = 0b11;

/// How far the bounds of a shroud filler's vision go past its zones, in world units (16 raw units).
const FILLER_MARGIN: f32 = (16.0 / RAW_SCALE_X) as f32;

/// Buckets of the hash map a tile keeps its shroud fillers in, keyed by settlement cell.
const FILLER_BUCKETS: u32 = 4;

/// Tiles per side of the split of the map the shroud fillers are grouped by.
const FILLER_TILES: f32 = 16.0;

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

    /// Shroud fillers of the settlements, in region order.
    fillers: Vec<Filler>,

    /// Size of a tile of the fillers' split of the map, in world units.
    filler_tile_size: (f32, f32),
}

/// A settlement that fills the shroud of a faction once the faction sees it.
#[derive(Debug, Clone)]
struct Filler {

    /// Cell of the settlement.
    cell: HexCell,

    /// Vision cell of the settlement: the faction gets the filler if it doesn't see this cell.
    vision_cell: (u32, u32),

    /// What the settlement reveals.
    vision: Vision,

    /// World bounds of what the settlement reveals.
    bounds: WorldBounds,
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
        let mut fillers = vec![];
        for region in plan.regions() {
            let corners = cells.get(region.key().as_str()).map(|cells| cells.iter().map(|cell| cell.world_position()).collect::<Vec<_>>()).unwrap_or_default();
            regions.insert(region.key().to_owned(), grid.around(&corners, REGION_SIGHT));

            let Some(settlement) = inputs.settlements.iter().find(|settlement| settlement.region() == region.key()) else { continue };
            if let Some(position) = settlement.slots().first().and_then(|slot| slot.position()) {
                settlements.insert(region.key().to_owned(), grid.disk(position.world_position(), SETTLEMENT_SIGHT));
                let (vision, bounds) = filler_vision(&grid, settlement);
                fillers.push(Filler { cell: position, vision_cell: grid.cell_at(position.world_position()), vision, bounds });
            }
        }

        let (width, height) = inputs.region_map.world_size();
        let filler_tile_size = (width / FILLER_TILES, height / FILLER_TILES);
        Self { grid, regions, settlements, fillers, filler_tile_size }
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
/// area itself. Without an area, it sees nothing, over the same grid.
///
/// # Errors
///
/// Returns an error if the layout lacks the records.
pub fn write_line_of_sight(layout: &StartposLayout, parent: &str, grid: &VisionGrid, vision: Option<&Vision>) -> Result<RecordNode> {
    let bounds = vision.and_then(|vision| vision.bounds(grid)).unwrap_or_default();
    line_of_sight(layout, parent, grid, vision, bounds)
}

/// Writes a `LINE_OF_SIGHT` record with the given world bounds.
fn line_of_sight(layout: &StartposLayout, parent: &str, grid: &VisionGrid, vision: Option<&Vision>, bounds: WorldBounds) -> Result<RecordNode> {
    let path = format!("{parent}/LINE_OF_SIGHT");
    let mut record = layout.builder(&path)?;
    let Some(vision) = vision else {
        record.group()?
            .set_bool("^+1", false)?
            .unset("^+2")?
            .unset("^+3")?
            .set_record("QUAD_TREE_BIT_ARRAY", Some(write_quad_tree(layout, &path, grid, &Vision::default())?))?;
        return record.build();
    };

    let ((min_x, min_y), (max_x, max_y)) = bounds;
    record.group()?
        .set_bool("^+1", true)?
        .set("^+2", coordinates(min_x, min_y))?
        .set("^+3", coordinates(max_x, max_y))?
        .set_record("QUAD_TREE_BIT_ARRAY", Some(write_quad_tree(layout, &path, grid, vision)?))?;
    record.build()
}

/// Writes the shroud of a playable faction (`CAMPAIGN_SHROUD`): everything it sees is also what it has seen, and the
/// settlements it doesn't see fill it later.
///
/// # Arguments
///
/// * `layout` - Layout of the startpos.
/// * `faction_path` - Path of the faction record.
/// * `sight` - Vision of the world.
/// * `vision` - What the faction sees.
///
/// # Errors
///
/// Returns an error if the layout lacks the records.
pub fn write_shroud(layout: &StartposLayout, faction_path: &str, sight: &SightPlan, vision: &Vision) -> Result<RecordNode> {
    let path = format!("{faction_path}/CAMPAIGN_SHROUD");
    let layers = [vision, vision, &Vision::default()];
    let mut record = layout.builder(&path)?;
    let group = record.group()?;
    for (key, layer) in SHROUD_LAYERS.iter().zip(layers) {
        group.set_record(key, Some(write_quad_tree(layout, &path, &sight.grid, layer)?))?;
    }
    group.set_record("CAMPAIGN_SHROUD_FILLERS", Some(write_fillers(layout, &path, sight, vision)?))?;
    record.build()
}

/// Writes the shroud fillers of a faction: the settlements outside its vision, by tile from the top row of tiles
/// down, each tile with the world bounds of itself and what its settlements reveal, and its settlements in the order
/// of its hash map.
fn write_fillers(layout: &StartposLayout, shroud_path: &str, sight: &SightPlan, vision: &Vision) -> Result<RecordNode> {
    let path = child_path(shroud_path, "CAMPAIGN_SHROUD_FILLERS");
    let block_path = child_path(&path, "FILLERS_QUADTREE_DATA_BLOCK");
    let los_path = child_path(&block_path, "FILLERS_QUADTREE_DATA_LOS_BLOCK");
    let (tile_width, tile_height) = sight.filler_tile_size;

    let mut tiles = BTreeMap::<(Reverse<u32>, u32), Vec<&Filler>>::new();
    for filler in sight.fillers.iter().filter(|filler| !vision.contains(filler.vision_cell)) {
        let (x, y) = filler.cell.world_position();
        tiles.entry((Reverse((y / tile_height) as u32), (x / tile_width) as u32)).or_default().push(filler);
    }

    let mut block = layout.builder(&block_path)?;
    for ((Reverse(row), column), mut fillers) in tiles {
        fillers.sort_by_key(|filler| filler_bucket(filler.cell));
        let (mut min_x, mut min_y) = (column as f32 * tile_width, row as f32 * tile_height);
        let (mut max_x, mut max_y) = (min_x + tile_width, min_y + tile_height);
        let mut los_block = layout.builder(&los_path)?;
        for filler in fillers {
            let ((x0, y0), (x1, y1)) = filler.bounds;
            (min_x, min_y, max_x, max_y) = (min_x.min(x0), min_y.min(y0), max_x.max(x1), max_y.max(y1));
            let line_of_sight = line_of_sight(layout, &los_path, &sight.grid, Some(&filler.vision), filler.bounds)?;
            los_block.push_children(vec![NodeType::U16(filler.cell.x()), NodeType::U16(filler.cell.y()), NodeType::Record(Box::new(line_of_sight))]);
        }
        block.push_children(vec![coordinates(min_x, min_y), coordinates(max_x, max_y), NodeType::Record(Box::new(los_block.build()?))]);
    }

    let mut record = layout.builder(&path)?;
    record.group()?.set_record("FILLERS_QUADTREE_DATA_BLOCK", Some(block.build()?))?;
    record.build()
}

/// Returns the bucket of a settlement cell in a tile's hash map of shroud fillers, which lists them bucket by bucket
/// (each bucket in region order).
fn filler_bucket(cell: HexCell) -> u32 {
    (u32::from(cell.y()) * 0x3f8 + u32::from(cell.x())) % FILLER_BUCKETS
}

/// Returns what a settlement reveals as a shroud filler, and the world bounds of it: the cells within
/// [`FILLER_STEPS`] hexes of its primary slots' zones, inside the zones' bounds widened by [`FILLER_MARGIN`].
fn filler_vision(grid: &VisionGrid, settlement: &SettlementLayout) -> (Vision, WorldBounds) {
    let zones = settlement.sprawl().iter().filter(|cell| cell.zone_slots() & FILLER_SLOTS != 0).map(|cell| cell.cell()).collect::<HashSet<_>>();
    if zones.is_empty() {
        return (Vision::default(), ((0.0, 0.0), (0.0, 0.0)));
    }
    let corners = zones.iter().map(|cell| cell.world_position()).collect::<Vec<_>>();
    let min = corners.iter().fold((f32::MAX, f32::MAX), |min, corner| (min.0.min(corner.0), min.1.min(corner.1)));
    let max = corners.iter().fold((f32::MIN, f32::MIN), |max, corner| (max.0.max(corner.0), max.1.max(corner.1)));

    let mut revealed = zones;
    for _ in 0..FILLER_STEPS {
        let ring = revealed.iter().flat_map(|cell| cell.neighbours()).collect::<Vec<_>>();
        revealed.extend(ring);
    }
    let min = (min.0 - FILLER_MARGIN, min.1 - FILLER_MARGIN);
    let max = (max.0 + FILLER_MARGIN, max.1 + FILLER_MARGIN);
    grid.cells_where(min, max, |(x, y)| revealed.contains(&HexCell::from_world(x, y)))
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
