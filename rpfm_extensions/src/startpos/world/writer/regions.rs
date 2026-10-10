//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Writer of the `REGION` records (`WORLD/REGION_MANAGER/REGIONS_ARRAY`), with their slots and settlements.

use std::collections::{HashMap, HashSet};

use rpfm_lib::error::{RLibError, Result};
use rpfm_lib::files::esf::RecordNode;

use super::factions::write_flag;
use super::forces::{ForcePlan, PlannedForce};
use super::sight::{write_line_of_sight, SightPlan};
use super::{child, horde_details, next_id, HORDE_BUILDING_COLUMNS, WorldInputs, WorldPlan, REBEL_FACTION_ID, WORLD_PATH};
use crate::startpos::coords::{EdgePiece, HexCell};
use crate::startpos::settlements::{SettlementLayout, SlotLayout};
use crate::startpos::tables::Row;
use crate::startpos::world::RegionEntry;
use crate::startpos::world::legacy::ForceKind;
use crate::startpos::world::rebels::rebel_setup;

/// Path of a region.
const REGION_PATH: &str = "CAMPAIGN_STARTPOS/CAMPAIGN_ENV/CAMPAIGN_MODEL/WORLD/REGION_MANAGER/REGIONS_ARRAY/REGION";

/// Path of a region slot.
const SLOT_PATH: &str = "CAMPAIGN_STARTPOS/CAMPAIGN_ENV/CAMPAIGN_MODEL/WORLD/REGION_MANAGER/REGIONS_ARRAY/REGION/REGION_SLOT_MANAGER/REGION_SLOT_ARRAY/REGION_SLOT";

/// Path of a settlement.
const SETTLEMENT_PATH: &str = "CAMPAIGN_STARTPOS/CAMPAIGN_ENV/CAMPAIGN_MODEL/WORLD/REGION_MANAGER/REGIONS_ARRAY/REGION/SETTLEMENT";

/// Path of a settlement's expansion data.
const EXPANSION_PATH: &str = "CAMPAIGN_STARTPOS/CAMPAIGN_ENV/CAMPAIGN_MODEL/WORLD/REGION_MANAGER/REGIONS_ARRAY/REGION/SETTLEMENT/SETTLEMENT_EXPANSION_MANAGER/SETTLEMENT_EXPANSION_MAP_DATA";

/// Localisation key prefix of the region names.
const REGION_NAME_PREFIX: &str = "regions_onscreen_";

/// Localisation key prefix of the settlement names (followed by `settlement:<region key><region row id>`).
const SETTLEMENT_NAME_PREFIX: &str = "start_pos_settlements_onscreen_name_";

/// Localisation key of the name of the rebels holding an abandoned region.
const ABANDONED_REBELS_NAME: &str = "campaign_localised_strings_string_abandoned_region_rebel_faction_name";

/// Value of `REBEL_SETUP^+5` for the rebels holding an abandoned region (meaning unknown).
const ABANDONED_REBELS_VALUE: u32 = 12;

/// Population classes of every region, in startpos order.
const POPULATION_CLASSES: [&str; 3] = ["upper", "lower", "middle"];

/// Value of the fifth field of every population class (meaning unknown).
const POPULATION_CLASS_VALUE: u32 = 7;

/// Coordinate the startpos uses for slots without position (the port slot of settlements without port).
const NO_POSITION: u32 = 65535;

/// Added to a building's level to get the lowest value of the two floats of its `BUILDING_BASE`.
const BUILDING_BASE_VALUE_OFFSET: i64 = 3;

//---------------------------------------------------------------------------//
//                              Types
//---------------------------------------------------------------------------//

/// Writes the regions, numbering their slots, residences and buildings as it goes.
struct RegionWriter<'a, 'b> {

    /// Tables, startpos rows and layouts.
    inputs: &'b WorldInputs<'a>,

    /// Order and runtime ids of the world's objects.
    plan: &'b WorldPlan<'a>,

    /// Characters and forces of the world.
    forces: &'b ForcePlan<'a>,

    /// Vision of the world.
    sight: &'b SightPlan,

    /// Table rows the regions use, indexed once.
    rows: RegionRows<'a>,

    /// Next slot id.
    next_slot: u32,

    /// Next garrison residence id (slots and settlements share the counter).
    next_residence: u32,

    /// Next building id (horde and region buildings share the counter).
    next_building: u32,
}

