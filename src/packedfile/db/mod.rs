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
        packed_file_header_packed_file_version: u32
    ) -> DBData {

        let packed_file_data_structure: Option<OrderMap<String, String>>;
        let mut packed_file_data_entries_fields: OrderMap<String,String> = OrderMap::new();

        // We depend on a very specific string to find the table. This need to be changed to something more... stable.
        let index_master_schema = master_schema.find(
                &*format!("<table table_name='{}'\n         table_version='{}' >",
                packed_file_db_type,
                packed_file_header_packed_file_version
            )
        );

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

                        // We delete from the beggining of the line to the first "'", and keep the text
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
            //println!("{:#?}", packed_file_data_entries_fields);
            // We return the structure of the DB PackedFile.
            packed_file_data_structure = Some(packed_file_data_entries_fields);
        }
        else {

            // In case we didn't found a definition in the master_schema, we return None.
            packed_file_data_structure = None;
        }

        DBData {
            packed_file_data_structure,
            packed_file_data,
        }
    }
}
