//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Writer of the forces of a faction (`FACTION/ARMY_ARRAY`): their units and legacies.

use std::collections::HashMap;

use rpfm_lib::error::{RLibError, Result};
use rpfm_lib::files::esf::RecordNode;

use super::characters::CharacterWriter;
use super::forces::{ForcePlan, PlannedForce, PlannedUnit};
use super::regions::{settlement_keys, settlement_name_key};
use super::{child, horde_details, next_id, start_pos_faction_row, WorldInputs, HORDE_BUILDING_COLUMNS};
use crate::startpos::world::FactionEntry;
use crate::startpos::world::factions::faction_summary;
use crate::startpos::world::legacy::{legacies, legacy_skill_set, ForceKind, Legacy, LegacyForce};

/// Path of the forces of a faction.
const ARMY_ARRAY_PATH: &str = "CAMPAIGN_STARTPOS/CAMPAIGN_ENV/CAMPAIGN_MODEL/WORLD/FACTION_ARRAY/FACTION/ARMY_ARRAY";

/// Path of a force.
const FORCE_PATH: &str = "CAMPAIGN_STARTPOS/CAMPAIGN_ENV/CAMPAIGN_MODEL/WORLD/FACTION_ARRAY/FACTION/ARMY_ARRAY/MILITARY_FORCE";

/// Path of a unit.
const UNIT_PATH: &str = "CAMPAIGN_STARTPOS/CAMPAIGN_ENV/CAMPAIGN_MODEL/WORLD/FACTION_ARRAY/FACTION/ARMY_ARRAY/MILITARY_FORCE/UNIT_CONTAINER/UNITS_ARRAY/UNIT";

/// Path of a force's legacy.
const LEGACY_PATH: &str = "CAMPAIGN_STARTPOS/CAMPAIGN_ENV/CAMPAIGN_MODEL/WORLD/FACTION_ARRAY/FACTION/ARMY_ARRAY/MILITARY_FORCE/MILITARY_FORCE_LEGACY";

/// Path of a force's legacy history.
const HISTORY_PATH: &str = "CAMPAIGN_STARTPOS/CAMPAIGN_ENV/CAMPAIGN_MODEL/WORLD/FACTION_ARRAY/FACTION/ARMY_ARRAY/MILITARY_FORCE/MILITARY_FORCE_LEGACY/MILITARY_FORCE_LEGACY_HISTORY";

/// Path of a force's horde details.
const HORDE_PATH: &str = "CAMPAIGN_STARTPOS/CAMPAIGN_ENV/CAMPAIGN_MODEL/WORLD/FACTION_ARRAY/FACTION/ARMY_ARRAY/MILITARY_FORCE/MILITARY_FORCE_HORDE_DETAILS";

/// Path of a building slot of a horde.
const HORDE_SLOT_PATH: &str = "CAMPAIGN_STARTPOS/CAMPAIGN_ENV/CAMPAIGN_MODEL/WORLD/FACTION_ARRAY/FACTION/ARMY_ARRAY/MILITARY_FORCE/MILITARY_FORCE_HORDE_DETAILS/MILITARY_FORCE_SLOT_MANAGER/SLOT_BLOCK/MILITARY_FORCE_SLOT";

/// Slot type of a horde's primary building slot.
const HORDE_PRIMARY_SLOT: &str = "horde_primary";

/// Slot type of a horde's other building slots.
const HORDE_SECONDARY_SLOT: &str = "horde_secondary";

/// Value of the two floats of a horde building (`BUILDING_BASE^+3` and `^+4`, meaning unknown).
const HORDE_BUILDING_VALUE: f32 = 1.0;

/// Action point multiplier of the units of generals' forces. Garrison units keep their base action points.
const GENERAL_ACTION_POINTS_MULTIPLIER: f32 = 1.1;

//---------------------------------------------------------------------------//
//                              Types
//---------------------------------------------------------------------------//

/// Writes the forces of the factions.
pub struct ArmyWriter<'a, 'b> {

    /// Tables, startpos rows and layouts.
    inputs: &'b WorldInputs<'a>,

