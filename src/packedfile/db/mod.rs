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

use std::io::{
    Error, ErrorKind
};
use common::coding_helpers;

pub mod schemas;
pub mod schemas_importer;

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
    pub packed_file_header_packed_file_guid: (String, usize),
    pub packed_file_header_packed_file_version: u32,
    pub packed_file_header_packed_file_version_marker: bool,
    pub packed_file_header_packed_file_mysterious_byte: u8,
    pub packed_file_header_packed_file_entry_count: u32,
}

/// Struct DBData: This stores the data of a decoded DB PackedFile in memory.
/// It stores the PackedFile's data in a Vec<u8> and his structure in an OrderMap, if exists.
#[derive(Clone, Debug)]
pub struct DBData {
    pub table_definition: schemas::TableDefinition,
    pub packed_file_data: Vec<Vec<DecodedData>>,
}

/// Enum DecodedData: This enum is used to store the data from the different fields of a row of a DB
/// PackedFile.
#[derive(Clone, Debug)]
pub enum DecodedData {
    Index(String),
    Boolean(bool),
    StringU8(String),
    StringU16(String),
    OptionalStringU8(String),
    OptionalStringU16(String),
    Integer(u32),
    Float(f32),
}

/// Implementation of "DB"
impl DB {

    /// This function creates a new decoded DB from a encoded PackedFile. This assumes the PackedFile is
    /// a DB PackedFile. It'll crash otherwise.
    pub fn read(packed_file_data: Vec<u8>, packed_file_db_type: &str, master_schema: &schemas::Schema) -> Result<DB, Error> {

        match DBHeader::read(packed_file_data.to_vec()) {
            Ok(packed_file_header) => {
                match DB::get_schema(packed_file_db_type, packed_file_header.0.packed_file_header_packed_file_version, master_schema) {
                    Some(table_definition) => {
                        match DBData::read(
                            packed_file_data[(packed_file_header.1)..].to_vec(),
                            &table_definition,
                            packed_file_header.0.packed_file_header_packed_file_entry_count,
                        ) {
                            Ok(packed_file_data) =>
                                Ok(
                                    DB {
                                        packed_file_db_type: packed_file_db_type.to_string(),
                                        packed_file_header: packed_file_header.0,
                                        packed_file_data,
                                    }
                                ),
                            Err(error) => Err(error)
                        }
                    }
                    None => return Err(Error::new(ErrorKind::Other, format!("Schema for this Packedfile not found")))
                }

            }
            Err(error) => Err(error)
        }
    }

    /// This function takes an entire DB and encode it to Vec<u8>, so it can be written in the disk.
    /// It returns a Vec<u8> with the entire DB encoded in it.
    pub fn save(packed_file_decoded: &DB) -> Result<Vec<u8>, Error> {

        let mut packed_file_data_encoded = DBData::save(&packed_file_decoded.packed_file_data)?;
        let mut packed_file_header_encoded = DBHeader::save(&packed_file_decoded.packed_file_header, packed_file_data_encoded.1);

        let mut packed_file_encoded: Vec<u8> = vec![];
        packed_file_encoded.append(&mut packed_file_header_encoded);
        packed_file_encoded.append(&mut packed_file_data_encoded.0);
        Ok(packed_file_encoded)
    }

    /// This function gets the schema corresponding to the table we passed it, if it exists.
    pub fn get_schema(db_name: &str, version: u32, schema: &schemas::Schema) -> Option<schemas::TableDefinition> {
        // If we find our table in the TableDefinitions vector...
        if let Some(index_table_definitions) = schema.get_table_definitions(db_name) {
            // And it has a definition created for our version of the table...
            if let Some(index_table_versions) = schema.tables_definitions[index_table_definitions].get_table_version(version) {
                // And that definition has any fields, we get it.
                if !schema.tables_definitions[index_table_definitions].versions[index_table_versions].fields.is_empty() {
                    return Some(schema.tables_definitions[index_table_definitions].versions[index_table_versions].clone())
                }
            }
        }
        None
    }
}


/// Implementation of "DBHeader"
impl DBHeader {

    /// This function creates a new DBHeader from nothing.
    pub fn new() -> DBHeader {
        let packed_file_header_packed_file_guid = (String::new(), 0);
        let packed_file_header_packed_file_version = 0;
        let packed_file_header_packed_file_version_marker = false;
        let packed_file_header_packed_file_mysterious_byte = 0;
        let packed_file_header_packed_file_entry_count = 0;

        DBHeader {
            packed_file_header_packed_file_guid,
            packed_file_header_packed_file_version,
            packed_file_header_packed_file_version_marker,
            packed_file_header_packed_file_mysterious_byte,
            packed_file_header_packed_file_entry_count,
        }
    }

