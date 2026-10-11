//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Writer of the factions' spying state (`WORLD/SPYING_ARRAY`): one entry per faction, in world order, with nothing
//! spied yet.

use rpfm_lib::error::Result;
use rpfm_lib::files::esf::RecordNode;

use super::{WorldInputs, WorldPlan, WORLD_PATH};
use crate::startpos::layout::child_path;

#[cfg(test)] #[path = "spying_test.rs"] mod spying_test;

/// Writes the spying state of the factions.
///
/// # Arguments
///
/// * `inputs` - Tables, startpos rows and layouts.
/// * `plan` - Order of the factions.
///
/// # Errors
///
/// Returns an error if the layout lacks the records.
pub fn write_spying_array(inputs: &WorldInputs, plan: &WorldPlan) -> Result<RecordNode> {
    let path = child_path(WORLD_PATH, "SPYING_ARRAY");
    let spying = inputs.layout.default_record(&child_path(&path, "CAMPAIGN_SPYING"))?;
    let mut array = inputs.layout.builder(&path)?;
    for faction in plan.factions() {
        array.push_group()?
            .set_str("^+1", faction.key())?
            .set_record("CAMPAIGN_SPYING", Some(spying.clone()))?;
    }
    array.build()
}
