//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! In-memory table implementation with import/export capabilities.
//!
//! This module provides [`TableInMemory`], the primary concrete implementation of the
//! [`Table`] trait. It stores all table data in memory and supports multiple serialization
//! formats for data exchange.
//!
//! # TableInMemory
//!
//! [`TableInMemory`] is the standard table implementation used throughout RPFM. It holds:
//! - Complete table data as `Vec<Vec<DecodedData>>`
//! - Schema definition and patches
//! - Metadata (table name, altered flag)
//!
//! ## Creation and Loading
//!
//! Tables can be created in several ways:
//!
//! ```rust
//! # use rpfm_lib::files::table::local::TableInMemory;
//! # use rpfm_lib::schema::Definition;
//! # fn example(definition: &Definition) {
//! // Create empty table from definition
//! let table = TableInMemory::new(definition, None, "units_tables");
//!
//! // Decode from binary data (most common)
//! // let table = TableInMemory::decode(&mut data, definition, &patches, Some(entry_count), false, "units_tables")?;
//! # }
//! ```
//!
//! # Supported Formats
//!
//! ## Binary Encoding
//! - **decode/encode**: Native Total War binary format
//! - Used for reading/writing binary files in PackFiles
//! - Compact, optimized for game performance
//!
//! ## TSV (Tab-Separated Values)
//! - **tsv_import/tsv_export**: Human-readable text format
//! - First line: metadata (`#table_name;version;path`)
//! - Second line: column names
//! - Remaining lines: data rows
//! - Handles special characters via escape sequences
//! - Key columns can be exported first for readability
//!
//! ## SQLite Database (optional, feature-gated)
//! - **db_to_sql/sql_to_db**: Store tables in SQLite for complex queries
//! - Each table version gets its own SQL table (`tablename_v123`)
//! - Tracks pack name, file name, vanilla status
//! - Useful for cross-table analysis and searching
//!
//! # Schema Migration
//!
//! The [`set_definition`](Table::set_definition) method enables **schema version migration**:
//! - Columns are mapped by name (not position)
//! - New columns get default values
//! - Removed columns are dropped
//! - Type changes trigger automatic conversion
//! - Data integrity is preserved where possible
//!
//! This allows tables from older game versions to be updated to newer schemas.
//!
//! # Data Integrity
//!
//! The `altered` flag tracks if data was modified during decoding:
//! - Set to `true` if invalid numeric values were clamped
//! - Set to `true` if type conversions occurred
//! - Used to warn users about potential data corruption
//!
//! # Implementation Details
//!
//! - Uses `getset` for accessor generation
//! - Implements `Clone`, `Debug`, `PartialEq` for testability
//! - Serializable via serde for IPC and caching
//! - Thread-safe through `Table` trait's `Send + Sync` requirement

use base64::{Engine, engine::general_purpose::STANDARD};
use csv::{StringRecordsIter, Writer};
use getset::*;
#[cfg(feature = "integration_sqlite")]use r2d2::Pool;
#[cfg(feature = "integration_sqlite")]use r2d2_sqlite::SqliteConnectionManager;
#[cfg(feature = "integration_sqlite")]use rusqlite::params_from_iter;
use serde_derive::{Serialize, Deserialize};

use std::borrow::Cow;
use std::collections::HashMap;
use std::fs::File;

use crate::binary::{ReadBytes, WriteBytes};
use crate::error::{Result, RLibError};
use crate::files::table::DecodedData;
//#[cfg(feature = "integration_log")] use crate::integrations::log::{info, warn};
use crate::schema::{Definition, DefinitionPatch, FieldType};
use crate::utils::parse_str_as_bool;

use super::{Table, decode_table, encode_table};

//---------------------------------------------------------------------------//
//                              Enum & Structs
//---------------------------------------------------------------------------//

