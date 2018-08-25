// In this file we define the PackedFile type DB for decoding and encoding it.
// This is the type used by database files.

// The structure of a header is:
// - (optional) 4 bytes for the GUID marker.
// - (optional) The GUID in u16 bytes, with the 2 first being his lenght in chars (bytes / 2).
// - (optional) 4 bytes for the Version marker, if it have it.
// - (optional) 4 bytes for the Version, in u32 reversed.
// 1 misteryous byte
// 4 bytes for the entry count, in u32 reversed.

extern crate csv;
extern crate uuid;

use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

use self::uuid::Uuid;
use self::csv::{ ReaderBuilder, WriterBuilder, QuoteStyle };
use common::coding_helpers::*;
use super::SerializableToTSV;
use self::schemas::*;
use error::{Error, ErrorKind, Result};

pub mod schemas;
pub mod schemas_importer;

/// These two const are the markers we need to check in the header of every DB file.
const GUID_MARKER: &[u8] = &[253, 254, 252, 255];
const VERSION_MARKER: &[u8] = &[252, 253, 254, 255];

/// `DB`: This stores the data of a decoded DB PackedFile in memory.
/// It stores the PackedFile divided in 3 parts:
/// - db_type: the name of the table's definition (usually, db/"this_name"/yourtable).
/// - header: header of the PackedFile, decoded.
/// - data: data of the PackedFile, decoded.
#[derive(Clone)]
pub struct DB {
    pub db_type: String,
    pub header: DBHeader,
    pub data: DBData,
}

/// `DBHeader`: This stores the header of a decoded DB PackedFile in memory.
/// It stores the PackedFile's header in different parts:
/// - guid: a randomly generated GUID. We generate a random one when saving.
/// - version_marker: true if there is a version marker.
/// - version: the version of our tabledefinition used to decode/encode this table. If there is no VERSION_MARKER, we default to 0.
/// - mysterious_byte: don't know his use, but it's in all the tables.
/// - entry_count: the amount of entries in our table.
#[derive(Clone, Debug)]
pub struct DBHeader {
    pub guid: String,
    pub version_marker: bool,
    pub version: u32,
    pub mysterious_byte: u8,
    pub entry_count: u32,
}

/// `DBData`: This stores the data of a decoded DB PackedFile in memory.
/// It stores the PackedFile's header in different parts:
/// - table_definition: a copy of the tabledefinition used by this table, so we don't have to check the schema everywhere.
/// - entries: a list of decoded entries. This list is a Vec(rows) of a Vec(fields of a row) of DecodedData (decoded field).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DBData {
    pub table_definition: TableDefinition,
    pub entries: Vec<Vec<DecodedData>>,
}

/// `DecodedData`: This enum is used to store the data from the different fields of a row of a DB PackedFile.
#[derive(PartialEq, Clone, Debug, Serialize, Deserialize)]
pub enum DecodedData {
    Boolean(bool),
    Float(f32),
    Integer(i32),
    LongInteger(i64),
    StringU8(String),
    StringU16(String),
    OptionalStringU8(String),
    OptionalStringU16(String),
}

/// Implementation of "DB".
impl DB {

    /// This function creates a new empty DB PackedFile.
    pub fn new(db_type: &str, version: u32, table_definition: TableDefinition) -> Self {
        Self{
            db_type: db_type.to_owned(),
            header: DBHeader::new(version),
            data: DBData::new(table_definition),
        }
    }

