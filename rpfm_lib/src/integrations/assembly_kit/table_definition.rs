//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Assembly Kit table definition parsing and schema generation.
//!
//! This module handles the parsing of Assembly Kit schema files (table structure definitions)
//! and their conversion to RPFM's internal schema format. It supports three different Assembly
//! Kit versions used across Total War games.
//!
//! # Assembly Kit Schema Formats
//!
//! Different Total War games use different schema file formats:
//!
//! - **Version 0** (Empire, Napoleon): `.xsd` XML schema files with basic type and constraint information
//! - **Version 1** (Shogun 2): `TWaD_*.xml` files with enhanced metadata
//! - **Version 2** (Rome 2+): `TWaD_*.xml` files with full relationship data and field descriptions
//!
//! # Main Types
//!
//! ## Version 1 & 2 Formats
//!
//! - [`RawDefinition`]: Represents a complete table definition with all fields
//! - [`RawField`]: Individual field definition with type, constraints, and relationship info
//! - [`RawRelationshipsTable`]: Foreign key relationships between tables
//! - [`RawRelationship`]: Single foreign key relationship
//!
//! ## Version 0 Format (Legacy)
//!
//! - [`RawDefinitionV0`]: XSD schema root structure
//! - [`Element`]: XSD element with type and constraint information
//! - [`Index`]: Database index definition (used to derive relationships)
//!
//! # Functionality
//!
//! The main operations this module provides:
//!
//! 1. **Batch Reading**: [`RawDefinition::read_all()`] reads all table definitions from a directory
//! 2. **Individual Reading**: [`RawDefinition::read()`] parses a single definition file
//! 3. **Field Filtering**: [`RawDefinition::get_non_localisable_fields()`] separates translatable fields
//! 4. **Schema Conversion**: `From<&RawDefinition>` for [`Definition`] converts to RPFM format
//!
//! # Version 0 Processing
//!
//! Version 0 (Empire/Napoleon) uses a two-pass approach:
//! 1. First pass: Parse XSD files and extract basic field information and primary keys
//! 2. Second pass: Analyze index definitions to derive foreign key relationships
//!
//! This is necessary because Version 0 uses database-style indexes rather than explicit
//! foreign key declarations.
//!
//! # Type Mapping
//!
//! Assembly Kit types are mapped to RPFM field types:
//! - `yesno` → `Boolean`
//! - `single` → `F32`, `double` → `F64`
//! - `integer` → `I32`, `autonumber`/`card64` → `I64`
//! - `colour` → `ColourRGB`
//! - `text`/`expression` → `StringU8`/`StringU16` (or optional variants)

use itertools::Itertools;
use rayon::prelude::*;
use serde_derive::Deserialize;
use serde_xml_rs::from_reader;

use std::collections::BTreeMap;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

use crate::error::{Result, RLibError};

use super::*;
use super::get_raw_definition_paths;
use super::localisable_fields::RawLocalisableField;
use super::table_data::RawTableRow;

//---------------------------------------------------------------------------//
// Types for parsing the Assembly Kit Schema Files into.
//---------------------------------------------------------------------------//

/// Raw table definition parsed from Assembly Kit schema files.
///
/// This is the raw equivalent to RPFM's [`Definition`] struct. In Assembly Kit files,
/// this corresponds to a `TWaD_*.xml` file (versions 1-2) or `.xsd` file (version 0).
///
/// # Fields
///
/// * `name` - Table name with `.xml` extension (e.g., `"units_tables.xml"`)
/// * `fields` - All field definitions for this table
///
/// # Example Structure
///
/// A `TWaD_units_tables.xml` file contains field definitions like:
/// ```xml
/// <root>
///   <field primary_key="1" name="key" field_type="text" required="1"/>
///   <field primary_key="0" name="category" field_type="text" required="0"
///          column_source_table="unit_categories_tables"
///          column_source_column="key"/>
/// </root>
/// ```
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename = "root")]
pub struct RawDefinition {

    /// Table name with `.xml` extension (e.g., `"units_tables.xml"`) and without the 'TWaD_' prefix.
    pub name: Option<String>,

