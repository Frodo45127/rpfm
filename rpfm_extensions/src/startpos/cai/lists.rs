//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Writer of the AI's smaller world lists: trade route segments, international trade routes, region boundaries,
//! provinces and the factions' parts of them, the factions' views of the seas, governorships and government posts.

use std::collections::HashMap;

use rpfm_lib::error::Result;
use rpfm_lib::files::esf::RecordNode;

use super::factions::{owned_direct, REBELS_WORLD_ID};
use super::map::MapRegion;
use super::plan::CaiPlan;
use super::world_view::WorldView;
use super::{component, CAI_WORLD_PATH};
use crate::startpos::layout::{child_path, StartposLayout};
use crate::startpos::world::CampaignIndex;

#[cfg(test)] #[path = "lists_test.rs"] mod lists_test;

//---------------------------------------------------------------------------//
//                              Implementations
//---------------------------------------------------------------------------//

/// Writes the trade route segments list: one per trade spline, holding its index.
///
/// # Arguments
///
/// * `layout` - Record layouts.
/// * `plan` - Component ids.
/// * `splines` - Number of trade splines of the map.
///
/// # Errors
///
/// Returns an error if the layout lacks the records.
pub fn write_trade_segments(layout: &StartposLayout, plan: &CaiPlan, splines: usize) -> Result<RecordNode> {
    let path = child_path(CAI_WORLD_PATH, "CAI_WORLD_TRADE_ROUTE_SEGMENTS");
    let mut list = layout.builder(&path)?;
    for index in 0..splines as u32 {
        let mut record = layout.builder(&child_path(&path, "CAI_TRADE_ROUTE_SEGMENT"))?;
        record.group()?.set_u32("^+1", index)?;
        list.push_group()?
            .set_record("CAI_BDI_COMPONENT", Some(component(layout, &path, plan.first_trade_segment() + index, vec![])?))?
            .set_record("CAI_TRADE_ROUTE_SEGMENT", Some(record.build()?))?;
    }
    list.build()
}

/// Writes the international trade routes list: one per route, in route id order, with its owner and the owner of the
/// other route of its agreement (routes come in pairs, one per side: 1 and 2, 3 and 4...).
///
/// # Arguments
///
/// * `layout` - Record layouts.
/// * `plan` - Component ids.
/// * `world` - The world's objects.
/// * `routes` - Each route's owner faction (runtime id) and route id.
///
/// # Errors
///
/// Returns an error if the layout lacks the records.
pub fn write_trade_routes(layout: &StartposLayout, plan: &CaiPlan, world: &WorldView, routes: &[(u32, u32)]) -> Result<RecordNode> {
    let path = child_path(CAI_WORLD_PATH, "CAI_WORLD_TRADE_ROUTES");
    let mut components = world.factions().iter().zip(plan.factions()).map(|(faction, ids)| (faction.id(), ids.faction())).collect::<HashMap<_, _>>();
    if let Some(rebels) = plan.factions().last() {
        components.insert(REBELS_WORLD_ID, rebels.faction());
    }
    let owners = routes.iter().map(|(owner, route)| (*route, components.get(owner).copied().unwrap_or_default())).collect::<HashMap<_, _>>();
    let mut ids = owners.keys().copied().collect::<Vec<_>>();
    ids.sort_unstable();

    let mut list = layout.builder(&path)?;
    for (index, route) in ids.iter().enumerate() {
        let partner = if route % 2 == 1 { route + 1 } else { route - 1 };
        let mut record = layout.builder(&child_path(&path, "CAI_TRADE_ROUTE"))?;
        record.group()?
            .set_u32("^+2", *route)?
            .set_u32("^+3", owners.get(&partner).copied().unwrap_or_default())?;
        list.push_group()?
            .set_record("CAI_BDI_COMPONENT", Some(component(layout, &path, plan.first_trade_route() + index as u32, vec![])?))?
            .set_record("OWNED_DIRECT", Some(owned_direct(layout, &path, owners[route])?))?
            .set_record("CAI_TRADE_ROUTE", Some(record.build()?))?;
    }
    list.build()
}

/// Writes the region boundaries list.
///
/// # Errors
///
/// Returns an error if the layout lacks the records.
pub fn write_boundaries(layout: &StartposLayout, plan: &CaiPlan) -> Result<RecordNode> {
    let path = child_path(CAI_WORLD_PATH, "CAI_WORLD_REGION_BOUNDARIES");
    let mut list = layout.builder(&path)?;
    for boundary in plan.boundaries() {
        let mut record = layout.builder(&child_path(&path, "CAI_REGION_BOUNDARY"))?;
        record.group()?
            .set_u32("^+1", plan.regions()[boundary.region()].id())?
            .set_u32("^+2", plan.regions()[boundary.other()].id())?
            .set_f32("^+3", boundary.value() as f32)?;
        list.push_group()?
            .set_record("CAI_BDI_COMPONENT", Some(component(layout, &path, boundary.id(), vec![])?))?
            .set_record("CAI_REGION_BOUNDARY", Some(record.build()?))?;
    }
    list.build()
}

