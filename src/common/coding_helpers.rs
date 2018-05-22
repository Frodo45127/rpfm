// In this file are all the helper functions used by the encoding/decoding PackedFiles process.
// As we may or may not use them, all functions here should have the "#[allow(dead_code)]"
// var set, so the compiler doesn't spam us every time we try to compile.
//
// Common helpers are used to just decode/encode data.
// Specific helpers are used to decode/encode data, returning the position from where continue to
// decode/encode. These are used specially in PackedFiles.
//
// Note: the specific decoders return tuples with (value, index of the new thing to decode).

extern crate failure;
extern crate byteorder;
extern crate encoding;

use self::encoding::{Encoding, DecoderTrap};
use self::encoding::all::ISO_8859_1;
use failure::Error;

use self::byteorder::{
    ByteOrder, LittleEndian
};

/*
--------------------------------------------------------
            Decoding helpers (Common decoders)
--------------------------------------------------------
*/

/// This function allow us to decode an UTF-16 encoded integer. This type of Integers are encoded in
/// in 2 bytes reversed (LittleEndian).
#[allow(dead_code)]
pub fn decode_integer_u16(integer_encoded: &[u8]) -> Result<u16, Error> {
    match integer_encoded.len() {
        2 => Ok(LittleEndian::read_u16(integer_encoded)),
        _ => Err(format_err!("Error trying to decode an u16 number.\n\n - Required bytes: 2.\n - Provided bytes: {}", integer_encoded.len()))
    }

}

/// This function allow us to decode an UTF-32 encoded integer. This type of Integers are encoded in
/// in 4 bytes reversed (LittleEndian).
#[allow(dead_code)]
pub fn decode_integer_u32(integer_encoded: &[u8]) -> Result<u32, Error> {
    match integer_encoded.len() {
        4 => Ok(LittleEndian::read_u32(integer_encoded)),
        _ => Err(format_err!("Error trying to decode an u32 number.\n\n - Required bytes: 4.\n - Provided bytes: {}", integer_encoded.len()))
    }
}

/// This function allow us to decode an encoded Long Integer. This type of Integers are encoded in
/// in 8 bytes reversed (LittleEndian).
#[allow(dead_code)]
pub fn decode_integer_u64(integer_encoded: &[u8]) -> Result<u64, Error> {
    match integer_encoded.len() {
        8 => Ok(LittleEndian::read_u64(integer_encoded)),
        _ => Err(format_err!("Error trying to decode an u64 number.\n\n - Required bytes: 8.\n - Provided bytes: {}", integer_encoded.len()))
    }
}

/// This function allow us to decode an signed UTF-32 encoded integer. This type of Integers are encoded in
/// in 4 bytes reversed (LittleEndian).
#[allow(dead_code)]
pub fn decode_integer_i32(integer_encoded: &[u8]) -> Result<i32, Error> {
    match integer_encoded.len() {
        4 => Ok(LittleEndian::read_i32(integer_encoded)),
        _ => Err(format_err!("Error trying to decode an i32 number.\n\n - Required bytes: 4.\n - Provided bytes: {}", integer_encoded.len()))
    }
}

/// This function allow us to decode an signed encoded Long Integer. This type of Integers are encoded in
/// in 8 bytes reversed (LittleEndian).
#[allow(dead_code)]
pub fn decode_integer_i64(integer_encoded: &[u8]) -> Result<i64, Error> {
    match integer_encoded.len() {
        8 => Ok(LittleEndian::read_i64(integer_encoded)),
        _ => Err(format_err!("Error trying to decode an i64 number.\n\n - Required bytes: 8.\n - Provided bytes: {}", integer_encoded.len()))
    }
}

/// This function allow us to decode an UTF-32 encoded float. This type of floats are encoded in
/// in 4 bytes reversed (LittleEndian).
#[allow(dead_code)]
pub fn decode_float_f32(float_encoded: &[u8]) -> Result<f32, Error> {
    match float_encoded.len() {
        4 => Ok(LittleEndian::read_f32(float_encoded)),
        _ => Err(format_err!("Error trying to decode a f32 number.\n\n - Required bytes: 4.\n - Provided bytes: {}", float_encoded.len()))
    }
}

/// This function allow us to decode an UTF-8 encoded String.
#[allow(dead_code)]
pub fn decode_string_u8(string_encoded: &[u8]) -> Result<String, Error> {
    String::from_utf8(string_encoded.to_vec()).map_err(|_| format_err!("Error trying to decode an UTF-8 String."))
}