    /// Characters and forces of the world.
    forces: &'b ForcePlan<'a>,

    /// Garrison residence id of each region's settlement, by region key.
    residences: &'b HashMap<String, u32>,

    /// Men and action points of every unit, by `main_units` key.
    stats: HashMap<String, UnitStats>,

    /// The campaign's start year.
    start_year: u32,

    /// Horde details of the horde armies, by force id.
    hordes: HashMap<u32, HordePlan>,

    /// Writer of the characters, for the commanders' positions and names.
    characters: &'b CharacterWriter<'a, 'b>,

    /// `start_pos_settlements.settlement_id` of every settlement, by row id.
    settlement_keys: HashMap<String, String>,

    /// Province of every region, by region key.
    provinces: HashMap<String, String>,
}

/// The horde details of a horde army: ids of its recruitment manager and slots, its buildings and development.
#[derive(Debug, Clone, Default)]
struct HordePlan {

    /// Recruitment manager id.
    recruitment_id: u32,

    /// Id of the first building slot (the others follow it).
    first_slot_id: u32,

    /// Building of each slot, with its id: the row's buildings in column order, packed into the first slots.
    buildings: Vec<Option<(String, u32)>>,

    /// Development points (`start_pos_horde_details.starting_dev_points` plus one).
    development: u32,

    /// Government type of the faction, for its buildings.
    government: String,
}

/// Men and action points of a unit (`main_units` and its land or naval unit).
#[derive(Debug, Clone, Copy, Default)]
struct UnitStats {

    /// Men of a full unit (`main_units.num_men`).
    men: u32,

    /// Base action points (`campaign_action_points` of the land or naval unit).
    action_points: u32,
}

//---------------------------------------------------------------------------//
//                              Implementations
//---------------------------------------------------------------------------//

impl<'a, 'b> ArmyWriter<'a, 'b> {

    /// Prepares the writer of the forces.
    ///
    /// # Arguments
    ///
    /// * `inputs` - Tables, startpos rows and layouts.
    /// * `forces` - Characters and forces of the world.
    /// * `characters` - Writer of the characters.
    /// * `residences` - Garrison residence id of each region's settlement, by region key.
    /// * `late_recruitment` - First recruitment manager id free after the provinces and seas, for horde armies
    ///   without horde details.
    ///
    /// # Errors
    ///
    /// Returns an error if a table or column is missing.
    pub fn new(inputs: &'b WorldInputs<'a>, forces: &'b ForcePlan<'a>, characters: &'b CharacterWriter<'a, 'b>, residences: &'b HashMap<String, u32>, late_recruitment: u32) -> Result<Self> {
        let mut provinces = HashMap::new();
        for row in inputs.tables.table("region_to_province_junctions")?.rows() {
            provinces.insert(row.str("region")?.into_owned(), row.str("province")?.into_owned());
        }

        Ok(Self {
            hordes: horde_plans(inputs, forces, late_recruitment)?,
            settlement_keys: settlement_keys(inputs)?,
            provinces,
            characters,
            stats: unit_stats(inputs)?,
            start_year: start_year(inputs)?,
            inputs,
            forces,
            residences,
        })
    }

    /// Writes the forces of a faction.
    ///
    /// # Errors
    ///
    /// Returns an error if a table, column or layout is missing.
    pub fn write_army_array(&self, faction: &FactionEntry) -> Result<RecordNode> {
        let faction_forces = self.forces.forces_of(faction.key()).collect::<Vec<_>>();
        let legacy_forces = faction_forces.iter().map(|force| LegacyForce {
            kind: force.kind(),
            general: self.forces.characters()[force.commander()].entry().map(|entry| entry.id().to_owned()),
        }).collect::<Vec<_>>();
        let force_legacies = legacies(self.inputs.tables, self.inputs.index, faction, &legacy_forces, self.inputs.legacy_overrides)?;

        let mut array = self.inputs.layout.builder(ARMY_ARRAY_PATH)?;
        for (force, legacy) in faction_forces.iter().zip(&force_legacies) {
            array.push_group()?.set_record("MILITARY_FORCE", Some(self.write_force(force, legacy)?))?;
        }
        array.build()
    }

