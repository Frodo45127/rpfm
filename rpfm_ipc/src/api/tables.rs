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

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use std::collections::BTreeMap;
use std::path::PathBuf;

use rpfm_extensions::merge::{MergeConflict, MergeResolution};

use rpfm_lib::schema::FieldType;

use super::{default_true, Done, Request};
use super::files::FileRef;

/// Default amount of rows returned by [`GetTableRows`].
pub const DEFAULT_ROWS_LIMIT: usize = 100;

/// A column of a table.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ColumnInfo {

    /// Name of the column.
    pub name: String,

    /// Type of the values of the column, like `StringU8`, `I32`, `F32` or `Boolean`.
    #[schemars(with = "serde_json::Value")]
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
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ColumnReference {

    /// Name of the table, without the `_tables` suffix.
    pub table: String,

    /// Name of the column.
    pub column: String,
}

/// `table.info`: returns the definition and row count of a table.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct GetTableInfo {

    /// The table.
    pub file: FileRef,
}

/// Definition and size of a table.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
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
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
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

    /// Maximum amount of rows to return. Defaults to 100.
    #[serde(default)]
    pub limit: Option<usize>,
}

/// A condition on the value of a column, compared as text.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
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

/// Comparison of a row filter.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
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
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct TableRows {

    /// Names of the columns of the values of each row.
    pub columns: Vec<String>,

    /// The rows in the page.
    pub rows: Vec<TableRow>,

    /// Amount of rows matching the filters, in all pages.
    pub total: usize,
}

/// A row of a table.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct TableRow {

    /// Position of the row in the table. Used to edit it.
    pub index: usize,

    /// Values of the row, one per returned column: booleans, numbers or strings.
    pub values: Vec<Value>,
}

/// `schema.definition`: returns the columns of a table as defined in the schema.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct GetTableDefinition {

    /// Name of the table, like `units_tables`.
    pub table_name: String,

    /// Version of the definition. If not set, the version the table has in the game files is used,
    /// or the newest one if the game files are not loaded or don't have the table.
    #[serde(default)]
    pub version: Option<i32>,
}

/// Definition of a table in the schema.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
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

/// `table.edit`: edits rows of a DB or Loc table in an open pack.
///
/// Edits are applied in order, each one on the result of the previous ones. If any of them fails, none is applied.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct EditTable {

    /// Key of the pack.
    pub pack: String,

    /// Path of the table in the pack.
    pub path: String,

    /// Edits to apply.
    pub edits: Vec<RowEdit>,
}

/// An edit of the rows of a table.
///
/// Values are given by column name, as booleans, numbers or strings, and converted to the type of their column.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "op", rename_all = "snake_case", deny_unknown_fields)]
pub enum RowEdit {

    /// Adds a row. Columns not set get their default value.
    Insert {

        /// Position of the new row. If not set, it's added at the end.
        #[serde(default)]
        index: Option<usize>,

        /// Values of the new row, by column name.
        #[serde(default)]
        values: BTreeMap<String, Value>,
    },

    /// Changes values of a row.
    Update {

        /// Position of the row.
        index: usize,

        /// New values, by column name. Columns not set keep their value.
        values: BTreeMap<String, Value>,
    },

    /// Removes rows.
    Delete {

        /// Positions of the rows to remove, as they're before this edit.
        indexes: Vec<usize>,
    },
}

/// Result of editing a table.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct TableEdited {

    /// Amount of rows the table has after the edits.
    pub row_count: usize,
}

impl Request for EditTable {
    const METHOD: &'static str = "table.edit";
    type Response = TableEdited;
}

/// Default amount of values returned by [`GetColumnValues`].
pub const DEFAULT_VALUES_LIMIT: usize = 500;

/// `table.column_values`: returns the distinct values of a column of a table, sorted.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct GetColumnValues {

    /// Name of the table, like `factions_tables`.
    pub table_name: String,

    /// Name of the column.
    pub column: String,

    /// If the tables of the open packs are included.
    #[serde(default = "default_true")]
    pub include_packs: bool,

    /// If the tables of the game files and the parent packs are included.
    #[serde(default = "default_true")]
    pub include_dependencies: bool,

    /// Only values starting with this are returned.
    #[serde(default)]
    pub prefix: String,

    /// Amount of values to skip.
    #[serde(default)]
    pub offset: usize,

    /// Maximum amount of values to return. Defaults to 500.
    #[serde(default)]
    pub limit: Option<usize>,
}

