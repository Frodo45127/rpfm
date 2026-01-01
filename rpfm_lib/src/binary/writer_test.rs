//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Tests for the [`WriteBytes`] trait.
//!
//! [`WriteBytes`]: crate::binary::WriteBytes

use nalgebra::{Vector2, Vector3, Vector4};

use super::WriteBytes;

//---------------------------------------------------------------------------//
//                          Normal Encoders
//---------------------------------------------------------------------------//

/// Test for WriteBytes::write_bool().
#[test]
fn write_bool() {

    // Check the writer works for a boolean.
    let mut data = vec![];
    assert!(data.write_bool(true).is_ok());
    assert_eq!(data, vec![1]);

    let mut data = vec![];
    assert!(data.write_bool(false).is_ok());
    assert_eq!(data, vec![0]);
}

/// Test for WriteBytes::write_u8().
#[test]
fn write_u8() {

    // Check the writer works properly.
    let mut data = vec![];
    assert!(data.write_u8(10).is_ok());
    assert_eq!(data, vec![10]);
}

/// Test for WriteBytes::write_u16().
#[test]
fn write_u16() {

    // Check the writer works properly.
    let mut data = vec![];
    assert!(data.write_u16(258).is_ok());
    assert_eq!(data, vec![2, 1]);
}

/// Test for WriteBytes::write_u24().
#[test]
fn write_u24() {

    // Check the writer works properly.
    let mut data = vec![];
    assert!(data.write_u24(8492696).is_ok());
    assert_eq!(data, vec![152, 150, 129]);
}

/// Test for WriteBytes::write_u32().
#[test]
fn write_u32() {

    // Check the writer works properly.
    let mut data = vec![];
    assert!(data.write_u32(258).is_ok());
    assert_eq!(data, vec![2, 1, 0, 0]);
}

/// Test for WriteBytes::write_u64().
#[test]
fn write_u64() {

    // Check the writer works properly.
    let mut data = vec![];
    assert!(data.write_u64(258).is_ok());
    assert_eq!(data, vec![2, 1, 0, 0, 0, 0, 0, 0]);
}

/// Test for WriteBytes::write_cauleb128().
#[test]
fn write_cauleb128() {

    // Check the writer works properly.
    let mut data = vec![];
    assert!(data.write_cauleb128(10, 0).is_ok());
    assert_eq!(data, vec![10]);
}

/// Test for WriteBytes::write_i8().
#[test]
fn write_i8() {

    // Check the writer works properly.
    let mut data = vec![];
    assert!(data.write_i8(-2).is_ok());
    assert_eq!(data, vec![254]);
}

/// Test for WriteBytes::write_i16().
#[test]
fn write_i16() {

    // Check the writer works properly.
    let mut data = vec![];
    assert!(data.write_i16(-258).is_ok());
    assert_eq!(data, vec![254, 254]);
}

/// Test for WriteBytes::write_i24().
#[test]
fn write_i24() {

    // Check the writer works properly.
    let mut data = vec![];
    assert!(data.write_i24(8492696).is_ok());
    assert_eq!(data, vec![152, 150, 129]);
}

/// Test for WriteBytes::write_i32().
#[test]
fn write_i32() {

    // Check the writer works properly.
    let mut data = vec![];
    assert!(data.write_i32(-258).is_ok());
    assert_eq!(data, vec![254, 254, 255, 255]);
}

/// Test for WriteBytes::write_i64().
#[test]
fn write_i64() {

    // Check the writer works properly.
    let mut data = vec![];
    assert!(data.write_i64(-258).is_ok());
    assert_eq!(data, vec![254, 254, 255, 255, 255, 255, 255, 255]);
}

/// Test for WriteBytes::write_optional_i16().
#[test]
fn write_optional_i16() {

    // Check the writer works properly.
    let mut data = vec![];
    assert!(data.write_optional_i16(-258).is_ok());
    assert_eq!(data, vec![1, 254, 254]);
}

/// Test for WriteBytes::write_optional_i32().
#[test]
fn write_optional_i32() {

    // Check the writer works properly.
    let mut data = vec![];
    assert!(data.write_optional_i32(-258).is_ok());
    assert_eq!(data, vec![1, 254, 254, 255, 255]);
}

/// Test for WriteBytes::write_optional_i64().
#[test]
fn write_optional_i64() {

    // Check the writer works properly.
    let mut data = vec![];
    assert!(data.write_optional_i64(-258).is_ok());
    assert_eq!(data, vec![1, 254, 254, 255, 255, 255, 255, 255, 255]);
}