    /// Writes a force: ids, legacy, units, the residences it belongs to and its kind.
    fn write_force(&self, force: &PlannedForce, legacy: &Legacy) -> Result<RecordNode> {
        let layout = self.inputs.layout;
        let residence = |region: Option<&String>| region.and_then(|region| self.residences.get(region)).copied().unwrap_or_default();
        let garrison_residence = |kind| residence(force.region().as_ref().filter(|_| force.kind() == kind));
        let commander_id = self.forces.characters()[force.commander()].id();

        let mut units = child(layout, &format!("{FORCE_PATH}/UNIT_CONTAINER"), "UNITS_ARRAY")?;
        for (unit, id) in force.units().iter().zip(force.unit_ids()) {
            let commander = if unit.commands() { commander_id } else { 0 };
            units.push_group()?.set_record("UNIT", Some(self.write_unit(force, unit, id, commander)?))?;
        }
        let mut container = child(layout, FORCE_PATH, "UNIT_CONTAINER")?;
        container.group()?.set_record("UNITS_ARRAY", Some(units.build()?))?;

        let mut record = layout.builder(FORCE_PATH)?;
        record.group()?
            .set_u32("^+1", force.id())?
            .set_u32("^+2", commander_id)?
            .set_record("MILITARY_FORCE_LEGACY", Some(self.write_legacy(force, legacy)?))?
            .set_record("UNIT_CONTAINER", Some(container.build()?))?
            .set_u32("UNIT_CONTAINER+1", residence(force.stationed_in().as_ref()))?
            .set_u32("UNIT_CONTAINER+11", garrison_residence(ForceKind::GarrisonArmy))?
            .set_u32("UNIT_CONTAINER+12", garrison_residence(ForceKind::GarrisonNavy))?
            .set_bool("UNIT_CONTAINER+13", is_navy(force))?;
        if let Some(horde) = self.hordes.get(&force.id()) {
            record.group()?
                .set_bool("MILITARY_FORCE_MORALE+1", true)?
                .set_record("MILITARY_FORCE_HORDE_DETAILS", Some(self.write_horde_details(force, horde)?))?;
        }
        record.build()
    }

    /// Writes the horde details of a horde army: its recruitment manager, building slots and development.
    fn write_horde_details(&self, force: &PlannedForce, horde: &HordePlan) -> Result<RecordNode> {
        let layout = self.inputs.layout;
        let mut recruitment = child(layout, HORDE_PATH, "RECRUITMENT_MANAGER")?;
        recruitment.group()?.set_u32("^+1", horde.recruitment_id)?;

        let mut slots = child(layout, &format!("{HORDE_PATH}/MILITARY_FORCE_SLOT_MANAGER"), "SLOT_BLOCK")?;
        for (index, building) in horde.buildings.iter().enumerate() {
            let primary = index == 0;
            let base = building.as_ref().map(|(key, id)| {
                let mut base = child(layout, &format!("{HORDE_SLOT_PATH}/MILITARY_FORCE_BUILDING_MANAGER"), "BUILDING_BASE")?;
                base.group()?
                    .set_u32("^+1", *id)?
                    .set_str("^+2", key)?
                    .set_f32("^+3", HORDE_BUILDING_VALUE)?
                    .set_f32("^+4", HORDE_BUILDING_VALUE)?
                    .set_str("^+5", force.faction().key())?
                    .set_str("^+6", &horde.government)?;
                base.build()
            }).transpose()?;

            let mut manager = child(layout, HORDE_SLOT_PATH, "MILITARY_FORCE_BUILDING_MANAGER")?;
            manager.group()?
                .set_bool("^+2", base.is_some())?
                .set_record("BUILDING_BASE", base)?;

            let slot_type = if primary { HORDE_PRIMARY_SLOT } else { HORDE_SECONDARY_SLOT };
            let mut slot = layout.builder(HORDE_SLOT_PATH)?;
            slot.group()?
                .set_u32("^+1", horde.first_slot_id + u32::try_from(index).unwrap_or_default())?
                .set_u32("^+2", u32::try_from(index).unwrap_or_default())?
                .set_record("MILITARY_FORCE_BUILDING_MANAGER", Some(manager.build()?))?
                .set_str("MILITARY_FORCE_BUILDING_MANAGER+1", slot_type)?
                .set_str("MILITARY_FORCE_BUILDING_MANAGER+2", slot_type)?
                .set_bool("MILITARY_FORCE_BUILDING_MANAGER+4", primary)?;
            slots.push_group()?.set_record("MILITARY_FORCE_SLOT", Some(slot.build()?))?;
        }
        let mut slot_manager = child(layout, HORDE_PATH, "MILITARY_FORCE_SLOT_MANAGER")?;
        slot_manager.group()?.set_record("SLOT_BLOCK", Some(slots.build()?))?;

        let mut development = child(layout, HORDE_PATH, "MILITARY_FORCE_DEVELOPMENT")?;
        development.group()?.set_u32("^+1", horde.development)?;

        let mut record = layout.builder(HORDE_PATH)?;
        record.group()?
            .set_record("RECRUITMENT_MANAGER", Some(recruitment.build()?))?
            .set_record("MILITARY_FORCE_SLOT_MANAGER", Some(slot_manager.build()?))?
            .set_record("MILITARY_FORCE_DEVELOPMENT", Some(development.build()?))?;
        record.build()
    }

