//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Localisation table files for Total War games.
//!
//! Loc files store key-value pairs for text localisation, enabling translation of in-game
//! strings. Each entry consists of a unique key, the localised text, and a boolean flag
//! (purpose unknown, commonly called "tooltip").
//!
//! # Overview
//!
//! Unlike DB tables which require schema definitions, Loc files have a fixed structure:
//! - **Key**: Unique identifier for the text entry (UTF-16 string)
//! - **Text**: The localised string content (UTF-16 string)
//! - **Tooltip**: Boolean flag of unknown purpose
//!
//! Loc files are used in all Total War games since Empire. In games prior to Troy, when
//! using a non-English language, only the main `localisation.loc` file is loaded -
//! individual loc files are ignored.
//!
//! # Binary Structure
//!
//! ## Header (14 bytes)
//!
//! | Bytes | Type            | Data                                           |
//! | ----- | --------------- | ---------------------------------------------- |
//! | 2     | [u16]           | Byte order mark. Always `0xFFFE`.              |
//! | 3     | UTF-8 String    | File type identifier. Always `"LOC"`.          |
//! | 1     | [u8]            | Unknown, always `0`. Possibly padding.         |
//! | 4     | [i32]           | Version. Always `1` in known files.            |
//! | 4     | [u32]           | Number of entries in the table.                |
//!
//! ## Data (per entry)
//!
//! | Bytes | Type            | Data                                           |
//! | ----- | --------------- | ---------------------------------------------- |
//! | 2 + * | Sized StringU16 | Localisation key (u16 length prefix + UTF-16). |
//! | 2 + * | Sized StringU16 | Localised text (u16 length prefix + UTF-16).   |
//! | 1     | [bool]          | Tooltip flag (unknown purpose).                |

use csv::{StringRecordsIter, Writer};
use getset::{Getters, Setters};
use rayon::prelude::*;
use serde_derive::{Serialize, Deserialize};

use std::borrow::Cow;
use std::collections::{BTreeMap, HashMap};
use std::fs::File;

use crate::binary::{ReadBytes, WriteBytes};
use crate::error::{RLibError, Result};
use crate::files::{DecodeableExtraData, Decodeable, EncodeableExtraData, Encodeable, table::{DecodedData, local::TableInMemory, Table}};
use crate::schema::*;
use crate::utils::check_size_mismatch;

/// This represents the value that every Loc file has in their first 2 bytes.
const BYTEORDER_MARK: u16 = 65279; // FF FE

/// This represents the value that every Loc file has in their 2-5 bytes. The sixth byte is always a 0.
const FILE_TYPE: &str = "LOC";

/// Size of the header of a Loc file.
const HEADER_SIZE: usize = 14;

/// This is the name used in TSV-exported Loc files to identify them as Loc files.
pub(crate) const TSV_NAME_LOC: &str = "Loc";
pub(crate) const TSV_NAME_LOC_OLD: &str = "Loc PackedFile";

/// Extension used by Loc files.
pub const EXTENSION: &str = ".loc";

/// Version used by Loc files. We've only seen version 1 so far, so we stick with that one.
const VERSION: i32 = 1;

#[cfg(test)] mod loc_test;

//---------------------------------------------------------------------------//
//                              Enum & Structs
//---------------------------------------------------------------------------//

/// In-memory representation of a decoded Loc (localisation) file.
///
/// Wraps a [`TableInMemory`] with a fixed three-column schema: key, text, and tooltip.
/// Unlike DB tables, Loc files don't require external schema definitions.
///
/// # Structure
///
/// Each row contains:
/// - `key` (StringU16): Unique identifier for the localised text
/// - `text` (StringU16): The localised string content
/// - `tooltip` (Boolean): Flag of unknown purpose
///
/// # Example
///
/// ```ignore
/// use rpfm_lib::files::{Decodeable, loc::Loc};
/// use std::io::Cursor;
///
/// # let loc_data = vec![];
/// let mut reader = Cursor::new(loc_data);
/// let loc = Loc::decode(&mut reader, &None).unwrap();
///
/// // Access entries
/// for row in loc.data().iter() {
///     // row[0] = key, row[1] = text, row[2] = tooltip
/// }
/// ```
#[derive(PartialEq, Clone, Debug, Getters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", set = "pub")]
pub struct Loc {

