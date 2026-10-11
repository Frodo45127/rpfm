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
use crate::startpos::cai::cai_test::{compare_list, game_trade_routes, test_inputs};
use crate::startpos::startpos_test::{test_data_tables, TEST_CAMPAIGN};

/// Share of the international trade routes' fields the writer must match. The rest are the end settlements and trade
/// segments of the routes the game has already traced (20 of 42), left empty like the game's other ones.
const MIN_TRADE_ROUTE_MATCH_RATIO: f64 = 0.94;

#[test]
#[ignore = "needs the game data, see startpos_test.rs"]
fn test_small_lists_match_the_game_startpos() {
    let inputs = test_inputs();
    let (layout, plan, world, map_regions) = (&inputs.layout, &inputs.plan, &inputs.world, &inputs.map_regions);
    let index = CampaignIndex::new(test_data_tables(), TEST_CAMPAIGN).unwrap();
    let (provinces, parts) = write_provinces(layout, plan, world, map_regions, &index).unwrap();

    compare_list(&inputs, &write_trade_segments(layout, plan, 2486).unwrap(), "CAI_WORLD_TRADE_ROUTE_SEGMENTS", 1.0);
    compare_list(&inputs, &write_trade_routes(layout, plan, world, &game_trade_routes(layout)).unwrap(), "CAI_WORLD_TRADE_ROUTES", MIN_TRADE_ROUTE_MATCH_RATIO);
    compare_list(&inputs, &write_boundaries(layout, plan).unwrap(), "CAI_WORLD_REGION_BOUNDARIES", 1.0);
    compare_list(&inputs, &provinces, "CAI_PROVINCES", 1.0);
    compare_list(&inputs, &parts, "CAI_FACTION_PROVINCES", 1.0);
    compare_list(&inputs, &write_faction_seas(layout, plan, world, map_regions).unwrap(), "CAI_FACTION_SEA_REGIONS", 1.0);
    compare_list(&inputs, &write_governorships(layout, plan).unwrap(), "CAI_WORLD_GOVERNORSHIPS", 1.0);
    compare_list(&inputs, &write_posts(layout, plan, world).unwrap(), "CAI_CHARACTER_POSTS", 1.0);
}