/// This function allow us to decode an UTF-8 encoded String.
#[allow(dead_code)]
pub fn decode_string_u8_iso_8859_1(string_encoded: &[u8]) -> Result<String, Error> {
    ISO_8859_1.decode(string_encoded, DecoderTrap::Replace).map(|x| x.to_string()).map_err(|_| format_err!("Error trying to decode an UTF-8 String."))
}

/// This function allow us to decode an (0-Padded) UTF-8 encoded String. This type of String has a
/// fixed size and, when the chars ends, it's filled with "0" bytes. We use a tuple to store
/// his text and his size when encoded.
#[allow(dead_code)]
pub fn decode_string_u8_0padded(string_encoded: &[u8]) -> Result<(String, usize), Error> {
    let mut string_encoded_without_0 = vec![];
    for character in string_encoded.iter() {
        match *character {
            0 => break,
            _ => string_encoded_without_0.push(*character)
        }
    }
    let string_decoded = String::from_utf8(string_encoded_without_0).map_err(|_| format_err!("Error trying to decode an UTF-8 0-Padded String."))?;
    Ok((string_decoded, string_encoded.len()))
}

/// This function allow us to decode an UTF-16 encoded String. This type of Strings are encoded in
/// in 2 bytes reversed (LittleEndian).
#[allow(dead_code)]
pub fn decode_string_u16(string_encoded: &[u8]) -> Result<String, Error> {
    let mut u16_characters = vec![];
    let mut offset: usize = 0;
    for _ in 0..(string_encoded.len() / 2) {

        // This unwrap() is allowed, as decoding an u16 can only fail if we don't provide an slice
        // of len() == 2.
        u16_characters.push(decode_integer_u16(&string_encoded[offset..offset + 2]).unwrap());
        offset += 2;
    }

    String::from_utf16(&u16_characters).map_err(|_| format_err!("Error trying to decode an UTF-16 String."))
}

/// This function allow us to decode an encoded boolean. This is simple: 0 is false, 1 is true.
/// It only uses a byte.
#[allow(dead_code)]
pub fn decode_bool(bool_encoded: u8) -> Result<bool, Error> {
    match bool_encoded {
        0 => Ok(false),
        1 => Ok(true),
        _ => Err(format_err!("Error trying to decode \"{}\" as boolean.", bool_encoded)),
    }
}

