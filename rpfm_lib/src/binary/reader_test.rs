//---------------------------------------------------------------------------//
// Copyright (c) 2017-2024 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Tests for the [`ReadBytes`] trait.
//!
//! [`ReadBytes`]: crate::binary::ReadBytes

use std::io::Cursor;

use super::ReadBytes;

//---------------------------------------------------------------------------//
//                                  Tests
//---------------------------------------------------------------------------//

/// Test for ReadBytes::len().
#[test]
fn len() {

    // Check the function works.
    assert_eq!(ReadBytes::len(&mut Cursor::new([0, 0, 0, 0])).unwrap(), 4);
}

/// Test to `ReadBytes::read_slice()`.
#[test]
fn read_slice() {

    // Check the reader works with proper slice and size.
    assert_eq!(ReadBytes::read_slice(&mut Cursor::new([1, 2, 3, 4]), 4, false).unwrap(), vec![1, 2, 3, 4]);
    assert_eq!(ReadBytes::read_slice(&mut Cursor::new(vec![0u8; 0]), 0, false).unwrap(), vec![0u8; 0]);

    // Check the reader returns an error for an invalid size value for the data provided.
    assert!(ReadBytes::read_slice(&mut Cursor::new([]), 4, false).is_err());
}

/// Test to `ReadBytes::read_bool()`.
#[test]
fn read_bool() {

    // Check the reader works for a proper value.
    assert!(!ReadBytes::read_bool(&mut Cursor::new([0])).unwrap());
    assert!(ReadBytes::read_bool(&mut Cursor::new([1])).unwrap());

    // Check the reader returns an error for an invalid value.
    assert!(ReadBytes::read_bool(&mut Cursor::new([2])).is_err());
}

/// Test to `ReadBytes::read_u8()`.
#[test]
fn read_u8() {

    // Check the reader works for a proper value.
    assert_eq!(ReadBytes::read_u8(&mut Cursor::new([10])).unwrap(), 10);

    // Check the reader returns an error for empty data.
    assert!(ReadBytes::read_u8(&mut Cursor::new([])).is_err());
}

/// Test to `ReadBytes::read_u16()`.
#[test]
fn read_u16() {

    // Check the reader works for a proper value.
    assert_eq!(ReadBytes::read_u16(&mut Cursor::new([10, 0])).unwrap(), 10);

    // Check the reader returns an error for a slice whose length is smaller than 2.
    assert!(ReadBytes::read_u16(&mut Cursor::new([10])).is_err());
}

/// Test to `ReadBytes::read_u24()`.
#[test]
fn read_u24() {

    // Check the reader works for a proper value.
    assert_eq!(ReadBytes::read_u24(&mut Cursor::new([152, 150, 129])).unwrap(), 8492696);

    // Check the reader returns an error for a slice whose length is smaller than 3.
    assert!(ReadBytes::read_u24(&mut Cursor::new([152, 150])).is_err());
}

/// Test to `ReadBytes::read_u32()`.
#[test]
fn read_u32() {

    // Check the reader works for a proper value.
    assert_eq!(ReadBytes::read_u32(&mut Cursor::new([10, 0, 0, 0])).unwrap(), 10);

    // Check the reader returns an error for a slice whose length is smaller than 4.
    assert!(ReadBytes::read_u32(&mut Cursor::new([10, 0, 0])).is_err());
}

/// Test to `ReadBytes::read_u64()`.
#[test]
fn read_u64() {

    // Check the reader works for a proper value.
    assert_eq!(ReadBytes::read_u64(&mut Cursor::new([10, 0, 0, 0, 0, 0, 0, 0])).unwrap(), 10);

    // Check the reader returns an error for a slice whose length is smaller than 8.
    assert!(ReadBytes::read_u64(&mut Cursor::new([10, 0, 0, 0, 0])).is_err());
}

/// Test to `ReadBytes::read_cauleb128()`.
#[test]
fn read_cauleb128() {

    // Check the reader works for a proper value.
    assert_eq!(ReadBytes::read_cauleb128(&mut Cursor::new([0x80, 10])).unwrap(), 10);

    // Check the reader returns an error for a slice that's not big enough.
    assert!(ReadBytes::read_cauleb128(&mut Cursor::new([])).is_err());
}

