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
fn test_religion_shares_are_normalised() {
    assert_eq!(religion_shares(&[30.0, 30.0, 0.0]), vec![0.5, 0.5, 0.0]);
}

#[test]
fn test_regions_without_percentages_share_every_religion() {
    assert_eq!(religion_shares(&[0.0; 28]), vec![1.0 / 28.0; 28]);
    assert_eq!(religion_shares(&[0.0; 28])[0], 0.035714287);
}
