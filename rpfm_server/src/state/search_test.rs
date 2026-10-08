//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Tests for listing and filtering search matches.

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