/// Test to `ReadBytes::read_i8()`.
#[test]
fn read_i8() {

    // Check the reader works for a proper value.
    assert_eq!(ReadBytes::read_i8(&mut Cursor::new([254])).unwrap(), -2);

    // Check the reader returns an error for a slice whose length is smaller than 1.
    assert!(ReadBytes::read_i8(&mut Cursor::new([])).is_err());
}

/// Test to `ReadBytes::read_u16()`.
#[test]
fn read_i16() {

    // Check the reader works for a proper value.
    assert_eq!(ReadBytes::read_i16(&mut Cursor::new([254, 254])).unwrap(), -258);

    // Check the reader returns an error for a slice whose length is smaller than 2.
    assert!(ReadBytes::read_i16(&mut Cursor::new([10])).is_err());
}

/// Test to `ReadBytes::read_i24()`.
#[test]
fn read_i24() {

    // Check the reader works for a proper value.
    assert_eq!(ReadBytes::read_i24(&mut Cursor::new([152, 150, 129])).unwrap(), -8_284_520);

    // Check the reader returns an error for a slice whose length is smaller than 3.
    assert!(ReadBytes::read_i24(&mut Cursor::new([152, 150])).is_err());
}

/// Test to `ReadBytes::read_u32()`.
#[test]
fn read_i32() {

    // Check the reader works for a proper value.
    assert_eq!(ReadBytes::read_i32(&mut Cursor::new([10, 0, 0, 0])).unwrap(), 10);

    // Check the reader returns an error for a slice whose length is smaller than 4.
    assert!(ReadBytes::read_i32(&mut Cursor::new([10, 0, 0])).is_err());
}

/// Test to `ReadBytes::read_i64()`.
#[test]
fn read_i64() {

    // Check the reader works for a proper value.
    assert_eq!(ReadBytes::read_i64(&mut Cursor::new([10, 0, 0, 0, 0, 0, 0, 0])).unwrap(), 10);

    // Check the reader returns an error for a slice whose length is smaller than 8.
    assert!(ReadBytes::read_i64(&mut Cursor::new([10, 0, 0])).is_err());
}

/// Test to `ReadBytes::read_optional_i16()`.
#[test]
fn read_optional_i16() {

    // Check the reader works for a proper value.
    assert_eq!(ReadBytes::read_optional_i16(&mut Cursor::new([1, 254, 254])).unwrap(), -258);

    // Check the reader returns an error for a slice whose length is smaller than 2.
    assert!(ReadBytes::read_optional_i16(&mut Cursor::new([1, 10])).is_err());

    // Check the reader returns an error if the first value is not a bool.
    assert!(ReadBytes::read_optional_i16(&mut Cursor::new([2, 10, 0])).is_err());
}

/// Test to `ReadBytes::read_optional_i32()`.
#[test]
fn read_optional_i32() {

    // Check the reader works for a proper value.
    assert_eq!(ReadBytes::read_optional_i32(&mut Cursor::new([1, 10, 0, 0, 0])).unwrap(), 10);

    // Check the reader returns an error for a slice whose length is smaller than 4.
    assert!(ReadBytes::read_optional_i32(&mut Cursor::new([1, 10, 0, 0])).is_err());

    // Check the reader returns an error if the first value is not a bool.
    assert!(ReadBytes::read_optional_i32(&mut Cursor::new([2, 10, 0, 0, 0])).is_err());
}

/// Test to `ReadBytes::read_optional_i64()`.
#[test]
fn read_optional_i64() {

    // Check the reader works for a proper value.
    assert_eq!(ReadBytes::read_optional_i64(&mut Cursor::new([1, 10, 0, 0, 0, 0, 0, 0, 0])).unwrap(), 10);

    // Check the reader returns an error for a slice whose length is smaller than 8.
    assert!(ReadBytes::read_optional_i64(&mut Cursor::new([1, 10, 0, 0])).is_err());

    // Check the reader returns an error if the first value is not a bool.
    assert!(ReadBytes::read_optional_i64(&mut Cursor::new([2, 10, 0, 0, 0])).is_err());
}

