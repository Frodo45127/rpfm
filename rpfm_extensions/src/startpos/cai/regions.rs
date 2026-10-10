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
use super::world_view::WorldView;
use crate::startpos::coords::HexCell;
use crate::startpos::region_map::RegionMap;
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

    /// Components of the factions with this region among their victory conditions.
    victory_factions: Vec<u32>,
}

/// What the regions' links are built from.
#[derive(Debug, Clone, Copy)]
pub struct LinkInputs<'a> {

    /// Component ids.
    pub plan: &'a CaiPlan,

    /// The world's objects.
    pub world: &'a WorldView,

    /// Regions of the map, in map order.
    pub map_regions: &'a [MapRegion],

    /// Region of each cell.
    pub region_map: &'a RegionMap,

    /// Each mobile's component and cell, in creation order.
    pub mobiles: &'a [(u32, HexCell)],

    /// Port cell of each garrison residence's settlement.
    pub ports: &'a HashMap<u32, HexCell>,

    /// Components of the factions with victory conditions on each region, by region key.
    pub victory_factions: &'a HashMap<String, Vec<u32>>,
}

//---------------------------------------------------------------------------//
//                              Implementations
//---------------------------------------------------------------------------//

/// Returns what each map region refers to beyond the map, in map order.
pub fn region_links(inputs: &LinkInputs) -> Vec<RegionLinks> {
    let plan = inputs.plan;
    let world = inputs.world;
    let mut links = vec![RegionLinks::default(); inputs.map_regions.len()];

    for boundary in plan.boundaries() {
        for region in [boundary.region(), boundary.other()] {
            links[region].boundaries.push(boundary.id());
        }
    }
    for (mobile, cell) in inputs.mobiles {
        if let Some(region) = inputs.region_map.region_index_at(*cell).filter(|region| *region < links.len()) {
            links[region].mobiles.push(*mobile);
        }
    }

    let index_of = inputs.map_regions.iter().enumerate().map(|(index, region)| (region.key().as_str(), index)).collect::<HashMap<_, _>>();
    for (index, region) in inputs.map_regions.iter().enumerate() {
        let link = &mut links[index];
        link.victory_factions = inputs.victory_factions.get(region.key()).cloned().unwrap_or_default();
        if let Some(settlement) = plan.settlements().get(&index) {
            link.settlement = settlement.settlement();
        }
        if let Some(world_region) = world.regions().iter().find(|world_region| world_region.key() == region.key()) {
            link.world_id = world_region.id();
        }
        if let Some(province) = world.provinces().iter().zip(plan.provinces()).find_map(|(province, ids)| province.regions().contains(region.key()).then(|| ids.province())) {
            link.province = province;
        }
        if let Some((sea, ids)) = world.seas().iter().zip(plan.faction_seas()).find(|(sea, _)| sea.key() == region.key()) {
            link.sea_index = sea.id();
            link.faction_seas = ids.to_vec();
        }
    }

    // Each port opens to the sea of its cell. Seas list their ports by world region id.
    let mut world_regions = world.regions().iter().collect::<Vec<_>>();
    world_regions.sort_by_key(|world_region| world_region.id());
    for world_region in world_regions {
        let (Some(cell), Some(land)) = (inputs.ports.get(&world_region.residence()), index_of.get(world_region.key().as_str())) else { continue };
        let Some(sea) = inputs.region_map.region_index_at(*cell).filter(|sea| *sea < links.len()) else { continue };
        links[*land].port_sea = plan.regions()[sea].id();
        links[sea].port_regions.push(plan.regions()[*land].id());
    }
    links
}

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
            .set_u32s("^+13", links.victory_factions.to_vec())?
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
