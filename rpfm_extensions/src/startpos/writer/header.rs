//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Writer of the records that name and date the campaign: the save game header (`SAVE_GAME_HEADER`), the calendar
//! (`CAMPAIGN_CALENDAR`), the local campaign setup (`CAMPAIGN_SETUP_LOCAL`) and the dates other records carry.
//!
//! The save game header describes the campaign's first faction, as the load screen shows it. Its map thumbnail is
//! kept from the layout.

use std::collections::HashMap;

use rpfm_lib::error::Result;
use rpfm_lib::files::esf::{NodeType, RecordNode};
use rpfm_lib::schema::MERGE_COLOUR_NO_NAME;

use super::preopen::army_positions;
use super::{StartposInputs, ENV_PATH, MODEL_PATH, ROOT_PATH};
use crate::startpos::layout::{child_path, StartposLayout};
use crate::startpos::world::calendar::Calendar;
use crate::startpos::world::government::faction_leader;
use crate::startpos::world::writer::forces::ForcePlan;
use crate::startpos::world::writer::WorldPlan;
use crate::startpos::world::FactionEntry;


/// Folder of the portraits the load screen shows, by culture.
const PORTRAITS_FOLDER: &str = "UI/Portraits/Portholes";

/// Suffix of the portrait files after the porthole name.
const PORTRAIT_SUFFIX: &str = "_0.png";

/// Colour group of the `regions` colour (its `r`, `g` and `b` columns), which RPFM decodes as one unnamed colour.
const REGION_COLOUR_GROUP: u8 = 1;

//---------------------------------------------------------------------------//
//                              Implementations
//---------------------------------------------------------------------------//

/// Writes the save game header: the first faction (portrait, flag, regions or army positions), the start date and
/// season, and the map.
///
/// # Arguments
///
/// * `inputs` - Tables, startpos rows, layouts and map data.
/// * `plan` - Order and runtime ids of the world's objects.
/// * `forces` - Characters and forces of the world.
/// * `calendar` - Calendar of the campaign.
///
/// # Errors
///
/// Returns an error if a table, column or layout is missing.
pub fn write_save_game_header(inputs: &StartposInputs, plan: &WorldPlan, forces: &ForcePlan, calendar: &Calendar) -> Result<RecordNode> {
    let world = inputs.world;
    let layout = world.layout;
    let path = child_path(ROOT_PATH, "SAVE_GAME_HEADER");
    let mut record = layout.builder(&path)?;
    let Some(faction) = plan.factions().first() else { return record.build() };

    let regions = plan.regions().iter()
        .filter(|region| region.owner().as_deref() == Some(faction.key().as_str()))
        .map(|region| region.key().to_owned())
        .collect::<Vec<_>>();
    let positions = if regions.is_empty() { army_positions(world, forces, faction) } else { vec![] };
    let (regions, palette_indices) = palette_indices(inputs, regions)?;

    let mut maps = layout.builder(&child_path(&path, "MAPS"))?;
    maps.push_group()?.set_str("^+1", inputs.map)?;

    record.group()?
        .set_str("^+1", faction.key())?
        .set_str("^+2", &portrait(inputs, faction)?)?
        .set_u32("^+4", calendar.start_year())?
        .set_str("^+5", inputs.season_name)?
        .set_str("^+6", &flags_path(inputs, faction)?)?
        .set_record("DATE", Some(write_date(layout, &path, calendar)?))?
        .set_record("MAPS", Some(maps.build()?))?
        .set_strings("MAPS+1", regions)?
        .set_u32s("MAPS+2", palette_indices)?
        .set("MAPS+3", NodeType::Coord2dArray(positions))?;
    record.build()
}

/// Writes the calendar: turns per year and start date.
///
/// # Errors
///
/// Returns an error if the layout lacks the records.
pub fn write_calendar(layout: &StartposLayout, calendar: &Calendar) -> Result<RecordNode> {
    let path = child_path(MODEL_PATH, "CAMPAIGN_CALENDAR");
    let mut record = layout.builder(&path)?;
    record.group()?
        .set_u32("^+1", calendar.turns_per_year())?
        .set_record("DATE", Some(write_date(layout, &path, calendar)?))?;
    record.build()
}

/// Writes the local campaign setup: the faction the local player starts as, the campaign's first one.
///
/// # Errors
///
/// Returns an error if the layout lacks the record.
pub fn write_setup_local(layout: &StartposLayout, plan: &WorldPlan) -> Result<RecordNode> {
    let mut record = layout.builder(&child_path(ENV_PATH, "CAMPAIGN_SETUP_LOCAL"))?;
    if let Some(faction) = plan.factions().first() {
        record.group()?.set_str("^+1", faction.key())?;
    }
    record.build()
}

/// Writes the start date (`DATE`) under a parent record.
///
/// # Errors
///
/// Returns an error if the layout lacks the record.
pub fn write_date(layout: &StartposLayout, parent: &str, calendar: &Calendar) -> Result<RecordNode> {
    let mut record = layout.builder(&child_path(parent, "DATE"))?;
    record.group()?.set_u32("^+1", calendar.start_year())?;
    record.build()
}

/// Returns the regions with a colour in the map's region lookup image, and the palette index of each one's colour.
fn palette_indices(inputs: &StartposInputs, regions: Vec<String>) -> Result<(Vec<String>, Vec<u32>)> {
    let mut colours = HashMap::new();
    let colour_column = format!("{}_{REGION_COLOUR_GROUP}", MERGE_COLOUR_NO_NAME.to_lowercase());
    for row in inputs.world.tables.table("regions")?.rows() {
        colours.insert(row.str("key")?.into_owned(), row.colour(&colour_column)?);
    }

    Ok(regions.into_iter()
        .filter_map(|region| {
            let colour = colours.get(&region)?;
            let index = inputs.region_palette.iter().position(|entry| entry == colour)?;
            Some((region, u32::try_from(index).ok()?))
        })
        .unzip())
}

/// Returns the portrait of a faction's leader: the porthole of the uniform of its `portrait_id`, in its culture's
/// folder. Empty if the leader has no portrait.
fn portrait(inputs: &StartposInputs, faction: &FactionEntry) -> Result<String> {
    let tables = inputs.world.tables;
    let Some(leader) = faction_leader(inputs.world.index, faction.key()) else { return Ok(String::new()) };

    let mut uniform = None;
    for row in tables.table("start_pos_characters")?.rows() {
        if row.str("ID")? == *leader.id() {
            uniform = Some(row.str("portrait_id")?.into_owned());
            break;
        }
    }
    let Some(uniform) = uniform.filter(|uniform| !uniform.is_empty()) else { return Ok(String::new()) };

    for row in tables.table("agent_uniforms")?.rows() {
        if row.str("uniform_name")? == uniform {
            return Ok(format!("{PORTRAITS_FOLDER}/{}/{}{PORTRAIT_SUFFIX}", faction.culture(), row.str("campaign_porthole_filename")?));
        }
    }
    Ok(String::new())
}

/// Returns the flag folder of a faction.
fn flags_path(inputs: &StartposInputs, faction: &FactionEntry) -> Result<String> {
    for row in inputs.world.tables.table("factions")?.rows() {
        if row.str("key")? == *faction.key() {
            return Ok(row.str("flags_path")?.into_owned());
        }
    }
    Ok(String::new())
}