    /// Writes the legacy of a force: its skill set, name, kind, emblem and number.
    fn write_legacy(&self, force: &PlannedForce, legacy: &Legacy) -> Result<RecordNode> {
        let layout = self.inputs.layout;
        let mut skills = child(layout, LEGACY_PATH, "CAMPAIGN_SKILLS")?;
        skills.group()?.set_str("^+2", &legacy_skill_set(force.faction(), is_navy(force)))?;

        let mut name = child(layout, LEGACY_PATH, "CAMPAIGN_LOCALISATION")?;
        name.group()?.set_str("^+1", legacy.name())?;

        let mut record = layout.builder(LEGACY_PATH)?;
        record.group()?
            .set_record("MILITARY_FORCE_LEGACY_HISTORY", Some(self.write_history(force)?))?
            .set_u32("^+1", force.id())?
            .set_record("CAMPAIGN_SKILLS", Some(skills.build()?))?
            .set_record("CAMPAIGN_LOCALISATION", Some(name.build()?))?
            .set_str("CAMPAIGN_LOCALISATION+1", legacy.override_name())?
            .set_u32("CAMPAIGN_LOCALISATION+2", force.kind() as u32)?
            .set_str("CAMPAIGN_LOCALISATION+3", legacy.emblem())?
            .set_u32("CAMPAIGN_LOCALISATION+4", legacy.number())?;
        record.build()
    }

