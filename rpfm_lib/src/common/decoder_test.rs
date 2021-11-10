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
Module containing test for the entire `Decoder` implementation for `&[u8]`.
!*/

use crate::common::decoder::Decoder;

//---------------------------------------------------------------------------//
//                          Normal Decoders
//---------------------------------------------------------------------------//

/// Test to make sure the boolean decoder (`decode_bool()`) works and fails properly.
#[test]
fn test_decode_bool() {

    // Check that a normal boolean value is decoded properly.
    assert_eq!(Decoder::decode_bool([0].as_ref(), 0).unwrap(), false);
    assert_eq!(Decoder::decode_bool([1].as_ref(), 0).unwrap(), true);

    // Check that trying to decode a non-boolean value returns an error.
    assert_eq!(Decoder::decode_bool([2].as_ref(), 0).is_err(), true);
}

/// Test to make sure the u8 integer decoder (`decode_integer_u8()`) works and fails properly.
#[test]
fn test_decode_integer_u8() {

    // Check the decoding works for a proper value.
    assert_eq!(Decoder::decode_integer_u8([10].as_ref(), 0).unwrap(), 10);

    // Check the decoder returns an error for a slice who's length is smaller than 1.
    assert_eq!(Decoder::decode_integer_u8([].as_ref(), 0).is_err(), true);
}

/// Test to make sure the u16 integer decoder (`decode_integer_u16()`) works and fails properly.
#[test]
fn test_decode_integer_u16() {

    // Check the decoding works for a proper value.
    assert_eq!(Decoder::decode_integer_u16([10, 0].as_ref(), 0).unwrap(), 10);

    // Check the decoder returns an error for a slice who's length is smaller than 2.
    assert_eq!(Decoder::decode_integer_u16([10].as_ref(), 0).is_err(), true);
}

/// Test to make sure the u24 integer decoder (`decode_integer_u24()`) works and fails properly.
#[test]
fn test_decode_integer_u24() {

    // Check the decoding works for a proper value.
    assert_eq!(Decoder::decode_integer_u24([152, 150, 129].as_ref(), 0).unwrap(), 8492696);

    // Check the decoder returns an error for a slice who's length is smaller than 3.
    assert_eq!(Decoder::decode_integer_u24([152, 150].as_ref(), 0).is_err(), true);
}


/// Test to make sure the u32 integer decoder (`decode_integer_u32()`) works and fails properly.
#[test]
fn test_decode_integer_u32() {

    // Check the decoding works for a proper value.
    assert_eq!(Decoder::decode_integer_u32([10, 0, 0, 0].as_ref(), 0).unwrap(), 10);

    // Check the decoder returns an error for a slice who's length is smaller than 4.
    assert_eq!(Decoder::decode_integer_u32([10, 0, 0].as_ref(), 0).is_err(), true);
}

/// Test to make sure the u64 integer decoder (`decode_integer_u64()`) works and fails properly.
#[test]
fn test_decode_integer_u64() {

    // Check the decoding works for a proper value.
    assert_eq!(Decoder::decode_integer_u64([10, 0, 0, 0, 0, 0, 0, 0].as_ref(), 0).unwrap(), 10);

    // Check the decoder returns an error for a slice who's length is smaller than 8.
    assert_eq!(Decoder::decode_integer_u64([10, 0, 0, 0, 0].as_ref(), 0).is_err(), true);
}

/// Test to make sure the i8 integer decoder (`decode_integer_i8()`) works and fails properly.
#[test]
fn test_decode_integer_i8() {

    // Check the decoding works for a proper value.
    assert_eq!(Decoder::decode_integer_i8([254].as_ref(), 0).unwrap(), -2);

    // Check the decoder returns an error for a slice who's length is smaller than 1.
    assert_eq!(Decoder::decode_integer_i8([].as_ref(), 0).is_err(), true);
}