/// Table rows the region writer looks up, indexed once.
struct RegionRows<'a> {

    /// `start_pos_regions` rows, by row id.
    regions: HashMap<String, Row<'a>>,

    /// `start_pos_settlements.settlement_id`, by row id.
    settlement_keys: HashMap<String, String>,

    /// Government types of the campaign's factions, by faction key.
    governments: HashMap<String, String>,

    /// Slot templates, by region key and slot type.
    templates: HashMap<(String, String), String>,

    /// `create_time` and `level` of the building levels, by key.
    building_levels: HashMap<String, (i64, i64)>,

    /// Religion keys, in `religions` order.
    religions: Vec<String>,

    /// Religion percentages, by region row id and religion key.
    religion_shares: HashMap<(String, String), f32>,
}

/// The kind of a region slot, with its building.
#[derive(Debug, Clone, Copy)]
struct SlotContent<'s> {

    /// Slot type (`primary`, `port` or `secondary`), as in `start_pos_region_slot_templates`.
    slot_type: &'s str,

    /// Building level in the slot, if any.
    building: Option<&'s str>,
}

//---------------------------------------------------------------------------//
//                              Implementations
//---------------------------------------------------------------------------//

/// Writes the region manager.
///
/// # Errors
///
/// Returns an error if a table, column or layout is missing, or a region has no settlement layout.
pub fn write_region_manager<'a>(inputs: &WorldInputs<'a>, plan: &WorldPlan<'a>, forces: &ForcePlan<'a>, sight: &SightPlan) -> Result<RecordNode> {
    let mut writer = RegionWriter {
        inputs,
        plan,
        forces,
        sight,
        rows: RegionRows::new(inputs)?,
        next_slot: 1,
        next_residence: 1,
        next_building: horde_building_count(inputs)? + 1,
    };

    let mut regions = child(inputs.layout, &format!("{WORLD_PATH}/REGION_MANAGER"), "REGIONS_ARRAY")?;
    for region in plan.regions() {
        regions.push_group()?.set_record("REGION", Some(writer.write_region(region)?))?;
    }

    let mut manager = child(inputs.layout, WORLD_PATH, "REGION_MANAGER")?;
    manager.group()?.set_record("REGIONS_ARRAY", Some(regions.build()?))?;
    manager.build()
}

impl<'a> RegionRows<'a> {

    /// Indexes the rows the region writer needs.
    fn new(inputs: &WorldInputs<'a>) -> Result<Self> {
        let tables = inputs.tables;
        let campaign = inputs.index.campaign();

        let mut regions = HashMap::new();
        for row in tables.table("start_pos_regions")?.rows() {
            regions.insert(row.str("id")?.into_owned(), row);
        }

        let settlement_keys = settlement_keys(inputs)?;

        let mut governments = HashMap::new();
        for row in tables.table("start_pos_factions")?.rows() {
            if row.str("campaign")? == *campaign {
                governments.insert(row.str("faction")?.into_owned(), row.str("government_type")?.into_owned());
            }
        }

        let mut templates = HashMap::new();
        for row in tables.table("start_pos_region_slot_templates")?.rows() {
            if row.str("campaign")? == *campaign {
                templates.insert((row.str("region")?.into_owned(), row.str("slot_type")?.into_owned()), row.str("slot_template")?.into_owned());
            }
        }

        let mut building_levels = HashMap::new();
        for row in tables.table("building_levels")?.rows() {
            building_levels.insert(row.str("level_name")?.into_owned(), (row.int("create_time")?, row.int("level")?));
        }

        let religions = tables.table("religions")?.rows()
            .map(|row| Ok(row.str("religion_key")?.into_owned()))
            .collect::<Result<Vec<_>>>()?;

        let mut religion_shares = HashMap::new();
        for row in tables.table("start_pos_region_religions")?.rows() {
            religion_shares.insert((row.str("region")?.into_owned(), row.str("religion")?.into_owned()), row.f32("percentage")?);
        }

        Ok(Self { regions, settlement_keys, governments, templates, building_levels, religions, religion_shares })
    }
}

impl RegionWriter<'_, '_> {

