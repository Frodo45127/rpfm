//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Extraction of the region tables: regions, settlements, religions, slot templates and unit resources.

use std::collections::HashMap;

use rpfm_lib::error::Result;

use super::{Ids, Source, TableRows};
use crate::startpos::coords::HexCell;
use crate::startpos::layout::{child_path, GroupView};
use crate::startpos::world::writer::{REBEL_FACTION_ID, WORLD_PATH};

#[cfg(test)] #[path = "regions_test.rs"] mod regions_test;

/// Path of a region.
const REGION_PATH: &str = "CAMPAIGN_STARTPOS/CAMPAIGN_ENV/CAMPAIGN_MODEL/WORLD/REGION_MANAGER/REGIONS_ARRAY/REGION";

/// Path of a province.
const PROVINCE_PATH: &str = "CAMPAIGN_STARTPOS/CAMPAIGN_ENV/CAMPAIGN_MODEL/WORLD/PROVINCE_MANAGER/PROVINCE_ARRAY/PROVINCE";

/// Localisation key prefix of the settlement names, followed by the settlement key and the region's row id.
const SETTLEMENT_NAME_PREFIX: &str = "start_pos_settlements_onscreen_name_";

/// Slot types of the region slots, as in `start_pos_region_slot_templates`.
const PRIMARY_SLOT: &str = "primary";
const PORT_SLOT: &str = "port";

/// Columns of `start_pos_settlements` with the buildings of the secondary slots.
const SECONDARY_BUILDING_COLUMNS: [&str; 5] = ["building1", "building2", "building3", "building4", "building5"];

/// Decimals kept of the religion percentages, which the startpos stores as shares of 1 in single precision.
const PERCENTAGE_DECIMALS: i32 = 3;

//---------------------------------------------------------------------------//
//                              Types
//---------------------------------------------------------------------------//

/// The region tables, and what the character tables need of them.
pub(super) struct Regions {

    /// The extracted tables, by name.
    pub tables: Vec<(&'static str, TableRows)>,

    /// The settlements, in region order.
    pub settlements: Vec<Settlement>,
}

/// A settlement of the startpos.
pub(super) struct Settlement {

    /// Row id in `start_pos_settlements`.
    pub id: String,

    /// Runtime id of the region's owner.
    pub owner: u32,

