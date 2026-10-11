//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Writer of the AI's mirror of the world (`CAI_WORLD`), from what the AI writer works out of a campaign.

use getset::Getters;

use std::collections::HashMap;

use rpfm_lib::error::Result;
use rpfm_lib::files::esf::{RecordNode, ESF};

use super::factions::{capitals, faction_links, victory_factions, write_factions, write_technologies, write_technology_trees};
use super::lists::{write_boundaries, write_faction_seas, write_governorships, write_posts, write_provinces, write_trade_segments};
use super::map::{pathfinder_nodes, theatre_key, AreaGroups, MapRegion, PathfinderNode};
use super::mobiles::{mobile_cells, write_characters, write_mobiles, write_units, MobileInputs};
use super::plan::{CaiPlan, PlanInputs};
use super::regions::{region_links, region_positions, write_hlcis, write_regions, write_theatres, LinkInputs};
use super::settlements::{port_cells, settlement_components, write_building_slots, write_settlements};
use super::technologies::faction_technologies;
use super::world_view::WorldView;
use super::{component, CAI_WORLD_PATH};
use crate::startpos::layout::StartposLayout;
use crate::startpos::region_map::RegionMap;
use crate::startpos::settlements::SettlementLayout;
use crate::startpos::tables::StartposTables;
use crate::startpos::world::CampaignIndex;

#[cfg(test)] #[path = "world_test.rs"] mod world_test;

/// Component id of the world mirror's own component.
const WORLD_COMPONENT: u32 = 3;

/// Component id of the world mirror's time.
const TIME_COMPONENT: u32 = 4;

//---------------------------------------------------------------------------//
//                              Types
//---------------------------------------------------------------------------//

/// What the AI writer is built from.
#[derive(Debug, Clone, Copy)]
pub struct CaiInputs<'a> {

    /// Tables of the game and the campaign.
    pub tables: &'a StartposTables,

    /// The campaign's startpos rows.
    pub index: &'a CampaignIndex,

    /// Record layouts of the game.
    pub layout: &'a StartposLayout,

    /// The written `WORLD` record.
    pub world: &'a RecordNode,

    /// Decoded `map_data.esf` of the campaign map.
    pub map_data: &'a ESF,

    /// Decoded `hlp_data.esf` of the campaign map.
    pub hlp_data: &'a ESF,

    /// Number of trade splines of the campaign map.
    pub trade_splines: usize,

    /// Campaign key.
    pub campaign: &'a str,
}

/// What the AI writer works out of a campaign before writing.
#[derive(Debug, Clone, Getters)]
#[getset(get = "pub")]
pub struct CaiContext {

    /// The world's objects.
    world: WorldView,

    /// Regions of the map, in map order.
    map_regions: Vec<MapRegion>,

    /// Pathfinder nodes, by region index.
    nodes: HashMap<usize, Vec<PathfinderNode>>,

    /// Region and area of each cell.
    region_map: RegionMap,

    /// Settlement layouts of the map.
    settlement_layouts: Vec<SettlementLayout>,

    /// Technology keys of each world faction's tree.
    technologies: Vec<Vec<String>>,

    /// Key of the map's theatre.
    theatre_key: String,

    /// Component ids.
    plan: CaiPlan,
}

//---------------------------------------------------------------------------//
//                              Implementations
//---------------------------------------------------------------------------//

impl CaiContext {

    /// Works out what the AI writer needs from a campaign.
    ///
    /// # Errors
    ///
    /// Returns an error if the world doesn't fit its layouts, a map file is malformed or a table is missing.
    pub fn new(inputs: &CaiInputs) -> Result<Self> {
        let world = WorldView::new(inputs.layout, inputs.world)?;
        let map_regions = MapRegion::all_from_map_data(inputs.map_data)?;
        let nodes = pathfinder_nodes(inputs.hlp_data)?;
        let region_map = RegionMap::from_map_data(inputs.map_data)?;
        let settlement_layouts = SettlementLayout::all_from_map_data(inputs.map_data, |_| false)?;
        let theatre_key = theatre_key(inputs.map_data)?;
        let technologies = world.factions().iter()
            .map(|faction| match inputs.index.faction(faction.key()) {
                Some(entry) => faction_technologies(inputs.tables, inputs.campaign, entry),
                None => Ok(vec![]),
            })
            .collect::<Result<Vec<_>>>()?;
        let technology_counts = technologies.iter().map(Vec::len).collect::<Vec<_>>();
        let plan = CaiPlan::new(&PlanInputs { map_regions: &map_regions, world: &world, technology_counts: &technology_counts, trade_splines: inputs.trade_splines });
        Ok(Self { world, map_regions, nodes, region_map, settlement_layouts, technologies, theatre_key, plan })
    }

