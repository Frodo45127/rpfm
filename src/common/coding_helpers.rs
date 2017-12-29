// In this file are all the helper functions used by the encoding/decoding PackedFiles process.
// As we may or may not use them, all functions here should have the "#[allow(dead_code)]"
// var set, so the compiler doesn't spam us every time we try to compile.

extern crate unescape;
extern crate byteorder;

use std::error;
use std::io::{
    Error, ErrorKind
};
use std::char::{
    decode_utf16, REPLACEMENT_CHARACTER
};

use self::byteorder::{
    ByteOrder, ReadBytesExt, LittleEndian
};

/*
--------------------------------------------------------
            Decoding helpers (Common decoders)
--------------------------------------------------------
*/
// TODO: Implement decoder for 0-padded strings (strings with a "max" size, and all the bytes from their end to
// that max size is 00)

/// This function allow us to decode an UTF-16 encoded integer. This type of Integers are encoded in
/// in 2 bytes reversed (LittleEndian).
#[allow(dead_code)]
pub fn decode_integer_u16(integer_encoded: Vec<u8>) -> Result<u16, Error> {
    (&integer_encoded[..]).read_u16::<LittleEndian>()
}

/// This function allow us to decode an UTF-32 encoded integer. This type of Integers are encoded in
/// in 4 bytes reversed (LittleEndian).
#[allow(dead_code)]
pub fn decode_integer_u32(integer_encoded: Vec<u8>) -> Result<u32, Error> {
    (&integer_encoded[..]).read_u32::<LittleEndian>()
}

/// This function allow us to decode an UTF-32 encoded float. This type of floats are encoded in
/// in 4 bytes reversed (LittleEndian).
#[allow(dead_code)]
pub fn decode_float_u32(float_encoded: Vec<u8>) -> Result<f32, Error> {
    (&float_encoded[..]).read_f32::<LittleEndian>()
}

/// This function allow us to decode an UTF-8 encoded String.
#[allow(dead_code)]
pub fn decode_string_u8(string_encoded: Vec<u8>) -> Result<String, Error> {
    match String::from_utf8(string_encoded) {
        Ok(string) => Ok(string),
        Err(error) => Err(Error::new(ErrorKind::Other, error::Error::description(&error).to_string())),
    }
}

/// This function allow us to decode an (0-Padded) UTF-8 encoded String. This type of String has a
/// fixed size and, when the chars ends, it's filled with \u{0} bytes. Also, due to how we are going
/// to decode them, this type of decoding cannot fail, but it's slower than a normal UTF-8 String decoding.
/// We use a tuple to store them and his size.
#[allow(dead_code)]
pub fn decode_string_u8_0padded(string_encoded: Vec<u8>) -> (String, usize) {
    let mut string_decoded = String::new();
    let size = string_encoded.len();
    for character in string_encoded.iter() {
        if (*character as char).escape_unicode().to_string() != ("\\u{0}") {
            string_decoded.push(*character as char);
        }
    }
    (string_decoded, size)
}

/// This function allow us to decode an UTF-16 encoded String. This type of Strings are encoded in
/// in 2 bytes reversed (LittleEndian). Also, this is extremely slow. Needs a lot of improvements.
///
/// NOTE: We return error if the length has returned an error. If a char return an error, we just replace
///       it, but return success.
#[allow(dead_code)]
pub fn decode_string_u16(string_encoded: Vec<u8>) -> Result<String, Error> {
    let mut string_decoded: String = String::new();
    let mut offset: usize = 0;

    for _ in 0..(string_encoded.len() / 2) {
        match decode_integer_u16(string_encoded[offset..offset + 2].to_vec()) {
            Ok(character_u16) => {
                let character = decode_utf16(vec![character_u16]
                        .iter()
                        .cloned())
                    .map( | r | r.unwrap_or(REPLACEMENT_CHARACTER))
                    .collect::<Vec<_>>();
                string_decoded.push_str(&character[0].escape_debug().to_string());
                offset += 2;
            }
            Err(error) => return Err(error)
        }
    }
    Ok(string_decoded)
}