/// In-memory representation of a decoded table with full data and schema.
///
/// This is the primary table implementation in RPFM, storing all table rows in memory
/// along with their schema definition. Tables are typically accessed through the
/// [`Table`] trait interface rather than directly.
///
/// # Fields
///
/// - **table_name**: Identifies the table type (e.g., "units_tables", "buildings_tables")
/// - **definition**: Complete schema definition including column types and constraints
/// - **definition_patch**: Runtime modifications to the base schema for this specific table
/// - **table_data**: All table rows as a `Vec<Vec<DecodedData>>` (outer vector is rows, inner is columns)
/// - **altered**: Flag indicating if data was modified during decoding (e.g., invalid values corrected)
///
/// # Accessors
///
/// The struct uses the `getset` macro for automatic accessor generation:
/// - `table_name()` / `set_table_name()`: Public getters/setters via getset
/// - Schema and data: Accessed through [`Table`] trait methods for type safety
///
/// # Thread Safety
///
/// This struct implements `Send + Sync` (via the `Table` trait requirement), allowing
/// safe concurrent read access and message passing between threads.
#[derive(Clone, Debug, PartialEq, Getters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", set = "pub")]
pub struct TableInMemory {

    /// Table type identifier (e.g., "units_tables").
    table_name: String,

    /// Schema definition for this table.
    #[getset(skip)]
    definition: Definition,

    /// Runtime schema modifications specific to this table instance.
    #[getset(skip)]
    definition_patch: DefinitionPatch,

    /// All table rows (outer vector is rows, inner is columns)
    #[getset(skip)]
    table_data: Vec<Vec<DecodedData>>,

    /// Flag indicating data was altered during decoding (e.g., invalid values corrected).
    altered: bool,
}

//----------------------------------------------------------------//
// Implementations for `Table`.
//----------------------------------------------------------------//

impl TableInMemory {

    /// Creates a new empty table from a schema definition.
    ///
    /// Initializes a table with no rows but with a complete schema definition.
    /// This is typically used when creating new tables from scratch or before
    /// importing data from external sources.
    ///
    /// # Parameters
    ///
    /// - `definition`: Schema defining column structure, types, and constraints
    /// - `definition_patch`: Optional runtime modifications to the base schema
    /// - `table_name`: Table type identifier (e.g., "units_tables")
    ///
    /// # Examples
    ///
    /// ```ignore
    /// # use rpfm_lib::files::table::{local::TableInMemory, Table};
    /// # use rpfm_lib::schema::Definition;
    /// # fn example(definition: &Definition) {
    /// // Create empty table for manual data entry
    /// let mut table = TableInMemory::new(definition, None, "units_tables");
    ///
    /// // Add rows using the Table trait methods
    /// let new_row = table.new_row();
    /// table.data_mut().push(new_row);
    /// # }
    /// ```
    pub fn new(definition: &Definition, definition_patch: Option<&DefinitionPatch>, table_name: &str) -> Self {
        let table_data = vec![];
        let definition_patch = if let Some(patch) = definition_patch { patch.clone() } else { HashMap::new() };

        Self {
            definition: definition.clone(),
            definition_patch,
            table_name: table_name.to_owned(),
            table_data,
            altered: false,
        }
    }