    /// All the field definitions within this table definition.
    #[serde(rename = "field")]
    pub fields: Vec<RawField>,
}

/// Individual field definition from Assembly Kit schema.
///
/// This is the raw equivalent to RPFM's [`Field`] struct, containing all metadata
/// about a single table column.
///
/// # Type Information
///
/// Assembly Kit uses string-based type names:
/// - `"yesno"` - Boolean value
/// - `"single"`, `"double"` - Floating point numbers
/// - `"integer"` - 32-bit integer
/// - `"autonumber"`, `"card64"` - 64-bit integer (often auto-incrementing)
/// - `"text"`, `"expression"` - String data
/// - `"colour"` - RGB color value
///
/// # Foreign Key Relationships
///
/// Relationships are defined via `column_source_table` and `column_source_column`:
/// - First element in `column_source_column` is the referenced primary key
/// - Additional elements (if present) are lookup columns for concatenated display
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename = "field")]
pub struct RawField {

    /// Primary key flag (`"1"` = true, `"0"` = false).
    pub primary_key: String,

    /// Field name (column name in the table).
    pub name: String,

    /// Assembly Kit type name (see struct documentation for type mapping).
    pub field_type: String,

    /// Required field flag (`"1"` = required, `"0"` = optional).
    pub required: String,

    /// Default value for this field when creating new rows.
    pub default_value: Option<String>,

    /// Maximum allowed string length for text fields.
    pub max_length: Option<String>,

    /// Filename flag - indicates this field contains a game file path.
    pub is_filename: Option<String>,

    /// Relative path where referenced files should be located.
    ///
    /// Multiple paths can be specified, separated by semicolons.
    pub filename_relative_path: Option<String>,

    /// Fragment path (internal use, not useful for modders).
    pub fragment_path: Option<String>,

    /// Referenced column names for foreign key relationships.
    ///
    /// First element is the referenced primary key column.
    /// Additional elements are lookup columns for composite display.
    pub column_source_column: Option<Vec<String>>,

    /// Referenced table name for foreign key relationships.
    pub column_source_table: Option<String>,

    /// Human-readable description of the field's purpose.
    pub field_description: Option<String>,

    /// Encyclopaedia export flag (`"1"` = export, `"0"` = don't export).
    ///
    /// Indicates if this field should be included in game encyclopaedia exports.
    pub encyclopaedia_export: Option<String>,

    /// Highlight color flag for marking unused/deprecated fields.
    ///
    /// `"#c8c8c8"` (gray) indicates an unused field in Warhammer 3.
    pub highlight_flag: Option<String>,

    /// Custom flag for old game (Empire/Napoleon/Shogun 2) type handling.
    ///
    /// When true, uses UTF-16 strings instead of UTF-8.
    pub is_old_game: Option<bool>,
}

/// Version 0 (Empire/Napoleon) XSD schema root structure.
///
/// Empire and Napoleon use `.xsd` XML Schema Definition files instead of
/// the `TWaD_` format used in later games. This struct represents the root
/// of such a schema file.
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename = "xsd_schema")]
pub struct RawDefinitionV0 {
    /// XSD elements defining the table structure.
    pub xsd_element: Vec<Element>,
}

/// Represents an XSD element definition from Assembly Kit v0 schema files.
///
/// Elements are the core building blocks of XSD schemas, representing individual
/// fields in database tables. Each element can have type constraints (via `SimpleType`),
/// nested structures (via `ComplexType`), and metadata annotations.
///
/// # Field Mapping
///
/// - `name`: Column name in the database table
/// - `jet_type`: Microsoft Jet database type (e.g., "Text", "Long", "Boolean")
/// - `min_occurs`: Minimum occurrences (0 = optional, 1 = required)
/// - `xsd_annotation`: Contains metadata like index definitions
/// - `xsd_simple_type`: Type constraints (e.g., string max length)
/// - `xsd_complex_type`: Nested element sequences for complex types
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename = "xsd_element")]
pub struct Element {
    /// The name of this element (field/column name).
    pub name: Option<String>,

