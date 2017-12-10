// In this file we define the PackedFile type DB for decoding and encoding it.
// This is the type used by database files.

// The structure of a header is:
// 4 bytes for the GUID marker.
// 2 bytes (u16) for the lenght of the GUID (* 2).
// The GUID in u16 bytes.
// 4 bytes for the Version marker, if it have it.
// 4 bytes for the Version, in u32 reversed.
// 1 misteryous byte
// 4 bytes for the entry count, in u32 reversed.

extern crate byteorder;
extern crate ordermap;

use std::u32;

use self::ordermap::OrderMap;

pub mod helpers;

/// These two const are the markers we need to check in the header of every DB file.
const GUID_MARKER: &[u8] = &[253, 254, 252, 255];
const VERSION_MARKER: &[u8] = &[252, 253, 254, 255];

/// Struct DB: This stores the data of a decoded DB PackedFile in memory.
/// It stores the PackedFile divided in 3 parts:
/// - packed_file_db_type: the type of table, so we know how his data is structured.
/// - packed_file_header: header of the PackedFile, decoded.
/// - packed_file_data: data of the PackedFile, decoded.
#[derive(Clone)]
pub struct DB {
    pub packed_file_db_type: String,
    pub packed_file_header: DBHeader,
    pub packed_file_data: DBData,
}

/// Struct DBHeader: This stores the header of a decoded DB PackedFile in memory.
/// It stores the PackedFile's header in different parts:
/// - packed_file_header_packed_file_guid:
/// - packed_file_header_packed_file_version:
/// - packed_file_header_packed_file_version_marker:
/// - packed_file_header_packed_file_entry_count:
#[derive(Clone, Debug)]
pub struct DBHeader {
    pub packed_file_header_packed_file_guid: String,
    pub packed_file_header_packed_file_version: u32,
    pub packed_file_header_packed_file_version_marker: bool,
    pub packed_file_header_packed_file_mysterious_byte: u8,
    pub packed_file_header_packed_file_entry_count: u32,
}

/// Struct DBData: This stores the data of a decoded DB PackedFile in memory.
/// It stores the PackedFile's data in a Vec<u8> and his structure in an OrderMap, if exists.
#[derive(Clone, Debug)]
pub struct DBData {
    pub packed_file_data_structure: Option<OrderMap<String, String>>,
    pub packed_file_data: Vec<Vec<DecodedData>>,
}

/// Enum DecodedData: This enum is used to store the data from the different fields of a row of a DB
/// PackedFile.
#[derive(Clone, Debug)]
pub enum DecodedData {
    Index(String),
    Boolean(bool),
    String(String),
    OptionalString(String),
    Integer(u32),
    Float(f32),
    RawData(Vec<u8>)
}

/// Implementation of "DB"
impl DB {

    /// This function creates a new decoded DB from a encoded PackedFile. This assumes the PackedFile is
    /// a DB PackedFile. It'll crash otherwise.
    pub fn read(packed_file_data: Vec<u8>, packed_file_db_type: &str, master_schema: &str) -> DB {
        let packed_file_header: (DBHeader, usize) = DBHeader::read(packed_file_data.to_vec());
        let packed_file_data = DBData::read(
            packed_file_data[(packed_file_header.1)..].to_vec(),
            packed_file_db_type,
            master_schema,
            packed_file_header.0.packed_file_header_packed_file_version,
            packed_file_header.0.packed_file_header_packed_file_entry_count
        );

        DB {
            packed_file_db_type: packed_file_db_type.to_string(),
            packed_file_header: packed_file_header.0,
            packed_file_data,
        }
    }

    /// This function takes an entire DB and encode it to Vec<u8>, so it can be written in the disk.
    /// It returns a Vec<u8> with the entire DB encoded in it.
    pub fn save(packed_file_decoded: &DB) -> Vec<u8> {

        let mut packed_file_data_encoded = DBData::save(&packed_file_decoded.packed_file_data);
        let mut packed_file_header_encoded = DBHeader::save(&packed_file_decoded.packed_file_header, packed_file_data_encoded.1);

        let mut packed_file_encoded: Vec<u8> = vec![];
        packed_file_encoded.append(&mut packed_file_header_encoded);
        packed_file_encoded.append(&mut packed_file_data_encoded.0);
        packed_file_encoded
    }
}