/// Test to make sure the i16 integer decoder (`decode_integer_i16()`) works and fails properly.
#[test]
fn test_decode_integer_i16() {

    // Check the decoding works for a proper value.
    assert_eq!(Decoder::decode_integer_i16([254, 254].as_ref(), 0).unwrap(), -258);

    // Check the decoder returns an error for a slice who's length is smaller than 2.
    assert_eq!(Decoder::decode_integer_i16([10].as_ref(), 0).is_err(), true);
}

/// Test to make sure the i24 integer decoder (`decode_integer_i24()`) works and fails properly.
#[test]
fn test_decode_integer_i24() {

    // Check the decoding works for a proper value.
    assert_eq!(Decoder::decode_integer_i24([152, 150, 129].as_ref(), 0).unwrap(), 8492696);

    // Check the decoder returns an error for a slice who's length is smaller than 3.
    assert_eq!(Decoder::decode_integer_i24([152, 150].as_ref(), 0).is_err(), true);
}

/// Test to make sure the i32 integer decoder (`decode_integer_i32()`) works and fails properly.
#[test]
fn test_decode_integer_i32() {

    // Check the decoding works for a proper value.
    assert_eq!(Decoder::decode_integer_i32([10, 0, 0, 0].as_ref(), 0).unwrap(), 10);

    // Check the decoder returns an error for a slice who's length is smaller than 4.
    assert_eq!(Decoder::decode_integer_i32([10, 0, 0].as_ref(), 0).is_err(), true);
}

/// Test to make sure the i64 integer decoder (`decode_integer_i64()`) works and fails properly.
#[test]
fn test_decode_integer_i64() {

    // Check the decoding works for a proper value.
    assert_eq!(Decoder::decode_integer_i64([10, 0, 0, 0, 0, 0, 0, 0].as_ref(), 0).unwrap(), 10);

    // Check the decoder returns an error for a slice who's length is smaller than 8.
    assert_eq!(Decoder::decode_integer_i64([10, 0, 0].as_ref(), 0).is_err(), true);
}


/// Test to make sure the f32 float decoder (`decode_float_f32()`) works and fails properly.
#[test]
fn test_decode_float_f32() {

    // Check the decoding works for a proper value.
    assert_eq!(Decoder::decode_float_f32([0, 0, 32, 65].as_ref(), 0).unwrap(), 10.0);

    // Check the decoder returns an error for a slice who's length is smaller than 4.
    assert_eq!(Decoder::decode_float_f32([0, 32, 65].as_ref(), 0).is_err(), true);
}

/// Test to make sure the f64 float decoder (`decode_float_f64()`) works and fails properly.
#[test]
fn test_decode_float_f64() {

    // Check the decoding works for a proper value.
    assert_eq!(Decoder::decode_float_f64([0, 0, 0, 0, 0, 0, 36, 64].as_ref(), 0).unwrap(), 10.0);

    // Check the decoder returns an error for a slice who's length is smaller than 8.
    assert_eq!(Decoder::decode_float_f64([0, 0, 0, 0, 36, 64].as_ref(), 0).is_err(), true);
}

/// Test to make sure the u8 string decoder (`decode_string_u8()`) works and fails properly.
#[test]
fn test_decode_string_u8() {

    // Check the decoding works for a proper encoded string.
    assert_eq!(Decoder::decode_string_u8([87, 97, 104, 97, 104, 97, 104, 97, 104, 97].as_ref(), 0, 10).unwrap(), "Wahahahaha");

    // Check the decoder returns an error for a slice with non-UTF8 characters (255).
    assert_eq!(Decoder::decode_string_u8([87, 97, 104, 97, 255, 104, 97, 104, 97, 104, 97].as_ref(), 0, 10).is_err(), true);
}

