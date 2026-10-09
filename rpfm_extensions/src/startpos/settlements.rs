//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Settlement layouts of a startpos (`REGION/SETTLEMENT/SETTLEMENT_EXPANSION_MANAGER/SETTLEMENT_EXPANSION_MAP_DATA`),
//! built from the campaign map's `map_data.esf` (`REGION_DATA/SETTLEMENT_INFO`).
//!
//! Verified against Attila's main campaign (186 settlements, 983 slots): slot ids, kinds, positions, footprints
//! and zones match the game-built startpos, and so do the sprawl cells and their footprint/zone flags. Not covered
//! yet: the order of the wall cells (the game stores a walk around the zone, with repeated cells; this module
//! returns the set of boundary cells, which matches 967 of 983 slots), the trailing value of each slot, and the
//! road/wall connection fields of the sprawl cells.

use getset::{CopyGetters, Getters};
use serde_derive::{Deserialize, Serialize};

use std::collections::{BTreeSet, HashSet};

use rpfm_lib::error::{RLibError, Result};
use rpfm_lib::files::esf::{ESF, NodeType, RecordNode};

use super::coords::HexCell;
use super::esf_nav::{as_record, as_str, as_u32, cell_groups, child_record, child_records, leaves};

#[cfg(test)] #[path = "settlements_test.rs"] mod settlements_test;

//---------------------------------------------------------------------------//
//                              Types
//---------------------------------------------------------------------------//

/// Layout of a settlement: its building slots and its sprawl cells.
#[derive(Debug, Clone, PartialEq, Default, Getters, Serialize, Deserialize)]
#[getset(get = "pub")]
pub struct SettlementLayout {

    /// Region key.
    region: String,

    /// Slots, in startpos order: main settlement, port (or placeholder), then the secondary slots.
    slots: Vec<SlotLayout>,

    /// Cells the settlement can sprawl over.
    sprawl: Vec<SprawlCell>,
}

/// Whether a slot is one of the two primary slots or a secondary one.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum SlotKind {

    /// Main settlement or port slot (`SLOT_MAP_DATA` directly under the expansion data).
    #[default]
    Primary,

    /// Extra slot (`SECONDARY_SLOT_ARRAY`).
    Secondary,
}

/// A building slot of a settlement.
#[derive(Debug, Clone, PartialEq, Eq, Default, CopyGetters, Getters, Serialize, Deserialize)]
pub struct SlotLayout {

    /// Slot id. The port placeholder of settlements without port uses 0.
    #[getset(get_copy = "pub")]
    id: u32,

    /// Primary or secondary slot.
    #[getset(get_copy = "pub")]
    kind: SlotKind,

    /// Cell the slot is anchored to. `None` for the port placeholder of settlements without port.
    #[getset(get_copy = "pub")]
    position: Option<HexCell>,

    /// Anchor cell and its neighbours, in the game's order.
    #[getset(get = "pub")]
    footprint: Vec<HexCell>,

    /// Zone of the slot (`SLOT_ZOE_ARRAY`), in map file order.
    #[getset(get = "pub")]
    zone: Vec<HexCell>,

    /// Zone cells with a neighbour outside the zone, for walled settlements. Empty otherwise.
    #[getset(get = "pub")]
    walls: BTreeSet<HexCell>,
}

/// A cell of the sprawl area of a settlement.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, CopyGetters, Serialize, Deserialize)]
#[getset(get_copy = "pub")]
pub struct SprawlCell {

    /// The cell.
    cell: HexCell,

    /// The cell is part of a slot footprint.
    in_footprint: bool,

    /// The cell is part of a slot zone.
    in_zone: bool,

    /// Bitmask of the slots whose zone contains the cell (`1 << slot index`).
    zone_slots: u32,
}

//---------------------------------------------------------------------------//
//                              Implementations
//---------------------------------------------------------------------------//

impl SettlementLayout {

