//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Writer of the `WORLD` record of a startpos.
//!
//! The writer first plans the world (the order and runtime id of every object), then writes each record from its
//! layout, filling the values the world generators compute. Runtime ids use one counter per kind of object, as the
//! game looks them up in separate registries.
//!
//! # Submodules
//!
//! - [`factions`]: the `FACTION` records.
//! - [`regions`]: the `REGION` records and their settlements.

use std::collections::HashMap;

use rpfm_lib::error::{RLibError, Result};
use rpfm_lib::files::esf::RecordNode;

use super::{CampaignIndex, FactionEntry, RegionEntry};
use crate::startpos::layout::{child_path, RecordBuilder, StartposLayout};
use crate::startpos::region_map::RegionMap;
use crate::startpos::settlements::SettlementLayout;
use crate::startpos::tables::{Row, StartposTables};

pub mod factions;
pub mod regions;

#[cfg(test)] mod writer_test;

/// Path of the `WORLD` record.
pub const WORLD_PATH: &str = "CAMPAIGN_STARTPOS/CAMPAIGN_ENV/CAMPAIGN_MODEL/WORLD";

/// Runtime id of the rebel faction. Real factions are numbered from the next one, in startpos order.
pub const REBEL_FACTION_ID: u32 = 1;

//---------------------------------------------------------------------------//
//                              Types
//---------------------------------------------------------------------------//

/// What the world writer needs.
#[derive(Debug, Clone, Copy)]
pub struct WorldInputs<'a> {

    /// Tables of the game and the campaign.
    pub tables: &'a StartposTables,

    /// The campaign's startpos rows.
    pub index: &'a CampaignIndex,

    /// Record layouts of the game.
    pub layout: &'a StartposLayout,

    /// Settlement layouts of the campaign map (walls only for major settlements).
    pub settlements: &'a [SettlementLayout],

    /// Region of each cell of the campaign map.
    pub region_map: &'a RegionMap,
}

/// Order and runtime ids of the objects of the world.
#[derive(Debug, Clone, Default)]
pub struct WorldPlan<'a> {

    /// Factions, in startpos order.
    factions: Vec<&'a FactionEntry>,

    /// Runtime ids of the factions, by key.
    faction_ids: HashMap<String, u32>,

    /// Regions, in startpos order (table order: the game uses a hash order).
    regions: Vec<&'a RegionEntry>,

    /// Runtime ids of the regions, by key.
    region_ids: HashMap<String, u32>,
}

//---------------------------------------------------------------------------//
//                              Implementations
//---------------------------------------------------------------------------//

/// Writes the `WORLD` record of a campaign.
///
/// # Arguments
///
/// * `inputs` - Tables, startpos rows and layouts.
///
/// # Errors
///
/// Returns an error if a table, column or layout is missing.
pub fn write_world(inputs: &WorldInputs) -> Result<RecordNode> {
    write_world_with_plan(inputs, &WorldPlan::new(inputs.index))
}

/// Writes the `WORLD` record of a campaign with a given plan.
///
/// # Arguments
///
/// * `inputs` - Tables, startpos rows and layouts.
/// * `plan` - Order and runtime ids of the world's objects.
///
/// # Errors
///
/// Returns an error if a table, column or layout is missing.
pub fn write_world_with_plan(inputs: &WorldInputs, plan: &WorldPlan) -> Result<RecordNode> {
    let mut world = inputs.layout.builder(WORLD_PATH)?;
    world.group()?
        .set_record("FACTION_ARRAY", Some(factions::write_faction_array(inputs, plan)?))?
        .set_record("REGION_MANAGER", Some(regions::write_region_manager(inputs, plan)?))?;
    world.build()
}

impl<'a> WorldPlan<'a> {

    /// Plans the world of a campaign: factions in startpos order, regions in table order.
    pub fn new(index: &'a CampaignIndex) -> Self {
        Self::with_orders(index.factions_in_startpos_order(), index.regions().iter().collect())
    }

    /// Plans the world of a campaign with the given object orders, for example the ones of a game-built startpos.
    ///
    /// # Arguments
    ///
    /// * `factions` - Factions, in the order of their runtime ids.
    /// * `regions` - Regions, in the order of their runtime ids.
    pub fn with_orders(factions: Vec<&'a FactionEntry>, regions: Vec<&'a RegionEntry>) -> Self {
        let faction_ids = factions.iter().zip(REBEL_FACTION_ID + 1..).map(|(faction, id)| (faction.key().to_owned(), id)).collect();
        let region_ids = regions.iter().zip(1..).map(|(region, id)| (region.key().to_owned(), id)).collect();
        Self { factions, faction_ids, regions, region_ids }
    }

    /// Returns the factions, in startpos order.
    pub fn factions(&self) -> &[&'a FactionEntry] {
        &self.factions
    }

    /// Returns the regions, in startpos order.
    pub fn regions(&self) -> &[&'a RegionEntry] {
        &self.regions
    }

    /// Returns the runtime id of a region.
    pub fn region_id(&self, region: &str) -> Option<u32> {
        self.region_ids.get(region).copied()
    }

    /// Returns the runtime id of a faction.
    pub fn faction_id(&self, faction: &str) -> Option<u32> {
        self.faction_ids.get(faction).copied()
    }
}

/// Starts building a child record from its parent's path and its key.
pub(crate) fn child<'a>(layout: &'a StartposLayout, parent: &str, key: &str) -> Result<RecordBuilder<'a>> {
    layout.builder(&child_path(parent, key))
}

/// Returns the `start_pos_factions` row of a faction of the campaign.
pub(crate) fn start_pos_faction_row<'a>(tables: &'a StartposTables, faction: &FactionEntry) -> Result<Row<'a>> {
    for row in tables.table("start_pos_factions")?.rows() {
        if row.str("ID")? == *faction.id() {
            return Ok(row);
        }
    }
    Err(RLibError::StartposInvalidValue("start_pos_factions".to_owned(), "ID".to_owned(), faction.id().to_owned()))
}
