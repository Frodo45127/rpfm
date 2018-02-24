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

/// Test to make sure the u16 integer decoder (`decode_integer_u16()`) works properly.
#[test]
fn test_decode_integer_u16() {

    // Check the decoding works for a proper value.
    assert_eq!(decode_integer_u16(&[10, 0]).unwrap(), 10);
}

/// Test to make sure the u16 integer decoder (`decode_integer_u16()`) fails properly.
#[test]
#[should_panic]
fn test_error_decode_integer_u16() {

    // Check the decoder returns an error for a slice who's length is different than 2.
    assert_eq!(decode_integer_u16(&[10, 0, 0, 0, 0]).unwrap(), 10);
}

/// Test to make sure the u32 integer decoder (`decode_integer_u32()`) works properly.
#[test]
fn test_decode_integer_u32() {

    // Check the decoding works for a proper value.
    assert_eq!(decode_integer_u32(&[10, 0, 0, 0]).unwrap(), 10);
}

/// Test to make sure the u32 integer decoder (`decode_integer_u32()`) fails properly.
#[test]
#[should_panic]
fn test_error_decode_integer_u32() {

    // Check the decoder returns an error for a slice who's length is different than 4.
    assert_eq!(decode_integer_u32(&[10, 0, 0, 0, 0]).unwrap(), 10);
}

/// Test to make sure the u64 integer decoder (`decode_integer_u64()`) works properly.
#[test]
fn test_decode_integer_u64() {

    // Check the decoding works for a proper value.
    assert_eq!(decode_integer_u64(&[10, 0, 0, 0, 0, 0, 0, 0]).unwrap(), 10);
}

/// Test to make sure the u64 integer decoder (`decode_integer_u64()`) fails properly.
#[test]
#[should_panic]
fn test_error_decode_integer_u64() {

    // Check the decoder returns an error for a slice who's length is different than 8.
    assert_eq!(decode_integer_u64(&[10, 0, 0, 0, 0]).unwrap(), 10);
}

/// Test to make sure the i32 integer decoder (`decode_integer_i32()`) works properly.
#[test]
fn test_decode_integer_i32() {

    // Check the decoding works for a proper value.
    assert_eq!(decode_integer_i32(&[10, 0, 0, 0]).unwrap(), 10);
}

/// Test to make sure the i32 integer decoder (`decode_integer_i32()`) fails properly.
#[test]
#[should_panic]
fn test_error_decode_integer_i32() {

    // Check the decoder returns an error for a slice who's length is different than 4.
    assert_eq!(decode_integer_i32(&[10, 0, 0, 0, 0]).unwrap(), 10);
}

/// Test to make sure the i64 integer decoder (`decode_integer_i64()`) works properly.
#[test]
fn test_decode_integer_i64() {

    // Check the decoding works for a proper value.
    assert_eq!(decode_integer_i64(&[10, 0, 0, 0, 0, 0, 0, 0]).unwrap(), 10);
}

/// Test to make sure the i64 integer decoder (`decode_integer_i64()`) fails properly.
#[test]
#[should_panic]
fn test_error_decode_integer_i64() {

    // Check the decoder returns an error for a slice who's length is different than 8.
    assert_eq!(decode_integer_i64(&[10, 0, 0, 0, 0]).unwrap(), 10);
}

/// Test to make sure the f32 float decoder (`decode_float_u32()`) works properly.
#[test]
fn test_decode_float_f32() {

    // Check the decoding works for a proper value.
    assert_eq!(decode_float_f32(&[0, 0, 32, 65]).unwrap(), 10.0);
}

/// Test to make sure the f32 float decoder (`decode_float_u64()`) fails properly.
#[test]
#[should_panic]
fn test_error_decode_float_f32() {

    // Check the decoder returns an error for a slice who's length is different than 4.
    assert_eq!(decode_float_f32(&[0, 0, 0, 32, 65]).unwrap(), 10.0);
}

/// Test to make sure the u8 string decoder (`decode_string_u8()`) works properly.
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


