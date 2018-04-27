// This module contain tests to make sure we don't break any of the encoding/decoding functions in
// a update, as these are used all over the program and if one of them breaks, half of the program
// will break with it. Also, this is the only place where ".unwrap()" will be allowed, as a fail in
// an unwrap means something got broken in the original function.

use coding_helpers::*;

/*
--------------------------------------------------------
            Decoding helpers (Common decoders)
--------------------------------------------------------
*/

/// Test to make sure the u16 integer decoder (`decode_integer_u16()`) works and fails properly.
#[test]
fn test_decode_integer_u16() {

    // Check the decoding works for a proper value.
    assert_eq!(decode_integer_u16(&[10, 0]).unwrap(), 10);

    // Check the decoder returns an error for a slice who's length is different than 2.
    assert_eq!(decode_integer_u16(&[10, 0, 0, 0, 0]).is_err(), true);
}

/// Test to make sure the u32 integer decoder (`decode_integer_u32()`) works and fails properly.
#[test]
fn test_decode_integer_u32() {

    // Check the decoding works for a proper value.
    assert_eq!(decode_integer_u32(&[10, 0, 0, 0]).unwrap(), 10);

    // Check the decoder returns an error for a slice who's length is different than 4.
    assert_eq!(decode_integer_u32(&[10, 0, 0, 0, 0]).is_err(), true);
}

/// Test to make sure the u64 integer decoder (`decode_integer_u64()`) works and fails properly.
#[test]
fn test_decode_integer_u64() {

    // Check the decoding works for a proper value.
    assert_eq!(decode_integer_u64(&[10, 0, 0, 0, 0, 0, 0, 0]).unwrap(), 10);

    // Check the decoder returns an error for a slice who's length is different than 8.
    assert_eq!(decode_integer_u64(&[10, 0, 0, 0, 0]).is_err(), true);
}

/// Test to make sure the i32 integer decoder (`decode_integer_i32()`) works and fails properly.
#[test]
fn test_decode_integer_i32() {

    // Check the decoding works for a proper value.
    assert_eq!(decode_integer_i32(&[10, 0, 0, 0]).unwrap(), 10);

    // Check the decoder returns an error for a slice who's length is different than 4.
    assert_eq!(decode_integer_i32(&[10, 0, 0, 0, 0]).is_err(), true);
}

/// Test to make sure the i64 integer decoder (`decode_integer_i64()`) works and fails properly.
#[test]
fn test_decode_integer_i64() {

    // Check the decoding works for a proper value.
    assert_eq!(decode_integer_i64(&[10, 0, 0, 0, 0, 0, 0, 0]).unwrap(), 10);

    // Check the decoder returns an error for a slice who's length is different than 8.
    assert_eq!(decode_integer_i64(&[10, 0, 0, 0, 0]).is_err(), true);
}


/// Test to make sure the f32 float decoder (`decode_float_f32()`) works and fails properly.
#[test]
fn test_decode_float_f32() {

    // Check the decoding works for a proper value.
    assert_eq!(decode_float_f32(&[0, 0, 32, 65]).unwrap(), 10.0);

    // Check the decoder returns an error for a slice who's length is different than 4.
    assert_eq!(decode_float_f32(&[0, 0, 0, 32, 65]).is_err(), true);
}

/// Test to make sure the u8 string decoder (`decode_string_u8()`) works and fails properly.
#[test]
fn test_decode_string_u8() {

    // Check the decoding works for a proper encoded string.
    assert_eq!(decode_string_u8(&[87, 97, 104, 97, 104, 97, 104, 97, 104, 97]).unwrap(), "Wahahahaha");

    // Check the decoder returns an error for a slice with non-UTF8 characters (255).
    assert_eq!(decode_string_u8(&[87, 97, 104, 97, 255, 104, 97, 104, 97, 104, 97]).is_err(), true);
}

