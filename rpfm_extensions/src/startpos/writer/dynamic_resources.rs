//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Writer of the dynamic resources manager (`DYNAMIC_RESOURCES_MANAGER`): one manager per region, without resources.
//!
//! The map's `dynamic_resources.esf` lists the farmland areas of each region, but the game only fills them while
//! playing. Attila's startpos leaves out 5 regions (deserts, likely without farmland areas) and has two empty
//! agriculture entries in two others; those regions are found through a hash of the map file that isn't known yet.

use rpfm_lib::error::Result;
use rpfm_lib::files::esf::RecordNode;

use crate::startpos::layout::{child_path, StartposLayout};
use crate::startpos::world::writer::WorldPlan;

/// Path of the dynamic resources manager.
const MANAGER_PATH: &str = "CAMPAIGN_STARTPOS/CAMPAIGN_ENV/CAMPAIGN_MODEL/DYNAMIC_RESOURCES_MANAGER";

//---------------------------------------------------------------------------//
//                              Implementations
//---------------------------------------------------------------------------//

/// Writes the dynamic resources manager of a campaign.
///
/// # Errors
///
/// Returns an error if the layout lacks the records.
pub fn write_dynamic_resources_manager(layout: &StartposLayout, plan: &WorldPlan) -> Result<RecordNode> {
    let mut managers = layout.builder(&child_path(MANAGER_PATH, "REGION_DYNAMIC_RESOURCES_MANAGERS_BLOCK"))?;
    for region in plan.regions() {
        managers.push_group()?.set_u32("^+1", plan.region_id(region.key()).unwrap_or_default())?;
    }

    let mut manager = layout.builder(MANAGER_PATH)?;
    manager.group()?.set_record("REGION_DYNAMIC_RESOURCES_MANAGERS_BLOCK", Some(managers.build()?))?;
    manager.build()
}
