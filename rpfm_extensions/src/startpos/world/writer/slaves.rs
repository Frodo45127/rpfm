//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Writer of the world's slaves (`WORLD/SLAVES`): the slave points of each region's settlement
//! (`start_pos_settlements.startpos_slave_points`), in region id order.

use rpfm_lib::error::Result;
use rpfm_lib::files::esf::RecordNode;

use super::{child, WorldInputs, WorldPlan, WORLD_PATH};
use crate::startpos::layout::child_path;

#[cfg(test)] #[path = "slaves_test.rs"] mod slaves_test;

/// Writes the slaves of the world.
///
/// # Arguments
///
/// * `inputs` - Tables, startpos rows and layouts.
/// * `plan` - Order and runtime ids of the regions.
///
/// # Errors
///
/// Returns an error if the layout lacks the records.
pub fn write_slaves(inputs: &WorldInputs, plan: &WorldPlan) -> Result<RecordNode> {
    let path = child_path(WORLD_PATH, "SLAVES");
    let mut block = child(inputs.layout, &path, "SLAVES_BLOCK")?;
    for region in plan.regions() {
        let points = inputs.index.settlement_of(region.key()).map(|settlement| *settlement.slave_points()).unwrap_or_default();
        block.push_group()?
            .set_u32("^+1", plan.region_id(region.key()).unwrap_or_default())?
            .set_u32("^+2", u32::try_from(points).unwrap_or_default())?;
    }

    let mut record = inputs.layout.builder(&path)?;
    record.group()?.set_record("SLAVES_BLOCK", Some(block.build()?))?;
    record.build()
}
