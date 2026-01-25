//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Assembly Kit table data parsing and conversion.
//!
//! This module handles the parsing of Assembly Kit sample table data files and their
//! conversion to RPFM's internal table format. These files contain actual row data
//! that can be used for testing, lookup generation, and schema validation.
//!
//! # Overview
//!
//! Assembly Kit provides not only table structure definitions (see the `table_definition` module)
//! but also sample data files containing actual table rows. These XML files are useful for:
//!
//! - **Schema validation**: Verifying field types match actual data
//! - **Lookup generation**: Extracting hardcoded enum/lookup values from descriptions
//! - **Testing**: Ensuring RPFM can correctly parse real game data
//! - **Reference**: Understanding what values appear in specific fields
//!
//! # File Format
//!
//! Table data files are XML files with the same name as their corresponding definition
//! files (without the `TWaD_` prefix). For example:
//! - Definition: `TWaD_units_tables.xml`
//! - Data: `units_tables.xml`
//!
//! Each file contains rows of data in XML format:
//! ```xml
//! <dataroot>
//!   <units_tables>
//!     <key>unit_1</key>
//!     <category>infantry</category>
//!     <is_naval>false</is_naval>
//!   </units_tables>
//!   <units_tables>
//!     <key>unit_2</key>
//!     <category>cavalry</category>
//!     <is_naval>false</is_naval>
//!   </units_tables>
//! </dataroot>
//! ```
//!
//! # Main Types
//!
//! - [`RawTable`]: Complete table with definition and all row data
//! - [`RawTableRow`]: Single row of data
//! - [`RawTableField`]: Individual field value within a row
//!
//! # Functionality
//!
//! The primary operations are:
//!
//! 1. **Batch Reading**: [`RawTable::read_all()`] reads all table data files from a directory
//! 2. **Individual Reading**: [`RawTable::read()`] parses a single table data file
//! 3. **Conversion to DB**: [`RawTable::to_db()`] converts to RPFM's [`DB`] format
//! 4. **Conversion to Table**: [`RawTable::to_table()`] converts to in-memory table format
//!
//! # Workarounds and Special Handling
//!
//! ## Missing Fields
//!
//! Some games (Thrones, Attila, Rome 2, Shogun 2) omit fields from rows when the field
//! value is empty. RPFM handles this by inserting default values for missing fields.
//!
//! ## Empty Field Markers
//!
//! Due to XML parser limitations, empty fields are temporarily filled with placeholder
//! text (`"Frodo Best Waifu"`) which is removed after parsing.
//!
//! ## Field Renaming
//!
//! The XML parser requires uniform field names, so table-specific field names are
//! replaced with generic `<datafield>` tags before parsing, with the original name
//! stored as an attribute.

use rayon::prelude::*;
use regex::Regex;
use serde_derive::Deserialize;
use serde_xml_rs::from_reader;

use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

use crate::error::{Result, RLibError};
use crate::files::{db::DB, table::{DecodedData, local::TableInMemory, Table}};
use crate::schema::{Definition, FieldType};

use super::table_definition::RawDefinition;

//---------------------------------------------------------------------------//
// Types for parsing the Assembly Kit DB Files into.
//---------------------------------------------------------------------------//

/// Complete table data parsed from Assembly Kit XML files.
///
/// This represents an entire table including its structure definition and all row data.
/// Corresponds to a `.xml` data file in the Assembly Kit (e.g., `units_tables.xml`).
///
/// # Structure
///
/// The table contains:
/// - An optional definition (field structure) - typically populated during parsing
/// - All rows of data from the XML file
///
/// # Usage
///
/// After parsing with [`RawTable::read()`] or [`RawTable::read_all()`], the table
/// can be converted to RPFM's internal formats:
/// - [`RawTable::to_db()`] - Convert to DB format for saving as a PackFile table
/// - [`RawTable::to_table()`] - Convert to in-memory table for manipulation
#[derive(Debug, Default, Deserialize)]
#[serde(rename = "dataroot")]
pub struct RawTable {
    /// Table structure definition (fields, types, relationships).
    ///
    /// This is populated by combining the parsed data structure with the
    /// corresponding `TWaD_` definition file.
    pub definition: Option<RawDefinition>,

