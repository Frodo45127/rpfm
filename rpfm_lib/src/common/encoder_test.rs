//---------------------------------------------------------------------------//
// Copyright (c) 2017-2020 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

/*!
Module containing test for the entire `Encoder` implementation for `Vec<u8>`.
!*/

use crate::common::encoder::Encoder;

//---------------------------------------------------------------------------//
//                          Normal Encoders
//---------------------------------------------------------------------------//

/// Test to make sure the boolean encoder (`encode_bool()`) works properly.
#[test]
fn test_encode_bool() {

    // Check the encoder works for a boolean.
    let mut data = vec![];
    data.encode_bool(true);
    assert_eq!(data, vec![1]);

    let mut data = vec![];
    data.encode_bool(false);
    assert_eq!(data, vec![0]);
}

/// Test to make sure the u16 integer encoder (`encode_integer_u16()`) works properly.
#[test]
fn test_encode_integer_u16() {

    // Check the encoder works properly.
    let mut data = vec![];
    data.encode_integer_u16(258);
    assert_eq!(data, vec![2, 1]);
}

/// Test to make sure the u32 integer encoder (`encode_integer_u32()`) works properly.
#[test]
fn test_encode_integer_u32() {

    // Check the encoder works properly.
    let mut data = vec![];
    data.encode_integer_u32(258);
    assert_eq!(data, vec![2, 1, 0, 0]);
}

/// Test to make sure the u64 integer encoder (`encode_integer_u64()`) works properly.
#[test]
fn test_encode_integer_u64() {

    // Check the encoder works properly.
    let mut data = vec![];
    data.encode_integer_u64(258);
    assert_eq!(data, vec![2, 1, 0, 0, 0, 0, 0, 0]);
}

/// Test to make sure the i8 integer encoder (`encode_integer_i8()`) works properly.
#[test]
fn test_encode_integer_i8() {

    // Check the encoder works properly.
    let mut data = vec![];
    data.encode_integer_i8(-2);
    assert_eq!(data, vec![254]);
}

/// Test to make sure the i16 integer encoder (`encode_integer_i16()`) works properly.
#[test]
fn test_encode_integer_i16() {

    // Check the encoder works properly.
    let mut data = vec![];
    data.encode_integer_i16(-258);
    assert_eq!(data, vec![254, 254]);
}

/// Test to make sure the i32 integer encoder (`encode_integer_i32()`) works properly.
#[test]
fn test_encode_integer_i32() {

    // Check the encoder works properly.
    let mut data = vec![];
    data.encode_integer_i32(-258);
    assert_eq!(data, vec![254, 254, 255, 255]);
}

/// Test to make sure the i64 integer encoder (`encode_integer_i64()`) works properly.
#[test]
fn test_encode_integer_i64() {

    // Check the encoder works properly.
    let mut data = vec![];
    data.encode_integer_i64(-258);
    assert_eq!(data, vec![254, 254, 255, 255, 255, 255, 255, 255]);
}

/// Test to make sure the f64 float encoder (`encode_float_f32()`) works properly.
#[test]
fn test_encode_float_f32() {

    // Check the encoder works properly.
    let mut data = vec![];
    data.encode_float_f32(-10.2);
    assert_eq!(data, vec![51, 51, 35, 193]);
}

/// Test to make sure the u8 string encoder (`encode_string_u8()`) works properly.
#[test]
fn test_encode_string_u8() {

    // Check the encoder works for a proper encoded string.
    let mut data = vec![];
    data.encode_string_u8("Wahahahaha");
    assert_eq!(data, vec![87, 97, 104, 97, 104, 97, 104, 97, 104, 97]);
}

/// Test to make sure the u8 0-padded string encoder (`encode_string_u8_0padded()`) works and fails properly.
#[test]
fn test_encode_string_u8_0padded() {

    // Check the encoder works for a proper encoded string.
    let mut data = vec![];
    assert_eq!(data.encode_string_u8_0padded(&("Waha".to_owned(), 8)).is_ok(), true);
    assert_eq!(data, vec![87, 97, 104, 97, 0, 0, 0, 0]);

    // Check the encoder fails properly when the lenght it's inferior to the current string's lenght.
    let mut data = vec![];
    let result = data.encode_string_u8_0padded(&("Waha".to_owned(), 3));
    assert_eq!(result.is_err(), true);
}

/// Test to make sure the u16 string encoder (`encode_string_u16()`) works properly.
#[test]
fn test_encode_string_u16() {

    // Check the encoder works for a proper encoded string.
    let mut data = vec![];
    data.encode_string_u16("Wahaha");
    assert_eq!(data, vec![87, 0, 97, 0, 104, 0, 97, 0, 104, 0, 97, 0]);
}

//---------------------------------------------------------------------------//
//                          Indexed Encoders
//---------------------------------------------------------------------------//

/// Test to make sure the u8 string specific encoder (`encode_packedfile_string_u8()`) works properly.
#[test]
fn test_encode_packedfile_string_u8() {

    // Check the encoder works for a proper encoded string.
    let mut data = vec![];
    data.encode_packedfile_string_u8("Wahaha");
    assert_eq!(data, vec![6, 0, 87, 97, 104, 97, 104, 97]);
}

/// Test to make sure the u16 string specific encoder (`encode_packedfile_string_u16()`) works properly.
#[test]
fn test_encode_packedfile_string_u16() {

    // Check the encoder works for a proper encoded string.
    let mut data = vec![];
    data.encode_packedfile_string_u16("Waha");
    assert_eq!(data, vec![4, 0, 87, 0, 97, 0, 104, 0, 97, 0]);
}

/// Test to make sure the u8 optional string specific encoder (`encode_packedfile_optional_string_u8()`)
/// works properly.
#[test]
fn test_encode_packedfile_optional_string_u8() {

    // Check the encoder works for a proper encoded string.
    let mut data = vec![];
    data.encode_packedfile_optional_string_u8("Wahaha");
    assert_eq!(data, vec![1, 6, 0, 87, 97, 104, 97, 104, 97]);

    let mut data = vec![];
    data.encode_packedfile_optional_string_u8("");
    assert_eq!(data, vec![0]);
}

/// Test to make sure the u16 optional string specific encoder (`encode_packedfile_optional_string_u16()`)
/// works properly.
#[test]
fn test_encode_packedfile_optional_string_u16() {

    // Check the encoder works for a proper encoded string.
    let mut data = vec![];
    data.encode_packedfile_optional_string_u16("Waha");
    assert_eq!(data, vec![1, 4, 0, 87, 0, 97, 0, 104, 0, 97, 0]);

    let mut data = vec![];
    data.encode_packedfile_optional_string_u16("");
    assert_eq!(data, vec![0]);
}
