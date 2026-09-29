//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Methods to read DB and Loc tables, and their definitions.
//!
//! Tables are described by their columns as rows see them: colour columns are merged,
//! bitwise columns split and enum columns converted, so each row has one value per column.

use serde::{Deserialize, Serialize};
use serde_json::Value;

use rpfm_lib::schema::FieldType;

use super::Request;
use super::files::FileRef;

/// Default amount of rows returned by [`GetTableRows`].
pub const DEFAULT_ROWS_LIMIT: usize = 100;

/// A column of a table.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ColumnInfo {

    /// Name of the column.
    pub name: String,

    /// Type of the values of the column.
    pub field_type: FieldType,

    /// If the column is part of the key of the table.
    pub is_key: bool,

    /// Column of another table the values of this one must exist in, if any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reference: Option<ColumnReference>,

    /// Value of the column in new rows, if it has one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_value: Option<String>,

    /// Description of the column, if it has one.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub description: String,
}

/// A column of another table.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ColumnReference {

    /// Name of the table, without the `_tables` suffix.
    pub table: String,

    /// Name of the column.
    pub column: String,
}

/// `table.info`: returns the definition and row count of a table.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GetTableInfo {

    /// The table.
    pub file: FileRef,
}

/// Definition and size of a table.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TableInfo {

    /// Name of the table, like `units_tables`, or `loc` for Loc files.
    pub table_name: String,

    /// Version of the table's definition.
    pub version: i32,

    /// Columns of the table, in the order row values are.
    pub columns: Vec<ColumnInfo>,

    /// Amount of rows in the table.
    pub row_count: usize,
}

/// `table.rows`: returns rows of a table, optionally filtered and with only some columns.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GetTableRows {

    /// The table.
    pub file: FileRef,

    /// If set, only these columns are returned, in this order.
    #[serde(default)]
    pub columns: Option<Vec<String>>,

    /// Only rows matching all these filters are returned.
    #[serde(default)]
    pub filters: Vec<RowFilter>,

    /// Amount of matching rows to skip.
    #[serde(default)]
    pub offset: usize,

    /// Maximum amount of rows to return. Defaults to [`DEFAULT_ROWS_LIMIT`].
    #[serde(default)]
    pub limit: Option<usize>,
}

/// A condition on the value of a column, compared as text.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RowFilter {

    /// Name of the column.
    pub column: String,

    /// How the value is compared.
    pub op: FilterOp,

    /// Value to compare with.
    pub value: String,

    /// If the comparison ignores case.
    #[serde(default)]
    pub ignore_case: bool,
}

/// Comparison of a [`RowFilter`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FilterOp {

    /// The value is the same as the filter's.
    Equals,

    /// The value is not the same as the filter's.
    NotEquals,

    /// The value contains the filter's.
    Contains,

    /// The value starts with the filter's.
    StartsWith,

    /// The value ends with the filter's.
    EndsWith,
}

/// A page of the rows of a table.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TableRows {

    /// Names of the columns of the values of each row.
    pub columns: Vec<String>,

    /// The rows in the page.
    pub rows: Vec<TableRow>,

    /// Amount of rows matching the filters, in all pages.
    pub total: usize,
}

/// A row of a table.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TableRow {

    /// Position of the row in the table. Used to edit it.
    pub index: usize,

    /// Values of the row, one per returned column: booleans, numbers or strings.
    pub values: Vec<Value>,
}

/// `schema.definition`: returns the columns of a table as defined in the schema.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GetTableDefinition {

    /// Name of the table, like `units_tables`.
    pub table_name: String,

    /// Version of the definition. If not set, the version the table has in the game files is used,
    /// or the newest one if the game files are not loaded or don't have the table.
    #[serde(default)]
    pub version: Option<i32>,
}

/// Definition of a table in the schema.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TableDefinition {

    /// Name of the table.
    pub table_name: String,

    /// Version of the definition.
    pub version: i32,

    /// Columns of the table, in the order row values are.
    pub columns: Vec<ColumnInfo>,
}

impl Request for GetTableInfo {
    const METHOD: &'static str = "table.info";
    type Response = TableInfo;
}

impl Request for GetTableRows {
    const METHOD: &'static str = "table.rows";
    type Response = TableRows;
}

impl Request for GetTableDefinition {
    const METHOD: &'static str = "schema.definition";
    type Response = TableDefinition;
}
