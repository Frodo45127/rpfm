//---------------------------------------------------------------------------//
// Copyright (c) 2017-2019 Ismael Gutiérrez González. All rights reserved.
// 
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
// 
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

// In this file are all the helper functions used by the encoding/decoding PackedFiles process.
// As we may or may not use them, all functions here should have the "#[allow(dead_code)]"
// var set, so the compiler doesn't spam us every time we try to compile.
//
// Common helpers are used to just decode/encode data.
// Specific helpers are used to decode/encode data, returning the position from where continue to
// decode/encode. These are used specially in PackedFiles.
//
// Note: the specific decoders return tuples with (value, index of the new thing to decode).

use byteorder::{ByteOrder, LittleEndian};
use encoding::{Encoding, DecoderTrap};
use encoding::all::ISO_8859_1;

use rpfm_error::{Error, ErrorKind, Result};

//-----------------------------------------------------//
//          Decoding helpers (Common decoders)
//-----------------------------------------------------//

/// Common helper. This function allows us to decode an encoded boolean. This is simple: 0 is false, 1 is true. It only uses a byte.
#[allow(dead_code)]
pub fn decode_bool(bool_encoded: u8) -> Result<bool> {
    match bool_encoded {
        0 => Ok(false),
        1 => Ok(true),
        _ => Err(ErrorKind::HelperDecodingEncodingError(format!("<p>Error trying to decode \"{}\" as boolean.</p>", bool_encoded)))?,
    }
}

/// Common helper. This function allows us to decode an u16 encoded integer.
#[allow(dead_code)]
pub fn decode_integer_u16(integer_encoded: &[u8]) -> Result<u16> {
    match integer_encoded.len() {
        2 => Ok(LittleEndian::read_u16(integer_encoded)),
        _ => Err(ErrorKind::HelperDecodingEncodingError(format!("<p>Error trying to decode an u16 number:</p><ul><li>Required bytes: 2.</li><li>Provided bytes: {}.</li></ul>", integer_encoded.len())))?
    }

}

/// Common helper. This function allows us to decode an u32 encoded integer.
#[allow(dead_code)]
pub fn decode_integer_u32(integer_encoded: &[u8]) -> Result<u32> {
    match integer_encoded.len() {
        4 => Ok(LittleEndian::read_u32(integer_encoded)),
        _ => Err(ErrorKind::HelperDecodingEncodingError(format!("<p>Error trying to decode an u32 number:</p><ul><li>Required bytes: 4.</li><li>Provided bytes: {}.</li></ul>", integer_encoded.len())))?
    }
}

/// Common helper. This function allows us to decode an u64 encoded integer.
#[allow(dead_code)]
pub fn decode_integer_u64(integer_encoded: &[u8]) -> Result<u64> {
    match integer_encoded.len() {
        8 => Ok(LittleEndian::read_u64(integer_encoded)),
        _ => Err(ErrorKind::HelperDecodingEncodingError(format!("<p>Error trying to decode an u64 number:</p><ul><li>Required bytes: 8.</li><li>Provided bytes: {}.</li></ul>", integer_encoded.len())))?
    }
}

/// Common helper. This function allows us to decode an i32 encoded integer.
#[allow(dead_code)]
pub fn decode_integer_i32(integer_encoded: &[u8]) -> Result<i32> {
    match integer_encoded.len() {
        4 => Ok(LittleEndian::read_i32(integer_encoded)),
        _ => Err(ErrorKind::HelperDecodingEncodingError(format!("<p>Error trying to decode an i32 number:</p><ul><li>Required bytes: 4.</li><li>Provided bytes: {}.</li></ul>", integer_encoded.len())))?
    }
}

/// Common helper. This function allows us to decode an i64 encoded integer.
#[allow(dead_code)]
pub fn decode_integer_i64(integer_encoded: &[u8]) -> Result<i64> {
    match integer_encoded.len() {
        8 => Ok(LittleEndian::read_i64(integer_encoded)),
        _ => Err(ErrorKind::HelperDecodingEncodingError(format!("<p>Error trying to decode an i64 number:</p><ul><li>Required bytes: 8.</li><li>Provided bytes: {}.</li></ul>", integer_encoded.len())))?
    }
}

