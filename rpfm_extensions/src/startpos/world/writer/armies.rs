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

use super::forces::{ForcePlan, PlannedForce, PlannedUnit};
use super::{child, WorldInputs};
use crate::startpos::world::FactionEntry;
use crate::startpos::world::legacy::{legacies, legacy_skill_set, ForceKind, Legacy, LegacyForce};

/// Path of the forces of a faction.
const ARMY_ARRAY_PATH: &str = "CAMPAIGN_STARTPOS/CAMPAIGN_ENV/CAMPAIGN_MODEL/WORLD/FACTION_ARRAY/FACTION/ARMY_ARRAY";

/// Path of a force.
const FORCE_PATH: &str = "CAMPAIGN_STARTPOS/CAMPAIGN_ENV/CAMPAIGN_MODEL/WORLD/FACTION_ARRAY/FACTION/ARMY_ARRAY/MILITARY_FORCE";

/// Path of a unit.
const UNIT_PATH: &str = "CAMPAIGN_STARTPOS/CAMPAIGN_ENV/CAMPAIGN_MODEL/WORLD/FACTION_ARRAY/FACTION/ARMY_ARRAY/MILITARY_FORCE/UNIT_CONTAINER/UNITS_ARRAY/UNIT";

/// Path of a force's legacy.
const LEGACY_PATH: &str = "CAMPAIGN_STARTPOS/CAMPAIGN_ENV/CAMPAIGN_MODEL/WORLD/FACTION_ARRAY/FACTION/ARMY_ARRAY/MILITARY_FORCE/MILITARY_FORCE_LEGACY";

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
    /// * `residences` - Garrison residence id of each region's settlement, by region key.
    ///
    /// # Errors
    ///
    /// Returns an error if a table or column is missing.
    pub fn new(inputs: &'b WorldInputs<'a>, forces: &'b ForcePlan<'a>, residences: &'b HashMap<String, u32>) -> Result<Self> {
        Ok(Self { inputs, forces, residences, stats: unit_stats(inputs)?, start_year: start_year(inputs)? })
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
            .set_u32("^+1", force.id())?
            .set_record("CAMPAIGN_SKILLS", Some(skills.build()?))?
            .set_record("CAMPAIGN_LOCALISATION", Some(name.build()?))?
            .set_str("CAMPAIGN_LOCALISATION+1", legacy.override_name())?
            .set_u32("CAMPAIGN_LOCALISATION+2", force.kind() as u32)?
            .set_str("CAMPAIGN_LOCALISATION+3", legacy.emblem())?
            .set_u32("CAMPAIGN_LOCALISATION+4", legacy.number())?;
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
