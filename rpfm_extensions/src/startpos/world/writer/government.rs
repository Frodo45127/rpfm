//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Plan of the government posts of the factions (`FACTION/GOVERNMENT/CHARACTER_POSTS`).
//!
//! Post ids are one counter over the factions, in startpos order. The game lists each faction's post groups in a
//! hash order; this plan uses table order.

use std::collections::HashMap;

use rpfm_lib::error::Result;

use super::{WorldInputs, WorldPlan};
use crate::startpos::world::government::{government_posts, PostGroup, FACTION_LEADER_POSITION};

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
