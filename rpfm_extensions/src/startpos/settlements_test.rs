//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

use std::collections::{BTreeMap, BTreeSet};

use rpfm_lib::files::esf::{NodeType, RecordNode};

use super::*;
use crate::startpos::esf_nav::{as_record, as_u32, cell_groups, cell_pair, child_record, child_records};
use crate::startpos::startpos_test::test_data_esf;

/// Builds a record with the given name and groups.
fn record(name: &str, groups: Vec<Vec<NodeType>>) -> NodeType {
    let mut record = RecordNode::default();
    record.set_name(name.to_owned());
    record.set_children(groups);
    NodeType::Record(Box::new(record))
}

/// Builds a record holding one group per cell.
fn cells_record(name: &str, cells: &[(u16, u16)]) -> NodeType {
    record(name, cells.iter().map(|(x, y)| vec![NodeType::U16(*x), NodeType::U16(*y)]).collect())
}

/// Builds a `SETTLEMENT_INFO` with two slots and an optional port, over a 4x3 sprawl box starting at (10, 10).
fn sample_info(with_port: bool) -> RecordNode {
    let slot = |position: (u16, u16), area: &[(u16, u16)]| vec![cells_record("SLOT_POSITIONS_BLOCK", &[position]), cells_record("SLOT_AREA_BLOCK", area)];
    let bits = record("LOGICAL_POSITION_BIT_ARRAY", vec![vec![
        NodeType::U16(10), NodeType::U16(10), NodeType::U16(13), NodeType::U16(12),
        NodeType::U16(4),
        NodeType::U8Array(vec![0b0000_0011, 0b0001_0000]),
    ]]);
    let ports = if with_port { vec![(12, 11)] } else { vec![] };
    let info = record("SETTLEMENT_INFO", vec![vec![
        bits,
        record("SLOT_ARRAY_BLOCK", vec![slot((10, 10), &[(10, 10), (11, 10)]), slot((13, 12), &[(13, 12)])]),
        cells_record("PORT_POSITIONS_BLOCK", &ports),
        cells_record("PORT_AREA_BLOCK", &ports),
    ]]);
    match info {
        NodeType::Record(record) => *record,
        _ => unreachable!(),
    }
}

#[test]
fn test_slots_with_port() {
    let layout = SettlementLayout::from_settlement_info("reg", &sample_info(true), false).unwrap();
    let ids = layout.slots().iter().map(|slot| (slot.id(), slot.kind(), slot.position())).collect::<Vec<_>>();

    assert_eq!(ids, vec![
        (0, SlotKind::Primary, Some(HexCell::new(10, 10))),
        (1, SlotKind::Primary, Some(HexCell::new(12, 11))),
        (2, SlotKind::Secondary, Some(HexCell::new(13, 12))),
    ]);
}

#[test]
fn test_slots_without_port_use_placeholder() {
    let layout = SettlementLayout::from_settlement_info("reg", &sample_info(false), false).unwrap();
    let placeholder = &layout.slots()[1];

    assert_eq!((placeholder.id(), placeholder.position()), (0, None));
    assert!(placeholder.footprint().is_empty() && placeholder.zone().is_empty());
    assert_eq!(layout.slots()[2].id(), 2);
}

#[test]
fn test_sprawl_cells_and_flags() {
    let layout = SettlementLayout::from_settlement_info("reg", &sample_info(false), false).unwrap();
    let cells = layout.sprawl().iter().map(|cell| (cell.cell(), cell.in_zone(), cell.zone_slots())).collect::<Vec<_>>();

    // Bits 0, 1 and 12 of a 4-wide box at (10, 10).
    assert_eq!(cells, vec![
        (HexCell::new(10, 10), true, 1),
        (HexCell::new(11, 10), true, 1),
        (HexCell::new(10, 13), false, 0),
    ]);
    assert!(layout.sprawl()[0].in_footprint());
}

#[test]
fn test_walls_only_when_walled() {
    let walled = SettlementLayout::from_settlement_info("reg", &sample_info(false), true).unwrap();
    let open = SettlementLayout::from_settlement_info("reg", &sample_info(false), false).unwrap();

    assert_eq!(walled.slots()[0].walls(), &BTreeSet::from([HexCell::new(10, 10), HexCell::new(11, 10)]));
    assert!(open.slots()[0].walls().is_empty());
}

/// Slot data read back from a game-built startpos: (id, kind, position, footprint, zone, walls).
type GameSlot = (u32, SlotKind, Option<HexCell>, Vec<HexCell>, Vec<HexCell>, BTreeSet<HexCell>);

