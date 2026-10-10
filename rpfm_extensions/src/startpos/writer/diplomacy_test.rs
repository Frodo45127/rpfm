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

#[test]
fn test_faction_pairs_follow_the_game_order() {
    let pairs = faction_pairs(vec![4, 2, 3]);

    assert_eq!(pairs, vec![(1, 2), (2, 3), (2, 4), (1, 3), (3, 4), (1, 4)]);
}
