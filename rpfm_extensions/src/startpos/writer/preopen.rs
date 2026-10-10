//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Writer of what the faction selection screen reads before loading the campaign (`CAMPAIGN_PREOPEN_MAP_INFO`), and
//! of the players setup it shares with the campaign setup (`CAMPAIGN_PLAYERS_SETUP`).

use std::collections::HashMap;

use rpfm_lib::error::Result;
use rpfm_lib::files::esf::{Coordinates2DNode, NodeType, RecordNode};

use super::victory::write_victory_mission_options;
use super::ENV_PATH;
use crate::startpos::coords::HexCell;
use crate::startpos::layout::{child_path, StartposLayout};
use crate::startpos::missions::VictoryObjectives;
use crate::startpos::world::FactionEntry;
use crate::startpos::world::government::faction_leader;
use crate::startpos::world::writer::characters::write_character_name;
use crate::startpos::world::writer::forces::ForcePlan;
use crate::startpos::world::writer::{start_pos_faction_row, WorldInputs, WorldPlan};

/// Path of the pre-open map info.
const PREOPEN_PATH: &str = "CAMPAIGN_STARTPOS/CAMPAIGN_PREOPEN_MAP_INFO";

/// Path of the faction infos of the pre-open map info.
const FACTION_INFOS_PATH: &str = "CAMPAIGN_STARTPOS/CAMPAIGN_PREOPEN_MAP_INFO/FACTION_INFOS";

/// Owner of the regions without a faction.
const REBELS_KEY: &str = "rebels";

/// Prefix of the localisation key of a faction's description.
const DESCRIPTION_KEY_PREFIX: &str = "start_pos_factions_description_";

/// Prefix of the localisation key of a difficulty.
const DIFFICULTY_KEY_PREFIX: &str = "random_localisation_strings_string_";

//---------------------------------------------------------------------------//
//                              Implementations
//---------------------------------------------------------------------------//

/// Writes the pre-open map info of a campaign: its players, victory missions, factions and region owners.
///
/// # Arguments
///
/// * `inputs` - Tables, startpos rows, layouts and map data.
/// * `plan` - Order and runtime ids of the world's objects.
/// * `forces` - Characters and forces of the world.
/// * `victory_objectives` - Victory objectives of the campaign.
///
/// # Errors
///
/// Returns an error if a table, column or layout is missing.
pub fn write_preopen_map_info(inputs: &WorldInputs, plan: &WorldPlan, forces: &ForcePlan, victory_objectives: &[VictoryObjectives]) -> Result<RecordNode> {
    let layout = inputs.layout;
    let mut record = layout.builder(PREOPEN_PATH)?;
    record.group()?
        .set_record("CAMPAIGN_PLAYERS_SETUP", Some(write_players_setup(layout, PREOPEN_PATH, plan)?))?
        .set_record("VICTORY_MISSION_OPTIONS", Some(write_victory_mission_options(layout, victory_objectives)?))?
        .set_record("FACTION_INFOS", Some(write_faction_infos(inputs, plan, forces)?))?
        .set_record("REGION_OWNERSHIPS_BY_THEATRE", Some(write_region_ownerships(layout, plan)?))?;
    record.build()
}

/// Writes the campaign setup (`CAMPAIGN_SETUP`): its players.
///
/// # Errors
///
/// Returns an error if the layout lacks the records.
pub fn write_campaign_setup(layout: &StartposLayout, plan: &WorldPlan) -> Result<RecordNode> {
    let path = child_path(ENV_PATH, "CAMPAIGN_SETUP");
    let mut record = layout.builder(&path)?;
    record.group()?.set_record("CAMPAIGN_PLAYERS_SETUP", Some(write_players_setup(layout, &path, plan)?))?;
    record.build()
}

/// Writes the players setup (`CAMPAIGN_PLAYERS_SETUP`) under a parent record: one entry per faction, saying if
/// players can pick it.
fn write_players_setup(layout: &StartposLayout, parent: &str, plan: &WorldPlan) -> Result<RecordNode> {
    let path = child_path(parent, "CAMPAIGN_PLAYERS_SETUP");
    let array_path = child_path(&path, "PLAYERS_ARRAY");
    let setup_path = child_path(&array_path, "CAMPAIGN_PLAYER_SETUP");

    let mut players = layout.builder(&array_path)?;
    for faction in plan.factions() {
        let mut setup = layout.builder(&setup_path)?;
        setup.group()?
            .set_str("CAMPAIGN_PLAYER_SETUP_INGAME_MODIFIABLES+1", faction.key())?
            .set_bool("CAMPAIGN_PLAYER_SETUP_INGAME_MODIFIABLES+3", faction.playable())?;
        players.push_group()?.set_record("CAMPAIGN_PLAYER_SETUP", Some(setup.build()?))?;
    }

    let mut record = layout.builder(&path)?;
    record.group()?.set_record("PLAYERS_ARRAY", Some(players.build()?))?;
    record.build()
}