    /// Writes the world mirror (`CAI_WORLD`).
    ///
    /// # Errors
    ///
    /// Returns an error if a table is missing or the layout lacks a record.
    pub fn write_world(&self, inputs: &CaiInputs) -> Result<RecordNode> {
        self.with_mobiles(inputs, |mobile_inputs| self.write_world_with(inputs, mobile_inputs))
    }

    /// Runs a writer with what the mobiles are written from: their settlements, ports and connectivity groups.
    pub(crate) fn with_mobiles<T>(&self, inputs: &CaiInputs, write: impl FnOnce(&MobileInputs) -> Result<T>) -> Result<T> {
        let (plan, world, map_regions) = (&self.plan, &self.world, &self.map_regions[..]);
        let groups = AreaGroups::new(&self.region_map, map_regions);
        let settlements = settlement_components(world, plan, map_regions);
        let ports = port_cells(world, map_regions, &groups);
        write(&MobileInputs { layout: inputs.layout, plan, world, region_map: &self.region_map, groups: &groups, settlements: &settlements, ports: &ports })
    }

    /// Writes the world mirror from what the mobiles are written from.
    pub(crate) fn write_world_with(&self, inputs: &CaiInputs, mobile_inputs: &MobileInputs) -> Result<RecordNode> {
        let layout = inputs.layout;
        let (plan, world, map_regions) = (&self.plan, &self.world, &self.map_regions[..]);
        let ports = mobile_inputs.ports;
        let mobiles = mobile_cells(mobile_inputs);
        let victory = victory_factions(inputs.tables, inputs.index, world, plan)?;
        let links = region_links(&LinkInputs { plan, world, map_regions, region_map: &self.region_map, mobiles: &mobiles, ports, victory_factions: &victory });
        let positions = region_positions(map_regions, &self.nodes);
        let capitals = capitals(inputs.index, world, plan, map_regions);
        let faction_links = faction_links(inputs.tables, inputs.index, world, plan, map_regions)?;
        let (provinces, faction_provinces) = write_provinces(layout, plan, world, map_regions, inputs.index)?;
        let mobile_ids = mobiles.iter().map(|(mobile, _)| *mobile).collect();

        let mut record = layout.builder(CAI_WORLD_PATH)?;
        record.group()?
            .set_record("CAI_WORLD_THEATRES", Some(write_theatres(layout, plan, &self.theatre_key, mobile_ids)?))?
            .set_record("CAI_WORLD_FACTIONS", Some(write_factions(layout, plan, &faction_links)?))?
            .set_record("CAI_WORLD_REGIONS", Some(write_regions(layout, plan, map_regions, &self.nodes, &links)?))?
            .set_record("CAI_WORLD_REGION_BOUNDARIES", Some(write_boundaries(layout, plan)?))?
            .set_record("CAI_WORLD_SETTLEMENTS", Some(write_settlements(mobile_inputs, map_regions, &positions, &capitals)?))?
            .set_record("CAI_WORLD_BUILDING_SLOTS", Some(write_building_slots(layout, plan, world, map_regions, &self.settlement_layouts)?))?
            .set_record("CAI_WORLD_RESOURCE_MOBILES", Some(write_mobiles(mobile_inputs)?))?
            .set_record("CAI_WORLD_UNITS", Some(write_units(mobile_inputs)?))?
            .set_record("CAI_WORLD_CHARACTERS", Some(write_characters(mobile_inputs)?))?
            .set_record("CAI_BDI_COMPONENT", Some(component(layout, CAI_WORLD_PATH, WORLD_COMPONENT, vec![])?))?
            .set_record("CAI_BDI_COMPONENT@2", Some(component(layout, CAI_WORLD_PATH, TIME_COMPONENT, vec![])?))?
            .set_record("CAI_WORLD_TRADE_ROUTE_SEGMENTS", Some(write_trade_segments(layout, plan, inputs.trade_splines)?))?
            .set_record("CAI_WORLD_TECHNOLOGY_TREES", Some(write_technology_trees(layout, plan)?))?
            .set_record("CAI_WORLD_TECHNOLOGIES", Some(write_technologies(layout, plan, &self.technologies)?))?
            .set_record("CAI_WORLD_GOVERNORSHIPS", Some(write_governorships(layout, plan)?))?
            .set_record("CAI_WORLD_REGION_HLCIS", Some(write_hlcis(layout, plan, map_regions, &self.nodes)?))?
            .set_record("CAI_CHARACTER_POSTS", Some(write_posts(layout, plan, world)?))?
            .set_record("CAI_PROVINCES", Some(provinces))?
            .set_record("CAI_FACTION_PROVINCES", Some(faction_provinces))?
            .set_record("CAI_FACTION_SEA_REGIONS", Some(write_faction_seas(layout, plan, world, map_regions)?))?;
        record.build()
    }
}
