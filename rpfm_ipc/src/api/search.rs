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

use rpfm_extensions::search::{GlobalSearch, SearchOn};

use super::Request;
use super::files::FileSource;

/// Default amount of matches returned by [`ListSearchMatches`].
pub const DEFAULT_MATCHES_LIMIT: usize = 100;

/// File types searched by [`RunSearch`] when it doesn't set them.
pub const DEFAULT_SEARCH_FILE_TYPES: [&str; 3] = ["DB", "Loc", "Text"];

/// Getter and setter of the flag of a type of file in a [`SearchOn`].
type SearchFlag = (&'static str, fn(&SearchOn) -> &bool, fn(&mut SearchOn, bool) -> &mut SearchOn);

/// Types of files a search can look into, by the name the API gives them, with their flags.
///
/// The names are the ones of their file type, like in `files.list`, plus `Schema` for the column names of the schema.
const SEARCH_FLAGS: [SearchFlag; 23] = [
    ("Anim", SearchOn::anim, SearchOn::set_anim),
    ("AnimFragmentBattle", SearchOn::anim_fragment_battle, SearchOn::set_anim_fragment_battle),
    ("AnimPack", SearchOn::anim_pack, SearchOn::set_anim_pack),
    ("AnimsTable", SearchOn::anims_table, SearchOn::set_anims_table),
    ("Atlas", SearchOn::atlas, SearchOn::set_atlas),
    ("Audio", SearchOn::audio, SearchOn::set_audio),
    ("BMD", SearchOn::bmd, SearchOn::set_bmd),
    ("DB", SearchOn::db, SearchOn::set_db),
    ("ESF", SearchOn::esf, SearchOn::set_esf),
    ("GroupFormations", SearchOn::group_formations, SearchOn::set_group_formations),
    ("Image", SearchOn::image, SearchOn::set_image),
    ("Loc", SearchOn::loc, SearchOn::set_loc),
    ("MatchedCombat", SearchOn::matched_combat, SearchOn::set_matched_combat),
    ("Pack", SearchOn::pack, SearchOn::set_pack),
    ("PortraitSettings", SearchOn::portrait_settings, SearchOn::set_portrait_settings),
    ("RigidModel", SearchOn::rigid_model, SearchOn::set_rigid_model),
    ("Schema", SearchOn::schema, SearchOn::set_schema),
    ("SoundBank", SearchOn::sound_bank, SearchOn::set_sound_bank),
    ("Text", SearchOn::text, SearchOn::set_text),
    ("UIC", SearchOn::uic, SearchOn::set_uic),
    ("UnitVariant", SearchOn::unit_variant, SearchOn::set_unit_variant),
    ("Unknown", SearchOn::unknown, SearchOn::set_unknown),
    ("Video", SearchOn::video, SearchOn::set_video),
];

/// `search.run`: searches text, keeping the matches for `search.matches`. Runs as a job.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
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

    /// Types of files to search, named like in `files.list`: `DB`, `Loc`, `Text`, `Atlas`, `PortraitSettings`,
    /// `RigidModel`, `UnitVariant`, `AnimFragmentBattle`, binary types like `Image`, `Audio` or `Unknown`,
    /// or `Schema` for the column names of the schema. Defaults to `DB`, `Loc` and `Text`.
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
#[serde(deny_unknown_fields)]
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

    /// Maximum amount of matches to return. Defaults to 100.
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

    /// Type of the file with the match, like `DB` or `Text`.
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
#[serde(deny_unknown_fields)]
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
#[serde(deny_unknown_fields)]
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

//-------------------------------------------------------------------------------//
//                                 Functions
//-------------------------------------------------------------------------------//

/// Returns the names of the types of files a search can look into.
pub fn search_file_types() -> impl Iterator<Item = &'static str> {
    SEARCH_FLAGS.iter().map(|(name, _, _)| *name)
}

/// Returns what a search looks into, from the names of the types of files.
///
/// # Errors
///
/// If any name isn't a type of file a search can look into, with the valid ones.
pub fn search_on_from_file_types<S: AsRef<str>>(file_types: &[S]) -> Result<SearchOn, String> {
    let mut search_on = SearchOn::default();
    for file_type in file_types {
        let file_type = file_type.as_ref();
        let (_, _, set) = SEARCH_FLAGS.iter()
            .find(|(name, _, _)| *name == file_type)
            .ok_or_else(|| format!("Unknown file type to search: {file_type}. Valid ones: {}.", search_file_types().collect::<Vec<_>>().join(", ")))?;

        set(&mut search_on, true);
    }

    Ok(search_on)
}

/// Returns the names of the types of files a search looks into.
pub fn file_types_from_search_on(search_on: &SearchOn) -> Vec<String> {
    SEARCH_FLAGS.iter()
        .filter(|(_, get, _)| *get(search_on))
        .map(|(name, _, _)| (*name).to_owned())
        .collect()
}

//-------------------------------------------------------------------------------//
//                                   Tests
//-------------------------------------------------------------------------------//

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn search_file_types_round_trip_through_search_on() {
        let search_on = search_on_from_file_types(&["DB", "Schema"]).unwrap();

        assert!(*search_on.db() && *search_on.schema() && !*search_on.loc());
        assert_eq!(file_types_from_search_on(&search_on), vec!["DB", "Schema"]);
    }

    #[test]
    fn unknown_search_file_types_are_rejected() {
        let error = search_on_from_file_types(&["db"]).unwrap_err();

        assert!(error.starts_with("Unknown file type to search: db."));
    }
}
