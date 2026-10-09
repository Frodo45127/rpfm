//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Helpers to navigate decoded ESF trees (map files and startposes).

use rpfm_lib::files::esf::{NodeType, RecordNode};

use super::coords::HexCell;

/// Returns the node as a record, if it is one.
pub(crate) fn as_record(node: &NodeType) -> Option<&RecordNode> {
    match node {
        NodeType::Record(record) => Some(record),
        _ => None,
    }
}

/// Returns the first child record with the given name, searching every group.
pub(crate) fn child_record<'a>(record: &'a RecordNode, name: &str) -> Option<&'a RecordNode> {
    record.children().iter()
        .flatten()
        .filter_map(as_record)
        .find(|child| child.name() == name)
}

/// Returns every child record with the given name, in order, searching every group.
pub(crate) fn child_records<'a>(record: &'a RecordNode, name: &'a str) -> impl Iterator<Item = &'a RecordNode> + 'a {
    record.children().iter()
        .flatten()
        .filter_map(as_record)
        .filter(move |child| child.name() == name)
}

/// Returns the non-record children of a group, in order.
pub(crate) fn leaves(group: &[NodeType]) -> impl Iterator<Item = &NodeType> {
    group.iter().filter(|node| as_record(node).is_none())
}

/// Returns the value of an unsigned integer leaf (`U8`, `U16` or `U32`).
pub(crate) fn as_u32(node: &NodeType) -> Option<u32> {
    match node {
        NodeType::U8(value) => Some(u32::from(*value)),
        NodeType::U16(value) => Some(u32::from(*value)),
        NodeType::U32(value) => Some(*value.value()),
        _ => None,
    }
}

/// Returns the value of a string leaf.
pub(crate) fn as_str(node: &NodeType) -> Option<&str> {
    match node {
        NodeType::Ascii(value) | NodeType::Utf16(value) => Some(value),
        _ => None,
    }
}

/// Reads a cell list stored as one group per cell, each with two `U16` leaves (x, y).
pub(crate) fn cell_groups(record: &RecordNode) -> Vec<HexCell> {
    record.children().iter().filter_map(|group| cell_pair(group)).collect()
}

/// Reads a cell stored as the first two `U16` leaves of a group.
pub(crate) fn cell_pair(group: &[NodeType]) -> Option<HexCell> {
    let mut values = leaves(group).filter_map(|node| match node {
        NodeType::U16(value) => Some(*value),
        _ => None,
    });
    Some(HexCell::new(values.next()?, values.next()?))
}