    /// Writes a region.
    fn write_region(&mut self, region: &RegionEntry) -> Result<RecordNode> {
        let layout = self.inputs.layout;
        let key = region.key();
        let row = *self.rows.regions.get(region.id())
            .ok_or_else(|| RLibError::StartposInvalidValue("start_pos_regions".to_owned(), "id".to_owned(), region.id().to_owned()))?;
        let wealth = u32::try_from(row.int("town_wealth")?).unwrap_or_default();
        let owner = region.owner().as_deref().and_then(|owner| self.plan.faction_id(owner)).unwrap_or(REBEL_FACTION_ID);
        let map_layout = settlement_layout(self.inputs, region)?;

        let mut localisation = child(layout, REGION_PATH, "CAMPAIGN_LOCALISATION")?;
        localisation.group()?.set_str("^+1", &format!("{REGION_NAME_PREFIX}{key}"))?;

        let slots = self.write_slot_manager(region, map_layout)?;
        let settlement = self.write_settlement(region, map_layout)?;

        let mut record = layout.builder(REGION_PATH)?;
        record.group()?
            .set_u32("^+1", self.plan.region_id(key).unwrap_or_default())?
            .set_str("^+2", key)?
            .set_record("POPULATION", Some(self.write_population(region)?))?
            .set_record("REGION_SLOT_MANAGER", Some(slots))?
            .set_record("SETTLEMENT", Some(settlement))?
            .set_u32("SETTLEMENT+3", wealth)?
            .set_u32("SETTLEMENT+4", wealth)?
            .set_u32("SETTLEMENT+5", wealth)?
            .set_u32("SETTLEMENT+9", owner)?
            .set_str("LINE_OF_SIGHT+1", &row.str("rebel_faction")?)?
            .set_str("LINE_OF_SIGHT+3", &row.str("alternative_rebel_faction")?)?
            .set_str("LINE_OF_SIGHT+4", &row.str("cultural_originator")?)?
            .set_record("CAMPAIGN_LOCALISATION", Some(localisation.build()?))?
            .set_record("REGION_REBEL_MANAGER", Some(self.write_rebel_manager(region)?))?
            .set_bool("REGION_REBEL_MANAGER+1", region.owner().is_none())?
            .set_record("LINE_OF_SIGHT", Some(write_line_of_sight(layout, REGION_PATH, self.sight.grid(), self.sight.region(key))?))?;
        record.build()
    }

    /// Writes the population of a region: its religion shares, in `religions` order.
    ///
    /// Shares are normalised, so percentages that don't add up to 100 still give a total of 1. The game computes them
    /// in single precision as `(percentage * 0.01) / (total * 0.01)`, which isn't always the same float as
    /// `percentage / total`.
    fn write_population(&self, region: &RegionEntry) -> Result<RecordNode> {
        let percentages = self.rows.religions.iter()
            .map(|religion| self.rows.religion_shares.get(&(region.id().to_owned(), religion.to_owned())).copied().unwrap_or_default())
            .collect::<Vec<_>>();
        let total = percentages.iter().sum::<f32>() * 0.01;

        let path = format!("{REGION_PATH}/POPULATION/REGION_FACTORS");
        let mut breakdown = child(self.inputs.layout, &path, "RELIGION_BREAKDOWN")?;
        for (religion, percentage) in self.rows.religions.iter().zip(percentages) {
            let share = if total > 0.0 { percentage * 0.01 / total } else { 0.0 };
            breakdown.push_group()?
                .set_str("^+1", religion)?
                .set_f32("^+2", share)?;
        }

        let classes_path = format!("{path}/POPULATION CLASSES");
        let mut classes = self.inputs.layout.builder(&classes_path)?;
        for class in POPULATION_CLASSES {
            let mut record = child(self.inputs.layout, &classes_path, "POPULATION_CLASS")?;
            record.group()?
                .set_str("^+1", class)?
                .set_u32("^+5", POPULATION_CLASS_VALUE)?;
            classes.push_group()?.set_record("POPULATION_CLASS", Some(record.build()?))?;
        }

        let mut factors = self.inputs.layout.builder(&path)?;
        factors.group()?
            .set_record("POPULATION CLASSES", Some(classes.build()?))?
            .set_record("RELIGION_BREAKDOWN", Some(breakdown.build()?))?;

        let mut population = child(self.inputs.layout, REGION_PATH, "POPULATION")?;
        population.group()?.set_record("REGION_FACTORS", Some(factors.build()?))?;
        population.build()
    }

