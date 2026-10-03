//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Global search and replace operations.

use anyhow::Result;
use serde::Serialize;
use serde_json::{Map, Value};

use std::collections::{BTreeMap, HashSet};

use rpfm_extensions::search::{GlobalSearch, Matches, MatchHolder, SearchSource};
use rpfm_extensions::search::anim_fragment_battle::AnimFragmentBattleMatches;
use rpfm_extensions::search::atlas::AtlasMatches;
use rpfm_extensions::search::portrait_settings::PortraitSettingsMatches;
use rpfm_extensions::search::rigid_model::RigidModelMatches;
use rpfm_extensions::search::schema::SchemaMatches;
use rpfm_extensions::search::table::TableMatches;
use rpfm_extensions::search::text::TextMatches;
use rpfm_extensions::search::unit_variant::UnitVariantMatches;
use rpfm_extensions::search::unknown::UnknownMatches;

use rpfm_ipc::api::ApiError;
use rpfm_ipc::api::files::FileSource;
use rpfm_ipc::api::search::{
    DEFAULT_MATCHES_LIMIT, DEFAULT_SEARCH_FILE_TYPES, search_on_from_file_types, ListSearchMatches, ReplaceSearchMatches, RunSearch, SearchMatch, SearchMatchList,
    SearchReplaced, SearchReport, SearchSummary,
};
use rpfm_ipc::helpers::RFileInfo;

use rpfm_lib::files::{Container, ContainerPath};

use super::{SessionState, loaded_schema};

/// Common access to the matches of a file, whatever its type.
trait FileMatches {

    /// Returns the path of the file.
    fn file_path(&self) -> &str;

    /// Returns where the file is, or `None` for matches outside of files.
    fn file_source(&self) -> Option<FileSource>;

    /// Returns the amount of matches in the file.
    fn match_count(&self) -> usize;

    /// Returns a match, serialized.
    fn match_details(&self, index: usize) -> Value;

    /// Keeps only the matches at the provided indexes.
    fn retain_matches(&mut self, indexes: &HashSet<usize>);
}

/// Implements [`FileMatches`] for match types with the same `path`, `source` and `matches` fields.
///
/// A macro, because the types share their shape but no trait.
macro_rules! impl_file_matches {
    ($($matches_type:ty),*) => {
        $(
            impl FileMatches for $matches_type {
                fn file_path(&self) -> &str {
                    self.path()
                }

                fn file_source(&self) -> Option<FileSource> {
                    Some(file_source(self.source()))
                }

                fn match_count(&self) -> usize {
                    self.matches().len()
                }

                fn match_details(&self, index: usize) -> Value {
                    to_value(&self.matches()[index])
                }

                fn retain_matches(&mut self, indexes: &HashSet<usize>) {
                    retain_indexes(self.matches_mut(), indexes);
                }
            }
        )*
    };
}

impl_file_matches!(AnimFragmentBattleMatches, AtlasMatches, PortraitSettingsMatches, RigidModelMatches, TableMatches, UnitVariantMatches, UnknownMatches);

/// Text matches keep their text apart from their position, so the text is added to the details.
impl FileMatches for TextMatches {
    fn file_path(&self) -> &str {
        self.path()
    }

    fn file_source(&self) -> Option<FileSource> {
        Some(file_source(self.source()))
    }

    fn match_count(&self) -> usize {
        self.matches().len()
    }

    fn match_details(&self, index: usize) -> Value {
        let text_match = &self.matches()[index];
        let mut details = to_value(text_match);
        if let (Value::Object(details), Some(text)) = (&mut details, self.matches_strings().get(*text_match.text_index())) {
            details.insert("text".to_owned(), Value::String(text.clone()));
        }

        details
    }

    fn retain_matches(&mut self, indexes: &HashSet<usize>) {
        retain_indexes(self.matches_mut(), indexes);
    }
}

/// Schema matches are about column names, not files, so they have no path nor source.
impl FileMatches for SchemaMatches {
    fn file_path(&self) -> &str {
        ""
    }

    fn file_source(&self) -> Option<FileSource> {
        None
    }

    fn match_count(&self) -> usize {
        self.matches().len()
    }

    fn match_details(&self, index: usize) -> Value {
        to_value(&self.matches()[index])
    }

    fn retain_matches(&mut self, indexes: &HashSet<usize>) {
        retain_indexes(self.matches_mut(), indexes);
    }
}

/// Position of a match in [`Matches`]: the type of its file, the index of the file in its type, and the index of the match in its file.
#[derive(Debug, Clone, Copy)]
struct MatchRef {
    file_type: &'static str,
    file_index: usize,
    match_index: usize,
}

impl SessionState {

