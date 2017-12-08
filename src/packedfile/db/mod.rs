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

use ::common;
use std::u32;

use std::collections::HashMap;

use self::ordermap::OrderMap;

use self::byteorder::{
    ReadBytesExt, BigEndian
};

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
    pub packed_file_header_packed_file_entry_count: u32,
}

/// Struct DBData: This stores the data of a decoded DB PackedFile in memory.
/// It stores the PackedFile's data in a Vec<u8> and his structure in an OrderMap, if exists.
#[derive(Clone, Debug)]
pub struct DBData {
    pub packed_file_data_structure: Option<OrderMap<String, String>>,
    pub packed_file_data: Vec<u8>,
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
            packed_file_header.0.packed_file_header_packed_file_entry_count,
            packed_file_header.0.packed_file_header_packed_file_version
        );

        DB {
            packed_file_db_type: packed_file_db_type.to_string(),
            packed_file_header: packed_file_header.0,
            packed_file_data,
        }
    }
}


/// Implementation of "DBHeader"
impl DBHeader {

    /// This function creates a decoded DBHeader from a encoded PackedFile. It also return an index,
    /// to know where the body starts.
    pub fn read(packed_file_header: Vec<u8>) -> (DBHeader, usize) {
        let mut index: usize = 0;
        let mut packed_file_header_packed_file_guid: String;
        let mut packed_file_header_packed_file_version: u32;
        let mut packed_file_header_packed_file_version_marker: bool;

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

        // We skip a misteryous byte I don't know what it does.
        index += 1;

        let packed_file_header_packed_file_entry_count =  ::common::coding_helpers::decode_integer_u32(packed_file_header[(index)..(index + 4)].to_vec());

        index += 4;

        (DBHeader {
            packed_file_header_packed_file_guid,
            packed_file_header_packed_file_version,
            packed_file_header_packed_file_version_marker,
            packed_file_header_packed_file_entry_count,
        },
        index)
    }
}

/// Implementation of "DBData"
impl DBData {

