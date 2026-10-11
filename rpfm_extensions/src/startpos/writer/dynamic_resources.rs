//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Writer of the dynamic resources manager (`DYNAMIC_RESOURCES_MANAGER`): one manager per region the map's
//! `dynamic_resources.esf` lists, in the order of the game's map of managers.
//!
//! The map file lists, for each region (by `0xFE00` plus its index in `map_data.esf`'s `REGION_DATA` list), building
//! superchains with a level and the areas they spread over. A region's manager holds an entry for each building it
//! starts with, in slot order, and each level from the building's down to 0 the map file lists for its superchain.
//! Entries start without areas: the game spreads them while playing. Verified on Attila's main campaign: every
//! manager and entry matches the game's.

use std::collections::{HashMap, HashSet};

use rpfm_lib::error::{RLibError, Result};
use rpfm_lib::files::esf::{NodeType, RecordNode, ESF};

use crate::startpos::cai::managers::component_map_order;
use crate::startpos::esf_nav::{as_record, as_str, as_u32, child_record};
use crate::startpos::layout::{child_path, StartposLayout};
use crate::startpos::settlements::{region_data_records, region_key};
use crate::startpos::tables::StartposTables;
use crate::startpos::world::CampaignIndex;
use crate::startpos::world::writer::WorldPlan;

#[cfg(test)] #[path = "dynamic_resources_test.rs"] mod dynamic_resources_test;

/// Path of the dynamic resources manager.
const MANAGER_PATH: &str = "CAMPAIGN_STARTPOS/CAMPAIGN_ENV/CAMPAIGN_MODEL/DYNAMIC_RESOURCES_MANAGER";

/// Added to a region's index in `map_data.esf` to get its key in `dynamic_resources.esf`.
const REGION_KEY_BASE: u16 = 0xFE00;

//---------------------------------------------------------------------------//
//                              Types
//---------------------------------------------------------------------------//

/// The (superchain, level) pairs `dynamic_resources.esf` lists for each region, by region key.
type MapLevels<'a> = HashMap<&'a str, HashSet<(&'a str, u32)>>;

/// What the dynamic resources manager is built from.
#[derive(Clone, Copy)]
pub struct DynamicResourcesInputs<'a> {

    /// Record layouts of the game.
    pub layout: &'a StartposLayout,

    /// Tables with `building_levels` and `building_chains`.
    pub tables: &'a StartposTables,

    /// The campaign's startpos rows, for the buildings regions start with.
    pub index: &'a CampaignIndex,

    /// Regions of the world, with their runtime ids.
    pub plan: &'a WorldPlan<'a>,

    /// Decoded `map_data.esf` of the campaign map.
    pub map_data: &'a ESF,

    /// Decoded `dynamic_resources.esf` of the campaign map.
    pub dynamic_resources: &'a ESF,
}

//---------------------------------------------------------------------------//
//                              Implementations
//---------------------------------------------------------------------------//

