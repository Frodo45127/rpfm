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

extern crate failure;
extern crate csv;
extern crate uuid;

use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

use self::uuid::Uuid;
use self::failure::Error;
use self::csv::{
    ReaderBuilder, WriterBuilder, QuoteStyle
};
use common::coding_helpers;
use super::SerializableToCSV;
use self::schemas::*;

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
    pub packed_file_header_packed_file_guid: String,
    pub packed_file_header_packed_file_version: u32,
    pub packed_file_header_packed_file_version_marker: bool,
    pub packed_file_header_packed_file_mysterious_byte: u8,
    pub packed_file_header_packed_file_entry_count: u32,
}

/// Struct DBData: This stores the data of a decoded DB PackedFile in memory.
/// It stores the PackedFile's data in a Vec<u8> and his structure in an OrderMap, if exists.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DBData {
    pub table_definition: TableDefinition,
    pub packed_file_data: Vec<Vec<DecodedData>>,
}

/// Enum DecodedData: This enum is used to store the data from the different fields of a row of a DB
/// PackedFile.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum DecodedData {
    Index(String),
    Boolean(bool),
    Float(f32),
    Integer(i32),
    LongInteger(i64),
    StringU8(String),
    StringU16(String),
    OptionalStringU8(String),
    OptionalStringU16(String),
}

/// Implementation of "DB"
impl DB {