/// This function allow us to decode an encoded boolean. This is simple: \u{0} is false, \u{1} is true.
/// It only uses a byte.
#[allow(dead_code)]
pub fn decode_bool(bool_encoded: u8) -> Result<bool, Error> {
    let bool_decoded = (bool_encoded as char).escape_unicode().to_string();

    match &*bool_decoded {
        "\\u{0}" => Ok(false),
        "\\u{1}" => Ok(true),
        _ => Err(Error::new(ErrorKind::Other, "Error:\nTrying to decode a non-boolean value as boolean.")),
    }
}

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
pub fn encode_string_u8(string_decoded: String) -> Vec<u8> {
    let string_encoded = string_decoded.as_bytes().to_vec();
    string_encoded
}

/// This function allow us to encode an UTF-8 decoded 0-padded String. This one requires us to provide a
/// "size", so we encode the String like a normal UTF-8 String and then extend the vector until we
/// reach the desired size.
#[allow(dead_code)]
pub fn encode_string_u8_0padded(string_decoded: (String, usize)) -> Vec<u8> {
    let mut string_encoded = string_decoded.0.as_bytes().to_vec();
    let size = string_decoded.1;
    let extra_zeroes_amount = size - string_encoded.len();
    string_encoded.reserve_exact(extra_zeroes_amount);
    string_encoded
}

/// This function allow us to encode an UTF-16 decoded String. This type of Strings are encoded in
/// in 2 bytes reversed (LittleEndian).
/// TODO: Improve this.
#[allow(dead_code)]
pub fn encode_string_u16(string_decoded: String) -> Vec<u8> {
    let mut string_encoded: Vec<u8> = vec![];

    // First we need to "unescape" all the escaped chars in the decoding process, so we write them
    // instead \n, \",...
    let string_decoded_unescaped = unescape::unescape(&string_decoded).unwrap();
    let string_decoded_length = string_decoded_unescaped.chars().count() as u16;
    let mut string_decoded_length_encoded = encode_integer_u16(string_decoded_length);
    string_encoded.append(&mut string_decoded_length_encoded);

    for i in 0..string_decoded_length {
        let mut character_u16_buffer = [0; 1];
        let character_u16 = string_decoded_unescaped.chars().nth(i as usize).unwrap().encode_utf16(&mut character_u16_buffer);
        let mut character_u8 = encode_integer_u16(character_u16[0]);
        string_encoded.append(&mut character_u8);
    }
    string_encoded
}

/// This function allow us to encode a boolean. This is simple: \u{0} is false, \u{1} is true.
/// It only uses a byte.
#[allow(dead_code)]
pub fn encode_bool(bool_decoded: bool) -> Vec<u8> {
    let mut bool_encoded: Vec<u8> = vec![];
    if bool_decoded {
        bool_encoded.extend_from_slice(("\u{1}").as_bytes());
    }
    else {
        bool_encoded.extend_from_slice(("\u{0}").as_bytes());
    }
    bool_encoded
}

/*
--------------------------------------------------------
          Decoding helpers (Specific decoders)
--------------------------------------------------------
*/

/// This function allow us to decode an UTF-32 encoded integer cell. We return the integer and the index
/// for the next cell's data.
#[allow(dead_code)]
pub fn decode_packedfile_integer_u32(packed_file_data: Vec<u8>, mut index: usize) -> Result<(u32, usize), Error> {
    if packed_file_data.len() >= (index + 4) {
        match decode_integer_u32(packed_file_data[index..(index + 4)].to_vec()) {
            Ok(number) => {
                index += 4;
                Ok((number, index))
            }
            Err(error) => Err(error)
        }
    }
    else {
        return Err(Error::new(ErrorKind::Other, format!("Error: Index out of bounds. Probably some error in the master_schema.")))
    }
}

/// This function allow us to decode an UTF-32 encoded float cell. We return the float and the index
/// for the next cell's data.
#[allow(dead_code)]
pub fn decode_packedfile_float_u32(packed_file_data: Vec<u8>, mut index: usize) -> Result<(f32, usize), Error> {
    if packed_file_data.len() >= (index + 4) {
        match decode_float_u32(packed_file_data[index..(index + 4)].to_vec()) {
            Ok(number) => {
                index += 4;
                Ok((number, index))
            }
            Err(error) => Err(error)
        }
    }
    else {
        return Err(Error::new(ErrorKind::Other, format!("Error: Index out of bounds. Probably some error in the master_schema.")))
    }
}