/*
--------------------------------------------------------
            Encoding helpers (Common encoders)
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
pub fn encode_float_f32(float_decoded: f32) -> Vec<u8> {
    let mut float_encoded: [u8;4] = [0;4];
    LittleEndian::write_f32(&mut float_encoded, float_decoded);
    float_encoded.to_vec()
}

/// This function allow us to encode an UTF-8 decoded String.
#[allow(dead_code)]
pub fn encode_string_u8(string_decoded: &str) -> Vec<u8> {
    string_decoded.as_bytes().to_vec()
}

/// This function allow us to encode an UTF-8 decoded 0-padded String. This one is a bit special.
/// It's uses a tuple with the String to encode and the total size of the encoded string.
/// So... we just encode the String as a normal string, then add 0 until we reach the desired size.
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
        Err(format_err!("Error: String \"{}\" has a lenght of {} chars, but his max length should be less or equal to {}).", string_decoded.0, string_encoded.len(), size))
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

/// This function allow us to encode a boolean. This is simple: 0 is false, 1 is true.
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

/// This function allow us to decode an UTF-16 encoded integer, returning with it the byte where
/// the next thing to decode is.
#[allow(dead_code)]
pub fn decode_packedfile_integer_u16(packed_file_data: &[u8], index: &mut usize) -> Result<u16, Error> {
    let result = decode_integer_u16(packed_file_data);
    if result.is_ok() { *index += 2; }
    result
}

/// This function allow us to decode an UTF-32 encoded integer, returning with it the byte where
/// the next thing to decode is.
#[allow(dead_code)]
pub fn decode_packedfile_integer_u32(packed_file_data: &[u8], index: &mut usize) -> Result<u32, Error> {
    let result = decode_integer_u32(packed_file_data);
    if result.is_ok() { *index += 4; }
    result
}

/// This function allow us to decode an encoded Long Integer (u64), returning with it the byte where
/// the next thing to decode is.
#[allow(dead_code)]
pub fn decode_packedfile_integer_u64(packed_file_data: &[u8], index: &mut usize) -> Result<u64, Error> {
    let result = decode_integer_u64(packed_file_data);
    if result.is_ok() { *index += 8; }
    result
}

/// This function allow us to decode an UTF-32 encoded signed integer, returning with it the byte where
/// the next thing to decode is.
#[allow(dead_code)]
pub fn decode_packedfile_integer_i32(packed_file_data: &[u8], index: &mut usize) -> Result<i32, Error> {
    let result = decode_integer_i32(packed_file_data);
    if result.is_ok() { *index += 4; }
    result
}

/// This function allow us to decode an encoded signed Long Integer (i64), returning with it the byte where
/// the next thing to decode is.
#[allow(dead_code)]
pub fn decode_packedfile_integer_i64(packed_file_data: &[u8], index: &mut usize) -> Result<i64, Error> {
    let result = decode_integer_i64(packed_file_data);
    if result.is_ok() { *index += 8; }
    result
}

/// This function allow us to decode an UTF-32 encoded float, returning with it the byte where
/// the next thing to decode is.
#[allow(dead_code)]
pub fn decode_packedfile_float_f32(packed_file_data: &[u8], index: &mut usize) -> Result<f32, Error> {
    let result = decode_float_f32(packed_file_data);
    if result.is_ok() { *index += 4; }
    result
}

/// This function allow us to decode an UTF-8 encoded String, returning with it the byte where
/// the next thing to decode is.
#[allow(dead_code)]
pub fn decode_packedfile_string_u8(packed_file_data: &[u8], mut index: &mut usize) -> Result<String, Error> {
    if packed_file_data.get(1).is_some() {

        // We have already checked this cannot fail (we have 2 or more bytes), so the unwrap() here is allowed.
        let string_lenght = decode_packedfile_integer_u16(&packed_file_data[..2], &mut index).unwrap() as usize;

        // If the last byte of the string exists, we decode it.
        if packed_file_data.get(string_lenght + 1).is_some() {
            let result = decode_string_u8(&packed_file_data[2..(2 + string_lenght)]);
            if result.is_err() { *index -= 2; } else { *index += string_lenght; }
            result
        }
        else {

            // Reduce the index, to ignore the success of the decoding of the size.
            *index -= 2;
            Err(format_err!("Error trying to decode an u8 String:\n\nSize specified ({}) is bigger than the amount of bytes we have ({}).", string_lenght, packed_file_data.len()))
        }
    }
    else {
        Err(format_err!("Error trying to decode an u8 String:\n\nNot enough bytes (only {}, minimum required is 2) to get his size.", packed_file_data.len()))
    }
}

/// This function allow us to decode an UTF-8 encoded optional String, returning with it the byte where
/// the next thing to decode is.
///
/// NOTE: These strings's first byte it's a boolean that indicates if the string has something.
#[allow(dead_code)]
pub fn decode_packedfile_optional_string_u8(packed_file_data: &[u8], mut index: &mut usize) -> Result<String, Error> {
    if packed_file_data.get(0).is_some() {
        match decode_packedfile_bool(packed_file_data[0], &mut index) {
            Ok(result) => {
                if result {
                    let result = decode_packedfile_string_u8(&packed_file_data[1..], &mut index);

                    // Reduce the index in 1, because despite the first byte being a boolean, there has been an error
                    // later in the decoding process, and we want to go back to our original index in that case.
                    if result.is_err() { *index -= 1 };
                    result
                } else { Ok(String::new()) }

            }
            Err(_) => Err(format_err!("Error trying to decode an u8 Optional String:\n\nThe first byte is not a boolean."))
        }
    }
    else {
        Err(format_err!("Error trying to decode an u8 Optional String:\n\nThere are no bytes provided to decode."))
    }
}

/// This function allow us to decode an UTF-16 encoded String, returning with it the byte where
/// the next thing to decode is.
#[allow(dead_code)]
pub fn decode_packedfile_string_u16(packed_file_data: &[u8], mut index: &mut usize) -> Result<String, Error> {
    if packed_file_data.get(1).is_some() {

        // We have already checked this cannot fail (we have 2 or more bytes), so the unwrap() here is allowed.
        let string_lenght = decode_packedfile_integer_u16(&packed_file_data[..2], &mut index).unwrap() as usize;

        // We wrap this to avoid overflow, as the limit of this is 65,535. We do this because u16 Strings
        // counts pairs of bytes (u16), not single bytes.
        let string_lenght_double = string_lenght.wrapping_mul(2) as usize;

        // If the last byte of the string exists, we decode it.
        if packed_file_data.get(string_lenght_double + 1).is_some() {
            let result = decode_string_u16(&packed_file_data[2..(2 + string_lenght_double)]);
            if result.is_err() { *index -= 2; } else { *index += string_lenght_double; }
            result
        }
        else {

            // Reduce the index, to ignore the success of the decoding of the size.
            *index -= 2;
            Err(format_err!("Error trying to decode an u16 String:\n\nSize specified ({}) is bigger than the amount of pairs of bytes we have ({}).", string_lenght_double, packed_file_data.len() / 2))
        }
    }
    else {
        Err(format_err!("Error trying to decode an u16 String:\n\nNot enough bytes (only {}, minimum required is 2) to get his size.", packed_file_data.len()))
    }
}

/// This function allow us to decode an UTF-16 encoded optional String, returning with it the byte where
/// the next thing to decode is.
///
/// NOTE: These strings's first byte it's a boolean that indicates if the string has something.
#[allow(dead_code)]
pub fn decode_packedfile_optional_string_u16(packed_file_data: &[u8], mut index: &mut usize) -> Result<String, Error> {
    if packed_file_data.get(0).is_some() {
        match decode_packedfile_bool(packed_file_data[0], &mut index) {
            Ok(result) => {
                if result {
                    let result = decode_packedfile_string_u16(&packed_file_data[1..], &mut index);

                    // Reduce the index in 1, because despite the first byte being a boolean, there has been an error
                    // later in the decoding process, and we want to go back to our original index in that case.
                    if result.is_err() { *index -= 1 };
                    result
                } else { Ok(String::new()) }
            }
            Err(_) => Err(format_err!("Error trying to decode an u16 Optional String:\n\nThe first byte is not a boolean."))
        }
    }
    else {
        Err(format_err!("Error trying to decode an u16 Optional String:\n\nThere are no bytes provided to decode."))
    }
}

/// This function allow us to decode a boolean, returning with it the byte where
/// the next thing to decode is.
#[allow(dead_code)]
pub fn decode_packedfile_bool(packed_file_data: u8, index: &mut usize) -> Result<bool, Error> {
    let result = decode_bool(packed_file_data);
    if result.is_ok() { *index += 1; }
    result
}

/*
--------------------------------------------------------
          Encoding helpers (Specific encoders)
--------------------------------------------------------
*/

