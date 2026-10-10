//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Writer of the provinces (`WORLD/PROVINCE_MANAGER`) and the sea regions (`WORLD/SEA_REGION_MANAGER`).
//!
//! Both kinds of per-faction managers take recruitment manager ids from the counter the horde armies start: hordes
//! first, then province managers, then sea managers.

use std::collections::HashMap;

use rpfm_lib::error::Result;
use rpfm_lib::files::esf::RecordNode;

use super::sight::{write_line_of_sight, SightPlan};
use super::{child, horde_details, next_id, WorldInputs, WorldPlan, REBEL_FACTION_ID, WORLD_PATH};
use crate::startpos::tables::Row;
use crate::startpos::vision::Vision;
use crate::startpos::world::pools::{province_mercenary_pool, MercenaryUnit};
use crate::startpos::world::provinces::{province_managers, sea_regions, ProvinceManager, SeaRegion};

/// Path of a province.
const PROVINCE_PATH: &str = "CAMPAIGN_STARTPOS/CAMPAIGN_ENV/CAMPAIGN_MODEL/WORLD/PROVINCE_MANAGER/PROVINCE_ARRAY/PROVINCE";

/// Path of a faction's manager of a province.
const PROVINCE_MANAGER_PATH: &str = "CAMPAIGN_STARTPOS/CAMPAIGN_ENV/CAMPAIGN_MODEL/WORLD/PROVINCE_MANAGER/PROVINCE_ARRAY/PROVINCE/FACTION_PROVINCE_MANAGER_ARRAY/FACTION_PROVINCE_MANAGER";

/// Path of a sea region.
const SEA_REGION_PATH: &str = "CAMPAIGN_STARTPOS/CAMPAIGN_ENV/CAMPAIGN_MODEL/WORLD/SEA_REGION_MANAGER/SEA_REGIONS_BLOCK/SEA_REGION";

/// Path of a faction's manager of a sea region.
const SEA_MANAGER_PATH: &str = "CAMPAIGN_STARTPOS/CAMPAIGN_ENV/CAMPAIGN_MODEL/WORLD/SEA_REGION_MANAGER/SEA_REGIONS_BLOCK/SEA_REGION/FACTION_SEA_REGION_MANAGER_BLOCK/FACTION_SEA_REGION_MANAGER";

/// Limit of a province's public order, both ways (`province_public_order_*_threshold` campaign variables).
const MAX_PUBLIC_ORDER: i64 = 100;

/// Lowest public order of each populace happiness level, from jubilant to unhappy; lower values are outraged. The
/// game hardcodes them: these fit every level of Attila's main campaign (jubilant, happy, indifferent, troubled).
const HAPPINESS_THRESHOLDS: [i64; 6] = [80, 50, 20, -20, -50, -80];

//---------------------------------------------------------------------------//
//                              Types
//---------------------------------------------------------------------------//

/// Writes the provinces and the sea regions, numbering their managers as it goes.
struct ProvinceWriter<'a, 'b> {

    /// Tables, startpos rows and layouts.
    inputs: &'b WorldInputs<'a>,

    /// Order and runtime ids of the world's objects.
    plan: &'b WorldPlan<'a>,

    /// Vision of the world.
    sight: &'b SightPlan,

    /// `start_pos_regions` rows, by region key.
    regions: HashMap<String, Row<'a>>,

    /// Next recruitment manager id.
    next_recruitment: u32,

    /// Next mercenary pool id.
    next_pool: u32,

    /// Runtime ids of the provinces, by key.
    province_ids: HashMap<String, u32>,
}

/// The province and sea region managers, with the ids later records continue from.
#[derive(Debug, Clone)]
pub struct ProvinceRecords {

    /// The province manager.
    pub provinces: RecordNode,

    /// The sea region manager.
    pub seas: RecordNode,

    /// Next free recruitment manager id (for horde armies without horde details).
    pub next_recruitment: u32,

    /// Next free mercenary pool id (for levy pools).
    pub next_pool: u32,

    /// Runtime ids of the provinces, by key.
    pub province_ids: HashMap<String, u32>,
}

//---------------------------------------------------------------------------//
//                              Implementations
//---------------------------------------------------------------------------//

