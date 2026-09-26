//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Tests for writing the settings file safely, and recovering it when it's broken.

use tempfile::TempDir;

use std::fs;
use std::path::{Path, PathBuf};

use super::settings::*;

const KEY: &str = "test_key";

/// Settings with a recognizable value, that never write to the real config folder.
fn sample(value: &str) -> Settings {
    let mut settings = Settings::default();
    settings.set_block_write(true);
    settings.set_string(KEY, value).unwrap();
    settings
}

fn settings_path(folder: &Path) -> PathBuf {
    folder.join("settings.json")
}

/// Writing leaves the new settings in place, and no temporary file behind.
#[test]
fn write_to_replaces_the_file() {
    let folder = TempDir::new().unwrap();
    let path = settings_path(folder.path());
    sample("old").write_to(&path).unwrap();

    sample("new").write_to(&path).unwrap();

    assert_eq!(Settings::read_from(&path).unwrap().string(KEY), "new");
    assert!(!path.with_extension("json.tmp").exists());
}

/// A good file is loaded, and becomes the backup.
#[test]
fn load_good_file_refreshes_backup() {
    let folder = TempDir::new().unwrap();
    let path = settings_path(folder.path());
    sample("good").write_to(&path).unwrap();

    let settings = Settings::load_or_recover(&path);

    assert_eq!(settings.string(KEY), "good");
    assert_eq!(Settings::read_from(&backup_path(&path)).unwrap().string(KEY), "good");
}

/// An empty file, as left by a power loss, is recovered from the backup, and the backup is kept.
#[test]
fn load_empty_file_restores_backup() {
    let folder = TempDir::new().unwrap();
    let path = settings_path(folder.path());
    sample("backup").write_to(&backup_path(&path)).unwrap();
    fs::write(&path, b"").unwrap();

    let settings = Settings::load_or_recover(&path);

    assert_eq!(settings.string(KEY), "backup");
    assert_eq!(Settings::read_from(&backup_path(&path)).unwrap().string(KEY), "backup");
}

/// A broken file with no usable backup falls back to defaults, keeping its contents in the backup for inspection.
#[test]
fn load_broken_file_without_backup_keeps_it() {
    let folder = TempDir::new().unwrap();
    let path = settings_path(folder.path());
    fs::write(&path, b"{ not json").unwrap();

    let settings = Settings::load_or_recover(&path);

    assert_eq!(settings.string(KEY), "");
    assert_eq!(fs::read(backup_path(&path)).unwrap(), b"{ not json");
}

/// An empty file with no backup falls back to defaults, without creating an empty backup.
#[test]
fn load_empty_file_without_backup_creates_no_backup() {
    let folder = TempDir::new().unwrap();
    let path = settings_path(folder.path());
    fs::write(&path, b"").unwrap();

    let settings = Settings::load_or_recover(&path);

    assert_eq!(settings.string(KEY), "");
    assert!(!backup_path(&path).exists());
}