    /// All rows of data in the table.
    pub rows: Vec<RawTableRow>,
}

/// Single row of data from an Assembly Kit table.
///
/// Each row contains a collection of field values. In the XML, this corresponds
/// to one `<tablename>` element containing multiple field elements.
#[derive(Debug, Default, Deserialize)]
#[serde(rename = "datarow")]
pub struct RawTableRow {

    /// All field values in this row.
    #[serde(rename = "datafield")]
    pub fields: Vec<RawTableField>,
}

/// Individual field value within a table row.
///
/// This is the raw equivalent to RPFM's [`DecodedData`]. Each field has a name,
/// a string value, and optionally a "state" flag marking localisable fields.
///
/// # XML Representation
///
/// In the original Assembly Kit XML, fields appear as:
/// ```xml
/// <field_name>value</field_name>
/// <other_field some_attribute="...">value with attributes</other_field>
/// ```
///
/// During parsing, these are normalized to:
/// ```xml
/// <datafield field_name="field_name">value</datafield>
/// <datafield field_name="other_field" state="1">value with attributes</datafield>
/// ```
///
/// # State Attribute for Localisable Fields
///
/// The `state` attribute is set to `"1"` when the original XML field tag had any
/// attributes. In Assembly Kit files, fields with attributes are localisable fields
/// (fields containing translatable text). These fields are filtered out when extracting
/// non-localisable field definitions, ensuring that regular data fields and translation
/// fields are processed separately.
#[derive(Debug, Default, Deserialize)]
#[serde(rename = "datafield")]
pub struct RawTableField {
    /// Name of the field (column name).
    pub field_name: String,

    /// String representation of the field value.
    ///
    /// All values are stored as strings in XML and must be parsed to their
    /// actual types during conversion.
    #[serde(rename = "$value")]
    pub field_data: String,

    /// State flag marking localisable (translatable) fields.
    ///
    /// Set to `"1"` when the original Assembly Kit XML field tag had any attributes,
    /// which indicates the field is localisable (contains translatable text).
    /// Such fields are filtered out during non-localisable field extraction to ensure
    /// translation fields are handled separately from regular data fields.
    pub state: Option<String>,
}

//---------------------------------------------------------------------------//
// Implementations
//---------------------------------------------------------------------------//

/// Implementation of `RawTable`.
impl RawTable {

    /// Reads all table data files from an Assembly Kit directory.
    ///
    /// This function scans the directory for table data XML files and parses them
    /// into [`RawTable`] instances. It first reads all table definitions, then
    /// reads the corresponding data files.
    ///
    /// # Arguments
    ///
    /// * `raw_tables_folder` - Directory containing both definition and data files
    /// * `version` - Assembly Kit version (0-2)
    /// * `tables_to_skip` - Table names to exclude from parsing
    ///
    /// # Returns
    ///
    /// Returns a vector of successfully parsed tables. Tables that fail to parse
    /// or are in the skip list are excluded.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The version is unsupported (not 0, 1, or 2)
    /// - The directory cannot be read
    /// - Definition files cannot be parsed
    ///
    /// # Note
    ///
    /// Individual table data files that fail to parse are silently skipped rather
    /// than causing the entire operation to fail.
    pub fn read_all(raw_tables_folder: &Path, version: i16, tables_to_skip: &[&str]) -> Result<Vec<Self>> {

        // First, we try to read all `RawDefinitions` from the same folder.
        let definitions = RawDefinition::read_all(raw_tables_folder, version, tables_to_skip)?;

        // Then, depending on the version, we have to use one logic or another.
        match version {

            // Version 2 is Rome 2+. Version 1 is Shogun 2. Almost the same format, but we have to
            // provide a different path for Shogun 2, so it has his own version.
            // Version 0 is Napoleon and Empire. These two don't have an assembly kit, but CA released years ago their table files.
            0..=2 => Ok(definitions.par_iter().filter_map(|definition| Self::read(definition, raw_tables_folder, version).ok()).collect()),
            _ => Err(RLibError::AssemblyKitUnsupportedVersion(version))
        }
    }