    /// Decodes a table from binary data using the provided schema.
    ///
    /// This is the primary method for loading tables from PackFiles. It reads the binary
    /// format used by Total War games and converts it into an in-memory representation.
    ///
    /// # Parameters
    ///
    /// - `data`: Binary data reader positioned at the table start
    /// - `definition`: Schema definition for interpreting the binary data
    /// - `definition_patch`: Runtime schema modifications
    /// - `entry_count`: Optional row count (if `None`, reads from data stream)
    /// - `return_incomplete`: If `true`, returns partial data on decode errors instead of failing
    /// - `table_name`: Table type identifier
    ///
    /// # Behavior
    ///
    /// - Reads entry count from stream if not provided
    /// - Decodes each row according to schema field definitions
    /// - Sets `altered` flag if invalid data is corrected during decoding
    /// - Can return incomplete tables for error recovery if requested
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Binary data is corrupted or truncated
    /// - Data types don't match schema expectations
    /// - Field decoding fails (unless `return_incomplete` is true)
    ///
    /// # Examples
    ///
    /// ```ignore
    /// # use rpfm_lib::files::table::local::TableInMemory;
    /// # use rpfm_lib::schema::Definition;
    /// # use rpfm_lib::binary::ReadBytes;
    /// # use std::collections::HashMap;
    /// # fn example<R: ReadBytes>(data: &mut R, definition: &Definition) -> anyhow::Result<()> {
    /// // Decode table from binary PackFile data
    /// let table = TableInMemory::decode(
    ///     data,
    ///     definition,
    ///     &HashMap::new(),  // No patches
    ///     Some(100),        // 100 entries
    ///     false,            // Fail on errors
    ///     "units_tables"
    /// )?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn decode<R: ReadBytes>(
        data: &mut R,
        definition: &Definition,
        definition_patch: &DefinitionPatch,
        entry_count: Option<u32>,
        return_incomplete: bool,
        table_name: &str,
    ) -> Result<Self> {

        let mut altered = false;
        let table_data = decode_table(data, definition, entry_count, return_incomplete, &mut altered)?;
        let table = Self {
            definition: definition.clone(),
            definition_patch: definition_patch.clone(),
            table_name: table_name.to_owned(),
            table_data,
            altered
        };

        Ok(table)
    }

    /// Encodes the table to binary format for writing to PackFiles.
    ///
    /// Converts the in-memory table representation back to the binary format used
    /// by Total War games. This is the inverse of [`decode`](Self::decode).
    ///
    /// # Parameters
    ///
    /// - `data`: Binary writer to receive the encoded table
    ///
    /// # Format
    ///
    /// The binary output includes:
    /// - Entry count (u32)
    /// - Row data encoded according to field types
    /// - Applied schema patches are used during encoding
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Writing to the output stream fails
    /// - Data contains values that cannot be encoded in the target type
    ///
    /// # Examples
    ///
    /// ```ignore
    /// # use rpfm_lib::files::table::local::TableInMemory;
    /// # use rpfm_lib::binary::WriteBytes;
    /// # fn example<W: WriteBytes>(table: &TableInMemory, output: &mut W) -> anyhow::Result<()> {
    /// // Encode table for saving to PackFile
    /// table.encode(output)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn encode<W: WriteBytes>(&self, data: &mut W) -> Result<()> {
        encode_table(&self.data(), data, self.definition(), &Some(self.patches()))
    }

    //----------------------------------------------------------------//
    // TSV Functions for tables.
    //----------------------------------------------------------------//
    // TODO: Make tsv trait.

    /// Imports a table from TSV (Tab-Separated Values) format.
    ///
    /// Parses TSV data and creates a new table with the imported rows. The TSV format
    /// is human-readable and editable, making it useful for bulk edits and external
    /// tools integration.
    ///
    /// # TSV Format
    ///
    /// Expected format:
    /// 1. **Header row**: Column names (must match schema field names)
    /// 2. **Metadata row**: `#table_name;version;path` (optional, can be skipped)
    /// 3. **Data rows**: Tab-separated values, one row per line
    ///
    /// # Parameters
    ///
    /// - `records`: CSV record iterator from the TSV file
    /// - `definition`: Schema definition for validation
    /// - `field_order`: Maps column indices to field names from the header
    /// - `table_name`: Table type identifier
    /// - `schema_patches`: Optional schema modifications
    ///
    /// # Column Mapping
    ///
    /// Columns are matched by name (not position), allowing:
    /// - Reordered columns in TSV files
    /// - Missing columns (filled with defaults)
    /// - Extra columns in TSV (ignored)
    ///
    /// # Type Parsing
    ///
    /// - **Booleans**: "true"/"false", "1"/"0", "yes"/"no" (case-insensitive)
    /// - **Numbers**: Standard decimal format, scientific notation for floats
    /// - **Colors**: Hexadecimal strings (e.g., "FF0000")
    /// - **Sequences**: Base64-encoded binary data
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - TSV format is invalid (wrong delimiter, malformed rows)
    /// - Data cannot be parsed to the expected type
    /// - Required columns are missing
    ///
    /// Error includes row and column numbers for debugging: `ImportTSVIncorrectRow(row, col)`
    pub(crate) fn tsv_import(records: StringRecordsIter<File>, definition: &Definition, field_order: &HashMap<u32, String>, table_name: &str, schema_patches: Option<&DefinitionPatch>) -> Result<Self> {
        let mut table = Self::new(definition, schema_patches, table_name);
        let mut entries = vec![];

        let fields_processed = definition.fields_processed();

        for (row, record) in records.enumerate() {
            match record {
                Ok(record) => {
                    let mut entry = table.new_row();
                    for (column, field) in record.iter().enumerate() {

                        // Get the column name from the header, and try to map it to a column in the table's.
                        if let Some(column_name) = field_order.get(&(column as u32)) {
                            if let Some(column_number) = fields_processed.iter().position(|x| x.name() == column_name) {

                                entry[column_number] = match fields_processed[column_number].field_type() {
                                    FieldType::Boolean => parse_str_as_bool(field).map(DecodedData::Boolean).map_err(|_| RLibError::ImportTSVIncorrectRow(row, column))?,
                                    FieldType::F32 => DecodedData::F32(field.parse::<f32>().map_err(|_| RLibError::ImportTSVIncorrectRow(row, column))?),
                                    FieldType::F64 => DecodedData::F64(field.parse::<f64>().map_err(|_| RLibError::ImportTSVIncorrectRow(row, column))?),
                                    FieldType::I16 => DecodedData::I16(field.parse::<i16>().map_err(|_| RLibError::ImportTSVIncorrectRow(row, column))?),
                                    FieldType::I32 => DecodedData::I32(field.parse::<i32>().map_err(|_| RLibError::ImportTSVIncorrectRow(row, column))?),
                                    FieldType::I64 => DecodedData::I64(field.parse::<i64>().map_err(|_| RLibError::ImportTSVIncorrectRow(row, column))?),
                                    FieldType::OptionalI16 => DecodedData::OptionalI16(field.parse::<i16>().map_err(|_| RLibError::ImportTSVIncorrectRow(row, column))?),
                                    FieldType::OptionalI32 => DecodedData::OptionalI32(field.parse::<i32>().map_err(|_| RLibError::ImportTSVIncorrectRow(row, column))?),
                                    FieldType::OptionalI64 => DecodedData::OptionalI64(field.parse::<i64>().map_err(|_| RLibError::ImportTSVIncorrectRow(row, column))?),
                                    FieldType::ColourRGB => DecodedData::ColourRGB(if u32::from_str_radix(field, 16).is_ok() {
                                        field.to_owned()
                                    } else {
                                        Err(RLibError::ImportTSVIncorrectRow(row, column))?
                                    }),
                                    FieldType::StringU8 => DecodedData::StringU8(field.to_owned()),
                                    FieldType::StringU16 => DecodedData::StringU16(field.to_owned()),
                                    FieldType::OptionalStringU8 => DecodedData::OptionalStringU8(field.to_owned()),
                                    FieldType::OptionalStringU16 => DecodedData::OptionalStringU16(field.to_owned()),

                                    // For now fail on Sequences. These are a bit special and I don't know if the're even possible in TSV.
                                    FieldType::SequenceU16(_) => DecodedData::SequenceU16(STANDARD.decode(field).map_err(|_| RLibError::ImportTSVIncorrectRow(row, column))?),
                                    FieldType::SequenceU32(_) => DecodedData::SequenceU32(STANDARD.decode(field).map_err(|_| RLibError::ImportTSVIncorrectRow(row, column))?),
                                }
                            }
                        }
                    }
                    entries.push(entry);
                }
                Err(_) => return Err(RLibError::ImportTSVIncorrectRow(row, 0)),
            }
        }

        // If we reached this point without errors, we replace the old data with the new one and return success.
        table.set_data(&entries)?;
        Ok(table)
    }

    /// Exports the table to TSV (Tab-Separated Values) format.
    ///
    /// Writes table data to a TSV file for human editing, version control, or
    /// external tool processing. The output format is compatible with
    /// [`tsv_import`](Self::tsv_import).
    ///
    /// # Parameters
    ///
    /// - `writer`: CSV writer configured for TSV output
    /// - `table_path`: Original file path (stored in metadata row)
    /// - `keys_first`: If `true`, sorts key columns to the left for easier reading
    ///
    /// # Output Format
    ///
    /// ```text
    /// column1    column2    column3
    /// #units_tables;5;db/units_tables/data.bin
    /// value1     value2     value3
    /// value4     value5     value6
    /// ```
    ///
    /// - **Line 1**: Column names from schema
    /// - **Line 2**: Metadata (`#table_name;version;path`) with padding cells
    /// - **Lines 3+**: Data rows
    ///
    /// # Data Formatting
    ///
    /// - **Floats**: Fixed 4 decimal places (e.g., "3.1416")
    /// - **Booleans**: "true" or "false"
    /// - **Sequences**: Base64-encoded binary data
    /// - **Special chars**: Newlines/tabs are escaped as `\\n`/`\\t`
    ///
    /// # Column Ordering
    ///
    /// If `keys_first` is `true`:
    /// - Key columns appear first (left-most)
    /// - Non-key columns follow
    /// - Makes primary keys visible without horizontal scrolling
    ///
    /// # Errors
    ///
    /// Returns an error if writing to the file fails.
    pub(crate) fn tsv_export(&self, writer: &mut Writer<File>, table_path: &str, keys_first: bool) -> Result<()> {

        let fields_processed = self.definition().fields_processed();
        let fields_sorted = self.definition().fields_processed_sorted(keys_first);
        let fields_sorted_properly = fields_sorted.iter()
            .map(|field_sorted| (fields_processed.iter().position(|field| field == field_sorted).unwrap(), field_sorted))
            .collect::<Vec<(_,_)>>();

        // We serialize the info of the table (name and version) in the first line, and the column names in the second one.
        let metadata = (format!("#{};{};{}", self.table_name(), self.definition().version(), table_path), vec![String::new(); fields_sorted_properly.len() - 1]);
        writer.serialize(fields_sorted_properly.iter().map(|(_, field)| field.name()).collect::<Vec<&str>>())?;
        writer.serialize(metadata)?;

        // Then we serialize each entry in the DB Table.
        let entries = self.data();
        for entry in &*entries {
            let sorted_entry = fields_sorted_properly.iter()
                .map(|(index, _)| entry[*index].data_to_string())
                .collect::<Vec<Cow<str>>>();
            writer.serialize(sorted_entry)?;
        }

        writer.flush().map_err(From::from)
    }

    //----------------------------------------------------------------//
    // SQL functions for tables.
    //----------------------------------------------------------------//

    /// Inserts the table into a SQLite database for querying and analysis.
    ///
    /// Creates or updates a SQL table with this table's data, enabling complex queries,
    /// cross-table joins, and full-text search. Each table version gets its own SQL table
    /// named `tablename_v{version}`.
    ///
    /// # Parameters
    ///
    /// - `pool`: SQLite connection pool
    /// - `pack_name`: Name of the PackFile containing this table
    /// - `file_name`: Path to this table within the PackFile
    /// - `is_vanilla_pack`: `true` if from official game data, `false` for mods
    ///
    /// # SQL Schema
    ///
    /// The created SQL table includes:
    /// - `pack_name` (TEXT): Source PackFile identifier
    /// - `file_name` (TEXT): Table path within PackFile
    /// - `is_vanilla` (INTEGER): 1 for vanilla, 0 for mods
    /// - Column for each schema field (types mapped from `FieldType`)
    ///
    /// # Behavior
    ///
    /// - Creates the SQL table if it doesn't exist (silently ignores if exists)
    /// - Uses `INSERT OR REPLACE` to update existing rows
    /// - Sequences are stored as BLOB for efficient binary storage
    ///
    /// # Use Cases
    ///
    /// - Finding all units with a specific ability across multiple mods
    /// - Analyzing stat distributions (e.g., average unit cost)
    /// - Detecting conflicts between mods
    /// - Building searchable databases of game content
    ///
    /// # Feature Gate
    ///
    /// Only available with the `integration_sqlite` feature enabled.
    #[cfg(feature = "integration_sqlite")]
    pub fn db_to_sql(&self, pool: &Pool<SqliteConnectionManager>, pack_name: &str, file_name: &str, is_vanilla_pack: bool) -> Result<()> {

        // Try to create the table, in case it doesn't exist yet. Ignore a failure here, as it'll mean the table already exists.
        let params: Vec<String> = vec![];
        let create_table = self.definition().map_to_sql_create_table_string(self.table_name());
        match pool.get()?.execute(&create_table, params_from_iter(params)) {
            Ok(_) => {
                //#[cfg(feature = "integration_log")] {
                //    info!("Table {} created succesfully.", self.table_name());
                //}
            },

            Err(error) => {
                //#[cfg(feature = "integration_log")] {
                //    warn!("Table {} failed to be created: {error}", self.table_name());
                //}
            },
        }

        self.insert_all_to_sql(pool, pack_name, file_name, is_vanilla_pack)?;
        Ok(())
    }

    /// Loads table data from a SQLite database.
    ///
    /// Retrieves previously stored table data from SQL and replaces the current
    /// table's rows. This is the inverse of [`db_to_sql`](Self::db_to_sql).
    ///
    /// # Parameters
    ///
    /// - `pool`: SQLite connection pool
    /// - `pack_name`: Name of the source PackFile
    /// - `file_name`: Path to the table within the PackFile
    ///
    /// # Behavior
    ///
    /// - Queries the SQL table for rows matching `pack_name` and `file_name`
    /// - Maintains row order via `ROWID`
    /// - Converts SQL types back to `DecodedData` variants
    /// - Replaces all current table data
    ///
    /// # Feature Gate
    ///
    /// Only available with the `integration_sqlite` feature enabled.
    #[cfg(feature = "integration_sqlite")]
    pub fn sql_to_db(&mut self, pool: &Pool<SqliteConnectionManager>, pack_name: &str, file_name: &str) -> Result<()> {
        self.table_data = self.select_all_from_sql(pool, pack_name, file_name)?;
        Ok(())
    }

    /// Inserts all table rows into a SQLite database.
    ///
    /// Converts each row's `DecodedData` fields to SQL-compatible values and performs
    /// a bulk `INSERT OR REPLACE` operation. Sequence fields are passed as binary parameters.
    ///
    /// # Arguments
    ///
    /// * `pool` - SQLite connection pool.
    /// * `pack_name` - Name of the pack containing this table.
    /// * `file_name` - Path of the table file within the pack.
    /// * `is_vanilla_pack` - Whether the pack is from the base game (stored as 1/0 flag).
    #[cfg(feature = "integration_sqlite")]
    fn insert_all_to_sql(&self, pool: &Pool<SqliteConnectionManager>, pack_name: &str, file_name: &str, is_vanilla_pack: bool) -> Result<()> {
        let mut params = vec![];
        let values = self.table_data.iter().map(|row| {
            format!("(\"{}\", \"{}\", {}, {})",
                pack_name,
                file_name,
                if is_vanilla_pack { "1" } else { "0" },
                row.iter().map(|field| {
                match field {
                    DecodedData::Boolean(data) => if *data { "1".to_owned() } else { "0".to_owned() },
                    DecodedData::F32(data) => format!("{data:.4}"),
                    DecodedData::F64(data) => format!("{data:.4}"),
                    DecodedData::I16(data) => format!("\"{data}\""),
                    DecodedData::I32(data) => format!("\"{data}\""),
                    DecodedData::I64(data) => format!("\"{data}\""),
                    DecodedData::ColourRGB(data) => format!("\"{}\"", data.replace('\"', "\"\"")),
                    DecodedData::StringU8(data) => format!("\"{}\"", data.replace('\"', "\"\"")),
                    DecodedData::StringU16(data) => format!("\"{}\"", data.replace('\"', "\"\"")),
                    DecodedData::OptionalI16(data) => format!("\"{data}\""),
                    DecodedData::OptionalI32(data) => format!("\"{data}\""),
                    DecodedData::OptionalI64(data) => format!("\"{data}\""),
                    DecodedData::OptionalStringU8(data) => format!("\"{}\"", data.replace('\"', "\"\"")),
                    DecodedData::OptionalStringU16(data) => format!("\"{}\"", data.replace('\"', "\"\"")),
                    DecodedData::SequenceU16(data) => {
                        params.push(data.to_vec());
                        "?".to_owned()
                    },
                    DecodedData::SequenceU32(data) => {
                        params.push(data.to_vec());
                        "?".to_owned()
                    },
                }
            }).collect::<Vec<_>>().join(","))
        }).collect::<Vec<_>>().join(",");

        // If there are no values, don't bother with the query.
        if values.is_empty() {
            return Ok(());
        }

        let query = format!("INSERT OR REPLACE INTO \"{}_v{}\" {} VALUES {}",
            self.table_name().replace('\"', "'"),
            self.definition().version(),
            self.definition().map_to_sql_insert_into_string(),
            values
        );

        pool.get()?.execute(&query, params_from_iter(params.iter()))
            .map(|_| ())
            .map_err(From::from)
    }

    /// Retrieves all table rows from a SQLite database.
    ///
    /// Queries the database for rows matching the pack and file name, converting
    /// SQL values back to `DecodedData` fields based on the table definition.
    ///
    /// # Arguments
    ///
    /// * `pool` - SQLite connection pool.
    /// * `pack_name` - Name of the pack containing this table.
    /// * `file_name` - Path of the table file within the pack.
    ///
    /// # Returns
    ///
    /// A vector of rows, each containing decoded field data in column order.
    #[cfg(feature = "integration_sqlite")]
    fn select_all_from_sql(&self, pool: &Pool<SqliteConnectionManager>, pack_name: &str, file_name: &str) -> Result<Vec<Vec<DecodedData>>> {
        let definition = self.definition();
        let fields_processed = definition.fields_processed();

        let field_names = fields_processed.iter().map(|field| field.name()).collect::<Vec<&str>>().join(",");
        let query = format!("SELECT {} FROM \"{}_v{}\" WHERE pack_name = \"{}\" AND file_name = \"{}\" order by ROWID",
            field_names,
            self.table_name().replace('\"', "'"),
            definition.version(),
            pack_name,
            file_name
        );

        let conn = pool.get()?;
        let mut stmt = conn.prepare(&query)?;
        let rows = stmt.query_map([], |row| {
            let mut data = Vec::with_capacity(fields_processed.len());
            for (i, field) in fields_processed.iter().enumerate() {
                data.push(match field.field_type() {
                    FieldType::Boolean => DecodedData::Boolean(row.get(i)?),
                    FieldType::F32 => DecodedData::F32(row.get(i)?),
                    FieldType::F64 => DecodedData::F64(row.get(i)?),
                    FieldType::I16 => DecodedData::I16(row.get(i)?),
                    FieldType::I32 => DecodedData::I32(row.get(i)?),
                    FieldType::I64 => DecodedData::I64(row.get(i)?),
                    FieldType::ColourRGB => DecodedData::ColourRGB(row.get(i)?),
                    FieldType::StringU8 => DecodedData::StringU8(row.get(i)?),
                    FieldType::StringU16 => DecodedData::StringU16(row.get(i)?),
                    FieldType::OptionalI16 => DecodedData::OptionalI16(row.get(i)?),
                    FieldType::OptionalI32 => DecodedData::OptionalI32(row.get(i)?),
                    FieldType::OptionalI64 => DecodedData::OptionalI64(row.get(i)?),
                    FieldType::OptionalStringU8 => DecodedData::OptionalStringU8(row.get(i)?),
                    FieldType::OptionalStringU16 => DecodedData::OptionalStringU16(row.get(i)?),
                    FieldType::SequenceU16(_) => DecodedData::SequenceU16(row.get(i)?),
                    FieldType::SequenceU32(_) => DecodedData::SequenceU32(row.get(i)?),
                });
            }

            Ok(data)
        })?;

        let mut data = vec![];
        for row in rows {
            data.push(row?);
        }

        Ok(data)
    }

    /// Counts the number of rows in a table matching a unique identifier.
    ///
    /// # Arguments
    ///
    /// * `pool` - SQLite connection pool.
    /// * `table_name` - Name of the table type (e.g., "units_tables").
    /// * `table_version` - Schema version of the table.
    /// * `table_unique_id` - Unique identifier to filter rows.
    ///
    /// # Returns
    ///
    /// The count of matching rows in the database.
    #[cfg(feature = "integration_sqlite")]
    pub fn count_table(
        pool: &Pool<SqliteConnectionManager>,
        table_name: &str,
        table_version: i32,
        table_unique_id: u64,
    ) -> Result<u64> {
        let query = format!("SELECT COUNT(*) FROM \"{}_v{}\" WHERE table_unique_id = {}",
            table_name.replace('\"', "'"),
            table_version,
            table_unique_id
        );

        let conn = pool.get()?;
        let mut stmt = conn.prepare(&query)?;
        let mut rows = stmt.query([])?;
        let mut count = 0;
        if let Some(row) = rows.next()? {
            count = row.get(0)?;
        }

        Ok(count)
    }
}

