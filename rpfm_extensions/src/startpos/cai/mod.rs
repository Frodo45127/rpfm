//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Writer of the campaign AI's state (`CAI_INTERFACE`).
//!
//! Every AI object (a mirror of a world object, a map region, an analyser...) has a component with an id from a single
//! counter, in the order the game creates them, and AI records refer to each other by those ids. The writer first
//! plans every id ([`plan::CaiPlan`]), then writes each list with its references.

use rpfm_lib::error::Result;
use rpfm_lib::files::esf::RecordNode;

use super::layout::{child_path, StartposLayout};

pub mod factions;
pub mod lists;
pub mod map;
pub mod mobiles;
pub mod plan;
pub mod regions;
pub mod settlements;
pub mod technologies;
pub mod world;
pub mod world_view;

#[cfg(test)] mod cai_test;

/// Path of the AI state.
pub const CAI_PATH: &str = "CAMPAIGN_STARTPOS/CAMPAIGN_ENV/CAMPAIGN_MODEL/CAI_INTERFACE";

/// Path of the AI's mirror of the world.
pub const CAI_WORLD_PATH: &str = "CAMPAIGN_STARTPOS/CAMPAIGN_ENV/CAMPAIGN_MODEL/CAI_INTERFACE/CAI_WORLD";

//---------------------------------------------------------------------------//
//                              Implementations
//---------------------------------------------------------------------------//

/// Writes the component (`CAI_BDI_COMPONENT`) of an AI object.
///
/// # Arguments
///
/// * `layout` - Record layouts.
/// * `parent` - Path of the record holding the component.
/// * `id` - Component id.
/// * `references` - Ids of the objects the component refers to (beliefs about it).
///
/// # Errors
///
/// Returns an error if the layout lacks the component.
pub(crate) fn component(layout: &StartposLayout, parent: &str, id: u32, references: Vec<u32>) -> Result<RecordNode> {
    let mut record = layout.builder(&child_path(parent, "CAI_BDI_COMPONENT"))?;
    record.group()?
        .set_u32("^+1", id)?
        .set_u32s("CAI_BDI_COMPONENT_BLOCK_OWNS+1", references)?;
    record.build()
}