    /// This function creates a decoded DBHeader from a encoded PackedFile. It also return an index,
    /// to know where the body starts.
    pub fn read(packed_file_header: Vec<u8>) -> Result<(DBHeader, usize), Error> {

        let mut packed_file_header_decoded = DBHeader::new();
        let mut index: usize = 0;

        // If it has a GUID_MARKER, we get the GUID.
        if &packed_file_header[index..(index + 4)] == GUID_MARKER {
            index += 4;
            packed_file_header_decoded.packed_file_header_packed_file_guid = coding_helpers::decode_packedfile_string_u16(packed_file_header[index..].to_vec(), index)?;
            index = packed_file_header_decoded.packed_file_header_packed_file_guid.1;
        }

        // If it has a VERSION_MARKER, we get the version of the table.
        if &packed_file_header[index..(index + 4)] == VERSION_MARKER {
            packed_file_header_decoded.packed_file_header_packed_file_version = coding_helpers::decode_integer_u32(packed_file_header[(index + 4)..(index + 8)].to_vec())?;
            packed_file_header_decoded.packed_file_header_packed_file_version_marker = true;
            index = index + 8;
        }

        // We save a mysterious byte I don't know what it does.
        packed_file_header_decoded.packed_file_header_packed_file_mysterious_byte = packed_file_header[index];
        index += 1;

        packed_file_header_decoded.packed_file_header_packed_file_entry_count = coding_helpers::decode_integer_u32(packed_file_header[(index)..(index + 4)].to_vec())?;
        index += 4;

        Ok((packed_file_header_decoded, index))
    }

