//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Writer of the AI state (`CAI_INTERFACE`): the world mirror, the central pool of analysers, the faction managers
//! and their AI levels, the director's pool, the high-level pathfinder and the influence maps.
//!
//! The pathfinder is copied from the map's `hlp_data.esf`. The influence maps and every belief start empty: the game
//! fills them in while playing. The region distances keep the layout's values, which are the campaign's own.

use rpfm_lib::error::{RLibError, Result};
use rpfm_lib::files::esf::{NodeType, RecordNode};

use super::managers::{component_map_order, pool_component, write_managers, ComponentFields, ManagerInputs, FACTION_LEVEL, REBEL_LEVEL};
use super::mobiles::mobile_cells;
use super::stances::StartingStances;
use super::world::{CaiContext, CaiInputs};
use super::CAI_PATH;
use crate::startpos::esf_nav::{as_record, child_record};
use crate::startpos::layout::{child_path, variant_path, StartposLayout};
use crate::startpos::writer::{bool_node, i32_node, u32_node};

#[cfg(test)] #[path = "interface_test.rs"] mod interface_test;

/// Type codes of the central pool's desires, in creation order.
pub const CENTRAL_DESIRES: [u32; 29] = [32, 33, 36, 35, 42, 31, 43, 30, 45, 47, 92, 39, 128, 130, 132, 136, 134, 46, 153, 155, 157, 162, 164, 166, 168, 178, 181, 187, 181];

/// Type codes of the director pool's beliefs, in creation order: its time and its world.
pub const DIRECTOR_BELIEFS: [u32; 2] = [3, 2];

/// Type codes of the director pool's desires, in creation order: the player rival score and unit balance analysers,
/// the top level desire and the event generator.
pub const DIRECTOR_DESIRES: [u32; 4] = [5, 21, 18, 34];

/// Values of the ownership entry linking the director's top level desire to its event generator.
const TOP_LEVEL_LINK: (f32, f32, u32) = (1000.0, 0.0, 0);

/// Size and tile size of the influence maps, which start without tiles.
const SPARSE_MAP_SIZE: (u32, u32) = (1024, 8);

/// Cell the influence maps of mobiles hold before their first update.
const NO_CELL: u16 = u16::MAX;

//---------------------------------------------------------------------------//
//                              Implementations
//---------------------------------------------------------------------------//

impl CaiContext {

    /// Writes the AI state (`CAI_INTERFACE`).
    ///
    /// # Arguments
    ///
    /// * `inputs` - What the AI writer is built from.
    ///
    /// # Errors
    ///
    /// Returns an error if a table is missing, the layout lacks a record or the pathfinder file has no pathfinder.
    pub fn write_interface(&self, inputs: &CaiInputs) -> Result<RecordNode> {
        let layout = inputs.layout;
        let plan = self.plan();
        let keys = self.world().factions().iter().map(|faction| faction.key().as_str()).collect::<Vec<_>>();
        let stances = StartingStances::new(inputs.tables, inputs.index, &keys)?;
        let characters = plan.characters().iter().map(|characters| characters.iter().map(|character| character.character()).collect()).collect::<Vec<_>>();
        let mobiles = plan.characters().iter().map(|characters| characters.iter().filter_map(|character| character.mobile()).collect()).collect::<Vec<_>>();
        let managers = write_managers(&ManagerInputs { layout, plan, stances: &stances, characters: &characters, mobiles: &mobiles })?;

        let (world, influence) = self.with_mobiles(inputs, |mobile_inputs| {
            let world = self.write_world_with(inputs, mobile_inputs)?;
            let cells = mobile_cells(mobile_inputs).into_iter().map(|(mobile, cell)| (mobile, (cell.x(), cell.y()))).collect::<Vec<_>>();
            Ok((world, self.write_influence_maps(layout, &cells)?))
        })?;

        let hlp_root = as_record(inputs.hlp_data.root_node()).ok_or_else(|| RLibError::StartposLayoutError("the pathfinder file has no root record".to_owned()))?;
        let pathfinder = child_record(hlp_root, "CAI_HIGH_LEVEL_PATHFINDER").ok_or_else(|| RLibError::StartposLayoutError("the pathfinder file has no pathfinder".to_owned()))?;

        let factions = plan.factions();
        let levels = factions.iter().enumerate()
            .flat_map(|(position, ids)| [ids.faction(), if position + 1 == factions.len() { REBEL_LEVEL } else { FACTION_LEVEL }])
            .collect();

        // The central pool's slots hold the central desires' ids, which match the game's: they're created first.
        let mut central = layout.builder(&child_path(CAI_PATH, "CAI_CENTRAL_BDI_POOL"))?;
        central.group()?.set_u32("^+1", plan.next_id())?;

        let mut record = layout.builder(CAI_PATH)?;
        record.group()?
            .set_record("CAI_WORLD", Some(world))?
            .set_record("CAI_BDI_POOL", Some(self.write_central_pool(layout)?))?
            .set_record("CAI_CENTRAL_BDI_POOL", Some(central.build()?))?
            .set_record("CAI_INTERFACE_MANAGERS", Some(managers))?
            .set_u32s("CAI_INTERFACE_MANAGERS+1", levels)?
            .set_record("CAI_INTERFACE_DIRECTOR_BDI_POOL", Some(self.write_director_pool(layout)?))?
            .set_record("CAI_HIGH_LEVEL_PATHFINDER", Some(pathfinder.clone()))?
            .set_record("CAI_INFLUENCE_MAP_MANAGER", Some(influence))?;
        record.build()
    }

