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
use crate::startpos::esf_nav::child_record;
use crate::startpos::startpos_test::{test_data_tables, test_startpos, world};
use crate::startpos::world::writer::writer_test::{game_plan, world_test_data};

#[test]
#[ignore = "needs the game data, see startpos_test.rs"]
fn test_slaves_match_the_game_startpos() {
    let (index, layout, settlements, region_map, legacy_overrides) = world_test_data();
    let inputs = WorldInputs { tables: test_data_tables(), index: &index, layout: &layout, settlements: &settlements, region_map: &region_map, legacy_overrides: &legacy_overrides };
    let plan = game_plan(&index);

    let written = write_slaves(&inputs, &plan).unwrap();
    assert_eq!(&written, child_record(world(test_startpos()), "SLAVES").unwrap());
}
