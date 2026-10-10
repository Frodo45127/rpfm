//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Writer of the AI's theatre (`CAI_WORLD_THEATRES`), regions (`CAI_WORLD_REGIONS`) and their HLCIs
//! (`CAI_WORLD_REGION_HLCIS`): one region per map region (land, then seas), each split into HLCIs, the connected groups
//! of its areas.

use getset::Setters;

use std::collections::HashMap;

use rpfm_lib::error::Result;
use rpfm_lib::files::esf::RecordNode;

use super::map::{MapRegion, PathfinderNode};
use super::plan::CaiPlan;
use super::{component, CAI_WORLD_PATH};
use crate::startpos::layout::{child_path, StartposLayout};

#[cfg(test)] #[path = "regions_test.rs"] mod regions_test;

/// Theatre key of single-theatre campaigns, from `map_data.esf`'s theatre.
pub const THEATRE_VALUE: u32 = 3;

//---------------------------------------------------------------------------//
//                              Types
//---------------------------------------------------------------------------//

/// What a region refers to beyond the map: AI objects of the world mirror.
#[derive(Debug, Clone, Default, Setters)]
#[getset(set = "pub")]
pub struct RegionLinks {

    /// Component of the region's settlement (land regions).
    settlement: u32,

    /// Components of the region's boundaries.
    boundaries: Vec<u32>,

    /// Components of the armies and fleets in the region.
    mobiles: Vec<u32>,

    /// World id of the region (land regions).
    world_id: u32,

    /// Component of the region's province (land regions).
    province: u32,

    /// Index of the sea region (seas).
    sea_index: u32,

    /// Components of the land regions with a port on this sea (seas).
    port_regions: Vec<u32>,

    /// Component of the sea this region's port opens to (land regions).
    port_sea: u32,

    /// Components of the factions' views of this sea (seas).
    faction_seas: Vec<u32>,
}

//---------------------------------------------------------------------------//
//                              Implementations
//---------------------------------------------------------------------------//

/// Writes the theatres list: one theatre holding every army, fleet and region.
///
/// # Arguments
///
/// * `layout` - Record layouts.
/// * `plan` - Component ids.
/// * `key` - Theatre key (`map_data.esf`'s theatre).
/// * `mobiles` - Components of every army and fleet.
///
/// # Errors
///
/// Returns an error if the layout lacks the records.
pub fn write_theatres(layout: &StartposLayout, plan: &CaiPlan, key: &str, mobiles: Vec<u32>) -> Result<RecordNode> {
    let path = child_path(CAI_WORLD_PATH, "CAI_WORLD_THEATRES");
    let mut theatre = layout.builder(&child_path(&path, "CAI_THEATRE"))?;
    theatre.group()?
        .set_str("^+1", key)?
        .set_u32s("^+2", mobiles)?
        .set_u32s("^+4", plan.regions().iter().map(|region| region.id()).collect())?
        .set_u32("^+6", THEATRE_VALUE)?;

    let mut theatres = layout.builder(&path)?;
    theatres.push_group()?
        .set_record("CAI_BDI_COMPONENT", Some(component(layout, &path, plan.theatre(), vec![])?))?
        .set_record("CAI_THEATRE", Some(theatre.build()?))?;
    theatres.build()
}

