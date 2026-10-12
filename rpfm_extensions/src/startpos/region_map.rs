//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Region of each cell of the campaign map, from `map_data.esf`
//! (`REGIONS_DATA/theatres/CAMPAIGN_THEATRE/HEX_MAP_DATA`).
//!
//! The data has 7 bytes per cell, row by row; the first byte is the index of the cell's region in
//! `REGIONS_DATA/REGIONS_BLOCK` (land regions, then seas and rivers), and the last two are hex-edge masks (the
//! startpos builds the road pieces of settlement sprawls from them). The meaning of the other bytes isn't known.
//! Verified on Attila's main campaign: every garrison stands on a cell of its own region. Prefer it over the map's
//! lookup image, which also paints seas the map data leaves empty.

use std::collections::{HashMap, HashSet};

use rpfm_lib::error::{RLibError, Result};
use rpfm_lib::files::esf::{ESF, NodeType};

use super::coords::{HexCell, MapScale};
use super::esf_nav::{as_record, as_u32, child_record, leaves};
use super::settlements::{region_data_records, region_key};

#[cfg(test)] #[path = "region_map_test.rs"] pub(crate) mod region_map_test;

/// Bytes stored per cell.
const BYTES_PER_CELL: usize = 7;

/// Position of the first hex-edge mask in a cell's bytes (the second one follows it).
const EDGE_MASKS_OFFSET: usize = 5;

/// Position of a cell's area byte: the index of the region's area the cell is in, shifted one bit left.
const AREA_OFFSET: usize = 1;

//---------------------------------------------------------------------------//
//                              Types
//---------------------------------------------------------------------------//

/// Region of each cell of the campaign map.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct RegionMap {

    /// Region keys, in `REGIONS_BLOCK` order.
    keys: Vec<String>,

    /// Map width, in cells.
    width: u32,

    /// Map height, in cells.
    height: u32,

    /// Raw cell data, row by row.
    cells: Vec<u8>,

    /// Size of the map in world units.
    world_size: (f32, f32),
}

//---------------------------------------------------------------------------//
//                              Implementations
//---------------------------------------------------------------------------//

impl RegionMap {

    /// Reads the region map of a campaign map.
    ///
    /// # Arguments
    ///
    /// * `map_data` - Decoded `map_data.esf` of the campaign map.
    ///
    /// # Errors
    ///
    /// Returns an error if the map data lacks the region list or the hex map, or the hex map is too short.
    pub fn from_map_data(map_data: &ESF) -> Result<Self> {
        let keys = region_data_records(map_data)?.into_iter()
            .map(|region| region_key(region).map(str::to_owned))
            .collect::<Result<Vec<_>>>()?;

        let missing = || RLibError::CampaignMapDataError("missing REGIONS_DATA/theatres/CAMPAIGN_THEATRE/HEX_MAP_DATA".to_owned());
        let theatre = as_record(map_data.root_node())
            .and_then(|root| child_record(root, "REGIONS_DATA"))
            .and_then(|regions| child_record(regions, "theatres"))
            .and_then(|theatres| child_record(theatres, "CAMPAIGN_THEATRE"))
            .ok_or_else(missing)?;
        let hex_map = child_record(theatre, "HEX_MAP_DATA").ok_or_else(missing)?;

        // The theatre's second position is its far corner: the size of the map in world units.
        let world_size = theatre.children().first()
            .and_then(|group| group.iter().filter_map(|node| match node { NodeType::Coord2d(corner) => Some((*corner.x(), *corner.y())), _ => None }).nth(1))
            .ok_or_else(|| RLibError::CampaignMapDataError("missing the size of REGIONS_DATA/theatres/CAMPAIGN_THEATRE".to_owned()))?;

        let group = hex_map.children().first().ok_or_else(missing)?;
        let mut values = leaves(group);
        let width = values.next().and_then(as_u32).ok_or_else(missing)?;
        let height = values.next().and_then(as_u32).ok_or_else(missing)?;
        let cells = match values.next() {
            Some(NodeType::U8Array(cells)) => cells.to_owned(),
            _ => return Err(missing()),
        };

        if cells.len() < width as usize * height as usize * BYTES_PER_CELL {
            return Err(RLibError::CampaignMapDataError(format!("the hex map has {} bytes, expected {width}x{height} cells", cells.len())));
        }

        Ok(Self { keys, width, height, cells, world_size })
    }