/// Test for WriteBytes::write_f16().
#[test]
fn write_f16() {

    // Check the writer works properly.
    let mut data = vec![];
    assert!(data.write_f16(half::f16::from_f32(-10.2)).is_ok());
    assert_eq!(data, vec![26, 201]);
}

/// Test for WriteBytes::write_f32().
#[test]
fn write_f32() {

    // Check the writer works properly.
    let mut data = vec![];
    assert!(data.write_f32(-10.2).is_ok());
    assert_eq!(data, vec![51, 51, 35, 193]);
}

/// Test for WriteBytes::write_f32_normal_as_u8().
#[test]
fn write_f32_normal_as_u8() {

    // Check the writer works properly.
    let mut data = vec![];
    assert!(data.write_f32_normal_as_u8(0.9843137).is_ok());
    assert_eq!(data, vec![253]);
}

/// Test for WriteBytes::write_f64().
#[test]
fn write_f64() {

    // Check the writer works properly.
    let mut data = vec![];
    assert!(data.write_f64(-10.2).is_ok());
    assert_eq!(data, vec![102, 102, 102, 102, 102, 102, 36, 192]);
}

/// Test for WriteBytes::write_string_u8().
#[test]
fn write_string_u8() {

    // Check the writer works for a properly encoded string.
    let mut data = vec![];
    assert!(data.write_string_u8("Wahahahaha").is_ok());
    assert_eq!(data, vec![87, 97, 104, 97, 104, 97, 104, 97, 104, 97]);
}

/// Test for WriteBytes::write_string_u8_iso_8859_1().
#[test]
fn write_string_u8_iso_8859_1() {

    // Check the writer works for a properly encoded string.
    let mut data = vec![];
    assert!(data.write_string_u8_iso_8859_1("Wahaÿhahaha").is_ok());
    assert_eq!(data, vec![87, 97, 104, 97, 255, 104, 97, 104, 97, 104, 97]);
}

/// Test for WriteBytes::write_string_u8_0padded().
#[test]
fn write_string_u8_0padded() {

    // Check the writer works for a properly encoded string.
    let mut data = vec![];
    assert!(data.write_string_u8_0padded("Waha", 8, false).is_ok());
    assert_eq!(data, vec![87, 97, 104, 97, 0, 0, 0, 0]);

    // Check the writer fails properly when the length it's inferior to the current string's length.
    let mut data = vec![];
    let result = data.write_string_u8_0padded("Waha", 3, false);
    assert!(result.is_err());
}

/// Test for WriteBytes::write_string_u8_0terminated().
#[test]
fn write_string_u8_0terminated() {

    // Check the writer works for a properly encoded string.
    let mut data = vec![];
    assert!(data.write_string_u8_0terminated("Wahahaha").is_ok());
    assert_eq!(data, vec![87, 97, 104, 97, 104, 97, 104, 97, 0]);
}

/// Test for WriteBytes::write_sized_string_u8().
#[test]
fn write_sized_string_u8() {

    // Check the writer works for a properly encoded string.
    let mut data = vec![];
    assert!(data.write_sized_string_u8("Wahaha").is_ok());
    assert_eq!(data, vec![6, 0, 87, 97, 104, 97, 104, 97]);
}

/// Test for WriteBytes::write_sized_string_u8_u32().
#[test]
fn write_sized_string_u8_u32() {

    // Check the writer works for a properly encoded string.
    let mut data = vec![];
    assert!(data.write_sized_string_u8_u32("Wahaha").is_ok());
    assert_eq!(data, vec![6, 0, 0, 0, 87, 97, 104, 97, 104, 97]);
}

/// Test for WriteBytes::write_sized_string_u8().
#[test]
fn write_optional_string_u8() {

    // Check the writer works for a properly encoded string.
    let mut data = vec![];
    assert!(data.write_optional_string_u8("Wahaha").is_ok());
    assert_eq!(data, vec![1, 6, 0, 87, 97, 104, 97, 104, 97]);

    let mut data = vec![];
    assert!(data.write_optional_string_u8("").is_ok());
    assert_eq!(data, vec![0]);
}

/// Test for WriteBytes::write_string_u16().
#[test]
fn write_string_u16() {

    // Check the writer works for a properly encoded string.
    let mut data = vec![];
    assert!(data.write_string_u16("Wahaha").is_ok());
    assert_eq!(data, vec![87, 0, 97, 0, 104, 0, 97, 0, 104, 0, 97, 0]);
}