    /// Cell of the settlement (its primary slot), if it has one.
    pub cell: Option<HexCell>,
}

/// Province values the build sums over regions, by region runtime id.
#[derive(Default)]
struct ProvinceValues {
    fertility: HashMap<u32, i32>,
    public_order: HashMap<u32, i32>,
    development: HashMap<u32, u32>,
}

//---------------------------------------------------------------------------//
//                              Implementations
//---------------------------------------------------------------------------//

/// Extracts the region tables.
pub(super) fn extract(source: &Source, ids: &mut Ids) -> Result<Regions> {
    let layout = source.layout;
    let province_values = province_values(source)?;
    let slave_points = slave_points(source)?;
    let capitals = source.factions.iter()
        .filter_map(|faction| Some((faction.id, faction.group.u32("MORGUE+3")?)))
        .collect::<HashMap<_, _>>();

    let mut regions = TableRows::new(&[
        "id", "region", "campaign", "owning_faction", "faction_capital", "base_max_population", "population", "base_gdp", "town_wealth",
        "rebel_faction", "cultural_originator", "settler_rebellions_enabled", "capture_prestige", "alternative_rebel_faction",
        "development_points", "base_fertility", "initial_public_order",
    ]);
    let mut settlements = TableRows::new(&[
        "settlement_id", "region", "building1", "building2", "building3", "building4", "building5", "wealth", "id", "primary_building",
        "port_building", "startpos_slave_points",
    ]);
    let mut religions = TableRows::new(&["region", "religion", "percentage"]);
    let mut templates = TableRows::new(&["campaign", "id", "region", "slot_template", "slot_type"]);
    let mut unit_resources = TableRows::new(&["key", "resource"]);
    let mut settlement_list = vec![];

    for region in &source.regions {
        let region_id = region.u32("^+1").unwrap_or_default();
        let key = region.str("^+2").unwrap_or_default();
        let owner = region.u32("SETTLEMENT+9").unwrap_or_default();
        let owner_table_id = source.faction(owner).filter(|_| owner != REBEL_FACTION_ID).map(|faction| faction.table_id.as_str()).unwrap_or_default();

        let settlement_path = child_path(REGION_PATH, "SETTLEMENT");
        let settlement = region.record("SETTLEMENT").map(|record| layout.read_single(&settlement_path, record)).transpose()?;
        let settlement_key = settlement.as_ref().and_then(|settlement| settlement.str("CAMPAIGN_LOCALISATION+2")).unwrap_or_default();
        let name_key = match settlement.as_ref().and_then(|settlement| settlement.record("CAMPAIGN_LOCALISATION")) {
            Some(record) => layout.read_single(&child_path(&settlement_path, "CAMPAIGN_LOCALISATION"), record)?.str("^+1").unwrap_or_default().to_owned(),
            None => String::new(),
        };

        // The settlement name key ends with the region's row id, which the startpos keeps nowhere else.
        let table_id = name_key.strip_prefix(SETTLEMENT_NAME_PREFIX)
            .and_then(|name| name.strip_prefix(settlement_key))
            .filter(|id| !id.is_empty())
            .map(str::to_owned)
            .unwrap_or_else(|| ids.next());

        let factors = population_factors(source, region)?;
        regions.push(vec![
            table_id.to_owned(),
            key.to_owned(),
            source.campaign.to_owned(),
            owner_table_id.to_owned(),
            (capitals.get(&owner) == Some(&region_id)).to_string(),
            factors.as_ref().and_then(|factors| factors.u32("POPULATION CLASSES+3")).unwrap_or_default().to_string(),
            factors.as_ref().and_then(|factors| factors.u32("POPULATION CLASSES+1")).unwrap_or_default().to_string(),
            region.u32("SETTLEMENT+1").unwrap_or_default().to_string(),
            region.u32("SETTLEMENT+3").unwrap_or_default().to_string(),
            region.str("LINE_OF_SIGHT+1").unwrap_or_default().to_owned(),
            region.str("LINE_OF_SIGHT+4").unwrap_or_default().to_owned(),
            region.bool("LINE_OF_SIGHT+5").unwrap_or_default().to_string(),
            region.u32("LINE_OF_SIGHT+9").unwrap_or_default().to_string(),
            region.str("LINE_OF_SIGHT+3").unwrap_or_default().to_owned(),
            province_values.development.get(&region_id).copied().unwrap_or_default().to_string(),
            province_values.fertility.get(&region_id).copied().unwrap_or_default().to_string(),
            province_values.public_order.get(&region_id).copied().unwrap_or_default().to_string(),
        ]);

        if let Some(factors) = &factors {
            for (religion, share) in religion_shares(source, factors)? {
                religions.push(vec![table_id.to_owned(), religion.to_owned(), percentage(share)]);
            }
        }

        for resource in region.strs("LINE_OF_SIGHT+7").unwrap_or_default() {
            unit_resources.push(vec![table_id.to_owned(), resource.to_owned()]);
        }

        let slots = slots(source, region)?;
        let mut seen_types = vec![];
        for (slot_type, template, _) in &slots {
            if !seen_types.contains(slot_type) {
                seen_types.push(slot_type.to_owned());
                templates.push(vec![source.campaign.to_owned(), ids.next(), key.to_owned(), template.to_owned(), slot_type.to_owned()]);
            }
        }

        let Some(settlement) = settlement else { continue };
        let building_of = |wanted: &str| slots.iter().find(|(slot_type, _, _)| slot_type == wanted).and_then(|(_, _, building)| building.clone()).unwrap_or_default();
        let mut secondary = slots.iter()
            .filter(|(slot_type, _, _)| slot_type != PRIMARY_SLOT && slot_type != PORT_SLOT)
            .filter_map(|(_, _, building)| building.clone())
            .collect::<Vec<_>>();
        secondary.resize(SECONDARY_BUILDING_COLUMNS.len(), String::new());

        let settlement_id = ids.next();
        let mut row = vec![settlement_key.to_owned(), table_id.to_owned()];
        row.extend(secondary);
        row.extend([
            settlement.i32("CAMPAIGN_LOCALISATION+1").unwrap_or_default().to_string(),
            settlement_id.to_owned(),
            building_of(PRIMARY_SLOT),
            building_of(PORT_SLOT),
            slave_points.get(&region_id).copied().unwrap_or_default().to_string(),
        ]);
        settlements.push(row);
        settlement_list.push(Settlement { id: settlement_id, owner, cell: settlement_cell(source, &settlement)? });
    }

    Ok(Regions {
        tables: vec![
            ("start_pos_regions", regions),
            ("start_pos_settlements", settlements),
            ("start_pos_region_religions", religions),
            ("start_pos_region_slot_templates", templates),
            ("start_pos_regions_to_unit_resources", unit_resources),
        ],
        settlements: settlement_list,
    })
}

impl Regions {