    /// Replaces matches of a global search.
    ///
    /// # Arguments
    ///
    /// * `search` - The search the matches come from.
    /// * `matches` - Matches to replace.
    ///
    /// # Returns
    ///
    /// The search, and the info of the edited files.
    fn global_search_replace(&mut self, mut search: GlobalSearch, matches: &[MatchHolder]) -> Result<(GlobalSearch, Vec<RFileInfo>)> {
        let schema = loaded_schema(&self.schema)?;
        let edited_paths = search.replace(&self.game, schema, &mut self.packs, &mut self.dependencies, matches)?;
        Ok((search, self.files_info_in_all_packs(&edited_paths)))
    }

    /// Searches text, keeping the matches for [`Self::list_search_matches`].
    ///
    /// # Returns
    ///
    /// A summary of the matches.
    pub fn run_search(&mut self, request: &RunSearch) -> Result<SearchSummary> {
        let mut search = GlobalSearch::default();
        search.set_pattern(request.pattern.clone());
        search.set_case_sensitive(request.case_sensitive);
        search.set_use_regex(request.use_regex);
        search.set_sources(request.sources.iter().map(search_source).collect());
        search.set_game_key(self.game.key().to_owned());

        let file_types = match request.file_types {
            Some(ref file_types) => file_types.iter().map(String::as_str).collect::<Vec<_>>(),
            None => DEFAULT_SEARCH_FILE_TYPES.to_vec(),
        };
        search.set_search_on(search_on_from_file_types(&file_types).map_err(ApiError::InvalidParams)?);

        let schema = loaded_schema(&self.schema)?;
        search.search(&self.game, schema, &mut self.packs, &mut self.dependencies, &[]);

        let summary = summarize(search.matches());
        self.search = Some(search);
        Ok(summary)
    }

    /// Returns a page of the matches of the last search matching the request.
    ///
    /// # Errors
    ///
    /// Fails if there hasn't been a search yet.
    pub fn list_search_matches(&self, request: &ListSearchMatches) -> Result<SearchMatchList> {
        let matches = self.search.as_ref().ok_or(ApiError::SearchNotRun)?.matches();
        let files = files_by_type(matches);

        let mut page = vec![];
        let mut total = 0;
        let limit = request.limit.unwrap_or(DEFAULT_MATCHES_LIMIT);
        for (id, match_ref) in match_refs(&files).into_iter().enumerate() {
            let file = files[match_ref.file_type][match_ref.file_index];
            let type_matches = request.file_types.as_ref().is_none_or(|file_types| file_types.iter().any(|file_type| file_type == match_ref.file_type));
            if !type_matches || !file.file_path().starts_with(&request.path_prefix) {
                continue;
            }

            if total >= request.offset && page.len() < limit {
                page.push(SearchMatch {
                    id,
                    source: file.file_source(),
                    path: file.file_path().to_owned(),
                    file_type: match_ref.file_type.to_owned(),
                    details: file.match_details(match_ref.match_index),
                });
            }

            total += 1;
        }

        Ok(SearchMatchList { matches: page, total })
    }

    /// Replaces matches of the last search, and searches the edited files again.
    ///
    /// # Errors
    ///
    /// Fails if there hasn't been a search yet, if an ID doesn't exist, or if replacing fails.
    pub fn replace_search_matches(&mut self, request: &ReplaceSearchMatches) -> Result<SearchReplaced> {
        let mut search = self.search.take().ok_or(ApiError::SearchNotRun)?;
        search.set_replace_text(request.replace_text.clone());

        // The search replaces matches in any open pack with the path of their file, so only matches from open packs are passed to it.
        let all_ids;
        let ids = match request.matches {
            Some(ref ids) => ids,
            None => {
                all_ids = (0..match_refs(&files_by_type(search.matches())).len()).collect::<Vec<_>>();
                &all_ids
            }
        };

        let result = match selected_match_holders(search.matches(), ids) {
            Ok(holders) => self.global_search_replace(search.clone(), &holders),
            Err(error) => Err(error.into()),
        };

        // Keep the previous search if replacing failed, so its matches can still be listed.
        let (mut replaced, files_info) = match result {
            Ok(result) => result,
            Err(error) => {
                self.search = Some(search);
                return Err(error);
            }
        };

        let edited = files_info.iter().map(|info| info.path().to_owned()).collect::<Vec<_>>();
        let edited_paths = edited.iter().map(|path| ContainerPath::File(path.clone())).collect::<Vec<_>>();
        let schema = loaded_schema(&self.schema)?;
        replaced.search(&self.game, schema, &mut self.packs, &mut self.dependencies, &edited_paths);

        let summary = summarize(replaced.matches());
        self.search = Some(replaced);
        Ok(SearchReplaced { edited, summary })
    }

