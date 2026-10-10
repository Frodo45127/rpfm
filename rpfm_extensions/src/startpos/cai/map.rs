//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! What the campaign AI reads from the map files: the regions of `map_data.esf` (land regions, then seas), their
//! areas and connections, and the high-level pathfinder nodes of `hlp_data.esf`.

use getset::{CopyGetters, Getters};

use std::collections::{HashMap, HashSet};

use rpfm_lib::error::{RLibError, Result};
use rpfm_lib::files::esf::{NodeType, RecordNode, ESF};

use crate::startpos::coords::HexCell;
use crate::startpos::esf_nav::{as_record, as_str, as_u32, cell_groups, child_record, leaves};
use crate::startpos::region_map::RegionMap;
use crate::startpos::settlements::{region_data_records, region_key};

#[cfg(test)] #[path = "map_test.rs"] mod map_test;

/// Bits of a pathfinder node id holding its region index. The bits above hold the index of the area it lies in.
const NODE_REGION_BITS: u32 = 9;

/// Connection lists of a map region, in the order the AI walks them.
const CONNECTIVITY_RECORDS: [&str; 2] = ["CONNECTIVITY_DIRECT_TRAVERSABLE_SAME", "CONNECTIVITY_DIRECT_TRAVERSABLE_OTHER"];

//---------------------------------------------------------------------------//
//                              Types
//---------------------------------------------------------------------------//

/// A region of the map (`REGION_DATA`), land or sea.
#[derive(Debug, Clone, PartialEq, Getters, CopyGetters)]
pub struct MapRegion {

    /// Region key.
    #[getset(get = "pub")]
    key: String,

    /// Connectivity group of each of the region's areas (`REGION_AREA_DATA`), in map order.
    #[getset(get = "pub")]
    area_groups: Vec<u32>,

    /// Size of each of the region's areas, in map order.
    #[getset(get = "pub")]
    area_sizes: Vec<u32>,

    /// Directly traversable connections to other regions (same kind of region first, then the other kind), as
    /// (region index, value) pairs.
    #[getset(get = "pub")]
    connections: Vec<(usize, u32)>,

    /// The region connects directly to a region of the other kind (land to sea or sea to land).
    #[getset(get_copy = "pub")]
    connects_to_other_kind: bool,

    /// The region has a settlement (land regions do, seas don't).
    #[getset(get_copy = "pub")]
    has_settlement: bool,

    /// Position of the settlement's port, if it has one (the first of `PORT_POSITIONS_BLOCK`).
    #[getset(get_copy = "pub")]
    port: Option<HexCell>,
}

/// The connectivity group of each cell (land masses and seas), from the group of the area it's in. Seas without areas
/// take the most common group of the sea areas around them.
#[derive(Debug, Clone)]
pub struct AreaGroups<'a> {

    /// Region and area of each cell.
    map: &'a RegionMap,

    /// Regions of the map, in map order.
    regions: &'a [MapRegion],

    /// Group of each sea without areas, by region index.
    sea_groups: HashMap<usize, u32>,
}

/// Reads the integer leaves of a flat list one by one.
struct Cursor<'a> {
    values: &'a [&'a NodeType],
    position: usize,
}

/// A high-level pathfinder node: a cell where the AI can place a region.
#[derive(Debug, Clone, Copy, PartialEq, Eq, CopyGetters)]
#[getset(get_copy = "pub")]
pub struct PathfinderNode {

    /// Index of the region's area the node lies in.
    area: u32,

    /// Cell column.
    x: u16,

    /// Cell row.
    y: u16,
}

//---------------------------------------------------------------------------//
//                              Implementations
//---------------------------------------------------------------------------//

impl MapRegion {

    /// Reads every region of a `map_data.esf`, in map order.
    ///
    /// # Errors
    ///
    /// Returns an error if the map lacks its regions block or a region is malformed.
    pub fn all_from_map_data(map_data: &ESF) -> Result<Vec<Self>> {
        region_data_records(map_data)?.into_iter().map(Self::from_region_data).collect()
    }