/// Test to make sure the u8 0-padded string decoder (`decode_string_u8_0padded()`) works and fails properly.
#[test]
fn test_decode_string_u8_0padded() {

    // Check the decoding works for a proper encoded string.
    assert_eq!(decode_string_u8_0padded(&[87, 97, 104, 97, 104, 97, 0, 0, 0, 0]).unwrap().0, "Wahaha");
    assert_eq!(decode_string_u8_0padded(&[87, 97, 104, 97, 104, 97, 0, 0, 0, 0]).unwrap().1, 10);

    // Check that, as soon as it finds a 0 (null character) the decoding stops.
    assert_eq!(decode_string_u8_0padded(&[87, 97, 104, 97, 0, 104, 97, 0, 0, 0]).unwrap().0, "Waha");
    assert_eq!(decode_string_u8_0padded(&[87, 97, 104, 97, 0, 104, 97, 0, 0, 0]).unwrap().1, 10);

    // Check the decoder returns an error for a slice with non-UTF8 characters (255).
    assert_eq!(decode_string_u8_0padded(&[87, 97, 104, 97, 255, 104, 97, 104, 97, 104, 97]).is_err(), true);
}

/// Test to make sure the u16 string decoder (`decode_string_u16()`) works and fails properly.
#[test]
fn test_decode_string_u16() {

    // Check the decoding works for a proper encoded string.
    assert_eq!(decode_string_u16(&[87, 0, 97, 0, 104, 0, 97, 0, 104, 0, 97, 0]).unwrap(), "Wahaha");

    // Check the decoder returns an error for a slice with non-UTF8 characters (216).
    assert_eq!(decode_string_u16(&[87, 0, 0, 216, 104, 0, 97, 0, 104, 0, 97, 0]).is_err(), true);
}


/// Test to make sure the boolean decoder (`decode_bool()`) works and fails properly.
#[test]
fn test_decode_bool() {

    // Check that a normal boolean value is decoded properly.
    assert_eq!(decode_bool(0).unwrap(), false);
    assert_eq!(decode_bool(1).unwrap(), true);

    // Check that trying to decode a non-boolean value returns an error.
    assert_eq!(decode_bool(2).is_err(), true);
}

/*
--------------------------------------------------------
            Encoding helpers (Common encoders)
--------------------------------------------------------
*/

/// Test to make sure the u16 integer encoder (`encode_integer_u16()`) works properly.
#[test]
fn test_encode_integer_u16() {

    // Check the encoder works properly.
    assert_eq!(encode_integer_u16(258), vec![2, 1]);
}

/// Test to make sure the u32 integer encoder (`encode_integer_u32()`) works properly.
#[test]
fn test_encode_integer_u32() {

    // Check the encoder works properly.
    assert_eq!(encode_integer_u32(258), vec![2, 1, 0, 0]);
}

/// Test to make sure the u64 integer encoder (`encode_integer_u64()`) works properly.
#[test]
fn test_encode_integer_u64() {

    // Check the encoder works properly.
    assert_eq!(encode_integer_u64(258), vec![2, 1, 0, 0, 0, 0, 0, 0]);
}

/// Test to make sure the i32 integer encoder (`encode_integer_i32()`) works properly.
#[test]
fn test_encode_integer_i32() {

    // Check the encoder works properly.
    assert_eq!(encode_integer_i32(-258), vec![254, 254, 255, 255]);
}

/// Test to make sure the i64 integer encoder (`encode_integer_i64()`) works properly.
#[test]
fn test_encode_integer_i64() {

    // Check the encoder works properly.
    assert_eq!(encode_integer_i64(-258), vec![254, 254, 255, 255, 255, 255, 255, 255]);
}

/// Test to make sure the f64 float encoder (`encode_float_f32()`) works properly.
#[test]
fn test_encode_float_f32() {

    // Check the encoder works properly.
    assert_eq!(encode_float_f32(-10.2), vec![51, 51, 35, 193]);
}

/// Test to make sure the u8 string encoder (`encode_string_u8()`) works properly.
#[test]
fn test_encode_string_u8() {

    // Check the encoder works for a proper encoded string.
    assert_eq!(encode_string_u8("Wahahahaha"), vec![87, 97, 104, 97, 104, 97, 104, 97, 104, 97]);
}