    /// Writes the central pool: its analysers and log manager, with their type's defaults.
    fn write_central_pool(&self, layout: &StartposLayout) -> Result<RecordNode> {
        let path = child_path(CAI_PATH, "CAI_BDI_POOL");
        let desires_path = child_path(&path, "CAI_BDI_POOL_DESIRES");
        let ids = (0..CENTRAL_DESIRES.len() as u32).map(|index| self.plan().first_central_desire() + index).collect::<Vec<_>>();
        let mut desires = layout.builder(&desires_path)?;
        for index in component_map_order(&ids) {
            let code = CENTRAL_DESIRES[index];
            let component = pool_component(layout, &variant_path(&desires_path, code), ids[index], ComponentFields::default())?;
            desires.push_variant(code)?.set_record("CAI_BDI_COMPONENT", Some(component))?;
        }

        let mut pool = layout.builder(&path)?;
        pool.group()?.set_record("CAI_BDI_POOL_DESIRES", Some(desires.build()?))?;
        pool.build()
    }

    /// Writes the director's pool: its time and world beliefs, its two analysers, its top level desire and the event
    /// generator the top level desire owns.
    fn write_director_pool(&self, layout: &StartposLayout) -> Result<RecordNode> {
        let path = child_path(CAI_PATH, "CAI_INTERFACE_DIRECTOR_BDI_POOL");
        let pool_path = child_path(&path, "CAI_BDI_POOL");
        let ids = self.plan().director();
        let (belief_ids, desire_ids) = ids.split_at(DIRECTOR_BELIEFS.len());
        let (time, generator) = (belief_ids[0], desire_ids[3]);

        let beliefs_path = child_path(&pool_path, "CAI_BDI_POOL_BELIEFS");
        let mut beliefs = layout.builder(&beliefs_path)?;
        for index in component_map_order(belief_ids) {
            let (code, id) = (DIRECTOR_BELIEFS[index], belief_ids[index]);
            let variant = variant_path(&beliefs_path, code);
            let references = if id == time { vec![generator] } else { vec![] };
            let group = beliefs.push_variant(code)?;
            group.set_record("CAI_BDI_COMPONENT", Some(pool_component(layout, &variant, id, ComponentFields { references, ..Default::default() })?))?;
            if id != time {
                let mut world = layout.builder(&child_path(&variant, "CDIR_WORLD"))?;
                world.group()?.set_u32("^+1", time)?;
                group.set_record("CDIR_WORLD", Some(world.build()?))?;
            }
        }

        let desires_path = child_path(&pool_path, "CAI_BDI_POOL_DESIRES");
        let mut desires = layout.builder(&desires_path)?;
        for index in component_map_order(desire_ids) {
            let (code, id) = (DIRECTOR_DESIRES[index], desire_ids[index]);
            let links = if index == 2 { vec![(generator, TOP_LEVEL_LINK)] } else { vec![] };
            let component = pool_component(layout, &variant_path(&desires_path, code), id, ComponentFields { links, ..Default::default() })?;
            desires.push_variant(code)?.set_record("CAI_BDI_COMPONENT", Some(component))?;
        }

        let mut pool = layout.builder(&pool_path)?;
        pool.group()?
            .set_record("CAI_BDI_POOL_BELIEFS", Some(beliefs.build()?))?
            .set_record("CAI_BDI_POOL_DESIRES", Some(desires.build()?))?;

        let mut analysers = layout.builder(&child_path(&path, "CDIR_BDI_POOL"))?;
        analysers.group()?
            .set_u32("^+1", desire_ids[0])?
            .set_u32("^+2", desire_ids[1])?;

        let mut director = layout.builder(&path)?;
        director.group()?
            .set_record("CAI_BDI_POOL", Some(pool.build()?))?
            .set_record("CDIR_BDI_POOL", Some(analysers.build()?))?;
        director.build()
    }

    /// Writes the influence maps: two per mobile (where it is and where it can reach), in their constructor state,
    /// and one per faction. Every map starts without tiles.
    ///
    /// # Arguments
    ///
    /// * `layout` - Record layouts.
    /// * `cells` - Each mobile's component and cell, in creation order.
    fn write_influence_maps(&self, layout: &StartposLayout, cells: &[(u32, (u16, u16))]) -> Result<RecordNode> {
        let path = child_path(CAI_PATH, "CAI_INFLUENCE_MAP_MANAGER");
        let map_path = child_path(&path, "CAI_SPARSE_MAP");
        let empty_map = || -> Result<NodeType> {
            let record = layout.raw_record(&map_path, vec![u32_node(SPARSE_MAP_SIZE.0), u32_node(SPARSE_MAP_SIZE.1), u32_node(0)])?;
            Ok(NodeType::Record(Box::new(record)))
        };

        let mut children = vec![u32_node(cells.len() as u32)];
        let mobiles = cells.iter().map(|(mobile, _)| *mobile).collect::<Vec<_>>();
        for index in component_map_order(&mobiles) {
            let (mobile, (x, y)) = cells[index];
            children.extend([u32_node(mobile), empty_map()?, bool_node(false), NodeType::U16(NO_CELL), NodeType::U16(NO_CELL), u32_node(0), u32_node(0), u32_node(0)]);
            children.extend([empty_map()?, i32_node(0), bool_node(false), NodeType::U16(x), NodeType::U16(y), u32_node(0), u32_node(0), u32_node(0)]);
        }

        let factions = self.plan().factions();
        children.push(u32_node(factions.len() as u32));
        for ids in factions {
            children.extend([u32_node(ids.faction()), empty_map()?]);
        }
        layout.raw_record(&path, children)
    }
}
