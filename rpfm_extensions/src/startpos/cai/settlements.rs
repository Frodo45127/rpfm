//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Writer of the AI's settlements (`CAI_WORLD_SETTLEMENTS`) and building slots (`CAI_WORLD_BUILDING_SLOTS`: the
//! settlements' slots, then the horde armies' slots).

use std::collections::HashMap;

use rpfm_lib::error::Result;
use rpfm_lib::files::esf::RecordNode;

use super::factions::{owned_direct, REBELS_WORLD_ID};
use super::map::{AreaGroups, MapRegion};
use super::mobiles::{situated, MobileInputs};
use super::plan::CaiPlan;
use super::world_view::{ForceView, WorldView};
use super::{component, CAI_WORLD_PATH};
use crate::startpos::coords::HexCell;
use crate::startpos::layout::{child_path, StartposLayout};
use crate::startpos::settlements::SettlementLayout;

#[cfg(test)] #[path = "settlements_test.rs"] mod settlements_test;

/// Slot type (`CAI_BUILDING_SLOT^+3`) of ports.
const PORT_SLOT_TYPE: u32 = 3;

/// Slot type of horde army slots.
const HORDE_SLOT_TYPE: u32 = 9;

/// Position of the port slot in a settlement's slots (the main slot comes first).
const PORT_SLOT: usize = 1;

//---------------------------------------------------------------------------//
//                              Implementations
//---------------------------------------------------------------------------//

/// Returns the cell where the AI places the port of each garrison residence's settlement.
///
/// # Arguments
///
/// * `world` - The world's objects.
/// * `map_regions` - Regions of the map, in map order.
/// * `groups` - Cell lookups of the map.
pub fn port_cells(world: &WorldView, map_regions: &[MapRegion], groups: &AreaGroups) -> HashMap<u32, HexCell> {
    world.regions().iter()
        .filter_map(|region| {
            let port = map_regions.iter().find(|map_region| map_region.key() == region.key())?.port()?;
            Some((region.residence(), groups.port_cell(port)?))
        })
        .collect()
}

/// Returns the component of the settlement of each garrison residence.
///
/// # Arguments
///
/// * `world` - The world's objects.
/// * `plan` - Component ids.
/// * `map_regions` - Regions of the map, in map order.
pub fn settlement_components(world: &WorldView, plan: &CaiPlan, map_regions: &[MapRegion]) -> HashMap<u32, u32> {
    map_regions.iter().enumerate()
        .filter_map(|(index, map_region)| {
            let region = world.regions().iter().find(|region| region.key() == map_region.key())?;
            Some((region.residence(), plan.settlements().get(&index)?.settlement()))
        })
        .collect()
}

/// Writes the settlements list, in map order.
///
/// # Arguments
///
/// * `inputs` - The world, plan and map lookups.
/// * `map_regions` - Regions of the map, in map order.
/// * `layouts` - Settlement layouts of the map.
/// * `positions` - Where the AI places each map region (see [`super::regions::region_positions`]).
/// * `capitals` - Component of the faction whose capital each map region is (see [`super::factions::capitals`]).
///
/// # Errors
///
/// Returns an error if the layout lacks the records.
pub fn write_settlements(inputs: &MobileInputs, map_regions: &[MapRegion], positions: &[(u16, u16)], capitals: &HashMap<usize, u32>) -> Result<RecordNode> {
    let layout = inputs.layout;
    let plan = inputs.plan;
    let path = child_path(CAI_WORLD_PATH, "CAI_WORLD_SETTLEMENTS");
    let garrisonable_path = child_path(&path, "CAI_GARRISONABLE");
    let threatened_path = child_path(&path, "CAI_THREATENED_GARRISONABLE");

    // The mobile of each force, by world force id, and the faction component of each world faction id.
    let mut mobiles = HashMap::new();
    for (position, faction) in inputs.world.factions().iter().enumerate() {
        for (character, ids) in faction.characters().iter().zip(&plan.characters()[position]) {
            if let Some(mobile) = ids.mobile().filter(|_| character.force() != 0) {
                mobiles.insert(character.force(), mobile);
            }
        }
    }
    let mut faction_components = inputs.world.factions().iter().zip(plan.factions()).map(|(faction, ids)| (faction.id(), ids.faction())).collect::<HashMap<_, _>>();
    if let Some(rebels) = plan.factions().last() {
        faction_components.insert(REBELS_WORLD_ID, rebels.faction());
    }

    let forces = inputs.world.factions().iter().flat_map(|faction| faction.forces()).collect::<Vec<_>>();
    let mut list = layout.builder(&path)?;
    for (index, map_region) in map_regions.iter().enumerate() {
        let (Some(ids), Some(region)) = (plan.settlements().get(&index), inputs.world.regions().iter().find(|region| region.key() == map_region.key())) else { continue };
        let mobile_of = |matches: fn(&ForceView, u32) -> bool| forces.iter()
            .find(|force| matches(force, region.residence()))
            .and_then(|force| mobiles.get(&force.id()))
            .copied()
            .unwrap_or_default();

        let mut garrisonable = layout.builder(&garrisonable_path)?;
        garrisonable.group()?
            .set_u32("^+1", mobile_of(|force, residence| force.residence() == residence && force.garrison_army_of() == 0 && force.garrison_navy_of() == 0))?
            .set_u32("^+3", mobile_of(|force, residence| force.garrison_army_of() == residence))?
            .set_u32("^+4", mobile_of(|force, residence| force.garrison_navy_of() == residence))?
            .set_u32s("^+10", vec![])?;

        let port = inputs.ports.get(&region.residence()).copied();
        let mut threatened = layout.builder(&threatened_path)?;
        threatened.group()?
            .set_bool("^+1", port.is_some())?
            .set_record("CAI_SITUATED", port.map(|cell| situated(inputs, &threatened_path, cell)).transpose()?)?;

        let mut settlement = layout.builder(&child_path(&path, "CAI_SETTLEMENT"))?;
        settlement.group()?
            .set_u32s("^+1", ids.slots().to_vec())?
            .set_u32("^+2", capitals.get(&index).copied().unwrap_or_default())?
            .set_u32("^+3", region.id())?;

        let (x, y) = positions.get(index).copied().unwrap_or_default();
        list.push_group()?
            .set_record("OWNED_DIRECT", Some(owned_direct(layout, &path, faction_components.get(&region.owner()).copied().unwrap_or_default())?))?
            .set_record("CAI_SITUATED", Some(situated(inputs, &path, HexCell::new(x, y))?))?
            .set_record("CAI_BDI_COMPONENT", Some(component(layout, &path, ids.settlement(), vec![])?))?
            .set_record("CAI_GARRISONABLE", Some(garrisonable.build()?))?
            .set_record("CAI_THREATENED_GARRISONABLE", Some(threatened.build()?))?
            .set_record("CAI_SETTLEMENT", Some(settlement.build()?))?;
    }
    list.build()
}