/// Writes the regions list.
///
/// # Arguments
///
/// * `layout` - Record layouts.
/// * `plan` - Component ids.
/// * `map_regions` - Regions of the map, in map order.
/// * `nodes` - Pathfinder nodes, by region index.
/// * `links` - What each region refers to, in map order.
///
/// # Errors
///
/// Returns an error if the layout lacks the records.
pub fn write_regions(layout: &StartposLayout, plan: &CaiPlan, map_regions: &[MapRegion], nodes: &HashMap<usize, Vec<PathfinderNode>>, links: &[RegionLinks]) -> Result<RecordNode> {
    let path = child_path(CAI_WORLD_PATH, "CAI_WORLD_REGIONS");
    let region_path = child_path(&path, "CAI_REGION");
    let owned_path = child_path(&path, "OWNED_INDIRECT");
    let empty = RegionLinks::default();

    let mut regions = layout.builder(&path)?;
    for (index, (region, ids)) in map_regions.iter().zip(plan.regions()).enumerate() {
        let links = links.get(index).unwrap_or(&empty);
        let (x, y) = region_node(region, nodes.get(&index)).map(|node| (node.x(), node.y())).unwrap_or((0, 0));

        let mut owned = layout.builder(&owned_path)?;
        owned.group()?.set_u32("^+1", links.settlement)?;

        let mut record = layout.builder(&region_path)?;
        record.group()?
            .set_u32s("^+1", vec![plan.theatre()])?
            .set_u32s("^+2", ids.hlcis().to_vec())?
            .set_u32("^+3", links.settlement)?
            .set_u32("^+5", u32::from(x))?
            .set_u32("^+6", u32::from(y))?
            .set_u32s("^+8", links.boundaries.to_vec())?
            .set_bool("^+9", region.connects_to_other_kind())?
            .set_u32s("^+10", links.mobiles.to_vec())?
            .set_str("^+11", region.key())?
            .set_u32("^+12", links.world_id)?
            .set_u32("^+14", links.province)?
            .set_u32("^+15", links.sea_index)?
            .set_u32s("^+16", links.port_regions.to_vec())?
            .set_u32("^+17", links.port_sea)?
            .set_u32s("^+18", links.faction_seas.to_vec())?;

        regions.push_group()?
            .set_record("OWNED_INDIRECT", Some(owned.build()?))?
            .set_record("CAI_BDI_COMPONENT", Some(component(layout, &path, ids.id(), vec![])?))?
            .set_record("CAI_REGION", Some(record.build()?))?;
    }
    regions.build()
}

/// Writes the HLCIs list: each region's HLCIs, region by region.
///
/// # Arguments
///
/// * `layout` - Record layouts.
/// * `plan` - Component ids.
/// * `map_regions` - Regions of the map, in map order.
/// * `nodes` - Pathfinder nodes, by region index.
///
/// # Errors
///
/// Returns an error if the layout lacks the records.
pub fn write_hlcis(layout: &StartposLayout, plan: &CaiPlan, map_regions: &[MapRegion], nodes: &HashMap<usize, Vec<PathfinderNode>>) -> Result<RecordNode> {
    let path = child_path(CAI_WORLD_PATH, "CAI_WORLD_REGION_HLCIS");
    let hlci_path = child_path(&path, "CAI_REGION_HLCI");

    let mut hlcis = layout.builder(&path)?;
    for (index, (region, ids)) in map_regions.iter().zip(plan.regions()).enumerate() {
        let node = region_node(region, nodes.get(&index));
        let main = node.and_then(|node| region.hlci_of_area(node.area() as usize));
        for (hlci_index, ((group, areas), id)) in region.hlcis().into_iter().zip(ids.hlcis()).enumerate() {
            let (x, y) = node.filter(|_| main == Some(hlci_index)).map(|node| (node.x(), node.y())).unwrap_or((0, 0));
            let mut record = layout.builder(&hlci_path)?;
            record.group()?
                .set_u32("^+1", ids.id())?
                .set_u32("^+2", group)?
                .set_u32s("^+3", areas)?
                .set_u32("^+4", u32::from(x))?
                .set_u32("^+5", u32::from(y))?;

            hlcis.push_group()?
                .set_record("CAI_BDI_COMPONENT", Some(component(layout, &path, *id, vec![])?))?
                .set_record("CAI_REGION_HLCI", Some(record.build()?))?;
        }
    }
    hlcis.build()
}

/// Returns where the AI places each region (see [`region_node`]), or (0, 0) for regions without nodes.
///
/// # Arguments
///
/// * `map_regions` - Regions of the map, in map order.
/// * `nodes` - Pathfinder nodes, by region index.
pub fn region_positions(map_regions: &[MapRegion], nodes: &HashMap<usize, Vec<PathfinderNode>>) -> Vec<(u16, u16)> {
    map_regions.iter().enumerate()
        .map(|(index, region)| region_node(region, nodes.get(&index)).map(|node| (node.x(), node.y())).unwrap_or((0, 0)))
        .collect()
}

/// Returns the pathfinder node where the AI places a region: the one in its largest area, else its first one.
fn region_node(region: &MapRegion, nodes: Option<&Vec<PathfinderNode>>) -> Option<PathfinderNode> {
    let nodes = nodes?;
    let largest = region.largest_area();
    nodes.iter().find(|node| Some(node.area() as usize) == largest).or_else(|| nodes.first()).copied()
}
