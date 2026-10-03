//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Methods about the schema of the selected game: its tables, local patches, and updates.
//!
//! Definitions of single tables are read with [`GetTableDefinition`](super::tables::GetTableDefinition).

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use serde_json::Value;

use std::collections::BTreeMap;

use super::{Done, Request};
use super::session::SessionStatus;

/// Keys a column patch can set, and what they change.
pub const PATCH_KEYS: [(&str, &str); 10] = [
    ("description", "Description of the column."),
    ("is_key", "If the column is part of the key: `true` or `false`."),
    ("default_value", "Value of the column in new rows."),
    ("is_filename", "If the column holds file paths: `true` or `false`."),
    ("filename_relative_path", "Folders the file paths are relative to, separated by `;`."),
    ("is_reference", "Referenced table (without `_tables`) and column, as `table;column`."),
    ("lookup", "Columns of the referenced table shown next to the value, separated by `,`."),
    ("lookup_hardcoded", "Hardcoded values to show next to specific values."),
    ("not_empty", "If the column can't be empty: `true` or `false`."),
    ("unused", "If the column is unused by the game: `true` or `false`."),
];

/// `schema.tables`: lists the tables of the schema, with the versions it has definitions for.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ListSchemaTables {

    /// Only tables whose name starts with this are listed.
    #[serde(default)]
    pub prefix: String,
}

/// The tables of the schema.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct SchemaTables {

    /// Versions with a definition, by table name, newest first.
    pub tables: BTreeMap<String, Vec<i32>>,
}

/// `schema.patch_column`: changes how the schema describes a column, with a local patch.
///
/// Local patches are kept apart from the schema, so they survive schema updates. The schema is reloaded
/// after saving the patch, so it applies right away.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PatchColumn {

    /// Name of the table, like `units_tables`.
    pub table_name: String,

    /// Name of the column.
    pub column: String,

    /// Values to set, by patch key. See [`PATCH_KEYS`] for the valid keys.
    pub patch: BTreeMap<String, String>,
}

/// `schema.remove_patches`: removes the local patches of a table, or of one of its columns, and reloads the schema.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RemovePatches {

    /// Name of the table, like `units_tables`.
    pub table_name: String,

    /// If set, only the patches of this column are removed.
    #[serde(default)]
    pub column: Option<String>,
}

/// `schema.update`: downloads the latest schemas, reloads the selected game's one, and rebuilds the dependencies. Runs as a job.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct UpdateSchemas {}

/// `schema.update_from_assembly_kit`: updates the schema of the selected game with the tables of its Assembly Kit, and saves it. Runs as a job.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct UpdateSchemaFromAssemblyKit {

    /// If tables in the game files are skipped. Defaults to the server's setting.
    #[serde(default)]
    pub ignore_game_files: Option<bool>,
}

impl Request for ListSchemaTables {
    const METHOD: &'static str = "schema.tables";
    type Response = SchemaTables;
}

impl Request for PatchColumn {
    const METHOD: &'static str = "schema.patch_column";
    type Response = Done;
}

impl Request for RemovePatches {
    const METHOD: &'static str = "schema.remove_patches";
    type Response = Done;
}

impl Request for UpdateSchemas {
    const METHOD: &'static str = "schema.update";
    type Response = SessionStatus;
    const IS_JOB: bool = true;
}

impl Request for UpdateSchemaFromAssemblyKit {
    const METHOD: &'static str = "schema.update_from_assembly_kit";
    type Response = SessionStatus;
    const IS_JOB: bool = true;
}

/// `schema.raw_definitions`: returns definitions of a table as the schema stores them, to edit them.
///
/// Their fields are the raw on-disk layout, which can differ from the columns rows have
/// (see [`GetTableDefinition`](super::tables::GetTableDefinition) for those).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct GetRawDefinitions {

    /// Name of the table, like `units_tables`.
    pub table_name: String,

    /// If set, only the definition of this version is returned.
    #[serde(default)]
    pub version: Option<i32>,
}

/// Definitions of a table, as the schema stores them.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct RawDefinitions {

    /// The definitions, newest first.
    pub definitions: Vec<Value>,
}

/// `schema.set_definition`: adds a definition to the schema of the selected game, or replaces the one with
/// its version, then saves and reloads the schema.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SetDefinition {

    /// Name of the table, like `units_tables`.
    pub table_name: String,

    /// The definition, in the format `schema.raw_definitions` returns.
    pub definition: Value,
}

/// `schema.delete_definition`: removes a definition from the schema of the selected game, then saves and reloads the schema.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct DeleteDefinition {

    /// Name of the table, like `units_tables`.
    pub table_name: String,

    /// Version of the definition.
    pub version: i32,
}

/// `schema.referencing_columns`: returns the columns of other tables referencing each column of a table.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct GetReferencingColumns {

    /// Name of the table, like `factions_tables`.
    pub table_name: String,

    /// Version of the table's definition. Defaults to the version in the game files, or the newest one.
    #[serde(default)]
    pub version: Option<i32>,
}

/// `schema.patches`: returns the patches applied to the columns of a table.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct GetTablePatches {

    /// Name of the table, like `factions_tables`.
    pub table_name: String,

    /// Version of the table's definition. If the table has no definition with this version, the patches of the table are returned.
    pub version: i32,
}

/// Patches applied to the columns of a table.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct TablePatches {

    /// Patches by column name, as key-value pairs. See the valid keys in `schema.patch_column`.
    pub patches: BTreeMap<String, BTreeMap<String, String>>,
}

/// Columns of other tables referencing the columns of a table.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ReferencingColumns {

    /// For each referenced column, the referencing columns by table name.
    pub columns: BTreeMap<String, BTreeMap<String, Vec<String>>>,
}

/// `schema.import_patches`: adds patches to the schema of the selected game itself, and saves it.
///
/// Unlike [`PatchColumn`], these patches are part of the schema, so a schema update replaces them.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ImportPatches {

    /// Patches, by table name, column name and patch key. See [`PATCH_KEYS`] for the keys.
    pub patches: BTreeMap<String, BTreeMap<String, BTreeMap<String, String>>>,
}

impl Request for GetRawDefinitions {
    const METHOD: &'static str = "schema.raw_definitions";
    type Response = RawDefinitions;
}

impl Request for SetDefinition {
    const METHOD: &'static str = "schema.set_definition";
    type Response = Done;
}

impl Request for DeleteDefinition {
    const METHOD: &'static str = "schema.delete_definition";
    type Response = Done;
}

impl Request for GetReferencingColumns {
    const METHOD: &'static str = "schema.referencing_columns";
    type Response = ReferencingColumns;
}

impl Request for GetTablePatches {
    const METHOD: &'static str = "schema.patches";
    type Response = TablePatches;
}

impl Request for ImportPatches {
    const METHOD: &'static str = "schema.import_patches";
    type Response = Done;
}

/// `schema.missing_definitions`: returns the tables of an open pack with rows that can't be decoded with the schema.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct GetMissingDefinitions {

    /// Key of the pack.
    pub pack: String,
}

/// Tables that need a new definition in the schema.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct MissingDefinitions {

    /// Paths of the tables, sorted.
    pub paths: Vec<String>,
}

impl Request for GetMissingDefinitions {
    const METHOD: &'static str = "schema.missing_definitions";
    type Response = MissingDefinitions;
}