    /// Returns the last search with all its matches, and the ID of the first match of each file.
    ///
    /// # Errors
    ///
    /// Fails if there is no search yet.
    pub fn search_report(&self) -> Result<SearchReport> {
        let search = self.search.as_ref().ok_or(ApiError::SearchNotRun)?;
        Ok(SearchReport { search: search.clone(), first_ids: first_ids(&files_by_type(search.matches())) })
    }

    /// Returns the info of the files at the provided paths, from every open pack.
    fn files_info_in_all_packs(&self, paths: &[ContainerPath]) -> Vec<RFileInfo> {
        paths.iter()
            .flat_map(|path| self.packs.values().flat_map(move |pack| pack.files_by_path(path, false)))
            .map(RFileInfo::from)
            .collect()
    }
}

/// Returns the matches of each file, by type of file, in a fixed order.
fn files_by_type(matches: &Matches) -> BTreeMap<&'static str, Vec<&dyn FileMatches>> {
    fn as_dyn<T: FileMatches>(files: &[T]) -> Vec<&dyn FileMatches> {
        files.iter().map(|file| file as &dyn FileMatches).collect()
    }

    BTreeMap::from([
        ("Anim", as_dyn(matches.anim())),
        ("AnimFragmentBattle", as_dyn(matches.anim_fragment_battle())),
        ("AnimPack", as_dyn(matches.anim_pack())),
        ("AnimsTable", as_dyn(matches.anims_table())),
        ("Atlas", as_dyn(matches.atlas())),
        ("Audio", as_dyn(matches.audio())),
        ("BMD", as_dyn(matches.bmd())),
        ("DB", as_dyn(matches.db())),
        ("ESF", as_dyn(matches.esf())),
        ("GroupFormations", as_dyn(matches.group_formations())),
        ("Image", as_dyn(matches.image())),
        ("Loc", as_dyn(matches.loc())),
        ("MatchedCombat", as_dyn(matches.matched_combat())),
        ("Pack", as_dyn(matches.pack())),
        ("PortraitSettings", as_dyn(matches.portrait_settings())),
        ("RigidModel", as_dyn(matches.rigid_model())),
        ("Schema", vec![matches.schema() as &dyn FileMatches]),
        ("SoundBank", as_dyn(matches.sound_bank())),
        ("Text", as_dyn(matches.text())),
        ("UIC", as_dyn(matches.uic())),
        ("UnitVariant", as_dyn(matches.unit_variant())),
        ("Unknown", as_dyn(matches.unknown())),
        ("Video", as_dyn(matches.video())),
    ])
}

/// Returns the position of every match, in the order their IDs follow.
fn match_refs(files: &BTreeMap<&'static str, Vec<&dyn FileMatches>>) -> Vec<MatchRef> {
    files.iter()
        .flat_map(|(file_type, files)| files.iter().enumerate()
            .flat_map(move |(file_index, file)| (0..file.match_count())
                .map(move |match_index| MatchRef { file_type, file_index, match_index })))
        .collect()
}

/// Returns the ID of the first match of each file, by type of file, numbering the matches like [`match_refs`].
fn first_ids(files: &BTreeMap<&'static str, Vec<&dyn FileMatches>>) -> BTreeMap<String, Vec<usize>> {
    let mut next_id = 0;
    files.iter()
        .map(|(file_type, files)| {
            let ids = files.iter()
                .map(|file| {
                    let id = next_id;
                    next_id += file.match_count();
                    id
                })
                .collect();

            ((*file_type).to_owned(), ids)
        })
        .collect()
}

/// Returns the matches with the provided IDs, grouped by file as the search expects them to replace them.
///
/// # Errors
///
/// Fails if any ID doesn't exist.
fn selected_match_holders(matches: &Matches, ids: &[usize]) -> Result<Vec<MatchHolder>, ApiError> {
    let refs = match_refs(&files_by_type(matches));
    let mut selected: BTreeMap<(&str, usize), HashSet<usize>> = BTreeMap::new();
    for id in ids {
        let match_ref = refs.get(*id).ok_or_else(|| ApiError::InvalidParams(format!("There is no match with ID {id}.")))?;
        selected.entry((match_ref.file_type, match_ref.file_index)).or_default().insert(match_ref.match_index);
    }

    let mut holders = vec![];
    holders.extend(selected_files(matches.anim_fragment_battle(), "AnimFragmentBattle", &selected).map(MatchHolder::AnimFragmentBattle));
    holders.extend(selected_files(matches.atlas(), "Atlas", &selected).map(MatchHolder::Atlas));
    holders.extend(selected_files(matches.db(), "DB", &selected).map(MatchHolder::Db));
    holders.extend(selected_files(matches.loc(), "Loc", &selected).map(MatchHolder::Loc));
    holders.extend(selected_files(matches.portrait_settings(), "PortraitSettings", &selected).map(MatchHolder::PortraitSettings));
    holders.extend(selected_files(matches.rigid_model(), "RigidModel", &selected).map(MatchHolder::RigidModel));
    holders.extend(selected_files(matches.text(), "Text", &selected).map(MatchHolder::Text));
    holders.extend(selected_files(matches.unit_variant(), "UnitVariant", &selected).map(MatchHolder::UnitVariant));
    Ok(holders)
}