/// Writes the province manager and the sea region manager.
///
/// # Returns
///
/// The two records, with the ids later records continue from.
///
/// # Errors
///
/// Returns an error if a table, column or layout is missing.
pub fn write_province_and_sea_managers(inputs: &WorldInputs, plan: &WorldPlan, sight: &SightPlan) -> Result<ProvinceRecords> {
    let mut regions = HashMap::new();
    for row in inputs.tables.table("start_pos_regions")?.rows() {
        if row.str("campaign")? == *inputs.index.campaign() {
            regions.insert(row.str("region")?.into_owned(), row);
        }
    }

    let first_recruitment = u32::try_from(horde_details(inputs)?.len()).unwrap_or(u32::MAX).saturating_add(1);
    // Mercenary pool ids continue the ones of the factions (the rebels' and one per faction).
    let first_pool = u32::try_from(plan.factions().len()).unwrap_or(u32::MAX).saturating_add(REBEL_FACTION_ID + 1);
    let mut writer = ProvinceWriter { inputs, plan, sight, regions, next_recruitment: first_recruitment, next_pool: first_pool, province_ids: HashMap::new() };
    let provinces = writer.write_province_manager()?;
    let seas = writer.write_sea_region_manager()?;
    Ok(ProvinceRecords {
        provinces,
        seas,
        next_recruitment: writer.next_recruitment,
        next_pool: writer.next_pool,
        province_ids: writer.province_ids,
    })
}

impl ProvinceWriter<'_, '_> {

    /// Writes the provinces, in `region_to_province_junctions` order (the game uses a hash order).
    fn write_province_manager(&mut self) -> Result<RecordNode> {
        let inputs = self.inputs;
        let managers = province_managers(inputs.tables, inputs.index)?;

        let mut provinces = Vec::<(String, Vec<String>)>::new();
        for row in inputs.tables.table("region_to_province_junctions")?.rows() {
            let region = row.str("region")?;
            if !self.regions.contains_key(region.as_ref()) {
                continue;
            }
            let province = row.str("province")?;
            match provinces.iter_mut().find(|(key, _)| *key == province) {
                Some((_, regions)) => regions.push(region.into_owned()),
                None => provinces.push((province.into_owned(), vec![region.into_owned()])),
            }
        }

        let mut next_manager = 1;
        let mut array = child(inputs.layout, &format!("{WORLD_PATH}/PROVINCE_MANAGER"), "PROVINCE_ARRAY")?;
        for (index, (province, regions)) in provinces.iter().enumerate() {
            let province_id = u32::try_from(index + 1).unwrap_or(u32::MAX);
            self.province_ids.insert(province.to_owned(), province_id);
            let mut manager_array = child(inputs.layout, PROVINCE_PATH, "FACTION_PROVINCE_MANAGER_ARRAY")?;
            for manager in managers.iter().filter(|manager| manager.province() == province) {
                let record = self.write_province_faction_manager(manager, next_id(&mut next_manager))?;
                manager_array.push_group()?.set_record("FACTION_PROVINCE_MANAGER", Some(record))?;
            }

            let mut record = self.inputs.layout.builder(PROVINCE_PATH)?;
            record.group()?
                .set_u32("^+1", province_id)?
                .set_record("FACTION_PROVINCE_MANAGER_ARRAY", Some(manager_array.build()?))?
                .set_record("PROVINCE_MERCENARY_POOL", Some(self.write_province_mercenary_pool(province)?))?
                .set_record("PROVINCE_FERTILITY", Some(self.write_fertility(regions)?))?;
            array.push_group()?
                .set_str("^+1", province)?
                .set_record("PROVINCE", Some(record.build()?))?;
        }

        let mut manager = child(inputs.layout, WORLD_PATH, "PROVINCE_MANAGER")?;
        manager.group()?.set_record("PROVINCE_ARRAY", Some(array.build()?))?;
        manager.build()
    }

