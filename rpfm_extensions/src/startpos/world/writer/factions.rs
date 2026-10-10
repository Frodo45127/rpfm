//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Writer of the `FACTION` records (`WORLD/FACTION_ARRAY`).

use rpfm_lib::error::Result;
use rpfm_lib::files::esf::RecordNode;

use super::{child, start_pos_faction_row, WorldInputs, WorldPlan, WORLD_PATH};
use crate::startpos::world::FactionEntry;
use crate::startpos::world::factions::{faction_flags, faction_summary, FactionFlag};

/// Path of a faction.
const FACTION_PATH: &str = "CAMPAIGN_STARTPOS/CAMPAIGN_ENV/CAMPAIGN_MODEL/WORLD/FACTION_ARRAY/FACTION";

/// Localisation key prefix of the faction names.
const SCREEN_NAME_PREFIX: &str = "factions_screen_name_";

/// Lowest `start_pos_factions.honour` of the factions that start with an empty diplomatic memory (`+21` field).
const EMPTY_MEMORY_HONOUR: i64 = 129;

/// Diplomatic memory of most factions (`FACTION_FLAG_AND_COLOURS@2+21`): only its fourth value is set.
const DEFAULT_MEMORY: [u8; 22] = [0, 0, 0, 10, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];

//---------------------------------------------------------------------------//
//                              Implementations
//---------------------------------------------------------------------------//

/// Writes the faction array.
///
/// # Errors
///
/// Returns an error if a table, column or layout is missing.
pub fn write_faction_array(inputs: &WorldInputs, plan: &WorldPlan) -> Result<RecordNode> {
    let mut array = child(inputs.layout, WORLD_PATH, "FACTION_ARRAY")?;
    for faction in plan.factions() {
        array.push_group()?.set_record("FACTION", Some(write_faction(inputs, plan, faction)?))?;
    }
    array.build()
}

/// Writes a faction.
///
/// # Errors
///
/// Returns an error if a table, column or layout is missing.
pub fn write_faction(inputs: &WorldInputs, plan: &WorldPlan, faction: &FactionEntry) -> Result<RecordNode> {
    let layout = inputs.layout;
    let key = faction.key();
    let row = start_pos_faction_row(inputs.tables, faction)?;
    let summary = faction_summary(inputs.tables, inputs.index, faction)?;
    let faction_id = |key: &Option<String>| key.as_deref().and_then(|key| plan.faction_id(key)).unwrap_or_default();

    let mut setup = child(layout, FACTION_PATH, "CAMPAIGN_PLAYER_SETUP")?;
    setup.group()?
        .set_str("CAMPAIGN_PLAYER_SETUP_INGAME_MODIFIABLES+1", key)?
        .set_bool("CAMPAIGN_PLAYER_SETUP_INGAME_MODIFIABLES+3", faction.playable())?;

    let mut economics = child(layout, FACTION_PATH, "FACTION_ECONOMICS")?;
    economics.group()?.set_i32("history+1", i32::try_from(row.int("treasury")?).unwrap_or(i32::MAX))?;

    let mut government = child(layout, FACTION_PATH, "GOVERNMENT")?;
    government.group()?.set_str("^+2", &row.str("government_type")?)?;

    let mut religion = child(layout, FACTION_PATH, "FACTION_RELIGION_MANAGER")?;
    religion.group()?.set_str("^+1", &row.str("state_religion")?)?;

    let mut localisation = child(layout, FACTION_PATH, "CAMPAIGN_LOCALISATION")?;
    localisation.group()?.set_str("^+1", &format!("{SCREEN_NAME_PREFIX}{key}"))?;

    let [flag, republican_flag] = faction_flags(inputs.tables, key)?;
    let honour = row.int("honour")?;
    let memory = if honour >= EMPTY_MEMORY_HONOUR { vec![0; DEFAULT_MEMORY.len()] } else { DEFAULT_MEMORY.to_vec() };

    let mut horde = None;
    if let Some(is_horde) = summary.horde() {
        let mut manager = child(layout, FACTION_PATH, "FACTION_HORDE_MANAGER")?;
        manager.group()?.set_i32("^+1", i32::from(is_horde))?;
        horde = Some(manager.build()?);
    }

    let mut record = layout.builder(FACTION_PATH)?;
    record.group()?
        .set_u32("^+1", plan.faction_id(key).unwrap_or_default())?
        .set_str("^+2", key)?
        .set_record("CAMPAIGN_PLAYER_SETUP", Some(setup.build()?))?
        .set_record("FACTION_ECONOMICS", Some(economics.build()?))?
        .set_record("GOVERNMENT", Some(government.build()?))?
        .set_record("FACTION_HORDE_MANAGER", horde)?
        .set_bool("GOVERNMENT+3", row.bool("is_major")?)?
        .set_record("FACTION_RELIGION_MANAGER", Some(religion.build()?))?
        .set_bool("MORGUE+1", faction.playable())?
        .set_strings("MORGUE+2", summary.victory_regions().to_vec())?
        .set_u32("MORGUE+3", summary.capital().as_deref().and_then(|capital| plan.region_id(capital)).unwrap_or_default())?
        .set_u32("MORGUE+4", summary.capital().as_deref().and_then(|capital| plan.region_id(capital)).unwrap_or_default())?
        .set_bool("PRESTIGE+1", summary.without_characters())?
        .set_str("PRESTIGE+6", &row.str("cdir_military_generator_config")?)?
        .set_record("FACTION_FLAG_AND_COLOURS", Some(write_flag(inputs, &flag)?))?
        .set_record("FACTION_FLAG_AND_COLOURS@2", Some(write_flag(inputs, &republican_flag)?))?
        .set_u32("FACTION_FLAG_AND_COLOURS@2+1", faction_id(summary.client_state_of()))?
        .set_u32("FACTION_FLAG_AND_COLOURS@2+2", faction_id(summary.vassal_of()))?
        .set_bool("FACTION_FLAG_AND_COLOURS@2+5", summary.has_war_or_vassal())?
        .set_u32("FACTION_FLAG_AND_COLOURS@2+14", u32::try_from(summary.region_count()).unwrap_or(u32::MAX))?
        .set_bytes("FACTION_FLAG_AND_COLOURS@2+21", memory)?
        .set_record("CAMPAIGN_LOCALISATION", Some(localisation.build()?))?
        .set_strings("FACTION_ACTION_LOG_BLOCK+2", summary.subjects().to_vec())?;

    record.build()
}

/// Writes a flag record (`FACTION_FLAG_AND_COLOURS`): the flag path, then each colour's red, green and blue.
fn write_flag(inputs: &WorldInputs, flag: &FactionFlag) -> Result<RecordNode> {
    let mut record = child(inputs.layout, FACTION_PATH, "FACTION_FLAG_AND_COLOURS")?;
    let group = record.group()?;
    group.set_str("^+1", flag.path())?;
    for (index, value) in flag.colours().iter().flatten().enumerate() {
        group.set_u32(&format!("^+{}", index + 2), u32::from(*value))?;
    }
    record.build()
}
