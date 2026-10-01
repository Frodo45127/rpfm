//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Methods to search text across the open packs, the dependencies and the schema, and replace it.
//!
//! The server keeps the matches of the last search, so clients get a summary when running it,
//! read the matches they need with [`ListSearchMatches`], and replace them by ID.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use std::collections::BTreeMap;

use rpfm_extensions::search::GlobalSearch;

use super::Request;
use super::files::FileSource;

/// Default amount of matches returned by [`ListSearchMatches`].
pub const DEFAULT_MATCHES_LIMIT: usize = 100;

/// File types searched by [`RunSearch`] when it doesn't set them.
pub const DEFAULT_SEARCH_FILE_TYPES: [&str; 3] = ["db", "loc", "text"];

/// `search.run`: searches text, keeping the matches for [`ListSearchMatches`]. Runs as a job.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct RunSearch {

    /// Text to search, or a regular expression if `use_regex` is set.
    pub pattern: String,

    /// If the search is case sensitive.
    #[serde(default)]
    pub case_sensitive: bool,

    /// If the pattern is a regular expression.
    #[serde(default)]
    pub use_regex: bool,

    /// Where to search: open packs, the game files, the parent packs, or the Assembly Kit tables.
    pub sources: Vec<FileSource>,

    /// Types of files to search: `db`, `loc`, `text`, `schema` (table column names), `atlas`,
    /// `portrait_settings`, `rigid_model`, `unit_variant`, `anim_fragment_battle`, or binary types like
    /// `image`, `audio` or `unknown`. Defaults to `db`, `loc` and `text`.
    #[serde(default)]
    pub file_types: Option<Vec<String>>,
}

/// Summary of the matches of a search.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct SearchSummary {

    /// Amount of matches.
    pub total: usize,

    /// Amount of files with matches.
    pub files: usize,

    /// Amount of matches in each type of file.
    pub by_type: BTreeMap<String, usize>,
}

/// `search.matches`: returns matches of the last search, optionally filtered.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ListSearchMatches {

    /// If set, only matches in these types of files are returned.
    #[serde(default)]
    pub file_types: Option<Vec<String>>,

    /// Only matches in files whose path starts with this are returned.
    #[serde(default)]
    pub path_prefix: String,

    /// Amount of matching matches to skip.
    #[serde(default)]
    pub offset: usize,

    /// Maximum amount of matches to return. Defaults to [`DEFAULT_MATCHES_LIMIT`].
    #[serde(default)]
    pub limit: Option<usize>,
}

/// A page of the matches of a search.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct SearchMatchList {

    /// The matches in the page.
    pub matches: Vec<SearchMatch>,

    /// Amount of matches matching the request, in all pages.
    pub total: usize,
}

/// A match of a search.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct SearchMatch {

    /// ID of the match, used to replace it. IDs change after every search and replace.
    pub id: usize,

    /// Where the file with the match is. Not set for matches in the schema.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<FileSource>,

    /// Path of the file with the match. Empty for matches in the schema.
    pub path: String,

    /// Type of the file with the match, like `db` or `text`.
    pub file_type: String,

    /// Where the match is in its file, and its text. The fields depend on the type of the file:
    /// tables have `column_name`, `row_number`, `start`, `end` and `text`, text files have `row`,
    /// `start`, `end` and `text`, and so on.
    pub details: Value,
}

/// `search.replace`: replaces matches of the last search, and searches the edited files again.
///
/// Only matches in open packs can be replaced, and only in tables, text files, atlases, portrait
/// settings, rigid models, unit variants and anim fragment battles.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ReplaceSearchMatches {

    /// Text to replace the matches with.
    pub replace_text: String,

    /// IDs of the matches to replace. If not set, all the matches are replaced.
    #[serde(default)]
    pub matches: Option<Vec<usize>>,
}

/// Result of replacing matches.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct SearchReplaced {

    /// Paths of the edited files.
    pub edited: Vec<String>,

    /// Summary of the matches left after searching the edited files again.
    pub summary: SearchSummary,
}

/// `search.report`: returns the last search with all its matches, as the search returns them.
///
/// Meant for clients showing the matches with all their details. Others should use `search.matches`.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct GetSearchReport {}

/// The last search with all its matches.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SearchReport {

    /// The search, with its matches.
    pub search: GlobalSearch,

    /// ID of the first match of each file, by type of file, in the order of the files in the search.
    /// The other matches of a file follow it, so its match number `n` has the ID `first + n`.
    pub first_ids: BTreeMap<String, Vec<usize>>,
}

impl Request for GetSearchReport {
    const METHOD: &'static str = "search.report";
    type Response = SearchReport;
}

impl Request for RunSearch {
    const METHOD: &'static str = "search.run";
    type Response = SearchSummary;
    const IS_JOB: bool = true;
}

impl Request for ListSearchMatches {
    const METHOD: &'static str = "search.matches";
    type Response = SearchMatchList;
}

impl Request for ReplaceSearchMatches {
    const METHOD: &'static str = "search.replace";
    type Response = SearchReplaced;
}