    /// Writes a faction's manager of a province: its regions, public order and development.
    fn write_province_faction_manager(&mut self, manager: &ProvinceManager, id: u32) -> Result<RecordNode> {
        let layout = self.inputs.layout;
        let public_order = self.sum(manager.regions(), "initial_public_order")?.clamp(-MAX_PUBLIC_ORDER, MAX_PUBLIC_ORDER);
        let happiness = HAPPINESS_THRESHOLDS.iter().position(|threshold| public_order >= *threshold).unwrap_or(HAPPINESS_THRESHOLDS.len());

        let mut info = child(layout, &format!("{PROVINCE_MANAGER_PATH}/FACTION_PROVINCE_PUBLIC_ORDER"), "FACTION_PROVINCE_PUBLIC_ORDER_INFO")?;
        info.group()?
            .set_i32("^+4", i32::try_from(public_order).unwrap_or_default())?
            .set_i32("^+7", i32::try_from(happiness).unwrap_or_default())?;
        let mut public_order_record = child(layout, PROVINCE_MANAGER_PATH, "FACTION_PROVINCE_PUBLIC_ORDER")?;
        public_order_record.group()?.set_record("FACTION_PROVINCE_PUBLIC_ORDER_INFO", Some(info.build()?))?;

        let mut development = child(layout, PROVINCE_MANAGER_PATH, "PROVINCE_DEVELOPMENT")?;
        development.group()?.set_u32("^+1", u32::try_from(self.sum(manager.regions(), "development_points")?).unwrap_or_default())?;

        let region_ids = manager.regions().iter().map(|region| self.plan.region_id(region).unwrap_or_default()).collect();
        let mut record = layout.builder(PROVINCE_MANAGER_PATH)?;
        record.group()?
            .set_u32("^+1", id)?
            .set_u32("^+2", self.faction_id(manager.faction().as_deref()))?
            .set_u32s("^+3", region_ids)?
            .set_record("RECRUITMENT_MANAGER", Some(self.write_recruitment_manager(PROVINCE_MANAGER_PATH)?))?
            .set_record("FACTION_PROVINCE_PUBLIC_ORDER", Some(public_order_record.build()?))?
            .set_record("PROVINCE_DEVELOPMENT", Some(development.build()?))?
            .set_u32("CAMPAIGN_EFFECT_BUNDLES+4", self.plan.region_id(manager.main_region()).unwrap_or_default())?;
        record.build()
    }

    /// Writes the mercenary pool of a province, with the next mercenary pool id.
    fn write_province_mercenary_pool(&mut self, province: &str) -> Result<RecordNode> {
        let id = next_id(&mut self.next_pool);
        let path = format!("{PROVINCE_PATH}/PROVINCE_MERCENARY_POOL");
        let units = province_mercenary_pool(self.inputs.tables, province)?;
        let mut record = self.inputs.layout.builder(&path)?;
        record.group()?
            .set_u32("^+1", id)?
            .set_record("MERCENARY_POOL", Some(write_mercenary_pool(self.inputs, &path, &units)?))?;
        record.build()
    }

    /// Writes the fertility of a province: the sum of its regions' base fertility, and the list of its regions.
    fn write_fertility(&self, regions: &[String]) -> Result<RecordNode> {
        let path = format!("{PROVINCE_PATH}/PROVINCE_FERTILITY");
        let mut block = child(self.inputs.layout, &path, "REGION_BLOCK")?;
        for region in regions {
            block.push_group()?.set_str("^+1", region)?;
        }

        let mut record = self.inputs.layout.builder(&path)?;
        record.group()?
            .set_i32("^+1", i32::try_from(self.sum(regions, "base_fertility")?).unwrap_or_default())?
            .set_record("REGION_BLOCK", Some(block.build()?))?;
        record.build()
    }

    /// Writes the sea regions with ports, in map order.
    fn write_sea_region_manager(&mut self) -> Result<RecordNode> {
        let inputs = self.inputs;
        let seas = sea_regions(inputs.region_map, inputs.settlements, inputs.index);

        let mut next_manager = 1;
        let mut block = child(inputs.layout, &format!("{WORLD_PATH}/SEA_REGION_MANAGER"), "SEA_REGIONS_BLOCK")?;
        for (index, sea) in seas.iter().enumerate() {
            let record = self.write_sea_region(sea, u32::try_from(index + 1).unwrap_or(u32::MAX), &mut next_manager)?;
            block.push_group()?
                .set_str("^+1", sea.key())?
                .set_record("SEA_REGION", Some(record))?;
        }

        let mut manager = child(inputs.layout, WORLD_PATH, "SEA_REGION_MANAGER")?;
        manager.group()?.set_record("SEA_REGIONS_BLOCK", Some(block.build()?))?;
        manager.build()
    }

