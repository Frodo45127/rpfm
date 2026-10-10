//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Agent data of characters and factions: abilities (`CHARACTER_DETAILS/AGENT_ABILITIES`), attributes
//! (`AGENT_ATTRIBUTES`), skill sets (`CAMPAIGN_SKILLS`) and the agent types of the recruitment pools
//! (`FACTION/CHARACTER_RECRUITMENT_POOL_MANAGER`).
//!
//! Verified on Attila's main campaign for every living character and every faction. The game stores the
//! recruitment pools in hash order.

use std::collections::HashMap;

use rpfm_lib::error::Result;

use super::{CampaignIndex, FactionEntry};
use crate::startpos::tables::StartposTables;

#[cfg(test)] #[path = "agents_test.rs"] mod agents_test;

/// Agent abilities, in the order the startpos stores them.
pub const ABILITIES: [&str; 9] = [
    "command_force",
    "assist_army",
    "assist_province",
    "hinder_army",
    "hinder_character",
    "hinder_agent",
    "hinder_province",
    "hinder_settlement",
    "passive_ability",
];

/// Agent attributes, in the order the startpos stores them.
pub const ATTRIBUTES: [&str; 3] = ["subterfuge", "zeal", "authority"];

//---------------------------------------------------------------------------//
//                              Implementations
//---------------------------------------------------------------------------//

/// Returns which abilities an agent type has, in [`ABILITIES`] order (`agent_to_agent_abilities`).
///
/// # Arguments
///
/// * `tables` - Tables with `agent_to_agent_abilities`.
/// * `agent` - Agent type.
///
/// # Errors
///
/// Returns an error if a table or column is missing.
pub fn agent_abilities(tables: &StartposTables, agent: &str) -> Result<[bool; ABILITIES.len()]> {
    let mut abilities = [false; ABILITIES.len()];
    for row in tables.table("agent_to_agent_abilities")?.rows() {
        if row.str("agent")? == agent {
            let ability = row.str("ability")?;
            if let Some(index) = ABILITIES.iter().position(|known| *known == ability) {
                abilities[index] = true;
            }
        }
    }
    Ok(abilities)
}

/// Returns the starting attributes of an agent type, in [`ATTRIBUTES`] order (`agent_to_agent_attributes`, 0 when
/// the agent has no row for an attribute).
///
/// # Arguments
///
/// * `tables` - Tables with `agent_to_agent_attributes`.
/// * `agent` - Agent type.
///
/// # Errors
///
/// Returns an error if a table or column is missing.
pub fn agent_attributes(tables: &StartposTables, agent: &str) -> Result<[i64; ATTRIBUTES.len()]> {
    let mut attributes = [0; ATTRIBUTES.len()];
    for row in tables.table("agent_to_agent_attributes")?.rows() {
        if row.str("agent")? == agent {
            let attribute = row.str("attribute")?;
            if let Some(index) = ATTRIBUTES.iter().position(|known| *known == attribute) {
                attributes[index] = row.int("default_value")?;
            }
        }
    }
    Ok(attributes)
}

/// Returns the skill set of a character (`character_skill_node_sets` row for its agent type and subculture).
///
/// # Arguments
///
/// * `tables` - Tables with `character_skill_node_sets`.
/// * `index` - The campaign's startpos rows.
/// * `agent` - Agent type.
/// * `faction` - The character's faction.
///
/// # Returns
///
/// The skill set key, or `None` if there isn't one (colonels).
///
/// # Errors
///
/// Returns an error if a table or column is missing.
pub fn skill_set(tables: &StartposTables, index: &CampaignIndex, agent: &str, faction: &FactionEntry) -> Result<Option<String>> {
    for row in tables.table("character_skill_node_sets")?.rows() {
        let campaign = row.str("campaign_key")?;
        let row_faction = row.str("faction_key")?;
        if row.str("agent_key")? == agent
            && row.str("subculture")? == *faction.subculture()
            && (row_faction.is_empty() || row_faction == *faction.key())
            && (campaign.is_empty() || campaign == *index.campaign()) {
            return Ok(Some(row.str("key")?.into_owned()));
        }
    }
    Ok(None)
}

/// Returns the agent types with a recruitment pool in a faction.
///
/// They're the agent types of the faction's culture in `agent_culture_details` whose `agents.faction_total_cap`
/// is above 0 (which leaves out colonels and ministers).
///
/// # Arguments
///
/// * `tables` - Tables with `agent_culture_details` and `agents`.
/// * `faction` - The faction.
///
/// # Returns
///
/// The agent types, in `agent_culture_details` order, without repeats.
///
/// # Errors
///
/// Returns an error if a table or column is missing.
pub fn recruitment_pool_agents(tables: &StartposTables, faction: &FactionEntry) -> Result<Vec<String>> {
    let mut caps = HashMap::new();
    for row in tables.table("agents")?.rows() {
        caps.insert(row.str("key")?.into_owned(), row.int("faction_total_cap")?);
    }

    let mut agents = vec![];
    for row in tables.table("agent_culture_details")?.rows() {
        let agent = row.str("agent")?.into_owned();
        if row.str("culture")? == *faction.culture() && caps.get(&agent).is_some_and(|&cap| cap > 0) && !agents.contains(&agent) {
            agents.push(agent);
        }
    }

    Ok(agents)
}