    /// Writes the rebel manager of a region: the default setup, or the setup of the rebels holding the region.
    fn write_rebel_manager(&self, region: &RegionEntry) -> Result<RecordNode> {
        let layout = self.inputs.layout;
        let path = format!("{REGION_PATH}/REGION_REBEL_MANAGER");
        let mut manager = layout.builder(&path)?;
        if let Some(setup) = rebel_setup(self.inputs.tables, region)? {
            let setup_path = format!("{path}/REBEL_SETUP");
            let mut name = child(layout, &setup_path, "CAMPAIGN_LOCALISATION")?;
            name.group()?.set_str("^+1", ABANDONED_REBELS_NAME)?.unset("^+2")?;
            let name = name.build()?;

            let mut record = layout.builder(&setup_path)?;
            record.group()?
                .set_str("^+2", setup.faction())?
                .set_str("^+3", setup.religion())?
                .set_u32("^+5", ABANDONED_REBELS_VALUE)?
                .set_record("CAMPAIGN_LOCALISATION", Some(name.clone()))?
                .set_record("CAMPAIGN_LOCALISATION@2", Some(name))?
                .set_record("FACTION_FLAG_AND_COLOURS", Some(write_flag(self.inputs, &setup_path, setup.flag())?))?;
            manager.group()?
                .set_record("REBEL_SETUP", Some(record.build()?))?
                .set_bool("REBEL_SETUP+1", true)?;
        }
        manager.build()
    }

    /// Writes the slot manager of a region: the primary slot, the port slot if the settlement has a port, then the
    /// secondary slots, with their buildings.
    fn write_slot_manager(&mut self, region: &RegionEntry, map_layout: &SettlementLayout) -> Result<RecordNode> {
        let inputs = self.inputs;
        let settlement = inputs.index.settlement_of(region.key());
        let primary = settlement.map(|settlement| settlement.primary_building().as_str()).filter(|building| !building.is_empty());
        let port = settlement.map(|settlement| settlement.port_building().as_str()).filter(|building| !building.is_empty());
        let others = settlement.map(|settlement| settlement.buildings().as_slice()).unwrap_or_default();

        let mut contents = vec![SlotContent { slot_type: "primary", building: primary }];
        if has_port(map_layout) {
            contents.push(SlotContent { slot_type: "port", building: port });
        }
        for index in 0..map_layout.slots().len().saturating_sub(2) {
            contents.push(SlotContent { slot_type: "secondary", building: others.get(index).map(String::as_str) });
        }

        let mut array = child(inputs.layout, &format!("{REGION_PATH}/REGION_SLOT_MANAGER"), "REGION_SLOT_ARRAY")?;
        for (index, content) in contents.iter().enumerate() {
            let slot = self.write_slot(region, index, content)?;
            array.push_group()?.set_record("REGION_SLOT", Some(slot))?;
        }

        let mut manager = child(inputs.layout, REGION_PATH, "REGION_SLOT_MANAGER")?;
        manager.group()?.set_record("REGION_SLOT_ARRAY", Some(array.build()?))?;
        manager.build()
    }

    /// Writes a region slot and its building.
    fn write_slot(&mut self, region: &RegionEntry, index: usize, content: &SlotContent) -> Result<RecordNode> {
        let layout = self.inputs.layout;
        let template = self.rows.templates.get(&(region.key().to_owned(), content.slot_type.to_owned())).cloned().unwrap_or_default();

        let mut residence = child(layout, SLOT_PATH, "GARRISON_RESIDENCE")?;
        residence.group()?.set_u32("^+1", next_id(&mut self.next_residence))?;

        let building = content.building.map(|building| self.write_building(region, building, content.slot_type == "primary")).transpose()?;
        let mut manager = child(layout, SLOT_PATH, "REGION_BUILDING_MANAGER")?;
        manager.group()?
            .set_bool("^+2", building.is_some())?
            .set_bool("^+5", building.is_some())?
            .set_bool("^+6", region.owner().is_none())?
            .set_record("BUILDING_BASE", building)?;

        let mut record = layout.builder(SLOT_PATH)?;
        record.group()?
            .set_u32("^+1", next_id(&mut self.next_slot))?
            .set_record("GARRISON_RESIDENCE", Some(residence.build()?))?
            .set_record("REGION_BUILDING_MANAGER", Some(manager.build()?))?
            .set_str("REGION_BUILDING_MANAGER+1", &format!("{}:{index}", region.key()))?
            .set_str("REGION_BUILDING_MANAGER+2", &template)?
            .set_str("REGION_BUILDING_MANAGER+3", content.slot_type)?;
        record.build()
    }