    /// Builds the layout of every settlement of a campaign map.
    ///
    /// # Arguments
    ///
    /// * `map_data` - Decoded `map_data.esf` of the campaign map.
    /// * `is_walled` - Returns `true` for regions whose settlement has walls (major settlements in Attila).
    ///
    /// # Returns
    ///
    /// One layout per region with a settlement, in map file order.
    ///
    /// # Errors
    ///
    /// Fails if the map data doesn't have the expected records.
    pub fn all_from_map_data(map_data: &ESF, is_walled: impl Fn(&str) -> bool) -> Result<Vec<Self>> {
        let mut layouts = vec![];
        for region in region_data_records(map_data)? {
            let key = region_key(region)?;
            if let Some(info) = child_record(region, "SETTLEMENT_INFO") {
                layouts.push(Self::from_settlement_info(key, info, is_walled(key))?);
            }
        }
        Ok(layouts)
    }

    /// Builds the layout of a settlement from its `SETTLEMENT_INFO` record.
    ///
    /// # Arguments
    ///
    /// * `region` - Region key.
    /// * `info` - The region's `SETTLEMENT_INFO` record from `map_data.esf`.
    /// * `walled` - If the settlement has walls.
    ///
    /// # Errors
    ///
    /// Fails if the record doesn't have the expected children.
    pub fn from_settlement_info(region: &str, info: &RecordNode, walled: bool) -> Result<Self> {
        let map_slots = child_records(info, "SLOT_ARRAY_BLOCK")
            .flat_map(|block| block.children().iter())
            .map(|group| map_slot(region, group))
            .collect::<Result<Vec<_>>>()?;

        let (main_position, main_zone) = map_slots.first().cloned()
            .ok_or_else(|| map_error(region, "the settlement has no slots"))?;

        let port_position = child_record(info, "PORT_POSITIONS_BLOCK").and_then(|block| cell_groups(block).first().copied());
        let port_zone = child_record(info, "PORT_AREA_BLOCK").map(cell_groups).unwrap_or_default();

        let mut slots = vec![SlotLayout::new(0, SlotKind::Primary, Some(main_position), main_zone, walled)];
        slots.push(match port_position {
            Some(position) => SlotLayout::new(1, SlotKind::Primary, Some(position), port_zone, walled),
            None => SlotLayout::default(),
        });

        // Secondary slot ids start at 2 whether the settlement has a port or not.
        for (index, (position, zone)) in map_slots.into_iter().enumerate().skip(1) {
            slots.push(SlotLayout::new(index as u32 + 1, SlotKind::Secondary, Some(position), zone, walled));
        }

        let sprawl_cells = child_record(info, "LOGICAL_POSITION_BIT_ARRAY")
            .ok_or_else(|| map_error(region, "missing LOGICAL_POSITION_BIT_ARRAY"))
            .and_then(|bits| bit_array_cells(region, bits))?;

        Ok(Self {
            region: region.to_owned(),
            sprawl: sprawl(&sprawl_cells, &slots),
            slots,
        })
    }
}

impl SlotLayout {

    /// Creates a slot anchored to a cell, computing its footprint and walls.
    fn new(id: u32, kind: SlotKind, position: Option<HexCell>, zone: Vec<HexCell>, walled: bool) -> Self {
        let walls = if walled { boundary(&zone) } else { BTreeSet::new() };
        Self {
            id,
            kind,
            footprint: position.map(|position| position.footprint()).unwrap_or_default(),
            position,
            zone,
            walls,
        }
    }
}

/// Returns the `REGION_DATA` records of the map, in file order.
fn region_data_records(map_data: &ESF) -> Result<Vec<&RecordNode>> {
    let root = as_record(map_data.root_node()).ok_or_else(|| RLibError::CampaignMapDataError("the root node is not a record".to_owned()))?;
    let block = child_record(root, "REGIONS_DATA")
        .and_then(|regions_data| child_record(regions_data, "REGIONS_BLOCK"))
        .ok_or_else(|| RLibError::CampaignMapDataError("missing MAP_DATA/REGIONS_DATA/REGIONS_BLOCK".to_owned()))?;
    Ok(child_records(block, "REGION_DATA").collect())
}