/// Writes the building slots list: each settlement's slots in map order, then each horde army's slots.
///
/// # Arguments
///
/// * `layout` - Record layouts.
/// * `plan` - Component ids.
/// * `world` - The world's objects.
/// * `map_regions` - Regions of the map, in map order.
/// * `layouts` - Settlement layouts of the map.
///
/// # Errors
///
/// Returns an error if the layout lacks the records.
pub fn write_building_slots(layout: &StartposLayout, plan: &CaiPlan, world: &WorldView, map_regions: &[MapRegion], layouts: &[SettlementLayout]) -> Result<RecordNode> {
    let path = child_path(CAI_WORLD_PATH, "CAI_WORLD_BUILDING_SLOTS");
    let slot_path = child_path(&path, "CAI_BUILDING_SLOT");
    let mut list = layout.builder(&path)?;
    // Settlement slots hold their world slot id first, horde slots in the second field.
    let mut push = |id: u32, settlement_slot: u32, horde_slot: u32, kind: u32, settlement: u32, horde: u32| -> Result<()> {
        let mut record = layout.builder(&slot_path)?;
        record.group()?
            .set_u32("^+1", settlement_slot)?
            .set_u32("^+2", horde_slot)?
            .set_u32("^+3", kind)?
            .set_u32("^+4", settlement)?
            .set_u32("^+5", horde)?;
        list.push_group()?
            .set_record("CAI_BDI_COMPONENT", Some(component(layout, &path, id, vec![])?))?
            .set_record("CAI_BUILDING_SLOT", Some(record.build()?))?;
        Ok(())
    };

    for (index, map_region) in map_regions.iter().enumerate() {
        let (Some(ids), Some(region)) = (plan.settlements().get(&index), world.regions().iter().find(|region| region.key() == map_region.key())) else { continue };
        let has_port = layouts.iter().find(|settlement| settlement.region() == map_region.key()).and_then(|settlement| settlement.slots().get(PORT_SLOT)).is_some_and(|slot| slot.position().is_some());
        for (slot, (id, world_id)) in ids.slots().iter().zip(region.slots()).enumerate() {
            let kind = if has_port && slot == PORT_SLOT { PORT_SLOT_TYPE } else { 0 };
            push(*id, *world_id, 0, kind, ids.settlement(), 0)?;
        }
    }

    for (position, faction) in world.factions().iter().enumerate() {
        for (character, ids) in faction.characters().iter().zip(&plan.characters()[position]) {
            let Some(force) = faction.forces().iter().find(|force| character.force() != 0 && force.id() == character.force()) else { continue };
            for (id, world_id) in ids.horde_slots().iter().zip(force.horde_slots()) {
                push(*id, 0, *world_id, HORDE_SLOT_TYPE, 0, ids.mobile().unwrap_or_default())?;
            }
        }
    }
    list.build()
}