/// Writes the provinces list and the list of the factions' parts of them.
///
/// # Arguments
///
/// * `layout` - Record layouts.
/// * `plan` - Component ids.
/// * `world` - The world's objects.
/// * `map_regions` - Regions of the map, in map order.
/// * `index` - The campaign's startpos rows, to find each province's major settlement.
///
/// # Returns
///
/// The provinces list and the factions' parts list.
///
/// # Errors
///
/// Returns an error if the layout lacks the records.
pub fn write_provinces(layout: &StartposLayout, plan: &CaiPlan, world: &WorldView, map_regions: &[MapRegion], index: &CampaignIndex) -> Result<(RecordNode, RecordNode)> {
    let lookups = Lookups::new(plan, world, map_regions);
    let provinces_path = child_path(CAI_WORLD_PATH, "CAI_PROVINCES");
    let parts_path = child_path(CAI_WORLD_PATH, "CAI_FACTION_PROVINCES");

    let mut provinces = layout.builder(&provinces_path)?;
    let mut parts = layout.builder(&parts_path)?;
    for (province, ids) in world.provinces().iter().zip(plan.provinces()) {
        for (part, id) in province.managers().iter().zip(ids.faction_provinces()) {

            // The main region swaps places with the first one.
            let mut regions = part.regions().iter().map(|region| lookups.region_by_id(*region)).collect::<Vec<_>>();
            let main = lookups.region_by_id(part.main_region());
            if let Some(position) = regions.iter().position(|region| *region == main) {
                regions.swap(0, position);
            }

            let mut record = layout.builder(&child_path(&parts_path, "CAI_FACTION_PROVINCE"))?;
            record.group()?
                .set_u32("^+1", part.id())?
                .set_u32("^+2", ids.province())?
                .set_u32s("^+3", regions)?
                .set_u32("^+4", lookups.region_by_id(part.main_region()))?;
            parts.push_group()?
                .set_record("OWNED_DIRECT", Some(owned_direct(layout, &parts_path, lookups.faction(part.faction()))?))?
                .set_record("CAI_BDI_COMPONENT", Some(component(layout, &parts_path, *id, vec![])?))?
                .set_record("CAI_FACTION_PROVINCE", Some(record.build()?))?;
        }

        // The province's main region is the one with its major settlement.
        let major = province.regions().iter().find(|key| index.settlement_of(key).is_some_and(|settlement| settlement.is_major()));
        let main_region = match major {
            Some(key) => lookups.region_by_key(key),
            None => province.managers().first().map(|part| lookups.region_by_id(part.main_region())).unwrap_or_default(),
        };
        let mut record = layout.builder(&child_path(&provinces_path, "CAI_PROVINCE"))?;
        record.group()?
            .set_u32("^+1", province.id())?
            .set_u32("^+2", main_region)?
            .set_u32s("^+3", province.regions().iter().map(|key| lookups.region_by_key(key)).collect())?
            .set_u32s("^+4", ids.faction_provinces().to_vec())?;
        provinces.push_group()?
            .set_record("CAI_BDI_COMPONENT", Some(component(layout, &provinces_path, ids.province(), vec![])?))?
            .set_record("CAI_PROVINCE", Some(record.build()?))?;
    }
    Ok((provinces.build()?, parts.build()?))
}

/// Writes the list of the factions' views of the seas: each sea's views, sea by sea.
///
/// # Arguments
///
/// * `layout` - Record layouts.
/// * `plan` - Component ids.
/// * `world` - The world's objects.
/// * `map_regions` - Regions of the map, in map order.
///
/// # Errors
///
/// Returns an error if the layout lacks the records.
pub fn write_faction_seas(layout: &StartposLayout, plan: &CaiPlan, world: &WorldView, map_regions: &[MapRegion]) -> Result<RecordNode> {
    let lookups = Lookups::new(plan, world, map_regions);
    let path = child_path(CAI_WORLD_PATH, "CAI_FACTION_SEA_REGIONS");
    let mut list = layout.builder(&path)?;
    for (sea, ids) in world.seas().iter().zip(plan.faction_seas()) {
        for (part, id) in sea.managers().iter().zip(ids) {
            let mut record = layout.builder(&child_path(&path, "CAI_FACTION_SEA_REGION"))?;
            record.group()?
                .set_u32("^+1", lookups.region_by_key(sea.key()))?
                .set_u32("^+2", part.id())?
                .set_u32s("^+3", part.regions().iter().map(|region| lookups.region_by_id(*region)).collect())?;
            list.push_group()?
                .set_record("OWNED_DIRECT", Some(owned_direct(layout, &path, lookups.faction(part.faction()))?))?
                .set_record("CAI_BDI_COMPONENT", Some(component(layout, &path, *id, vec![])?))?
                .set_record("CAI_FACTION_SEA_REGION", Some(record.build()?))?;
        }
    }
    list.build()
}

