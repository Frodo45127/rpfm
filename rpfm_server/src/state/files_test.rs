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
    let (added, not_added) = state.import_dependencies(&pack_key, &paths).unwrap();
    assert_eq!(added.len(), 1, "not added: {not_added:?}");

    // Timestamps are compared in seconds, so move it far enough to always land on another second.
    let parent_file = File::options().write(true).open(&parent_path).unwrap();
    parent_file.set_modified(SystemTime::now() + Duration::from_secs(10)).unwrap();
    drop(parent_file);

    let child_path = folder.path().join("child_test.pack");
    pack_mut(&mut state.packs, &pack_key).unwrap().save(Some(&child_path), &game, &None).unwrap();
}
