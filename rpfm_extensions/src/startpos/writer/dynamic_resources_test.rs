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
use crate::startpos::startpos_test::{test_data_esf, test_data_tables, test_startpos};
use crate::startpos::world::writer::writer_test::{game_plan, world_test_data};

#[test]
#[ignore = "needs the game data, see startpos_test.rs"]
fn test_dynamic_resources_match_the_game_startpos() {
    let (index, layout, _, _, _) = world_test_data();
    let plan = game_plan(&index);
    let (map_data, dynamic_resources) = (test_data_esf("map_data.esf"), test_data_esf("dynamic_resources.esf"));
    let inputs = DynamicResourcesInputs { layout: &layout, tables: test_data_tables(), index: &index, plan: &plan, map_data: &map_data, dynamic_resources: &dynamic_resources };

    let written = write_dynamic_resources_manager(&inputs).unwrap();
    let root = as_record(test_startpos().root_node()).unwrap();
    let model = child_record(child_record(root, "CAMPAIGN_ENV").unwrap(), "CAMPAIGN_MODEL").unwrap();
    assert_eq!(&written, child_record(model, "DYNAMIC_RESOURCES_MANAGER").unwrap());
}
