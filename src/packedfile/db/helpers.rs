// Here are all the helper functions needed to decode and encode entries in a DB PackedFile.

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




/// This function allow us to encode to Vec<u8> a boolean cell. We return the Vec<u8> and the index
/// for the next cell's data.
pub fn encode_bool(bool_decoded: bool, mut index: usize) -> (Vec<u8>, usize) {
    let bool_encoded = ::common::coding_helpers::encode_bool(bool_decoded);
    index += 1;
    (bool_encoded, index)
}