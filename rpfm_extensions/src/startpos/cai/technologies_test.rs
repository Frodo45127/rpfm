//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

use std::collections::HashMap;

use super::*;
use crate::startpos::cai::cai_test::{game_cai_world_list, single_field_u32};
use crate::startpos::cai::world_view::WorldView;
use crate::startpos::cai::CAI_WORLD_PATH;
use crate::startpos::layout::{child_path, StartposLayout};
use crate::startpos::startpos_test::{test_data_tables, test_startpos, world, TEST_CAMPAIGN};
use crate::startpos::world::CampaignIndex;

#[test]
#[ignore = "needs the game data, see startpos_test.rs"]
fn test_technologies_match_the_game_startpos() {
    let tables = test_data_tables();
    let index = CampaignIndex::new(tables, TEST_CAMPAIGN).unwrap();
    let layout = StartposLayout::from_esf(test_startpos()).unwrap();
    let view = WorldView::new(&layout, world(test_startpos())).unwrap();

    let technologies_path = child_path(CAI_WORLD_PATH, "CAI_WORLD_TECHNOLOGIES");
    let mut game: HashMap<u32, Vec<String>> = HashMap::new();
    for group in layout.read(&technologies_path, game_cai_world_list("CAI_WORLD_TECHNOLOGIES")).unwrap() {
        let owner = single_field_u32(&layout, &technologies_path, &group, "OWNED_DIRECT");
        let technology = layout.read(&child_path(&technologies_path, "CAI_TECHNOLOGY"), group.record("CAI_TECHNOLOGY").unwrap()).unwrap();
        game.entry(owner).or_default().push(technology[0].str("^+2").unwrap().to_owned());
    }
    let factions_path = child_path(CAI_WORLD_PATH, "CAI_WORLD_FACTIONS");
    let owners = layout.read(&factions_path, game_cai_world_list("CAI_WORLD_FACTIONS")).unwrap().iter()
        .map(|group| single_field_u32(&layout, &factions_path, group, "OWNED_DIRECT"))
        .collect::<Vec<_>>();

    let mismatches = view.factions().iter().zip(&owners)
        .filter(|(faction, owner)| game.get(owner).cloned().unwrap_or_default() != faction_technologies(tables, TEST_CAMPAIGN, index.faction(faction.key()).unwrap()).unwrap())
        .map(|(faction, _)| faction.key().to_owned())
        .collect::<Vec<_>>();
    assert!(mismatches.is_empty(), "mismatches: {mismatches:?}");
}