/// Common helper. This function allows us to decode a f32 encoded float.
#[allow(dead_code)]
pub fn decode_float_f32(float_encoded: &[u8]) -> Result<f32> {
    match float_encoded.len() {
        4 => Ok(LittleEndian::read_f32(float_encoded)),
        _ => Err(ErrorKind::HelperDecodingEncodingError(format!("<p>Error trying to decode an f32 number:</p><ul><li>Required bytes: 4.</li><li>Provided bytes: {}.</li></ul>", float_encoded.len())))?
    }
}

/// Common helper. This function allows us to decode an UTF-8 encoded String.
#[allow(dead_code)]
pub fn decode_string_u8(string_encoded: &[u8]) -> Result<String> {
    String::from_utf8(string_encoded.to_vec()).map_err(|_| Error::from(ErrorKind::HelperDecodingEncodingError("<p>Error trying to decode an UTF-8 String.</p>".to_owned())))
}

/// Common helper. This function allows us to decode an ISO_8859_1 encoded String.
#[allow(dead_code)]
pub fn decode_string_u8_iso_8859_1(string_encoded: &[u8]) -> Result<String> {
    ISO_8859_1.decode(string_encoded, DecoderTrap::Replace).map(|x| x.to_string()).map_err(|_| Error::from(ErrorKind::HelperDecodingEncodingError("<p>Error trying to decode an UTF-8 String.</p>".to_owned())))
}

/// Common helper. This function allows us to decode a 00-Padded UTF-8 encoded String. This type of String has a
/// fixed size and, when the characters end, it's filled with `00` bytes until it reach his size.
/// We return the decoded String and his full size when encoded.
#[allow(dead_code)]
pub fn decode_string_u8_0padded(string_encoded: &[u8]) -> Result<(String, usize)> {
    let mut string_encoded_without_0 = vec![];
    for character in string_encoded.iter() {
        match *character {
            0 => break,
            _ => string_encoded_without_0.push(*character)
        }
    }
    let string_decoded = String::from_utf8(string_encoded_without_0).map_err(|_| Error::from(ErrorKind::HelperDecodingEncodingError("<p>Error trying to decode an UTF-8 0-Padded String.</p>".to_owned())))?;
    Ok((string_decoded, string_encoded.len()))
}

/// Common helper. This function allows us to decode a 00-Terminated UTF-8 encoded String. This type of String has 
/// a 00 byte at his end and variable size. It advances the provided offset while decoding. We return the decoded String.
#[allow(dead_code)]
pub fn decode_string_u8_0terminated(string_encoded: &[u8], offset: &mut usize) -> Result<String> {
    let mut string = String::new();
    let mut index = 0;
    loop {
        let character = *string_encoded.get(index).ok_or_else(|| Error::from(ErrorKind::HelperDecodingEncodingError("<p>Error trying to decode an UTF-8 0-Terminated String.</p>".to_owned())))?;
        index += 1;
        if character == 0 { break; }
        string.push(character as char);
    }
    *offset += index;
    Ok(string)
}

/// Common helper. This function allows us to decode an UTF-16 encoded String.
#[allow(dead_code)]
pub fn decode_string_u16(string_encoded: &[u8]) -> Result<String> {
    let mut u16_characters = vec![];
    let mut offset: usize = 0;
    for _ in 0..(string_encoded.len() / 2) {
        u16_characters.push(decode_integer_u16(&string_encoded[offset..offset + 2]).unwrap());
        offset += 2;
    }

    String::from_utf16(&u16_characters).map_err(|_| Error::from(ErrorKind::HelperDecodingEncodingError("<p>Error trying to decode an UTF-16 String.</p>".to_owned())))
}

//-----------------------------------------------------//
//          Encoding helpers (Common encoders)
//-----------------------------------------------------//

/// Common helper. This function allows us to encode a boolean. This is simple: 0 is false, 1 is true. It only uses a byte.
#[allow(dead_code)]
pub fn encode_bool(bool_decoded: bool) -> u8 {
    if bool_decoded { 1 } else { 0 }
}

/// Common helper. This function allows us to encode an u16 decoded integer.
#[allow(dead_code)]
pub fn encode_integer_u16(integer_decoded: u16) -> Vec<u8> {
    let mut integer_encoded: [u8;2] = [0;2];
    LittleEndian::write_u16(&mut integer_encoded, integer_decoded);
    integer_encoded.to_vec()
}

/// Common helper. This function allows us to encode an u32 decoded integer.
#[allow(dead_code)]
pub fn encode_integer_u32(integer_decoded: u32) -> Vec<u8> {
    let mut integer_encoded: [u8;4] = [0;4];
    LittleEndian::write_u32(&mut integer_encoded, integer_decoded);
    integer_encoded.to_vec()
}

