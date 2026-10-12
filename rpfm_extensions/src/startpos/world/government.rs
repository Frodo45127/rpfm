//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Government of each faction: its posts (`FACTION/GOVERNMENT/CHARACTER_POSTS`), its political parties
//! (`FACTION/CAMPAIGN_POLITICS`) and its leader.
//!
//! Verified on Attila's main campaign: posts match for 118 of 118 factions, party sets for 118 of 118 and party
//! powers for 100 of 100 parties, and the leader for the 60 factions with characters. The game stores the post
//! groups and some party lists in hash order.

use getset::{CopyGetters, Getters};

use std::collections::HashMap;

use rpfm_lib::error::Result;

use super::{CampaignIndex, CharacterEntry, FactionEntry};
use crate::startpos::tables::StartposTables;

#[cfg(test)] #[path = "government_test.rs"] mod government_test;

/// Ministerial position held by faction leaders.
pub const FACTION_LEADER_POSITION: &str = "faction_leader";

//---------------------------------------------------------------------------//
//                              Types
//---------------------------------------------------------------------------//

/// A group of identical posts of a faction's government.
#[derive(Debug, Clone, PartialEq, Eq, Default, Getters, CopyGetters)]
pub struct PostGroup {

    /// Ministerial position key.
    #[getset(get = "pub")]
    position: String,

    /// `unique_id` of the `ministerial_positions_culture_details` row that gives the posts.
    #[getset(get_copy = "pub")]
    unique_id: i64,

    /// Number of posts (`ministerial_positions.maximum_concurrent_ministers`).
    #[getset(get_copy = "pub")]
    count: u32,
}

/// A political party of a faction.
#[derive(Debug, Clone, PartialEq, Eq, Default, Getters, CopyGetters)]
pub struct PoliticalParty {

    /// Party key.
    #[getset(get = "pub")]
    key: String,

    /// Starting power.
    #[getset(get_copy = "pub")]
    power: u32,
}

//---------------------------------------------------------------------------//
//                              Implementations
//---------------------------------------------------------------------------//

/// Returns the post groups of a faction's government.
///
/// The groups are the `ministerial_positions_culture_details` rows for the faction's culture (without subculture
/// and faction), its subculture (without faction) and the faction itself: the campaign's rows if it has any for the
/// faction, else the rows without campaign.
///
/// # Arguments
///
/// * `tables` - Tables with `ministerial_positions_culture_details` and `ministerial_positions`.
/// * `index` - The campaign's startpos rows.
/// * `faction` - The faction.
///
/// # Returns
///
/// The post groups, in table order.
///
/// # Errors
///
/// Returns an error if a table or column is missing.
pub fn government_posts(tables: &StartposTables, index: &CampaignIndex, faction: &FactionEntry) -> Result<Vec<PostGroup>> {
    let mut concurrent = HashMap::new();
    for row in tables.table("ministerial_positions")?.rows() {
        concurrent.insert(row.str("minister_key")?.into_owned(), row.int("maximum_concurrent_ministers")?);
    }

    // Rows of the campaign replace the ones without campaign, if the campaign has any for the faction.
    let mut generic = vec![];
    let mut own = vec![];
    for row in tables.table("ministerial_positions_culture_details")?.rows() {
        let campaign = row.str("campaign")?;
        if !campaign.is_empty() && campaign != *index.campaign() {
            continue;
        }

        let row_faction = row.str("faction_key")?;
        let row_subculture = row.str("subculture_key")?;
        let matches = row_faction == *faction.key()
            || (row_faction.is_empty() && row_subculture == *faction.subculture())
            || (row_faction.is_empty() && row_subculture.is_empty() && row.str("culture_key")? == *faction.culture());
        if !matches {
            continue;
        }

        let position = row.str("ministerial_position_key")?.into_owned();
        let group = PostGroup {
            count: concurrent.get(&position).and_then(|&count| u32::try_from(count).ok()).unwrap_or_default(),
            unique_id: row.int("unique_id")?,
            position,
        };
        if campaign.is_empty() { generic.push(group) } else { own.push(group) }
    }

    let groups = if own.is_empty() { generic } else { own };
    Ok(groups)
}

/// Returns the political parties of a faction with their starting power.
///
/// The faction's `senator_total` is split between its parties (`faction_political_parties_junctions`) in
/// proportion to their `initial_power`, or evenly if they all start with no power.
///
/// # Arguments
///
/// * `tables` - Tables with `faction_political_parties_junctions`, `political_parties` and `factions`.
/// * `faction` - Faction key.
///
/// # Returns
///
/// The parties, in table order. Empty for factions without parties.
///
/// # Errors
///
/// Returns an error if a table or column is missing.
pub fn political_parties(tables: &StartposTables, faction: &str) -> Result<Vec<PoliticalParty>> {
    let mut senator_total = 0;
    for row in tables.table("factions")?.rows() {
        if row.str("key")? == faction {
            senator_total = row.int("senator_total")?;
        }
    }

    let mut initial_power = HashMap::new();
    for row in tables.table("political_parties")?.rows() {
        initial_power.insert(row.str("key")?.into_owned(), row.int("initial_power")?);
    }

    let mut parties = vec![];
    for row in tables.table("faction_political_parties_junctions")?.rows() {
        if row.str("faction_key")? == faction {
            let key = row.str("political_party_key")?.into_owned();
            parties.push((initial_power.get(&key).copied().unwrap_or_default(), key));
        }
    }

    let total_power = parties.iter().map(|(power, _)| power).sum::<i64>();
    let party_count = i64::try_from(parties.len()).unwrap_or(i64::MAX);
    Ok(parties.into_iter().map(|(power, key)| {
        let share = if total_power == 0 { senator_total / party_count.max(1) } else { senator_total * power / total_power };
        PoliticalParty { key, power: u32::try_from(share.max(0)).unwrap_or(u32::MAX) }
    }).collect())
}

/// Returns the leader of a faction: its first living character with the `faction_leader` position, or its first
/// living character if no row has it. The startpos lists the leader first in the faction's characters.
///
/// The leader holds the faction's `faction_leader` post (`CHARACTER` #8 is the post id, with #28 = 4 and #29 = true).
///
/// # Arguments
///
/// * `index` - The campaign's startpos rows.
/// * `faction` - Faction key.
///
/// # Returns
///
/// The leader, or `None` for factions without living characters.
pub fn faction_leader<'a>(index: &'a CampaignIndex, faction: &'a str) -> Option<&'a CharacterEntry> {
    let mut living = index.characters_of(faction).filter(|character| character.death_type().is_empty()).peekable();
    let first = living.peek().copied();
    living.find(|character| character.ministerial_position() == FACTION_LEADER_POSITION).or(first)
}