    /// Reads a `REGION_DATA` record.
    fn from_region_data(record: &RecordNode) -> Result<Self> {
        let key = region_key(record)?.to_owned();
        let mut area_groups = vec![];
        let mut area_sizes = vec![];
        for area in child_record(record, "REGION_AREAS").iter().flat_map(|areas| areas.children().iter().flatten().filter_map(as_record)) {
            let values = area.children().iter().flat_map(|group| leaves(group)).collect::<Vec<_>>();

            // After the type byte and the two corners: the group, four bounds, two lists, a cell, and the size.
            let group = values.get(3).and_then(|node| as_u32(node));
            let size = values.last().and_then(|node| as_u32(node));
            let (Some(group), Some(size)) = (group, size) else {
                return Err(RLibError::CampaignMapDataError(format!("region {key}: an area is malformed")));
            };
            area_groups.push(group);
            area_sizes.push(size);
        }

        let mut connections = vec![];
        for name in CONNECTIVITY_RECORDS {
            for group in child_record(record, name).iter().flat_map(|list| list.children()) {
                let mut values = leaves(group).filter_map(as_u32);
                if let (Some(region), Some(value)) = (values.next(), values.next()) {
                    connections.push((region as usize, value));
                }
            }
        }
        let connects_to_other_kind = child_record(record, CONNECTIVITY_RECORDS[1]).is_some_and(|list| !list.children().is_empty());
        let settlement = child_record(record, "SETTLEMENT_INFO");
        let has_settlement = settlement.is_some();
        let port = settlement.and_then(|settlement| child_record(settlement, "PORT_POSITIONS_BLOCK")).and_then(|ports| cell_groups(ports).first().copied());

        Ok(Self { key, area_groups, area_sizes, connections, connects_to_other_kind, has_settlement, port })
    }

    /// Returns the region's HLCIs: the distinct connectivity groups of its areas, in the order they first appear,
    /// each with the indexes of its areas.
    pub fn hlcis(&self) -> Vec<(u32, Vec<u32>)> {
        let mut hlcis: Vec<(u32, Vec<u32>)> = vec![];
        for (index, group) in self.area_groups.iter().enumerate() {
            let index = index as u32;
            match hlcis.iter_mut().find(|(hlci_group, _)| hlci_group == group) {
                Some((_, areas)) => areas.push(index),
                None => hlcis.push((*group, vec![index])),
            }
        }
        hlcis
    }

    /// Returns the index of the region's largest area (the first one on ties), if it has areas.
    pub fn largest_area(&self) -> Option<usize> {
        self.area_sizes.iter().enumerate().max_by_key(|(index, size)| (**size, std::cmp::Reverse(*index))).map(|(index, _)| index)
    }

    /// Returns the index of the HLCI holding an area.
    pub fn hlci_of_area(&self, area: usize) -> Option<usize> {
        let group = self.area_groups.get(area)?;
        self.hlcis().iter().position(|(hlci_group, _)| hlci_group == group)
    }
}

impl<'a> AreaGroups<'a> {

    /// Finds the groups of the seas without areas.
    ///
    /// # Arguments
    ///
    /// * `map` - Region and area of each cell.
    /// * `regions` - Regions of the map, in map order.
    pub fn new(map: &'a RegionMap, regions: &'a [MapRegion]) -> Self {
        let mut groups = Self { map, regions, sea_groups: HashMap::new() };
        let is_bare_sea = |region: usize| regions.get(region).is_some_and(|region| !region.has_settlement && region.area_groups.is_empty());

        // Seas next to other bare seas only get a group once those have one, so repeat until nothing changes.
        let (width, height) = map.size();
        loop {
            let mut counts: HashMap<usize, HashMap<u32, usize>> = HashMap::new();
            for y in 0..height as u16 {
                for x in 0..width as u16 {
                    let cell = HexCell::new(x, y);
                    let Some(region) = map.region_index_at(cell).filter(|region| is_bare_sea(*region) && !groups.sea_groups.contains_key(region)) else { continue };
                    for neighbour in cell.neighbours() {
                        let is_other_sea = map.region_index_at(neighbour).is_some_and(|other| other != region && regions.get(other).is_some_and(|other| !other.has_settlement));
                        if let Some(group) = groups.group_at(neighbour).filter(|_| is_other_sea) {
                            *counts.entry(region).or_default().entry(group).or_default() += 1;
                        }
                    }
                }
            }
            if counts.is_empty() {
                break;
            }
            for (region, counts) in counts {
                let group = counts.into_iter().max_by_key(|(group, count)| (*count, std::cmp::Reverse(*group))).map(|(group, _)| group).unwrap_or_default();
                groups.sea_groups.insert(region, group);
            }
        }
        groups
    }

