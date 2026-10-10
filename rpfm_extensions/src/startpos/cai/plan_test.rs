//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

use crate::startpos::cai::cai_test::test_inputs;

#[test]
#[ignore = "needs the game data, see startpos_test.rs"]
fn test_plan_matches_the_game_ids() {
    let plan = test_inputs().plan;

    assert_eq!(plan.theatre(), 34);
    assert_eq!(plan.regions()[0].id(), 35);
    assert_eq!(plan.factions()[0].faction(), 1976);
    assert_eq!(plan.factions().len(), 119);
    assert_eq!(plan.factions()[118].faction(), 8014);
    assert_eq!(plan.settlements()[&0].slots().first(), Some(&8016));
    assert_eq!(plan.settlements()[&0].settlement(), 8022);
    assert_eq!(plan.first_trade_segment(), 9066);
    assert_eq!((plan.boundaries().len(), plan.boundaries()[0].id()), (545, 11552));
    let first = &plan.characters()[0][0];
    assert_eq!((first.character(), first.mobile(), first.units().first()), (12097, Some(12098), Some(&12099)));
}