    /// The underlying table data with key, text, and tooltip columns.
    table: TableInMemory,
}

//---------------------------------------------------------------------------//
//                           Implementation of Loc
//---------------------------------------------------------------------------//

impl Default for Loc {
    fn default() -> Self {
        Self::new()
    }
}

/// Implementation of `Loc`.
impl Loc {

    /// Creates a new empty Loc table.
    ///
    /// Initializes with the standard three-column schema (key, text, tooltip)
    /// but no data rows.
    pub fn new() -> Self {
        let definition = Self::new_definition();

        Self {
            table: TableInMemory::new(&definition, None, TSV_NAME_LOC),
        }
    }

    /// Returns the fixed schema definition for Loc tables.
    ///
    /// The definition contains three fields:
    /// - `key` (StringU16, primary key)
    /// - `text` (StringU16)
    /// - `tooltip` (Boolean)
    pub(crate) fn new_definition() -> Definition {
        let mut definition = Definition::new(VERSION, None);
        let fields = vec![
            Field::new("key".to_owned(), FieldType::StringU16, true, Some("PLACEHOLDER".to_owned()), false, None, None, None, String::new(), 0, 0, BTreeMap::new(), None),
            Field::new("text".to_owned(), FieldType::StringU16, false, Some("PLACEHOLDER".to_owned()), false, None, None, None, String::new(), 0, 0, BTreeMap::new(), None),
            Field::new("tooltip".to_owned(), FieldType::Boolean, false, Some("PLACEHOLDER".to_owned()), false, None, None, None, String::new(), 0, 0, BTreeMap::new(), None),
        ];
        definition.set_fields(fields);
        definition
    }

    /// Returns the schema definition used by this Loc table.
    pub fn definition(&self) -> &Definition {
        self.table.definition()
    }