/// Test to make sure the u8 0-padded string encoder (`encode_string_u8_0padded()`) works and fails properly.
#[test]
fn test_encode_string_u8_0padded() {

    // Check the encoder works for a proper encoded string.
    assert_eq!(encode_string_u8_0padded(&("Waha".to_owned(), 8)).unwrap(), vec![87, 97, 104, 97, 0, 0, 0, 0]);

    // Check the encoder fails properly when the lenght it's inferior to the current string's lenght.
    assert_eq!(encode_string_u8_0padded(&("Waha".to_owned(), 3)).is_err(), true);
}

/// Test to make sure the u16 string encoder (`encode_string_u16()`) works properly.
#[test]
fn test_encode_string_u16() {

    // Check the encoder works for a proper encoded string.
    assert_eq!(encode_string_u16("Wahaha"), vec![87, 0, 97, 0, 104, 0, 97, 0, 104, 0, 97, 0]);
}

/// Test to make sure the boolean encoder (`encode_bool()`) works properly.
#[test]
fn test_encode_bool() {

    // Check the encoder works for a boolean.
    assert_eq!(encode_bool(true), 1);
    assert_eq!(encode_bool(false), 0);
}

/*
--------------------------------------------------------
          Decoding helpers (Specific decoders)
--------------------------------------------------------
*/

/// Test to make sure the u16 integer specific decoder (`decode_packedfile_integer_u16()`) works
/// and fails properly.
#[test]
fn test_decode_packedfile_integer_u16() {

    // Check the decoding works for a proper value.
    {
        let mut index = 0;
        assert_eq!(decode_packedfile_integer_u16(&[10, 0], &mut index).unwrap(), 10);
        assert_eq!(index, 2);
    }

    // Check the decoder returns an error for a slice whose lenght is other than 2.
    {
        let mut index = 0;
        assert_eq!(decode_packedfile_integer_u16(&[10], &mut index).is_err(), true);
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
        assert_eq!(decode_packedfile_integer_u32(&[10, 0, 0, 0], &mut index).unwrap(), 10);
        assert_eq!(index, 4);
    }


    // Check the decoder returns an error for a slice whose lenght is other than 4.
    {
        let mut index = 0;
        assert_eq!(decode_packedfile_integer_u32(&[10, 0], &mut index).is_err(), true);
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
        assert_eq!(decode_packedfile_integer_u64(&[10, 0, 0, 0, 0, 0, 0, 0], &mut index).unwrap(), 10);
        assert_eq!(index, 8);
    }


    // Check the decoder returns an error for a slice whose lenght is other than 8.
    {
        let mut index = 0;
        assert_eq!(decode_packedfile_integer_u64(&[10, 0], &mut index).is_err(), true);
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
        assert_eq!(decode_packedfile_integer_i32(&[254, 254, 255, 255], &mut index).unwrap(), -258);
        assert_eq!(index, 4);
    }

    // Check the decoder returns an error for a slice whose lenght is other than 4.
    {
        let mut index = 0;
        assert_eq!(decode_packedfile_integer_i32(&[10, 0], &mut index).is_err(), true);
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
        assert_eq!(decode_packedfile_integer_i64(&[254, 254, 255, 255, 255, 255, 255, 255], &mut index).unwrap(), -258);
        assert_eq!(index, 8);
    }


    // Check the decoder returns an error for a slice whose lenght is other than 8.
    {
        let mut index = 0;
        assert_eq!(decode_packedfile_integer_i64(&[10, 0], &mut index).is_err(), true);
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
        assert_eq!(decode_packedfile_float_f32(&[51, 51, 35, 193], &mut index).unwrap(), -10.2);
        assert_eq!(index, 4);
    }

    // Check the decoder returns an error for a slice whose lenght is other than 4.
    {
        let mut index = 0;
        assert_eq!(decode_packedfile_float_f32(&[10, 0], &mut index).is_err(), true);
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
        assert_eq!(decode_packedfile_string_u8(&[10, 0, 87, 97, 104, 97, 104, 97, 104, 97, 104, 97], &mut index).unwrap(), "Wahahahaha".to_owned());
        assert_eq!(index, 12);
    }

    // Check the decoder returns an error for a slice with less than two bytes.
    {
        let mut index = 0;
        assert_eq!(decode_packedfile_string_u8(&[5], &mut index).is_err(), true);
        assert_eq!(index, 0);
    }

    // Check the decoder returns an error for a slice shorter than it's specified lenght.
    {
        let mut index = 0;
        assert_eq!(decode_packedfile_string_u8(&[4, 0, 2], &mut index).is_err(), true);
        assert_eq!(index, 0);
    }

    // Check the decoder returns an error for a slice with non-UTF8 characters (255).
    {
        let mut index = 0;
        assert_eq!(decode_packedfile_string_u8(&[11, 0, 87, 97, 104, 97, 255, 104, 97, 104, 97, 104, 97], &mut index).is_err(), true);
        assert_eq!(index, 0);
    }
}