    /// Returns the size of the map in world units.
    pub fn world_size(&self) -> (f32, f32) {
        self.world_size
    }

    /// Returns the scale between the map's world and raw units.
    pub fn scale(&self) -> MapScale {
        MapScale::new((self.width, self.height), self.world_size)
    }

    /// Returns the region keys of the map, in `REGIONS_BLOCK` order.
    pub fn keys(&self) -> &[String] {
        &self.keys
    }

    /// Returns the key of the region a cell belongs to.
    ///
    /// # Returns
    ///
    /// `None` for cells outside the map or with an unknown region index.
    pub fn region_at(&self, cell: HexCell) -> Option<&str> {
        let index = self.cell_offset(cell)?;
        self.keys.get(usize::from(self.cells[index])).map(String::as_str)
    }

    /// Returns the index (in `REGIONS_BLOCK` order) of the region a cell belongs to, or `None` for cells outside the map.
    pub fn region_index_at(&self, cell: HexCell) -> Option<usize> {
        self.cell_offset(cell).map(|index| usize::from(self.cells[index]))
    }

    /// Returns the index of the area (in the region's `REGION_AREAS` order) a cell is in, or `None` for cells outside
    /// the map.
    pub fn area_at(&self, cell: HexCell) -> Option<usize> {
        self.cell_offset(cell).map(|index| usize::from(self.cells[index + AREA_OFFSET] >> 1))
    }

    /// Returns the size of the map in cells.
    pub fn size(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    /// Returns the two hex-edge masks of a cell (bit `n` stands for the `n`-th neighbour in the game's direction
    /// order), or zeros for cells outside the map.
    pub fn edge_masks(&self, cell: HexCell) -> [u8; 2] {
        self.cell_offset(cell)
            .map(|index| [self.cells[index + EDGE_MASKS_OFFSET], self.cells[index + EDGE_MASKS_OFFSET + 1]])
            .unwrap_or_default()
    }

    /// Returns the cells of every region, by region key.
    pub fn cells_by_region(&self) -> HashMap<&str, Vec<HexCell>> {
        let mut cells = HashMap::<&str, Vec<HexCell>>::new();
        for y in 0..self.height {
            for x in 0..self.width {
                let cell = HexCell::new(x as u16, y as u16);
                if let Some(region) = self.region_at(cell) {
                    cells.entry(region).or_default().push(cell);
                }
            }
        }
        cells
    }

    /// Returns every pair of different regions with neighbouring cells, each pair once with the keys in order.
    pub fn neighbour_pairs(&self) -> HashSet<(&str, &str)> {
        let mut pairs = HashSet::new();
        for y in 0..self.height {
            for x in 0..self.width {
                let cell = HexCell::new(x as u16, y as u16);
                let Some(region) = self.region_at(cell) else { continue };
                for neighbour in cell.neighbours() {
                    if let Some(other) = self.region_at(neighbour).filter(|other| *other != region) {
                        pairs.insert(if region < other { (region, other) } else { (other, region) });
                    }
                }
            }
        }
        pairs
    }

    /// Returns the position of a cell's first byte, or `None` for cells outside the map.
    fn cell_offset(&self, cell: HexCell) -> Option<usize> {
        if u32::from(cell.x()) >= self.width || u32::from(cell.y()) >= self.height {
            return None;
        }
        Some((u32::from(cell.y()) * self.width + u32::from(cell.x())) as usize * BYTES_PER_CELL)
    }
}
