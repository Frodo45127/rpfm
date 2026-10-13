//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Module containing tests for the helpers of RigidModel materials.

use std::io::Cursor;

use super::*;

/// Reads a padded string with its tail and writes it back, checking both.
fn round_trip_with_tail(bytes: &[u8], utf16: bool, expected_string: &str, expected_tail: &[u8]) {
    let (string, tail) = read_string_0padded_with_tail(&mut Cursor::new(bytes), bytes.len(), utf16).unwrap();
    assert_eq!(string, expected_string);
    assert_eq!(tail, expected_tail);

    let mut written = vec![];
    write_string_0padded_with_tail(&mut written, &string, &tail, bytes.len(), utf16).unwrap();
    assert_eq!(written, bytes);
}

#[test]
fn test_string_0padded_with_tail_utf8() {
    round_trip_with_tail(b"path\0\0\0\0", false, "path", b"");
    round_trip_with_tail(b"path\0ab\0", false, "path", b"ab");
    round_trip_with_tail(b"path\0\0\xcc\0", false, "path", b"\0\xcc");
    round_trip_with_tail(b"fullpath", false, "fullpath", b"");
}

#[test]
fn test_string_0padded_with_tail_utf16() {
    round_trip_with_tail(b"b\0n\0\0\0\0\0", true, "bn", b"");
    round_trip_with_tail(b"b\0n\0\0\0\xcc\xcc", true, "bn", b"\xcc\xcc");
    round_trip_with_tail(b"b\0n\0_\0x\0", true, "bn_x", b"");
}

#[test]
fn test_string_0padded_with_tail_too_long() {
    assert!(write_string_0padded_with_tail(&mut vec![], "path", b"abcd", 8, false).is_err());
}

#[test]
fn test_padded_string_by_version() {
    let mut v5 = vec![];
    write_padded_string(&mut v5, 5, "ab", 4).unwrap();
    assert_eq!(v5, b"a\0b\0\0\0\0\0");
    assert_eq!(read_padded_string(&mut Cursor::new(&v5), 5, 4).unwrap(), "ab");

    let mut v6 = vec![];
    write_padded_string(&mut v6, 6, "ab", 4).unwrap();
    assert_eq!(v6, b"ab\0\0");
    assert_eq!(read_padded_string(&mut Cursor::new(&v6), 6, 4).unwrap(), "ab");
}