/// Test to make sure the u8 optional string specific decoder (`decode_packedfile_optional_string_u8()`)
/// works and fails properly.
#[test]
fn test_decode_packedfile_optional_string_u8() {

    // Check the decoding works for a non-existant string.
    {
        let mut index = 0;
        assert_eq!(decode_packedfile_optional_string_u8(&[0], &mut index).unwrap(), "".to_owned());
        assert_eq!(index, 1);
    }

    // Check the decoding works for a proper encoded string.
    {
        let mut index = 0;
        assert_eq!(decode_packedfile_optional_string_u8(&[1, 10, 0, 87, 97, 104, 97, 104, 97, 104, 97, 104, 97], &mut index).unwrap(), "Wahahahaha".to_owned());
        assert_eq!(index, 13);
    }

    // Check the decoder returns an error for a slice with less than two bytes.
    {
        let mut index = 0;
        assert_eq!(decode_packedfile_optional_string_u8(&[1, 5], &mut index).is_err(), true);
        assert_eq!(index, 0);
    }

    // Check the decoder returns an error for a slice shorter than it's specified lenght.
    {
        let mut index = 0;
        assert_eq!(decode_packedfile_optional_string_u8(&[1, 4, 0, 2], &mut index).is_err(), true);
        assert_eq!(index, 0);
    }

    // Check the decoder returns an error for a slice with non-UTF8 characters (255).
    {
        let mut index = 0;
        assert_eq!(decode_packedfile_optional_string_u8(&[1, 11, 0, 87, 97, 104, 97, 255, 104, 97, 104, 97, 104, 97], &mut index).is_err(), true);
        assert_eq!(index, 0);
    }
}

/// Test to make sure the u16 string specific decoder (`decode_packedfile_string_u16()`) works
/// and fails properly.
#[test]
fn test_decode_packedfile_string_u16() {

    // Check the decoding works for a proper encoded string.
    {
        let mut index = 0;
        assert_eq!(decode_packedfile_string_u16(&[4, 0, 87, 0, 97, 0, 104, 0, 97, 0], &mut index).unwrap(), "Waha".to_owned());
        assert_eq!(index, 10);
    }

    // Check the decoder returns an error for a slice with less than two bytes.
    {
        let mut index = 0;
        assert_eq!(decode_packedfile_string_u16(&[5], &mut index).is_err(), true);
        assert_eq!(index, 0);
    }

    // Check the decoder returns an error for a slice shorter than it's specified lenght.
    {
        let mut index = 0;
        assert_eq!(decode_packedfile_string_u16(&[4, 0, 2], &mut index).is_err(), true);
        assert_eq!(index, 0);
    }

    // Check the decoder returns an error for a slice with non-UTF16 characters (1, 216, DC01).
    {
        let mut index = 0;
        assert_eq!(decode_packedfile_string_u16(&[4, 0, 87, 0, 97, 0, 1, 216, 97, 0], &mut index).is_err(), true);
        assert_eq!(index, 0);
    }
}

