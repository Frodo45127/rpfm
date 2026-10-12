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

use std::collections::HashMap;

use rpfm_lib::error::Result;
use rpfm_lib::files::esf::RecordNode;

use super::armies::ArmyWriter;
use super::characters::CharacterWriter;
use super::government::{write_character_posts, PostPlan};
use super::provinces::write_mercenary_pool;
use super::sight::{write_shroud, SightPlan};
use super::{child, next_id, start_pos_faction_row, WorldInputs, WorldPlan, WORLD_PATH};
use crate::startpos::layout::{child_path, StartposLayout};
use crate::startpos::vision::Vision;
use crate::startpos::world::agents::recruitment_pool_agents;
use crate::startpos::world::diplomacy::Diplomacy;
use crate::startpos::world::government::political_parties;
use crate::startpos::world::pools::{faction_mercenary_pool, levy_pools};
use crate::startpos::world::FactionEntry;
use crate::startpos::world::factions::{faction_flags, faction_summary, FactionFlag};

/// Path of a faction.
const FACTION_PATH: &str = "CAMPAIGN_STARTPOS/CAMPAIGN_ENV/CAMPAIGN_MODEL/WORLD/FACTION_ARRAY/FACTION";

/// Localisation key prefix of the faction names.
const SCREEN_NAME_PREFIX: &str = "factions_screen_name_";

/// Lowest `start_pos_factions.honour` of the factions that start with an empty diplomatic memory (`+21` field).
const EMPTY_MEMORY_HONOUR: i64 = 129;

/// Length of the per-type event counters of a faction (`FACTION_FLAG_AND_COLOURS@2+19`). The game fills some of them
/// for playable factions during the build (source unknown); zeros are compatible.
const EVENT_COUNTERS: usize = 151;

/// Culture whose factions have `FACTION_ACTION_LOG_BLOCK+12` unset (meaning unknown).
const NOMADIC_CULTURE: &str = "att_cult_nomadic";

/// Diplomatic memory of most factions (`FACTION_FLAG_AND_COLOURS@2+21`): only its fourth value is set.
const DEFAULT_MEMORY: [u8; 22] = [0, 0, 0, 10, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];

//---------------------------------------------------------------------------//
//                              Types
//---------------------------------------------------------------------------//

/// What the faction writer takes from the other writers and plans.
pub struct FactionContext<'c, 'a, 'b> {

    /// Writer of the forces.
    pub armies: &'c ArmyWriter<'a, 'b>,

    /// Writer of the characters.
    pub characters: &'c CharacterWriter<'a, 'b>,

    /// Government posts of the factions.
    pub posts: &'c PostPlan,

    /// Levy pools of the overlords.
    pub levy_pools: &'c [PlannedLevyPool],

    /// Diplomatic relationships between the factions.
    pub diplomacy: &'c Diplomacy,

    /// Vision of the world.
    pub sight: &'c SightPlan,
}

/// A levy pool of an overlord, with its runtime ids.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlannedLevyPool {

    /// Mercenary pool id.
    id: u32,

    /// Overlord faction key.
    overlord: String,

    /// Runtime id of the subject faction.
    subject: u32,

    /// Runtime id of the province.
    province: u32,
}

//---------------------------------------------------------------------------//
//                              Implementations
//---------------------------------------------------------------------------//

/// Writes the rebel faction (`REBEL_FACTION`): the layout's, with the number of regions without an owner.
///
/// # Errors
///
/// Returns an error if the layout lacks the records.
pub fn write_rebel_faction(layout: &StartposLayout, plan: &WorldPlan) -> Result<RecordNode> {
    let path = child_path(WORLD_PATH, "REBEL_FACTION");
    let region_count = plan.regions().iter().filter(|region| region.owner().is_none()).count();
    let mut faction = layout.builder(&child_path(&path, "FACTION"))?;
    faction.group()?.set_u32("FACTION_FLAG_AND_COLOURS@2+14", u32::try_from(region_count).unwrap_or(u32::MAX))?;

    let mut rebels = layout.builder(&path)?;
    rebels.group()?.set_record("FACTION", Some(faction.build()?))?;
    rebels.build()
}

