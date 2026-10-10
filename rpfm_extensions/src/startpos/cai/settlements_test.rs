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
use crate::startpos::cai::map::AreaGroups;
use crate::startpos::cai::regions::region_positions;
use crate::startpos::region_map::RegionMap;
use crate::startpos::cai::factions::capitals;
use crate::startpos::startpos_test::{test_data_esf, test_data_tables, TEST_CAMPAIGN};
use crate::startpos::world::CampaignIndex;

/// Share of the settlements' fields the writer must match. The rest are the cells of the ports (see
/// [`AreaGroups::port_cell`]), the manager objects some settlements refer to, the beliefs they refer to (left empty,
/// the game rebuilds them) and Hafn's position (see the regions' tests).
const MIN_SETTLEMENT_MATCH_RATIO: f64 = 0.98;

#[test]
#[ignore = "needs the game data, see startpos_test.rs"]
fn test_settlements_and_slots_match_the_game_startpos() {
    let inputs = test_inputs();
    let map_data = test_data_esf("map_data.esf");
    let region_map = RegionMap::from_map_data(&map_data).unwrap();
    let groups = AreaGroups::new(&region_map, &inputs.map_regions);
    let layouts = SettlementLayout::all_from_map_data(&map_data, |_| false).unwrap();
    let settlements = settlement_components(&inputs.world, &inputs.plan, &inputs.map_regions);
    let ports = port_cells(&inputs.world, &inputs.map_regions, &groups);
    let mobile_inputs = MobileInputs { layout: &inputs.layout, plan: &inputs.plan, world: &inputs.world, region_map: &region_map, groups: &groups, settlements: &settlements, ports: &ports };
    let positions = region_positions(&inputs.map_regions, &inputs.nodes);
    let index = CampaignIndex::new(test_data_tables(), TEST_CAMPAIGN).unwrap();
    let capitals = capitals(&index, &inputs.world, &inputs.plan, &inputs.map_regions);

    compare_list(&inputs, &write_settlements(&mobile_inputs, &inputs.map_regions, &positions, &capitals).unwrap(), "CAI_WORLD_SETTLEMENTS", MIN_SETTLEMENT_MATCH_RATIO);
    compare_list(&inputs, &write_building_slots(&inputs.layout, &inputs.plan, &inputs.world, &inputs.map_regions, &layouts).unwrap(), "CAI_WORLD_BUILDING_SLOTS", 1.0);
}