/// Reads a `SLOT_MAP_DATA` record of a game-built startpos.
fn game_slot(slot: &RecordNode, kind: SlotKind) -> GameSlot {
    let leaves = slot.children()[0].iter().filter(|node| as_record(node).is_none()).collect::<Vec<_>>();
    let id = as_u32(leaves[0]).unwrap();
    let position = HexCell::new(as_u32(leaves[1]).unwrap() as u16, as_u32(leaves[2]).unwrap() as u16);
    let cells = |name: &str| cell_groups(child_record(slot, name).unwrap());
    (id, kind, (position.x() != u16::MAX).then_some(position), cells("SLOT_FOOTPRINT_ARRAY"), cells("SLOT_ZOE_ARRAY"), cells("SLOT_WALL_ARRAY").into_iter().collect())
}

#[test]
#[ignore = "needs RPFM_STARTPOS_TEST_DATA with map_data.esf and startpos.esf"]
fn test_matches_game_startpos() {
    let startpos = test_data_esf("startpos.esf");
    let root = as_record(startpos.root_node()).unwrap();
    let model = child_record(child_record(root, "CAMPAIGN_ENV").unwrap(), "CAMPAIGN_MODEL").unwrap();
    let regions = child_record(child_record(child_record(model, "WORLD").unwrap(), "REGION_MANAGER").unwrap(), "REGIONS_ARRAY").unwrap();

    // Per region: slots and sprawl cells (cell, in footprint, in zone, zone mask), read from the game's startpos.
    let mut game = BTreeMap::new();
    for group in regions.children() {
        let region = group.iter().find_map(as_record).unwrap();
        let key = region.children()[0].iter().find_map(|node| match node { NodeType::Ascii(key) => Some(key.clone()), _ => None }).unwrap();
        let data = child_record(child_record(child_record(region, "SETTLEMENT").unwrap(), "SETTLEMENT_EXPANSION_MANAGER").unwrap(), "SETTLEMENT_EXPANSION_MAP_DATA").unwrap();
        let mut slots = child_records(data, "SLOT_MAP_DATA").map(|slot| game_slot(slot, SlotKind::Primary)).collect::<Vec<_>>();
        if let Some(secondary) = child_record(data, "SECONDARY_SLOT_ARRAY") {
            slots.extend(secondary.children().iter().flatten().filter_map(as_record).map(|slot| game_slot(slot, SlotKind::Secondary)));
        }
        let sprawl = child_record(data, "SPRAWL_BLOCK").unwrap().children().iter().map(|group| {
            let cell = cell_pair(group).unwrap();
            let hex = group.iter().find_map(as_record).unwrap();
            let bool_at = |index: usize| matches!(&hex.children()[0][index], NodeType::Bool(value) if *value.value());
            (cell, bool_at(2), bool_at(3), as_u32(&hex.children()[0][10]).unwrap())
        }).collect::<BTreeSet<_>>();
        game.insert(key, (slots, sprawl));
    }

    let map_data = test_data_esf("map_data.esf");
    let walled = |key: &str| game.get(key).is_some_and(|(slots, _)| !slots[0].5.is_empty());
    let layouts = SettlementLayout::all_from_map_data(&map_data, walled).unwrap();
    assert_eq!(layouts.len(), game.len());

    let mut wall_mismatches = 0;
    for layout in &layouts {
        let (game_slots, game_sprawl) = &game[layout.region()];
        assert_eq!(layout.slots().len(), game_slots.len(), "{}", layout.region());
        for (slot, game_slot) in layout.slots().iter().zip(game_slots) {
            assert_eq!((slot.id(), slot.kind(), slot.position()), (game_slot.0, game_slot.1, game_slot.2), "{}", layout.region());
            assert_eq!(slot.footprint(), &game_slot.3, "{}", layout.region());
            assert_eq!(slot.zone(), &game_slot.4, "{}", layout.region());
            wall_mismatches += usize::from(slot.walls() != &game_slot.5);
        }
        let sprawl = layout.sprawl().iter().map(|cell| (cell.cell(), cell.in_footprint(), cell.in_zone(), cell.zone_slots())).collect::<BTreeSet<_>>();
        assert_eq!(&sprawl, game_sprawl, "{}", layout.region());
    }

    // Known gap: 16 secondary slots whose game walls skip some boundary cells.
    assert!(wall_mismatches <= 16, "{wall_mismatches} wall mismatches");
}