/// Writes the faction array.
///
/// # Errors
///
/// Returns an error if a table, column or layout is missing.
pub fn write_faction_array(inputs: &WorldInputs, plan: &WorldPlan, context: &FactionContext) -> Result<RecordNode> {
    let mut array = child(inputs.layout, WORLD_PATH, "FACTION_ARRAY")?;
    for faction in plan.factions() {
        array.push_group()?.set_record("FACTION", Some(write_faction(inputs, plan, context, faction)?))?;
    }
    array.build()
}

/// Plans the levy pools of the overlords: their ids continue the province pools, in order of the subject's and the
/// province's runtime ids.
///
/// # Arguments
///
/// * `inputs` - Tables, startpos rows and layouts.
/// * `plan` - Order and runtime ids of the world's objects.
/// * `province_ids` - Runtime ids of the provinces, by key.
/// * `first_pool` - Id of the first levy pool.
///
/// # Errors
///
/// Returns an error if a table or column is missing.
pub fn plan_levy_pools(inputs: &WorldInputs, plan: &WorldPlan, province_ids: &HashMap<String, u32>, first_pool: u32) -> Result<Vec<PlannedLevyPool>> {
    let mut pools = levy_pools(inputs.tables, inputs.index)?.into_iter()
        .map(|pool| PlannedLevyPool {
            id: 0,
            subject: plan.faction_id(pool.subject()).unwrap_or_default(),
            province: province_ids.get(pool.province()).copied().unwrap_or_default(),
            overlord: pool.overlord().to_owned(),
        })
        .collect::<Vec<_>>();
    pools.sort_by_key(|pool| (pool.subject, pool.province));

    let mut next_pool = first_pool;
    for pool in &mut pools {
        pool.id = next_id(&mut next_pool);
    }
    Ok(pools)
}

