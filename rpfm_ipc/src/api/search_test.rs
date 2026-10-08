//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Tests for the search methods' params.

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