/// This function allow us to encode an UTF-8 decoded String that requires having his lenght
/// (two bytes, an u16 integer) encoded before the encoded string.
#[allow(dead_code)]
pub fn encode_packedfile_string_u8(string_u8_decoded: &str) -> Vec<u8> {
    let mut string_u8_encoded = vec![];
    let mut string_u8_data = encode_string_u8(string_u8_decoded);
    let mut string_u8_lenght = encode_integer_u16(string_u8_data.len() as u16);

    string_u8_encoded.append(&mut string_u8_lenght);
    string_u8_encoded.append(&mut string_u8_data);

    string_u8_encoded
}

/// This function allow us to encode an UTF-8 decoded String that requires having a boolean (one
/// byte, true if exists, false if it's empty) and his lenght (two bytes, an u16 integer) encoded
/// before the encoded string.
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

/// This function allow us to encode an UTF-16 decoded String that requires having his lenght
/// (two bytes, an u16 integer) encoded before the encoded string.
#[allow(dead_code)]
pub fn encode_packedfile_string_u16(string_u16_decoded: &str) -> Vec<u8> {
    let mut string_u16_encoded = vec![];
    let mut string_u16_data = encode_string_u16(string_u16_decoded);
    let mut string_u16_lenght = encode_integer_u16(string_u16_data.len() as u16 / 2);

    string_u16_encoded.append(&mut string_u16_lenght);
    string_u16_encoded.append(&mut string_u16_data);

    string_u16_encoded
}

/// This function allow us to encode an UTF-16 decoded String that requires having a boolean (one
/// byte, true if exists, false if it's empty) and his lenght (two bytes, an u16 integer) encoded
/// before the encoded string.
#[allow(dead_code)]
pub fn encode_packedfile_optional_string_u16(optional_string_u16_decoded: &str) -> Vec<u8> {
    let mut optional_string_u16_encoded = vec![];

    if optional_string_u16_decoded.is_empty() {
        optional_string_u16_encoded.push(encode_bool(false));
    }
    else {
        let mut optional_string_u16_data = encode_string_u16(optional_string_u16_decoded);
        let mut optional_string_u16_lenght = encode_integer_u16(optional_string_u16_data.len() as u16 / 2);

        optional_string_u16_encoded.push(encode_bool(true));
        optional_string_u16_encoded.append(&mut optional_string_u16_lenght);
        optional_string_u16_encoded.append(&mut optional_string_u16_data);
    }

    optional_string_u16_encoded
}
