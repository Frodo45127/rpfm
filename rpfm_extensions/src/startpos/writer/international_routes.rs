//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Writer of the international trade routes (`CAMPAIGN_TRADE_MANAGER/INTERNATIONAL_TRADE_ROUTES`).
//!
//! Each `start_pos_diplomacy` row with a trade agreement gives two routes, from each faction's capital to the other's:
//! first `faction1`'s, then `faction2`'s. Rows are taken by `faction1` in world order, then by `faction2` in world
//! order, and routes are numbered from 1 in that order. A route is a path of regions with the fewest steps over the
//! map's region and port routes, through regions of the two factions only, with at most one sea step.
//!
//! Verified on Attila's main campaign: every route exists with the game's owner, destination and shape. 37 of 42 paths
//! match (the rest have as many steps: the game's tie-break isn't known), and 34 of 42 ids (the game takes a faction's
//! agreements in its relationships' hash order). Routes with a sea step don't reproduce the float two of them have.

use std::collections::{HashMap, VecDeque};

use rpfm_lib::error::Result;
use rpfm_lib::files::esf::{NodeType, RecordNode};

use super::{bool_node, f32_node, u32_node, u32s_node};
use super::trade::faction_map_order;
use crate::startpos::layout::{child_path, StartposLayout};
use crate::startpos::tables::StartposTables;
use crate::startpos::trade::TradeNetwork;
use crate::startpos::world::CampaignIndex;
use crate::startpos::world::writer::WorldPlan;

#[cfg(test)] #[path = "international_routes_test.rs"] mod international_routes_test;

/// Path of the per-faction route lists.
const ROUTES_PATH: &str = "CAMPAIGN_STARTPOS/CAMPAIGN_ENV/CAMPAIGN_MODEL/CAMPAIGN_TRADE_MANAGER/INTERNATIONAL_TRADE_ROUTES";

/// Sea steps a route can have.
const MAX_SEA_STEPS: usize = 1;

//---------------------------------------------------------------------------//
//                              Types
//---------------------------------------------------------------------------//

/// An international trade route.
#[derive(Debug, Clone, PartialEq)]
struct Route {

    /// Route id.
    id: u32,

    /// Owner faction key.
    owner: String,

    /// Region ids of the path, from the owner's capital to the partner's.
    path: Vec<u32>,

    /// Position in the path of the step made by sea, if any.
    sea_step: Option<usize>,
}

/// Region and port routes of the trade network, as neighbours of each region (by index in the network's region
/// list), in file order: region routes first. Each neighbour says if the step goes by sea.
struct TradeGraph {
    neighbours: Vec<Vec<(usize, bool)>>,
}

//---------------------------------------------------------------------------//
//                              Implementations
//---------------------------------------------------------------------------//

impl TradeGraph {

    /// Builds the graph of a trade network.
    fn new(network: &TradeNetwork) -> Self {
        let mut neighbours = vec![vec![]; network.regions().len()];
        for (routes, sea) in [(network.region_routes(), false), (network.port_routes(), true)] {
            for route in routes {
                let (from, to) = (*route.from() as usize, *route.to() as usize);
                for (a, b) in [(from, to), (to, from)] {
                    if let Some(list) = neighbours.get_mut(a) {
                        if !list.contains(&(b, sea)) {
                            list.push((b, sea));
                        }
                    }
                }
            }
        }
        Self { neighbours }
    }

    /// Returns a path with the fewest steps between two regions through the allowed ones, with at most
    /// [`MAX_SEA_STEPS`] sea steps, as the regions and whether each step goes by sea.
    fn path(&self, from: usize, to: usize, allowed: impl Fn(usize) -> bool) -> Option<(Vec<usize>, Vec<bool>)> {

        // States are (region, sea steps so far), so a path by sea doesn't block a later one by land.
        let mut previous = HashMap::from([((from, 0), None)]);
        let mut queue = VecDeque::from([(from, 0)]);
        while let Some((region, seas)) = queue.pop_front() {
            if region == to {
                let mut path = vec![];
                let mut state = Some((region, seas));
                while let Some(current) = state {
                    path.push(current);
                    state = previous[&current];
                }
                path.reverse();
                let steps = path.windows(2).map(|pair| pair[1].1 > pair[0].1).collect();
                return Some((path.into_iter().map(|(region, _)| region).collect(), steps));
            }

            for &(next, sea) in self.neighbours.get(region)? {
                let state = (next, seas + usize::from(sea));
                if state.1 <= MAX_SEA_STEPS && allowed(next) && !previous.contains_key(&state) {
                    previous.insert(state, Some((region, seas)));
                    queue.push_back(state);
                }
            }
        }
        None
    }
}