    /// Microsoft Jet database type identifier.
    ///
    /// Common values: "Text" (string), "Long" (i32), "Boolean", "Single" (f32), "Double" (f64).
    #[serde(rename = "od_jetType")]
    pub jet_type: Option<String>,

    /// Minimum number of occurrences for this element.
    ///
    /// - `0`: Field is optional
    /// - `1` or higher: Field is required
    #[serde(rename = "minOccurs")]
    pub min_occurs: Option<i32>,

    /// Annotation containing metadata like index definitions.
    #[serde(rename = "xsd_annotation")]
    pub xsd_annotation: Option<Annotation>,

    /// Simple type definition with constraints (e.g., max string length).
    #[serde(rename = "xsd_simpleType")]
    pub xsd_simple_type: Option<Vec<SimpleType>>,

    /// Complex type definition for nested element sequences.
    #[serde(rename = "xsd_complexType")]
    pub xsd_complex_type: Option<Vec<ComplexType>>,
}

/// Defines a simple type with restrictions in XSD schemas.
///
/// Simple types are used to apply constraints to basic data types, such as
/// limiting the maximum length of a string field.
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename = "xsd_simpleType")]
pub struct SimpleType {
    /// The restriction applied to this simple type (e.g., max length).
    pub xsd_restriction: Option<Restriction>,
}

/// Defines a complex type containing nested element sequences.
///
/// Complex types are used when a field contains multiple sub-elements organized
/// in a specific order. In Assembly Kit schemas, these are typically used for
/// nested table structures, though most tables use simple flat structures.
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename = "xsd_complexType")]
pub struct ComplexType {
    /// The ordered sequence of elements within this complex type.
    #[serde(rename = "xsd_sequence")]
    pub xsd_sequence: Sequence,
}

/// Represents an ordered sequence of XSD elements.
///
/// Sequences define the order in which child elements must appear within
/// a complex type. Each element in the sequence can itself be a simple or
/// complex type.
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename = "xsd_sequence")]
pub struct Sequence {
    /// The ordered list of elements in this sequence.
    pub xsd_element: Vec<Element>,
}

/// Defines restrictions/constraints on an XSD simple type.
///
/// Restrictions are used to constrain the values of a simple type, such as
/// limiting the maximum length of a string. The `base` field specifies which
/// base type the restriction applies to.
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename = "xsd_restriction")]
pub struct Restriction {
    /// The base XSD type being restricted (e.g., "xsd:string", "xsd:int").
    pub base: String,

    /// Maximum length constraint for string types.
    #[serde(rename = "xsd_maxLength")]
    pub max_lenght: Option<MaxLength>
}

/// Specifies the maximum length constraint for a string field.
///
/// This constraint limits how many characters a string field can contain.
/// Used in XSD restrictions to define database column size limits.
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename = "xsd_maxLength")]
pub struct MaxLength {
    /// The maximum number of characters allowed.
    pub value: i32
}

/// Contains annotation metadata for XSD elements.
///
/// Annotations provide additional information about schema elements that isn't
/// part of the core validation rules. In Assembly Kit schemas, annotations are
/// primarily used to store database index definitions via the `AppInfo` structure.
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename = "xsd_annotation")]
pub struct Annotation {
    /// Application-specific information, containing index definitions.
    #[serde(rename = "xsd_appinfo")]
    pub xsd_appinfo: Option<AppInfo>
}

/// Contains application-specific information within XSD annotations.
///
/// This structure holds database-specific metadata that extends the base XSD schema.
/// In Assembly Kit schemas, it primarily contains index definitions that describe
/// primary keys, foreign keys, and unique constraints on table columns.
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename = "xsd_appinfo")]
pub struct AppInfo {
    /// List of database index definitions for this element.
    #[serde(rename = "od_index")]
    pub od_index: Option<Vec<Index>>
}