    /// Writes a building of a region slot (`BUILDING_BASE`).
    ///
    /// Buildings of unowned regions belong to the first faction of the world. The two floats of the record (meaning
    /// unknown) are the level's `create_time`, raised to its `level` plus 3, plus 1 in the primary slot.
    fn write_building(&mut self, region: &RegionEntry, building: &str, in_primary_slot: bool) -> Result<RecordNode> {
        let owner = region.owner().as_deref()
            .or_else(|| self.plan.factions().first().map(|faction| faction.key().as_str()))
            .unwrap_or_default();
        let government = self.rows.governments.get(owner).map(String::as_str).unwrap_or_default();
        let (create_time, level) = self.rows.building_levels.get(building).copied()
            .ok_or_else(|| RLibError::StartposInvalidValue("building_levels".to_owned(), "level_name".to_owned(), building.to_owned()))?;
        let value = create_time.max(level + BUILDING_BASE_VALUE_OFFSET + i64::from(in_primary_slot)) as f32;

        let mut record = child(self.inputs.layout, &format!("{SLOT_PATH}/REGION_BUILDING_MANAGER"), "BUILDING_BASE")?;
        record.group()?
            .set_u32("^+1", next_id(&mut self.next_building))?
            .set_str("^+2", building)?
            .set_f32("^+3", value)?
            .set_f32("^+4", value)?
            .set_str("^+5", owner)?
            .set_str("^+6", government)?;
        record.build()
    }

    /// Writes the settlement of a region: its residence (id after the ones of the region's slots, the general's army
    /// starting in it and its garrison forces), name and expansion data.
    fn write_settlement(&mut self, region: &RegionEntry, map_layout: &SettlementLayout) -> Result<RecordNode> {
        let layout = self.inputs.layout;
        let key = region.key();
        let settlement_key = self.inputs.index.settlement_of(key)
            .and_then(|settlement| self.rows.settlement_keys.get(settlement.id()))
            .cloned()
            .unwrap_or_default();

        let mut localisation = child(layout, SETTLEMENT_PATH, "CAMPAIGN_LOCALISATION")?;
        localisation.group()?.set_str("^+1", &settlement_name_key(&settlement_key, region))?;

        let zone = write_cells(self.inputs, &format!("{SETTLEMENT_PATH}/GARRISON_RESIDENCE/ZOE_BLOCK"), &garrison_zone(map_layout))?;
        let mut residence = child(layout, SETTLEMENT_PATH, "GARRISON_RESIDENCE")?;
        let force_id = |force: Option<&PlannedForce>| force.map(|force| force.id()).unwrap_or_default();
        residence.group()?
            .set_u32("^+1", next_id(&mut self.next_residence))?
            .set_u32("^+2", force_id(self.forces.stationed_in(key)))?
            .set_record("ZOE_BLOCK", Some(zone))?
            .set_record("LINE_OF_SIGHT", Some(write_line_of_sight(layout, &format!("{SETTLEMENT_PATH}/GARRISON_RESIDENCE"), self.sight.grid(), self.sight.settlement(key))?))?
            .set_u32("ZOE_BLOCK+3", force_id(self.forces.garrison_of(key, ForceKind::GarrisonArmy)))?
            .set_u32("ZOE_BLOCK+4", force_id(self.forces.garrison_of(key, ForceKind::GarrisonNavy)))?;

        let mut expansion = child(layout, SETTLEMENT_PATH, "SETTLEMENT_EXPANSION_MANAGER")?;
        expansion.group()?.set_record("SETTLEMENT_EXPANSION_MAP_DATA", Some(write_expansion_map_data(self.inputs, map_layout)?))?;

        let mut record = layout.builder(SETTLEMENT_PATH)?;
        record.group()?
            .set_record("GARRISON_RESIDENCE", Some(residence.build()?))?
            .set_u32("GARRISON_RESIDENCE+1", self.plan.region_id(key).unwrap_or_default())?
            .set_record("CAMPAIGN_LOCALISATION", Some(localisation.build()?))?
            .set_str("CAMPAIGN_LOCALISATION+2", &settlement_key)?
            .set_record("SETTLEMENT_EXPANSION_MANAGER", Some(expansion.build()?))?;
        record.build()
    }
}

