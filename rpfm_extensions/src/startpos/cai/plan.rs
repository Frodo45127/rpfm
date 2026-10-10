//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Component ids of every AI object, in the order the game creates them.
//!
//! The game counts from [`ROOT_ID`]: the interface, the two components of its world mirror, the central pool's
//! desires, the theatre, then each map region followed by its HLCIs, and the rest of the world mirror. Beliefs the game
//! creates in between aren't written (the game rebuilds them), so their ids are skipped.

use getset::{CopyGetters, Getters};

use super::map::MapRegion;

/// Component id of the AI interface itself.
pub const ROOT_ID: u32 = 2;

/// Components of the AI's world mirror (its own and its time's).
const WORLD_COMPONENTS: u32 = 2;

/// Desires of the central pool: its analysers and log manager.
pub const CENTRAL_DESIRES: u32 = 29;

//---------------------------------------------------------------------------//
//                              Types
//---------------------------------------------------------------------------//

/// Component ids of the AI objects.
#[derive(Debug, Clone, Default, Getters, CopyGetters)]
pub struct CaiPlan {

    /// Next free id.
    #[getset(get_copy = "pub")]
    next_id: u32,

    /// Id of the first central desire.
    #[getset(get_copy = "pub")]
    first_central_desire: u32,

    /// Id of the theatre (the whole map).
    #[getset(get_copy = "pub")]
    theatre: u32,

    /// Ids of each map region, in map order.
    #[getset(get = "pub")]
    regions: Vec<RegionIds>,
}

/// Ids of a map region and its HLCIs.
#[derive(Debug, Clone, Default, Getters, CopyGetters)]
pub struct RegionIds {

    /// The region's id.
    #[getset(get_copy = "pub")]
    id: u32,

    /// Its HLCIs' ids, in the region's HLCI order.
    #[getset(get = "pub")]
    hlcis: Vec<u32>,
}

//---------------------------------------------------------------------------//
//                              Implementations
//---------------------------------------------------------------------------//

impl CaiPlan {

    /// Plans the ids of a campaign's AI objects.
    ///
    /// # Arguments
    ///
    /// * `map_regions` - Regions of the map, in map order.
    pub fn new(map_regions: &[MapRegion]) -> Self {
        let mut plan = Self { next_id: ROOT_ID + 1 + WORLD_COMPONENTS, ..Default::default() };
        plan.first_central_desire = plan.take(CENTRAL_DESIRES);
        plan.theatre = plan.take(1);
        plan.regions = map_regions.iter().map(|region| {
            let id = plan.take(1);
            let hlcis = (0..region.hlcis().len()).map(|_| plan.take(1)).collect();
            RegionIds { id, hlcis }
        }).collect();
        plan
    }

    /// Takes the next ids.
    ///
    /// # Returns
    ///
    /// The first of them.
    fn take(&mut self, count: u32) -> u32 {
        let first = self.next_id;
        self.next_id += count;
        first
    }
}