/// Returns copies of the files of a type in open packs with selected matches, keeping only those matches.
fn selected_files<'a, T: FileMatches + Clone>(files: &'a [T], file_type: &'a str, selected: &'a BTreeMap<(&str, usize), HashSet<usize>>) -> impl Iterator<Item = T> + 'a {
    files.iter().enumerate().filter_map(move |(file_index, file)| {
        if !matches!(file.file_source(), Some(FileSource::Pack(_))) {
            return None;
        }

        let indexes = selected.get(&(file_type, file_index))?;
        let mut file = file.clone();
        file.retain_matches(indexes);
        Some(file)
    })
}

/// Counts the matches of a search, by type of file.
fn summarize(matches: &Matches) -> SearchSummary {
    let mut summary = SearchSummary::default();
    for (file_type, files) in files_by_type(matches) {
        let count = files.iter().map(|file| file.match_count()).sum::<usize>();
        if count > 0 {
            summary.total += count;
            summary.files += files.iter().filter(|file| file.match_count() > 0 && !file.file_path().is_empty()).count();
            summary.by_type.insert(file_type.to_owned(), count);
        }
    }

    summary
}

/// Returns the search source of a file source.
fn search_source(source: &FileSource) -> SearchSource {
    match source {
        FileSource::Pack(pack_key) => SearchSource::Pack(pack_key.clone()),
        FileSource::GameFiles => SearchSource::GameFiles,
        FileSource::ParentFiles => SearchSource::ParentFiles,
        FileSource::AssemblyKit => SearchSource::AssKitFiles,
    }
}

/// Returns the file source of a search source.
fn file_source(source: &SearchSource) -> FileSource {
    match source {
        SearchSource::Pack(pack_key) => FileSource::Pack(pack_key.clone()),
        SearchSource::GameFiles => FileSource::GameFiles,
        SearchSource::ParentFiles => FileSource::ParentFiles,
        SearchSource::AssKitFiles => FileSource::AssemblyKit,
    }
}

/// Keeps only the items at the provided indexes.
fn retain_indexes<T>(items: &mut Vec<T>, indexes: &HashSet<usize>) {
    let mut index = 0;
    items.retain(|_| {
        index += 1;
        indexes.contains(&(index - 1))
    });
}

/// Serializes a match, or returns an empty object if it can't be serialized.
fn to_value(value: &impl Serialize) -> Value {
    serde_json::to_value(value).unwrap_or_else(|_| Value::Object(Map::new()))
}

//-------------------------------------------------------------------------------//
//                                   Tests
//-------------------------------------------------------------------------------//

#[cfg(test)]
mod tests {
    use rpfm_ipc::api::search::search_file_types;

    use super::*;

    #[test]
    fn retain_keeps_only_the_selected_indexes() {
        let mut items = vec!["a", "b", "c", "d"];
        retain_indexes(&mut items, &HashSet::from([0, 2, 9]));
        assert_eq!(items, vec!["a", "c"]);
    }

    /// Matches of a file for tests: only their amount matters.
    struct FakeMatches(usize);

    impl FileMatches for FakeMatches {
        fn file_path(&self) -> &str { "" }
        fn file_source(&self) -> Option<FileSource> { None }
        fn match_count(&self) -> usize { self.0 }
        fn match_details(&self, _index: usize) -> Value { Value::Null }
        fn retain_matches(&mut self, _indexes: &HashSet<usize>) {}
    }

    #[test]
    fn first_ids_number_matches_like_match_refs() {
        let (db, text) = ([FakeMatches(2), FakeMatches(0), FakeMatches(3)], [FakeMatches(1)]);
        let files: BTreeMap<&'static str, Vec<&dyn FileMatches>> = BTreeMap::from([
            ("DB", db.iter().map(|file| file as &dyn FileMatches).collect()),
            ("Text", text.iter().map(|file| file as &dyn FileMatches).collect()),
        ]);

        let first_ids = first_ids(&files);

        assert_eq!(first_ids["DB"], vec![0, 2, 2]);
        assert_eq!(first_ids["Text"], vec![5]);
        for (id, match_ref) in match_refs(&files).iter().enumerate() {
            assert_eq!(first_ids[match_ref.file_type][match_ref.file_index] + match_ref.match_index, id);
        }
    }

    #[test]
    fn every_search_file_type_has_matches_to_list() {
        let matches = Matches::default();
        let files = files_by_type(&matches);

        assert_eq!(files.keys().copied().collect::<Vec<_>>(), search_file_types().collect::<Vec<_>>());
    }
}
