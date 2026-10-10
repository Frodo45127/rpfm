//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Writer of the AI's characters (`CAI_WORLD_CHARACTERS`), mobiles (`CAI_WORLD_RESOURCE_MOBILES`: forces, and agents
//! moving on their own) and units (`CAI_WORLD_UNITS`).

use std::collections::HashMap;

use rpfm_lib::error::Result;
use rpfm_lib::files::esf::RecordNode;

use super::factions::owned_direct;
use super::map::AreaGroups;
use super::plan::CaiPlan;
use super::world_view::{CharacterView, ForceView, WorldView};
use super::{component, CAI_WORLD_PATH};
use crate::startpos::coords::HexCell;
use crate::startpos::layout::{child_path, StartposLayout};
use crate::startpos::region_map::RegionMap;

#[cfg(test)] #[path = "mobiles_test.rs"] mod mobiles_test;

/// Agent codes (`CAI_CHARACTER^+7` and `^+8`) of characters moving with a force.
const FORCE_AGENT_CODES: (u32, u32) = (0, 3);

/// Agent codes of agents moving on their own (only spies are known).
const AGENT_CODES: (u32, u32) = (1, 0);

//---------------------------------------------------------------------------//
//                              Types
//---------------------------------------------------------------------------//

/// What the mobiles' records need beyond the world and the plan.
#[derive(Debug, Clone, Copy)]
pub struct MobileInputs<'a> {

    /// Record layouts.
    pub layout: &'a StartposLayout,

    /// Component ids.
    pub plan: &'a CaiPlan,

    /// The world's objects.
    pub world: &'a WorldView,

    /// Region of each cell.
    pub region_map: &'a RegionMap,

    /// Connectivity group of each cell.
    pub groups: &'a AreaGroups<'a>,

    /// Component of the settlement of each garrison residence.
    pub settlements: &'a HashMap<u32, u32>,

    /// Cell where the AI places the port of each garrison residence's settlement.
    pub ports: &'a HashMap<u32, HexCell>,
}

//---------------------------------------------------------------------------//
//                              Implementations
//---------------------------------------------------------------------------//

/// Writes the characters list, faction by faction.
///
/// # Errors
///
/// Returns an error if the layout lacks the records.
pub fn write_characters(inputs: &MobileInputs) -> Result<RecordNode> {
    let layout = inputs.layout;
    let path = child_path(CAI_WORLD_PATH, "CAI_WORLD_CHARACTERS");
    let mut list = layout.builder(&path)?;
    for (position, faction) in inputs.world.factions().iter().enumerate() {
        let faction_id = inputs.plan.factions()[position].faction();
        let posts = &inputs.plan.posts()[position];
        for (character, ids) in faction.characters().iter().zip(&inputs.plan.characters()[position]) {
            let force = force_of(faction.forces(), character);
            let mobile = ids.mobile().unwrap_or_default();
            let own_unit = force.and_then(|force| force.units().iter().position(|unit| *unit == character.unit())).map(|unit| ids.units()[unit]).unwrap_or_default();
            let post = faction.posts().iter().position(|(_, holder)| *holder == character.id()).map(|post| posts[post]).unwrap_or_default();
            let codes = if force.is_none() && ids.mobile().is_some() { AGENT_CODES } else { FORCE_AGENT_CODES };

            let mut owned = layout.builder(&child_path(&path, "OWNED_INDIRECT"))?;
            owned.group()?.set_u32("^+1", faction_id)?;
            let mut record = layout.builder(&child_path(&path, "CAI_CHARACTER"))?;
            record.group()?
                .set_u32("^+1", mobile)?
                .set_u32("^+2", own_unit)?
                .set_u32("^+3", mobile)?
                .set_u32("^+4", character.id())?
                .set_u32("^+6", if post == 0 { 0 } else { faction_id })?
                .set_u32("^+7", codes.0)?
                .set_u32("^+8", codes.1)?
                .set_u32("^+9", post)?;

            list.push_group()?
                .set_record("OWNED_INDIRECT", Some(owned.build()?))?
                .set_record("CAI_BDI_COMPONENT", Some(component(layout, &path, ids.character(), vec![])?))?
                .set_record("CAI_CHARACTER", Some(record.build()?))?;
        }
    }
    list.build()
}

/// Returns where each mobile stands, in creation order: its component and the cell of its `CAI_SITUATED`.
///
/// Mobiles stand on their commander's cell, except garrison fleets, which wait at their settlement's port.
pub fn mobile_cells(inputs: &MobileInputs) -> Vec<(u32, HexCell)> {
    let mut cells = vec![];
    for (position, faction) in inputs.world.factions().iter().enumerate() {
        for (character, ids) in faction.characters().iter().zip(&inputs.plan.characters()[position]) {
            let Some(mobile) = ids.mobile() else { continue };
            let port = force_of(faction.forces(), character).filter(|force| force.garrison_navy_of() != 0).and_then(|force| inputs.ports.get(&force.garrison_navy_of()));
            let (x, y) = character.cell();
            cells.push((mobile, port.copied().unwrap_or(HexCell::new(x, y))));
        }
    }
    cells
}