    /// Returns the settlement on a cell owned by a faction, if any.
    pub fn settlement_at(&self, cell: HexCell, owner: u32) -> Option<&Settlement> {
        self.settlements.iter().find(|settlement| settlement.owner == owner && settlement.cell == Some(cell))
    }
}

/// Reads the population factors of a region (`POPULATION/REGION_FACTORS`).
fn population_factors<'a>(source: &Source<'a>, region: &GroupView<'a>) -> Result<Option<GroupView<'a>>> {
    let population_path = child_path(REGION_PATH, "POPULATION");
    let Some(population) = region.record("POPULATION") else { return Ok(None) };
    let population = source.layout.read_single(&population_path, population)?;
    population.record("REGION_FACTORS")
        .map(|factors| source.layout.read_single(&child_path(&population_path, "REGION_FACTORS"), factors))
        .transpose()
}

/// Returns the religions of a region with a share above 0, with their share of 1.
fn religion_shares<'a>(source: &Source<'a>, factors: &GroupView<'a>) -> Result<Vec<(&'a str, f32)>> {
    let path = format!("{REGION_PATH}/POPULATION/REGION_FACTORS/RELIGION_BREAKDOWN");
    let Some(breakdown) = factors.record("RELIGION_BREAKDOWN") else { return Ok(vec![]) };
    Ok(source.layout.read(&path, breakdown)?.iter()
        .filter_map(|religion| Some((religion.str("^+1")?, religion.f32("^+2")?)))
        .filter(|(_, share)| *share > 0.0)
        .collect())
}

/// Formats a religion share of 1 as a percentage, rounded to undo the single precision of the share.
fn percentage(share: f32) -> String {
    let scale = 10_f64.powi(PERCENTAGE_DECIMALS);
    ((f64::from(share) * 100.0 * scale).round() / scale).to_string()
}

/// Returns the slots of a region in order: their type, template and building, if any.
fn slots(source: &Source, region: &GroupView) -> Result<Vec<(String, String, Option<String>)>> {
    let layout = source.layout;
    let manager_path = child_path(REGION_PATH, "REGION_SLOT_MANAGER");
    let slot_path = format!("{manager_path}/REGION_SLOT_ARRAY/REGION_SLOT");
    let building_manager_path = child_path(&slot_path, "REGION_BUILDING_MANAGER");
    let Some(manager) = region.record("REGION_SLOT_MANAGER") else { return Ok(vec![]) };
    let manager = layout.read_single(&manager_path, manager)?;

    let mut slots = vec![];
    for (_, slot) in layout.read_items(&child_path(&manager_path, "REGION_SLOT_ARRAY"), &manager, "REGION_SLOT_ARRAY", "REGION_SLOT")? {
        let mut building = None;
        if let Some(building_manager) = slot.record("REGION_BUILDING_MANAGER") {
            let building_manager = layout.read_single(&building_manager_path, building_manager)?;
            if let Some(base) = building_manager.record("BUILDING_BASE") {
                building = layout.read_single(&child_path(&building_manager_path, "BUILDING_BASE"), base)?.str("^+2").map(str::to_owned);
            }
        }
        slots.push((
            slot.str("REGION_BUILDING_MANAGER+3").unwrap_or_default().to_owned(),
            slot.str("REGION_BUILDING_MANAGER+2").unwrap_or_default().to_owned(),
            building,
        ));
    }
    Ok(slots)
}

