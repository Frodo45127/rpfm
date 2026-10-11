//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Writer of the trade manager (`CAMPAIGN_TRADE_MANAGER`): the trade network of the map, the tradeable resources and
//! the trade routes of each faction.
//!
//! Factions start without domestic trade routes; the international ones come from the starting trade agreements.

use std::collections::HashMap;

use rpfm_lib::error::Result;
use rpfm_lib::files::esf::{NodeType, RecordNode};

use super::{bool_node, coordinates_node, f32_node, u32_node};
use super::international_routes::write_international_routes;
use crate::startpos::layout::{child_path, StartposLayout};
use crate::startpos::tables::StartposTables;
use crate::startpos::trade::{NetworkRoute, NetworkSpline, TradeNetwork};
use crate::startpos::world::CampaignIndex;
use crate::startpos::world::writer::WorldPlan;

/// Path of the trade manager.
const TRADE_PATH: &str = "CAMPAIGN_STARTPOS/CAMPAIGN_ENV/CAMPAIGN_MODEL/CAMPAIGN_TRADE_MANAGER";

/// Value of each unit of a resource's baseline price. Fitted on Attila's main campaign (10000 / 177); the source of
/// the 177 isn't known.
const RESOURCE_VALUE_PER_PRICE: f32 = 10000.0 / 177.0;

/// Last leaf of every spline. Its meaning isn't known.
const SPLINE_TRAILER: u32 = 1000;

//---------------------------------------------------------------------------//
//                              Implementations
//---------------------------------------------------------------------------//

/// Writes the trade manager of a campaign.
///
/// # Arguments
///
/// * `layout` - Record layouts of the game.
/// * `tables` - Tables with `resources`, `commodities` and `start_pos_diplomacy`.
/// * `index` - The campaign's startpos rows.
/// * `plan` - Factions and regions of the world, with their runtime ids.
/// * `network` - Trade network of the campaign map.
///
/// # Errors
///
/// Returns an error if a table, column or layout is missing.
pub fn write_trade_manager(layout: &StartposLayout, tables: &StartposTables, index: &CampaignIndex, plan: &WorldPlan, network: &TradeNetwork) -> Result<RecordNode> {
    let mut manager = layout.builder(TRADE_PATH)?;
    manager.group()?
        .set_strings("^+1", network.regions().clone())?
        .set_record("SPLINES", Some(write_splines(layout, network.splines())?))?
        .set_record("REGION_ROUTES", Some(write_routes(layout, "REGION_ROUTES", network.region_routes())?))?
        .set_record("PORT_ROUTES", Some(write_routes(layout, "PORT_ROUTES", network.port_routes())?))?
        .set_record("RESOURCES", Some(write_resources(layout, tables)?))?
        .set_record("DOMESTIC_TRADE_ROUTES", Some(write_faction_routes(layout, plan, "DOMESTIC_TRADE_ROUTES")?))?
        .set_record("INTERNATIONAL_TRADE_ROUTES", Some(write_international_routes(layout, tables, index, plan, network)?))?;
    manager.build()
}

/// Writes the splines of the trade network (`SPLINES`): flags, segment count, then each segment's control points and
/// speed table, the segment lengths, where each segment starts, the total length and a constant.
fn write_splines(layout: &StartposLayout, splines: &[NetworkSpline]) -> Result<RecordNode> {
    let path = child_path(TRADE_PATH, "SPLINES");
    let raids = layout.default_record(&child_path(&path, "COMMERCE_RAIDS"))?;
    let mut array = layout.builder(&path)?;
    for spline in splines {
        let mut children = spline.flags().iter().map(|&flag| bool_node(flag)).collect::<Vec<_>>();
        children.push(u32_node(spline.segments().len() as u32));
        for (points, table) in spline.segments().iter().zip(spline.speed_tables()) {
            children.extend(points.iter().map(|&(x, y)| coordinates_node(x, y)));
            children.push(NodeType::F32Array(table.clone()));
        }

        let starts = spline.lengths().iter().scan(0.0, |start, length| {
            let current = *start;
            *start += length;
            Some(current)
        }).collect();
        children.push(NodeType::F32Array(spline.lengths().clone()));
        children.push(NodeType::F32Array(starts));
        children.push(f32_node(*spline.total_length()));
        children.push(u32_node(SPLINE_TRAILER));
        children.push(NodeType::Record(Box::new(raids.clone())));
        array.push_children(children);
    }
    array.build()
}

/// Writes routes between regions or ports: origin, destination and the spline edges to follow.
fn write_routes(layout: &StartposLayout, key: &str, routes: &[NetworkRoute]) -> Result<RecordNode> {
    let mut array = layout.builder(&child_path(TRADE_PATH, key))?;
    for route in routes {
        array.push_group()?
            .set_u32("^+1", *route.from())?
            .set_u32("^+2", *route.to())?
            .set_u32s("^+3", route.edges().iter().map(|edge| edge.value()).collect())?;
    }
    array.build()
}

/// Writes the tradeable resources (`RESOURCES`): every `resources` row, with the baseline price of its commodity (1
/// without one) as its supply and demand, and that price's value.
fn write_resources(layout: &StartposLayout, tables: &StartposTables) -> Result<RecordNode> {
    let mut prices = HashMap::new();
    for row in tables.table("commodities")?.rows() {
        prices.insert(row.str("key")?.into_owned(), row.f32("baseline_price_per_unit")? as u32);
    }

    let mut array = layout.builder(&child_path(TRADE_PATH, "RESOURCES"))?;
    for row in tables.table("resources")?.rows() {
        let key = row.str("key")?;
        let price = prices.get(key.as_ref()).copied().unwrap_or(1);
        array.push_group()?
            .set_str("^+1", &key)?
            .set_u32("^+2", price)?
            .set_f32("^+3", price as f32 * RESOURCE_VALUE_PER_PRICE)?
            .set_u32("^+4", price)?
            .set_u32("^+5", price)?;
    }
    array.build()
}

/// Writes one entry per faction, with its id and no routes, for `DOMESTIC_TRADE_ROUTES`.
fn write_faction_routes(layout: &StartposLayout, plan: &WorldPlan, key: &str) -> Result<RecordNode> {
    let mut array = layout.builder(&child_path(TRADE_PATH, key))?;
    for faction in plan.factions() {
        array.push_group()?.set_u32("^+1", plan.faction_id(faction.key()).unwrap_or_default())?;
    }
    array.build()
}