    /// Returns the sea cell where the AI places a port: the sea cell closest to the port's position, searching
    /// neighbours ring by ring in the game's direction order. The game picks other cells for most ports (its rule
    /// isn't known), always within a few cells.
    pub fn port_cell(&self, port: HexCell) -> Option<HexCell> {
        let is_sea = |cell: HexCell| self.map.region_index_at(cell).and_then(|region| self.regions.get(region)).is_some_and(|region| !region.has_settlement);
        let mut seen = HashSet::from([port]);
        let mut ring = vec![port];
        while !ring.is_empty() {
            let mut next = vec![];
            for cell in ring {
                for neighbour in cell.neighbours() {
                    if self.map.region_index_at(neighbour).is_none() || !seen.insert(neighbour) {
                        continue;
                    }
                    if is_sea(neighbour) {
                        return Some(neighbour);
                    }
                    next.push(neighbour);
                }
            }
            ring = next;
        }
        None
    }

    /// Returns the group of a cell, or `None` for cells outside the map or in seas without a known group.
    pub fn group_at(&self, cell: HexCell) -> Option<u32> {
        let region = self.map.region_index_at(cell)?;
        let area = self.map.area_at(cell)?;
        self.regions.get(region)?.area_groups.get(area).copied().or_else(|| self.sea_groups.get(&region).copied())
    }
}

/// Returns the key of the map's theatre (`REGIONS_DATA/theatres/CAMPAIGN_THEATRE`'s string).
///
/// # Errors
///
/// Returns an error if the map lacks its theatre.
pub fn theatre_key(map_data: &ESF) -> Result<String> {
    as_record(map_data.root_node())
        .and_then(|root| child_record(root, "REGIONS_DATA"))
        .and_then(|regions| child_record(regions, "theatres"))
        .and_then(|theatres| child_record(theatres, "CAMPAIGN_THEATRE"))
        .and_then(|theatre| theatre.children().first())
        .and_then(|group| leaves(group).find_map(as_str))
        .map(str::to_owned)
        .ok_or_else(|| RLibError::CampaignMapDataError("missing the key of REGIONS_DATA/theatres/CAMPAIGN_THEATRE".to_owned()))
}

/// Reads the nodes of the high-level pathfinder of a `hlp_data.esf`, by region index.
///
/// The pathfinder record is a flat list: the region count, then for each region its index, its node count and its
/// nodes. Each node has its id, cell, a value, its edges (9 values each) and an edge-to-edge cost table.
///
/// # Errors
///
/// Returns an error if the file lacks the pathfinder or it ends early.
pub fn pathfinder_nodes(hlp_data: &ESF) -> Result<HashMap<usize, Vec<PathfinderNode>>> {
    let root = as_record(hlp_data.root_node()).ok_or_else(|| pathfinder_error("the root node is not a record"))?;
    let pathfinder = child_record(root, "CAI_HIGH_LEVEL_PATHFINDER").ok_or_else(|| pathfinder_error("missing CAI_HIGH_LEVEL_PATHFINDER"))?;
    let values = pathfinder.children().iter().flat_map(|group| leaves(group)).collect::<Vec<&NodeType>>();
    let mut cursor = Cursor { values: &values, position: 0 };

    let mut nodes = HashMap::new();
    for _ in 0..cursor.next()? {
        let region = cursor.next()? as usize;
        let mut region_nodes = vec![];
        for _ in 0..cursor.next()? {
            let id = cursor.next()?;
            let x = cursor.next()? as u16;
            let y = cursor.next()? as u16;
            cursor.next()?;
            let edges = cursor.next()? as usize;

            // Each edge has 9 values (cells, cost, target, index and two flags), then a cost per pair of edges.
            cursor.position += edges * 9 + edges * edges.saturating_sub(1);
            region_nodes.push(PathfinderNode { area: id >> NODE_REGION_BITS, x, y });
        }
        nodes.insert(region, region_nodes);
    }
    Ok(nodes)
}

impl Cursor<'_> {

    /// Returns the next leaf as an unsigned integer.
    fn next(&mut self) -> Result<u32> {
        let value = self.values.get(self.position).and_then(|node| as_u32(node)).ok_or_else(|| pathfinder_error("the node list ends early"))?;
        self.position += 1;
        Ok(value)
    }
}

/// Builds the error for a malformed pathfinder.
fn pathfinder_error(message: &str) -> RLibError {
    RLibError::CampaignMapDataError(format!("high-level pathfinder: {message}"))
}
