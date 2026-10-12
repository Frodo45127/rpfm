//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Plan of the characters and forces of the world: their order, runtime ids and commanders.
//!
//! Character ids go first to the living table characters (faction by faction, the leader first), then to one colonel
//! per garrison force. Force ids go first to the generals' forces (in their commanders' order), then to the garrison
//! forces (faction by faction, region by region, the army before the navy).

use std::collections::HashMap;

use getset::{CopyGetters, Getters};

use rpfm_lib::error::Result;

use super::WorldPlan;
use crate::startpos::tables::StartposTables;
use crate::startpos::world::{CampaignIndex, CharacterEntry, FactionEntry};
use crate::startpos::world::bodyguards::bodyguard;
use crate::startpos::world::characters::{placement, Placement};
use crate::startpos::world::garrisons::garrisons;
use crate::startpos::world::government::faction_leader;
use crate::startpos::world::legacy::ForceKind;

#[cfg(test)] #[path = "forces_test.rs"] pub(crate) mod forces_test;

/// Percentage of men of a full unit.
const FULL_UNIT: u32 = 100;

//---------------------------------------------------------------------------//
//                              Types
//---------------------------------------------------------------------------//

/// Characters and forces of the world, in startpos order.
#[derive(Debug, Clone, Default, Getters)]
#[getset(get = "pub")]
pub struct ForcePlan<'a> {

    /// Characters, in id order.
    characters: Vec<PlannedCharacter<'a>>,

    /// Forces, in id order.
    forces: Vec<PlannedForce<'a>>,
}

/// A character of the world.
#[derive(Debug, Clone, Getters, CopyGetters)]
pub struct PlannedCharacter<'a> {

    /// Runtime id.
    #[getset(get_copy = "pub")]
    id: u32,

    /// Faction of the character.
    #[getset(get_copy = "pub")]
    faction: &'a FactionEntry,

    /// Faction the character was created in: its table faction, also for wives moved to their husband's.
    #[getset(get_copy = "pub")]
    origin: &'a FactionEntry,

    /// Table row of the character, `None` for generated colonels.
    #[getset(get_copy = "pub")]
    entry: Option<&'a CharacterEntry>,

    /// Position of the character's force in the plan, if it commands one.
    #[getset(get_copy = "pub")]
    force: Option<usize>,

    /// The character was moved to its faction after the world was created, so it comes after the faction's own.
    moved: bool,
}

/// A force of the world.
#[derive(Debug, Clone, Getters, CopyGetters)]
pub struct PlannedForce<'a> {

    /// Runtime id.
    #[getset(get_copy = "pub")]
    id: u32,

    /// Faction of the force.
    #[getset(get_copy = "pub")]
    faction: &'a FactionEntry,

    /// Kind of force.
    #[getset(get_copy = "pub")]
    kind: ForceKind,

    /// Region of the settlement a garrison force defends.
    #[getset(get = "pub")]
    region: Option<String>,

    /// Region of the settlement a general's army starts in.
    #[getset(get = "pub")]
    stationed_in: Option<String>,

    /// Units, in startpos order: a general's bodyguard or flagship first.
    #[getset(get = "pub")]
    units: Vec<PlannedUnit>,

    /// Runtime id of the force's first unit.
    #[getset(get_copy = "pub")]
    first_unit_id: u32,

    /// Position of the force's commander in the plan.
    #[getset(get_copy = "pub")]
    commander: usize,

    /// The force was moved to its faction with its commander, so it comes after the faction's own.
    moved: bool,
}

/// A unit of a force.
#[derive(Debug, Clone, PartialEq, Eq, Getters, CopyGetters)]
pub struct PlannedUnit {

    /// Unit key.
    #[getset(get = "pub")]
    unit: String,

    /// Percentage of the unit's men present (`start_pos_land_units.soldiers`; 100 for the others).
    #[getset(get_copy = "pub")]
    soldiers: u32,

    /// The unit is the commander's own: a general's bodyguard or flagship, or a colonel's first unit.
    #[getset(get_copy = "pub")]
    commands: bool,
}

//---------------------------------------------------------------------------//
//                              Implementations
//---------------------------------------------------------------------------//

impl<'a> ForcePlan<'a> {