/*
--------------------------------------------------------
            Encoding helpers (Common decoders)
--------------------------------------------------------
*/
/// This function allow us to encode an UTF-16 decoded Integer. This type of Integers are encoded in
/// in 2 bytes reversed (LittleEndian).
#[allow(dead_code)]
pub fn encode_integer_u16(integer_decoded: u16) -> Vec<u8> {
    let mut integer_encoded: [u8;2] = [0;2];
    LittleEndian::write_u16(&mut integer_encoded, integer_decoded);
    integer_encoded.to_vec()
}

/// This function allow us to encode an UTF-32 decoded Integer. This type of Integers are encoded in
/// in 4 bytes reversed (LittleEndian).
#[allow(dead_code)]
pub fn encode_integer_u32(integer_decoded: u32) -> Vec<u8> {
    let mut integer_encoded: [u8;4] = [0;4];
    LittleEndian::write_u32(&mut integer_encoded, integer_decoded);
    integer_encoded.to_vec()
}

/// This function allow us to encode a decoded Long Integer. This type of Integers are encoded in
/// in 8 bytes reversed (LittleEndian).
#[allow(dead_code)]
pub fn encode_integer_u64(integer_decoded: u64) -> Vec<u8> {
    let mut integer_encoded: [u8;8] = [0;8];
    LittleEndian::write_u64(&mut integer_encoded, integer_decoded);
    integer_encoded.to_vec()
}


/// This function allow us to encode an signed UTF-32 decoded Integer. This type of Integers are encoded in
/// in 4 bytes reversed (LittleEndian).
#[allow(dead_code)]
pub fn encode_integer_i32(integer_decoded: i32) -> Vec<u8> {
    let mut integer_encoded: [u8;4] = [0;4];
    LittleEndian::write_i32(&mut integer_encoded, integer_decoded);
    integer_encoded.to_vec()
}

/// This function allow us to encode a signed decoded Long Integer. This type of Integers are encoded in
/// in 8 bytes reversed (LittleEndian).
#[allow(dead_code)]
pub fn encode_integer_i64(integer_decoded: i64) -> Vec<u8> {
    let mut integer_encoded: [u8;8] = [0;8];
    LittleEndian::write_i64(&mut integer_encoded, integer_decoded);
    integer_encoded.to_vec()
}

/// This function allow us to encode an UTF-32 decoded Float. This type of Floats are encoded in
/// in 4 bytes reversed (LittleEndian).
#[allow(dead_code)]
pub fn encode_float_u32(float_decoded: f32) -> Vec<u8> {
    let mut float_encoded: [u8;4] = [0;4];
    LittleEndian::write_f32(&mut float_encoded, float_decoded);
    float_encoded.to_vec()
}

/// This function allow us to encode an UTF-8 decoded String.
#[allow(dead_code)]
pub fn encode_string_u8(string_decoded: &str) -> Vec<u8> {
    string_decoded.as_bytes().to_vec()
}

/// This function allow us to encode an UTF-8 decoded 0-padded String. This one requires us to provide a
/// "size", so we encode the String like a normal UTF-8 String and then extend the vector until we
/// reach the desired size.
#[allow(dead_code)]
pub fn encode_string_u8_0padded(string_decoded: &(String, usize)) -> Result<Vec<u8>, Error> {
    let mut string_encoded = string_decoded.0.as_bytes().to_vec();
    let size = string_decoded.1;
    if string_encoded.len() <= size {
        let extra_zeroes_amount = size - string_encoded.len();
        for _ in 0..extra_zeroes_amount {
            string_encoded.push(0);
        }
        Ok(string_encoded)
    }
    else {
        Err(format_err!("Error: String \"{}\" has a lenght of {} chars, but his max length should be {}).", string_decoded.0, string_encoded.len(), size))
    }
}

/// This function allow us to encode an UTF-16 decoded String. This type of Strings are encoded in
/// in 2 bytes reversed (LittleEndian).
#[allow(dead_code)]
pub fn encode_string_u16(string_decoded: &str) -> Vec<u8> {
    let mut string_encoded: Vec<u8> = vec![];
    string_decoded.encode_utf16().for_each(|character| string_encoded.append(&mut encode_integer_u16(character)));
    string_encoded
}

