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

use super::trade::TradeNetwork;
use super::world::writer::{write_world_with_plan, WorldInputs, WorldPlan};

mod trade;

#[cfg(test)] mod writer_test;

/// Path of the root record.
pub const ROOT_PATH: &str = "CAMPAIGN_STARTPOS";

/// Path of the campaign environment.
pub const ENV_PATH: &str = "CAMPAIGN_STARTPOS/CAMPAIGN_ENV";

/// Path of the campaign model.
pub const MODEL_PATH: &str = "CAMPAIGN_STARTPOS/CAMPAIGN_ENV/CAMPAIGN_MODEL";

//---------------------------------------------------------------------------//
//                              Types
//---------------------------------------------------------------------------//

/// What a startpos is built from.
#[derive(Clone, Copy)]
pub struct StartposInputs<'a> {

    /// Inputs of the world: tables, startpos rows, layouts and map data.
    pub world: &'a WorldInputs<'a>,

    /// Trade network of the campaign map.
    pub trade_network: &'a TradeNetwork,
}

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
pub fn write_startpos<'a>(inputs: &StartposInputs<'a>, plan: &WorldPlan<'a>) -> Result<ESF> {
    let world = inputs.world;
    let layout = world.layout;

    let mut model = layout.builder(MODEL_PATH)?;
    model.group()?
        .set_record("WORLD", Some(write_world_with_plan(world, plan)?))?
        .set_record("CAMPAIGN_TRADE_MANAGER", Some(trade::write_trade_manager(layout, world.tables, plan, inputs.trade_network)?))?;

    let mut env = layout.builder(ENV_PATH)?;
    env.group()?.set_record("CAMPAIGN_MODEL", Some(model.build()?))?;

    let mut root = layout.builder(ROOT_PATH)?;
    root.group()?.set_record("CAMPAIGN_ENV", Some(env.build()?))?;

    let mut startpos = ESF::default();
    startpos.set_root_node(NodeType::Record(Box::new(root.build()?)));
    Ok(startpos)
}