/// Test for WriteBytes::write_string_u16_0padded().
#[test]
fn write_string_u16_0padded() {

    // Check the writer works for a properly encoded string.
    let mut data = vec![];
    assert!(data.write_string_u16_0padded("Waha", 16, false).is_ok());
    assert_eq!(data, vec![87, 0, 97, 0, 104, 0, 97, 0, 0, 0, 0, 0, 0, 0, 0, 0]);

    // Check the writer fails properly when the length it's inferior to the current string's length.
    let mut data = vec![];
    let result = data.write_string_u16_0padded("Waha", 6, false);
    assert!(result.is_err());
}

/// Test for WriteBytes::write_sized_string_u16().
#[test]
fn write_sized_string_u16() {

    // Check the writer works for a properly encoded string.
    let mut data = vec![];
    assert!(data.write_sized_string_u16("¡Bebes mejor de lo que luchas, Zhang Fei!").is_ok());
    assert_eq!(data, vec![0x29, 0x00, 0xA1, 0x00, 0x42, 0x00, 0x65, 0x00, 0x62, 0x00, 0x65, 0x00, 0x73, 0x00, 0x20, 0x00, 0x6D, 0x00, 0x65, 0x00, 0x6A, 0x00, 0x6F, 0x00, 0x72, 0x00, 0x20, 0x00, 0x64, 0x00, 0x65, 0x00, 0x20, 0x00, 0x6C, 0x00, 0x6F, 0x00, 0x20, 0x00, 0x71, 0x00, 0x75, 0x00, 0x65, 0x00, 0x20, 0x00, 0x6C, 0x00, 0x75, 0x00, 0x63, 0x00, 0x68, 0x00, 0x61, 0x00, 0x73, 0x00, 0x2C, 0x00, 0x20, 0x00, 0x5A, 0x00, 0x68, 0x00, 0x61, 0x00, 0x6E, 0x00, 0x67, 0x00, 0x20, 0x00, 0x46, 0x00, 0x65, 0x00, 0x69, 0x00, 0x21, 0x00]);
}

/// Test for WriteBytes::write_sized_string_u16_u32().
#[test]
fn write_sized_string_u16_u32() {

    // Check the writer works for a properly encoded string.
    let mut data = vec![];
    assert!(data.write_sized_string_u16_u32("¡Bebes mejor de lo que luchas, Zhang Fei!").is_ok());
    assert_eq!(data, vec![0x29, 0x00, 0x00, 0x00, 0xA1, 0x00, 0x42, 0x00, 0x65, 0x00, 0x62, 0x00, 0x65, 0x00, 0x73, 0x00, 0x20, 0x00, 0x6D, 0x00, 0x65, 0x00, 0x6A, 0x00, 0x6F, 0x00, 0x72, 0x00, 0x20, 0x00, 0x64, 0x00, 0x65, 0x00, 0x20, 0x00, 0x6C, 0x00, 0x6F, 0x00, 0x20, 0x00, 0x71, 0x00, 0x75, 0x00, 0x65, 0x00, 0x20, 0x00, 0x6C, 0x00, 0x75, 0x00, 0x63, 0x00, 0x68, 0x00, 0x61, 0x00, 0x73, 0x00, 0x2C, 0x00, 0x20, 0x00, 0x5A, 0x00, 0x68, 0x00, 0x61, 0x00, 0x6E, 0x00, 0x67, 0x00, 0x20, 0x00, 0x46, 0x00, 0x65, 0x00, 0x69, 0x00, 0x21, 0x00]);
}

/// Test for WriteBytes::write_optional_string_u16().
#[test]
fn write_optional_string_u16() {

    // Check the writer works for a properly encoded string.
    let mut data = vec![];
    assert!(data.write_optional_string_u16("Waha").is_ok());
    assert_eq!(data, vec![1, 4, 0, 87, 0, 97, 0, 104, 0, 97, 0]);

    let mut data = vec![];
    assert!(data.write_optional_string_u16("").is_ok());
    assert_eq!(data, vec![0]);
}

/// Test for WriteBytes::write_string_colour_rgb().
#[test]
fn write_string_colour_rgb() {

    // Check the writer works for a properly encoded string.
    let mut data = vec![];
    assert!(data.write_string_colour_rgb("0504FF").is_ok());
    assert_eq!(data, vec![0xFF, 0x04, 0x05, 0x00]);
}