    /// Parses a single Assembly Kit table data file.
    ///
    /// Reads the XML data file corresponding to the provided definition and parses
    /// it into a [`RawTable`]. The data file must have the same name as the definition
    /// (without the `TWaD_` prefix).
    ///
    /// # Arguments
    ///
    /// * `raw_definition` - Table structure definition
    /// * `raw_table_data_folder` - Directory containing the data XML files
    /// * `version` - Assembly Kit version (0-2)
    ///
    /// # Returns
    ///
    /// Returns a [`RawTable`] with the definition and all parsed row data.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The version is unsupported (not 0, 1, or 2)
    /// - The data file cannot be opened
    /// - The XML is malformed
    /// - The table is `translated_texts.xml` (returns [`RLibError::AssemblyKitTableTableIgnored`])
    ///
    /// # Special Cases
    ///
    /// ## translated_texts.xml
    ///
    /// This file (present in Rome 2, Attila, Thrones) is ~400MB and not needed for
    /// schema processing, so it's explicitly ignored.
    ///
    /// ## XML Preprocessing
    ///
    /// Before parsing, the XML undergoes several transformations to work around
    /// `serde_xml_rs` limitations:
    /// 1. Table-specific row tags are renamed to generic `<rows>`
    /// 2. Field tags are renamed to `<datafield>` with the name as an attribute
    /// 3. Empty fields are filled with placeholder text (removed after parsing)
    pub fn read(raw_definition: &RawDefinition, raw_table_data_folder: &Path, version: i16) -> Result<Self> {
        match version {
            0..=2 => {
                let name_no_xml = raw_definition.name.as_ref().unwrap().split_at(raw_definition.name.as_ref().unwrap().len() - 4).0;

                // This file is present in Rome 2, Attila and Thrones. It's almost 400mb. And we don't need it.
                if raw_definition.name.as_ref().unwrap() == "translated_texts.xml" {
                    return Err(RLibError::AssemblyKitTableTableIgnored)
                }

                let raw_table_data_path = raw_table_data_folder.join(raw_definition.name.as_ref().unwrap());
                let mut raw_table_data_file = BufReader::new(File::open(raw_table_data_path)?);

                // Before deserializing the data, due to limitations of serde_xml_rs, we have to rename all rows, because unique names for
                // rows in each file is not supported for deserializing. Same for the fields, we have to change them to something more generic.
                let mut buffer = String::new();
                raw_table_data_file.read_to_string(&mut buffer)?;
                buffer = buffer.replace(&format!("<{name_no_xml} record_uuid"), "<rows record_uuid");
                buffer = buffer.replace(&format!("<{name_no_xml}>"), "<rows>");
                buffer = buffer.replace(&format!("</{name_no_xml}>"), "</rows>");
                for field in &raw_definition.fields {
                    let field_name_regex = Regex::new(&format!("\n<{}>", field.name)).unwrap();
                    let field_name_regex2 = Regex::new(&format!("\n<{} .+?\">", field.name)).unwrap();
                    buffer = field_name_regex.replace_all(&buffer, &*format!("\n<datafield field_name=\"{}\">", field.name)).to_string();
                    buffer = field_name_regex2.replace_all(&buffer, &*format!("\n<datafield field_name=\"{}\" state=\"1\">", field.name)).to_string();
                    buffer = buffer.replace(&format!("</{}>", field.name), "</datafield>");
                }

                // Serde shits itself if it sees an empty field, so we have to work around that.
                buffer = buffer.replace("\"></datafield>", "\">Frodo Best Waifu</datafield>");
                buffer = buffer.replace("\"> </datafield>", "\"> Frodo Best Waifu</datafield>");
                buffer = buffer.replace("\">  </datafield>", "\">  Frodo Best Waifu</datafield>");
                buffer = buffer.replace("\">   </datafield>", "\">   Frodo Best Waifu</datafield>");
                buffer = buffer.replace("\">    </datafield>", "\">    Frodo Best Waifu</datafield>");

                // Only if the table has data we deserialize it. If not, we just create an empty one.
                let mut raw_table = if buffer.contains("</rows>\r\n</dataroot>") {
                    from_reader(buffer.as_bytes())?
                } else {
                    Self::default()
                };

                // Remove the best waifus, because they end up appearing in lookups!!!
                for row in &mut raw_table.rows {
                    for field in &mut row.fields {
                        field.field_data = field.field_data.replace("Frodo Best Waifu", "").trim().to_owned();
                    }
                }

                raw_table.definition = Some(raw_definition.clone());
                Ok(raw_table)
            }
            _ => Err(RLibError::AssemblyKitUnsupportedVersion(version))
        }
    }