    /// Writes a sea region: one manager per faction with ports on it, with the sorted ids of its port regions.
    fn write_sea_region(&mut self, sea: &SeaRegion, id: u32, next_manager: &mut u32) -> Result<RecordNode> {
        let layout = self.inputs.layout;
        let mut managers = child(layout, SEA_REGION_PATH, "FACTION_SEA_REGION_MANAGER_BLOCK")?;
        for (faction, regions) in sea.ports() {
            let mut region_ids = regions.iter().map(|region| self.plan.region_id(region).unwrap_or_default()).collect::<Vec<_>>();
            region_ids.sort_unstable();

            // What the faction's port settlements see (an approximation of the game's sea vision).
            let mut vision = Vision::default();
            for region in regions {
                if let Some(settlement) = self.sight.settlement(region) {
                    vision.extend(settlement);
                }
            }

            let mut manager = layout.builder(SEA_MANAGER_PATH)?;
            manager.group()?
                .set_u32("^+1", next_id(next_manager))?
                .set_u32("^+2", self.faction_id(faction.as_deref()))?
                .set_u32s("^+3", region_ids)?
                .set_record("RECRUITMENT_MANAGER", Some(self.write_recruitment_manager(SEA_MANAGER_PATH)?))?
                .set_record("LINE_OF_SIGHT", Some(write_line_of_sight(layout, SEA_MANAGER_PATH, self.sight.grid(), Some(&vision))?))?;
            managers.push_group()?.set_record("FACTION_SEA_REGION_MANAGER", Some(manager.build()?))?;
        }

        let mut record = layout.builder(SEA_REGION_PATH)?;
        record.group()?
            .set_u32("^+1", id)?
            .set_record("FACTION_SEA_REGION_MANAGER_BLOCK", Some(managers.build()?))?;
        record.build()
    }

    /// Writes an empty recruitment manager under a parent record, with the next recruitment manager id.
    fn write_recruitment_manager(&mut self, parent: &str) -> Result<RecordNode> {
        let mut record = child(self.inputs.layout, parent, "RECRUITMENT_MANAGER")?;
        record.group()?.set_u32("^+1", next_id(&mut self.next_recruitment))?;
        record.build()
    }

    /// Returns the runtime id of a faction, or the rebels' one.
    fn faction_id(&self, faction: Option<&str>) -> u32 {
        faction.and_then(|faction| self.plan.faction_id(faction)).unwrap_or(REBEL_FACTION_ID)
    }

    /// Returns the sum of an integer column of `start_pos_regions` over some regions.
    fn sum(&self, regions: &[String], column: &str) -> Result<i64> {
        let mut total = 0;
        for region in regions {
            if let Some(row) = self.regions.get(region) {
                total += row.int(column)?;
            }
        }
        Ok(total)
    }
}

/// Writes a mercenary pool (`MERCENARY_POOL`) under a parent record: one entry per unit, numbered in order.
pub(super) fn write_mercenary_pool(inputs: &WorldInputs, parent: &str, units: &[MercenaryUnit]) -> Result<RecordNode> {
    let path = format!("{parent}/MERCENARY_POOL");
    let mut array = child(inputs.layout, &path, "MERCENARY_POOL_UNIT_ARRAY")?;
    for (index, unit) in units.iter().enumerate() {
        array.push_group()?
            .set_str("^+1", unit.unit())?
            .set_u32("^+2", unit.initial_count())?
            .set_f32("^+3", unit.replenish_chance())?
            .set_u32("^+4", unit.max_count())?
            .set_u32("^+5", unit.max_replenish_per_turn())?
            .set_u32("^+6", unit.max_count())?
            .set_u32("^+8", u32::try_from(index).unwrap_or(u32::MAX))?;
    }

    let mut record = inputs.layout.builder(&path)?;
    record.group()?.set_record("MERCENARY_POOL_UNIT_ARRAY", Some(array.build()?))?;
    record.build()
}