/// Test for WriteBytes::write_vector_2_u8().
#[test]
fn write_vector_2_u8() {
    let mut data = vec![];
    assert!(data.write_vector_2_u8(Vector2::new(10, 10)).is_ok());
    assert_eq!(data, vec![0x0A, 0x0A]);
}

/// Test for WriteBytes::write_vector_2_f32_pct_as_vector_2_u8().
#[test]
fn write_vector_2_f32_pct_as_vector_2_u8() {
    let mut data = vec![];
    assert!(data.write_vector_2_f32_pct_as_vector_2_u8(Vector2::new(0.039215688, 0.039215688)).is_ok());
    assert_eq!(data, vec![0x0A, 0x0A]);
}

/// Test for WriteBytes::write_vector_2_f32_as_vector_2_f16().
#[test]
fn write_vector_2_f32_as_vector_2_f16() {
    let mut data = vec![];
    assert!(data.write_vector_2_f32_as_vector_2_f16(Vector2::new(0.00018429756, 0.00018429756)).is_ok());
    assert_eq!(data, vec![
        0x0A, 0x0A,
        0x0A, 0x0A
    ]);
}

/// Test for WriteBytes::write_vector_3_f32_normal_as_vector_4_u8().
#[test]
fn write_vector_3_f32_normal_as_vector_4_u8() {
    let mut data = vec![];
    assert!(data.write_vector_3_f32_normal_as_vector_4_u8(Vector3::new(-0.92156863, -0.92156863, -0.92156863)).is_ok());
    assert_eq!(data, vec![0x0A, 0x0A, 0x0A, 0x00]);
}

/// Test for WriteBytes::write_vector_4_u8().
#[test]
fn write_vector_4_u8() {
    let mut data = vec![];
    assert!(data.write_vector_4_u8(Vector4::new(10, 10, 10, 10)).is_ok());
    assert_eq!(data, vec![0x0A, 0x0A, 0x0A, 0x0A]);
}

/// Test for WriteBytes::write_vector_4_f32().
#[test]
fn write_vector_4_f32() {
    let mut data = vec![];
    assert!(data.write_vector_4_f32(Vector4::new(10.0, 10.0, 10.0, 10.0)).is_ok());
    assert_eq!(data, vec![0, 0, 32, 65, 0, 0, 32, 65, 0, 0, 32, 65, 0, 0, 32, 65]);
}

/// Test for WriteBytes::write_vector_4_f32_to_vector_3_f32().
#[test]
fn write_vector_4_f32_to_vector_3_f32() {
    let mut data = vec![];
    assert!(data.write_vector_4_f32_to_vector_3_f32(Vector4::new(10.0, 10.0, 10.0, 0.0)).is_ok());
    assert_eq!(data, vec![0, 0, 32, 65, 0, 0, 32, 65, 0, 0, 32, 65]);
}

/// Test for WriteBytes::write_vector_4_f32_pct_as_vector_4_u8().
#[test]
fn write_vector_4_f32_pct_as_vector_4_u8() {
    let mut data = vec![];
    assert!(data.write_vector_4_f32_pct_as_vector_4_u8(Vector4::new(0.039215688, 0.039215688, 0.039215688, 0.039215688)).is_ok());
    assert_eq!(data, vec![0x0A, 0x0A, 0x0A, 0x0A]);
}

/// Test for WriteBytes::write_vector_4_f32_normal_as_vector_4_u8().
#[test]
fn write_vector_4_f32_normal_as_vector_4_u8() {
    let mut data = vec![];
    assert!(data.write_vector_4_f32_normal_as_vector_4_u8(Vector4::new(-0.92156863, -0.92156863, -0.92156863, -0.92156863)).is_ok());
    assert_eq!(data, vec![
        0x0A,
        0x0A,
        0x0A,
        0x0A
    ]);
}

/// Test for WriteBytes::write_vector_4_f32_normal_as_vector_4_f16().
#[test]
fn write_vector_4_f32_normal_as_vector_4_f16() {
    let mut data = vec![];
    assert!(data.write_vector_4_f32_normal_as_vector_4_f16(Vector4::new(3.096775, 3.096775, 3.096775, 1.7597656)).is_ok());
    assert_eq!(data, vec![
        0x0A, 0x3F,
        0x0A, 0x3F,
        0x0A, 0x3F,
        0x0A, 0x3F
    ]);
}