/// Writes a faction.
///
/// # Errors
///
/// Returns an error if a table, column or layout is missing.
pub fn write_faction(inputs: &WorldInputs, plan: &WorldPlan, context: &FactionContext, faction: &FactionEntry) -> Result<RecordNode> {
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

    let posts = write_character_posts(inputs, context.posts, key, context.characters.leader_id(faction))?;
    let mut government = child(layout, FACTION_PATH, "GOVERNMENT")?;
    government.group()?
        .set_str("^+2", &row.str("government_type")?)?
        .set_record("CHARACTER_POSTS", Some(posts))?;

    let mut religion = child(layout, FACTION_PATH, "FACTION_RELIGION_MANAGER")?;
    religion.group()?.set_str("^+1", &row.str("state_religion")?)?;

    let mut localisation = child(layout, FACTION_PATH, "CAMPAIGN_LOCALISATION")?;
    localisation.group()?.set_str("^+1", &format!("{SCREEN_NAME_PREFIX}{key}"))?.unset("^+2")?;

    let [flag, republican_flag] = faction_flags(inputs.tables, key)?;
    let honour = row.int("honour")?;
    let memory = if honour >= EMPTY_MEMORY_HONOUR { vec![0; DEFAULT_MEMORY.len()] } else { DEFAULT_MEMORY.to_vec() };

    let mut horde = None;
    if let Some(is_horde) = summary.horde() {
        let mut manager = child(layout, FACTION_PATH, "FACTION_HORDE_MANAGER")?;
        manager.group()?.set_i32("^+1", i32::from(is_horde))?;
        horde = Some(manager.build()?);
    }

    let mission_manager = faction.playable().then(|| layout.default_record(&format!("{FACTION_PATH}/CAMPAIGN_MISSION_MANAGER"))).transpose()?;
    let shroud = if faction.playable() { Some(write_shroud(layout, FACTION_PATH, context.sight, &faction_vision(inputs, context, faction)?)?) } else { None };

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
        .set_record("CHARACTER_ARRAY", Some(context.characters.write_character_array(faction)?))?
        .set_record("FACTION_TECHNOLOGY_MANAGER", Some(write_technologies(inputs, faction)?))?
        .set_record("ARMY_ARRAY", Some(context.armies.write_army_array(faction)?))?
        .set_bool("MORGUE+1", faction.playable())?
        .set_strings("MORGUE+2", summary.victory_regions().to_vec())?
        .set_u32("MORGUE+3", summary.capital().as_deref().and_then(|capital| plan.region_id(capital)).unwrap_or_default())?
        .set_u32("MORGUE+4", summary.capital().as_deref().and_then(|capital| plan.region_id(capital)).unwrap_or_default())?
        .set_bool("PRESTIGE+1", summary.without_characters())?
        .set_str("PRESTIGE+6", &row.str("cdir_military_generator_config")?)?
        .set_record("FACTION_FLAG_AND_COLOURS", Some(write_flag(inputs, FACTION_PATH, &flag)?))?
        .set_record("FACTION_FLAG_AND_COLOURS@2", Some(write_flag(inputs, FACTION_PATH, &republican_flag)?))?
        .set_u32("FACTION_FLAG_AND_COLOURS@2+1", faction_id(summary.client_state_of()))?
        .set_u32("FACTION_FLAG_AND_COLOURS@2+2", faction_id(summary.vassal_of()))?
        .set_bool("FACTION_FLAG_AND_COLOURS@2+5", summary.has_war_or_vassal())?
        .set_u32("FACTION_FLAG_AND_COLOURS@2+14", u32::try_from(summary.region_count()).unwrap_or(u32::MAX))?
        .set_bytes("FACTION_FLAG_AND_COLOURS@2+21", memory)?
        .set_record("CAMPAIGN_LOCALISATION", Some(localisation.build()?))?
        .set_strings("FACTION_ACTION_LOG_BLOCK+2", summary.subjects().to_vec())?
        .set_record("FACTION_MERCENARY_POOL", Some(write_faction_mercenary_pool(inputs, plan, faction)?))?
        .set_record("LEVY_MERCENARY_MANAGER", Some(write_levy_pools(inputs, context.levy_pools, faction)?))?
        .set_record("CHARACTER_RECRUITMENT_POOL_MANAGER", Some(write_recruitment_pools(inputs, faction)?))?
        .set_record("CAMPAIGN_POLITICS", Some(write_politics(inputs, faction)?))?
        .set_record("OLD_DIPLOMACY_MANAGER", Some(write_diplomacy(inputs, plan, context.diplomacy, faction)?))?
        .set_record("MILITARY_FORCE_LEGACY_NAMES_AND_EMBLEMS", Some(context.armies.write_used_legacies(faction)?))?
        .set_record("CAMPAIGN_MISSION_MANAGER", mission_manager)?
        .set_record("CAMPAIGN_SHROUD", shroud)?
        .set_u32s("FACTION_FLAG_AND_COLOURS@2+19", vec![0; EVENT_COUNTERS])?
        .set_u32("FACTION_ACTION_LOG_BLOCK+11", context.characters.progenitor_family_id(faction))?
        .set_bool("FACTION_ACTION_LOG_BLOCK+12", *faction.culture() != NOMADIC_CULTURE)?;

    record.build()
}

/// Returns what a faction sees: its regions, its settlements and its forces' commanders.
fn faction_vision(inputs: &WorldInputs, context: &FactionContext, faction: &FactionEntry) -> Result<Vision> {
    let mut vision = Vision::default();
    for region in inputs.index.regions_of(faction.key()) {
        for area in [context.sight.region(region.key()), context.sight.settlement(region.key())].into_iter().flatten() {
            vision.extend(area);
        }
    }
    for character in context.characters.characters_of(faction) {
        if let Some(area) = context.characters.vision(character)? {
            vision.extend(&area);
        }
    }
    Ok(vision)
}