/// Test to make sure the u8 0-padded string decoder (`decode_string_u8_0padded()`) works and fails properly.
#[test]
fn test_decode_string_u8_0padded() {

    // Check the decoding works for a proper encoded string.
    assert_eq!(Decoder::decode_string_u8_0padded([87, 97, 104, 97, 104, 97, 0, 0, 0, 0].as_ref(), 0, 10).unwrap().0, "Wahaha");
    assert_eq!(Decoder::decode_string_u8_0padded([87, 97, 104, 97, 104, 97, 0, 0, 0, 0].as_ref(), 0, 10).unwrap().1, 10);

    // Check that, as soon as it finds a 0 (null character) the decoding stops.
    assert_eq!(Decoder::decode_string_u8_0padded([87, 97, 104, 97, 0, 104, 97, 0, 0, 0].as_ref(), 0, 10).unwrap().0, "Waha");
    assert_eq!(Decoder::decode_string_u8_0padded([87, 97, 104, 97, 0, 104, 97, 0, 0, 0].as_ref(), 0, 10).unwrap().1, 10);

    // Check the decoder returns an error for a slice with non-UTF8 characters (255).
    assert_eq!(Decoder::decode_string_u8_0padded([87, 97, 104, 97, 255, 104, 97, 104, 97, 104, 97, 0, 0].as_ref(), 0, 12).is_err(), true);

    // Check the decoder returns the full string if no zeros have been found before the end of the slice.
    assert_eq!(Decoder::decode_string_u8_0padded([87, 97, 104, 97, 104, 97, 104, 97, 104, 97].as_ref(), 0, 10).unwrap().0, "Wahahahaha");
    assert_eq!(Decoder::decode_string_u8_0padded([87, 97, 104, 97, 104, 97, 104, 97, 104, 97].as_ref(), 0, 10).unwrap().1, 10);
}

/// Test to make sure the u8 0-terminated string decoder (`decode_string_u8_0terminated()`)
/// works and fails properly.
#[test]
fn test_decode_string_u8_0terminated() {

    // Check the decoding works for a proper encoded string.
    {
        let result = Decoder::decode_string_u8_0terminated([87, 97, 104, 97, 104, 97, 104, 97, 0, 97].as_ref(), 0).unwrap();
        assert_eq!(result.0, "Wahahaha".to_owned());
        assert_eq!(result.1, 9);
    }

    // Check the decoder works for a string that doesn't end in zero, but in end of slice.
    {
        let result = Decoder::decode_string_u8_0terminated([87, 97, 104, 97, 104, 97, 104, 97, 104, 97].as_ref(), 0).unwrap();
        assert_eq!(result.0, "Wahahahaha".to_owned());
        assert_eq!(result.1, 10);
    }

    // Check the decoder works for a slice with non-UTF8 characters (255).
    {
        let result = Decoder::decode_string_u8_0terminated([87, 97, 104, 97, 255, 104, 97, 104, 97, 104, 97].as_ref(), 0).unwrap();
        assert_eq!(result.0, "Waha�hahaha".to_owned());
        assert_eq!(result.1, 11);
    }

    // Check to avoid a regression.
    {
        let result = Decoder::decode_string_u8_0terminated([87, 97, 104, 97, 255, 104, 97, 104, 97, 104, 97].as_ref(), 6).unwrap();
        assert_eq!(result.0, "ahaha".to_owned());
        assert_eq!(result.1, 5);
    }
}

/// Test to make sure the u16 string decoder (`decode_string_u16()`) works and fails properly.
#[test]
fn test_decode_string_u16() {

    // Check the decoding works for a proper encoded string.
    assert_eq!(Decoder::decode_string_u16([87, 0, 97, 0, 104, 0, 97, 0, 104, 0, 97, 0].as_ref(), 0, 12).unwrap(), "Wahaha");

    // Check the decoder returns an error for a slice with non-UTF8 characters (216).
    assert_eq!(Decoder::decode_string_u16([87, 0, 0, 216, 104, 0, 97, 0, 104, 0, 97, 0].as_ref(), 0, 12).is_err(), true);
}

