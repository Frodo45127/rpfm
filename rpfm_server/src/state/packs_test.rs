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
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::channel;
use std::thread::{sleep, spawn};
use std::time::Duration;

use rpfm_lib::files::RFile;
use rpfm_lib::games::pfh_version::PFHVersion;
use tempfile::TempDir;

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

/// Saves a mod pack with one text file in a new temp folder, and opens it lazy-loaded, so its file stays on disk.
fn open_lazy_pack(state: &mut SessionState) -> (TempDir, String) {
    let folder = tempfile::tempdir().unwrap();
    let pack_path = folder.path().join("autosave_test.pack");
    let mut pack = Pack::new_with_name_and_version("autosave_test.pack", PFHVersion::PFH5);
    pack.set_pfh_file_type(PFHFileType::Mod);
    pack.insert(RFile::new_from_vec(b"out(\"autosave\")", FileType::Text, 0, "script/campaign/mod/autosave.lua")).unwrap();
    pack.save(Some(&pack_path), &state.game, &None).unwrap();

    let (pack_key, _) = state.open_packs(&[pack_path], true, false).unwrap();
    (folder, pack_key)
}

/// Settings keeping some autosaves, as the default ones keep none.
fn autosave_settings() -> Settings {
    let mut settings = Settings::default();
    settings.initialize_i32(AUTOSAVE_AMOUNT, 10);
    settings
}

/// Returns how many autosaves of the test pack are in `autosave_path`.
fn autosave_count(autosave_path: &Path) -> usize {
    files_from_subdir(&autosave_path.join("autosave_test.pack"), false).unwrap_or_default().iter()
        .filter(|path| path.extension().is_some_and(|extension| extension == "pack"))
        .count()
}

#[tokio::test]
async fn saves_wait_for_the_running_autosave() {
    let mut state = SessionState::new(Session::new(1, true));
    let (_folder, pack_key) = open_lazy_pack(&mut state);

    let autosave_done = Arc::new(AtomicBool::new(false));
    let thread_done = autosave_done.clone();
    state.autosave_thread = Some(spawn(move || {
        sleep(Duration::from_millis(300));
        thread_done.store(true, Ordering::SeqCst);
    }));

    state.save_pack(&pack_key, None, SaveOptions::default()).unwrap();
    assert!(autosave_done.load(Ordering::SeqCst));
}

#[tokio::test]
async fn autosaves_are_skipped_while_the_previous_one_runs() {
    let mut state = SessionState::new(Session::new(1, true));
    let (folder, pack_key) = open_lazy_pack(&mut state);
    let autosave_path = folder.path().join("autosaves");
    let settings = autosave_settings();

    let (release, released) = channel::<()>();
    state.autosave_thread = Some(spawn(move || { let _ = released.recv(); }));
    state.mark_packs_changed(Some(&pack_key));
    state.autosave_to(&settings, autosave_path.clone());
    assert_eq!(state.changed_packs, BTreeSet::from([pack_key.clone()]));

    release.send(()).unwrap();
    state.wait_for_autosave();
    state.autosave_to(&settings, autosave_path.clone());
    state.wait_for_autosave();
    assert!(state.changed_packs.is_empty());
    assert_eq!(autosave_count(&autosave_path), 1);
}

#[tokio::test]
async fn autosave_and_save_both_work_when_they_overlap() {
    let mut state = SessionState::new(Session::new(1, true));
    let (folder, pack_key) = open_lazy_pack(&mut state);
    let autosave_path = folder.path().join("autosaves");

    // Timestamps are compared in seconds, so the save must land on a later second than the open.
    sleep(Duration::from_millis(1100));
    state.mark_packs_changed(Some(&pack_key));
    state.autosave_to(&autosave_settings(), autosave_path.clone());
    state.save_pack(&pack_key, None, SaveOptions::default()).unwrap();

    assert_eq!(autosave_count(&autosave_path), 1);
}

#[tokio::test]
async fn temporary_packs_are_kept_out_of_the_open_packs() {
    let mut state = SessionState::new(Session::new(1, true));
    let folder = tempfile::tempdir().unwrap();
    let pack_path = folder.path().join("temporary_test.pack");
    let mut pack = Pack::new_with_name_and_version("temporary_test.pack", PFHVersion::PFH5);
    pack.insert(RFile::new_from_vec(b"out(\"temporary\")", FileType::Text, 0, "script/temporary.lua")).unwrap();
    pack.save(Some(&pack_path), &state.game, &None).unwrap();

    let (pack_key, _) = state.open_packs(&[pack_path], false, true).unwrap();

    assert!(state.session_status().packs.is_empty());
    assert!(state.session.pack_names().is_empty());
    assert_eq!(state.pack_details(&pack_key).unwrap().summary.name, "temporary_test.pack");
    assert_eq!(state.files_info(&GetFilesInfo { pack: pack_key.clone(), paths: None }).unwrap().files.len(), 1);

    state.close_pack(&pack_key).unwrap();
    assert!(state.pack_details(&pack_key).is_err());
}
