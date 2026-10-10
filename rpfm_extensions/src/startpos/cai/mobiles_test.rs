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
use crate::startpos::cai::settlements::{port_cells, settlement_components};
use crate::startpos::startpos_test::test_data_esf;

/// Share of the mobiles' fields the writer must match. The rest are the beliefs the components refer to (left empty,
/// the game rebuilds them) and the cells of garrison fleets, which wait at their port's cell (see
/// [`AreaGroups::port_cell`]).
const MIN_MOBILE_MATCH_RATIO: f64 = 0.95;

#[test]
#[ignore = "needs the game data, see startpos_test.rs"]
fn test_characters_mobiles_and_units_match_the_game_startpos() {
    let inputs = test_inputs();
    let region_map = RegionMap::from_map_data(&test_data_esf("map_data.esf")).unwrap();
    let groups = AreaGroups::new(&region_map, &inputs.map_regions);
    let settlements = settlement_components(&inputs.world, &inputs.plan, &inputs.map_regions);
    let ports = port_cells(&inputs.world, &inputs.map_regions, &groups);
    let mobile_inputs = MobileInputs { layout: &inputs.layout, plan: &inputs.plan, world: &inputs.world, region_map: &region_map, groups: &groups, settlements: &settlements, ports: &ports };

    compare_list(&inputs, &write_characters(&mobile_inputs).unwrap(), "CAI_WORLD_CHARACTERS", 1.0);
    compare_list(&inputs, &write_mobiles(&mobile_inputs).unwrap(), "CAI_WORLD_RESOURCE_MOBILES", MIN_MOBILE_MATCH_RATIO);
    compare_list(&inputs, &write_units(&mobile_inputs).unwrap(), "CAI_WORLD_UNITS", 1.0);
}