    /// This function creates a new decoded DB from a encoded PackedFile. This assumes the PackedFile is
    /// a DB PackedFile. It'll crash otherwise.
    pub fn read(
        packed_file_data: &[u8],
        db_type: &str,
        master_schema: &schemas::Schema
    ) -> Result<Self> {

        // Create the index that we'll use to decode the entire table.
        let mut index = 0;

        // We try to read the header.
        let header = DBHeader::read(packed_file_data, &mut index)?;

        // These tables use the not-yet-implemented type "List" in the following versions:
        // - models_artillery: 0
        // - models_building: 3, 7.
        // - models_naval: 11.
        // - models_sieges: 2, 3.
        // So we disable everything for any problematic version of these tables.
        // TODO: Implement the needed type for these tables.
        if (db_type == "models_artillery_tables" && header.version == 0) ||
            (db_type == "models_building_tables" && (header.version == 3 ||
                                                    header.version == 7)) ||
            (db_type == "models_naval_tables" && header.version == 11) ||
            (db_type == "models_sieges_tables" && (header.version == 2 ||
                                                    header.version == 3))
        { return Err(ErrorKind::DBTableContainsListField)? }

        // Then, we try to get the schema for our table, if exists.
        match Self::get_schema(db_type, header.version, master_schema) {

            // If we got an schema...
            Some(table_definition) => {

                // We try to decode his rows.
                let data = DBData::read(&packed_file_data, table_definition, &header, &mut index)?;

                // Return the decoded DB file.
                Ok(Self {
                    db_type: db_type.to_owned(),
                    header,
                    data,
                })
            }

            // If we got nothing...
            None => {

                // If the table is empty, return his specific error.
                if header.entry_count == 0 { Err(ErrorKind::DBTableEmptyWithNoTableDefinition)? }

                // Otherwise, return the generic "No Table Definition" error.
                else { Err(ErrorKind::SchemaTableDefinitionNotFound)? }
            }
        }
    }

    /// This function takes an entire DB and encode it to Vec<u8>, so it can be written in the disk.
    /// It returns a Vec<u8> with the entire DB encoded in it.
    pub fn save(&self) -> Vec<u8> {

        // Encode the header.
        let mut packed_file = DBHeader::save(&self.header, self.data.entries.len() as u32);

        // Add the data to the encoded header.
        DBData::save(&self.data, &mut packed_file);

        // Return the encoded PackedFile.
        packed_file
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
    pub fn remove_table_version(table_name: &str, version: u32, schema: &mut schemas::Schema) -> Result<()> {

        // If we find our table in the TableDefinitions vector...
        if let Some(index_table_definitions) = schema.get_table_definitions(table_name) {

            // And it has a definition created for our version of the table...
            if let Some(index_table_versions) = schema.tables_definitions[index_table_definitions].get_table_version(version) {

                // We remove that version and return success.
                schema.tables_definitions[index_table_definitions].versions.remove(index_table_versions);
                return Ok(())
            }

            // Unless we break some code, this should never happen. If it does, crash so we can check it with Sentry.
            unreachable!();
        }

        // Unless we break some code, this should never happen. If it does, crash so we can check it with Sentry.
        unreachable!();
    }
}


/// Implementation of "DBHeader".
impl DBHeader {

    /// This function creates a new DBHeader from nothing. For the GUID, we generate a random GUID.
    /// This is the same Assembly Kit does every time you export a table (it generates a random GUID)
    /// so I guess the GUID doesn't really affects how the table works.
    pub fn new(version: u32) -> Self {
        Self {
            guid: format!("{}", Uuid::new_v4()),
            version_marker: true,
            version,
            mysterious_byte: 1,
            entry_count: 0,
        }
    }

    /// This function creates a decoded DBHeader from a encoded PackedFile. It also return an index,
    /// to know where the body starts.
    pub fn read(packed_file_header: &[u8], mut index: &mut usize) -> Result<Self> {

        // Create the default header and set the index to 0.
        let mut header = Self::new(0);

        // If the first four bytes are the GUID_MARKER, or the VERSION_MARKER, we try to decode them. Otherwise,
        // it's a veeery old table (Empire maybe?). We skip the decoding of both of those fields and use the defaults,
        // as they will be written on save.
        if &packed_file_header[*index..(*index + 4)] == GUID_MARKER || &packed_file_header[*index..(*index + 4)] == VERSION_MARKER {

            // If it has a GUID_MARKER, we get his guid. Otherwise, we ignore it and use the default value.
            if &packed_file_header[*index..(*index + 4)] == GUID_MARKER {
                *index += 4;
                header.guid = decode_packedfile_string_u16(&packed_file_header[*index..], &mut index)?;
            }

            // If it has a VERSION_MARKER, we get the version of the table. Otherwise, use 0 as his version.
            if &packed_file_header[*index..(*index + 4)] == VERSION_MARKER {
                header.version_marker = true;
                header.version = decode_integer_u32(&packed_file_header[(*index + 4)..(*index + 8)])?;
                *index += 8;
            }
        }

        // We save a mysterious byte I don't know what it does.
        header.mysterious_byte = packed_file_header[*index];
        *index += 1;

        // We get the number of entries.
        header.entry_count = decode_packedfile_integer_u32(&packed_file_header[(*index)..(*index + 4)], &mut index)?;

        // Return the header.
        Ok(header)
    }

