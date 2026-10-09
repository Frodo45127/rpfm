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