/// Writes the dynamic resources manager of a campaign.
///
/// # Errors
///
/// Returns an error if a table, column or layout is missing, or the map files are malformed.
pub fn write_dynamic_resources_manager(inputs: &DynamicResourcesInputs) -> Result<RecordNode> {
    let (date, map_levels) = read_map_levels(inputs.map_data, inputs.dynamic_resources)?;
    let building_levels = building_levels(inputs.tables)?;

    let regions = inputs.plan.regions().iter().filter(|region| map_levels.contains_key(region.key().as_str())).collect::<Vec<_>>();
    let ids = regions.iter().map(|region| inputs.plan.region_id(region.key()).unwrap_or_default()).collect::<Vec<_>>();

    let managers_path = child_path(MANAGER_PATH, "REGION_DYNAMIC_RESOURCES_MANAGERS_BLOCK");
    let manager_path = child_path(&managers_path, "REGION_DYNAMIC_RESOURCES_MANAGER");
    let resources_path = child_path(&manager_path, "REGION_DYNAMIC_RESOURCES_BLOCK");
    let resource_path = child_path(&resources_path, "REGION_DYNAMIC_RESOURCE");

    let mut managers = inputs.layout.builder(&managers_path)?;
    for position in component_map_order(&ids) {
        let region = regions[position];
        let levels = &map_levels[region.key().as_str()];
        let buildings = inputs.index.settlement_of(region.key())
            .map(|settlement| [settlement.primary_building(), settlement.port_building()].into_iter().chain(settlement.buildings()).filter(|building| !building.is_empty()).collect::<Vec<_>>())
            .unwrap_or_default();

        let mut resources = inputs.layout.builder(&resources_path)?;
        for (superchain, level) in buildings.iter().filter_map(|building| building_levels.get(building.as_str())) {
            for level in (0..=*level).rev().filter(|level| levels.contains(&(superchain.as_str(), *level))) {
                resources.push_group()?
                    .set_str("^+1", superchain)?
                    .set_u32("^+2", level)?
                    .set_record("REGION_DYNAMIC_RESOURCE", Some(inputs.layout.default_record(&resource_path)?))?;
            }
        }

        let mut manager = inputs.layout.builder(&manager_path)?;
        manager.group()?.set_record("REGION_DYNAMIC_RESOURCES_BLOCK", Some(resources.build()?))?;
        managers.push_group()?
            .set_u32("^+1", ids[position])?
            .set_record("REGION_DYNAMIC_RESOURCES_MANAGER", Some(manager.build()?))?;
    }

    let mut manager = inputs.layout.builder(MANAGER_PATH)?;
    manager.group()?
        .set_str("^+1", &date)?
        .set_record("REGION_DYNAMIC_RESOURCES_MANAGERS_BLOCK", Some(managers.build()?))?;
    manager.build()
}

/// Reads `dynamic_resources.esf`: its date stamp, and the (superchain, level) pairs it lists for each region key.
fn read_map_levels<'a>(map_data: &'a ESF, dynamic_resources: &'a ESF) -> Result<(String, MapLevels<'a>)> {
    let malformed = |what: &str| RLibError::CampaignMapDataError(format!("dynamic_resources.esf: {what}"));
    let regions = region_data_records(map_data)?;
    let root = as_record(dynamic_resources.root_node()).ok_or_else(|| malformed("the root node is not a record"))?;
    let date = root.children().first().and_then(|group| group.iter().find_map(as_str)).unwrap_or_default().to_owned();
    let block = child_record(root, "REGION_RESOURCE_INFO_BLOCK").ok_or_else(|| malformed("missing REGION_RESOURCE_INFO_BLOCK"))?;

    let mut levels = HashMap::new();
    for group in block.children() {
        let Some(NodeType::U16(key)) = group.first() else { return Err(malformed("a region without key")) };
        let region = key.checked_sub(REGION_KEY_BASE)
            .and_then(|index| regions.get(usize::from(index)))
            .ok_or_else(|| malformed(&format!("unknown region key {key}")))?;
        let areas = group.iter().find_map(as_record).ok_or_else(|| malformed("a region without areas"))?;
        let entries = areas.children().iter()
            .filter_map(|area| Some((area.first().and_then(as_str)?, area.get(1).and_then(as_u32)?)))
            .collect::<HashSet<_>>();
        levels.insert(region_key(region)?, entries);
    }
    Ok((date, levels))
}

/// Returns the superchain and level of each building level, from `building_levels` and `building_chains`.
fn building_levels(tables: &StartposTables) -> Result<HashMap<String, (String, u32)>> {
    let mut superchains = HashMap::new();
    for row in tables.table("building_chains")?.rows() {
        superchains.insert(row.str("key")?.into_owned(), row.str("building_superchain")?.into_owned());
    }

    let mut levels = HashMap::new();
    for row in tables.table("building_levels")?.rows() {
        if let Some(superchain) = superchains.get(row.str("chain")?.as_ref()) {
            levels.insert(row.str("level_name")?.into_owned(), (superchain.to_owned(), u32::try_from(row.int("level")?).unwrap_or_default()));
        }
    }
    Ok(levels)
}