/// Test to `ReadBytes::read_f32()`.
#[test]
fn read_f32() {

    // Check the reader works for a proper value.
    assert_eq!(ReadBytes::read_f32(&mut Cursor::new([0, 0, 32, 65])).unwrap(), 10.0);

    // Check the reader returns an error for a slice whose length is smaller than 4.
    assert!(ReadBytes::read_f32(&mut Cursor::new([0, 32, 65])).is_err());
}

/// Test to `ReadBytes::read_f64()`.
#[test]
fn read_f64() {

    // Check the reader works for a proper value.
    assert_eq!(ReadBytes::read_f64(&mut Cursor::new([0, 0, 0, 0, 0, 0, 36, 64])).unwrap(), 10.0);

    // Check the reader returns an error for a slice whose length is smaller than 8.
    assert!(ReadBytes::read_f64(&mut Cursor::new([0, 0, 0, 0, 36, 64])).is_err());
}

/// Test to `ReadBytes::read_string_u8()`.
#[test]
fn read_string_u8() {

    // Check the reader works for a proper encoded string.
    assert_eq!(ReadBytes::read_string_u8(&mut Cursor::new([87, 97, 104, 97, 104, 97, 104, 97, 104, 97]), 10).unwrap(), "Wahahahaha");

    // Check the reader returns an error for a slice with non-UTF8 characters (255).
    assert!(ReadBytes::read_string_u8(&mut Cursor::new([87, 97, 104, 97, 255, 104, 97, 104, 97, 104, 97]), 10).is_err());
}

/// Test to `ReadBytes::read_string_u8_iso_8859_15()`.
#[test]
fn read_string_u8_iso_8859_15() {

    // Check the reader works for a proper encoded string.
    assert_eq!(ReadBytes::read_string_u8_iso_8859_15(&mut Cursor::new([87, 97, 104, 97, 104, 97, 104, 97, 104, 97]), 10).unwrap(), "Wahahahaha");

    // Check the reader works mapping characters when an invalid UTF-8 character is detected.
    assert_eq!(ReadBytes::read_string_u8_iso_8859_15(&mut Cursor::new([87, 97, 104, 97, 255, 104, 97, 104, 97, 104, 97]), 11).unwrap(), "Wahaÿhahaha");
}

/// Test to `ReadBytes::read_string_u8_0padded()`.
#[test]
fn read_string_u8_0padded() {

    // Check the reader works for a proper encoded string.
    assert_eq!(ReadBytes::read_string_u8_0padded(&mut Cursor::new([87, 97, 104, 97, 104, 97, 0, 0, 0, 0]), 10).unwrap(), "Wahaha");

    // Check that, as soon as it finds a 0 (null character) the reader stops.
    assert_eq!(ReadBytes::read_string_u8_0padded(&mut Cursor::new([87, 97, 104, 97, 0, 104, 97, 0, 0, 0]), 10).unwrap(), "Waha");

    // Check the reader returns an error for a slice with non-UTF8 characters (255).
    assert!(ReadBytes::read_string_u8_0padded(&mut Cursor::new([87, 97, 104, 97, 255, 104, 97, 104, 97, 104, 97, 0, 0]), 12).is_err());

    // Check the reader returns the full string if no zeros have been found before the end of the slice.
    assert_eq!(ReadBytes::read_string_u8_0padded(&mut Cursor::new([87, 97, 104, 97, 104, 97, 104, 97, 104, 97]), 10).unwrap(), "Wahahahaha");
}

/// Test to `ReadBytes::read_string_u8_0terminated()`.
#[test]
fn read_string_u8_0terminated() {

    // Check the reader works for a proper encoded string.
    assert_eq!(ReadBytes::read_string_u8_0terminated(&mut Cursor::new([87, 97, 104, 97, 104, 97, 104, 97, 0, 97])).unwrap(), "Wahahaha".to_owned());

    // Check the reader works for a string that doesn't end in zero, but in end of slice.
    assert!(ReadBytes::read_string_u8_0terminated(&mut Cursor::new([87, 97, 104, 97, 104, 97, 104, 97, 104, 97])).is_err());

    // Check the reader works for a slice with non-UTF8 characters (255).
    assert!(ReadBytes::read_string_u8_0terminated(&mut Cursor::new([87, 97, 104, 97, 255, 104, 97, 104, 97, 104, 97])).is_err());
}