/// Test to make sure the u16 optional string specific decoder (`decode_packedfile_optional_string_u16()`)
/// works and fails properly.
#[test]
fn test_decode_packedfile_optional_string_u16() {

    // Check the decoding works for a non-existant string.
    {
        let mut index = 0;
        assert_eq!(decode_packedfile_optional_string_u16(&[0], &mut index).unwrap(), "".to_owned());
        assert_eq!(index, 1);
    }

    // Check the decoding works for a proper encoded string.
    {
        let mut index = 0;
        assert_eq!(decode_packedfile_optional_string_u16(&[1, 4, 0, 87, 0, 97, 0, 104, 0, 97, 0], &mut index).unwrap(), "Waha".to_owned());
        assert_eq!(index, 11);
    }

    // Check the decoder returns an error for a slice with less than two bytes.
    {
        let mut index = 0;
        assert_eq!(decode_packedfile_optional_string_u16(&[1, 5], &mut index).is_err(), true);
        assert_eq!(index, 0);
    }

    // Check the decoder returns an error for a slice shorter than it's specified lenght.
    {
        let mut index = 0;
        assert_eq!(decode_packedfile_optional_string_u16(&[1, 4, 0, 2], &mut index).is_err(), true);
        assert_eq!(index, 0);
    }

    // Check the decoder returns an error for a slice with non-UTF16 characters (1, 216, DC01).
    {
        let mut index = 0;
        assert_eq!(decode_packedfile_optional_string_u16(&[1, 4, 0, 87, 0, 97, 0, 1, 216, 97, 0], &mut index).is_err(), true);
        assert_eq!(index, 0);
    }
}

/// Test to make sure the boolean specific decoder (`decode_packedfile_bool()`) works and fails properly.
#[test]
fn test_decode_packedfile_bool() {

    // Check that a normal boolean value is decoded properly.
    {
        let mut index = 0;
        assert_eq!(decode_packedfile_bool(0, &mut index).unwrap(), false);
        assert_eq!(index, 1);
        assert_eq!(decode_packedfile_bool(1, &mut index).unwrap(), true);
        assert_eq!(index, 2);
    }

    // Check that trying to decode a non-boolean value returns an error.
    {
        let mut index = 0;
        assert_eq!(decode_packedfile_bool(2, &mut index).is_err(), true);
        assert_eq!(index, 0);
    }
}

/*
--------------------------------------------------------
          Encoding helpers (Specific encoders)
--------------------------------------------------------
*/

/// Test to make sure the u8 string specific encoder (`encode_packedfile_string_u8()`) works properly.
#[test]
fn test_encode_packedfile_string_u8() {

    // Check the encoder works for a proper encoded string.
    assert_eq!(encode_packedfile_string_u8("Wahaha"), vec![6, 0, 87, 97, 104, 97, 104, 97]);
}

/// Test to make sure the u8 optional string specific encoder (`encode_packedfile_optional_string_u8()`)
/// works properly.
#[test]
fn test_encode_packedfile_optional_string_u8() {

    // Check the encoder works for a proper encoded string.
    assert_eq!(encode_packedfile_optional_string_u8("Wahaha"), vec![1, 6, 0, 87, 97, 104, 97, 104, 97]);
}

/// Test to make sure the u16 string specific encoder (`encode_packedfile_string_u16()`) works properly.
#[test]
fn test_encode_packedfile_string_u16() {

    // Check the encoder works for a proper encoded string.
    assert_eq!(encode_packedfile_string_u16("Waha"), vec![4, 0, 87, 0, 97, 0, 104, 0, 97, 0]);
}

/// Test to make sure the u16 optional string specific encoder (`encode_packedfile_optional_string_u16()`)
/// works properly.
#[test]
fn test_encode_packedfile_optional_string_u16() {

    // Check the encoder works for a proper encoded string.
    assert_eq!(encode_packedfile_optional_string_u16("Waha"), vec![1, 4, 0, 87, 0, 97, 0, 104, 0, 97, 0]);
}