/// Returns the cell of a settlement: the position of its primary slot.
fn settlement_cell(source: &Source, settlement: &GroupView) -> Result<Option<HexCell>> {
    let manager_path = format!("{REGION_PATH}/SETTLEMENT/SETTLEMENT_EXPANSION_MANAGER");
    let map_data_path = child_path(&manager_path, "SETTLEMENT_EXPANSION_MAP_DATA");
    let Some(manager) = settlement.record("SETTLEMENT_EXPANSION_MANAGER") else { return Ok(None) };
    let manager = source.layout.read_single(&manager_path, manager)?;
    let Some(map_data) = manager.record("SETTLEMENT_EXPANSION_MAP_DATA") else { return Ok(None) };
    let map_data = source.layout.read_single(&map_data_path, map_data)?;
    let Some(slot) = map_data.record("SLOT_MAP_DATA") else { return Ok(None) };
    let slot = source.layout.read_single(&child_path(&map_data_path, "SLOT_MAP_DATA"), slot)?;

    let coordinate = |name: &str| slot.u32(name).and_then(|value| u16::try_from(value).ok());
    Ok(coordinate("^+2").zip(coordinate("^+3")).map(|(x, y)| HexCell::new(x, y)))
}

/// Returns the slave points of each region (`WORLD/SLAVES`), by region runtime id.
fn slave_points(source: &Source) -> Result<HashMap<u32, u32>> {
    let path = child_path(WORLD_PATH, "SLAVES");
    let Some(slaves) = source.world.record("SLAVES") else { return Ok(HashMap::new()) };
    let slaves = source.layout.read_single(&path, slaves)?;
    let Some(block) = slaves.record("SLAVES_BLOCK") else { return Ok(HashMap::new()) };
    Ok(source.layout.read(&child_path(&path, "SLAVES_BLOCK"), block)?.iter()
        .filter_map(|entry| Some((entry.u32("^+1")?, entry.u32("^+2")?)))
        .collect())
}

/// Returns the province values the build sums over regions, each sum on one region: the fertility on the province's
/// first region, and each faction's public order and development on its main region of the province.
fn province_values(source: &Source) -> Result<ProvinceValues> {
    let layout = source.layout;
    let region_ids = source.regions.iter()
        .filter_map(|region| Some((region.str("^+2")?, region.u32("^+1")?)))
        .collect::<HashMap<_, _>>();
    let fertility_path = child_path(PROVINCE_PATH, "PROVINCE_FERTILITY");
    let managers_path = child_path(PROVINCE_PATH, "FACTION_PROVINCE_MANAGER_ARRAY");
    let manager_path = child_path(&managers_path, "FACTION_PROVINCE_MANAGER");
    let public_order_path = child_path(&manager_path, "FACTION_PROVINCE_PUBLIC_ORDER");

    let mut values = ProvinceValues::default();
    for province in &source.provinces {
        if let Some(fertility) = province.record("PROVINCE_FERTILITY") {
            let fertility = layout.read_single(&fertility_path, fertility)?;
            let first_region = match fertility.record("REGION_BLOCK") {
                Some(block) => layout.read(&child_path(&fertility_path, "REGION_BLOCK"), block)?.first().and_then(|entry| entry.str("^+1")).and_then(|key| region_ids.get(key)).copied(),
                None => None,
            };
            if let Some(region) = first_region {
                values.fertility.insert(region, fertility.i32("^+1").unwrap_or_default());
            }
        }

        for (_, manager) in layout.read_items(&managers_path, province, "FACTION_PROVINCE_MANAGER_ARRAY", "FACTION_PROVINCE_MANAGER")? {
            let main_region = manager.u32("CAMPAIGN_EFFECT_BUNDLES+4")
                .filter(|region| *region != 0)
                .or_else(|| manager.u32s("^+3").and_then(|regions| regions.first().copied()));
            let Some(region) = main_region else { continue };

            if let Some(public_order) = manager.record("FACTION_PROVINCE_PUBLIC_ORDER") {
                let public_order = layout.read_single(&public_order_path, public_order)?;
                if let Some(info) = public_order.record("FACTION_PROVINCE_PUBLIC_ORDER_INFO") {
                    let info = layout.read_single(&child_path(&public_order_path, "FACTION_PROVINCE_PUBLIC_ORDER_INFO"), info)?;
                    values.public_order.insert(region, info.i32("^+4").unwrap_or_default());
                }
            }
            if let Some(development) = manager.record("PROVINCE_DEVELOPMENT") {
                let development = layout.read_single(&child_path(&manager_path, "PROVINCE_DEVELOPMENT"), development)?;
                values.development.insert(region, development.u32("^+1").unwrap_or_default());
            }
        }
    }
    Ok(values)
}