/// Common helper. This function allows us to encode an u64 decoded integer.
#[allow(dead_code)]
pub fn encode_integer_u64(integer_decoded: u64) -> Vec<u8> {
    let mut integer_encoded: [u8;8] = [0;8];
    LittleEndian::write_u64(&mut integer_encoded, integer_decoded);
    integer_encoded.to_vec()
}


/// Common helper. This function allows us to encode an i32 decoded integer.
#[allow(dead_code)]
pub fn encode_integer_i32(integer_decoded: i32) -> Vec<u8> {
    let mut integer_encoded: [u8;4] = [0;4];
    LittleEndian::write_i32(&mut integer_encoded, integer_decoded);
    integer_encoded.to_vec()
}

/// Common helper. This function allows us to encode an i64 decoded integer.
#[allow(dead_code)]
pub fn encode_integer_i64(integer_decoded: i64) -> Vec<u8> {
    let mut integer_encoded: [u8;8] = [0;8];
    LittleEndian::write_i64(&mut integer_encoded, integer_decoded);
    integer_encoded.to_vec()
}

/// Common helper. This function allows us to encode a f32 decoded Float.
#[allow(dead_code)]
pub fn encode_float_f32(float_decoded: f32) -> Vec<u8> {
    let mut float_encoded: [u8;4] = [0;4];
    LittleEndian::write_f32(&mut float_encoded, float_decoded);
    float_encoded.to_vec()
}

/// Common helper. This function allows us to encode an UTF-8 decoded String.
#[allow(dead_code)]
pub fn encode_string_u8(string_decoded: &str) -> Vec<u8> {
    string_decoded.as_bytes().to_vec()
}

/// Common helper. This function allows us to encode a 00-Padded UTF-8 decoded String.
///
/// This one is a bit special. It's uses a tuple with the String to encode and the total size of the encoded string.
/// So... we just encode the String as a normal string, then add 0 until we reach the desired size. If the String is
/// longer than the size, we throw an error.
#[allow(dead_code)]
pub fn encode_string_u8_0padded(string_decoded: &(String, usize)) -> Result<Vec<u8>> {
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
        Err(ErrorKind::HelperDecodingEncodingError(format!("Error trying to encode an UTF-8 0-Padded String: \"{}\" has a lenght of {} chars, but his length should be less or equal than {}.", string_decoded.0, string_encoded.len(), size)))?
    }
}

/// Common helper. This function allows us to encode an UTF-16 decoded String.
#[allow(dead_code)]
pub fn encode_string_u16(string_decoded: &str) -> Vec<u8> {
    let mut string_encoded: Vec<u8> = vec![];
    string_decoded.encode_utf16().for_each(|character| string_encoded.append(&mut encode_integer_u16(character)));
    string_encoded
}

//-----------------------------------------------------//
//        Decoding helpers (Specific decoders)
//-----------------------------------------------------//

/// Specific helper. This function allows us to decode a boolean, moving the index to the byte where the next data starts.
#[allow(dead_code)]
pub fn decode_packedfile_bool(packed_file_data: u8, index: &mut usize) -> Result<bool> {
    let result = decode_bool(packed_file_data);
    if result.is_ok() { *index += 1; }
    result
}

/// Specific helper. This function allows us to decode an u16 encoded integer, moving the index to the byte where the next data starts.
#[allow(dead_code)]
pub fn decode_packedfile_integer_u16(packed_file_data: &[u8], index: &mut usize) -> Result<u16> {
    let result = decode_integer_u16(packed_file_data);
    if result.is_ok() { *index += 2; }
    result
}

/// Specific helper. This function allows us to decode an u32 encoded integer, moving the index to the byte where the next data starts.
#[allow(dead_code)]
pub fn decode_packedfile_integer_u32(packed_file_data: &[u8], index: &mut usize) -> Result<u32> {
    let result = decode_integer_u32(packed_file_data);
    if result.is_ok() { *index += 4; }
    result
}

/// Specific helper. This function allows us to decode an u64 encoded integer, moving the index to the byte where the next data starts.
#[allow(dead_code)]
pub fn decode_packedfile_integer_u64(packed_file_data: &[u8], index: &mut usize) -> Result<u64> {
    let result = decode_integer_u64(packed_file_data);
    if result.is_ok() { *index += 8; }
    result
}