    /// Plans the characters and forces of a world.
    ///
    /// # Arguments
    ///
    /// * `tables` - Tables of the game and the campaign.
    /// * `index` - The campaign's startpos rows.
    /// * `plan` - Order of the factions and regions.
    ///
    /// # Errors
    ///
    /// Returns an error if a table or column is missing.
    pub fn new(tables: &StartposTables, index: &'a CampaignIndex, plan: &WorldPlan<'a>) -> Result<Self> {
        let mut land_units = HashMap::<String, Vec<PlannedUnit>>::new();
        for row in tables.table("start_pos_land_units")?.rows() {
            let soldiers = u32::try_from(row.int("soldiers")?).unwrap_or(FULL_UNIT);
            land_units.entry(row.str("general")?.into_owned()).or_default().push(PlannedUnit::new(row.str("unit_type")?.into_owned(), soldiers));
        }
        let mut naval_units = HashMap::<String, Vec<PlannedUnit>>::new();
        for row in tables.table("start_pos_naval_units")?.rows() {
            naval_units.entry(row.str("admiral")?.into_owned()).or_default().push(PlannedUnit::new(row.str("unit_type")?.into_owned(), FULL_UNIT));
        }

        let mut force_plan = Self::default();
        for faction in plan.factions() {
            let bodyguards = [bodyguard(tables, faction, false)?, bodyguard(tables, faction, true)?];
            for character in characters_in_startpos_order(index, plan, faction) {
                let (kind, units) = match (land_units.get(character.id()), naval_units.get(character.id())) {
                    (Some(units), _) => (Some(ForceKind::Army), units.as_slice()),
                    (None, Some(units)) => (Some(ForceKind::Navy), units.as_slice()),
                    (None, None) => (None, [].as_slice()),
                };
                let commander = force_plan.characters.len();
                let force = kind.map(|kind| {
                    let override_unit = Some(character.override_general_unit()).filter(|unit| !unit.is_empty());
                    let own_unit = override_unit.cloned().or_else(|| bodyguards[usize::from(kind == ForceKind::Navy)].as_ref().map(|bodyguard| bodyguard.unit().to_owned()));
                    let mut all_units = own_unit.map(|unit| PlannedUnit { unit, soldiers: FULL_UNIT, commands: true }).into_iter().collect::<Vec<_>>();
                    all_units.extend(grouped(units));
                    force_plan.push_force(faction, kind, None, all_units, commander)
                });
                if let (Some(force), Placement::Settlement(region)) = (force, placement(tables, index, character)?) {
                    if force_plan.forces[force].kind == ForceKind::Army {
                        force_plan.forces[force].stationed_in = Some(region);
                    }
                }
                force_plan.push_character(faction, Some(character), force);
            }
        }

        let garrisons = garrisons(tables, index)?.into_iter().map(|garrison| (garrison.region().to_owned(), garrison)).collect::<HashMap<_, _>>();
        for faction in plan.factions() {
            for region in plan.regions().iter().filter(|region| region.owner().as_deref() == Some(faction.key().as_str())) {
                let Some(garrison) = garrisons.get(region.key()) else { continue };
                force_plan.push_garrison(faction, ForceKind::GarrisonArmy, region.key(), garrison.army().to_vec());
                if !garrison.navy().is_empty() {
                    force_plan.push_garrison(faction, ForceKind::GarrisonNavy, region.key(), garrison.navy().to_vec());
                }
            }
        }

        force_plan.move_wives(index);
        Ok(force_plan)
    }

    /// Moves the women married to a man of another faction, with their forces, to their husband's faction.
    fn move_wives(&mut self, index: &'a CampaignIndex) {
        for position in 0..self.characters.len() {
            let character = &self.characters[position];
            let Some(husband_faction) = character.entry.and_then(|entry| entry.husband_faction().as_deref()).and_then(|key| index.faction(key)) else { continue };
            let force = character.force;
            self.characters[position].faction = husband_faction;
            self.characters[position].moved = true;
            if let Some(force) = force {
                self.forces[force].faction = husband_faction;
                self.forces[force].moved = true;
            }
        }
    }

    /// Adds a garrison force and its colonel, who commands its first unit.
    fn push_garrison(&mut self, faction: &'a FactionEntry, kind: ForceKind, region: &str, units: Vec<String>) {
        let commander = self.characters.len();
        let units = units.into_iter().enumerate().map(|(index, unit)| PlannedUnit { unit, soldiers: FULL_UNIT, commands: index == 0 }).collect();
        let force = self.push_force(faction, kind, Some(region.to_owned()), units, commander);
        self.push_character(faction, None, Some(force));
    }

    /// Adds a force, returning its position. Unit ids continue the ones of the previous force.
    fn push_force(&mut self, faction: &'a FactionEntry, kind: ForceKind, region: Option<String>, units: Vec<PlannedUnit>, commander: usize) -> usize {
        let id = u32::try_from(self.forces.len() + 1).unwrap_or(u32::MAX);
        let first_unit_id = self.forces.last().map(|force| force.first_unit_id + u32::try_from(force.units.len()).unwrap_or(u32::MAX)).unwrap_or(1);
        self.forces.push(PlannedForce { id, faction, kind, region, stationed_in: None, units, first_unit_id, commander, moved: false });
        self.forces.len() - 1
    }

