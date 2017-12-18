// In this file we define the PackedFile type Loc for decoding and encoding it.
// This is the type used by localisation files.

use common::coding_helpers;

/// Struct Loc: This stores the data of a decoded Localisation PackedFile in memory.
/// It stores the PackedFile divided in 2 parts:
/// - packed_file_header: header of the PackedFile, decoded.
/// - packed_file_data: data of the PackedFile, decoded.
#[derive(Clone)]
pub struct Loc {
    pub packed_file_header: LocHeader,
    pub packed_file_data: LocData,
}

/// Struct LocHeader: This stores the header of a decoded Localisation PackedFile in memory.
/// It stores the PackedFile's header in different parts:
/// - packed_file_header_byte_order_mark: an u16 (2 bytes) that marks the beginning of the PackedFile (FF FE).
/// - packed_file_header_packed_file_type: LOC (3 bytes) in our case. After this it should be a 0 byte.
/// - packed_file_header_packed_file_version: if this is not 1, the file is invalid, don't know why.
/// - packed_file_header_packed_file_entry_count: amount of entries in the file.
#[derive(Clone)]
pub struct LocHeader {
    pub packed_file_header_byte_order_mark: u16,
    pub packed_file_header_packed_file_type: String,
    pub packed_file_header_packed_file_version: u32,
    pub packed_file_header_packed_file_entry_count: u32,
}

/// Struct LocData: This stores the data of a decoded Localisation PackedFile in memory.
/// It stores the PackedFile's data in a Vec<LocDataEntry>.
#[derive(Clone, Debug)]
pub struct LocData {
    pub packed_file_data_entries: Vec<LocDataEntry>,
}

/// Struct LocDataEntry: This stores an entry of a decoded Localisation PackedFile in memory.
/// It stores the entry's data in multiple parts:
/// - key: the "key" column of the entry.
/// - text: the text you'll see ingame.
/// - tooltip (bool): this one I believe it was to enable or disable certain lines ingame.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LocDataEntry {
    pub key: String,
    pub text: String,
    pub tooltip: bool,
}

/// Implementation of "Loc"
impl Loc {

    /// This function creates a new decoded Loc from the data of a PackedFile. Note that this assume
    /// the file is a loc. It'll crash otherwise.
    pub fn read(packed_file_data: Vec<u8>) -> Loc {
        let packed_file_header = LocHeader::read(packed_file_data[..14].to_vec());
        let packed_file_data = LocData::read(packed_file_data[14..].to_vec(), &packed_file_header.packed_file_header_packed_file_entry_count);
        Loc {
            packed_file_header,
            packed_file_data,
        }
    }

    /// This function takes a LocHeader and a LocData and put them together in a Vec<u8>, encoding an
    /// entire LocFile ready to write on disk.
    pub fn save(packed_file_decoded: &Loc) -> Vec<u8> {
        let mut packed_file_data_encoded = LocData::save(&packed_file_decoded.packed_file_data);
        let mut packed_file_header_encoded = LocHeader::save(&packed_file_decoded.packed_file_header, packed_file_data_encoded.1);

        let mut packed_file_encoded: Vec<u8> = vec![];
        packed_file_encoded.append(&mut packed_file_header_encoded);
        packed_file_encoded.append(&mut packed_file_data_encoded.0);
        packed_file_encoded
    }
}

/// Implementation of "LocHeader"
impl LocHeader {

    /// This function creates a new decoded LocHeader from the data of a PackedFile. To see what are
    /// these values, check the LocHeader struct.
    pub fn read(packed_file_header: Vec<u8>) -> LocHeader {
        let packed_file_header_byte_order_mark: u16 = coding_helpers::decode_integer_u16((&packed_file_header[0..2]).to_vec());
        let packed_file_header_packed_file_type = coding_helpers::decode_string_u8(packed_file_header[2..5].to_vec());
        let packed_file_header_packed_file_version: u32 = coding_helpers::decode_integer_u32((&packed_file_header[6..10]).to_vec());
        let packed_file_header_packed_file_entry_count: u32 = coding_helpers::decode_integer_u32((&packed_file_header[10..14]).to_vec());

        LocHeader {
            packed_file_header_byte_order_mark,
            packed_file_header_packed_file_type,
            packed_file_header_packed_file_version,
            packed_file_header_packed_file_entry_count,
        }
    }