    /// Writes where and when a force was raised (`MILITARY_FORCE_LEGACY_HISTORY`): the campaign's start (the game
    /// subtracts up to 5 random years), its commander's name, and the settlement and province of the region its
    /// commander stands on. Forces at sea use their faction's capital; at sea without capital, the history is empty.
    fn write_history(&self, force: &PlannedForce) -> Result<RecordNode> {
        let layout = self.inputs.layout;
        let index = self.inputs.index;
        let commander = &self.forces.characters()[force.commander()];
        let (_, cell) = self.characters.position(commander)?;
        let region = self.inputs.region_map.region_at(cell)
            .and_then(|key| index.region(key))
            .or_else(|| index.capital_of(force.faction().key()));

        let mut record = layout.builder(HISTORY_PATH)?;
        let mut localisation = child(layout, HISTORY_PATH, "CAMPAIGN_LOCALISATION")?;
        let Some(region) = region else {
            localisation.group()?.set_str("^+1", "")?;
            let mut date = child(layout, HISTORY_PATH, "DATE")?;
            date.group()?.set_u32("^+1", 0)?;
            record.group()?
                .set_record("DATE", Some(date.build()?))?
                .set_record("CHARACTER_NAME", Some(self.characters.write_name(HISTORY_PATH, None)?))?
                .set_record("CAMPAIGN_LOCALISATION", Some(localisation.build()?))?
                .set_str("CAMPAIGN_LOCALISATION+1", "")?;
            return record.build();
        };

        let settlement_key = index.settlement_of(region.key()).and_then(|settlement| self.settlement_keys.get(settlement.id())).cloned().unwrap_or_default();
        localisation.group()?.set_str("^+1", &settlement_name_key(&settlement_key, region))?.unset("^+2")?;
        let mut date = child(layout, HISTORY_PATH, "DATE")?;
        date.group()?.set_u32("^+1", self.start_year)?;
        record.group()?
            .set_record("DATE", Some(date.build()?))?
            .set_record("CHARACTER_NAME", Some(self.characters.write_name(HISTORY_PATH, self.characters.row(commander))?))?
            .set_record("CAMPAIGN_LOCALISATION", Some(localisation.build()?))?
            .set_str("CAMPAIGN_LOCALISATION+1", self.provinces.get(region.key()).map(String::as_str).unwrap_or_default())?;
        record.build()
    }

    /// Writes a unit: its id, key, recruitment date, owner, men and action points.
    ///
    /// Units start with `soldiers` percent of their men. Their action points are the base ones, raised by
    /// [`GENERAL_ACTION_POINTS_MULTIPLIER`] in generals' forces. The commander's own unit stores the commander's id.
    fn write_unit(&self, force: &PlannedForce, unit: &PlannedUnit, id: u32, commander: u32) -> Result<RecordNode> {
        let layout = self.inputs.layout;
        let stats = self.stats.get(unit.unit()).copied().unwrap_or_default();
        let multiplier = if matches!(force.kind(), ForceKind::Army | ForceKind::Navy) { GENERAL_ACTION_POINTS_MULTIPLIER } else { 1.0 };
        let action_points = (stats.action_points as f32 * multiplier).round() as u32;

        let mut key = child(layout, UNIT_PATH, "UNIT_RECORD_KEY")?;
        key.group()?.set_str("^+1", unit.unit())?;

        let mut date = child(layout, &format!("{UNIT_PATH}/UNIT_HISTORY"), "DATE")?;
        date.group()?.set_u32("^+1", self.start_year)?;
        let mut history = child(layout, UNIT_PATH, "UNIT_HISTORY")?;
        history.group()?.set_record("DATE", Some(date.build()?))?;

        let mut commander_details = child(layout, UNIT_PATH, "COMMANDER_DETAILS")?;
        commander_details.group()?.set_str("^+1", force.faction().key())?;

        let mut record = layout.builder(UNIT_PATH)?;
        record.group()?
            .set_u32("^+1", id)?
            .set_record("UNIT_RECORD_KEY", Some(key.build()?))?
            .set_record("UNIT_HISTORY", Some(history.build()?))?
            .set_record("COMMANDER_DETAILS", Some(commander_details.build()?))?
            .set_u32("COMMANDER_DETAILS+1", stats.men * unit.soldiers() / 100)?
            .set_u32("COMMANDER_DETAILS+2", stats.men)?
            .set_u32("COMMANDER_DETAILS+3", action_points)?
            .set_u32("COMMANDER_DETAILS+4", commander)?
            .set_u32("CAMPAIGN_LOCALISATION+1", stats.action_points)?
            .set_f32("CAMPAIGN_LOCALISATION+2", multiplier)?
            .set_u32("CAMPAIGN_LOCALISATION+3", id.saturating_sub(1))?;
        record.build()
    }
}

/// Returns if a force is a navy.
fn is_navy(force: &PlannedForce) -> bool {
    matches!(force.kind(), ForceKind::Navy | ForceKind::GarrisonNavy)
}

