//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Tests for the filter bar predicate parser.

use super::*;

//---------------------------------------------------------------------------//
//                                  Tests
//---------------------------------------------------------------------------//

// Display order: alphabetical by chance, but logical order is different — entry 0
// shows "Cost" but its logical model column is 1; entry 1 shows "Faction"/logical 2;
// entry 2 shows "Name"/logical 0. This mirrors what fields_processed_sorted does in
// practice (keys first, then alphabetical).
fn cols() -> Vec<String> {
    vec!["Cost".into(), "Faction".into(), "Name".into()]
}
fn logical() -> Vec<i32> {
    vec![1, 2, 0]
}

#[test]
fn parses_bare_value() {
    let s = parse_predicate("ork", &cols(), &logical());
    assert_eq!(s.pattern, "ork");
    assert_eq!(s.column_index, -1);
    assert!(!s.not);
    assert!(s.regex);
}

#[test]
fn parses_column_value_returns_logical_index() {
    // "Faction" is at sorted position 1 but its logical column is 2.
    let s = parse_predicate("faction:ork", &cols(), &logical());
    assert_eq!(s.pattern, "ork");
    assert_eq!(s.column_index, 2, "expected logical index 2 for Faction, got sorted index");
}

#[test]
fn parses_negation_and_flags_with_logical_index() {
    let s = parse_predicate("!faction:ork /s /n @2", &cols(), &logical());
    assert!(s.not);
    assert_eq!(s.column_index, 2);
    assert_eq!(s.pattern, "ork");
    assert!(s.case_sensitive);
    assert!(!s.regex);
    assert_eq!(s.group, 2);
}

#[test]
fn unknown_column_falls_back_to_literal_value() {
    let s = parse_predicate("http://example.com", &cols(), &logical());
    assert_eq!(s.column_index, -1);
    assert_eq!(s.pattern, "http://example.com");
}