    /// This function creates a new decoded DB from a encoded PackedFile. This assumes the PackedFile is
    /// a DB PackedFile. It'll crash otherwise.
    pub fn read(packed_file_data: &[u8], packed_file_db_type: &str, master_schema: &schemas::Schema) -> Result<DB, Error> {

        match DBHeader::read(packed_file_data) {
            Ok(packed_file_header) => {
                match DB::get_schema(packed_file_db_type, packed_file_header.0.packed_file_header_packed_file_version, master_schema) {
                    Some(table_definition) => {
                        match DBData::read(
                            &packed_file_data[(packed_file_header.1)..],
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
                    None => Err(format_err!("Schema for this DB Table not found"))
                }

            }
            Err(error) => Err(error)
        }
    }

    /// This function takes an entire DB and encode it to Vec<u8>, so it can be written in the disk.
    /// It returns a Vec<u8> with the entire DB encoded in it.
    pub fn save(packed_file_decoded: &DB) -> Result<Vec<u8>, Error> {

        let mut packed_file_data_encoded = DBData::save(&packed_file_decoded.packed_file_data);
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

    /// This function gets all the schemas corresponding to the table we passed it, if any of them exists.
    pub fn get_schema_versions_list(db_name: &str, schema: &schemas::Schema) -> Option<Vec<schemas::TableDefinition>> {

        // If we find our table in the TableDefinitions vector...
        if let Some(index_table_definitions) = schema.get_table_definitions(db_name) {

            // And it has at least one definition created...
            if !schema.tables_definitions[index_table_definitions].versions.is_empty() {

                // Return the vector with our tables's versions.
                return Some(schema.tables_definitions[index_table_definitions].versions.to_vec())
            }
        }
        None
    }

    /// This function removes from the schema the version of a table with the provided version.
    pub fn remove_table_version(table_name: &str, version: u32, schema: &mut schemas::Schema) -> Result<(), Error>{

        // If we find our table in the TableDefinitions vector...
        if let Some(index_table_definitions) = schema.get_table_definitions(table_name) {

            // And it has a definition created for our version of the table...
            if let Some(index_table_versions) = schema.tables_definitions[index_table_definitions].get_table_version(version) {

                // We remove that version and return success.
                schema.tables_definitions[index_table_definitions].versions.remove(index_table_versions);
                Ok(())
            }

            // If the table doesn't have the version we asked for...
            else {
                return Err(format_err!("Error while deleting the definition for version {} of table {}:\nThis table doesn't have this version decoded.", version, table_name));
            }
        }

        // If the table hasn't been found in the Schema...
        else {
            return Err(format_err!("Error while deleting the definition for version {} of table {}:\nThis table is not in the currently loaded Schema.", version, table_name));
        }
    }
}


/// Implementation of "DBHeader"
impl DBHeader {

    /// This function creates a new DBHeader from nothing. For the GUID, we generate a random GUID
    /// so, if for some reason RPFM fails to save the default GUID, we have one to back it up.
    pub fn new() -> DBHeader {
        let packed_file_header_packed_file_guid = format!("{}", Uuid::new_v4());
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
    pub fn read(packed_file_header: &[u8]) -> Result<(DBHeader, usize), Error> {

        // Create the default header and set the index to 0.
        let mut packed_file_header_decoded = DBHeader::new();
        let mut index: usize = 0;

        // If the first four bytes are the GUID_MARKER, or the VERSION_MARKER, we try to decode them. Otherwise,
        // it's a veeery old table (Empire maybe?). We skip the decoding of both of those fields and use the defaults,
        // as they will be written on save.
        if &packed_file_header[index..(index + 4)] == GUID_MARKER || &packed_file_header[index..(index + 4)] == VERSION_MARKER {

            // If it has a GUID_MARKER, we get his guid. Otherwise, we ignore it and use the default value.
            if &packed_file_header[index..(index + 4)] == GUID_MARKER {
                index += 4;
                packed_file_header_decoded.packed_file_header_packed_file_guid = coding_helpers::decode_packedfile_string_u16(&packed_file_header[index..], &mut index)?;
            }

            // If it has a VERSION_MARKER, we get the version of the table. Otherwise, use 0 as his version.
            if &packed_file_header[index..(index + 4)] == VERSION_MARKER {
                packed_file_header_decoded.packed_file_header_packed_file_version = coding_helpers::decode_integer_u32(&packed_file_header[(index + 4)..(index + 8)])?;
                packed_file_header_decoded.packed_file_header_packed_file_version_marker = true;
                index += 8;
            }
        }

        // We save a mysterious byte I don't know what it does.
        packed_file_header_decoded.packed_file_header_packed_file_mysterious_byte = packed_file_header[index];
        index += 1;

        packed_file_header_decoded.packed_file_header_packed_file_entry_count = coding_helpers::decode_packedfile_integer_u32(&packed_file_header[(index)..(index + 4)], &mut index)?;
        
        Ok((packed_file_header_decoded, index))
    }

    /// This function takes an entire DBHeader and a packed_file_entry_count, and encode it to Vec<u8>,
    /// so it can be written in the disk. It returns a Vec<u8> with the entire DBHeader encoded in it.
    pub fn save(packed_file_header_decoded: &DBHeader, packed_file_entry_count: u32) -> Vec<u8> {
        let mut packed_file_header_encoded: Vec<u8> = vec![];

        // We are always going to have a GUID_MARKER, so we add it directly.
        packed_file_header_encoded.extend_from_slice(GUID_MARKER);

        // If the GUID is empty, is a bugged table from RPFM 0.4.1 and below. Generate a
        // new GUID for it and use it.
        if packed_file_header_decoded.packed_file_header_packed_file_guid.is_empty() {
            let guid_encoded = coding_helpers::encode_packedfile_string_u16(&format!("{}", Uuid::new_v4()));
            packed_file_header_encoded.extend_from_slice(&guid_encoded);
        }

        // Otherwise, just encode the current GUID and put it in the encoded data vector.
        else {
            let guid_encoded = coding_helpers::encode_packedfile_string_u16(&packed_file_header_decoded.packed_file_header_packed_file_guid);
            packed_file_header_encoded.extend_from_slice(&guid_encoded);
        }


        if packed_file_header_decoded.packed_file_header_packed_file_version_marker {
            let version_encoded = coding_helpers::encode_integer_u32(packed_file_header_decoded.packed_file_header_packed_file_version);

            packed_file_header_encoded.extend_from_slice(VERSION_MARKER);
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
        packed_file_data: &[u8],
        table_definition: &TableDefinition,
        entry_count: u32
    ) -> Result<DBData, Error> {

        // Create the vector to store all the stuff inside this table.
        let mut table: Vec<Vec<DecodedData>> = vec![];

        // Create the almighty index.
        let mut index = 0;

        // Get the list of fields for our table.
        let fields_list = &table_definition.fields;

        // For each row in our list...
        for row_number in 0..entry_count {

            // If we decoded it...
            match DBData::read_row(packed_file_data, &fields_list, entry_count, row_number + 1, &mut index, false) {

                // If it succeed, add the row to the table, otherwise, return error.
                Ok(decoded_row) => table.push(decoded_row),
                Err(error) => return Err(error),
            }
        }

        // If there has been no errors, return the DBData.
        Ok(DBData {
            table_definition: table_definition.clone(),
            packed_file_data: table,
        })
    }

    /// This function takes an entire DBData and encode it to Vec<u8>, so it can be written in the disk.
    /// It returns a tuple with the encoded DBData in a Vec<u8> and the new entry count to update the
    /// header.
    pub fn save(packed_file_data_decoded: &DBData) -> (Vec<u8>, u32) {

        // Create the vector to store the encoded table's data.
        let mut table = vec![];

        // For each row on the list...
        for decoded_row in &packed_file_data_decoded.packed_file_data {

            // Encode it and add it to the list.
            table.append(&mut DBData::save_row(decoded_row));
        }

        // Return the encoded table and the amount of entries it has now.
        (table, packed_file_data_decoded.packed_file_data.len() as u32)
    }

    /// This function returns a `Vec<DecodedData>` or an error, depending if the entire row could be decoded or not.
    fn read_row(
        packed_file_data: &[u8],
        field_list: &[Field],
        entry_count: u32,
        row_number: u32,
        mut index: &mut usize,
        is_list: bool,
    ) -> Result<Vec<DecodedData>, Error> {

        // First, we get the amount of columns we have.
        let column_amount = field_list.len();

        // Create the Vector to store the decoded row.
        let mut decoded_row: Vec<DecodedData> = vec![];

        // If we are not using this to get a list's fields, add the index column to the row.
        if !is_list {
            decoded_row.push(DecodedData::Index(format!("{:0count$}", (row_number), count = (entry_count.to_string().len() + 1))));
        }

        // For each column...
        for column in 0..column_amount {

            // Get his field_type...
            let field_type = &field_list[column].field_type;

            // Decode it, depending of his type...
            match *field_type {

                // If it's a boolean field...
                FieldType::Boolean => {

                    // If the index exists...
                    if packed_file_data.get(*index).is_some() {

                        // Try to decode the field.
                        match coding_helpers::decode_packedfile_bool(packed_file_data[*index], &mut index) {

                            // If it succeed, add the row to the list.
                            Ok(data) => decoded_row.push(DecodedData::Boolean(data)),

                            // Otherwise, return error.
                            Err(error) => return Err(error)
                        };
                    }

                    // Otherwise, return error.
                    else { return Err(format_err!("Error: trying to decode a bool without a byte.")) }
                }

                // If it's a float field...
                FieldType::Float => {

                    // Check if the index does even exist, to avoid crashes. +3 because the ranges are exclusive.
                    if packed_file_data.get(*index + 3).is_some() {

                        // Try to decode the field.
                        match coding_helpers::decode_packedfile_float_f32(&packed_file_data[*index..(*index + 4)], &mut index) {

                            // If it succeed, add the row to the list.
                            Ok(data) => decoded_row.push(DecodedData::Float(data)),

                            // Otherwise, return error.
                            Err(error) => return Err(error)
                        };

                    }

                    // Otherwise, return error.
                    else { return Err(format_err!("Error: trying to decode a Float without enough bytes.")) }
                }

                // If it's an integer field...
                FieldType::Integer => {

                    // Check if the index does even exist, to avoid crashes. +3 because the ranges are exclusive.
                    if packed_file_data.get(*index + 3).is_some() {

                        // Try to decode the field.
                        match coding_helpers::decode_packedfile_integer_i32(&packed_file_data[*index..(*index + 4)], &mut index) {

                            // If it succeed, add the row to the list.
                            Ok(data) => decoded_row.push(DecodedData::Integer(data)),

                            // Otherwise, return error.
                            Err(error) => return Err(error)
                        };
                    }

                    // Otherwise, return error.
                    else { return Err(format_err!("Error: trying to decode a signed Integer without enough bytes.")) }
                }

                // If it's a long integer (i64)...
                FieldType::LongInteger => {

                    // Check if the index does even exist, to avoid crashes. +7 because the ranges are exclusive.
                    if packed_file_data.get(*index + 7).is_some() {

                        // Try to decode the field.
                        match coding_helpers::decode_packedfile_integer_i64(&packed_file_data[*index..(*index + 8)], &mut index) {

                            // If it succeed, add the row to the list.
                            Ok(data) => decoded_row.push(DecodedData::LongInteger(data)),

                            // Otherwise, return error.
                            Err(error) => return Err(error)
                        };
                    }

                    // Otherwise, return error.
                    else { return Err(format_err!("Error: trying to decode a signed Long Integer without enough bytes.")) }
                }

                // If it's a common StringU8...
                FieldType::StringU8 => {

                    // Check if the index does even exist, to avoid crashes. +1 because strings start with an u16 with his size.
                    if packed_file_data.get(*index + 1).is_some() {

                        // Try to decode the field.
                        match coding_helpers::decode_packedfile_string_u8(&packed_file_data[*index..], &mut index) {

                            // If it succeed, add the row to the list.
                            Ok(data) => decoded_row.push(DecodedData::StringU8(data)),

                            // Otherwise, return error.
                            Err(error) => return Err(error)
                        };
                    }

                    // Otherwise, return error.
                    else { return Err(format_err!("Error: trying to decode a StringU8 without enought bytes.")) }
                }

                // If it's a StringU16...
                FieldType::StringU16 => {

                    // Check if the index does even exist, to avoid crashes. +1 because strings start with an u16 with his size.
                    if packed_file_data.get(*index + 1).is_some() {

                        // Try to decode the field.
                        match coding_helpers::decode_packedfile_string_u16(&packed_file_data[*index..], &mut index) {

                            // If it succeed, add the row to the list.
                            Ok(data) => decoded_row.push(DecodedData::StringU16(data)),

                            // Otherwise, return error.
                            Err(error) => return Err(error)
                        };
                    }

                    // Otherwise, return error.
                    else { return Err(format_err!("Error: trying to decode a StringU16 without enought bytes.")) }
                }

                // If it's an optional StringU8...
                FieldType::OptionalStringU8 => {

                    // Check if the index does even exist, to avoid crashes.
                    if packed_file_data.get(*index).is_some() {

                        // Try to decode the field.
                        match coding_helpers::decode_packedfile_optional_string_u8(&packed_file_data[*index..], &mut index) {

                            // If it succeed, add the row to the list.
                            Ok(data) => decoded_row.push(DecodedData::OptionalStringU8(data)),

                            // Otherwise, return error.
                            Err(error) => return Err(error)
                        };
                    }
                    else { return Err(format_err!("Error: trying to decode an OptionalStringU8 without enought bytes.")) }
                }

                // If it's an optional StringU16...
                FieldType::OptionalStringU16 => {

                    // Check if the index does even exist, to avoid crashes.
                    if packed_file_data.get(*index).is_some() {

                        // Try to decode the field.
                        match coding_helpers::decode_packedfile_optional_string_u16(&packed_file_data[*index..], &mut index) {

                            // If it succeed, add the row to the list.
                            Ok(data) => decoded_row.push(DecodedData::OptionalStringU16(data)),

                            // Otherwise, return error.
                            Err(error) => return Err(error)
                        };
                    }
                    else { return Err(format_err!("Error: trying to decode an OptionalStringU16 without enought bytes.")) }
                }
            }
        }
        Ok(decoded_row)
    }

    /// This function returns a `Vec<u8>` with the encoded data of a row, or an error, depending if the data could be encoded or not.
    fn save_row(decoded_row: &[DecodedData]) -> Vec<u8> {

        // Create the vector to store the encoded row's data.
        let mut encoded_row: Vec<u8> = vec![];

        // For each field in the row...
        for field in decoded_row {

            // Depending on what kind of field it is we encode it and add it to the row.
            match *field {
                DecodedData::Index(_) => {

                    // We skip the index column, as we only have it for easy manipulation, it has
                    // nothing to do with the PackedFile.
                    continue;
                },
                DecodedData::Boolean(data) => {
                    let encoded_data = coding_helpers::encode_bool(data);
                    encoded_row.push(encoded_data);
                },
                DecodedData::Float(data) => {
                    let mut encoded_data = coding_helpers::encode_float_f32(data);
                    encoded_row.append(&mut encoded_data);
                },
                DecodedData::Integer(data) => {
                    let mut encoded_data = coding_helpers::encode_integer_i32(data);
                    encoded_row.append(&mut encoded_data);
                },
                DecodedData::LongInteger(data) => {
                    let mut encoded_data = coding_helpers::encode_integer_i64(data);
                    encoded_row.append(&mut encoded_data);
                },
                DecodedData::StringU8(ref data) => {
                    let mut encoded_data = coding_helpers::encode_packedfile_string_u8(data);
                    encoded_row.append(&mut encoded_data);
                },
                DecodedData::StringU16(ref data) => {
                    let mut encoded_data = coding_helpers::encode_packedfile_string_u16(data);
                    encoded_row.append(&mut encoded_data);
                },
                DecodedData::OptionalStringU8(ref data) => {
                    let mut encoded_data = coding_helpers::encode_packedfile_optional_string_u8(data);
                    encoded_row.append(&mut encoded_data);
                },
                DecodedData::OptionalStringU16(ref data) => {
                    let mut encoded_data = coding_helpers::encode_packedfile_optional_string_u16(data);
                    encoded_row.append(&mut encoded_data);
                },
            }
        }

        // Return the encoded row.
        encoded_row
    }
}


/// Implementation of `SerializableToCSV` for `DBData`.
impl SerializableToCSV for DBData {

    fn import_csv(&mut self, csv_file_path: &PathBuf) -> Result<(), Error> {

        // We expect no headers, so we need to tweak our reader first.
        let mut reader_builder = ReaderBuilder::new();
        reader_builder.has_headers(false);

        // Get the file and it's entries.
        match reader_builder.from_path(&csv_file_path) {
            Ok(mut reader) => {

                // We create here the vector to store the date while it's being decoded.
                let mut new_packed_file_data = vec![];

                // Then we add the new entries to the decoded entry list.
                for (index, reader_entry) in reader.records().enumerate() {

                    // If the entry record hasn't returned any error, we try decode it using the schema of the open table.
                    match reader_entry {
                        Ok(entry) => {

                            // We need to check if the length of the imported entries is the same than the one from the schema.
                            // If not, then we stop the import and return an error. This should avoid the problem with undecodeable
                            // tables after importing into them a CSV from another table that passes the schema filter from below.
                            if entry.len() == self.table_definition.fields.len() {
                                let mut entry_complete = vec![DecodedData::Index(format!("{}", index + 1))];
                                for (j, field) in entry.iter().enumerate() {
                                    match self.table_definition.fields[j].field_type {
                                        FieldType::Boolean => entry_complete.push(DecodedData::Boolean(field.parse::<bool>()?)),
                                        FieldType::Float => entry_complete.push(DecodedData::Float(field.parse::<f32>()?)),
                                        FieldType::Integer => entry_complete.push(DecodedData::Integer(field.parse::<i32>()?)),
                                        FieldType::LongInteger => entry_complete.push(DecodedData::LongInteger(field.parse::<i64>()?)),
                                        FieldType::StringU8 => entry_complete.push(DecodedData::StringU8(field.to_owned())),
                                        FieldType::StringU16 => entry_complete.push(DecodedData::StringU16(field.to_owned())),
                                        FieldType::OptionalStringU8 => entry_complete.push(DecodedData::OptionalStringU8(field.to_owned())),
                                        FieldType::OptionalStringU16 => entry_complete.push(DecodedData::OptionalStringU16(field.to_owned())),
                                    }
                                }
                                new_packed_file_data.push(entry_complete);
                            }
                            else {
                                return Err(format_err!("Error while trying import the csv file:\n{}\n\nIf you see this message, you probably tried to import a .csv file into a table with different structure.", &csv_file_path.display()));
                            }
                        }
                        Err(_) => return Err(format_err!("Error while trying import the csv file:\n{}", &csv_file_path.display())),
                    }
                }

                // If we reached this point without errors, we replace the old data with the new one.
                self.packed_file_data.clear();
                self.packed_file_data.append( &mut new_packed_file_data);

                Ok(())
            }
            Err(_) => Err(format_err!("Error while trying to read the csv file \"{}\".", &csv_file_path.display()))
        }
    }

    fn export_csv(&self, packed_file_path: &PathBuf) -> Result<String, Error> {

        // We want no headers and quotes around the fields, so we need to tweak our writer first.
        let mut writer_builder = WriterBuilder::new();
        writer_builder.has_headers(false);
        writer_builder.quote_style(QuoteStyle::Always);
        let mut writer = writer_builder.from_writer(vec![]);

        // For every entry, we serialize every one of it's fields (except the index).
        for entry in &self.packed_file_data {

            // We don't want the index, as that's not really needed outside the program.
            writer.serialize(&entry[1..])?;
        }

        // Get it all into an string, and write them to disk.
        let csv_serialized = String::from_utf8(writer.into_inner().unwrap().to_vec()).unwrap();
        match File::create(&packed_file_path) {
            Ok(mut file) => {
                match file.write_all(csv_serialized.as_bytes()) {
                    Ok(_) => Ok(format!("DB PackedFile successfully exported:\n{}", packed_file_path.display())),
                    Err(_) => Err(format_err!("Error while writing the following file to disk:\n{}", packed_file_path.display()))
                }
            }
            Err(_) => Err(format_err!("Error while trying to write the following file to disk:\n{}", packed_file_path.display()))
        }
    }
}