/// This function allow us to encode a boolean. This is simple: \u{0} is false, \u{1} is true.
/// It only uses a byte.
#[allow(dead_code)]
pub fn encode_bool(bool_decoded: bool) -> u8 {
    if bool_decoded { 1 } else { 0 }
}

/*
--------------------------------------------------------
          Decoding helpers (Specific decoders)
--------------------------------------------------------
*/

/// This function allow us to decode an UTF-16 encoded integer cell. We return the integer and the index
/// for the next cell's data.
#[allow(dead_code)]
pub fn decode_packedfile_integer_u16(packed_file_data: &[u8], mut index: usize) -> Result<(u16, usize), Error> {
    if packed_file_data.len() >= 2 {
        match decode_integer_u16(&packed_file_data[..2]) {
            Ok(number) => {
                index += 2;
                Ok((number, index))
            }
            Err(error) => Err(error)
        }
    }
    else {
        Err(format_err!("Error decoding an u16: Index \"{}\" out of bounds (Max length: {}).", index, packed_file_data.len()))
    }
}

/// This function allow us to decode an UTF-32 encoded integer cell. We return the integer and the index
/// for the next cell's data.
#[allow(dead_code)]
pub fn decode_packedfile_integer_u32(packed_file_data: &[u8], mut index: usize) -> Result<(u32, usize), Error> {
    if packed_file_data.len() >= 4 {
        match decode_integer_u32(&packed_file_data[..4]) {
            Ok(number) => {
                index += 4;
                Ok((number, index))
            }
            Err(error) => Err(error)
        }
    }
    else {
        Err(format_err!("Error decoding an u32: Index \"{}\" out of bounds (Max length: {}).", index, packed_file_data.len()))
    }
}

/// This function allow us to decode an encoded Long Integer cell. We return the integer and the index
/// for the next cell's data.
#[allow(dead_code)]
pub fn decode_packedfile_integer_u64(packed_file_data: &[u8], mut index: usize) -> Result<(u64, usize), Error> {
    if packed_file_data.len() >= 8 {
        match decode_integer_u64(&packed_file_data[..8]) {
            Ok(number) => {
                index += 8;
                Ok((number, index))
            }
            Err(error) => Err(error)
        }
    }
    else {
        Err(format_err!("Error decoding an u64: Index \"{}\" out of bounds (Max length: {}).", index, packed_file_data.len()))
    }
}

/// This function allow us to decode a signed UTF-32 encoded integer cell. We return the integer and the index
/// for the next cell's data.
#[allow(dead_code)]
pub fn decode_packedfile_integer_i32(packed_file_data: &[u8], mut index: usize) -> Result<(i32, usize), Error> {
    if packed_file_data.len() >= 4 {
        match decode_integer_i32(&packed_file_data[..4]) {
            Ok(number) => {
                index += 4;
                Ok((number, index))
            }
            Err(error) => Err(error)
        }
    }
    else {
        Err(format_err!("Error decoding an i32: Index \"{}\" out of bounds (Max length: {}).", index, packed_file_data.len()))
    }
}

/// This function allow us to decode a signed encoded Long Integer cell. We return the integer and the index
/// for the next cell's data.
#[allow(dead_code)]
pub fn decode_packedfile_integer_i64(packed_file_data: &[u8], mut index: usize) -> Result<(i64, usize), Error> {
    if packed_file_data.len() >= 8 {
        match decode_integer_i64(&packed_file_data[..8]) {
            Ok(number) => {
                index += 8;
                Ok((number, index))
            }
            Err(error) => Err(error)
        }
    }
    else {
        Err(format_err!("Error decoding an i64: Index \"{}\" out of bounds (Max length: {}).", index, packed_file_data.len()))
    }
}

/// This function allow us to decode an UTF-32 encoded float cell. We return the float and the index
/// for the next cell's data.
#[allow(dead_code)]
pub fn decode_packedfile_float_u32(packed_file_data: &[u8], mut index: usize) -> Result<(f32, usize), Error> {
    if packed_file_data.len() >= 4 {
        match decode_float_u32(&packed_file_data[..4]) {
            Ok(number) => {
                index += 4;
                Ok((number, index))
            }
            Err(error) => Err(error)
        }
    }
    else {
        Err(format_err!("Error decoding a f32: Index \"{}\" out of bounds (Max length: {}).", index, packed_file_data.len()))
    }
}

