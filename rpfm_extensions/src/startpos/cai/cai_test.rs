//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Helpers shared by the AI writer's tests.

use std::collections::HashMap;

use rpfm_lib::files::esf::RecordNode;

use super::map::{pathfinder_nodes, MapRegion, PathfinderNode};
use crate::startpos::esf_nav::{as_record, child_record};
use crate::startpos::layout::{child_path, GroupView, StartposLayout};
use crate::startpos::startpos_test::{test_data_esf, test_startpos};

/// Returns the regions of the test map and their pathfinder nodes.
pub(crate) fn map_inputs() -> (Vec<MapRegion>, HashMap<usize, Vec<PathfinderNode>>) {
    let map_regions = MapRegion::all_from_map_data(&test_data_esf("map_data.esf")).unwrap();
    let nodes = pathfinder_nodes(&test_data_esf("hlp_data.esf")).unwrap();
    (map_regions, nodes)
}

/// Returns a list of the game startpos's `CAI_WORLD`.
pub(crate) fn game_cai_world_list(name: &str) -> &'static RecordNode {
    let mut record = as_record(test_startpos().root_node()).unwrap();
    for name in ["CAMPAIGN_ENV", "CAMPAIGN_MODEL", "CAI_INTERFACE", "CAI_WORLD", name] {
        record = child_record(record, name).unwrap();
    }
    record
}

/// Returns the first leaf of a group's child record holding a single value (like `OWNED_DIRECT`).
pub(crate) fn single_field_u32(layout: &StartposLayout, parent: &str, group: &GroupView, key: &str) -> u32 {
    layout.read(&child_path(parent, key), group.record(key).unwrap()).unwrap()[0].u32("^+1").unwrap()
}