/// Plans the horde details of the horde armies: the generals' armies of horde factions (factions that can be
/// hordes and own no regions).
///
/// Recruitment manager, slot and building ids are counters over the horde armies in force order. Buildings fill the
/// slots in column order, without gaps for empty columns. Armies without a
/// `start_pos_horde_details` row get empty slots, and a recruitment manager id after the ones of the provinces and
/// seas.
fn horde_plans(inputs: &WorldInputs, forces: &ForcePlan, late_recruitment: u32) -> Result<HashMap<u32, HordePlan>> {
    let rows = horde_details(inputs)?.into_iter()
        .map(|row| Ok((row.str("general")?.into_owned(), row)))
        .collect::<Result<HashMap<_, _>>>()?;

    let mut horde_factions = HashMap::new();
    let (mut next_recruitment, mut late_recruitment, mut next_slot, mut next_building) = (1, late_recruitment, 1, 1);
    let mut plans = HashMap::new();
    for force in forces.forces().iter().filter(|force| force.kind() == ForceKind::Army) {
        let faction = force.faction();
        if !horde_factions.contains_key(faction.key()) {
            let is_horde = faction_summary(inputs.tables, inputs.index, faction)?.horde() == Some(true);
            horde_factions.insert(faction.key().to_owned(), is_horde);
        }
        if !horde_factions[faction.key()] {
            continue;
        }

        let row = forces.characters()[force.commander()].entry().and_then(|entry| rows.get(entry.id()));
        let mut buildings = vec![];
        for column in HORDE_BUILDING_COLUMNS {
            if let Some(building) = row.map(|row| row.str(column)).transpose()?.filter(|building| !building.is_empty()) {
                buildings.push(Some((building.into_owned(), next_id(&mut next_building))));
            }
        }
        buildings.resize(HORDE_BUILDING_COLUMNS.len(), None);

        let first_slot_id = next_slot;
        next_slot += u32::try_from(HORDE_BUILDING_COLUMNS.len()).unwrap_or_default();
        plans.insert(force.id(), HordePlan {
            recruitment_id: if row.is_some() { next_id(&mut next_recruitment) } else { next_id(&mut late_recruitment) },
            first_slot_id,
            buildings,
            development: u32::try_from(row.map(|row| row.int("starting_dev_points")).transpose()?.unwrap_or_default()).unwrap_or_default() + 1,
            government: start_pos_faction_row(inputs.tables, faction)?.str("government_type")?.into_owned(),
        });
    }
    Ok(plans)
}

/// Returns the men and base action points of every unit, by `main_units` key.
fn unit_stats(inputs: &WorldInputs) -> Result<HashMap<String, UnitStats>> {
    let mut land_points = HashMap::new();
    for row in inputs.tables.table("land_units")?.rows() {
        land_points.insert(row.str("key")?.into_owned(), row.int("campaign_action_points")?);
    }
    let mut naval_points = HashMap::new();
    for row in inputs.tables.table("naval_units")?.rows() {
        naval_points.insert(row.str("key")?.into_owned(), row.int("campaign_action_points")?);
    }

    let mut stats = HashMap::new();
    for row in inputs.tables.table("main_units")?.rows() {
        let points = if row.bool("is_naval")? { naval_points.get(row.str("naval_unit")?.as_ref()) } else { land_points.get(row.str("land_unit")?.as_ref()) };
        stats.insert(row.str("unit")?.into_owned(), UnitStats {
            men: u32::try_from(row.int("num_men")?).unwrap_or_default(),
            action_points: points.and_then(|points| u32::try_from(*points).ok()).unwrap_or_default(),
        });
    }
    Ok(stats)
}

/// Returns the campaign's start year (`start_pos_calendars.start_year`).
pub(crate) fn start_year(inputs: &WorldInputs) -> Result<u32> {
    for row in inputs.tables.table("start_pos_calendars")?.rows() {
        if row.str("campaign")? == *inputs.index.campaign() {
            return Ok(u32::try_from(row.int("start_year")?).unwrap_or_default());
        }
    }
    Err(RLibError::StartposInvalidValue("start_pos_calendars".to_owned(), "campaign".to_owned(), inputs.index.campaign().to_owned()))
}
