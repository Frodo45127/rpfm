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

use crate::startpos::cai::cai_test::{compare_list, game_cai_world_list, test_inputs};
use crate::startpos::cai::factions::victory_factions;
use crate::startpos::cai::map::AreaGroups;
use crate::startpos::cai::mobiles::{mobile_cells, MobileInputs};
use crate::startpos::cai::settlements::{port_cells, settlement_components};
use crate::startpos::startpos_test::{test_data_esf, test_data_tables, TEST_CAMPAIGN};
use crate::startpos::world::CampaignIndex;
use crate::startpos::startpos_test::{compare, Report};

/// HLCI fields the writer gets wrong: the position of Hafn's main HLCI (the game picks the node of its second largest
/// area; the rule isn't known).
const KNOWN_HLCI_MISMATCHES: usize = 2;

/// Share of the regions' fields the writer must match. The rest are the beliefs the components refer to (left empty,
/// the game rebuilds them) and Hafn's position.
const MIN_REGION_MATCH_RATIO: f64 = 0.96;

#[test]
#[ignore = "needs the game data, see startpos_test.rs"]
fn test_hlcis_match_the_game_startpos() {
    let inputs = test_inputs();
    let layout = &inputs.layout;

    let written = write_hlcis(layout, &inputs.plan, &inputs.map_regions, &inputs.nodes).unwrap();
    let mut report = Report::default();
    compare(layout, &written, game_cai_world_list("CAI_WORLD_REGION_HLCIS"), &child_path(CAI_WORLD_PATH, "CAI_WORLD_REGION_HLCIS"), &mut report);

    let (matches, total) = report.totals();
    assert_eq!(matches + KNOWN_HLCI_MISMATCHES, total, "{matches}/{total} fields match. Worst:\n{}", report.worst(20));
}

#[test]
#[ignore = "needs the game data, see startpos_test.rs"]
fn test_regions_match_the_game_startpos() {
    let inputs = test_inputs();
    let map_data = test_data_esf("map_data.esf");
    let region_map = RegionMap::from_map_data(&map_data).unwrap();
    let groups = AreaGroups::new(&region_map, &inputs.map_regions);
    let settlements = settlement_components(&inputs.world, &inputs.plan, &inputs.map_regions);
    let ports = port_cells(&inputs.world, &inputs.map_regions, &groups);
    let mobile_inputs = MobileInputs { layout: &inputs.layout, plan: &inputs.plan, world: &inputs.world, region_map: &region_map, groups: &groups, settlements: &settlements, ports: &ports };
    let mobiles = mobile_cells(&mobile_inputs);
    let index = CampaignIndex::new(test_data_tables(), TEST_CAMPAIGN).unwrap();
    let victory = victory_factions(test_data_tables(), &index, &inputs.world, &inputs.plan).unwrap();
    let links = region_links(&LinkInputs { plan: &inputs.plan, world: &inputs.world, map_regions: &inputs.map_regions, region_map: &region_map, mobiles: &mobiles, ports: &ports, victory_factions: &victory });

    let written = write_regions(&inputs.layout, &inputs.plan, &inputs.map_regions, &inputs.nodes, &links).unwrap();
    compare_list(&inputs, &written, "CAI_WORLD_REGIONS", MIN_REGION_MATCH_RATIO);
}