impl Table for TableInMemory {
    fn name(&self) -> &str {
        &self.table_name
    }

    fn definition(&self) -> &Definition {
        &self.definition
    }

    fn patches(&self) -> &DefinitionPatch {
        &self.definition_patch
    }

    fn data(&'_ self) -> Cow<'_, [Vec<DecodedData>]> {
        Cow::from(&self.table_data)
    }

    fn data_mut(&mut self) -> &mut Vec<Vec<DecodedData>> {
        &mut self.table_data
    }

    fn set_name(&mut self, val: String) {
        self.table_name = val;
    }

    fn set_definition(&mut self, new_definition: &Definition) {

        // It's simple: we compare both schemas, and get the original and final positions of each column.
        // If a column is new, his original position is -1. If has been removed, his final position is -1.
        let mut positions: Vec<(i32, i32)> = vec![];
        let new_fields_processed = new_definition.fields_processed();
        let old_fields_processed = self.definition.fields_processed();

        for (new_pos, new_field) in new_fields_processed.iter().enumerate() {
            if let Some(old_pos) = old_fields_processed.iter().position(|x| x.name() == new_field.name()) {
                positions.push((old_pos as i32, new_pos as i32))
            } else { positions.push((-1, new_pos as i32)); }
        }

        // Then, for each field in the old definition, check if exists in the new one.
        for (old_pos, old_field) in old_fields_processed.iter().enumerate() {
            if !new_fields_processed.iter().any(|x| x.name() == old_field.name()) { positions.push((old_pos as i32, -1)); }
        }

        // We sort the columns by their destination.
        positions.sort_by_key(|x| x.1);

        // Then, we create the new data using the old one and the column changes.
        let mut new_entries: Vec<Vec<DecodedData>> = Vec::with_capacity(self.table_data.len());
        for row in self.table_data.iter() {
            let mut entry = vec![];
            for (old_pos, new_pos) in &positions {

                // If the new position is -1, it means the column got removed. We skip it.
                if *new_pos == -1 { continue; }

                // If the old position is -1, it means we got a new column. We need to get his type and create a `Default` field with it.
                else if *old_pos == -1 {
                    let field_type = new_fields_processed[*new_pos as usize].field_type();
                    let default_value = new_fields_processed[*new_pos as usize].default_value(Some(&self.definition_patch));
                    entry.push(DecodedData::new_from_type_and_value(field_type, &default_value));
                }

                // Otherwise, we got a moved column. Check here if it needs type conversion.
                else if new_fields_processed[*new_pos as usize].field_type() != old_fields_processed[*old_pos as usize].field_type() {
                    let converted_data = match row[*old_pos as usize].convert_between_types(new_fields_processed[*new_pos as usize].field_type()) {
                        Ok(data) => data,
                        Err(_) => {
                            let field_type = new_fields_processed[*new_pos as usize].field_type();
                            let default_value = new_fields_processed[*new_pos as usize].default_value(Some(&self.definition_patch));
                            DecodedData::new_from_type_and_value(field_type, &default_value)
                        }
                    };
                    entry.push(converted_data);
                }

                // If we reach this, we just got a moved column without any extra change.
                else {
                    entry.push(row[*old_pos as usize].clone());
                }
            }
            new_entries.push(entry);
        }

        self.table_data = new_entries;

        // Then, we finally replace our definition and our data.
        self.definition = new_definition.clone();
    }

    fn set_data(&mut self, data: &[Vec<DecodedData>]) -> Result<()> {
        let fields_processed = self.definition.fields_processed();
        for row in data {

            // First, we need to make sure all rows we have are exactly what we expect.
            if row.len() != fields_processed.len() {
                return Err(RLibError::TableRowWrongFieldCount(fields_processed.len(), row.len()))
            }

            for (index, cell) in row.iter().enumerate() {

                // Next, we need to ensure each file is of the type we expected.
                let field = fields_processed.get(index).unwrap();
                if !cell.is_field_type_correct(field.field_type()) {
                    return Err(RLibError::EncodingTableWrongFieldType(FieldType::from(cell).to_string(), field.field_type().to_string()))
                }
            }
        }

        // If we passed all the checks, replace the data.
        self.table_data = data.to_vec();
        Ok(())
    }

    fn column_position_by_name(&self, column_name: &str) -> Option<usize> {
        self.definition().column_position_by_name(column_name)
    }

    fn is_empty(&self) -> bool {
        self.data().is_empty()
    }

    fn len(&self) -> usize {
        self.data().len()
    }

    fn rows_containing_data(&self, column_name: &str, data: &str) -> Option<(usize, Vec<usize>)> {
        let mut row_indexes = vec![];

        let column_index = self.column_position_by_name(column_name)?;
        for (row_index, row) in self.data().iter().enumerate() {
            if let Some(cell_data) = row.get(column_index) {
                if cell_data.data_to_string() == data {
                    row_indexes.push(row_index);
                }
            }
        }

        if row_indexes.is_empty() {
            None
        } else {
            Some((column_index, row_indexes))
        }
    }
}