    /// This function takes an entire DBHeader and the amount of entries we have, and encode it to Vec<u8>,
    /// so it can be written in the disk. It returns a Vec<u8> with the entire DBHeader encoded in it.
    fn save(&self, packed_file_entry_count: u32) -> Vec<u8> {

        // Create the vector for the encoded PackedFile.
        let mut packed_file: Vec<u8> = vec![];

        // We are always going to have a GUID_MARKER, so we add it directly.
        packed_file.extend_from_slice(GUID_MARKER);

        // If the GUID is empty, is a bugged table from RPFM 0.4.1 and below. Generate a
        // new GUID for it and use it.
        if self.guid.is_empty() { packed_file.extend_from_slice(&mut encode_packedfile_string_u16(&format!("{}", Uuid::new_v4()))); }

        // Otherwise, just encode the current GUID and put it in the encoded data vector.
        else { packed_file.extend_from_slice(&mut encode_packedfile_string_u16(&self.guid)); }

        // If there is a version marker, we add the version data.
        if self.version_marker {
            packed_file.extend_from_slice(VERSION_MARKER);
            packed_file.extend_from_slice(&mut encode_integer_u32(self.version));
        }

        // Then we add the mysterious_byte and the encoded entry count.
        packed_file.push(self.mysterious_byte);
        packed_file.extend_from_slice(&mut encode_integer_u32(packed_file_entry_count));

        // And finally return the encoded header.
        packed_file
    }
}


/// Implementation of "DBData".
impl DBData {

    /// This function creates a new empty DBData.
    pub fn new(table_definition: TableDefinition) -> Self {
        Self {
            table_definition,
            entries: vec![]
        }
    }

    /// This function creates a decoded DBData from a encoded PackedFile's data.
    pub fn read(
        packed_file_data: &[u8],
        table_definition: TableDefinition,
        header: &DBHeader,
        mut index: &mut usize,
    ) -> Result<Self> {

        // Create the new DBData.
        let mut data = Self::new(table_definition);

        {
            // Get the list of fields for our table.
            let fields_list = &data.table_definition.fields;

            // For each row in our list...
            for _ in 0..header.entry_count {

                // If we decoded it...
                match Self::read_row(packed_file_data, &fields_list, &mut index) {

                    // If it succeed, add the row to the table, otherwise, return error.
                    Ok(decoded_row) => data.entries.push(decoded_row),
                    Err(error) => return Err(error),
                }
            }
        }

        // If there has been no errors, return the DBData.
        Ok(data)
    }

    /// This function returns a `Vec<DecodedData>` or an error, depending if the entire row could be decoded or not.
    fn read_row(
        packed_file_data: &[u8],
        field_list: &[Field],
        mut index: &mut usize,
    ) -> Result<Vec<DecodedData>> {

        // First, we get the amount of columns we have.
        let column_amount = field_list.len();

        // Create the Vector to store the decoded row.
        let mut decoded_row: Vec<DecodedData> = vec![];

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

                        // Try to decode the field. If it succeed, add the field to the row. Otherwise, return error.
                        match decode_packedfile_bool(packed_file_data[*index], &mut index) {
                            Ok(data) => decoded_row.push(DecodedData::Boolean(data)),
                            Err(error) => return Err(error)
                        };
                    }

                    // Otherwise, return error.
                    else { return Err(ErrorKind::HelperDecodingEncodingError("<p>Error trying to decode a boolean value: insufficient bytes to decode.</p>".to_owned()))? }
                }