/// Test to `ReadBytes::read_sized_string_u8()`.
#[test]
fn read_sized_string_u8() {

    // Check the reader works for a proper encoded string.
    assert_eq!(ReadBytes::read_sized_string_u8(&mut Cursor::new([10, 0, 87, 97, 104, 97, 104, 97, 104, 97, 104, 97])).unwrap(), "Wahahahaha".to_owned());

    // Check the reader returns an error for a slice with less than two bytes.
    assert!(ReadBytes::read_sized_string_u8(&mut Cursor::new([5])).is_err());

    // Check the reader returns an error for a slice shorter than it's specified length.
    assert!(ReadBytes::read_sized_string_u8(&mut Cursor::new([4, 0, 2])).is_err());

    // Check the reader returns an error for a slice with non-UTF8 characters (255).
    assert!(ReadBytes::read_sized_string_u8(&mut Cursor::new([11, 0, 87, 97, 104, 97, 255, 104, 97, 104, 97, 104, 97])).is_err());
}

/// Test to `ReadBytes::read_optional_string_u8()`.
#[test]
fn read_optional_string_u8() {

    // Check the reader works for a nonexistent string.
    assert_eq!(ReadBytes::read_optional_string_u8(&mut Cursor::new([0])).unwrap(), "".to_owned());

    // Check the reader works for a proper encoded string.
    assert_eq!(ReadBytes::read_optional_string_u8(&mut Cursor::new([1, 10, 0, 87, 97, 104, 97, 104, 97, 104, 97, 104, 97])).unwrap(), "Wahahahaha".to_owned());

    // Check the reader returns an error for a slice when it expects a string after the bool, but founds nothing.
    assert!(ReadBytes::read_optional_string_u8(&mut Cursor::new([1])).is_err());

    // Check the reader returns an error if the first value is not a boolean
    assert!(ReadBytes::read_optional_string_u8(&mut Cursor::new([2])).is_err());

    // Check the reader returns an error for a slice with less than two bytes.
    assert!(ReadBytes::read_optional_string_u8(&mut Cursor::new([1, 5])).is_err());

    // Check the reader returns an error for a slice shorter than it's specified length.
    assert!(ReadBytes::read_optional_string_u8(&mut Cursor::new([1, 4, 0, 2])).is_err());

    // Check the reader returns an error for a slice with non-UTF8 characters (255).
    assert!(ReadBytes::read_optional_string_u8(&mut Cursor::new([1, 11, 0, 87, 97, 104, 97, 255, 104, 97, 104, 97, 104, 97])).is_err());
}

/// Test to `ReadBytes::read_string_u16()`.
#[test]
fn read_string_u16() {

    // Check the reader works for a proper encoded string.
    assert_eq!(ReadBytes::read_string_u16(&mut Cursor::new([87, 0, 97, 0, 104, 0, 97, 0, 104, 0, 97, 0]), 12).unwrap(), "Wahaha");

    // Check the reader returns an error for a slice with uneven amount of bytes.
    assert!(ReadBytes::read_string_u16(&mut Cursor::new([87, 0, 0, 216, 104, 0, 97, 0, 104, 0, 97, 0, 1]), 13).is_err());

    // Check the reader returns an error for a slice with the wrong size.
    assert!(ReadBytes::read_string_u16(&mut Cursor::new([87, 0, 0, 216, 104, 0, 97, 0, 104, 0, 97, 0]), 14).is_err());
}