    /// Converts the raw table to RPFM's DB format.
    ///
    /// This is a convenience wrapper around [`RawTable::to_table()`] that converts
    /// the result to a [`DB`] struct suitable for saving as a PackFile table.
    ///
    /// # Arguments
    ///
    /// * `definition` - Optional RPFM schema definition for type validation and patching
    ///
    /// # Returns
    ///
    /// Returns a [`DB`] instance containing the converted table data.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The table has no definition
    /// - Field types cannot be determined
    /// - Data conversion fails (e.g., invalid number format)
    pub fn to_db(&self, definition: Option<&Definition>) -> Result<DB> {
        let table = Self::to_table(self, definition)?;
        Ok(DB::from(table))
    }

    /// Converts the raw table to RPFM's in-memory table format.
    ///
    /// This function performs the main conversion from Assembly Kit's XML representation
    /// to RPFM's internal table structure, including type conversion and handling of
    /// missing fields.
    ///
    /// # Arguments
    ///
    /// * `definition` - Optional RPFM schema definition used for:
    ///   - Type validation and patching (e.g., fixing string types on empty fields)
    ///   - Providing default values for missing fields
    ///
    /// # Returns
    ///
    /// Returns a [`TableInMemory`] with all data converted to proper types.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The raw table has no definition (returns [`RLibError::RawTableMissingDefinition`])
    /// - Field data cannot be parsed to the expected type
    /// - The table structure is invalid
    ///
    /// # Type Conversion
    ///
    /// String values from XML are converted to typed data:
    /// - `"true"`, `"1"` → `Boolean(true)`
    /// - `"123"` → `I32(123)`, `F32(123.0)`, etc.
    /// - `""` → Appropriate default value for the type
    ///
    /// # Missing Field Handling
    ///
    /// Some games (Thrones, Attila, Rome 2, Shogun 2) omit empty fields from rows.
    /// This function inserts default values for any missing fields based on their type.
    pub fn to_table(&self, definition: Option<&Definition>) -> Result<TableInMemory> {
        let mut raw_definition = self.definition.as_ref().cloned().ok_or(RLibError::RawTableMissingDefinition)?;
        let table_name = if let Some(ref raw_definition) = raw_definition.name {

            // Remove the .xml of the name in the most awesome way there is.
            let mut x = raw_definition.to_owned();
            x.pop();
            x.pop();
            x.pop();
            x.pop();

            format!("{x}_tables")
        } else { String::new() };

        // We need to pre-patch some of the raw definition fields to avoid the "0 on empty fields" bug.
        if let Some(definition) = definition {
            for field in definition.fields_processed() {
                if let Some(raw_field) = raw_definition.fields.iter_mut().find(|x| x.name == field.name()) {
                    match field.field_type() {
                        FieldType::StringU8 |
                        FieldType::OptionalStringU8 => {
                            if raw_field.field_type == "integer" {
                                raw_field.field_type = "text".to_owned();
                            }
                        },
                        _ => continue,
                    }
                }
            }
        }

        let mut table = TableInMemory::new(&From::from(&raw_definition), None, &table_name);
        let mut entries = vec![];
        for row in &self.rows {
            let mut entry = vec![];

            // Some games (Thrones, Attila, Rome 2 and Shogun 2) may have missing fields when said field is empty.
            // To compensate it, if we don't find a field from the definition in the table, we add it empty.
            for field_def in table.definition().fields() {
                let mut exists = false;
                for field in &row.fields {
                    if field_def.name() == field.field_name {
                        exists = true;

                        entry.push(match field_def.field_type() {
                            FieldType::Boolean => DecodedData::Boolean(field.field_data == "true" || field.field_data == "1"),
                            FieldType::F32 => DecodedData::F32(field.field_data.parse::<f32>().unwrap_or_default()),
                            FieldType::F64 => DecodedData::F64(field.field_data.parse::<f64>().unwrap_or_default()),
                            FieldType::I16 => DecodedData::I16(field.field_data.parse::<i16>().unwrap_or_default()),
                            FieldType::I32 => DecodedData::I32(field.field_data.parse::<i32>().unwrap_or_default()),
                            FieldType::I64 => DecodedData::I64(field.field_data.parse::<i64>().unwrap_or_default()),
                            FieldType::OptionalI16 => DecodedData::OptionalI16(field.field_data.parse::<i16>().unwrap_or_default()),
                            FieldType::OptionalI32 => DecodedData::OptionalI32(field.field_data.parse::<i32>().unwrap_or_default()),
                            FieldType::OptionalI64 => DecodedData::OptionalI64(field.field_data.parse::<i64>().unwrap_or_default()),
                            FieldType::ColourRGB => DecodedData::ColourRGB(field.field_data.to_string()),
                            FieldType::StringU8 => DecodedData::StringU8(if field.field_data == "Frodo Best Waifu" { String::new() } else { field.field_data.to_string() }),
                            FieldType::StringU16 => DecodedData::StringU16(if field.field_data == "Frodo Best Waifu" { String::new() } else { field.field_data.to_string() }),
                            FieldType::OptionalStringU8 => DecodedData::OptionalStringU8(if field.field_data == "Frodo Best Waifu" { String::new() } else { field.field_data.to_string() }),
                            FieldType::OptionalStringU16 => DecodedData::OptionalStringU16(if field.field_data == "Frodo Best Waifu" { String::new() } else { field.field_data.to_string() }),

                            // This type is not used in the raw tables so, if we find it, we skip it.
                            FieldType::SequenceU16(_) | FieldType::SequenceU32(_) => continue,
                        });
                        break;
                    }
                }

                // If the field doesn't exist, we create it empty.
                if !exists {
                    entry.push(match field_def.field_type() {
                        FieldType::Boolean => DecodedData::Boolean(false),
                        FieldType::F32 => DecodedData::F32(0.0),
                        FieldType::F64 => DecodedData::F64(0.0),
                        FieldType::I16 => DecodedData::I16(0),
                        FieldType::I32 => DecodedData::I32(0),
                        FieldType::I64 => DecodedData::I64(0),
                        FieldType::OptionalI16 => DecodedData::OptionalI16(0),
                        FieldType::OptionalI32 => DecodedData::OptionalI32(0),
                        FieldType::OptionalI64 => DecodedData::OptionalI64(0),
                        FieldType::ColourRGB => DecodedData::ColourRGB(String::new()),
                        FieldType::StringU8 => DecodedData::StringU8(String::new()),
                        FieldType::StringU16 => DecodedData::StringU16(String::new()),
                        FieldType::OptionalStringU8 => DecodedData::OptionalStringU8(String::new()),
                        FieldType::OptionalStringU16 => DecodedData::OptionalStringU16(String::new()),

                        // This type is not used in the raw tables so, if we find it, we skip it.
                        FieldType::SequenceU16(_) | FieldType::SequenceU32(_) => unimplemented!("Does this ever happen?"),
                    });
                }
            }
            entries.push(entry);
        }

        table.set_data(&entries)?;
        Ok(table)
    }
}