/// Writes the technologies a faction starts with (`start_pos_technologies`), in table order (the game uses a hash
/// order).
fn write_technologies(inputs: &WorldInputs, faction: &FactionEntry) -> Result<RecordNode> {
    let path = format!("{FACTION_PATH}/FACTION_TECHNOLOGY_MANAGER");
    let mut researched = child(inputs.layout, &path, "RESEARCHED_TECHS")?;
    for row in inputs.tables.table("start_pos_technologies")?.rows() {
        if row.str("faction")? == *faction.id() {
            researched.push_group()?
                .set_str("^+1", &row.str("technology")?)?
                .set_bool("^+2", true)?;
        }
    }

    let mut record = inputs.layout.builder(&path)?;
    record.group()?
        .set_str("^+1", faction.key())?
        .set_record("RESEARCHED_TECHS", Some(researched.build()?))?;
    record.build()
}

/// Writes the mercenary pool of a faction. Its id is the faction's runtime id.
fn write_faction_mercenary_pool(inputs: &WorldInputs, plan: &WorldPlan, faction: &FactionEntry) -> Result<RecordNode> {
    let path = format!("{FACTION_PATH}/FACTION_MERCENARY_POOL");
    let units = faction_mercenary_pool(inputs.tables, faction.key())?;
    let mut record = inputs.layout.builder(&path)?;
    record.group()?
        .set_u32("^+1", plan.faction_id(faction.key()).unwrap_or_default())?
        .set_record("MERCENARY_POOL", Some(write_mercenary_pool(inputs, &path, &units)?))?;
    record.build()
}

/// Writes the levy pools of an overlord: an empty mercenary pool per subject and province.
fn write_levy_pools(inputs: &WorldInputs, pools: &[PlannedLevyPool], faction: &FactionEntry) -> Result<RecordNode> {
    let path = format!("{FACTION_PATH}/LEVY_MERCENARY_MANAGER");
    let pool_path = format!("{path}/LEVY_POOLS_BLOCK/LEVY_MERCENARY_POOL");
    let mut block = child(inputs.layout, &path, "LEVY_POOLS_BLOCK")?;
    for pool in pools.iter().filter(|pool| pool.overlord == *faction.key()) {
        let mut record = inputs.layout.builder(&pool_path)?;
        record.group()?
            .set_u32("^+1", pool.id)?
            .set_record("MERCENARY_POOL", Some(write_mercenary_pool(inputs, &pool_path, &[])?))?
            .set_u32("MERCENARY_POOL+1", pool.subject)?
            .set_u32("MERCENARY_POOL+2", pool.province)?;
        block.push_group()?.set_record("LEVY_MERCENARY_POOL", Some(record.build()?))?;
    }

    let mut manager = inputs.layout.builder(&path)?;
    manager.group()?.set_record("LEVY_POOLS_BLOCK", Some(block.build()?))?;
    manager.build()
}

/// Writes the character recruitment pools of a faction: an empty pool per agent type it can recruit.
fn write_recruitment_pools(inputs: &WorldInputs, faction: &FactionEntry) -> Result<RecordNode> {
    let path = format!("{FACTION_PATH}/CHARACTER_RECRUITMENT_POOL_MANAGER");
    let mut block = child(inputs.layout, &path, "CHARACTER_RECRUITMENT_POOL_BLOCK")?;
    for agent in recruitment_pool_agents(inputs.tables, faction)? {
        block.push_group()?.set_str("^+1", &agent)?;
    }

    let mut manager = inputs.layout.builder(&path)?;
    manager.group()?.set_record("CHARACTER_RECRUITMENT_POOL_BLOCK", Some(block.build()?))?;
    manager.build()
}