/// Defines a database index on a table column.
///
/// Indexes are used to derive foreign key relationships in Assembly Kit v0 schemas.
/// Since v0 schemas don't explicitly define relationships between tables, RPFM
/// infers them by matching index names across tables.
///
/// # Relationship Inference
///
/// When an index name appears in multiple tables, RPFM creates a foreign key
/// relationship between them. For example:
///
/// - Table A has index "fk_building" on column "building_key"
/// - Table B has index "fk_building" on column "key"
/// - RPFM infers: A.building_key → B.key
///
/// # Boolean String Fields
///
/// The `primary`, `unique`, and `clustered` fields use string values "true"/"false"
/// instead of booleans due to the XSD format.
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename = "od_index")]
pub struct Index {
    /// The name of this index.
    ///
    /// Index names are used to match relationships across tables. Identical names
    /// in different tables indicate a foreign key relationship.
    #[serde(rename = "index-name")]
    pub name: String,

    /// The column(s) this index applies to.
    ///
    /// Multiple columns are separated by semicolons (e.g., "col1;col2").
    #[serde(rename = "index-key")]
    pub key: String,

    /// Whether this is a primary key index ("true"/"false").
    #[serde(rename = "primary")]
    pub primary: String,

    /// Whether this index enforces uniqueness ("true"/"false").
    #[serde(rename = "unique")]
    pub unique: String,

    /// Whether this is a clustered index ("true"/"false").
    #[serde(rename = "clustered")]
    pub clustered: String,
}

/// Foreign key relationships table from Assembly Kit.
///
/// This corresponds to the `TWaD_relationships.xml` file found in Version 2
/// Assembly Kits (Rome 2+). It defines all foreign key relationships between tables.
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename = "root")]
pub struct RawRelationshipsTable {
    /// Table name (should be "relationships").
    pub name: Option<String>,

    /// All foreign key relationships defined in the Assembly Kit.
    #[serde(rename = "relationship")]
    pub relationships: Vec<RawRelationship>,
}

/// Single foreign key relationship definition.
///
/// Defines a foreign key constraint from one table's column to another table's column.
///
/// # Example
///
/// A relationship from `units_tables.category` to `unit_categories_tables.key`:
/// ```xml
/// <relationship>
///   <table_name>units_tables</table_name>
///   <column_name>category</column_name>
///   <foreign_table_name>unit_categories_tables</foreign_table_name>
///   <foreign_column_name>key</foreign_column_name>
/// </relationship>
/// ```
#[derive(Clone, Debug, Default, Deserialize)]
pub struct RawRelationship {
    /// Source table name containing the foreign key column.
    pub table_name: String,

    /// Source column name (the foreign key field).
    pub column_name: String,

    /// Referenced table name.
    pub foreign_table_name: String,

    /// Referenced column name (typically a primary key).
    pub foreign_column_name: String
}

//---------------------------------------------------------------------------//
// Implementations
//---------------------------------------------------------------------------//

/// Implementation of `RawDefinition`.
impl RawDefinition {

