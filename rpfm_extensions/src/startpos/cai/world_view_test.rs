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
use crate::startpos::startpos_test::{test_startpos, world};

#[test]
#[ignore = "needs the game data, see startpos_test.rs"]
fn test_world_view_counts_the_game_objects() {
    let layout = StartposLayout::from_esf(test_startpos()).unwrap();

    let view = WorldView::new(&layout, world(test_startpos())).unwrap();
    let factions = view.factions();
    let forces = factions.iter().flat_map(|faction| faction.forces()).collect::<Vec<_>>();

    assert_eq!(factions.iter().map(|faction| faction.posts().len()).sum::<usize>(), 1434);
    assert_eq!(forces.len(), 343);
    assert_eq!(forces.iter().map(|force| force.units().len()).sum::<usize>(), 1949);
    assert_eq!(forces.iter().map(|force| force.horde_slots().len()).sum::<usize>(), 297);
    assert_eq!(view.regions().len(), 186);
    assert_eq!(view.regions().iter().map(|region| region.slots().len()).sum::<usize>(), 864);
    assert_eq!(view.provinces().iter().map(|province| province.managers().len()).sum::<usize>(), 94);
    assert_eq!(view.seas().iter().map(|sea| sea.managers().len()).sum::<usize>(), 38);
}