/// A page of the values of a column.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ColumnValues {

    /// The values in the page, sorted.
    pub values: Vec<String>,

    /// Amount of values matching the request, in all pages.
    pub total: usize,
}

impl Request for GetColumnValues {
    const METHOD: &'static str = "table.column_values";
    type Response = ColumnValues;
}

/// `table.merge`: merges tables of the same type of an open pack into a new one.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct MergeTables {

    /// Key of the pack.
    pub pack: String,

    /// Paths of the tables to merge.
    pub paths: Vec<String>,

    /// Path of the merged table.
    pub merged_path: String,

    /// If the merged tables are deleted afterwards.
    #[serde(default)]
    pub delete_sources: bool,

    /// If rows are merged by key against the vanilla data, instead of concatenated.
    /// Rows that can't be reconciled automatically are returned as conflicts, and nothing is written.
    #[serde(default)]
    pub delta: bool,

    /// How to resolve the conflicts of a previous delta merge.
    #[serde(default)]
    #[schemars(with = "Vec<serde_json::Value>")]
    pub resolutions: Vec<MergeResolution>,
}

/// Result of merging tables.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct TablesMerged {

    /// Path of the merged table, if the merge was done.
    pub merged: Option<String>,

    /// Rows that couldn't be reconciled, if the merge wasn't done. Pass resolutions for them to merge again.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[schemars(with = "Vec<serde_json::Value>")]
    pub conflicts: Vec<MergeConflict>,
}

/// `table.upgrade`: updates a table of an open pack to the version it has in the game files.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct UpgradeTable {

    /// Key of the pack.
    pub pack: String,

    /// Path of the table.
    pub path: String,
}

/// Result of updating a table.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct TableUpgraded {

    /// Version the table had.
    pub old_version: i32,

    /// Version the table has now.
    pub new_version: i32,

    /// Columns removed by the update.
    pub deleted_columns: Vec<String>,

    /// Columns added by the update, with their default value.
    pub added_columns: Vec<String>,
}

/// `table.rename_key`: changes a value of a key column of a table in every table of an open pack,
/// including the columns referencing it and the loc keys generated from it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RenameKey {

    /// Key of the pack.
    pub pack: String,

    /// Name of the table the key is from, like `factions_tables`.
    pub table_name: String,

    /// Name of the key column.
    pub column: String,

    /// Current value.
    pub old_value: String,

    /// New value.
    pub new_value: String,

    /// Version of the table's definition. Defaults to the version in the game files, or the newest one.
    #[serde(default)]
    pub version: Option<i32>,
}

/// Files edited by an operation.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct FilesEdited {

    /// Paths of the edited files.
    pub edited: Vec<String>,
}

/// `table.add_key_deletes`: adds rows to a key deletes table of an open pack, to delete keys of a table in the game.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AddKeyDeletes {

    /// Key of the pack.
    pub pack: String,

    /// File name of the key deletes table, under `db/twad_key_deletes_tables/`.
    pub file_name: String,

    /// Name of the table the keys belong to, like `units_tables`.
    pub table_name: String,

    /// Keys to delete.
    pub keys: Vec<String>,
}

/// `table.export_tsv`: writes a table to a TSV file.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ExportTsv {

    /// The table. Assembly Kit tables can't be exported.
    pub file: FileRef,

    /// Path of the TSV file to write.
    pub destination: PathBuf,

    /// If the TSV uses the old column order, with keys first. Defaults to the server's setting.
    #[serde(default)]
    pub keys_first: Option<bool>,
}

/// `table.import_tsv`: replaces a table of an open pack with the contents of a TSV file, keeping its GUID.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ImportTsv {

    /// Key of the pack.
    pub pack: String,

    /// Path of the table in the pack.
    pub path: String,

    /// Path of the TSV file to read.
    pub tsv_path: PathBuf,
}

impl Request for MergeTables {
    const METHOD: &'static str = "table.merge";
    type Response = TablesMerged;
}

impl Request for UpgradeTable {
    const METHOD: &'static str = "table.upgrade";
    type Response = TableUpgraded;
}

impl Request for RenameKey {
    const METHOD: &'static str = "table.rename_key";
    type Response = FilesEdited;
}

impl Request for AddKeyDeletes {
    const METHOD: &'static str = "table.add_key_deletes";
    type Response = FilesEdited;
}

impl Request for ExportTsv {
    const METHOD: &'static str = "table.export_tsv";
    type Response = Done;
}

impl Request for ImportTsv {
    const METHOD: &'static str = "table.import_tsv";
    type Response = TableEdited;
}
