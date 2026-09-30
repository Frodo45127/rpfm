//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Methods to follow references between tables, and between tables and Loc files.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::Request;
use super::files::FileSource;

/// Default amount of usages returned by [`FindUsages`].
pub const DEFAULT_USAGES_LIMIT: usize = 200;

/// A row of a table in one of the sources.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct RowLocation {

    /// Where the table is.
    pub source: FileSource,

    /// Path of the table in its source.
    pub path: String,

    /// Index of the column the value is in.
    pub column_index: usize,

    /// Index of the row.
    pub row_index: usize,
}

/// `references.definition`: finds the row where a value of a referenced table is defined.
///
/// Searches the open packs (starting with `pack`), then the parent packs, the game files and the Assembly Kit tables.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct FindDefinition {

    /// Referenced table, like `factions` or `factions_tables`.
    pub table: String,

    /// Referenced column. If it's a localised column, the table's key is searched instead.
    pub column: String,

    /// Value to find.
    pub value: String,

    /// Key of the pack to search first.
    #[serde(default)]
    pub pack: Option<String>,
}

/// `references.usages`: finds the rows of other tables referencing a value of a table.
///
/// The referencing columns are taken from the schema. Searches the open packs, the parent packs and the game files.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct FindUsages {

    /// Table the value is from, like `factions_tables`.
    pub table_name: String,

    /// Column the value is from, like `key`.
    pub column: String,

    /// Value to find usages of.
    pub value: String,

    /// If set, only this open pack is searched, instead of all of them.
    #[serde(default)]
    pub pack: Option<String>,

    /// Amount of usages to skip.
    #[serde(default)]
    pub offset: usize,

    /// Maximum amount of usages to return. Defaults to [`DEFAULT_USAGES_LIMIT`].
    #[serde(default)]
    pub limit: Option<usize>,
}

/// A page of the usages of a value.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct Usages {

    /// The usages in the page.
    pub usages: Vec<Usage>,

    /// Amount of usages, in all pages.
    pub total: usize,
}

/// A row referencing a value.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct Usage {

    /// Where the row is.
    #[serde(flatten)]
    pub location: RowLocation,

    /// Name of the column referencing the value.
    pub column: String,
}

/// `references.loc`: finds the row of a Loc file with a key.
///
/// Searches the open packs (starting with `pack`), then the parent packs and the game files.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct FindLoc {

    /// Key to find, like `factions_screen_name_wh_main_emp_empire`.
    pub key: String,

    /// Key of the pack to search first.
    #[serde(default)]
    pub pack: Option<String>,
}

/// `references.loc_source`: returns the table, column and row a loc key belongs to.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct GetLocSource {

    /// The loc key, like `factions_screen_name_wh_main_emp_empire`.
    pub key: String,
}

/// Result of looking for the source of a loc key.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct LocSourceLookup {

    /// Table, column and row the key belongs to, if they were found.
    pub source: Option<LocSource>,
}

/// Table, column and row a loc key belongs to.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct LocSource {

    /// Name of the table, without the `_tables` suffix.
    pub table: String,

    /// Name of the localised column.
    pub column: String,

    /// Values of the key columns of the row.
    pub key_values: Vec<String>,
}

impl Request for FindDefinition {
    const METHOD: &'static str = "references.definition";
    type Response = RowLocation;
}

impl Request for FindUsages {
    const METHOD: &'static str = "references.usages";
    type Response = Usages;
}

impl Request for FindLoc {
    const METHOD: &'static str = "references.loc";
    type Response = RowLocation;
}

impl Request for GetLocSource {
    const METHOD: &'static str = "references.loc_source";
    type Response = LocSourceLookup;
}