                // If it's a float field...
                FieldType::Float => {

                    // Check if the index does even exist, to avoid crashes. +3 because the ranges are exclusive.
                    if packed_file_data.get(*index + 3).is_some() {

                        // Try to decode the field. If it succeed, add the field to the row. Otherwise, return error.
                        match decode_packedfile_float_f32(&packed_file_data[*index..(*index + 4)], &mut index) {
                            Ok(data) => decoded_row.push(DecodedData::Float(data)),
                            Err(error) => return Err(error)
                        };

                    }

                    // Otherwise, return error.
                    else { return Err(ErrorKind::HelperDecodingEncodingError("<p>Error trying to decode an f32 number: insufficient bytes to decode.</p>".to_owned()))? }
                }

                // If it's an integer field...
                FieldType::Integer => {

                    // Check if the index does even exist, to avoid crashes. +3 because the ranges are exclusive.
                    if packed_file_data.get(*index + 3).is_some() {

                        // Try to decode the field. If it succeed, add the field to the row. Otherwise, return error.
                        match decode_packedfile_integer_i32(&packed_file_data[*index..(*index + 4)], &mut index) {
                            Ok(data) => decoded_row.push(DecodedData::Integer(data)),
                            Err(error) => return Err(error)
                        };
                    }

                    // Otherwise, return error.
                    else { return Err(ErrorKind::HelperDecodingEncodingError("<p>Error trying to decode an i32 number: insufficient bytes to decode.</p>".to_owned()))? }
                }

                // If it's a long integer (i64)...
                FieldType::LongInteger => {

                    // Check if the index does even exist, to avoid crashes. +7 because the ranges are exclusive.
                    if packed_file_data.get(*index + 7).is_some() {

                        // Try to decode the field. If it succeed, add the field to the row. Otherwise, return error.
                        match decode_packedfile_integer_i64(&packed_file_data[*index..(*index + 8)], &mut index) {
                            Ok(data) => decoded_row.push(DecodedData::LongInteger(data)),
                            Err(error) => return Err(error)
                        };
                    }

                    // Otherwise, return error.
                    else { return Err(ErrorKind::HelperDecodingEncodingError("<p>Error trying to decode an i64 number: insufficient bytes to decode.</p>".to_owned()))? }
                }

                // If it's a common StringU8...
                FieldType::StringU8 => {

                    // Check if the index does even exist, to avoid crashes. +1 because strings start with an u16 with his size.
                    if packed_file_data.get(*index + 1).is_some() {

                        // Try to decode the field. If it succeed, add the field to the row. Otherwise, return error.
                        match decode_packedfile_string_u8(&packed_file_data[*index..], &mut index) {
                            Ok(data) => decoded_row.push(DecodedData::StringU8(data)),
                            Err(error) => return Err(error)
                        };
                    }

                    // Otherwise, return error.
                    else { return Err(ErrorKind::HelperDecodingEncodingError("<p>Error trying to decode an UTF-8 string: insufficient bytes to decode.</p>".to_owned()))? }
                }

                // If it's a StringU16...
                FieldType::StringU16 => {

                    // Check if the index does even exist, to avoid crashes. +1 because strings start with an u16 with his size.
                    if packed_file_data.get(*index + 1).is_some() {

                        // Try to decode the field. If it succeed, add the field to the row. Otherwise, return error.
                        match decode_packedfile_string_u16(&packed_file_data[*index..], &mut index) {
                            Ok(data) => decoded_row.push(DecodedData::StringU16(data)),
                            Err(error) => return Err(error)
                        };
                    }

                    // Otherwise, return error.
                    else { return Err(ErrorKind::HelperDecodingEncodingError("<p>Error trying to decode an UTF-16 string: insufficient bytes to decode.</p>".to_owned()))? }
                }

                // If it's an optional StringU8...
                FieldType::OptionalStringU8 => {

                    // Check if the index does even exist, to avoid crashes.
                    if packed_file_data.get(*index).is_some() {

                        // Try to decode the field. If it succeed, add the field to the row. Otherwise, return error.
                        match decode_packedfile_optional_string_u8(&packed_file_data[*index..], &mut index) {
                            Ok(data) => decoded_row.push(DecodedData::OptionalStringU8(data)),
                            Err(error) => return Err(error)
                        };
                    }
                    else { return Err(ErrorKind::HelperDecodingEncodingError("<p>Error trying to decode an optional UTF-8 string: insufficient bytes to decode.</p>".to_owned()))? }
                }