/// Test to `ReadBytes::read_string_u16_0padded()`.
#[test]
fn read_string_u16_0padded() {

    // Check the reader works for a proper encoded string.
    assert_eq!(ReadBytes::read_string_u16_0padded(&mut Cursor::new([87, 0, 97, 0, 104, 0, 97, 0, 104, 0, 97, 0, 0, 0, 0, 0, 0, 0, 0, 0]), 20).unwrap(), "Wahaha");

    // Check that, as soon as it finds a 0 (null character) the reader stops.
    assert_eq!(ReadBytes::read_string_u16_0padded(&mut Cursor::new([87, 0, 97, 0, 104, 0, 97, 0, 0, 0, 104, 0, 97, 0, 0, 0, 0, 0, 0, 0]), 20).unwrap(), "Waha");

    // Check the reader returns the full string if no zeros have been found before the end of the slice.
    assert_eq!(ReadBytes::read_string_u16_0padded(&mut Cursor::new([87, 0, 97, 0, 104, 0, 97, 0, 104, 0, 97, 0, 104, 0, 97, 0, 104, 0, 97, 0]), 20).unwrap(), "Wahahahaha");

    // Check that fails properly if the size is wrong
    assert!(ReadBytes::read_string_u16_0padded(&mut Cursor::new([87, 0, 97, 0, 104, 0, 97, 0, 104, 0, 97, 0, 0, 0, 0, 0, 0, 0, 0, 0]), 40).is_err());
}

/// Test to `ReadBytes::read_string_u16_0terminated()`.
#[test]
fn read_string_u16_0terminated() {

    // Check the reader works for a proper encoded string.
    assert_eq!(ReadBytes::read_string_u16_0terminated(&mut Cursor::new([87, 0, 97, 0, 104, 0, 97, 0, 104, 0, 97, 0, 104, 0, 97, 0, 0, 0])).unwrap(), "Wahahaha".to_owned());

    // Check the reader works for a string that doesn't end in zero, but in end of slice.
    assert!(ReadBytes::read_string_u16_0terminated(&mut Cursor::new([87, 0, 97, 0, 104, 0, 97, 0, 104, 0, 97, 0, 104, 0, 97, 0, 104, 0, 97, 0])).is_err());
}

/// Test to `ReadBytes::read_sized_string_u16()`.
#[test]
fn read_sized_string_u16() {

    // Check the reader works for a proper encoded string.
    assert_eq!(ReadBytes::read_sized_string_u16(&mut Cursor::new([4, 0, 87, 0, 97, 0, 104, 0, 97, 0])).unwrap(), "Waha".to_owned());

    // Check the reader returns an error for a slice with less than two bytes.
    assert!(ReadBytes::read_sized_string_u16(&mut Cursor::new([5])).is_err());

    // Check the reader returns an error for a slice shorter than it's specified length.
    assert!(ReadBytes::read_sized_string_u16(&mut Cursor::new([4, 0, 2])).is_err());
}

/// Test to `ReadBytes::read_optional_string_u16()`.
#[test]
fn read_optional_string_u16() {

    // Check the reader works for a nonexistent string.
    assert_eq!(ReadBytes::read_optional_string_u16(&mut Cursor::new([0])).unwrap(), "".to_owned());

    // Check the reader works for a proper encoded string.
    assert_eq!(ReadBytes::read_optional_string_u16(&mut Cursor::new([1, 4, 0, 87, 0, 97, 0, 104, 0, 97, 0])).unwrap(), "Waha".to_owned());

    // Check the reader returns an error if the first value is not a boolean
    assert!(ReadBytes::read_optional_string_u16(&mut Cursor::new([2, 5])).is_err());

    // Check the reader returns an error for a slice with less than two bytes.
    assert!(ReadBytes::read_optional_string_u16(&mut Cursor::new([1, 5])).is_err());

    // Check the reader returns an error for a slice shorter than it's specified length.
    assert!(ReadBytes::read_optional_string_u16(&mut Cursor::new([1, 4, 0, 2])).is_err());
}

/// Test to `ReadBytes::read_string_colour_rgb()`.
#[test]
fn read_string_colour_rgb() {

    // Check the reader works for a proper encoded string.
    assert_eq!(ReadBytes::read_string_colour_rgb(&mut Cursor::new([0xFF, 0x04, 0x05, 0x00])).unwrap(), "0504FF");

    // Check the reader returns an error for a slice shorter than expected.
    assert!(ReadBytes::read_string_colour_rgb(&mut Cursor::new([0x87, 0x97])).is_err());
}
