// Here are all the helper functions needed to decode and encode entries in a DB PackedFile.
// TODO: Move all this stuff to the coding helpers file.
extern crate byteorder;

use self::byteorder::{
    ByteOrder, BigEndian, WriteBytesExt
};
/*
--------------------------------------------------------
                   Decoding helpers
--------------------------------------------------------
*/

/// This function allow us to decode an UTF-8 encoded string cell. We return the string and the
/// index for the next cell's data.
pub fn decode_string_u8(packed_file_data: Vec<u8>, mut index: usize) -> (String, usize) {
    let size: usize = ::common::coding_helpers::decode_size_string_u16(packed_file_data[index..(index + 2)].to_vec()) as usize;
    index += 2;
    let string = ::common::latin1_to_string(&packed_file_data[index..(index + size)]);
    index += size;
    (string, index)
}

/// This function allow us to decode an UTF-8 encoded optional string cell. We return the string (or
/// an empty string if it doesn't exist) and the index for the next cell's data.
///
/// NOTE: These strings's first byte it's a boolean that indicates if the string has something.
pub fn decode_optional_string_u8(packed_file_data: Vec<u8>, mut index: usize) -> (String, usize) {
    let exists = ::common::coding_helpers::decode_bool(packed_file_data[index]);
    index += 1;
    if exists {
        let size = ::common::coding_helpers::decode_size_string_u16(packed_file_data[index..(index + 2)].to_vec()) as usize;
        index += 2;
        let string = ::common::latin1_to_string(&packed_file_data[index..(index + size)]);
        index += size;
        (string, index)
    } else {
        let string = String::new();
        (string, index)
    }
}

/// This function allow us to decode an UTF-32 encoded integer cell. We return the integer and the index
/// for the next cell's data.
pub fn decode_integer_u32(packed_file_data: Vec<u8>, mut index: usize) -> (u32, usize) {
    let number = ::common::coding_helpers::decode_integer_u32(packed_file_data[index..(index + 4)].to_vec());
    index += 4;
    (number, index)
}

/// This function allow us to decode an UTF-32 encoded float cell. We return the float and the index
/// for the next cell's data.
pub fn decode_float_u32(packed_file_data: Vec<u8>, mut index: usize) -> (f32, usize) {
    let number = ::common::coding_helpers::decode_float_u32(packed_file_data[index..(index + 4)].to_vec());
    index += 4;
    (number, index)
}

/// This function allow us to decode a boolean cell. We return the boolean's value and the index
/// for the next cell's data.
pub fn decode_bool(packed_file_data: Vec<u8>, mut index: usize) -> (bool, usize) {
    let is_true = ::common::coding_helpers::decode_bool(packed_file_data[index]);
    index += 1;
    (is_true, index)
}

/*
--------------------------------------------------------
                   Encoding helpers
--------------------------------------------------------
*/

/// This function allow us to encode an UTF-8 decoded string cell. We return the Vec<u8> of
/// the encoded string.
pub fn encode_string_u8(string_u8_decoded: String) -> Vec<u8> {
    let mut string_u8_encoded = vec![];
    let string_u8_data = string_u8_decoded.as_bytes();
    let string_u8_lenght = &::common::u16_to_u8_reverse(string_u8_data.len() as u16);

    string_u8_encoded.extend_from_slice(string_u8_lenght);
    string_u8_encoded.extend_from_slice(string_u8_data);

    string_u8_encoded
}

/// This function allow us to encode an UTF-8 decoded string cell. We return the Vec<u8> of
/// the encoded string.
pub fn encode_optional_string_u8(optional_string_u8_decoded: String) -> Vec<u8> {
    let mut optional_string_u8_encoded = vec![];

    if optional_string_u8_decoded.is_empty() {
        optional_string_u8_encoded.extend_from_slice(("\u{0}").as_bytes());
    }
    else {
        let optional_string_u8_data = optional_string_u8_decoded.as_bytes();
        let optional_string_u8_lenght = &::common::u16_to_u8_reverse(optional_string_u8_data.len() as u16);

        optional_string_u8_encoded.extend_from_slice(("\u{1}").as_bytes());
        optional_string_u8_encoded.extend_from_slice(optional_string_u8_lenght);
        optional_string_u8_encoded.extend_from_slice(optional_string_u8_data);
    }

    optional_string_u8_encoded
}


/// This function allow us to encode to Vec<u8> an u32 cell. We return the Vec<u8>.
pub fn encode_integer_u32(integer_u32_decoded: u32) -> Vec<u8> {
    let integer_u32_encoded = ::common::u32_to_u8_reverse(integer_u32_decoded).to_vec();
    integer_u32_encoded
}

/// This function allow us to encode to Vec<u8> an f32 cell. We return the Vec<u8>.
pub fn encode_float_f32(float_f32_decoded: f32) -> Vec<u8> {
    let mut float_f32_encoded: [u8;4] = [0; 4];
    BigEndian::write_f32(&mut float_f32_encoded, float_f32_decoded);
    float_f32_encoded.reverse();
    float_f32_encoded.to_vec()
}


/// This function allow us to encode to Vec<u8> a boolean cell. We return the Vec<u8>.
pub fn encode_bool(bool_decoded: bool) -> Vec<u8> {
    let bool_encoded = ::common::coding_helpers::encode_bool(bool_decoded);
    bool_encoded
}