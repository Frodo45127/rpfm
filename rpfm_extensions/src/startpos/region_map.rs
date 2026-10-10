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
//! `REGIONS_DATA/REGIONS_BLOCK` (land regions, then seas and rivers). The meaning of the other bytes isn't known.
//! Verified on Attila's main campaign: every garrison stands on a cell of its own region. Prefer it over the map's
//! lookup image, which also paints seas the map data leaves empty.

use rpfm_lib::error::{RLibError, Result};
use rpfm_lib::files::esf::{ESF, NodeType};

use super::coords::HexCell;
use super::esf_nav::{as_record, as_u32, child_record, leaves};
use super::settlements::{region_data_records, region_key};

#[cfg(test)] #[path = "region_map_test.rs"] mod region_map_test;

/// Bytes stored per cell.
const BYTES_PER_CELL: usize = 7;

//---------------------------------------------------------------------------//
//                              Types
//---------------------------------------------------------------------------//

/// Region of each cell of the campaign map.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RegionMap {

    /// Region keys, in `REGIONS_BLOCK` order.
    keys: Vec<String>,

    /// Map width, in cells.
    width: u32,

    /// Map height, in cells.
    height: u32,

    /// Raw cell data, row by row.
    cells: Vec<u8>,
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
        let hex_map = as_record(map_data.root_node())
            .and_then(|root| child_record(root, "REGIONS_DATA"))
            .and_then(|regions| child_record(regions, "theatres"))
            .and_then(|theatres| child_record(theatres, "CAMPAIGN_THEATRE"))
            .and_then(|theatre| child_record(theatre, "HEX_MAP_DATA"))
            .ok_or_else(missing)?;

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

        Ok(Self { keys, width, height, cells })
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
        if u32::from(cell.x()) >= self.width || u32::from(cell.y()) >= self.height {
            return None;
        }
        let index = (u32::from(cell.y()) * self.width + u32::from(cell.x())) as usize * BYTES_PER_CELL;
        self.keys.get(usize::from(self.cells[index])).map(String::as_str)
    }
}
