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
use rpfm_lib::files::esf::{BoolNode, Coordinates2DNode, ESF, F32Node, I32Node, NodeType, U32Node};

use super::cai::world::{CaiContext, CaiInputs};
use super::layout::{COMPACT_I32, MAX_COMPACT_U32};
use super::missions::VictoryObjectives;
use super::trade::TradeNetwork;
use super::world::writer::forces::ForcePlan;
use super::world::writer::{write_world_with_plan, WorldInputs, WorldPlan};

mod diplomacy;
mod dynamic_resources;
mod map_data;
mod preopen;
mod trade;
mod victory;

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

    /// Victory objectives of the campaign (its `victory_objectives.txt`).
    pub victory_objectives: &'a [VictoryObjectives],

    /// Decoded `map_data.esf` of the campaign map.
    pub map_data: &'a ESF,

    /// Decoded `hlp_data.esf` of the campaign map.
    pub hlp_data: &'a ESF,

    /// Campaign key.
    pub campaign: &'a str,
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

    let world_record = write_world_with_plan(world, plan)?;
    let cai_inputs = CaiInputs {
        tables: world.tables,
        index: world.index,
        layout,
        world: &world_record,
        map_data: inputs.map_data,
        hlp_data: inputs.hlp_data,
        trade_splines: inputs.trade_network.splines().len(),
        campaign: inputs.campaign,
    };
    let cai = CaiContext::new(&cai_inputs)?.write_interface(&cai_inputs)?;

    let mut model = layout.builder(MODEL_PATH)?;
    model.group()?
        .set_record("WORLD", Some(world_record))?
        .set_record("CAI_INTERFACE", Some(cai))?
        .set_record("CAMPAIGN_TRADE_MANAGER", Some(trade::write_trade_manager(layout, world.tables, plan, inputs.trade_network)?))?
        .set_record("DIPLOMACY_MANAGER", Some(diplomacy::write_diplomacy_manager(layout, plan)?))?
        .set_record("CAMPAIGN_MAP_DATA", Some(map_data::write_map_data(world, plan)?))?
        .set_record("DYNAMIC_RESOURCES_MANAGER", Some(dynamic_resources::write_dynamic_resources_manager(layout, plan)?))?;

    let mut env = layout.builder(ENV_PATH)?;
    env.group()?
        .set_record("CAMPAIGN_SETUP", Some(preopen::write_campaign_setup(layout, plan)?))?
        .set_record("CAMPAIGN_MODEL", Some(model.build()?))?;

    let forces = ForcePlan::new(world.tables, world.index, plan)?;
    let mut root = layout.builder(ROOT_PATH)?;
    root.group()?
        .set_record("CAMPAIGN_PREOPEN_MAP_INFO", Some(preopen::write_preopen_map_info(world, plan, &forces, inputs.victory_objectives)?))?
        .set_record("CAMPAIGN_ENV", Some(env.build()?))?;

    let mut startpos = ESF::default();
    startpos.set_root_node(NodeType::Record(Box::new(root.build()?)));
    Ok(startpos)
}

/// Returns a boolean leaf, with the compact encoding the game uses for them.
pub(super) fn bool_node(value: bool) -> NodeType {
    let mut node = BoolNode::default();
    node.set_value(value);
    node.set_optimized(true);
    NodeType::Bool(node)
}

/// Returns an unsigned leaf, compact when the value fits.
pub(super) fn u32_node(value: u32) -> NodeType {
    let mut node = U32Node::default();
    node.set_value(value);
    node.set_optimized(value <= MAX_COMPACT_U32);
    NodeType::U32(node)
}

/// Returns a signed leaf, compact when the value fits.
pub(super) fn i32_node(value: i32) -> NodeType {
    let mut node = I32Node::default();
    node.set_value(value);
    node.set_optimized(COMPACT_I32.contains(&value));
    NodeType::I32(node)
}

/// Returns a float leaf, compact when it's zero.
pub(super) fn f32_node(value: f32) -> NodeType {
    let mut node = F32Node::default();
    node.set_value(value);
    node.set_optimized(value == 0.0);
    NodeType::F32(node)
}

/// Returns a `Coord2d` leaf.
pub(super) fn coordinates_node(x: f32, y: f32) -> NodeType {
    let mut node = Coordinates2DNode::default();
    node.set_x(x);
    node.set_y(y);
    NodeType::Coord2d(node)
}
