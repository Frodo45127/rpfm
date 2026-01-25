//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Localisable fields parsing for Assembly Kit integration.
//!
//! This module handles the parsing of Assembly Kit's localisable fields definition file,
//! which identifies which table fields contain translatable text that should be extracted
//! to `.loc` (localisation) files.
//!
//! # Overview
//!
//! Total War games support multiple languages through localisation files (`.loc` files).
//! Rather than storing translated text directly in database tables, certain text fields
//! are marked as "localisable" and their content is stored in separate translation files.
//!
//! The Assembly Kit includes a `TExc_LocalisableFields.xml` file that defines which
//! fields in which tables should be treated as localisable.
//!
//! # File Format
//!
//! The localisable fields file is an XML file with this structure:
//! ```xml
//! <dataroot>
//!   <TExc_LocalisableFields>
//!     <table_name>units_tables</table_name>
//!     <field>onscreen_name</field>
//!   </TExc_LocalisableFields>
//!   <TExc_LocalisableFields>
//!     <table_name>units_tables</table_name>
//!     <field>short_description</field>
//!   </TExc_LocalisableFields>
//! </dataroot>
//! ```
//!
//! # Main Types
//!
//! - [`RawLocalisableFields`]: Root structure containing all localisable field definitions
//! - [`RawLocalisableField`]: Single field marked as localisable
//!
//! # Availability
//!
//! Localisable fields files are only available in Assembly Kit versions 1 and 2:
//! - **Version 0** (Empire/Napoleon): Not available - must be determined through analysis
//! - **Version 1** (Shogun 2): Available as `TExc_LocalisableFields.xml`
//! - **Version 2** (Rome 2+): Available as `TExc_LocalisableFields.xml`

use serde_derive::Deserialize;
use serde_xml_rs::from_reader;

use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use crate::error::{Result, RLibError};

use super::*;

//---------------------------------------------------------------------------//
// Types for parsing the Assembly Kit's TExc_LocalisableFields Files into.
//---------------------------------------------------------------------------//

/// Complete localisable fields definition from Assembly Kit.
///
/// This is the root structure parsed from `TExc_LocalisableFields.xml`. It contains
/// a list of all table fields that should be treated as localisable.
///
/// # Structure
///
/// Each entry in the file maps a table field to its localisable status. Multiple
/// fields from the same table will appear as separate entries.
///
/// # Example
///
/// For a units table with two localisable fields, the file would contain:
/// ```xml
/// <dataroot>
///   <TExc_LocalisableFields>
///     <table_name>units_tables</table_name>
///     <field>onscreen_name</field>
///   </TExc_LocalisableFields>
///   <TExc_LocalisableFields>
///     <table_name>units_tables</table_name>
///     <field>description</field>
///   </TExc_LocalisableFields>
/// </dataroot>
/// ```
#[derive(Clone, Debug, Deserialize)]
#[serde(rename = "dataroot")]
pub struct RawLocalisableFields {

    /// All localisable field definitions.
    #[serde(rename = "TExc_LocalisableFields")]
    pub fields: Vec<RawLocalisableField>,
}

/// Single localisable field definition.
///
/// Identifies one field in one table that contains translatable text.
///
/// # Fields
///
/// * `table_name` - Name of the table (without `_tables` suffix in some versions)
/// * `field` - Name of the field/column that is localisable
#[derive(Clone, Debug, Deserialize)]
#[serde(rename = "datafield")]
pub struct RawLocalisableField {
    /// Table name containing the localisable field.
    pub table_name: String,

    /// Field/column name that is localisable.
    pub field: String,
}

//---------------------------------------------------------------------------//
// Implementations
//---------------------------------------------------------------------------//

/// Implementation of `RawLocalisableFields`.
impl RawLocalisableFields {

    /// Parses the localisable fields definition file from Assembly Kit.
    ///
    /// Reads and parses the `TExc_LocalisableFields.xml` file which lists all table
    /// fields that contain translatable text.
    ///
    /// # Arguments
    ///
    /// * `raw_data_path` - Path to the Assembly Kit data directory
    /// * `version` - Assembly Kit version (1 = Shogun 2, 2 = Rome 2+)
    ///
    /// # Returns
    ///
    /// Returns a [`RawLocalisableFields`] containing all field definitions.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The version is not 1 or 2 (returns [`RLibError::AssemblyKitUnsupportedVersion`])
    /// - The `TExc_LocalisableFields.xml` file cannot be found or opened
    /// - The XML is malformed
    ///
    /// # Version 0 Note
    ///
    /// Empire and Napoleon (Version 0) do not include a localisable fields file.
    /// For these games, localisable fields must be determined through other means
    /// (typically by analyzing actual game data or manual specification).
    pub fn read(raw_data_path: &Path, version: i16) -> Result<Self> {
        match version {
            2 | 1 => {
                let localisable_fields_path = get_raw_localisable_fields_path(raw_data_path, version)?;
                let localisable_fields_file = BufReader::new(File::open(localisable_fields_path)?);
                from_reader(localisable_fields_file).map_err(From::from)
            }

            // Version 0 doesn't have loc fields as is. We have to bruteforce them.
            _ => Err(RLibError::AssemblyKitUnsupportedVersion(version))
        }
    }
}