/// Implementation of "DBHeader"
impl DBHeader {

    /// This function creates a decoded DBHeader from a encoded PackedFile. It also return an index,
    /// to know where the body starts.
    pub fn read(packed_file_header: Vec<u8>) -> (DBHeader, usize) {
        let mut index: usize = 0;
        let packed_file_header_packed_file_guid: String;
        let packed_file_header_packed_file_version: u32;
        let packed_file_header_packed_file_version_marker: bool;

        // If it has a GUID_MARKER, we get the GUID.
        if &packed_file_header[index..(index + 4)] == GUID_MARKER {
            let packed_file_header_guid_lenght: u16 = (::common::coding_helpers::decode_integer_u16(packed_file_header[4..6].to_vec())) * 2;
            packed_file_header_packed_file_guid = ::common::coding_helpers::decode_string_u16(packed_file_header[6..(6 + (packed_file_header_guid_lenght as usize))].to_vec());
            index = 6 + packed_file_header_guid_lenght as usize;
        }
        else {
            packed_file_header_packed_file_guid = String::new();
        }

        // If it has a VERSION_MARKER, we get the version of the table.
        if &packed_file_header[index..(index + 4)] == VERSION_MARKER {
            packed_file_header_packed_file_version = ::common::coding_helpers::decode_integer_u32(packed_file_header[(index + 4)..(index + 8)].to_vec());
            packed_file_header_packed_file_version_marker = true;
            index = index + 8;
        }
        else {
            packed_file_header_packed_file_version = 0;
            packed_file_header_packed_file_version_marker = false;
        }

        // We save a mysterious byte I don't know what it does.
        let packed_file_header_packed_file_mysterious_byte = packed_file_header[index];
        index += 1;

        let packed_file_header_packed_file_entry_count =  ::common::coding_helpers::decode_integer_u32(packed_file_header[(index)..(index + 4)].to_vec());

        index += 4;

        (DBHeader {
            packed_file_header_packed_file_guid,
            packed_file_header_packed_file_version,
            packed_file_header_packed_file_version_marker,
            packed_file_header_packed_file_mysterious_byte,
            packed_file_header_packed_file_entry_count,
        },
        index)
    }


    /// This function takes an entire DBHeader and a packed_file_entry_count, and encode it to Vec<u8>,
    /// so it can be written in the disk. It returns a Vec<u8> with the entire DBHeader encoded in it.
    pub fn save(packed_file_header_decoded: &DBHeader, packed_file_entry_count: u32) -> Vec<u8> {
        let mut packed_file_header_encoded: Vec<u8> = vec![];

        // First we get the lenght of the GUID (u16 reversed) and the GUID, in a u16 string.
        let guid_encoded = ::common::coding_helpers::encode_string_u16(packed_file_header_decoded.packed_file_header_packed_file_guid.clone()).to_vec();

        packed_file_header_encoded.extend_from_slice(&GUID_MARKER);
        packed_file_header_encoded.extend_from_slice(&guid_encoded);

        if packed_file_header_decoded.packed_file_header_packed_file_version_marker {
            let version_encoded = ::common::u32_to_u8_reverse(packed_file_header_decoded.packed_file_header_packed_file_version).to_vec();

            packed_file_header_encoded.extend_from_slice(&VERSION_MARKER);
            packed_file_header_encoded.extend_from_slice(&version_encoded);
        }

        let packed_file_entry_count_encoded = ::common::u32_to_u8_reverse(packed_file_entry_count).to_vec();

        packed_file_header_encoded.push(packed_file_header_decoded.packed_file_header_packed_file_mysterious_byte);
        packed_file_header_encoded.extend_from_slice(&packed_file_entry_count_encoded);

        packed_file_header_encoded
    }
}


/// Implementation of "DBData"
impl DBData {