    /// This function creates a decoded DBData from a encoded PackedFile.
    pub fn read(
        packed_file_data: Vec<u8>,
        packed_file_db_type: &str,
        master_schema: &str,
        packed_file_header_packed_file_entry_count: u32,
        packed_file_header_packed_file_version: u32
    ) -> DBData {
        let index: usize = 0;
        let packed_file_data_structure: Option<OrderMap<String, String>>;
        let mut packed_file_data_entries_fields: OrderMap<String,String> = OrderMap::new();

        let index_master_schema = master_schema.find(&*format!("<table table_name='{}'\n         table_version='{}' >", packed_file_db_type, packed_file_header_packed_file_version));

        // First, we check if it exists in the master schema
        if index_master_schema != None {

            // If we have found it in the schema, we take only that part of the schema
            let mut filtered_schema = master_schema.to_string().clone();
            filtered_schema.drain(..index_master_schema.unwrap());

            let mut index_filtered_schema = filtered_schema.find(&*format!("</table>")).unwrap();
            filtered_schema.drain(index_filtered_schema..);

            // We take out the name and version lines, leaving only the fields
            index_filtered_schema = filtered_schema.find(&*format!("\n")).unwrap();
            filtered_schema.drain(..(index_filtered_schema + 1));
            index_filtered_schema = filtered_schema.find(&*format!("\n")).unwrap();
            filtered_schema.drain(..(index_filtered_schema + 1));

            // Then we split the fields and delete the last one, as it's empty
            let mut fields: Vec<&str> = filtered_schema.split("\n").collect();
            fields.pop();

            // And get the data from every field to the entry fields hashmap
            for i in fields.iter() {

                let mut entry = i.to_string().clone();
                let mut index_field = entry.find(&*format!("\'")).unwrap();

                entry.drain(..(index_field + 1));
                index_field = entry.find(&*format!("\'")).unwrap();

                let name = entry.drain(..index_field).collect();

                entry.drain(..1);
                index_field = entry.find(&*format!("\'")).unwrap();
                entry.drain(..(index_field + 1));
                index_field = entry.find(&*format!("\'")).unwrap();

                let field_type = entry.drain(..index_field).collect();

                packed_file_data_entries_fields.insert(name, field_type);
            }




            println!("{:#?}", packed_file_data_entries_fields);
        }
        else {
            println!("DB PackedFile Type not supported. Yet.");
        }

        if packed_file_data_entries_fields.is_empty() {
            packed_file_data_structure = None;
        }
        else {
            packed_file_data_structure = Some(packed_file_data_entries_fields);
        }

        DBData {
            packed_file_data_structure,
            packed_file_data,
        }














/*

        match packed_file_db_type {
            "battles_tables" => {
                match &*packed_file_header_packed_file_version.to_string() {
                    "11" => {
                        packed_file_data_entries = vec![];
                        for _ in 0..packed_file_header_packed_file_entry_count {
                            let entry = tables::DBTables::BattlesTable(tables::Battles::V11 {
                                key: {
                                    let data = helpers::decode_string_u8(packed_file_data.to_vec(), index);
                                    index = data.1;
                                    data.0
                                },
                                battle_type: {
                                    let data = helpers::decode_string_u8(packed_file_data.to_vec(), index);
                                    index = data.1;
                                    data.0
                                },
                                is_naval: {
                                    let data = helpers::decode_bool(packed_file_data.to_vec(), index);
                                    index = data.1;
                                    data.0
                                },
                                specification: {
                                    let data = helpers::decode_string_u8(packed_file_data.to_vec(), index);
                                    index = data.1;
                                    data.0
                                },
                                screenshot_path: {
                                    let data = helpers::decode_optional_string_u8(packed_file_data.to_vec(), index);
                                    index = data.1;
                                    data.0
                                },
                                map_path: {
                                    let data = helpers::decode_optional_string_u8(packed_file_data.to_vec(), index);
                                    index = data.1;
                                    data.0
                                },
                                team_size_1: {
                                    let data = helpers::decode_integer_u32(packed_file_data.to_vec(), index);
                                    index = data.1;
                                    data.0
                                },
                                team_size_2: {
                                    let data = helpers::decode_integer_u32(packed_file_data.to_vec(), index);
                                    index = data.1;
                                    data.0
                                },
                                release: {
                                    let data = helpers::decode_bool(packed_file_data.to_vec(), index);
                                    index = data.1;
                                    data.0
                                },
                                multiplayer: {
                                    let data = helpers::decode_bool(packed_file_data.to_vec(), index);
                                    index = data.1;
                                    data.0
                                },
                                singleplayer: {
                                    let data = helpers::decode_bool(packed_file_data.to_vec(), index);
                                    index = data.1;
                                    data.0
                                },
                                intro_movie: {
                                    let data = helpers::decode_optional_string_u8(packed_file_data.to_vec(), index);
                                    index = data.1;
                                    data.0
                                },
                                year: {
                                    let data = helpers::decode_integer_u32(packed_file_data.to_vec(), index);
                                    index = data.1;
                                    data.0
                                },
                                defender_funds_ratio: {
                                    let data = helpers::decode_float_u32(packed_file_data.to_vec(), index);
                                    index = data.1;
                                    data.0
                                },
                                has_key_buildings: {
                                    let data = helpers::decode_bool(packed_file_data.to_vec(), index);
                                    index = data.1;
                                    data.0
                                },
                                matchmaking: {
                                    let data = helpers::decode_bool(packed_file_data.to_vec(), index);
                                    index = data.1;
                                    data.0
                                },
                                playable_area_width: {
                                    let data = helpers::decode_integer_u32(packed_file_data.to_vec(), index);
                                    index = data.1;
                                    data.0
                                },
                                playable_area_height: {
                                    let data = helpers::decode_integer_u32(packed_file_data.to_vec(), index);
                                    index = data.1;
                                    data.0
                                },
                                is_large_settlement: {
                                    let data = helpers::decode_bool(packed_file_data.to_vec(), index);
                                    index = data.1;
                                    data.0
                                },
                                has_15m_walls: {
                                    let data = helpers::decode_bool(packed_file_data.to_vec(), index);
                                    index = data.1;
                                    data.0
                                },
                                is_underground: {
                                    let data = helpers::decode_bool(packed_file_data.to_vec(), index);
                                    index = data.1;
                                    data.0
                                },
                                catchment_name: {
                                    let data = helpers::decode_optional_string_u8(packed_file_data.to_vec(), index);
                                    index = data.1;
                                    data.0
                                },
                                tile_upgrade: {
                                    let data = helpers::decode_optional_string_u8(packed_file_data.to_vec(), index);
                                    index = data.1;
                                    data.0
                                },
                                battle_environment: {
                                    let data = helpers::decode_optional_string_u8(packed_file_data.to_vec(), index);
                                    index = data.1;
                                    data.0
                                },
                                battle_environment_audio: {
                                    let data = helpers::decode_optional_string_u8(packed_file_data.to_vec(), index);
                                    index = data.1;
                                    data.0
                                },
                            });
                            packed_file_data_entries.push(entry);
                        }
                    }
                    _ => {
                        println!("DB PackedFile Version not yet implemented.");
                    }

                }
            }
            _ =>
                for i in 0..packed_file_header_packed_file_entry_count {
                    let entry = tables::DBTables::BattlesTable(tables::Battles::V11 {
                    key: format!("D"),
                    battle_type: format!("D"),
                    is_naval: true,
                    specification: format!("D"),
                    screenshot_path: format!("D"),
                    map_path: format!("D"),
                    team_size_1: 10,
                    team_size_2: 10,
                    release: true,
                    multiplayer: true,
                    singleplayer: true,
                    intro_movie: format!("D"),
                    year: 10,
                    defender_funds_ratio: 10.0,
                    has_key_buildings: true,
                    matchmaking: true,
                    playable_area_width: 10,
                    playable_area_height: 10,
                    is_large_settlement: true,
                    has_15m_walls: true,
                    is_underground: true,
                    catchment_name: format!("D"),
                    tile_upgrade: format!("D"),
                    battle_environment: format!("D"),
                    battle_environment_audio: format!("D"),
                });
                packed_file_data_entries.push(entry);
            }
        }
        DBData {
            packed_file_data_entries,
        }*/
    }
}
