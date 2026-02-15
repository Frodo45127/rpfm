//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Tests for the updater module.

use self_update::{Download, Extract, get_target, update::Release};
use tempfile::Builder;

use std::fs::File;

use rpfm_ipc::helpers::APIResponse;
use rpfm_lib::utils::files_from_subdir;

use super::updater::*;

/// Helper to create a fake `Release` with a given version string.
fn fake_release(version: &str) -> Release {
    Release {
        version: version.to_string(),
        ..Default::default()
    }
}

// -----------------------------------------------------------------------
// Version comparison unit tests
// -----------------------------------------------------------------------

#[test]
fn test_newer_major_version_stable() {
    let result = check_updates_rpfm_with("4.0.0", UpdateChannel::Stable, || Ok(fake_release("5.0.0"))).unwrap();
    assert_eq!(result, APIResponse::NewStableUpdate("v5.0.0".to_string()));
}

#[test]
fn test_newer_minor_version_stable() {
    let result = check_updates_rpfm_with("4.0.0", UpdateChannel::Stable, || Ok(fake_release("4.1.0"))).unwrap();
    assert_eq!(result, APIResponse::NewStableUpdate("v4.1.0".to_string()));
}

#[test]
fn test_hotfix_version_stable() {
    let result = check_updates_rpfm_with("4.0.0", UpdateChannel::Stable, || Ok(fake_release("4.0.1"))).unwrap();
    assert_eq!(result, APIResponse::NewUpdateHotfix("v4.0.1".to_string()));
}

#[test]
fn test_same_version_no_update() {
    let result = check_updates_rpfm_with("4.7.0", UpdateChannel::Stable, || Ok(fake_release("4.7.0"))).unwrap();
    assert_eq!(result, APIResponse::NoUpdate);
}

#[test]
fn test_local_newer_no_update() {
    let result = check_updates_rpfm_with("5.0.0", UpdateChannel::Stable, || Ok(fake_release("4.7.0"))).unwrap();
    assert_eq!(result, APIResponse::NoUpdate);
}

#[test]
fn test_local_newer_minor_no_update() {
    let result = check_updates_rpfm_with("4.8.0", UpdateChannel::Stable, || Ok(fake_release("4.7.0"))).unwrap();
    assert_eq!(result, APIResponse::NoUpdate);
}

#[test]
fn test_local_newer_patch_no_update() {
    let result = check_updates_rpfm_with("4.7.2", UpdateChannel::Stable, || Ok(fake_release("4.7.1"))).unwrap();
    assert_eq!(result, APIResponse::NoUpdate);
}

#[test]
fn test_beta_update_on_beta_channel() {
    let result = check_updates_rpfm_with("4.0.0", UpdateChannel::Beta, || Ok(fake_release("4.0.99"))).unwrap();
    assert_eq!(result, APIResponse::NewBetaUpdate("v4.0.99".to_string()));
}

#[test]
fn test_beta_major_update_on_beta_channel() {
    let result = check_updates_rpfm_with("4.0.0", UpdateChannel::Beta, || Ok(fake_release("5.0.0"))).unwrap();
    assert_eq!(result, APIResponse::NewBetaUpdate("v5.0.0".to_string()));
}

#[test]
fn test_beta_user_switches_to_stable() {
    // Current version is a beta (patch >= 99), channel set to stable.
    // Should return the latest stable regardless of version comparison.
    let result = check_updates_rpfm_with("4.7.99", UpdateChannel::Stable, || Ok(fake_release("4.7.0"))).unwrap();
    assert_eq!(result, APIResponse::NewStableUpdate("v4.7.0".to_string()));
}

#[test]
fn test_beta_user_switches_to_stable_higher_beta() {
    // Even if the beta is "higher" than stable, switching to stable should offer the stable version.
    let result = check_updates_rpfm_with("4.8.99", UpdateChannel::Stable, || Ok(fake_release("4.7.0"))).unwrap();
    assert_eq!(result, APIResponse::NewStableUpdate("v4.7.0".to_string()));
}

// -----------------------------------------------------------------------
// Integration tests (require network access)
// -----------------------------------------------------------------------

#[test]
#[ignore]
fn test_last_release_stable_returns_valid() {
    let release = last_release(UpdateChannel::Stable).unwrap();
    assert!(!release.version.is_empty(), "Version should not be empty");
    let patch: i32 = release.version.split('.').nth(2).unwrap().parse().unwrap();
    assert!(patch < 99, "Stable release patch should be < 99, got {}", patch);
}

#[test]
#[ignore]
fn test_last_release_beta_returns_valid() {
    let release = last_release(UpdateChannel::Beta).unwrap();
    assert!(!release.version.is_empty(), "Version should not be empty");
}

#[test]
#[ignore]
fn test_download_and_extract_release() {
    let release = last_release(UpdateChannel::Stable).unwrap();

    // Find an asset for the current platform.
    let target = get_target();
    let asset = release.asset_for(target, None);
    assert!(asset.is_some(), "No asset found for target: {}", target);
    let asset = asset.unwrap();

    // Download to a temp directory.
    let tmp_dir = Builder::new()
        .prefix("rpfm_updater_test")
        .tempdir()
        .unwrap();

    let tmp_zip_path = tmp_dir.path().join(&asset.name);
    let tmp_zip = File::create(&tmp_zip_path).unwrap();

    Download::from_url(&asset.download_url)
        .set_header(reqwest::header::ACCEPT, "application/octet-stream".parse().unwrap())
        .download_to(&tmp_zip)
        .unwrap();

    assert!(tmp_zip_path.exists(), "Downloaded file should exist");
    assert!(tmp_zip_path.metadata().unwrap().len() > 0, "Downloaded file should not be empty");

    // Extract and verify files were created.
    Extract::from_source(&tmp_zip_path)
        .extract_into(tmp_dir.path())
        .unwrap();

    let extracted_files = files_from_subdir(tmp_dir.path(), true).unwrap();

    // Filter out the zip itself.
    let non_zip_files: Vec<_> = extracted_files.iter()
        .filter(|f| f.extension().and_then(|e| e.to_str()) != Some("zip"))
        .collect();

    assert!(!non_zip_files.is_empty(), "Should have extracted at least one file");
}