    /// This function takes an entire DBHeader and a packed_file_entry_count, and encode it to Vec<u8>,
    /// so it can be written in the disk. It returns a Vec<u8> with the entire DBHeader encoded in it.
    pub fn save(packed_file_header_decoded: &DBHeader, packed_file_entry_count: u32) -> Vec<u8> {
        let mut packed_file_header_encoded: Vec<u8> = vec![];

        let guid_encoded = coding_helpers::encode_packedfile_string_u16(packed_file_header_decoded.packed_file_header_packed_file_guid.0.clone());

        packed_file_header_encoded.extend_from_slice(&GUID_MARKER);
        packed_file_header_encoded.extend_from_slice(&guid_encoded);

        if packed_file_header_decoded.packed_file_header_packed_file_version_marker {
            let version_encoded = coding_helpers::encode_integer_u32(packed_file_header_decoded.packed_file_header_packed_file_version);

            packed_file_header_encoded.extend_from_slice(&VERSION_MARKER);
            packed_file_header_encoded.extend_from_slice(&version_encoded);
        }

        let packed_file_entry_count_encoded = coding_helpers::encode_integer_u32(packed_file_entry_count);

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
        table_definition: &schemas::TableDefinition,
        packed_file_header_packed_file_entry_count: u32
    ) -> Result<DBData, Error> {

        let packed_file_data_decoded: Vec<Vec<DecodedData>>;
        let table_definition = table_definition.clone();

        // First, we get the amount of columns we have.
        let column_amount = table_definition.fields.len();

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
                // enum, as enums are the only thing I found that can store them
                else {
                    let field_type = &table_definition.fields[column as usize - 1].field_type;
                    match *field_type {
                        schemas::FieldType::Boolean => {
                            if index <= packed_file_data.len() {
                                match coding_helpers::decode_packedfile_bool(packed_file_data[index], index) {
                                    Ok(data) => {
                                        index = data.1;
                                        entry.push(DecodedData::Boolean(data.0));
                                    }
                                    Err(error) => return Err(error)
                                };
                            }
                            else {
                                return Err(Error::new(ErrorKind::Other, format!("Error: trying to decode a bool without a byte.")))
                            }
                        }
                        schemas::FieldType::Float => {
                            // Check if the index does even exist, to avoid crashes.
                            if (index + 4) <= packed_file_data.len() {
                                match coding_helpers::decode_packedfile_float_u32(packed_file_data[index..(index + 4)].to_vec(), index) {
                                    Ok(data) => {
                                        index = data.1;
                                        entry.push(DecodedData::Float( data.0));
                                    }
                                    Err(error) => return Err(error)
                                };
                            }
                            else {
                                return Err(Error::new(ErrorKind::Other, format!("Error: trying to decode a Float without enough bytes.")))
                            }
                        }
                        schemas::FieldType::Integer => {
                            // Check if the index does even exist, to avoid crashes.
                            if (index + 4) <= packed_file_data.len() {
                                match coding_helpers::decode_packedfile_integer_u32(packed_file_data[index..(index + 4)].to_vec(), index) {
                                    Ok(data) => {
                                        index = data.1;
                                        entry.push(DecodedData::Integer(data.0));
                                    }
                                    Err(error) => return Err(error)
                                };
                            }
                            else {
                                return Err(Error::new(ErrorKind::Other, format!("Error: trying to decode an Integer without enough bytes.")))
                            }
                        }
                        schemas::FieldType::StringU8 => {
                            if index < packed_file_data.len() {
                                match coding_helpers::decode_packedfile_string_u8(packed_file_data[index..].to_vec(), index) {
                                    Ok(data) => {
                                        index = data.1;
                                        entry.push(DecodedData::StringU8(data.0));
                                    }
                                    Err(error) => return Err(error)
                                };
                            }
                            else {
                                return Err(Error::new(ErrorKind::Other, format!("Error: trying to decode a StringU8 without enought bytes.")))
                            }
                        }
                        schemas::FieldType::StringU16 => {
                            if index < packed_file_data.len() {
                                match coding_helpers::decode_packedfile_string_u16(packed_file_data[index..].to_vec(), index) {
                                    Ok(data) => {
                                        index = data.1;
                                        entry.push(DecodedData::StringU16(data.0));
                                    }
                                    Err(error) => return Err(error)
                                };
                            }
                            else {
                                return Err(Error::new(ErrorKind::Other, format!("Error: trying to decode a StringU16 without enought bytes.")))
                            }
                        }
                        schemas::FieldType::OptionalStringU8 => {
                            if index <= packed_file_data.len() {
                                match coding_helpers::decode_packedfile_optional_string_u8(packed_file_data[index..].to_vec(), index) {
                                    Ok(data) => {
                                        index = data.1;
                                        entry.push(DecodedData::OptionalStringU8(data.0));
                                    }
                                    Err(error) => return Err(error)
                                };
                            }
                            else {
                                return Err(Error::new(ErrorKind::Other, format!("Error: trying to decode an OptionalStringU8 without enought bytes.")))
                            }
                        }
                        schemas::FieldType::OptionalStringU16 => {
                            if index <= packed_file_data.len() {
                                match coding_helpers::decode_packedfile_optional_string_u16(packed_file_data[index..].to_vec(), index) {
                                    Ok(data) => {
                                        index = data.1;
                                        entry.push(DecodedData::OptionalStringU16(data.0));
                                    }
                                    Err(error) => return Err(error)
                                };
                            }
                            else {
                                return Err(Error::new(ErrorKind::Other, format!("Error: trying to decode an OptionalStringU16 without enought bytes.")))
                            }
                        }
                    }
                }
            }
            packed_file_data_decoded_rows.push(entry.clone());
        }

        // We return the structure of the DB PackedFile and his decoded data.
        packed_file_data_decoded = packed_file_data_decoded_rows;

        Ok(DBData {
            table_definition,
            packed_file_data: packed_file_data_decoded,
        })
    }

    /// This function takes an entire DBData and encode it to Vec<u8>, so it can be written in the disk.
    /// It returns a tuple with the encoded DBData in a Vec<u8> and the new entry count to update the
    /// header.
    pub fn save(packed_file_data_decoded: &DBData) -> Result<(Vec<u8>, u32), Error> {

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
                        let mut encoded_data = coding_helpers::encode_bool(data.clone());
                        packed_file_data_encoded.append(&mut encoded_data);
                    },
                    DecodedData::Float(data) => {
                        let mut encoded_data = coding_helpers::encode_float_u32(data.clone());
                        packed_file_data_encoded.append(&mut encoded_data);
                    },
                    DecodedData::Integer(data) => {
                        let mut encoded_data = coding_helpers::encode_integer_u32(data.clone());
                        packed_file_data_encoded.append(&mut encoded_data);
                    },
                    DecodedData::StringU8(ref data) => {
                        let mut encoded_data = coding_helpers::encode_packedfile_string_u8(data.clone());
                        packed_file_data_encoded.append(&mut encoded_data);
                    },
                    DecodedData::StringU16(ref data) => {
                        let mut encoded_data = coding_helpers::encode_packedfile_string_u16(data.clone());
                        packed_file_data_encoded.append(&mut encoded_data);
                    },
                    DecodedData::OptionalStringU8(ref data) => {
                        let mut encoded_data = coding_helpers::encode_packedfile_optional_string_u8(data.clone());
                        packed_file_data_encoded.append(&mut encoded_data);
                    },
                    DecodedData::OptionalStringU16(ref data) => {
                        let mut encoded_data = coding_helpers::encode_packedfile_optional_string_u16(data.clone());
                        packed_file_data_encoded.append(&mut encoded_data);
                    },
                }
            }
            packed_file_entry_count += 1;
        }
        Ok((packed_file_data_encoded, packed_file_entry_count))
    }
}