/// Writes the mobiles list: each force, and each agent moving on its own, in character order.
///
/// # Errors
///
/// Returns an error if the layout lacks the records.
pub fn write_mobiles(inputs: &MobileInputs) -> Result<RecordNode> {
    let layout = inputs.layout;
    let path = child_path(CAI_WORLD_PATH, "CAI_WORLD_RESOURCE_MOBILES");
    let mut cells = mobile_cells(inputs).into_iter();
    let mut list = layout.builder(&path)?;
    for (position, faction) in inputs.world.factions().iter().enumerate() {
        let faction_id = inputs.plan.factions()[position].faction();
        for (character, ids) in faction.characters().iter().zip(&inputs.plan.characters()[position]) {
            let Some(mobile) = ids.mobile() else { continue };
            let force = force_of(faction.forces(), character);
            let residence = force.map(|force| [force.garrison_army_of(), force.garrison_navy_of(), force.residence()].into_iter().find(|residence| *residence != 0).unwrap_or_default()).unwrap_or_default();

            // Agents have no position of their own.
            let cell = cells.next().map(|(_, cell)| cell).unwrap_or_default();
            let (own_x, own_y) = if force.is_none() { (0, 0) } else { (cell.x(), cell.y()) };

            let mut record = layout.builder(&child_path(&path, "CAI_RESOURCE_MOBILE"))?;
            record.group()?
                .set_u32("^+1", ids.character())?
                .set_u32("^+2", inputs.settlements.get(&residence).copied().unwrap_or_default())?
                .set_u32s("^+4", vec![ids.character()])?
                .set_u32s("^+5", ids.units().to_vec())?
                .set_bool("^+6", !force.is_some_and(ForceView::navy))?
                .set_u32("^+12", force.map(ForceView::id).unwrap_or_default())?
                .set_u32("^+17", u32::from(own_x))?
                .set_u32("^+18", u32::from(own_y))?
                .set_u32s("^+20", ids.horde_slots().to_vec())?;

            list.push_group()?
                .set_record("OWNED_DIRECT", Some(owned_direct(layout, &path, faction_id)?))?
                .set_record("CAI_SITUATED", Some(situated(inputs, &path, cell)?))?
                .set_record("CAI_BDI_COMPONENT", Some(component(layout, &path, mobile, vec![])?))?
                .set_record("CAI_RESOURCE_MOBILE", Some(record.build()?))?;
        }
    }
    list.build()
}

/// Writes the units list: each force's units, in character order.
///
/// # Errors
///
/// Returns an error if the layout lacks the records.
pub fn write_units(inputs: &MobileInputs) -> Result<RecordNode> {
    let layout = inputs.layout;
    let path = child_path(CAI_WORLD_PATH, "CAI_WORLD_UNITS");
    let mut list = layout.builder(&path)?;
    for (position, faction) in inputs.world.factions().iter().enumerate() {
        for (character, ids) in faction.characters().iter().zip(&inputs.plan.characters()[position]) {
            let Some(force) = force_of(faction.forces(), character) else { continue };
            for (unit, id) in force.units().iter().zip(ids.units()) {
                let mut record = layout.builder(&child_path(&path, "CAI_UNIT"))?;
                record.group()?
                    .set_u32("^+1", if *unit == character.unit() { ids.character() } else { 0 })?
                    .set_u32("^+2", *unit)?
                    .set_u32("^+3", ids.mobile().unwrap_or_default())?;

                list.push_group()?
                    .set_record("CAI_BDI_COMPONENT", Some(component(layout, &path, *id, vec![])?))?
                    .set_record("CAI_UNIT", Some(record.build()?))?;
            }
        }
    }
    list.build()
}

/// Writes where an AI object stands (`CAI_SITUATED`): its cell, the component of the map region the cell is in, the
/// theatre, and the cell's connectivity group.
pub(crate) fn situated(inputs: &MobileInputs, parent: &str, cell: HexCell) -> Result<RecordNode> {
    let region = inputs.region_map.region_index_at(cell).and_then(|region| inputs.plan.regions().get(region)).map(|region| region.id()).unwrap_or_default();
    let mut record = inputs.layout.builder(&child_path(parent, "CAI_SITUATED"))?;
    record.group()?
        .set_u32("^+1", u32::from(cell.x()))?
        .set_u32("^+2", u32::from(cell.y()))?
        .set_u32("^+3", region)?
        .set_u32s("^+4", vec![inputs.plan.theatre()])?
        .set_u32("^+5", inputs.groups.group_at(cell).unwrap_or_default())?;
    record.build()
}

/// Returns the force a character commands.
fn force_of<'a>(forces: &'a [ForceView], character: &CharacterView) -> Option<&'a ForceView> {
    forces.iter().find(|force| character.force() != 0 && force.id() == character.force())
}
