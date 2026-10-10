//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

use super::*;
use crate::startpos::cai::cai_test::{compare_list, test_inputs};
use crate::startpos::startpos_test::{test_data_tables, TEST_CAMPAIGN};

/// Share of the factions' fields the writer must match. The rest are the beliefs the components refer to (left empty,
/// the game rebuilds them), the manager objects some factions refer to, the order of some victory regions and the
/// sea groups of the two empires.
const MIN_FACTION_MATCH_RATIO: f64 = 0.96;

#[test]
#[ignore = "needs the game data, see startpos_test.rs"]
fn test_factions_match_the_game_startpos() {
    let inputs = test_inputs();
    let index = CampaignIndex::new(test_data_tables(), TEST_CAMPAIGN).unwrap();
    let links = faction_links(test_data_tables(), &index, &inputs.world, &inputs.plan, &inputs.map_regions).unwrap();

    let written = write_factions(&inputs.layout, &inputs.plan, &links).unwrap();
    compare_list(&inputs, &written, "CAI_WORLD_FACTIONS", MIN_FACTION_MATCH_RATIO);
}

#[test]
#[ignore = "needs the game data, see startpos_test.rs"]
fn test_technologies_and_trees_match_the_game_startpos() {
    let inputs = test_inputs();

    compare_list(&inputs, &write_technology_trees(&inputs.layout, &inputs.plan).unwrap(), "CAI_WORLD_TECHNOLOGY_TREES", 1.0);
    compare_list(&inputs, &write_technologies(&inputs.layout, &inputs.plan, &inputs.technologies).unwrap(), "CAI_WORLD_TECHNOLOGIES", 1.0);
}