/// Writes the diplomatic relationships of a faction with every other faction, in startpos order (the game uses a
/// hash order).
fn write_diplomacy(inputs: &WorldInputs, plan: &WorldPlan, diplomacy: &Diplomacy, faction: &FactionEntry) -> Result<RecordNode> {
    let path = format!("{FACTION_PATH}/OLD_DIPLOMACY_MANAGER");
    let relationship_path = format!("{path}/DIPLOMACY_RELATIONSHIPS_ARRAY/OLD_DIPLOMACY_RELATIONSHIP");
    let mut array = child(inputs.layout, &path, "DIPLOMACY_RELATIONSHIPS_ARRAY")?;
    for other in plan.factions().iter().filter(|other| other.key() != faction.key()) {
        let relationship = diplomacy.relationship(faction.key(), other.key());
        let other_id = plan.faction_id(other.key()).unwrap_or_default();

        let mut allied = child(inputs.layout, &relationship_path, "ALLIED_IN_WAR_AGAINST")?;
        for (enemy, defensive_ally) in relationship.allied_against() {
            allied.push_group()?
                .set_u32("^+1", plan.faction_id(enemy).unwrap_or_default())?
                .set_i32("^+2", if *defensive_ally { -1 } else { 0 })?;
        }

        let (vassal_value, master_value) = relationship.vassal_values();
        let mut record = inputs.layout.builder(&relationship_path)?;
        record.group()?
            .set_u32("^+1", other_id)?
            .set_bool("^+2", relationship.trade())?
            .set_i32("^+3", if relationship.allied_against().is_empty() { 0 } else { -1 })?
            .set_str("^+4", relationship.stance())?
            .set_i32("^+8", vassal_value)?
            .set_i32("^+9", master_value)?
            .set_record("ALLIED_IN_WAR_AGAINST", Some(allied.build()?))?
            .set_bool("HOSTAGES+2", relationship.known())?
            .set_bool("HOSTAGES+3", relationship.non_aggression())?
            .set_i32("HOSTAGES+4", if relationship.military_access() { -1 } else { 0 })?;
        array.push_group()?
            .set_u32("^+1", other_id)?
            .set_record("OLD_DIPLOMACY_RELATIONSHIP", Some(record.build()?))?;
    }

    let mut manager = inputs.layout.builder(&path)?;
    manager.group()?.set_record("DIPLOMACY_RELATIONSHIPS_ARRAY", Some(array.build()?))?;
    manager.build()
}

/// Writes the politics of a faction: its political parties with their starting power.
fn write_politics(inputs: &WorldInputs, faction: &FactionEntry) -> Result<RecordNode> {
    let path = format!("{FACTION_PATH}/CAMPAIGN_POLITICS");
    let parties_path = format!("{path}/POLITICAL_PARTIES");
    let parties = political_parties(inputs.tables, faction.key())?;
    let mut array = child(inputs.layout, &path, "POLITICAL_PARTIES")?;
    for party in &parties {
        let mut record = child(inputs.layout, &parties_path, "CAMPAIGN_POLITICAL_PARTY")?;
        record.group()?
            .set_str("^+1", party.key())?
            .set_u32("^+2", party.power())?;
        array.push_group()?
            .set_str("^+1", party.key())?
            .set_record("CAMPAIGN_POLITICAL_PARTY", Some(record.build()?))?;
    }

    let mut politics = inputs.layout.builder(&path)?;
    politics.group()?
        .set_record("POLITICAL_PARTIES", Some(array.build()?))?
        .set_bool("POLITICAL_PARTIES+1", !parties.is_empty())?;
    politics.build()
}

/// Writes the flag record (`FACTION_FLAG_AND_COLOURS`) of a parent record: the flag path, then each colour's red,
/// green and blue.
pub(super) fn write_flag(inputs: &WorldInputs, parent: &str, flag: &FactionFlag) -> Result<RecordNode> {
    let mut record = child(inputs.layout, parent, "FACTION_FLAG_AND_COLOURS")?;
    let group = record.group()?;
    group.set_str("^+1", flag.path())?;
    for (index, value) in flag.colours().iter().flatten().enumerate() {
        group.set_u32(&format!("^+{}", index + 2), u32::from(*value))?;
    }
    record.build()
}