/// Test to make sure the u16 0-padded string decoder (`decode_string_u16_0padded()`) works and fails properly.
#[test]
fn test_decode_string_u16_0padded() {

    // Check the decoding works for a proper encoded string.
    assert_eq!(Decoder::decode_string_u16_0padded([87, 0, 97, 0, 104, 0, 97, 0, 104, 0, 97, 0, 0, 0, 0, 0, 0, 0, 0, 0].as_ref(), 0, 20).unwrap().0, "Wahaha");
    assert_eq!(Decoder::decode_string_u16_0padded([87, 0, 97, 0, 104, 0, 97, 0, 104, 0, 97, 0, 0, 0, 0, 0, 0, 0, 0, 0].as_ref(), 0, 20).unwrap().1, 20);

    // Check that, as soon as it finds a 0 (null character) the decoding stops.
    assert_eq!(Decoder::decode_string_u16_0padded([87, 0, 97, 0, 104, 0, 97, 0, 0, 0, 104, 0, 97, 0, 0, 0, 0, 0, 0, 0].as_ref(), 0, 20).unwrap().0, "Waha");
    assert_eq!(Decoder::decode_string_u16_0padded([87, 0, 97, 0, 104, 0, 97, 0, 0, 0, 104, 0, 97, 0, 0, 0, 0, 0, 0, 0].as_ref(), 0, 20).unwrap().1, 20);

    // Check the decoder returns the full string if no zeros have been found before the end of the slice.
    assert_eq!(Decoder::decode_string_u16_0padded([87, 0, 97, 0, 104, 0, 97, 0, 104, 0, 97, 0, 104, 0, 97, 0, 104, 0, 97, 0].as_ref(), 0, 20).unwrap().0, "Wahahahaha");
    assert_eq!(Decoder::decode_string_u16_0padded([87, 0, 97, 0, 104, 0, 97, 0, 104, 0, 97, 0, 104, 0, 97, 0, 104, 0, 97, 0].as_ref(), 0, 20).unwrap().1, 20);
}

//---------------------------------------------------------------------------//
//                          Indexed Decoders
//---------------------------------------------------------------------------//

/// Test to make sure the boolean specific decoder (`decode_packedfile_bool()`) works and fails properly.
#[test]
fn test_decode_packedfile_bool() {

    // Check that a normal boolean value is decoded properly.
    {
        let mut index = 0;
        assert_eq!(Decoder::decode_packedfile_bool([0].as_ref(), 0, &mut index).unwrap(), false);
        assert_eq!(index, 1);
        assert_eq!(Decoder::decode_packedfile_bool([1].as_ref(), 0, &mut index).unwrap(), true);
        assert_eq!(index, 2);
    }

    // Check that trying to decode a non-boolean value returns an error.
    {
        let mut index = 0;
        assert_eq!(Decoder::decode_packedfile_bool([2].as_ref(), 0, &mut index).is_err(), true);
        assert_eq!(index, 0);
    }
}

/// Test to make sure the u8 integer specific decoder (`decode_packedfile_integer_u8()`) works
/// and fails properly.
#[test]
fn test_decode_packedfile_integer_u8() {

    // Check the decoding works for a proper value.
    {
        let mut index = 0;
        assert_eq!(Decoder::decode_packedfile_integer_u8([10].as_ref(), 0, &mut index).unwrap(), 10);
        assert_eq!(index, 1);
    }

    // Check the decoder returns an error for a slice whose length is smaller than 1.
    {
        let mut index = 0;
        assert_eq!(Decoder::decode_packedfile_integer_u8([].as_ref(), 0, &mut index).is_err(), true);
        assert_eq!(index, 0);
    }
}

/// Test to make sure the u16 integer specific decoder (`decode_packedfile_integer_u16()`) works
/// and fails properly.
#[test]
fn test_decode_packedfile_integer_u16() {

    // Check the decoding works for a proper value.
    {
        let mut index = 0;
        assert_eq!(Decoder::decode_packedfile_integer_u16([10, 0].as_ref(), 0, &mut index).unwrap(), 10);
        assert_eq!(index, 2);
    }

    // Check the decoder returns an error for a slice whose length is smaller than 2.
    {
        let mut index = 0;
        assert_eq!(Decoder::decode_packedfile_integer_u16([10].as_ref(), 0, &mut index).is_err(), true);
        assert_eq!(index, 0);
    }
}