                // If it's an optional StringU16...
                FieldType::OptionalStringU16 => {

                    // Check if the index does even exist, to avoid crashes.
                    if packed_file_data.get(*index).is_some() {

                        // Try to decode the field. If it succeed, add the field to the row. Otherwise, return error.
                        match decode_packedfile_optional_string_u16(&packed_file_data[*index..], &mut index) {
                            Ok(data) => decoded_row.push(DecodedData::OptionalStringU16(data)),
                            Err(error) => return Err(error)
                        };
                    }
                    else { return Err(ErrorKind::HelperDecodingEncodingError("<p>Error trying to decode an optional UTF-16 string: insufficient bytes to decode.</p>".to_owned()))? }
                }
            }
        }
        Ok(decoded_row)
    }

    /// This function takes an entire DBData, encode it to Vec<u8> and add it to the supplied Vec<u8>.
    fn save(&self, packed_file: &mut Vec<u8>) {
        packed_file.append(&mut self.entries.iter().flat_map(|x| Self::save_row(x)).collect::<Vec<u8>>());
    }

    /// This function returns a `Vec<u8>` with the encoded data of a row.
    fn save_row(decoded_row: &[DecodedData]) -> Vec<u8> {

        // Create the vector to store the encoded row's data.
        let mut encoded_row: Vec<u8> = vec![];

        // For each field in the row...
        for field in decoded_row {

            // Depending on what kind of field it is we encode it and add it to the row.
            match *field {

                // In case of index, skip the column. We don't need it other than for the UI.
                DecodedData::Boolean(data) => encoded_row.push(encode_bool(data)),
                DecodedData::Float(data) => encoded_row.append(&mut encode_float_f32(data)),
                DecodedData::Integer(data) => encoded_row.append(&mut encode_integer_i32(data)),
                DecodedData::LongInteger(data) => encoded_row.append(&mut encode_integer_i64(data)),
                DecodedData::StringU8(ref data) => encoded_row.append(&mut encode_packedfile_string_u8(data)),
                DecodedData::StringU16(ref data) => encoded_row.append(&mut encode_packedfile_string_u16(data)),
                DecodedData::OptionalStringU8(ref data) => encoded_row.append(&mut encode_packedfile_optional_string_u8(data)),
                DecodedData::OptionalStringU16(ref data) => encoded_row.append(&mut encode_packedfile_optional_string_u16(data)),
            }
        }

        // Return the encoded row.
        encoded_row
    }
}


/// Implementation of `SerializableToTSV` for `DBData`.
impl SerializableToTSV for DBData {