/// Specific helper. This function allows us to decode an i32 encoded integer, moving the index to the byte where the next data starts.
#[allow(dead_code)]
pub fn decode_packedfile_integer_i32(packed_file_data: &[u8], index: &mut usize) -> Result<i32> {
    let result = decode_integer_i32(packed_file_data);
    if result.is_ok() { *index += 4; }
    result
}

/// Specific helper. This function allows us to decode an i64 encoded integer, moving the index to the byte where the next data starts.
#[allow(dead_code)]
pub fn decode_packedfile_integer_i64(packed_file_data: &[u8], index: &mut usize) -> Result<i64> {
    let result = decode_integer_i64(packed_file_data);
    if result.is_ok() { *index += 8; }
    result
}

/// Specific helper. This function allows us to decode an f32 encoded float, moving the index to the byte where the next data starts.
#[allow(dead_code)]
pub fn decode_packedfile_float_f32(packed_file_data: &[u8], index: &mut usize) -> Result<f32> {
    let result = decode_float_f32(packed_file_data);
    if result.is_ok() { *index += 4; }
    result
}

/// Specific helper. This function allows us to decode an UTF-8 encoded String, moving the index to the byte where the next data starts.
#[allow(dead_code)]
pub fn decode_packedfile_string_u8(packed_file_data: &[u8], mut index: &mut usize) -> Result<String> {
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
            Err(ErrorKind::HelperDecodingEncodingError(format!("<p>Error trying to decode an UTF-8 String:</p><p>Size specified ({}) is bigger than the amount of bytes we have ({}).</p>", string_lenght, packed_file_data.len())))?
        }
    }
    else {
        Err(ErrorKind::HelperDecodingEncodingError(format!("<p>Error trying to decode an UTF-8 String:</p><p>Not enough bytes (only {}, minimum required is 2) to get his size.</p>", packed_file_data.len())))?
    }
}

/// Specific helper. This function allows us to decode an UTF-16 encoded String, moving the index to the byte where the next data starts.
#[allow(dead_code)]
pub fn decode_packedfile_string_u16(packed_file_data: &[u8], mut index: &mut usize) -> Result<String> {
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
            Err(ErrorKind::HelperDecodingEncodingError(format!("<p>Error trying to decode an UTF-16 String:</p><p>Size specified ({}) is bigger than the amount of pairs of bytes we have ({}).</p>", string_lenght_double, packed_file_data.len() / 2)))?
        }
    }
    else {
        Err(ErrorKind::HelperDecodingEncodingError(format!("<p>Error trying to decode an UTF-16 String:</p><p>Not enough bytes (only {}, minimum required is 2) to get his size.</p>", packed_file_data.len())))?
    }
}

/// Specific helper. This function allows us to decode an i32 encoded integer, moving the index to the byte where the next data starts.
///
/// These integer's first byte it's a boolean that indicates if the integer has something. If false, the integer it's just that byte.
/// If true, there is a normal i32 integer after that byte.
#[allow(dead_code)]
pub fn decode_packedfile_optional_integer_i32(packed_file_data: &[u8], mut index: &mut usize) -> Result<Option<i32>> {
    if packed_file_data.get(0).is_some() {
        match decode_packedfile_bool(packed_file_data[0], &mut index) {
            Ok(result) => {
                if result {
                    if packed_file_data.get(4).is_some() {
                        let result = decode_packedfile_integer_i32(&packed_file_data[1..5], &mut index);

                        // Reduce the index in 1, because despite the first byte being a boolean, there has been an error
                        // later in the decoding process, and we want to go back to our original index in that case.
                        if result.is_err() { *index -= 1 };
                        result.map(|x| Some(x))
                    }
                    else {
                        *index -= 1;
                        Err(ErrorKind::HelperDecodingEncodingError(format!("<p>Error trying to decode an Optional I32:</p><ul><li>Required bytes: 5.</li><li>Provided bytes: {}.</li></ul>", packed_file_data[1..].len())))?
                    }
                } else { Ok(None) }

            }
            Err(_) => Err(ErrorKind::HelperDecodingEncodingError(format!("<p>Error trying to decode an Optional I32:</p><p>The first byte is not a boolean.</p>")))?
        }
    }
    else {
        Err(ErrorKind::HelperDecodingEncodingError(format!("<p>Error trying to decode an Optional I32:</p><p>There are no bytes provided to decode.</p>")))?
    }
}