/// Test to make sure the u24 integer specific decoder (`decode_packedfile_integer_u24()`) works
/// and fails properly.
#[test]
fn test_decode_packedfile_integer_u24() {

    // Check the decoding works for a proper value.
    {
        let mut index = 0;
        assert_eq!(Decoder::decode_packedfile_integer_u24([152, 150, 129].as_ref(), 0, &mut index).unwrap(), 8492696);
        assert_eq!(index, 3);
    }


    // Check the decoder returns an error for a slice whose length is smaller than 3.
    {
        let mut index = 0;
        assert_eq!(Decoder::decode_packedfile_integer_u24([152, 150].as_ref(), 0, &mut index).is_err(), true);
        assert_eq!(index, 0);
    }
}

/// Test to make sure the u32 integer specific decoder (`decode_packedfile_integer_u32()`) works
/// and fails properly.
#[test]
fn test_decode_packedfile_integer_u32() {

    // Check the decoding works for a proper value.
    {
        let mut index = 0;
        assert_eq!(Decoder::decode_packedfile_integer_u32([10, 0, 0, 0].as_ref(), 0, &mut index).unwrap(), 10);
        assert_eq!(index, 4);
    }


    // Check the decoder returns an error for a slice whose length is smaller than 4.
    {
        let mut index = 0;
        assert_eq!(Decoder::decode_packedfile_integer_u32([10, 0].as_ref(), 0, &mut index).is_err(), true);
        assert_eq!(index, 0);
    }
}

/// Test to make sure the u64 integer specific decoder (`decode_packedfile_integer_u64()`) works
/// and fails properly.
#[test]
fn test_decode_packedfile_integer_u64() {

    // Check the decoding works for a proper value.
    {
        let mut index = 0;
        assert_eq!(Decoder::decode_packedfile_integer_u64([10, 0, 0, 0, 0, 0, 0, 0].as_ref(), 0, &mut index).unwrap(), 10);
        assert_eq!(index, 8);
    }

    // Check the decoder returns an error for a slice whose length is smaller than 8.
    {
        let mut index = 0;
        assert_eq!(Decoder::decode_packedfile_integer_u64([10, 0].as_ref(), 0, &mut index).is_err(), true);
        assert_eq!(index, 0);
    }
}

/// Test to make sure the uleb_128 encoded integer decoder (`decode_packedfile_integer_cauleb128()`) works and fails properly.
#[test]
fn test_decode_packedfile_integer_cauleb128() {

    // Check the decoding works for a proper value.
    assert_eq!(Decoder::decode_packedfile_integer_cauleb128([0x80, 10].as_ref(), &mut 0).unwrap(), 10);

    // Check the decoder returns an error for a slice that's not big enough.
    assert_eq!(Decoder::decode_packedfile_integer_cauleb128([].as_ref(), &mut 0).is_err(), true);
}


/// Test to make sure the i8 integer specific decoder (`decode_packedfile_integer_i8()`) works
/// and fails properly.
#[test]
fn test_decode_packedfile_integer_i8() {

    // Check the decoding works for a proper value.
    {
        let mut index = 0;
        assert_eq!(Decoder::decode_packedfile_integer_i8([254].as_ref(), 0, &mut index).unwrap(), -2);
        assert_eq!(index, 1);
    }

    // Check the decoder returns an error for a slice whose length is smaller than 1.
    {
        let mut index = 0;
        assert_eq!(Decoder::decode_packedfile_integer_i8([].as_ref(), 0, &mut index).is_err(), true);
        assert_eq!(index, 0);
    }
}

