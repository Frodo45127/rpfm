// In this file are all the helper functions used by the encoding/decoding PackedFiles process.
// As we may or may not use them, all functions here should have the "#[allow(dead_code)]"
// var set, so the compiler doesn't spam us every time we try to compile.

extern crate unescape;
extern crate byteorder;

use std::char;

use self::byteorder::{
    ReadBytesExt, BigEndian
};

/*
--------------------------------------------------------
                    Decoding helpers
--------------------------------------------------------
*/

/// This function allow us to decode an UTF-16 encoded String. This type of Strings are encoded in
/// in 2 bytes reversed. Also, this is extremely slow. Needs a lot of improvements.
pub fn decode_string_u16(string_encoded: Vec<u8>) -> String {
    let mut string_decoded: String = String::new();
    let mut offset: usize = 0;

    for _ in 0..(string_encoded.len() / 2) {
        let mut character_encoded: Vec<u8> = string_encoded[offset..offset + 2].into();
        character_encoded.reverse();
        let character_u16: u16 = (&character_encoded[..]).read_u16::<BigEndian>().unwrap();
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


/// This function allow us to decode the size of an UTF-16 encoded String. His size are 2 UTF-16 bytes
/// at the start of the string.
pub fn decode_size_string_u16(mut string_size_encoded: Vec<u8>) -> u16 {
    string_size_encoded.reverse();
    let string_size_decoded: u16 = (&string_size_encoded[..]).read_u16::<BigEndian>().unwrap();
    string_size_decoded
}

/*
--------------------------------------------------------
                    Encoding helpers
--------------------------------------------------------
*/

/// This function allow us to encode an UTF-16 decoded String. This type of Strings are encoded in
/// in 2 bytes reversed.
pub fn encode_string_u16(string_decoded: String) -> Vec<u8> {
    let mut string_encoded: Vec<u8> = vec![];

    // First we need to "unescape" all the escaped chars in the decoding process, so we write them
    // instead \n, \",...
    let string_decoded_unescaped = unescape::unescape(&string_decoded).unwrap();
    let string_decoded_length = string_decoded_unescaped.chars().count() as u16;
    let string_decoded_length_encoded = ::common::u16_to_u8_reverse(string_decoded_length);
    string_encoded.append(&mut string_decoded_length_encoded.to_vec());

    for i in 0..string_decoded_length {
        let mut character_u16_buffer = [0; 1];
        let character_u16 = string_decoded_unescaped.chars().nth(i as usize).unwrap().encode_utf16(&mut character_u16_buffer);
        let mut character_u8 = ::common::u16_to_u8_reverse(character_u16[0]).to_vec();
        string_encoded.append(&mut character_u8);
    }
    string_encoded
}

/// This function allow us to encode a boolean. This is simple: \u{0} is false, \u{1} is true.
/// It only uses a byte.
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