/// This function allow us to decode an UTF-8 encoded string cell. We return the string and the
/// index for the next cell's data.
#[allow(dead_code)]
pub fn decode_packedfile_string_u8(packed_file_data: &[u8], index: usize) -> Result<(String, usize), Error> {
    if packed_file_data.len() >= 2 {
        match decode_packedfile_integer_u16(&packed_file_data[..2], index) {
            Ok(result) => {
                let size = result.0;
                let mut index = result.1;
                if packed_file_data.len() >= (size as usize + 2) {
                    match decode_string_u8(&packed_file_data[2..(2 + size as usize)]) {
                        Ok(string) => {
                            index += size as usize;
                            Ok((string, index))
                        }
                        Err(error) => Err(error)
                    }
                }
                else {
                    Err(format_err!("Error decoding an u8 String: Index \"{}\" out of bounds (Max length: {}).", index, packed_file_data.len()))
                }
            }
            Err(error) => Err(error)
        }
    }
    else {
        Err(format_err!("Error decoding an u16 (String size): Index \"{}\" out of bounds (Max length: {}).", index, packed_file_data.len()))
    }
}

/// This function allow us to decode an UTF-8 encoded optional string cell. We return the string (or
/// an empty string if it doesn't exist) and the index for the next cell's data.
///
/// NOTE: These strings's first byte it's a boolean that indicates if the string has something.
#[allow(dead_code)]
pub fn decode_packedfile_optional_string_u8(packed_file_data: &[u8], index: usize) -> Result<(String, usize), Error> {
    if packed_file_data.len() >= 1 {
        match decode_packedfile_bool(packed_file_data[0], index) {
            Ok(result) => {
                let exist = result.0;
                let index = result.1;
                if exist {
                    match decode_packedfile_string_u8(&packed_file_data[1..], index) {
                        Ok(result) => Ok(result),
                        Err(error) => Err(error),
                    }
                }
                else {
                    Ok((String::new(), result.1))
                }
            }
            Err(error) => Err(error)
        }
    }
    else {
        Err(format_err!("Error decoding an u8 Optional String: Index \"{}\" out of bounds (Max length: {}).", index, packed_file_data.len()))
    }
}

/// This function allow us to decode an UTF-16 encoded string cell. We return the string and the
/// index for the next cell's data.
#[allow(dead_code)]
pub fn decode_packedfile_string_u16(packed_file_data: &[u8], index: usize) -> Result<(String, usize), Error> {
    if packed_file_data.len() >= 2 {
        match decode_packedfile_integer_u16(&packed_file_data[..2], index) {
            Ok(result) => {
                // We wrap this to avoid overflow, as the limit of this is 65,535.
                let size = result.0.wrapping_mul(2);
                let mut index = result.1;
                if packed_file_data.len() >= (size as usize + 2) {
                    match decode_string_u16(&packed_file_data[2..(2 + size as usize)]) {
                        Ok(string) => {
                            index += size as usize;
                            Ok((string, index))
                        }
                        Err(error) => Err(error)
                    }
                }
                else {
                    Err(format_err!("Error decoding an u8 String: Index \"{}\" out of bounds (Max length: {}).", index, packed_file_data.len()))
                }
            }
            Err(error) => Err(error)
        }
    }
    else {
        Err(format_err!("Error decoding an u16 (String size): Index \"{}\" out of bounds (Max length: {}).", index, packed_file_data.len()))
    }
}

