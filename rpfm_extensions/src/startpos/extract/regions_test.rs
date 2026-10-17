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
fn test_percentages_undo_the_single_precision_of_the_shares() {
    assert_eq!(percentage(0.79999995), "80");
    assert_eq!(percentage(0.19999999), "20");
    assert_eq!(percentage(0.45455), "45.455");
}