    /// Reads all table definitions from an Assembly Kit directory.
    ///
    /// This function scans the provided directory for Assembly Kit definition files
    /// and parses them into [`RawDefinition`] structs. The parsing logic varies
    /// significantly by version.
    ///
    /// # Version-Specific Behavior
    ///
    /// ## Version 1 & 2 (Shogun 2, Rome 2+)
    /// - Reads `TWaD_*.xml` files directly
    /// - Each file is a complete, self-contained definition
    ///
    /// ## Version 0 (Empire, Napoleon)
    /// - Reads `.xsd` XML Schema files
    /// - Uses two-pass processing:
    ///   1. Parse all XSD files and extract field info + primary keys
    ///   2. Analyze index definitions to derive foreign key relationships
    /// - This is necessary because Version 0 uses database-style indexes rather than
    ///   explicit foreign key declarations
    ///
    /// # Arguments
    ///
    /// * `raw_definitions_folder` - Directory containing Assembly Kit definition files
    /// * `version` - Assembly Kit version (0 = Empire/Napoleon, 1 = Shogun 2, 2 = Rome 2+)
    /// * `tables_to_skip` - Table names (without extension) to exclude from parsing
    ///
    /// # Returns
    ///
    /// Returns a vector of successfully parsed table definitions. Tables in the
    /// blacklist or skip list are excluded.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The version is unsupported (not 0, 1, or 2)
    /// - The directory cannot be read
    /// - Any definition file has malformed XML
    pub fn read_all(raw_definitions_folder: &Path, version: i16, tables_to_skip: &[&str]) -> Result<Vec<Self>> {
        let definitions = get_raw_definition_paths(raw_definitions_folder, version)?;
        match version {
            2 | 1 => {
                definitions.iter()
                    .filter(|x| !BLACKLISTED_TABLES.contains(&x.file_name().unwrap().to_str().unwrap()))
                    .filter(|x| {
                        let table_name = x.file_stem().unwrap().to_str().unwrap().split_at(5).1;
                        !tables_to_skip.par_iter().any(|vanilla_name| vanilla_name == &table_name)
                    })
                    .map(|x| Self::read(x, version))
                    .collect::<Result<Vec<Self>>>()
            }
            0 => {
                let v0s = definitions.iter()
                    .filter(|x| !BLACKLISTED_TABLES.contains(&x.file_name().unwrap().to_str().unwrap()))
                    .filter(|x| {
                        let table_name = x.file_stem().unwrap().to_str().unwrap();
                        !tables_to_skip.par_iter().any(|vanilla_name| vanilla_name == &table_name)
                    })
                    .filter_map(|x| RawDefinitionV0::read(x).transpose())
                    .map(|def_v0| {

                        // NOTE: This from processes the primary keys already.
                        let raw = match def_v0 {
                            Ok(ref def_v0) => Self::from(def_v0),
                            Err(_) => Self::default(),
                        };
                        def_v0.map(|def_v0| (def_v0, raw))
                    })
                    .collect::<Result<Vec<(RawDefinitionV0, RawDefinition)>>>()?;

                // We need to do a second pass because without the entire set available we cannot figure out the references.
                Ok(v0s.iter()
                    .map(|(def_v0, new_def)| {
                        let mut new_def = new_def.clone();

                        if let Some(elements) = def_v0.xsd_element.get(1) {
                            if let Some(ref table_name) = elements.name {
                                if let Some(ref ann) = elements.xsd_annotation {
                                    if let Some(ref app) = ann.xsd_appinfo {
                                        if let Some(ref od_index) = app.od_index {
                                            for index in od_index {

                                                // Ignore indexes of unused fields, the primary key, and field-specific indexes.
                                                if index.name == "PrimaryKey" || index.name == index.key.trim() {
                                                    continue;
                                                }

                                                // Indexes follow the format "remotetablelocaltable", with a 61 char limit. To find the remote table,
                                                // we need to remove the local one, and to do so, we need to find what part of the local one is actually in the index name.
                                                let remote_table_name = if index.name.chars().count() == 61 {
                                                    let mut table_name = table_name.clone();
                                                    let mut remote_table_name = String::new();
                                                    loop {
                                                        if index.name.ends_with(&*table_name) {
                                                            remote_table_name = index.name.clone();
                                                            if let Some(sub) = index.name.len().checked_sub(table_name.len()) {
                                                                remote_table_name.truncate(sub);
                                                            } else {
                                                                remote_table_name = String::new();
                                                            }
                                                            break;
                                                        } else {
                                                            if table_name.is_empty() {
                                                                break;
                                                            }

                                                            table_name.pop();
                                                        }
                                                    }

                                                    remote_table_name
                                                } else {
                                                    let mut remote_table_name = index.name.clone();
                                                    if let Some(sub) = index.name.len().checked_sub(table_name.len()) {
                                                        remote_table_name.truncate(sub);
                                                    } else {
                                                        remote_table_name = String::new();
                                                    }
                                                    remote_table_name
                                                };

                                                // Now we need to find the primary key of the remote table, if any.
                                                if !remote_table_name.is_empty() {
                                                    if let Some(remote_def) = v0s.par_iter().find_map_first(|(def_v0, new_def)| {
                                                        if let Some(elements) = def_v0.xsd_element.get(1) {
                                                            if let Some(ref table_name) = elements.name {
                                                                if table_name == &remote_table_name {
                                                                    Some(new_def)
                                                                } else { None }
                                                            } else { None }
                                                        } else { None }
                                                    }) {

                                                        // No fucking clue if ANY reference is to a multikey table, but if is, we'll use the first key as ref key, and the rest as lookups.
                                                        let primary_keys = remote_def.fields.iter().filter(|x| x.primary_key == "1" || x.name == "key").collect::<Vec<_>>();
                                                        if !primary_keys.is_empty() {
                                                            for field in &mut new_def.fields {
                                                                if field.name == index.key.trim() {
                                                                    field.column_source_table = Some(remote_table_name.to_string());
                                                                    field.column_source_column = Some(primary_keys.iter().map(|x| x.name.to_string()).collect());
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        new_def
                    })
                    .collect())
            }
            _ => Err(RLibError::AssemblyKitUnsupportedVersion(version))
        }
    }

    /// Parses a single Assembly Kit definition file.
    ///
    /// Reads and parses one table definition file from the Assembly Kit.
    ///
    /// # Arguments
    ///
    /// * `raw_definition_path` - Path to the definition file (e.g., `TWaD_units_tables.xml`)
    /// * `version` - Assembly Kit version (1 = Shogun 2, 2 = Rome 2+)
    ///
    /// # Returns
    ///
    /// Returns the parsed [`RawDefinition`] with the table name set to the filename
    /// without the `TWaD_` prefix (e.g., `"units_tables.xml"`).
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The version is not 1 or 2 (use [`RawDefinitionV0::read()`] for version 0)
    /// - The file cannot be opened (returns [`RLibError::AssemblyKitNotFound`])
    /// - The XML is malformed
    ///
    /// # Note
    ///
    /// For Version 0 (Empire/Napoleon), use [`RawDefinitionV0::read()`] instead as the
    /// file format is completely different (.xsd vs .xml).
    pub fn read(raw_definition_path: &Path, version: i16) -> Result<Self> {
        match version {
            2 | 1 => {
                let definition_file = BufReader::new(File::open(raw_definition_path).map_err(|_| RLibError::AssemblyKitNotFound)?);
                let mut definition: Self = from_reader(definition_file)?;
                definition.name = Some(raw_definition_path.file_name().unwrap().to_str().unwrap().split_at(5).1.to_string());
                Ok(definition)
            }

            _ => Err(RLibError::AssemblyKitUnsupportedVersion(version))
        }
    }

    /// Filters out localisable fields from the definition.
    ///
    /// Returns only the fields that are not marked as localisable (translatable) and
    /// are present in the test row data. This is used when processing Assembly Kit
    /// table data to separate regular fields from translation fields.
    ///
    /// # Arguments
    ///
    /// * `raw_localisable_fields` - List of all localisable fields from `TExc_LocalisableFields.xml`
    /// * `test_row` - Sample row data used to verify field presence
    ///
    /// # Returns
    ///
    /// Returns a vector of [`Field`] instances for non-localisable fields that exist
    /// in the test data.
    ///
    /// # Note
    ///
    /// Fields are excluded if:
    /// - They're listed in `raw_localisable_fields` for this table
    /// - They don't appear in the test row
    /// - They have a "state" attribute (marked as modified/deprecated)
    pub fn get_non_localisable_fields(&self, raw_localisable_fields: &[RawLocalisableField], test_row: &RawTableRow) -> Vec<Field> {
        let raw_table_name = &self.name.as_ref().unwrap()[..self.name.as_ref().unwrap().len() - 4];
        let localisable_fields_names = raw_localisable_fields.iter()
            .filter(|x| x.table_name == raw_table_name)
            .map(|x| &*x.field)
            .collect::<Vec<&str>>();

        self.fields.iter()
            .filter(|x| match test_row.fields.iter().find(|y| x.name == y.field_name) {
                Some(y) => y.state.is_none(),
                None => false,
            })
            .filter(|x| !localisable_fields_names.contains(&&*x.name))
            .map(From::from)
            .collect::<Vec<Field>>()
    }
}

impl From<&RawDefinition> for Definition {
    fn from(raw_definition: &RawDefinition) -> Self {
        let fields = raw_definition.fields.iter().map(From::from).collect::<Vec<_>>();
        Self::new_with_fields(-100, &fields, &[], None)
    }
}


impl From<&RawField> for Field {
    fn from(raw_field: &RawField) -> Self {

        let is_old_game = raw_field.is_old_game.unwrap_or(false);

        let field_type = match &*raw_field.field_type {
            "yesno" => FieldType::Boolean,
            "single" => FieldType::F32,
            "double" => FieldType::F64,
            "integer" => FieldType::I32,
            "autonumber" | "card64" => FieldType::I64,
            "colour" => FieldType::ColourRGB,
            "expression" | "text" => {
                if raw_field.required == "1" {
                    if is_old_game {
                        FieldType::StringU16
                    } else {
                        FieldType::StringU8
                    }
                }
                else if is_old_game {
                    FieldType::OptionalStringU16
                } else {
                    FieldType::OptionalStringU8
                }
            },
            _ => if is_old_game {
                FieldType::StringU16
            } else {
                FieldType::StringU8
            },
        };

        let (is_reference, lookup) = if let Some(x) = &raw_field.column_source_table {
            if let Some(y) = &raw_field.column_source_column {
                if y.len() > 1 { (Some((x.to_owned(), y[0].to_owned())), Some(y[1..].to_vec()))}
                else { (Some((x.to_owned(), y[0].to_owned())), None) }
            } else { (None, None) }
        }
        else { (None, None) };

        // CA sometimes uses comma as separator, and has random spaces between paths.
        let filename_relative_path = raw_field.filename_relative_path.clone().map(|x| {
            x.split(',').map(|y| y.trim()).join(";")
        });

        // Some fields are marked as filename, but only have fragment paths, which do not seem to correlate to game file paths.
        // We need to disable those to avoid false positives on diagnostics.
        let is_filename = match raw_field.is_filename {
            Some(_) => !(raw_field.fragment_path.is_some() && raw_field.filename_relative_path.is_none()),
            None => false,
        };

        Self::new(
            raw_field.name.to_owned(),
            field_type,
            raw_field.primary_key == "1",
            raw_field.default_value.clone(),
            is_filename,
            filename_relative_path,
            is_reference,
            lookup,
            if let Some(x) = &raw_field.field_description { x.to_owned() } else { String::new() },
            0,
            0,
            BTreeMap::new(),
            None
        )
    }
}

impl RawDefinitionV0 {

    /// Parses a Version 0 (Empire/Napoleon) XSD schema file.
    ///
    /// Reads and parses an XSD (XML Schema Definition) file from the Empire or
    /// Napoleon Assembly Kit. The XSD format is significantly different from the
    /// `TWaD_` format used in later games.
    ///
    /// # Arguments
    ///
    /// * `raw_definition_path` - Path to the `.xsd` file
    ///
    /// # Returns
    ///
    /// Returns `Ok(Some(definition))` if the file was parsed successfully, `Ok(None)`
    /// if the file was empty, or an error if parsing failed.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The file cannot be opened (returns [`RLibError::AssemblyKitNotFound`])
    /// - The XML/XSD is malformed
    ///
    /// # Implementation Note
    ///
    /// Due to limitations in `serde_xml_rs`, this function performs extensive string
    /// replacements on the XSD content before parsing to normalize XML namespace
    /// prefixes (`xsd:` and `xs:` → `xsd_`, `od:` → `od_`).
    pub fn read(raw_definition_path: &Path) -> Result<Option<Self>> {
        let mut definition_file = BufReader::new(File::open(raw_definition_path).map_err(|_| RLibError::AssemblyKitNotFound)?);

        // Before deserializing the data, due to limitations of serde_xml_rs, we have to rename all rows, because unique names for
        // rows in each file is not supported for deserializing. Same for the fields, we have to change them to something more generic.
        let mut buffer = String::new();
        definition_file.read_to_string(&mut buffer)?;

        if buffer.is_empty() {
            return Ok(None)
        }

        // Rust doesn't like : in variable names when deserializing.
        buffer = buffer.replace("xsd:schema", "xsd_schema");
        buffer = buffer.replace("xsd:element", "xsd_element");
        buffer = buffer.replace("xsd:complexType", "xsd_complexType");
        buffer = buffer.replace("xsd:sequence", "xsd_sequence");
        buffer = buffer.replace("xsd:attribute", "xsd_attribute");
        buffer = buffer.replace("xsd:annotation", "xsd_annotation");
        buffer = buffer.replace("xsd:appinfo", "xsd_appinfo");
        buffer = buffer.replace("od:index", "od_index");
        buffer = buffer.replace("xsd:sequence", "xsd_sequence");
        buffer = buffer.replace("xsd:simpleType", "xsd_simpleType");
        buffer = buffer.replace("xsd:restriction", "xsd_restriction");
        buffer = buffer.replace("xsd:maxLength", "xsd_maxLength");
        buffer = buffer.replace("od:jetType", "od_jetType");

        buffer = buffer.replace("xs:schema", "xsd_schema");
        buffer = buffer.replace("xs:element", "xsd_element");
        buffer = buffer.replace("xs:complexType", "xsd_complexType");
        buffer = buffer.replace("xs:sequence", "xsd_sequence");
        buffer = buffer.replace("xs:attribute", "xsd_attribute");
        buffer = buffer.replace("xs:annotation", "xsd_annotation");
        buffer = buffer.replace("xs:appinfo", "xsd_appinfo");
        buffer = buffer.replace("xs:sequence", "xsd_sequence");
        buffer = buffer.replace("xs:simpleType", "xsd_simpleType");
        buffer = buffer.replace("xs:restriction", "xsd_restriction");
        buffer = buffer.replace("xs:maxLength", "xsd_maxLength");

        // Only if the table has data we deserialize it. If not, we just create an empty one.
        let definition: RawDefinitionV0 = from_reader(buffer.as_bytes())?;

        //dbg!(&definition);
        Ok(Some(definition))
    }
}

/// Old games don't use references, but rather indexes like a database. This means we're unable to find
/// the referenced column without having the reference definition. So ref data needs to be calculated after this.
impl From<&RawDefinitionV0> for RawDefinition {
    fn from(value: &RawDefinitionV0) -> Self {
        let mut definition = Self::default();

        // Second element has the fields.
        if let Some(elements) = value.xsd_element.get(1) {
            definition.name = elements.name.clone().map(|x| format!("{x}.xml"));

            // Try to get the indexes to check what do we need to mark as key.
            let primary_keys = if let Some(ref ann) = elements.xsd_annotation {
                if let Some(ref app) = ann.xsd_appinfo {
                    if let Some(ref od_index) = app.od_index {
                        od_index.iter().find_map(|index| {
                            if index.name == "PrimaryKey" {

                                // Always trim to remove the final space, then split by space to find all the keys of the table.
                                let keys = index.key.trim().split(' ').collect::<Vec<_>>();
                                if keys.is_empty() {
                                    None
                                } else {
                                    Some(keys)
                                }
                            } else {
                                None
                            }
                        }).unwrap_or(vec![])
                    } else { vec![] }
                } else { vec![] }
            } else { vec![] };

            if let Some(complex) = &elements.xsd_complex_type {
                if let Some(elements) = complex.first() {
                    for element in &elements.xsd_sequence.xsd_element {

                        // For a field to be valid we need name and type.
                        if let Some(ref name) = element.name {
                            if let Some(ref jet_type) = element.jet_type {

                                let mut field = RawField::default();
                                field.name = name.to_owned();

                                field.field_type = match &**jet_type {
                                    "yesno" => "yesno".to_owned(),
                                    "integer" => "integer".to_owned(),
                                    "longinteger" | "autonumber" => "autonumber".to_owned(),
                                    "decimal" | "single" => "single".to_owned(),
                                    "double" => "double".to_owned(),
                                    "text" | "memo" | "oleobject" | "replicationid" => "text".to_owned(),

                                    // These are dates as in a DateTime format. Treat them as text for now.
                                    "datetime" => "text".to_owned(),

                                    _ => todo!("{}", jet_type),
                                };

                                if primary_keys.contains(&&*field.name) {
                                    field.primary_key = "1".to_owned();
                                } else {
                                    field.primary_key = "0".to_owned();
                                }

                                field.is_old_game = Some(true);

                                definition.fields.push(field);
                            }
                        }
                    }
                }
            }
        }

        definition
    }
}