    /// This function takes a LocHeader and an entry count and creates a Vec<u8> encoded version of
    /// the LocHeader, ready to write it on disk.
    pub fn save(packed_file_header_decoded: &LocHeader, packed_file_entry_count: u32) -> Vec<u8> {
        let mut packed_file_header_encoded: Vec<u8> = vec![];

        packed_file_header_encoded.extend_from_slice(&coding_helpers::encode_integer_u16(packed_file_header_decoded.packed_file_header_byte_order_mark));
        packed_file_header_encoded.extend_from_slice(&coding_helpers::encode_string_u8(packed_file_header_decoded.packed_file_header_packed_file_type.clone()));
        packed_file_header_encoded.extend_from_slice("\0".as_bytes());
        packed_file_header_encoded.extend_from_slice(&coding_helpers::encode_integer_u32(packed_file_header_decoded.packed_file_header_packed_file_version));
        packed_file_header_encoded.extend_from_slice(&coding_helpers::encode_integer_u32(packed_file_entry_count));

        packed_file_header_encoded
    }
}

/// Implementation of "LocData"
impl LocData {

    /// This function returns an empty LocData.
    pub fn new() -> LocData {
        let packed_file_data_entries: Vec<LocDataEntry> = vec![];
        LocData {
            packed_file_data_entries,
        }
    }

    /// This function creates a new decoded LocData from the data of a PackedFile. A LocData is a
    /// Vec<LocDataEntry>. This pass through all the data of the Loc PackedFile and decodes every
    /// entry.
    pub fn read(packed_file_data: Vec<u8>, packed_file_entry_count: &u32) -> LocData {
        let mut packed_file_data_entries: Vec<LocDataEntry> = vec![];

        let mut entry_offset: u32 = 0;
        let mut entry_field_offset: u32;
        let mut entry_size_byte_offset: u32 = 0;
        let mut entry_field: u32 = 0;
        let mut entry_field_size: u16 = 0;

        // For each entry
        for _ in 0..packed_file_entry_count.clone() {

            let mut key: String = String::new();
            let mut text: String = String::new();
            let tooltip: bool;

            let done = false;
            while !done {

                // The first 2 bytes of a String is the length of the String in reversed utf-16.
                if entry_size_byte_offset == 0 && entry_field < 2 {

                    entry_field_size = coding_helpers::decode_integer_u16(packed_file_data[(entry_offset as usize)..(entry_offset as usize) + 2].into());
                    entry_size_byte_offset = 2;
                }
                else {
                    entry_field_offset = 0;
                    match entry_field {

                        // If is the key or the text, we decode it. Remember, the chars are reversed
                        // utf-16 so they use 2 bytes and need to be reversed before using them.
                        0 | 1 => {
                            let string_encoded_begin = (entry_offset + entry_field_offset + entry_size_byte_offset) as usize;
                            let string_encoded_end = (entry_offset + entry_field_offset + entry_size_byte_offset + ((entry_field_size * 2) as u32)) as usize;
                            let string_encoded: Vec<u8> = packed_file_data[string_encoded_begin..string_encoded_end].to_vec();
                            let string_decoded = coding_helpers::decode_string_u16(string_encoded);

                            if entry_field == 0 {
                                key = string_decoded;
                            }
                            else {
                                text = string_decoded;
                            }
                            entry_field_offset += (entry_field_size * 2) as u32;

                            entry_field += 1;
                            entry_offset = entry_offset + entry_size_byte_offset + entry_field_offset;
                            entry_size_byte_offset = 0;
                        }

                        // If it's the boolean, it's a byte, so it doesn't have a size byte offset.
                        _ => {
                            tooltip = ::common::coding_helpers::decode_bool(packed_file_data[(entry_offset as usize)]);
                            packed_file_data_entries.push(LocDataEntry::new(key, text, tooltip));

                            entry_field = 0;
                            entry_offset += 1;
                            break;
                        }
                    }
                }
            }
        }
        LocData {
            packed_file_data_entries,
        }
    }

    /// This function takes an entire LocData and encode it to Vec<u8> to write it on disk. Also, it
    /// returns his entry count for the header.
    pub fn save(packed_file_data_decoded: &LocData) -> (Vec<u8>, u32) {
        let mut packed_file_data_encoded: Vec<u8> = vec![];
        let mut packed_file_entry_count = 0;

        for i in &packed_file_data_decoded.packed_file_data_entries {
            packed_file_data_encoded.append(&mut coding_helpers::encode_string_u16(i.key.clone()));
            packed_file_data_encoded.append(&mut coding_helpers::encode_string_u16(i.text.clone()));
            packed_file_data_encoded.append(&mut coding_helpers::encode_bool(i.tooltip));
            packed_file_entry_count += 1;
        }
        (packed_file_data_encoded, packed_file_entry_count)
    }
}

/// Implementation of "LocDataEntry"
impl LocDataEntry {

    /// This function takes the key, text and tooltip values and makes a LocDataEntry with them.
    pub fn new(key: String, text: String, tooltip: bool) -> LocDataEntry {
        LocDataEntry {
            key,
            text,
            tooltip,
        }
    }
}