/// Writes the international trade routes of each faction: one entry per faction, with the routes it owns.
///
/// # Arguments
///
/// * `layout` - Record layouts of the game.
/// * `tables` - Tables with `start_pos_diplomacy`.
/// * `index` - The campaign's startpos rows, for capitals and region owners.
/// * `plan` - Factions and regions of the world, with their runtime ids.
/// * `network` - Trade network of the campaign map.
///
/// # Errors
///
/// Returns an error if a table, column or layout is missing.
pub fn write_international_routes(layout: &StartposLayout, tables: &StartposTables, index: &CampaignIndex, plan: &WorldPlan, network: &TradeNetwork) -> Result<RecordNode> {
    let routes = plan_routes(tables, index, plan, network)?;
    let resources = tables.table("resources")?.rows().count();

    let array_path = child_path(ROUTES_PATH, "FACTION_INTERNATIONAL_TRADE_ROUTES_ARRAY");
    let route_path = child_path(&array_path, "INTERNATIONAL_TRADE_ROUTE");
    let mut entries = layout.builder(ROUTES_PATH)?;
    for faction in faction_map_order(plan) {
        let mut array = layout.builder(&array_path)?;
        for route in routes.iter().filter(|route| route.owner == *faction.key()) {
            let mut children = vec![u32_node(route.id), u32s_node(route.path.clone()), bool_node(route.sea_step.is_some())];
            if let Some(step) = route.sea_step {
                children.extend([u32_node(step as u32), u32_node(step as u32 + 1)]);
            }

            // Land routes store their step count; the game's value for routes by sea isn't known.
            let steps = if route.sea_step.is_some() { 0.0 } else { (route.path.len() - 1) as f32 };
            children.extend([u32s_node(vec![0; resources]), u32_node(0), f32_node(steps)]);
            children.push(NodeType::Record(Box::new(layout.default_record(&child_path(&route_path, "COMMERCE_RAIDS"))?)));
            children.push(u32_node(0));
            array.push_children(vec![NodeType::Record(Box::new(layout.raw_record(&route_path, children)?))]);
        }

        entries.push_group()?
            .set_u32("^+1", plan.faction_id(faction.key()).unwrap_or_default())?
            .set_record("FACTION_INTERNATIONAL_TRADE_ROUTES_ARRAY", Some(array.build()?))?;
    }
    entries.build()
}

/// Works out the routes of the trade agreements, in id order.
fn plan_routes(tables: &StartposTables, index: &CampaignIndex, plan: &WorldPlan, network: &TradeNetwork) -> Result<Vec<Route>> {
    let mut agreements = vec![];
    for row in tables.table("start_pos_diplomacy")?.rows() {
        if !row.bool("grants_trade_agreement")? {
            continue;
        }
        let (Some(first), Some(second)) = (index.faction_by_id(&row.str("faction1")?), index.faction_by_id(&row.str("faction2")?)) else { continue };
        let position = |key: &str| plan.factions().iter().position(|faction| faction.key() == key);
        if let (Some(first_position), Some(second_position)) = (position(first.key()), position(second.key())) {
            agreements.push(((first_position, second_position), first.key().to_owned(), second.key().to_owned()));
        }
    }
    agreements.sort_by_key(|(positions, _, _)| *positions);

    let graph = TradeGraph::new(network);
    let network_index = network.regions().iter().enumerate().map(|(position, key)| (key.as_str(), position)).collect::<HashMap<_, _>>();
    let capital = |faction: &str| index.capital_of(faction).and_then(|region| network_index.get(region.key().as_str()).copied());
    let owner = |position: usize| network.regions().get(position).and_then(|key| index.region(key)).and_then(|region| region.owner().as_deref());

    let mut routes = vec![];
    for (_, first, second) in agreements {
        for (from, to) in [(&first, &second), (&second, &first)] {
            let (Some(start), Some(end)) = (capital(from), capital(to)) else { continue };
            let Some((path, steps)) = graph.path(start, end, |region| owner(region).is_some_and(|owner| owner == from || owner == to)) else { continue };
            let path = path.iter().map(|&region| plan.region_id(&network.regions()[region]).unwrap_or_default()).collect();
            routes.push(Route { id: routes.len() as u32 + 1, owner: from.to_owned(), path, sea_step: steps.iter().position(|sea| *sea) });
        }
    }
    Ok(routes)
}