/// Specific helper. This function allows us to decode an i64 encoded integer, moving the index to the byte where the next data starts.
///
/// These integer's first byte it's a boolean that indicates if the integer has something. If false, the integer it's just that byte.
/// If true, there is a normal i64 integer after that byte.
#[allow(dead_code)]
pub fn decode_packedfile_optional_integer_i64(packed_file_data: &[u8], mut index: &mut usize) -> Result<Option<i64>> {
    if packed_file_data.get(0).is_some() {
        match decode_packedfile_bool(packed_file_data[0], &mut index) {
            Ok(result) => {
                if result {
                    if packed_file_data.get(8).is_some() {
                        let result = decode_packedfile_integer_i64(&packed_file_data[1..9], &mut index);

                        // Reduce the index in 1, because despite the first byte being a boolean, there has been an error
                        // later in the decoding process, and we want to go back to our original index in that case.
                        if result.is_err() { *index -= 1 };
                        result.map(|x| Some(x))
                    }
                    else {
                        *index -= 1;
                        Err(ErrorKind::HelperDecodingEncodingError(format!("<p>Error trying to decode an Optional I64:</p><ul><li>Required bytes: 9.</li><li>Provided bytes: {}.</li></ul>", packed_file_data[1..].len())))?
                    }
                } else { Ok(None) }
            }
            Err(_) => Err(ErrorKind::HelperDecodingEncodingError(format!("<p>Error trying to decode an Optional I64:</p><p>The first byte is not a boolean.</p>")))?
        }
    }
    else {
        Err(ErrorKind::HelperDecodingEncodingError(format!("<p>Error trying to decode an Optional I64:</p><p>There are no bytes provided to decode.</p>")))?
    }
}

/// Specific helper. This function allows us to decode an f32 encoded float, moving the index to the byte where the next data starts.
///
/// These float's first byte it's a boolean that indicates if the float has something. If false, the float it's just that byte.
/// If true, there is a normal f32 float after that byte.
#[allow(dead_code)]
pub fn decode_packedfile_optional_float_f32(packed_file_data: &[u8], mut index: &mut usize) -> Result<Option<f32>> {
    if packed_file_data.get(0).is_some() {
        match decode_packedfile_bool(packed_file_data[0], &mut index) {
            Ok(result) => {
                if result {
                    if packed_file_data.get(4).is_some() {
                        let result = decode_packedfile_float_f32(&packed_file_data[1..5], &mut index);

                        // Reduce the index in 1, because despite the first byte being a boolean, there has been an error
                        // later in the decoding process, and we want to go back to our original index in that case.
                        if result.is_err() { *index -= 1 };
                        result.map(|x| Some(x))
                    }
                    else {
                        *index -= 1;
                        Err(ErrorKind::HelperDecodingEncodingError(format!("<p>Error trying to decode an Optional F32:</p><ul><li>Required bytes: 5.</li><li>Provided bytes: {}.</li></ul>", packed_file_data[1..].len())))?
                    }
                } else { Ok(None) }
            }
            Err(_) => Err(ErrorKind::HelperDecodingEncodingError(format!("<p>Error trying to decode an Optional F32:</p><p>The first byte is not a boolean.</p>")))?
        }
    }
    else {
        Err(ErrorKind::HelperDecodingEncodingError(format!("<p>Error trying to decode an Optional F32:</p><p>There are no bytes provided to decode.</p>")))?
    }
}

/// Specific helper. This function allows us to decode an UTF-8 encoded optional String, moving the index to the byte where the next data starts.
///
/// These Strings's first byte it's a boolean that indicates if the string has something. If false, the string it's just that byte.
/// If true, there is a normal UTF-8 encoded String after that byte.
#[allow(dead_code)]
pub fn decode_packedfile_optional_string_u8(packed_file_data: &[u8], mut index: &mut usize) -> Result<String> {
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
            Err(_) => Err(ErrorKind::HelperDecodingEncodingError(format!("<p>Error trying to decode an UTF-8 Optional String:</p><p>The first byte is not a boolean.</p>")))?
        }
    }
    else {
        Err(ErrorKind::HelperDecodingEncodingError(format!("<p>Error trying to decode an UTF-8 Optional String:</p><p>There are no bytes provided to decode.</p>")))?
    }
}