    /// This function imports a TSV file and loads his contents into a DB Table.
    fn import_tsv(&mut self, tsv_file_path: &PathBuf, packed_file_type: &str) -> Result<()> {

        // We want the reader to have no quotes, tab as delimiter and custom headers, because otherwise
        // Excel, Libreoffice and all the programs that edit this kind of files break them on save.
        match ReaderBuilder::new()
            .delimiter(b'\t')
            .quoting(false)
            .has_headers(false)
            .flexible(true)
            .from_path(&tsv_file_path) {

            // If we succesfully read the TSV file into a reader...
            Ok(mut reader) => {

                // We create here the vector to store the data while it's being decoded.
                let mut packed_file_data = vec![];

                // For each entry in the reader...
                for (index, reader_entry) in reader.records().enumerate() {

                    // We use the first entry to make sure this TSV file belongs to this table.
                    if index == 0 {
                        match reader_entry {
                            Ok(entry) => {

                                // Get his original table's name and version.
                                let table_name = entry.get(0).unwrap_or("error");
                                let table_version = entry.get(1).unwrap_or("99999").parse::<u32>().unwrap_or(99999);

                                // If the name or version are the defaults, return error.
                                if table_name == "error" || table_version == 99999 {
                                    return Err(ErrorKind::ImportTSVIncorrectFirstRow)?;
                                }

                                // If any of them doesn't match the name and version of the table we are importing to, return error.
                                if table_name != packed_file_type || table_version != self.table_definition.version {
                                    return Err(ErrorKind::ImportTSVWrongType)?;
                                }
                            }

                            // If it fails, return error.
                            Err(_) => return Err(ErrorKind::ImportTSVIncorrectFirstRow)?,
                        }
                    }

                    // The rest of the lines should be decoded normally.
                    else {

                        // If the entry record hasn't returned any error, we try decode it using the schema of the open table.
                        match reader_entry {
                            Ok(entry) => {

                                // We need to check if the length of the imported entries is the same than the one from the schema.
                                // If not, then we stop the import and return an error. This should avoid the problem with undecodeable
                                // tables after importing into them a TSV from another table that passes the schema filter from below.
                                if entry.len() == self.table_definition.fields.len() {
                                    let mut entry_complete = vec![];
                                    for (index, field) in entry.iter().enumerate() {
                                        match self.table_definition.fields[index].field_type {
                                            FieldType::Boolean => entry_complete.push(DecodedData::Boolean(field.parse::<bool>().map_err(|_| Error::from(ErrorKind::DBTableParse))?)),
                                            FieldType::Float => entry_complete.push(DecodedData::Float(field.parse::<f32>().map_err(|_| Error::from(ErrorKind::DBTableParse))?)),
                                            FieldType::Integer => entry_complete.push(DecodedData::Integer(field.parse::<i32>().map_err(|_| Error::from(ErrorKind::DBTableParse))?)),
                                            FieldType::LongInteger => entry_complete.push(DecodedData::LongInteger(field.parse::<i64>().map_err(|_| Error::from(ErrorKind::DBTableParse))?)),
                                            FieldType::StringU8 => entry_complete.push(DecodedData::StringU8(field.to_owned())),
                                            FieldType::StringU16 => entry_complete.push(DecodedData::StringU16(field.to_owned())),
                                            FieldType::OptionalStringU8 => entry_complete.push(DecodedData::OptionalStringU8(field.to_owned())),
                                            FieldType::OptionalStringU16 => entry_complete.push(DecodedData::OptionalStringU16(field.to_owned())),
                                        }
                                    }
                                    packed_file_data.push(entry_complete);
                                }

                                // If the entry lenght doesn't match with the one of the current table, return error.
                                else {
                                    return Err(ErrorKind::ImportTSVIncompatibleFile)?;
                                }
                            }

                            // If it fails, return error.
                            Err(error) => return Err(Error::from(error))
                        }
                    }
                }

                // If we reached this point without errors, we replace the old data with the new one.
                self.entries = packed_file_data;

                // Return success.
                Ok(())
            }

            // If we couldn't read the TSV file, return error.
            Err(error) => Err(Error::from(error))
        }
    }

    /// This function creates a TSV file with the contents of the DB Table.
    fn export_tsv(&self, packed_file_path: &PathBuf, table_info: (&str, u32)) -> Result<String> {

        // We want the writer to have no quotes, tab as delimiter and custom headers, because otherwise
        // Excel, Libreoffice and all the programs that edit this kind of files break them on save.
        let mut writer = WriterBuilder::new()
            .delimiter(b'\t')
            .quote_style(QuoteStyle::Never)
            .has_headers(false)
            .flexible(true)
            .from_writer(vec![]);

        // We serialize the info of the table in the first line, so we can use it in the future to create tables from a TSV.
        writer.serialize(table_info)?;

        // For every entry, we serialize every one of his fields (except the index).
        for entry in &self.entries {

            // We don't want the index, as that's not really needed outside the program.
            writer.serialize(&entry)?;
        }

        // Then, we try to write it on disk. If there is an error, report it.
        match File::create(&packed_file_path) {
            Ok(mut file) => {
                match file.write_all(String::from_utf8(writer.into_inner().unwrap())?.as_bytes()) {
                    Ok(_) => Ok(format!("<p>DB PackedFile successfully exported:</p><ul><li>{}</li></ul>", packed_file_path.display())),
                    Err(_) => Err(ErrorKind::IOGenericWrite(vec![packed_file_path.display().to_string();1]))?
                }
            }
            Err(_) => Err(ErrorKind::IOGenericWrite(vec![packed_file_path.display().to_string();1]))?
        }
    }
}
