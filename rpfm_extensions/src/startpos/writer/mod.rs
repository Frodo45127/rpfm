//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Writer of whole startposes (`CAMPAIGN_STARTPOS`): the world, and the records around it.
//!
//! Records without a generator yet keep the values of their layouts.

use rpfm_lib::error::Result;
use rpfm_lib::files::esf::{ESF, NodeType};

use super::world::writer::{write_world_with_plan, WorldInputs, WorldPlan};

#[cfg(test)] mod writer_test;

/// Path of the root record.
pub const ROOT_PATH: &str = "CAMPAIGN_STARTPOS";

/// Path of the campaign environment.
pub const ENV_PATH: &str = "CAMPAIGN_STARTPOS/CAMPAIGN_ENV";

/// Path of the campaign model.
pub const MODEL_PATH: &str = "CAMPAIGN_STARTPOS/CAMPAIGN_ENV/CAMPAIGN_MODEL";

//---------------------------------------------------------------------------//
//                              Implementations
//---------------------------------------------------------------------------//

/// Writes the startpos of a campaign.
///
/// # Arguments
///
/// * `inputs` - Tables, startpos rows, layouts and map data.
/// * `plan` - Order and runtime ids of the world's objects.
///
/// # Errors
///
/// Returns an error if a table, column or layout is missing.
pub fn write_startpos(inputs: &WorldInputs, plan: &WorldPlan) -> Result<ESF> {
    let layout = inputs.layout;

    let mut model = layout.builder(MODEL_PATH)?;
    model.group()?.set_record("WORLD", Some(write_world_with_plan(inputs, plan)?))?;

    let mut env = layout.builder(ENV_PATH)?;
    env.group()?.set_record("CAMPAIGN_MODEL", Some(model.build()?))?;

    let mut root = layout.builder(ROOT_PATH)?;
    root.group()?.set_record("CAMPAIGN_ENV", Some(env.build()?))?;

    let mut startpos = ESF::default();
    startpos.set_root_node(NodeType::Record(Box::new(root.build()?)));
    Ok(startpos)
}
