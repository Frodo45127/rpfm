//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Tests for creating packs and tracking their changes.

use std::collections::BTreeSet;

use crate::session::Session;

use super::*;

#[tokio::test]
async fn pack_keys_are_new_uuids() {
    let mut state = SessionState::new(Session::new(1, true));
    let settings = Settings::default();

    let first = state.new_pack(None, &settings).unwrap();
    state.close_pack(&first).unwrap();
    let second = state.new_pack(None, &settings).unwrap();

    assert!(Uuid::parse_str(&first).is_ok());
    assert_ne!(first, second);
}

#[tokio::test]
async fn new_packs_get_free_names_and_no_path() {
    let mut state = SessionState::new(Session::new(1, true));
    let settings = Settings::default();

    let named = state.new_pack(Some("my_mod.pack"), &settings).unwrap();
    let first = state.new_pack(None, &settings).unwrap();
    let second = state.new_pack(None, &settings).unwrap();

    assert_eq!(state.pack_summary(&named).unwrap().name, "my_mod.pack");
    assert_eq!(state.pack_summary(&first).unwrap().name, "new_pack.pack");
    assert_eq!(state.pack_summary(&second).unwrap().name, "new_pack_2.pack");
    assert_eq!(state.pack_summary(&second).unwrap().path, None);
}

#[tokio::test]
async fn new_pack_names_must_be_pack_file_names() {
    let mut state = SessionState::new(Session::new(1, true));
    let settings = Settings::default();

    assert!(state.new_pack(Some("my_mod"), &settings).is_err());
    assert!(state.new_pack(Some("folder/my_mod.pack"), &settings).is_err());
}

#[tokio::test]
async fn changes_mark_their_pack_or_all_open_ones() {
    let mut state = SessionState::new(Session::new(1, true));
    let settings = Settings::default();
    let first = state.new_pack(None, &settings).unwrap();
    let second = state.new_pack(None, &settings).unwrap();

    state.mark_packs_changed(Some(&first));
    state.mark_packs_changed(Some("not_open"));
    assert_eq!(state.changed_packs, BTreeSet::from([first.clone()]));

    state.mark_packs_changed(None);
    assert_eq!(state.changed_packs, BTreeSet::from([first.clone(), second.clone()]));

    state.close_pack(&first).unwrap();
    assert_eq!(state.changed_packs, BTreeSet::from([second]));
}