/// This function allow us to decode an UTF-16 encoded optional string cell. We return the string (or
/// an empty string if it doesn't exist) and the index for the next cell's data.
///
/// NOTE: These strings's first byte it's a boolean that indicates if the string has something.
#[allow(dead_code)]
pub fn decode_packedfile_optional_string_u16(packed_file_data: &[u8], index: usize) -> Result<(String, usize), Error> {
    if packed_file_data.len() >= 1 {
        match decode_packedfile_bool(packed_file_data[0], index) {
            Ok(result) => {
                let exist = result.0;
                let index = result.1;
                if exist {
                    match decode_packedfile_string_u16(&packed_file_data[1..], index) {
                        Ok(result) => Ok(result),
                        Err(error) => Err(error),
                    }
                }
                else {
                    Ok((String::new(), result.1))
                }
            }
            Err(error) => Err(error)
        }
    }
    else {
        Err(format_err!("Error decoding an u8 Optional String: Index \"{}\" out of bounds (Max length: {}).", index, packed_file_data.len()))
    }
}

/// This function allow us to decode a boolean cell. We return the boolean's value and the index
/// for the next cell's data.
#[allow(dead_code)]
pub fn decode_packedfile_bool(packed_file_data: u8, mut index: usize) -> Result<(bool, usize), Error> {
    match decode_bool(packed_file_data) {
        Ok(value) => {
            index += 1;
            Ok((value, index))
        }
        Err(error) => Err(error)
    }
}

/*
--------------------------------------------------------
          Encoding helpers (Specific decoders)
--------------------------------------------------------
*/

/// This function allow us to encode an UTF-8 decoded string cell. We return the Vec<u8> of
/// the encoded string.
#[allow(dead_code)]
pub fn encode_packedfile_string_u8(string_u8_decoded: &str) -> Vec<u8> {
    let mut string_u8_encoded = vec![];
    let mut string_u8_data = encode_string_u8(string_u8_decoded);
    let mut string_u8_lenght = encode_integer_u16(string_u8_data.len() as u16);

    string_u8_encoded.append(&mut string_u8_lenght);
    string_u8_encoded.append(&mut string_u8_data);

    string_u8_encoded
}

/// This function allow us to encode an UTF-8 decoded string cell. We return the Vec<u8> of
/// the encoded string.
#[allow(dead_code)]
pub fn encode_packedfile_optional_string_u8(optional_string_u8_decoded: &str) -> Vec<u8> {
    let mut optional_string_u8_encoded = vec![];

    if optional_string_u8_decoded.is_empty() {
        optional_string_u8_encoded.push(encode_bool(false));
    }
    else {
        let mut optional_string_u8_data = encode_string_u8(optional_string_u8_decoded);
        let mut optional_string_u8_lenght = encode_integer_u16(optional_string_u8_data.len() as u16);

        optional_string_u8_encoded.push(encode_bool(true));
        optional_string_u8_encoded.append(&mut optional_string_u8_lenght);
        optional_string_u8_encoded.append(&mut optional_string_u8_data);
    }

    optional_string_u8_encoded
}

/// This function allow us to encode an UTF-16 decoded string cell. We return the Vec<u8> of
/// the encoded string.
#[allow(dead_code)]
pub fn encode_packedfile_string_u16(string_u16_decoded: &str) -> Vec<u8> {
    let mut string_u16_encoded = vec![];
    let mut string_u16_data = encode_string_u16(string_u16_decoded);
    let mut string_u16_lenght = encode_integer_u16(string_u16_data.len() as u16 / 2);

    string_u16_encoded.append(&mut string_u16_lenght);
    string_u16_encoded.append(&mut string_u16_data);

    string_u16_encoded
}

/// This function allow us to encode an UTF-8 decoded string cell. We return the Vec<u8> of
/// the encoded string.
#[allow(dead_code)]
pub fn encode_packedfile_optional_string_u16(optional_string_u16_decoded: &str) -> Vec<u8> {
    let mut optional_string_u16_encoded = vec![];

    if optional_string_u16_decoded.is_empty() {
        optional_string_u16_encoded.push(encode_bool(false));
    }
    else {
        let mut optional_string_u16_data = encode_string_u16(optional_string_u16_decoded);
        let mut optional_string_u16_lenght = encode_integer_u16(optional_string_u16_data.len() as u16);

        optional_string_u16_encoded.push(encode_bool(true));
        optional_string_u16_encoded.append(&mut optional_string_u16_lenght);
        optional_string_u16_encoded.append(&mut optional_string_u16_data);
    }

    optional_string_u16_encoded
}*/