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
fn test_string_hash() {
    assert_eq!(string_hash_u8(""), STRING_HASH_SEED);

    // The last character is xored unshifted, the one before it shifted by 8 bits.
    assert_eq!(string_hash_u8("ab"), STRING_HASH_SEED ^ 0x62 ^ (0x61 << 8));
    assert_eq!(string_hash_u16("ab"), string_hash_u8("ab"));
}

#[test]
fn test_hash_map_order_without_growth() {
    assert_eq!(hash_map_order(&[9, 0]), vec![1, 0]);
}

#[test]
fn test_hash_map_order_after_growth() {

    // The fifth entry grows the map from 4 to 9 buckets, moving entries bucket by bucket: [1, 2] from bucket 0, then
    // [0, 3] from bucket 1, into buckets 0 (hashes 0 and 9) and 4 (hashes 4 and 13).
    assert_eq!(hash_map_order(&[9, 0, 4, 13, 18]), vec![1, 0, 4, 2, 3]);
}