/// This function allow us to decode an UTF-8 encoded string cell. We return the string and the
/// index for the next cell's data.
#[allow(dead_code)]
pub fn decode_packedfile_string_u8(packed_file_data: Vec<u8>, mut index: usize) -> Result<(String, usize), Error> {
    if packed_file_data.len() >= (index + 2) {
        match decode_integer_u16(packed_file_data[index..(index + 2)].to_vec()) {
            Ok(size) => {
                index += 2;
                if packed_file_data.len() >= (index + size as usize) {
                    match decode_string_u8(packed_file_data[index..(index + size as usize)].to_vec()) {
                        Ok(string) => {
                            index += size as usize;
                            Ok((string, index))
                        }
                        Err(error) => Err(error)
                    }
                }
                else {
                    return Err(Error::new(ErrorKind::Other, format!("Error: Inddex out of bounds. Probably some error in the master_schema.")))
                }
            }
            Err(error) => Err(error)
        }
    }
    else {
        return Err(Error::new(ErrorKind::Other, format!("Error: Index aout of bounds. Probably some error in the master_schema.")))
    }
}

/// This function allow us to decode an UTF-8 encoded optional string cell. We return the string (or
/// an empty string if it doesn't exist) and the index for the next cell's data.
///
/// NOTE: These strings's first byte it's a boolean that indicates if the string has something.
#[allow(dead_code)]
pub fn decode_packedfile_optional_string_u8(packed_file_data: Vec<u8>, index: usize) -> Result<(String, usize), Error> {
    if packed_file_data.len() >= (index) {
        match decode_packedfile_bool(packed_file_data[index], index) {
            Ok(result) => {
                if result.0 {
                    match decode_packedfile_string_u8(packed_file_data, result.1) {
                        Ok(result) => Ok(result),
                        Err(error) => Err(Error::new(ErrorKind::Other, error::Error::description(&error).to_string())),
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
        return Err(Error::new(ErrorKind::Other, format!("Error: Insdex out of bounds. Probably some error in the master_schema.")))
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

/// This function allow us to encode to Vec<u8> an u32 cell. We return the Vec<u8>.
#[allow(dead_code)]
pub fn encode_packedfile_integer_u32(integer_u32_decoded: u32) -> Vec<u8> {
    let integer_u32_encoded = encode_integer_u32(integer_u32_decoded);
    integer_u32_encoded
}

/// This function allow us to encode to Vec<u8> an f32 cell. We return the Vec<u8>.
#[allow(dead_code)]
pub fn encode_packedfile_float_u32(float_f32_decoded: f32) -> Vec<u8> {
    let float_f32_encoded = encode_float_u32(float_f32_decoded);
    float_f32_encoded
}

/// This function allow us to encode an UTF-8 decoded string cell. We return the Vec<u8> of
/// the encoded string.
#[allow(dead_code)]
pub fn encode_packedfile_string_u8(string_u8_decoded: String) -> Vec<u8> {
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
pub fn encode_packedfile_optional_string_u8(optional_string_u8_decoded: String) -> Vec<u8> {
    let mut optional_string_u8_encoded = vec![];

    if optional_string_u8_decoded.is_empty() {
        optional_string_u8_encoded.append(&mut encode_bool(false));
    }
    else {
        let mut optional_string_u8_data = encode_string_u8(optional_string_u8_decoded);
        let mut optional_string_u8_lenght = encode_integer_u16(optional_string_u8_data.len() as u16);

        optional_string_u8_encoded.append(&mut encode_bool(true));
        optional_string_u8_encoded.append(&mut optional_string_u8_lenght);
        optional_string_u8_encoded.append(&mut optional_string_u8_data);
    }

    optional_string_u8_encoded
}

/// This function allow us to encode to Vec<u8> a boolean cell. We return the Vec<u8>.
#[allow(dead_code)]
pub fn encode_packedfile_bool(bool_decoded: bool) -> Vec<u8> {
    let bool_encoded = encode_bool(bool_decoded);
    bool_encoded
}