/// Specific helper. This function allows us to decode an UTF-16 encoded optional String, moving the index to the byte where the next data starts.
///
/// These Strings's first byte it's a boolean that indicates if the string has something. If false, the string it's just that byte.
/// If true, there is a normal UTF-16 encoded String after that byte.
#[allow(dead_code)]
pub fn decode_packedfile_optional_string_u16(packed_file_data: &[u8], mut index: &mut usize) -> Result<String> {
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
            Err(_) => Err(ErrorKind::HelperDecodingEncodingError(format!("<p>Error trying to decode an UTF-16 Optional String:</p><p>The first byte is not a boolean.</p>")))?
        }
    }
    else {
        Err(ErrorKind::HelperDecodingEncodingError(format!("<p>Error trying to decode an UTF-16 Optional String:</p><p>There are no bytes provided to decode.</p>")))?
    }
}

//-----------------------------------------------------//
//        Encoding helpers (Specific encoders)
//-----------------------------------------------------//

/// Specific helper. This function allows us to encode an UTF-8 decoded String that requires having his lenght (an u16 integer) encoded before the encoded string.
#[allow(dead_code)]
pub fn encode_packedfile_string_u8(string_u8_decoded: &str) -> Vec<u8> {
    let mut string_u8_encoded = vec![];
    let mut string_u8_data = encode_string_u8(string_u8_decoded);
    let mut string_u8_lenght = encode_integer_u16(string_u8_data.len() as u16);

    string_u8_encoded.append(&mut string_u8_lenght);
    string_u8_encoded.append(&mut string_u8_data);

    string_u8_encoded
}

/// Specific helper. This function allows us to encode an UTF-16 decoded String that requires having his lenght (an u16 integer) encoded before the encoded string.
#[allow(dead_code)]
pub fn encode_packedfile_string_u16(string_u16_decoded: &str) -> Vec<u8> {
    let mut string_u16_encoded = vec![];
    let mut string_u16_data = encode_string_u16(string_u16_decoded);
    let mut string_u16_lenght = encode_integer_u16(string_u16_data.len() as u16 / 2);

    string_u16_encoded.append(&mut string_u16_lenght);
    string_u16_encoded.append(&mut string_u16_data);

    string_u16_encoded
}

/// Specific helper. This function allows us to encode an I32 that requires having a boolean (one byte, true if exists, 
/// false if it's empty) encoded before the encoded integer.
#[allow(dead_code)]
pub fn encode_packedfile_optional_integer_i32(optional_decoded: &Option<i32>) -> Vec<u8> {
    let mut optional_encoded = vec![];
    match optional_decoded {
        Some(value) => {
            optional_encoded.push(encode_bool(true));
            optional_encoded.append(&mut encode_integer_i32(*value));
        }
        None => optional_encoded.push(encode_bool(false)),
    }

    optional_encoded
}

/// Specific helper. This function allows us to encode an I64 that requires having a boolean (one byte, true if exists, 
/// false if it's empty) encoded before the encoded integer.
#[allow(dead_code)]
pub fn encode_packedfile_optional_integer_i64(optional_decoded: &Option<i64>) -> Vec<u8> {
    let mut optional_encoded = vec![];
    match optional_decoded {
        Some(value) => {
            optional_encoded.push(encode_bool(true));
            optional_encoded.append(&mut encode_integer_i64(*value));
        }
        None => optional_encoded.push(encode_bool(false)),
    }

    optional_encoded
}

/// Specific helper. This function allows us to encode an F32 that requires having a boolean (one byte, true if exists, 
/// false if it's empty) encoded before the encoded float.
#[allow(dead_code)]
pub fn encode_packedfile_optional_float_f32(optional_decoded: &Option<f32>) -> Vec<u8> {
    let mut optional_encoded = vec![];
    match optional_decoded {
        Some(value) => {
            optional_encoded.push(encode_bool(true));
            optional_encoded.append(&mut encode_float_f32(*value));
        }
        None => optional_encoded.push(encode_bool(false)),
    }

    optional_encoded
}

/// Specific helper. This function allows us to encode an UTF-8 decoded String that requires having a boolean (one
/// byte, true if exists, false if it's empty) and his lenght (an u16 integer) encoded before the encoded string.
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

/// Specific helper. This function allows us to encode an UTF-16 decoded String that requires having a boolean (one
/// byte, true if exists, false if it's empty) and his lenght (an u16 integer) encoded before the encoded string.
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
