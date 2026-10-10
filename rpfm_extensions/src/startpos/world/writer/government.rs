//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Plan and writer of the government posts of the factions (`FACTION/GOVERNMENT/CHARACTER_POSTS`).
//!
//! Post ids are one counter over the factions, in startpos order. The game lists each faction's post groups in a
//! hash order; this plan uses table order.

use std::collections::HashMap;

use rpfm_lib::error::Result;
use rpfm_lib::files::esf::{NodeType, RecordNode};

use super::{child, WorldInputs, WorldPlan};
use crate::startpos::world::government::{government_posts, PostGroup, FACTION_LEADER_POSITION};

/// Path of a faction's government.
const GOVERNMENT_PATH: &str = "CAMPAIGN_STARTPOS/CAMPAIGN_ENV/CAMPAIGN_MODEL/WORLD/FACTION_ARRAY/FACTION/GOVERNMENT";

/// Path of the post groups of a government.
const POSTS_PATH: &str = "CAMPAIGN_STARTPOS/CAMPAIGN_ENV/CAMPAIGN_MODEL/WORLD/FACTION_ARRAY/FACTION/GOVERNMENT/CHARACTER_POSTS";

//---------------------------------------------------------------------------//
//                              Types
//---------------------------------------------------------------------------//

/// Government posts of every faction, with their runtime ids.
#[derive(Debug, Clone, Default)]
pub struct PostPlan {

    /// Post groups of each faction with the id of their first post, by faction key.
    groups: HashMap<String, Vec<(PostGroup, u32)>>,
}

//---------------------------------------------------------------------------//
//                              Implementations
//---------------------------------------------------------------------------//

impl PostPlan {

    /// Plans the posts of every faction.
    ///
    /// # Errors
    ///
    /// Returns an error if a table or column is missing.
    pub fn new(inputs: &WorldInputs, plan: &WorldPlan) -> Result<Self> {
        let mut groups = HashMap::new();
        let mut next_post = 1;
        for faction in plan.factions() {
            let mut faction_groups = vec![];
            for group in government_posts(inputs.tables, inputs.index, faction)? {
                let count = group.count();
                faction_groups.push((group, next_post));
                next_post += count;
            }
            groups.insert(faction.key().to_owned(), faction_groups);
        }
        Ok(Self { groups })
    }

    /// Returns the post groups of a faction, with the id of their first post.
    pub fn groups_of(&self, faction: &str) -> &[(PostGroup, u32)] {
        self.groups.get(faction).map(Vec::as_slice).unwrap_or_default()
    }

    /// Returns the id of a faction's leader post, if it has one.
    pub fn leader_post(&self, faction: &str) -> Option<u32> {
        self.groups_of(faction).iter().find(|(group, _)| group.position() == FACTION_LEADER_POSITION).map(|(_, id)| *id)
    }
}

/// Writes the posts of a faction's government (`CHARACTER_POSTS`): one group per post group, with its post count and
/// that many posts. Only the leader post has a holder.
///
/// # Arguments
///
/// * `inputs` - Tables, startpos rows and layouts.
/// * `posts` - Government posts of the factions.
/// * `faction` - Faction key.
/// * `leader` - Runtime id of the faction's leader, if it has one.
///
/// # Errors
///
/// Returns an error if the layout lacks a record, or a group has more posts than the layout knows.
pub fn write_character_posts(inputs: &WorldInputs, posts: &PostPlan, faction: &str, leader: Option<u32>) -> Result<RecordNode> {
    let mut array = child(inputs.layout, GOVERNMENT_PATH, "CHARACTER_POSTS")?;
    for (group, first_id) in posts.groups_of(faction) {
        let holder = if group.position() == FACTION_LEADER_POSITION { leader.unwrap_or_default() } else { 0 };
        let entry = array.push_group()?;
        entry.set_u32("^+1", group.count())?;
        for index in 0..group.count() {
            let mut post = child(inputs.layout, POSTS_PATH, "CHARACTER_POST")?;
            post.group()?
                .set_u32("^+1", first_id + index)?
                .set_str("^+2", group.position())?
                .set_u32("^+3", holder)?
                .set("^+5", NodeType::I64(group.unique_id()))?;
            let key = if index == 0 { "CHARACTER_POST".to_owned() } else { format!("CHARACTER_POST@{}", index + 1) };
            entry.set_record(&key, Some(post.build()?))?;
        }
    }
    array.build()
}