/// Writes the governorships list: one per world faction, over the theatre.
///
/// # Errors
///
/// Returns an error if the layout lacks the records.
pub fn write_governorships(layout: &StartposLayout, plan: &CaiPlan) -> Result<RecordNode> {
    let path = child_path(CAI_WORLD_PATH, "CAI_WORLD_GOVERNORSHIPS");
    let mut list = layout.builder(&path)?;
    for (id, faction) in plan.governorships().iter().zip(plan.factions()) {
        let mut record = layout.builder(&child_path(&path, "CAI_GOVERNORSHIP"))?;
        record.group()?.set_u32("^+1", plan.theatre())?;
        list.push_group()?
            .set_record("CAI_BDI_COMPONENT", Some(component(layout, &path, *id, vec![])?))?
            .set_record("OWNED_DIRECT", Some(owned_direct(layout, &path, faction.faction())?))?
            .set_record("CAI_GOVERNORSHIP", Some(record.build()?))?;
    }
    list.build()
}

/// Writes the government posts list: each faction's posts, with their holders.
///
/// # Errors
///
/// Returns an error if the layout lacks the records.
pub fn write_posts(layout: &StartposLayout, plan: &CaiPlan, world: &WorldView) -> Result<RecordNode> {
    let path = child_path(CAI_WORLD_PATH, "CAI_CHARACTER_POSTS");
    let mut list = layout.builder(&path)?;
    for (position, faction) in world.factions().iter().enumerate() {
        let characters = &plan.characters()[position];
        for ((post, holder), id) in faction.posts().iter().zip(&plan.posts()[position]) {
            let holder = faction.characters().iter().position(|character| *holder != 0 && character.id() == *holder).map(|index| characters[index].character()).unwrap_or_default();
            let mut record = layout.builder(&child_path(&path, "CAI_CHARACTER_POST"))?;
            record.group()?
                .set_u32("^+1", *post)?
                .set_u32("^+2", holder)?;
            list.push_group()?
                .set_record("OWNED_DIRECT", Some(owned_direct(layout, &path, plan.factions()[position].faction())?))?
                .set_record("CAI_BDI_COMPONENT", Some(component(layout, &path, *id, vec![])?))?
                .set_record("CAI_CHARACTER_POST", Some(record.build()?))?;
        }
    }
    list.build()
}

/// Component lookups by world ids and keys.
struct Lookups {

    /// Component of each region, by key.
    regions: HashMap<String, u32>,

    /// Key of each world region, by runtime id.
    region_keys: HashMap<u32, String>,

    /// Component of each faction, by world id.
    factions: HashMap<u32, u32>,
}

impl Lookups {

    /// Builds the lookups of a world.
    fn new(plan: &CaiPlan, world: &WorldView, map_regions: &[MapRegion]) -> Self {
        let regions = map_regions.iter().zip(plan.regions()).map(|(region, ids)| (region.key().to_owned(), ids.id())).collect();
        let region_keys = world.regions().iter().map(|region| (region.id(), region.key().to_owned())).collect();
        let mut factions = world.factions().iter().zip(plan.factions()).map(|(faction, ids)| (faction.id(), ids.faction())).collect::<HashMap<_, _>>();
        if let Some(rebels) = plan.factions().last() {
            factions.insert(REBELS_WORLD_ID, rebels.faction());
        }
        Self { regions, region_keys, factions }
    }

    /// Returns the component of a region by key, or 0.
    fn region_by_key(&self, key: &str) -> u32 {
        self.regions.get(key).copied().unwrap_or_default()
    }

    /// Returns the component of a region by world id, or 0.
    fn region_by_id(&self, id: u32) -> u32 {
        self.region_keys.get(&id).map(|key| self.region_by_key(key)).unwrap_or_default()
    }

    /// Returns the component of a faction by world id, or 0.
    fn faction(&self, id: u32) -> u32 {
        self.factions.get(&id).copied().unwrap_or_default()
    }
}