/// Test to make sure the i16 integer specific decoder (`decode_packedfile_integer_i16()`) works
/// and fails properly.
#[test]
fn test_decode_packedfile_integer_i16() {

    // Check the decoding works for a proper value.
    {
        let mut index = 0;
        assert_eq!(Decoder::decode_packedfile_integer_i16([254, 254].as_ref(), 0, &mut index).unwrap(), -258);
        assert_eq!(index, 2);
    }

    // Check the decoder returns an error for a slice whose length is smaller than 2.
    {
        let mut index = 0;
        assert_eq!(Decoder::decode_packedfile_integer_i16([10].as_ref(), 0, &mut index).is_err(), true);
        assert_eq!(index, 0);
    }
}

/// Test to make sure the i24 integer specific decoder (`decode_packedfile_integer_i24()`) works
/// and fails properly.
#[test]
fn test_decode_packedfile_integer_i24() {

    // Check the decoding works for a proper value.
    {
        let mut index = 0;
        assert_eq!(Decoder::decode_packedfile_integer_i24([152, 150, 129].as_ref(), 0, &mut index).unwrap(), 8492696);
        assert_eq!(index, 3);
    }


    // Check the decoder returns an error for a slice whose length is smaller than 3.
    {
        let mut index = 0;
        assert_eq!(Decoder::decode_packedfile_integer_i24([152, 150].as_ref(), 0, &mut index).is_err(), true);
        assert_eq!(index, 0);
    }
}

/// Test to make sure the i32 integer specific decoder (`decode_packedfile_integer_i32()`) works
/// and fails properly.
#[test]
fn test_decode_packedfile_integer_i32() {

    // Check the decoding works for a proper value.
    {
        let mut index = 0;
        assert_eq!(Decoder::decode_packedfile_integer_i32([254, 254, 255, 255].as_ref(), 0, &mut index).unwrap(), -258);
        assert_eq!(index, 4);
    }

    // Check the decoder returns an error for a slice whose length is smaller than 4.
    {
        let mut index = 0;
        assert_eq!(Decoder::decode_packedfile_integer_i32([10, 0].as_ref(), 0, &mut index).is_err(), true);
        assert_eq!(index, 0);
    }
}

/// Test to make sure the i64 integer specific decoder (`decode_packedfile_integer_i64()`) works
/// and fails properly.
#[test]
fn test_decode_packedfile_integer_i64() {

    // Check the decoding works for a proper value.
    {
        let mut index = 0;
        assert_eq!(Decoder::decode_packedfile_integer_i64([254, 254, 255, 255, 255, 255, 255, 255].as_ref(), 0, &mut index).unwrap(), -258);
        assert_eq!(index, 8);
    }


    // Check the decoder returns an error for a slice whose length is smaller than 8.
    {
        let mut index = 0;
        assert_eq!(Decoder::decode_packedfile_integer_i64([10, 0].as_ref(), 0, &mut index).is_err(), true);
        assert_eq!(index, 0);
    }
}

/// Test to make sure the f32 float specific decoder (`decode_packedfile_float_f32()`) works
/// and fails properly.
#[test]
fn test_decode_packedfile_float_f32() {

    // Check the decoding works for a proper value.
    {
        let mut index = 0;
        assert_eq!(Decoder::decode_packedfile_float_f32([51, 51, 35, 193].as_ref(), 0, &mut index).unwrap(), -10.2);
        assert_eq!(index, 4);
    }

    // Check the decoder returns an error for a slice whose length is smaller than 4.
    {
        let mut index = 0;
        assert_eq!(Decoder::decode_packedfile_float_f32([10, 0].as_ref(), 0, &mut index).is_err(), true);
        assert_eq!(index, 0);
    }
}