    /// Adds a character.
    fn push_character(&mut self, faction: &'a FactionEntry, entry: Option<&'a CharacterEntry>, force: Option<usize>) {
        let id = u32::try_from(self.characters.len() + 1).unwrap_or(u32::MAX);
        self.characters.push(PlannedCharacter { id, faction, origin: faction, entry, force, moved: false });
    }

    /// Returns the positions in the plan of the characters of a faction, in startpos order: its own, then the ones
    /// moved to it.
    pub fn character_positions(&self, faction: &str) -> Vec<usize> {
        let of = |moved: bool| self.characters.iter().enumerate()
            .filter(move |(_, character)| *character.faction.key() == faction && character.moved == moved)
            .map(|(position, _)| position);
        of(false).chain(of(true)).collect()
    }

    /// Returns the characters of a faction, in startpos order: its own, then the ones moved to it.
    pub fn characters_of(&self, faction: &str) -> impl Iterator<Item = &PlannedCharacter<'a>> {
        self.character_positions(faction).into_iter().map(|position| &self.characters[position])
    }

    /// Returns the forces of a faction, in startpos order: its own, then the ones moved to it.
    pub fn forces_of(&self, faction: &str) -> impl Iterator<Item = &PlannedForce<'a>> {
        let of = |moved: bool| {
            let faction = faction.to_owned();
            self.forces.iter().filter(move |force| *force.faction.key() == faction && force.moved == moved)
        };
        of(false).chain(of(true))
    }

    /// Returns the first general's army starting in a region's settlement, if any.
    pub fn stationed_in(&self, region: &str) -> Option<&PlannedForce<'a>> {
        self.forces.iter().find(|force| force.stationed_in.as_deref() == Some(region))
    }

    /// Returns the garrison force of a kind defending a region, if any.
    pub fn garrison_of(&self, region: &str, kind: ForceKind) -> Option<&PlannedForce<'a>> {
        self.forces.iter().find(|force| force.kind == kind && force.region.as_deref() == Some(region))
    }
}

impl PlannedForce<'_> {

    /// Returns the runtime ids of the force's units, in startpos order. A general's own unit is created after the
    /// others, so it takes the last id although it comes first.
    pub fn unit_ids(&self) -> Vec<u32> {
        let count = u32::try_from(self.units.len()).unwrap_or(u32::MAX);
        let general = matches!(self.kind, ForceKind::Army | ForceKind::Navy) && self.units.first().is_some_and(|unit| unit.commands);
        (0..count).map(|index| match (general, index) {
            (true, 0) => self.first_unit_id + count - 1,
            (true, index) => self.first_unit_id + index - 1,
            (false, index) => self.first_unit_id + index,
        }).collect()
    }

    /// Returns the runtime id of the commander's own unit, if the force has one.
    pub fn commander_unit_id(&self) -> Option<u32> {
        self.units.iter().position(|unit| unit.commands).and_then(|position| self.unit_ids().get(position).copied())
    }
}

impl PlannedUnit {

    /// Creates a unit that isn't its commander's.
    fn new(unit: String, soldiers: u32) -> Self {
        Self { unit, soldiers, commands: false }
    }
}

/// Returns units with the same key next to each other, in the order each key first appears (the game uses a hash
/// order).
fn grouped(units: &[PlannedUnit]) -> Vec<PlannedUnit> {
    let mut keys = Vec::<&str>::new();
    for unit in units {
        if !keys.contains(&unit.unit.as_str()) {
            keys.push(&unit.unit);
        }
    }
    keys.iter().flat_map(|key| units.iter().filter(move |unit| unit.unit == *key).cloned()).collect()
}

/// Returns the living characters of a faction in startpos order: the plan's order if it has one for the faction,
/// otherwise the leader, then the others in table order (the game uses a hash order for them). Living characters
/// left out of the plan's order go after it, in table order.
fn characters_in_startpos_order<'a>(index: &'a CampaignIndex, plan: &WorldPlan<'a>, faction: &'a FactionEntry) -> Vec<&'a CharacterEntry> {
    let first = match plan.character_order(faction.key()) {
        Some(order) => order.to_vec(),
        None => faction_leader(index, faction.key()).into_iter().collect(),
    };
    let mut characters = Vec::<&CharacterEntry>::new();
    for character in first.into_iter().chain(index.characters_of(faction.key()).filter(|character| character.death_type().is_empty())) {
        if !characters.contains(&character) {
            characters.push(character);
        }
    }
    characters
}