    /// This function creates a decoded DBData from a encoded PackedFile.
    pub fn read(
        packed_file_data: Vec<u8>,
        packed_file_db_type: &str,
        master_schema: &str,
        packed_file_header_packed_file_version: u32,
        packed_file_header_packed_file_entry_count: u32
    ) -> DBData {

        let packed_file_data_decoded: Vec<Vec<DecodedData>>;
        let packed_file_data_structure: Option<OrderMap<String, String>>;
        let mut packed_file_data_entries_fields: OrderMap<String,String> = OrderMap::new();

        // We depend on a very specific string to find the table. This need to be changed to something more... stable.
        let index_master_schema;
        if cfg!(target_os = "linux") {
            index_master_schema = master_schema.find(
                &*format!("<table table_name='{}'\n         table_version='{}' >",
                          packed_file_db_type,
                          packed_file_header_packed_file_version
                )
            );
        }
        else {
            index_master_schema = master_schema.find(
                &*format!("<table table_name='{}'\r\n         table_version='{}' >",
                          packed_file_db_type,
                          packed_file_header_packed_file_version
                )
            );
        }

        // First, we check if it exists in the master_schema.
        if index_master_schema != None {

            // If we have found it in the schema, we take only that part of the master_schema.
            let mut filtered_schema = master_schema.to_string().clone();
            filtered_schema.drain(..index_master_schema.unwrap());

            let mut index_filtered_schema = filtered_schema.find(&*format!("</table>")).unwrap();
            filtered_schema.drain(index_filtered_schema..);

            // We take out the name and version lines, leaving only the fields. The +1 is to delete the
            // index character too.
            index_filtered_schema = filtered_schema.find(&*format!("\n")).unwrap();
            filtered_schema.drain(..(index_filtered_schema + 1));
            index_filtered_schema = filtered_schema.find(&*format!("\n")).unwrap();
            filtered_schema.drain(..(index_filtered_schema + 1));

            // Then we split the fields and delete the last one, as it's empty.
            let mut fields: Vec<&str> = filtered_schema.split("\n").collect();
            fields.pop();

            // And get the data from every field to the entry fields OrderMap.
            for i in fields.iter() {

                let mut entry = i.to_string().clone();

                // We need to skip the line if it doesn't have a "name=\'" string, as we do not support
                // foreign keys yet.
                match entry.find(&*format!("name=\'")) {
                    Some(mut index_field) => {

                        // We delete from the beginning of the line to the first "'", and keep the text
                        // from there until the next "'". That's our name
                        entry.drain(..(index_field + 6));
                        index_field = entry.find(&*format!("\'")).unwrap();

                        let name = entry.drain(..index_field).collect();

                        // The same for the field type. We need to take out the first character before
                        // doing it, because it's the "'" closing the name, not the one we want.
                        entry.drain(..1);
                        index_field = entry.find(&*format!("\'")).unwrap();
                        entry.drain(..(index_field + 1));
                        index_field = entry.find(&*format!("\'")).unwrap();

                        let field_type = entry.drain(..index_field).collect();

                        // Then we add the entry to the OrderMap.
                        packed_file_data_entries_fields.insert(name, field_type);
                    },
                    None => {

                        // In case there is no name, we skip the line.
                        continue;
                    },
                }
            }

            // Now that we have the structure of the DB PackedFile, we decode his data.
            // First, we get the amount of columns we have.
            let column_amount = packed_file_data_entries_fields.len();

            let mut packed_file_data_decoded_rows: Vec<Vec<DecodedData>> = vec![];

            // Then we go field by field putting data into a Vec<DecodedData>, and every row
            // (Vec<DecodedData) into a Vec<Vec<DecodedData>>.
            let mut index = 0;
            for row in 0..packed_file_header_packed_file_entry_count {
                let mut entry: Vec<DecodedData> = vec![];
                for column in 0..(column_amount + 1) {

                    // First column it's always the index.
                    if column == 0 {
                        let entry_index = DecodedData::Index(format!("{:0count$}", (row + 1), count = (packed_file_header_packed_file_entry_count.to_string().len() + 1)));
                        entry.push(entry_index);
                    }

                    // The rest of the columns, we decode them based on his type and store them in a DecodedData
                    // enum, as enums are the only thing I found that can store them.
                    else {
                        let field = packed_file_data_entries_fields.get_index((column as usize) - 1).unwrap();
                        let field_type = field.1;

                        match &**field_type {
                            "boolean" => {
                                let data = helpers::decode_bool(packed_file_data.to_vec(), index);
                                index = data.1;
                                entry.push(DecodedData::Boolean(data.0));
                            }
                            "string_ascii" => {
                                let data = helpers::decode_string_u8(packed_file_data.to_vec(), index);
                                index = data.1;
                                entry.push(DecodedData::String(data.0));
                            }
                            "optstring_ascii" => {
                                let data = helpers::decode_optional_string_u8(packed_file_data.to_vec(), index);
                                index = data.1;
                                entry.push(DecodedData::OptionalString(data.0));
                            }
                            "int" => {
                                let data = helpers::decode_integer_u32(packed_file_data.to_vec(), index);
                                index = data.1;
                                entry.push(DecodedData::Integer(data.0));
                            }
                            "float" => {
                                let data = helpers::decode_float_u32(packed_file_data.to_vec(), index);
                                index = data.1;
                                entry.push(DecodedData::Float(data.0));
                            }
                            _ => {
                                // If this fires up, the table has a non-implemented field. Current non-
                                // implemented fields are "string" and "oopstring".
                                println!("Unkown field_type 4 {}", field_type);
                            }
                        }
                    }
                }
                packed_file_data_decoded_rows.push(entry.clone());
            }
            // We return the structure of the DB PackedFile and his decoded data.
            packed_file_data_structure = Some(packed_file_data_entries_fields);
            packed_file_data_decoded = packed_file_data_decoded_rows;
        }
        else {

            // In case we didn't found a definition in the master_schema, we return None and the RawData
            // of the DB PackedFile.
            packed_file_data_structure = None;
            packed_file_data_decoded = vec![vec![DecodedData::RawData(packed_file_data)]];
        }

        let packed_file_data = packed_file_data_decoded;

        DBData {
            packed_file_data_structure,
            packed_file_data,
        }
    }