/// Writes the expansion map data of a settlement: its slots and its sprawl cells.
///
/// Each sprawl cell gets the two edge masks of the map, its road pieces (the edges of both masks leading to other
/// sprawl cells) and its coast pieces (the edges leading to water cells, the ones outside every campaign region).
fn write_expansion_map_data(inputs: &WorldInputs, map_layout: &SettlementLayout) -> Result<RecordNode> {
    let mut secondary = child(inputs.layout, EXPANSION_PATH, "SECONDARY_SLOT_ARRAY")?;
    for slot in map_layout.slots().iter().skip(2) {
        secondary.push_group()?.set_record("SLOT_MAP_DATA", Some(write_slot_map_data(inputs, &format!("{EXPANSION_PATH}/SECONDARY_SLOT_ARRAY/SLOT_MAP_DATA"), slot)?))?;
    }

    let sprawl_path = format!("{EXPANSION_PATH}/SPRAWL_BLOCK");
    let mut sprawl = inputs.layout.builder(&sprawl_path)?;
    let sprawl_cells = map_layout.sprawl().iter().map(|cell| cell.cell()).collect::<HashSet<_>>();
    let is_water = |cell: HexCell| inputs.region_map.region_at(cell).is_some_and(|region| inputs.index.region(region).is_none());
    for cell in map_layout.sprawl() {
        let hex_cell = cell.cell();
        let [first_edges, second_edges] = inputs.region_map.edge_masks(hex_cell);
        let roads = (first_edges | second_edges) & hex_cell.neighbour_mask(|neighbour| sprawl_cells.contains(&neighbour));
        let coast = hex_cell.neighbour_mask(is_water);
        let road_piece = EdgePiece::from_mask(roads);
        let coast_piece = EdgePiece::from_mask(coast);

        let mut hex = child(inputs.layout, &sprawl_path, "SPRAWL_HEX_MAP_DATA")?;
        hex.group()?
            .set_u32("^+1", u32::from(hex_cell.x()))?
            .set_u32("^+2", u32::from(hex_cell.y()))?
            .set_bool("^+3", cell.in_footprint())?
            .set_bool("^+4", cell.in_zone())?
            .set_u32("^+5", u32::from(first_edges))?
            .set_u32("^+6", u32::from(second_edges))?
            .set_bool("^+7", is_water(hex_cell))?
            .set_u32("^+8", u32::from(roads))?
            .set_i32("^+9", i32::from(road_piece.shape()))?
            .set_u32("^+10", u32::from(road_piece.rotation()))?
            .set_u32("^+11", cell.zone_slots())?
            .set_u32("^+12", u32::from(coast))?
            .set_i32("^+13", i32::from(coast_piece.shape()))?
            .set_u32("^+14", u32::from(coast_piece.rotation()))?;
        sprawl.push_group()?
            .set_u32("^+1", u32::from(hex_cell.x()))?
            .set_u32("^+2", u32::from(hex_cell.y()))?
            .set_record("SPRAWL_HEX_MAP_DATA", Some(hex.build()?))?;
    }

    let slot_path = format!("{EXPANSION_PATH}/SLOT_MAP_DATA");
    let mut record = inputs.layout.builder(EXPANSION_PATH)?;
    record.group()?
        .set_str("^+1", map_layout.region())?
        .set_record("SLOT_MAP_DATA", Some(write_slot_map_data(inputs, &slot_path, &map_layout.slots()[0])?))?
        .set_record("SLOT_MAP_DATA@2", Some(write_slot_map_data(inputs, &slot_path, &map_layout.slots()[1])?))?
        .set_record("SECONDARY_SLOT_ARRAY", Some(secondary.build()?))?
        .set_record("SPRAWL_BLOCK", Some(sprawl.build()?))?;
    record.build()
}

