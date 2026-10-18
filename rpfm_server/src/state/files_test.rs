//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Tests for adding files to packs.

use std::fs::{DirBuilder, File};
use std::time::{Duration, SystemTime};

use rpfm_ipc::settings::Settings;
use rpfm_lib::games::{pfh_file_type::PFHFileType, pfh_version::PFHVersion};
use tempfile::TempDir;

use crate::session::Session;

use super::*;

const PARENT_PACK_NAME: &str = "parent_test.pack";
const IMPORTED_FILE_PATH: &str = "script/campaign/mod/repro.lua";

/// Imports a lua file from a parent pack into a new pack, without opening it, then changes the parent pack's modified time on disk.
///
/// Saving the new pack afterwards, the same save autosaves use, must not need the parent pack anymore.
#[tokio::test]
async fn saving_works_when_the_parent_of_an_imported_file_changes() {
    let folder = tempfile::tempdir().unwrap();
    let mut state = SessionState::new(Session::new(1, true));
    let game = state.game.clone();

    let game_path = folder.path().join("game");
    DirBuilder::new().recursive(true).create(&game_path).unwrap();
    let data_path = game.data_path(&game_path).unwrap();
    DirBuilder::new().recursive(true).create(&data_path).unwrap();

    let parent_path = data_path.join(PARENT_PACK_NAME);
    let mut parent = Pack::new_with_name_and_version(PARENT_PACK_NAME, PFHVersion::PFH5);
    parent.set_pfh_file_type(PFHFileType::Mod);
    parent.insert(RFile::new_from_vec(b"out(\"repro\")", FileType::Text, 0, IMPORTED_FILE_PATH)).unwrap();
    parent.save(Some(&parent_path), &game, &None).unwrap();

    state.dependencies.rebuild(&None, &[PARENT_PACK_NAME.to_owned()], None, &game, &game_path, &folder.path().join("secondary")).unwrap();

    let pack_key = state.new_pack(Some("child_test.pack"), &Settings::default()).unwrap();
    let paths = BTreeMap::from([(DataSource::ParentFiles, vec![ContainerPath::File(IMPORTED_FILE_PATH.to_owned())])]);
    let (added, not_added, _) = state.import_dependencies(&pack_key, &paths).unwrap();
    assert_eq!(added.len(), 1, "not added: {not_added:?}");

    // Timestamps are compared in seconds, so move it far enough to always land on another second.
    let parent_file = File::options().write(true).open(&parent_path).unwrap();
    parent_file.set_modified(SystemTime::now() + Duration::from_secs(10)).unwrap();
    drop(parent_file);

    let child_path = folder.path().join("child_test.pack");
    pack_mut(&mut state.packs, &pack_key).unwrap().save(Some(&child_path), &game, &None).unwrap();
}

/// Opens a lazy-loaded mod pack with one text file, then changes its modified time on disk, so its file can't be read anymore.
///
/// Returns the temp folder holding it, the key of the pack, and the key of a new pack to copy into.
fn open_unreadable_pack(state: &mut SessionState) -> (TempDir, String, String) {
    let folder = tempfile::tempdir().unwrap();
    let source_path = folder.path().join("source_test.pack");
    let mut source = Pack::new_with_name_and_version("source_test.pack", PFHVersion::PFH5);
    source.set_pfh_file_type(PFHFileType::Mod);
    source.insert(RFile::new_from_vec(b"out(\"source\")", FileType::Text, 0, IMPORTED_FILE_PATH)).unwrap();
    source.save(Some(&source_path), &state.game, &None).unwrap();

    let (source_key, _) = state.open_packs(&[source_path.clone()], true).unwrap();
    let target_key = state.new_pack(Some("target_test.pack"), &Settings::default()).unwrap();

    let source_file = File::options().write(true).open(&source_path).unwrap();
    source_file.set_modified(SystemTime::now() + Duration::from_secs(10)).unwrap();
    (folder, source_key, target_key)
}

#[tokio::test]
async fn copying_reports_files_that_cannot_be_read() {
    let mut state = SessionState::new(Session::new(1, true));
    let (_folder, source_key, target_key) = open_unreadable_pack(&mut state);

    let request = CopyFiles { source: FileSource::Pack(source_key), paths: vec![IMPORTED_FILE_PATH.to_owned()], pack: target_key.clone() };
    let result = state.copy_files_to_pack(&request).unwrap();

    assert!(result.added.is_empty());
    assert_eq!(result.not_added, vec![IMPORTED_FILE_PATH.to_owned()]);
    assert!(result.error.is_some_and(|error| error.contains("source_test.pack")));
    assert!(!state.packs[&target_key].has_file(IMPORTED_FILE_PATH));
}

#[tokio::test]
async fn cut_files_that_cannot_be_read_are_reported_and_kept() {
    let mut state = SessionState::new(Session::new(1, true));
    let (_folder, source_key, target_key) = open_unreadable_pack(&mut state);

    let request = PasteFiles { sources: BTreeMap::from([(source_key.clone(), vec![IMPORTED_FILE_PATH.to_owned()])]), cut: true, pack: target_key, folder: String::new() };
    let result = state.paste_files(&request).unwrap();

    assert!(result.added.is_empty());
    assert!(result.deleted.is_empty());
    assert_eq!(result.not_pasted, vec![IMPORTED_FILE_PATH.to_owned()]);
    assert!(result.error.is_some());
    assert!(state.packs[&source_key].has_file(IMPORTED_FILE_PATH));
}