    /// This function takes an entire DBData and encode it to Vec<u8>, so it can be written in the disk.
    /// It returns a tuple with the encoded DBData in a Vec<u8> and the new entry count to update the
    /// header.
    pub fn save(packed_file_data_decoded: &DBData) -> (Vec<u8>, u32) {

        let mut packed_file_data_encoded: Vec<u8> = vec![];
        let mut packed_file_entry_count = 0;

        for row in &packed_file_data_decoded.packed_file_data {
            for field in row {
                match *field {
                    DecodedData::Index(_) => {

                        // We skip the index column, as we only have it for easy manipulation, it has
                        // nothing to do with the PackedFile.
                        continue;
                    },
                    DecodedData::Boolean(data) => {
                        let mut encoded_data = helpers::encode_bool(data.clone());
                        packed_file_data_encoded.append(&mut encoded_data);
                    },
                    DecodedData::String(ref data) => {
                        let mut encoded_data = helpers::encode_string_u8(data.clone());
                        packed_file_data_encoded.append(&mut encoded_data);
                    },
                    DecodedData::OptionalString(ref data) => {
                        let mut encoded_data = helpers::encode_optional_string_u8(data.clone());
                        packed_file_data_encoded.append(&mut encoded_data);
                    },
                    DecodedData::Integer(data) => {
                        let mut encoded_data = helpers::encode_integer_u32(data.clone());
                        packed_file_data_encoded.append(&mut encoded_data);
                    },
                    DecodedData::Float(data) => {
                        let mut encoded_data = helpers::encode_float_f32(data.clone());
                        packed_file_data_encoded.append(&mut encoded_data);
                    },
                    DecodedData::RawData(_) => {
                        // If this is reached, we fucked it up somewhere. For now, just print a warning.
                        println!("Error, trying to write a RawData DB field.")
                    },
                }
            }
            packed_file_entry_count += 1;
        }
        (packed_file_data_encoded, packed_file_entry_count)
    }
}