    /// Returns the table rows as a slice of decoded data.
    pub fn data(&'_ self) -> Cow<'_, [Vec<DecodedData>]> {
        self.table.data()
    }

    /// Returns a mutable reference to the table rows.
    ///
    /// Ensure modifications maintain valid structure (3 columns per row).
    pub fn data_mut(&mut self) -> &mut Vec<Vec<DecodedData>> {
        self.table.data_mut()
    }

    /// Creates a new row with default placeholder values.
    pub fn new_row(&self) -> Vec<DecodedData> {
        self.table().new_row()
    }

    /// Replaces all table data with the provided rows.
    ///
    /// # Errors
    ///
    /// Returns an error if rows don't match the expected 3-column structure.
    pub fn set_data(&mut self, data: &[Vec<DecodedData>]) -> Result<()> {
        self.table.set_data(data)
    }

    /// Returns the column index for a given column name, or `None` if not found.
    ///
    /// Valid column names: `"key"` (0), `"text"` (1), `"tooltip"` (2).
    pub fn column_position_by_name(&self, column_name: &str) -> Option<usize> {
        self.table().column_position_by_name(column_name)
    }

    /// Returns the number of entries in the Loc table.
    pub fn len(&self) -> usize {
        self.table.len()
    }

    /// Returns `true` if the Loc table has no entries.
    pub fn is_empty(&self) -> bool {
        self.table.is_empty()
    }

    /// Replaces the table definition and migrates existing data to match.
    ///
    /// Typically not needed for Loc files since the definition is fixed.
    pub fn set_definition(&mut self, new_definition: &Definition) {
        self.table.set_definition(new_definition);
    }

    /// Reads and validates the Loc file header.
    ///
    /// # Returns
    ///
    /// A tuple of `(version, entry_count)`. Version is always 1 in known files.
    ///
    /// # Errors
    ///
    /// Returns [`RLibError::DecodingLocNotALocTable`] if the header is invalid
    /// (wrong byte order mark, wrong file type, or insufficient data).
    pub fn read_header<R: ReadBytes>(data: &mut R) -> Result<(i32, u32)> {

        // A valid Loc PackedFile has at least 14 bytes. This ensures they exists before anything else.
        if data.len()? < HEADER_SIZE as u64 {
            return Err(RLibError::DecodingLocNotALocTable)
        }

        // More checks to ensure this is a valid Loc file.
        if BYTEORDER_MARK != data.read_u16()? {
            return Err(RLibError::DecodingLocNotALocTable)
        }

        if FILE_TYPE != data.read_string_u8(3)? {
            return Err(RLibError::DecodingLocNotALocTable)
        }

        let _ = data.read_u8()?;
        let version = data.read_i32()?;
        let entry_count = data.read_u32()?;

        Ok((version, entry_count))
    }

    /// Merges multiple Loc tables into a single new table.
    ///
    /// Combines all rows from the source tables. Duplicate keys are preserved
    /// (not deduplicated).
    pub fn merge(sources: &[&Self]) -> Result<Self> {
        let mut new_table = Self::new();
        let sources = sources.par_iter()
            .map(|table| {
                let mut table = table.table().clone();
                table.set_definition(new_table.definition());
                table
            })
            .collect::<Vec<_>>();

        let new_data = sources.par_iter()
            .map(|table| table.data().to_vec())
            .flatten()
            .collect::<Vec<_>>();
        new_table.set_data(&new_data)?;

        Ok(new_table)
    }

    /// Imports a Loc table from TSV (tab-separated values) format.
    ///
    /// # Arguments
    ///
    /// * `records` - CSV reader iterator over TSV records.
    /// * `field_order` - Mapping of column positions to field names.
    pub fn tsv_import(records: StringRecordsIter<File>, field_order: &HashMap<u32, String>) -> Result<Self> {
        let definition = Self::new_definition();
        let table = TableInMemory::tsv_import(records, &definition, field_order, TSV_NAME_LOC, None)?;
        let loc = Loc::from(table);
        Ok(loc)
    }

    /// Exports the Loc table to TSV (tab-separated values) format.
    ///
    /// # Arguments
    ///
    /// * `writer` - CSV writer for the output file.
    /// * `table_path` - Path used in the TSV metadata header.
    pub fn tsv_export(&self, writer: &mut Writer<File>, table_path: &str) -> Result<()> {
        self.table.tsv_export(writer, table_path, true)
    }
}

impl Decodeable for Loc {

    fn decode<R: ReadBytes>(data: &mut R, _extra_data: &Option<DecodeableExtraData>) -> Result<Self> {

        // Version is always 1, so we ignore it.
        let (_version, entry_count) = Self::read_header(data)?;

        let definition = Self::new_definition();
        let table = TableInMemory::decode(data, &definition, &HashMap::new(), Some(entry_count), false, TSV_NAME_LOC)?;

        // If we are not in the last byte, it means we didn't parse the entire file, which means this file is corrupt.
        check_size_mismatch(data.stream_position()? as usize, data.len()? as usize)?;

        Ok(Self {
            table,
        })
    }
}

impl Encodeable for Loc {

    fn encode<W: WriteBytes>(&mut self, buffer: &mut W, _extra_data: &Option<EncodeableExtraData>) -> Result<()> {
        buffer.write_u16(BYTEORDER_MARK)?;
        buffer.write_string_u8(FILE_TYPE)?;
        buffer.write_u8(0)?;
        buffer.write_i32(*self.table.definition().version())?;
        buffer.write_u32(self.table.len() as u32)?;

        self.table.encode(buffer)
    }
}

/// Implementation to create a `Loc` from a `Table` directly.
impl From<TableInMemory> for Loc {
    fn from(mut table: TableInMemory) -> Self {
        table.set_table_name(TSV_NAME_LOC.to_owned());
        Self {
            table,
        }
    }
}