/// Returns the key of a `REGION_DATA` record (its first string leaf).
fn region_key(region: &RecordNode) -> Result<&str> {
    region.children().first()
        .and_then(|group| leaves(group).find_map(as_str))
        .ok_or_else(|| RLibError::CampaignMapDataError("a REGION_DATA record has no key".to_owned()))
}

/// Reads a map slot (a `SLOT_ARRAY_BLOCK` group): its first candidate position and its zone.
fn map_slot(region: &str, group: &[NodeType]) -> Result<(HexCell, Vec<HexCell>)> {
    let record = |name: &str| group.iter().filter_map(as_record).find(|record| record.name() == name);
    let position = record("SLOT_POSITIONS_BLOCK")
        .and_then(|positions| cell_groups(positions).first().copied())
        .ok_or_else(|| map_error(region, "a slot has no position"))?;
    let zone = record("SLOT_AREA_BLOCK").map(cell_groups).unwrap_or_default();
    Ok((position, zone))
}

/// Decodes a `LOGICAL_POSITION_BIT_ARRAY`: bounding box, row width and an LSB-first, row-major bitmap.
fn bit_array_cells(region: &str, bits: &RecordNode) -> Result<Vec<HexCell>> {
    let group = bits.children().first().ok_or_else(|| map_error(region, "empty LOGICAL_POSITION_BIT_ARRAY"))?;
    let values = leaves(group).collect::<Vec<_>>();
    let number = |index: usize| values.get(index).and_then(|node| as_u32(node));
    let (x0, y0, width) = match (number(0), number(1), number(4)) {
        (Some(x0), Some(y0), Some(width)) if width > 0 => (x0, y0, width),
        _ => return Err(map_error(region, "malformed LOGICAL_POSITION_BIT_ARRAY header")),
    };
    let bytes = match values.get(5) {
        Some(NodeType::U8Array(bytes)) => bytes,
        _ => return Err(map_error(region, "LOGICAL_POSITION_BIT_ARRAY has no bitmap")),
    };

    let cells = bytes.iter().enumerate()
        .flat_map(|(byte_index, byte)| (0..8).filter(move |bit| byte & (1 << bit) != 0).map(move |bit| byte_index as u32 * 8 + bit))
        .map(|index| HexCell::new((x0 + index % width) as u16, (y0 + index / width) as u16))
        .collect();
    Ok(cells)
}

/// Computes the footprint and zone flags of every sprawl cell.
fn sprawl(cells: &[HexCell], slots: &[SlotLayout]) -> Vec<SprawlCell> {
    let footprint = slots.iter().flat_map(|slot| slot.footprint.iter().copied()).collect::<HashSet<_>>();
    let zones = slots.iter().map(|slot| slot.zone.iter().copied().collect::<HashSet<_>>()).collect::<Vec<_>>();
    cells.iter().map(|cell| {
        let zone_slots = zones.iter().enumerate()
            .filter(|(_, zone)| zone.contains(cell))
            .fold(0, |mask, (index, _)| mask | (1 << index));
        SprawlCell {
            cell: *cell,
            in_footprint: footprint.contains(cell),
            in_zone: zone_slots != 0,
            zone_slots,
        }
    }).collect()
}

/// Returns the cells of a zone with at least one neighbour outside the zone.
fn boundary(zone: &[HexCell]) -> BTreeSet<HexCell> {
    let cells = zone.iter().copied().collect::<HashSet<_>>();
    cells.iter()
        .filter(|cell| cell.neighbours().count() < 6 || cell.neighbours().any(|neighbour| !cells.contains(&neighbour)))
        .copied()
        .collect()
}

fn map_error(region: &str, message: &str) -> RLibError {
    RLibError::CampaignMapDataError(format!("region {region}: {message}"))
}
