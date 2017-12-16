// In this file are all the helper functions used by the encoding/decoding PackedFiles process.
// As we may or may not use them, all functions here should have the "#[allow(dead_code)]"
// var set, so the compiler doesn't spam us every time we try to compile.

extern crate unescape;
extern crate byteorder;

use std::char;

use self::byteorder::{
    ByteOrder, ReadBytesExt, LittleEndian
};

/*
--------------------------------------------------------
            Decoding helpers (Common decoders)
--------------------------------------------------------
*/

/// This function allow us to decode an UTF-16 encoded integer. This type of Integers are encoded in
/// in 2 bytes reversed (LittleEndian).
#[allow(dead_code)]
pub fn decode_integer_u16(integer_encoded: Vec<u8>) -> u16 {
    let integer_decoded: u16 = (&integer_encoded[..]).read_u16::<LittleEndian>().unwrap();
    integer_decoded
}

/// This function allow us to decode an UTF-32 encoded integer. This type of Integers are encoded in
/// in 4 bytes reversed (LittleEndian).
#[allow(dead_code)]
pub fn decode_integer_u32(integer_encoded: Vec<u8>) -> u32 {
    let integer_decoded: u32 = (&integer_encoded[..]).read_u32::<LittleEndian>().unwrap();
    integer_decoded
}

/// This function allow us to decode an UTF-32 encoded float. This type of floats are encoded in
/// in 4 bytes reversed (LittleEndian).
#[allow(dead_code)]
pub fn decode_float_u32(float_encoded: Vec<u8>) -> f32 {
    let float_decoded: f32 = (&float_encoded[..]).read_f32::<LittleEndian>().unwrap();
    float_decoded
}

/// This function allow us to decode an UTF-8 encoded String.
#[allow(dead_code)]
pub fn decode_string_u8(string_encoded: Vec<u8>) -> String {
    let string_decoded = string_encoded.iter().map(|&c| {c as char}).collect();
    string_decoded
}

/// This function allow us to decode an UTF-16 encoded String. This type of Strings are encoded in
/// in 2 bytes reversed (LittleEndian). Also, this is extremely slow. Needs a lot of improvements.
#[allow(dead_code)]
pub fn decode_string_u16(string_encoded: Vec<u8>) -> String {
    let mut string_decoded: String = String::new();
    let mut offset: usize = 0;

    for _ in 0..(string_encoded.len() / 2) {
        let character_u16: u16 = (&string_encoded[offset..offset + 2]).read_u16::<LittleEndian>().unwrap();
        let character_u16 = char::decode_utf16(vec![character_u16]
                .iter()
                .cloned())
            .map( | r | r.map_err( | e | e.unpaired_surrogate()))
            .collect::< Vec < _ >> ();
        let character_decoded = character_u16[0].unwrap().escape_debug().to_string();

        string_decoded.push_str(&character_decoded);
        offset += 2;
    }
    string_decoded
}

/// This function allow us to decode an encoded boolean. This is simple: \u{0} is false, \u{1} is true.
/// It only uses a byte.
#[allow(dead_code)]
pub fn decode_bool(bool_encoded: u8) -> bool {
    let bool_decoded: bool;
    if (bool_encoded as char).escape_unicode().to_string() == ("\\u{1}") {
        bool_decoded = true;
    }
    else {
        bool_decoded = false;
    }
    bool_decoded
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
pub fn decode_packedfile_integer_u32(packed_file_data: Vec<u8>, mut index: usize) -> (u32, usize) {
    let number = decode_integer_u32(packed_file_data[index..(index + 4)].to_vec());
    index += 4;
    (number, index)
}

/// This function allow us to decode an UTF-32 encoded float cell. We return the float and the index
/// for the next cell's data.
#[allow(dead_code)]
pub fn decode_packedfile_float_u32(packed_file_data: Vec<u8>, mut index: usize) -> (f32, usize) {
    let number = decode_float_u32(packed_file_data[index..(index + 4)].to_vec());
    index += 4;
    (number, index)
}

/// This function allow us to decode an UTF-8 encoded string cell. We return the string and the
/// index for the next cell's data.
#[allow(dead_code)]
pub fn decode_packedfile_string_u8(packed_file_data: Vec<u8>, mut index: usize) -> (String, usize) {
    let size: usize = decode_integer_u16(packed_file_data[index..(index + 2)].to_vec()) as usize;
    index += 2;
    let string = decode_string_u8(packed_file_data[index..(index + size)].to_vec());
    index += size;
    (string, index)
}

/// This function allow us to decode an UTF-8 encoded optional string cell. We return the string (or
/// an empty string if it doesn't exist) and the index for the next cell's data.
///
/// NOTE: These strings's first byte it's a boolean that indicates if the string has something.
#[allow(dead_code)]
pub fn decode_packedfile_optional_string_u8(packed_file_data: Vec<u8>, mut index: usize) -> (String, usize) {
    let exists = decode_bool(packed_file_data[index]);
    index += 1;
    if exists {
        decode_packedfile_string_u8(packed_file_data, index)
    } else {
        let string = String::new();
        (string, index)
    }
}

/// This function allow us to decode a boolean cell. We return the boolean's value and the index
/// for the next cell's data.
#[allow(dead_code)]
pub fn decode_packedfile_bool(packed_file_data: Vec<u8>, mut index: usize) -> (bool, usize) {
    let is_true = decode_bool(packed_file_data[index]);
    index += 1;
    (is_true, index)
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