/// Writes the map data of a slot (`SLOT_MAP_DATA`): id, position, footprint, zone, walls and touching slots.
fn write_slot_map_data(inputs: &WorldInputs, path: &str, slot: &SlotLayout) -> Result<RecordNode> {
    let (x, y) = slot.position().map(|cell| (u32::from(cell.x()), u32::from(cell.y()))).unwrap_or((NO_POSITION, NO_POSITION));
    let mut record = inputs.layout.builder(path)?;
    record.group()?
        .set_u32("^+1", slot.id())?
        .set_u32("^+2", x)?
        .set_u32("^+3", y)?
        .set_record("SLOT_FOOTPRINT_ARRAY", Some(write_cells(inputs, &format!("{path}/SLOT_FOOTPRINT_ARRAY"), slot.footprint())?))?
        .set_record("SLOT_ZOE_ARRAY", Some(write_cells(inputs, &format!("{path}/SLOT_ZOE_ARRAY"), slot.zone())?))?
        .set_record("SLOT_WALL_ARRAY", Some(write_cells(inputs, &format!("{path}/SLOT_WALL_ARRAY"), slot.walls())?))?
        .set_u32("SLOT_WALL_ARRAY+1", slot.touching_slots())?;
    record.build()
}

/// Writes a cell array: one group per cell, with its column and row.
fn write_cells(inputs: &WorldInputs, path: &str, cells: &[HexCell]) -> Result<RecordNode> {
    let mut record = inputs.layout.builder(path)?;
    for cell in cells {
        record.push_group()?
            .set_u32("^+1", u32::from(cell.x()))?
            .set_u32("^+2", u32::from(cell.y()))?;
    }
    record.build()
}

/// Returns the zone of control of a settlement's garrison: the main slot's footprint and zone, and the port's zone.
fn garrison_zone(map_layout: &SettlementLayout) -> Vec<HexCell> {
    let mut seen = HashSet::new();
    let main = &map_layout.slots()[0];
    let port = &map_layout.slots()[1];
    main.footprint().iter().chain(main.zone()).chain(port.zone())
        .filter(|cell| seen.insert(**cell))
        .copied()
        .collect()
}

/// Returns the garrison residence id of each region's settlement, by region key. The residence counter numbers each
/// region's slots, then its settlement, in region order.
///
/// # Errors
///
/// Returns an error if a region has no settlement layout.
pub(crate) fn settlement_residences(inputs: &WorldInputs, plan: &WorldPlan) -> Result<HashMap<String, u32>> {
    let mut residences = HashMap::new();
    let mut next_residence = 1;
    for region in plan.regions() {
        let map_layout = settlement_layout(inputs, region)?;
        let slots = 1 + usize::from(has_port(map_layout)) + map_layout.slots().len().saturating_sub(2);
        next_residence += u32::try_from(slots).unwrap_or(u32::MAX);
        residences.insert(region.key().to_owned(), next_id(&mut next_residence));
    }
    Ok(residences)
}

/// Returns the `start_pos_settlements.settlement_id` of every settlement, by row id.
///
/// # Errors
///
/// Returns an error if the table or a column is missing.
pub(crate) fn settlement_keys(inputs: &WorldInputs) -> Result<HashMap<String, String>> {
    let mut keys = HashMap::new();
    for row in inputs.tables.table("start_pos_settlements")?.rows() {
        keys.insert(row.str("id")?.into_owned(), row.str("settlement_id")?.into_owned());
    }
    Ok(keys)
}

/// Returns the localisation key of the name of a region's settlement, from its `settlement_id`.
pub(crate) fn settlement_name_key(settlement_key: &str, region: &RegionEntry) -> String {
    format!("{SETTLEMENT_NAME_PREFIX}{settlement_key}{}", region.id())
}

/// Returns the settlement layout of a region.
fn settlement_layout<'a>(inputs: &WorldInputs<'a>, region: &RegionEntry) -> Result<&'a SettlementLayout> {
    inputs.settlements.iter().find(|settlement| settlement.region() == region.key())
        .ok_or_else(|| RLibError::StartposInvalidValue("map_data".to_owned(), "SETTLEMENT_INFO".to_owned(), region.key().to_owned()))
}

/// Returns if a settlement has a port (its second primary slot has a position).
fn has_port(map_layout: &SettlementLayout) -> bool {
    map_layout.slots().get(1).is_some_and(|slot| slot.position().is_some())
}

/// Returns the number of horde buildings of the campaign. The game numbers them before the region buildings, with
/// the same counter.
fn horde_building_count(inputs: &WorldInputs) -> Result<u32> {
    let mut count = 0;
    for row in horde_details(inputs)? {
        for column in HORDE_BUILDING_COLUMNS {
            if !row.str(column)?.is_empty() {
                count += 1;
            }
        }
    }
    Ok(count)
}