/// Test to make sure the u8 string specific decoder (`decode_packedfile_string_u8()`) works
/// and fails properly.
#[test]
fn test_decode_packedfile_string_u8() {

    // Check the decoding works for a proper encoded string.
    {
        let mut index = 0;
        assert_eq!(Decoder::decode_packedfile_string_u8([10, 0, 87, 97, 104, 97, 104, 97, 104, 97, 104, 97].as_ref(), 0, &mut index).unwrap(), "Wahahahaha".to_owned());
        assert_eq!(index, 12);
    }

    // Check the decoder returns an error for a slice with less than two bytes.
    {
        let mut index = 0;
        assert_eq!(Decoder::decode_packedfile_string_u8([5].as_ref(), 0, &mut index).is_err(), true);
        assert_eq!(index, 0);
    }

    // Check the decoder returns an error for a slice shorter than it's specified length.
    {
        let mut index = 0;
        assert_eq!(Decoder::decode_packedfile_string_u8([4, 0, 2].as_ref(), 0, &mut index).is_err(), true);
        assert_eq!(index, 0);
    }

    // Check the decoder returns an error for a slice with non-UTF8 characters (255).
    {
        let mut index = 0;
        assert_eq!(Decoder::decode_packedfile_string_u8([11, 0, 87, 97, 104, 97, 255, 104, 97, 104, 97, 104, 97].as_ref(), 0, &mut index).is_err(), true);
        assert_eq!(index, 0);
    }
}

/// Test to make sure the u8 0-terminated string specific decoder (`decode_packedfile_string_u8_0terminated()`)
/// works and fails properly.
#[test]
fn test_decode_packedfile_string_u8_0terminated() {

    // Check the decoding works for a proper encoded string.
    {
        let mut index = 0;
        assert_eq!(Decoder::decode_packedfile_string_u8_0terminated([87, 97, 104, 97, 104, 97, 104, 97, 0, 97].as_ref(), 0, &mut index).unwrap(), "Wahahaha".to_owned());
        assert_eq!(index, 9);
    }

    // Check the decoder works for a string that doesn't end in zero, but in end of slice.
    {
        let mut index = 0;
        assert_eq!(Decoder::decode_packedfile_string_u8_0terminated([87, 97, 104, 97, 104, 97, 104, 97, 104, 97].as_ref(), 0, &mut index).unwrap(), "Wahahahaha".to_owned());
        assert_eq!(index, 10);
    }

    // Check the decoder works for a slice with non-UTF8 characters (255).
    {
        let mut index = 0;
        assert_eq!(Decoder::decode_packedfile_string_u8_0terminated([87, 97, 104, 97, 255, 104, 97, 104, 97, 104, 97].as_ref(), 0, &mut index).unwrap(), "Waha�hahaha".to_owned());
        assert_eq!(index, 11);
    }
}

/// Test to make sure the u16 string specific decoder (`decode_packedfile_string_u16()`) works
/// and fails properly.
#[test]
fn test_decode_packedfile_string_u16() {

    // Check the decoding works for a proper encoded string.
    {
        let mut index = 0;
        assert_eq!(Decoder::decode_packedfile_string_u16([4, 0, 87, 0, 97, 0, 104, 0, 97, 0].as_ref(), 0, &mut index).unwrap(), "Waha".to_owned());
        assert_eq!(index, 10);
    }

    // Check the decoder returns an error for a slice with less than two bytes.
    {
        let mut index = 0;
        assert_eq!(Decoder::decode_packedfile_string_u16([5].as_ref(), 0, &mut index).is_err(), true);
        assert_eq!(index, 0);
    }

    // Check the decoder returns an error for a slice shorter than it's specified length.
    {
        let mut index = 0;
        assert_eq!(Decoder::decode_packedfile_string_u16([4, 0, 2].as_ref(), 0, &mut index).is_err(), true);
        assert_eq!(index, 0);
    }

    // Check the decoder returns an error for a slice with non-UTF16 characters (1, 216, DC01).
    {
        let mut index = 0;
        assert_eq!(Decoder::decode_packedfile_string_u16([4, 0, 87, 0, 97, 0, 1, 216, 97, 0].as_ref(), 0, &mut index).is_err(), true);
        assert_eq!(index, 0);
    }
}

