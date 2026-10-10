//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Writer of the diplomacy manager (`DIPLOMACY_MANAGER`): one relationship per pair of factions, rebels included. The
//! relationships themselves live in each faction's `OLD_DIPLOMACY_MANAGER`.

use rpfm_lib::error::Result;
use rpfm_lib::files::esf::RecordNode;

use crate::startpos::layout::StartposLayout;
use crate::startpos::world::writer::{WorldPlan, REBEL_FACTION_ID};

#[cfg(test)] #[path = "diplomacy_test.rs"] mod diplomacy_test;

/// Path of the diplomacy manager.
const MANAGER_PATH: &str = "CAMPAIGN_STARTPOS/CAMPAIGN_ENV/CAMPAIGN_MODEL/DIPLOMACY_MANAGER";

/// Path of the faction pairs.
const RELATIONSHIPS_PATH: &str = "CAMPAIGN_STARTPOS/CAMPAIGN_ENV/CAMPAIGN_MODEL/DIPLOMACY_MANAGER/DIPLOMACY_RELATIONSHIP";

//---------------------------------------------------------------------------//
//                              Implementations
//---------------------------------------------------------------------------//

/// Writes the diplomacy manager of a campaign.
///
/// # Errors
///
/// Returns an error if the layout lacks the records.
pub fn write_diplomacy_manager(layout: &StartposLayout, plan: &WorldPlan) -> Result<RecordNode> {
    let ids = plan.factions().iter().filter_map(|faction| plan.faction_id(faction.key())).collect::<Vec<_>>();
    let mut relationships = layout.builder(RELATIONSHIPS_PATH)?;
    for (first, second) in faction_pairs(ids) {
        relationships.push_group()?
            .set_u32("^+1", first)?
            .set_u32("^+2", second)?;
    }

    let mut manager = layout.builder(MANAGER_PATH)?;
    manager.group()?.set_record("DIPLOMACY_RELATIONSHIP", Some(relationships.build()?))?;
    manager.build()
}

/// Returns every pair of faction ids, rebels included, in the game's order: for each faction by id, its pair with the
/// rebels, then its pairs with the factions after it.
fn faction_pairs(mut ids: Vec<u32>) -> Vec<(u32, u32)> {
    ids.sort_unstable();
    let mut pairs = Vec::with_capacity(ids.len() * (ids.len() + 1) / 2);
    for (index, &id) in ids.iter().enumerate() {
        pairs.push((REBEL_FACTION_ID, id));
        pairs.extend(ids[index + 1..].iter().map(|&other| (id, other)));
    }
    pairs
}