/// Writes what the faction selection screen shows of each faction (`FACTION_INFOS`): its table values, leader, region
/// count and, for factions without regions, where its armies are (as a fraction of the map size).
fn write_faction_infos(inputs: &WorldInputs, plan: &WorldPlan, forces: &ForcePlan) -> Result<RecordNode> {
    let tables = inputs.tables;
    let mut flags = HashMap::new();
    for row in tables.table("factions")?.rows() {
        flags.insert(row.str("key")?.into_owned(), row.str("flags_path")?.into_owned());
    }
    let mut characters = HashMap::new();
    for row in tables.table("start_pos_characters")?.rows() {
        characters.insert(row.str("ID")?.into_owned(), row);
    }

    let mut infos = inputs.layout.builder(FACTION_INFOS_PATH)?;
    for faction in plan.factions() {
        let row = start_pos_faction_row(tables, faction)?;
        let leader = faction_leader(inputs.index, faction.key()).and_then(|leader| characters.get(leader.id()));
        let region_count = plan.regions().iter().filter(|region| region.owner().as_deref() == Some(faction.key().as_str())).count();
        let army_positions = if region_count == 0 { army_positions(inputs, forces, faction) } else { vec![] };

        infos.push_group()?
            .set_str("^+1", faction.key())?
            .set_bool("^+2", row.bool("is_major")?)?
            .set_bool("^+3", faction.playable())?
            .set_i32("^+4", i32::try_from(faction.starting_order()).unwrap_or_default())?
            .set_str("^+5", &format!("{DESCRIPTION_KEY_PREFIX}{}", faction.id()))?
            .set_str("^+6", flags.get(faction.key()).map(String::as_str).unwrap_or_default())?
            .set_i32("^+7", i32::try_from(region_count).unwrap_or_default())?
            .set_str("^+8", &format!("{DIFFICULTY_KEY_PREFIX}{}", row.str("difficulty")?))?
            .set_str("^+9", &row.str("state_religion")?)?
            .set_record("CHARACTER_NAME", Some(write_character_name(inputs.layout, FACTION_INFOS_PATH, leader)?))?
            .set("CHARACTER_NAME+2", NodeType::Coord2dArray(army_positions))?;
    }
    infos.build()
}

/// Returns where the armies of a faction start, as a fraction of the map size, without repeating positions.
fn army_positions(inputs: &WorldInputs, forces: &ForcePlan, faction: &FactionEntry) -> Vec<Coordinates2DNode> {
    let (width, height) = inputs.region_map.world_size();
    let mut positions = vec![];
    for force in forces.forces_of(faction.key()) {
        let Some(entry) = forces.characters()[force.commander()].entry() else { continue };
        if entry.start_x() == 0 && entry.start_y() == 0 {
            continue;
        }

        let (x, y) = HexCell::new(entry.start_x(), entry.start_y()).world_position();
        let mut position = Coordinates2DNode::default();
        position.set_x(x / width);
        position.set_y(y / height);
        if !positions.contains(&position) {
            positions.push(position);
        }
    }
    positions
}

/// Writes the owner of each region (`REGION_OWNERSHIPS_BY_THEATRE`), the rebels for regions without one.
fn write_region_ownerships(layout: &StartposLayout, plan: &WorldPlan) -> Result<RecordNode> {
    let path = child_path(PREOPEN_PATH, "REGION_OWNERSHIPS_BY_THEATRE");
    let mut owners = layout.builder(&child_path(&path, "REGION_OWNERSHIPS"))?;
    for region in plan.regions() {
        owners.push_group()?
            .set_str("^+1", region.key())?
            .set_str("^+2", region.owner().as_deref().unwrap_or(REBELS_KEY))?;
    }

    let mut theatres = layout.builder(&path)?;
    theatres.push_group()?.set_record("REGION_OWNERSHIPS", Some(owners.build()?))?;
    theatres.build()
}