/// Test to make sure the u8 optional string specific decoder (`decode_packedfile_optional_string_u8()`)
/// works and fails properly.
#[test]
fn test_decode_packedfile_optional_string_u8() {

    // Check the decoding works for a nonexistent string.
    {
        let mut index = 0;
        assert_eq!(Decoder::decode_packedfile_optional_string_u8([0].as_ref(), 0, &mut index).unwrap(), "".to_owned());
        assert_eq!(index, 1);
    }

    // Check the decoding works for a proper encoded string.
    {
        let mut index = 0;
        assert_eq!(Decoder::decode_packedfile_optional_string_u8([1, 10, 0, 87, 97, 104, 97, 104, 97, 104, 97, 104, 97].as_ref(), 0, &mut index).unwrap(), "Wahahahaha".to_owned());
        assert_eq!(index, 13);
    }

    // Check the decoder returns an error for a slice when it expects a string after the bool, but founds nothing.
    {
        let mut index = 0;
        assert_eq!(Decoder::decode_packedfile_optional_string_u8([1].as_ref(), 0, &mut index).is_err(), true);
        assert_eq!(index, 0);
    }

    // Check the decoder returns an error for a slice with less than two bytes.
    {
        let mut index = 0;
        assert_eq!(Decoder::decode_packedfile_optional_string_u8([1, 5].as_ref(), 0, &mut index).is_err(), true);
        assert_eq!(index, 0);
    }

    // Check the decoder returns an error for a slice shorter than it's specified length.
    {
        let mut index = 0;
        assert_eq!(Decoder::decode_packedfile_optional_string_u8([1, 4, 0, 2].as_ref(), 0, &mut index).is_err(), true);
        assert_eq!(index, 0);
    }

    // Check the decoder returns an error for a slice with non-UTF8 characters (255).
    {
        let mut index = 0;
        assert_eq!(Decoder::decode_packedfile_optional_string_u8([1, 11, 0, 87, 97, 104, 97, 255, 104, 97, 104, 97, 104, 97].as_ref(), 0, &mut index).is_err(), true);
        assert_eq!(index, 0);
    }
}

/// Test to make sure the u16 optional string specific decoder (`decode_packedfile_optional_string_u16()`)
/// works and fails properly.
#[test]
fn test_decode_packedfile_optional_string_u16() {

    // Check the decoding works for a nonexistent string.
    {
        let mut index = 0;
        assert_eq!(Decoder::decode_packedfile_optional_string_u16([0].as_ref(), 0, &mut index).unwrap(), "".to_owned());
        assert_eq!(index, 1);
    }

    // Check the decoding works for a proper encoded string.
    {
        let mut index = 0;
        assert_eq!(Decoder::decode_packedfile_optional_string_u16([1, 4, 0, 87, 0, 97, 0, 104, 0, 97, 0].as_ref(), 0, &mut index).unwrap(), "Waha".to_owned());
        assert_eq!(index, 11);
    }

    // Check the decoder returns an error for a slice with less than two bytes.
    {
        let mut index = 0;
        assert_eq!(Decoder::decode_packedfile_optional_string_u16([1, 5].as_ref(), 0, &mut index).is_err(), true);
        assert_eq!(index, 0);
    }

    // Check the decoder returns an error for a slice shorter than it's specified length.
    {
        let mut index = 0;
        assert_eq!(Decoder::decode_packedfile_optional_string_u16([1, 4, 0, 2].as_ref(), 0, &mut index).is_err(), true);
        assert_eq!(index, 0);
    }

    // Check the decoder returns an error for a slice with non-UTF16 characters (1, 216, DC01).
    {
        let mut index = 0;
        assert_eq!(Decoder::decode_packedfile_optional_string_u16([1, 4, 0, 87, 0, 97, 0, 1, 216, 97, 0].as_ref(), 0, &mut index).is_err(), true);
        assert_eq!(index, 0);
    }
}
