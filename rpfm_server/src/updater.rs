//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Self-update checks against GitHub releases.
//!
//! On Linux, in-app updates are typically disabled (the distro / Flatpak
//! manages updates instead). On Windows, the standalone server can pull a
//! release zip from GitHub, extract it next to the running binary, replace
//! the executable atomically, and open the changelog so the user actually
//! reads it.
//!
//! The two relevant pieces of public surface are:
//!
//! - [`check_updates_rpfm`] — non-destructive: returns an [`APIResponse`]
//!   describing whether an update is available, what kind, and what version.
//! - [`update_main_program`] — performs the actual download / extract /
//!   replace, then opens the changelog.
//!
//! Both have `*_with` variants that take an explicit release-fetching
//! closure; those are the actual implementations and the ones the unit tests
//! exercise.

use anyhow::{anyhow, Result};
use itertools::Itertools;
use self_update::{backends::github::ReleaseList, Download, get_target, cargo_crate_version, Move, update::Release};
use tempfile::Builder;
use zip::ZipArchive;

use std::env::current_exe;
use std::fmt::Display;
use std::fs::{self, DirBuilder, File};
use std::io;
use std::path::Path;

use rpfm_ipc::helpers::*;

use rpfm_lib::utils::files_from_subdir;

use crate::settings::Settings;

const UPDATE_EXTENSION: &str = "zip";
const REPO_OWNER: &str = "Frodo45127";
const REPO_NAME: &str = "rpfm";

const UPDATE_FOLDER_PREFIX: &str = "updates";

/// Filename of the changelog inside the release archive. Opened with the
/// system handler at the end of [`update_main_program_with`].
pub const CHANGELOG_FILE: &str = "Changelog.txt";

/// Setting value identifying the stable update channel.
pub const STABLE: &str = "Stable";

/// Setting value identifying the beta update channel.
pub const BETA: &str = "Beta";

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// Marker type used as a namespace for updater-related items.
///
/// Currently empty; lives here so future utility functions can hang off
/// `Updater::*` without breaking the existing function-style API.
pub struct Updater {}

/// Channels RPFM can pull updates from.
///
/// Versions in the third semver component greater than or equal to `99` are
/// treated as **betas** (e.g. `4.7.99`), versions below `99` as **stable**.
/// The check logic in [`check_updates_rpfm_with`] uses this to allow opting
/// out of betas: a beta user with the stable channel selected will see the
/// most recent stable release as an available "update" even when its
/// version number is lower.
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum UpdateChannel {
    /// Only stable releases are considered.
    Stable,
    /// Both stable and beta releases are considered, latest first.
    Beta
}

//---------------------------------------------------------------------------//
//                              Backend functions
//---------------------------------------------------------------------------//

/// Download the latest release for the configured update channel and replace
/// the running install with it. Opens the changelog at the end.
///
/// Errors if the network request fails, no asset is available for the
/// current architecture, or the in-place file replace fails.
pub fn update_main_program(settings: &Settings) -> Result<()> {
    let channel = update_channel(settings);
    update_main_program_with(|| last_release(channel))
}

/// Implementation backing [`update_main_program`], with the release source
/// abstracted as a closure so unit tests can inject a stub release without
/// hitting the network.
pub fn update_main_program_with(fetch_release: impl FnOnce() -> Result<Release>) -> Result<()> {
    let last_release = fetch_release()?;

    // Get the download for our architecture.
    let asset = last_release.asset_for(get_target(), None).ok_or_else(|| anyhow!("No download available for your architecture."))?;
    let mut tmp_path = std::env::current_exe().unwrap();
    tmp_path.pop();
    let tmp_dir = Builder::new()
        .prefix(UPDATE_FOLDER_PREFIX)
        .tempdir_in(tmp_path)?;

    DirBuilder::new().recursive(true).create(&tmp_dir)?;

    // Nested stuff, because this seems to have problems with creating his own files before using them.
    {
        let tmp_zip_path = tmp_dir.path().join(&asset.name);
        let tmp_zip = File::create(&tmp_zip_path)?;

        Download::from_url(&asset.download_url)
            .set_header(reqwest::header::ACCEPT, "application/octet-stream".parse().unwrap())
            .download_to(&tmp_zip)?;

        // Due to bugs in the `self_update` crate, we can't use `Extract` for the zip path.
        extract_zip(&tmp_zip_path, tmp_dir.path()).map_err(|e| anyhow!("There was an error while extracting the update. This means either I uploaded a broken file, or your download was incomplete. In any case, no changes have been done so… try again later: {e}"))?;
    }

    let mut dest_base_path = current_exe()?;
    dest_base_path.pop();

    for updated_file in &files_from_subdir(tmp_dir.path(), true)? {

        // Ignore the downloaded ZIP.
        if let Some(extension) = updated_file.extension() {
            if let Some(extension) = extension.to_str() {
                if extension == UPDATE_EXTENSION {
                    continue;
                }
            }
        }

        let mut tmp_file = updated_file.to_path_buf();
        tmp_file.set_file_name(format!("{}_replacement_tmp", updated_file.file_name().unwrap().to_str().unwrap()));

        // Fix for files in folders: we have to get the destination path with the folders included.
        let tmp_file_relative = updated_file.strip_prefix(tmp_dir.path()).unwrap();
        let dest_file = dest_base_path.join(tmp_file_relative);

        // Make sure the destination folder actually exists, or this will fail.
        let mut dest_folder = dest_base_path.join(tmp_file_relative);
        dest_folder.pop();
        DirBuilder::new().recursive(true).create(&dest_folder)?;

        Move::from_source(updated_file)
            .replace_using_temp(&tmp_file)
            .to_dest(&dest_file)?;
    }

    // Open the changelog because people don't read it.
    let changelog_path = dest_base_path.join(CHANGELOG_FILE);
    let _ = open::that(changelog_path);

    Ok(())
}

/// Extract a zip archive at `source` into `into_dir`.
///
/// Replacement for `self_update::Extract` which on Windows fails with
/// `os error 267` ("The directory name is invalid") on any zip that
/// contains directory entries — it calls `fs::File::create` on every
/// entry without checking whether the entry is a directory.
fn extract_zip(source: &Path, into_dir: &Path) -> Result<()> {
    let file = File::open(source)?;
    let mut archive = ZipArchive::new(file)?;
    for i in 0..archive.len() {
        let mut entry = archive.by_index(i)?;
        let Some(rel_path) = entry.enclosed_name() else { continue };
        let dest = into_dir.join(rel_path);

        if entry.is_dir() {
            fs::create_dir_all(&dest)?;
        } else {
            if let Some(parent) = dest.parent() {
                fs::create_dir_all(parent)?;
            }
            let mut out = File::create(&dest)?;
            io::copy(&mut entry, &mut out)?;
        }
    }
    Ok(())
}

/// This function takes care of checking for new RPFM updates.
///
/// Also, this has a special behavior: If we have a beta version and we have the stable channel selected,
/// it'll pick the newest stable release, even if it's older than our beta. That way we can easily opt-out of betas.
pub fn check_updates_rpfm(settings: &Settings) -> Result<APIResponse> {
    let channel = update_channel(settings);
    let current_version = cargo_crate_version!();
    check_updates_rpfm_with(current_version, channel, || last_release(channel))
}

/// Inner function that accepts injectable parameters for testability.
///
/// `current_version_str` is a semver string like "4.7.99".
/// `fetch_release` provides the latest release to compare against.
pub fn check_updates_rpfm_with(current_version_str: &str, update_channel: UpdateChannel, fetch_release: impl FnOnce() -> Result<Release>) -> Result<APIResponse> {
    let last_release = fetch_release()?;

    let current_version = current_version_str.split('.').map(|x| x.parse::<i32>().unwrap_or(0)).collect::<Vec<i32>>();
    let last_version = &last_release.version.split('.').map(|x| x.parse::<i32>().unwrap_or(0)).collect::<Vec<i32>>();

    // Before doing anything else, check if we are going back to stable after a beta, and we are currently in a beta version.
    // In that case, return the last stable as valid.
    if let UpdateChannel::Stable = update_channel {
        if current_version[2] >= 99 {
            return Ok(APIResponse::NewStableUpdate(format!("v{}", last_version.iter().map(|x| x.to_string()).join("."))));
        }
    }

    // Get the version numbers from our version and from the latest released version, so we can compare them.
    let first = (last_version[0], current_version[0]);
    let second = (last_version[1], current_version[1]);
    let third = (last_version[2], current_version[2]);

    // If this is triggered, there has been a problem parsing the current/remote version.
    if first.0 == 0 && second.0 == 0 && third.0 == 0 || first.1 == 0 && second.1 == 0 && third.1 == 0 {
        Ok(APIResponse::UnknownVersion)
    }

    // If the current version is different than the last released version...
    else if last_version != &current_version {

        // If the latest released version is lesser than the current version...
        // No update. We are using a newer build than the last build released (dev?).
        if first.0 < first.1 { Ok(APIResponse::NoUpdate) }

        // If the latest released version is greater than the current version...
        // New major update. No more checks needed.
        else if first.0 > first.1 {
            match update_channel {
                UpdateChannel::Stable => Ok(APIResponse::NewStableUpdate(format!("v{}", last_version.iter().map(|x| x.to_string()).join(".")))),
                UpdateChannel::Beta => Ok(APIResponse::NewBetaUpdate(format!("v{}", last_version.iter().map(|x| x.to_string()).join(".")))),
            }
        }

        // If the latest released version the same than the current version, we check the second, then the third number.
        // No update. We are using a newer build than the last build released (dev?).
        else if second.0 < second.1 { Ok(APIResponse::NoUpdate) }

        // New major update. No more checks needed.
        else if second.0 > second.1 {
            match update_channel {
                UpdateChannel::Stable => Ok(APIResponse::NewStableUpdate(format!("v{}", last_version.iter().map(|x| x.to_string()).join(".")))),
                UpdateChannel::Beta => Ok(APIResponse::NewBetaUpdate(format!("v{}", last_version.iter().map(|x| x.to_string()).join(".")))),
            }
        }

        // We check the last number in the versions, and repeat. Scraping the barrel...
        // No update. We are using a newer build than the last build released (dev?).
        else if third.0 < third.1 { Ok(APIResponse::NoUpdate) }

        // If the latest released version only has the last number higher, is a hotfix.
        else if third.0 > third.1 {
            match update_channel {
                UpdateChannel::Stable => Ok(APIResponse::NewUpdateHotfix(format!("v{}", last_version.iter().map(|x| x.to_string()).join(".")))),
                UpdateChannel::Beta => Ok(APIResponse::NewBetaUpdate(format!("v{}", last_version.iter().map(|x| x.to_string()).join(".")))),
            }
        }

        // This means both are the same, and the checks will never reach this place thanks to the parent if.
        else { unreachable!("check_updates") }
    }
    else {
        Ok(APIResponse::NoUpdate)
    }
}

/// Fetch the most recent release from GitHub matching `update_channel`.
///
/// Returns the first release whose third semver component is `< 99` for
/// `Stable`, or the absolute latest for `Beta`.
pub fn last_release(update_channel: UpdateChannel) -> Result<Release> {
    let releases = ReleaseList::configure()
        .repo_owner(REPO_OWNER)
        .repo_name(REPO_NAME)
        .build()?
        .fetch()?;

    match releases.iter().find(|release| {
        match update_channel {
            UpdateChannel::Stable => release.version.split('.').collect::<Vec<&str>>()[2].parse::<i32>().unwrap_or(0) < 99,
            UpdateChannel::Beta => true
        }
    }) {
        Some(last_release) => Ok(last_release.clone()),
        None => Err(anyhow!("Failed to get last release (should never happen)."))
    }
}

/// Read the persisted update channel from settings. Defaults to
/// [`UpdateChannel::Stable`] when the setting is missing or unrecognised.
pub fn update_channel(settings: &Settings) -> UpdateChannel {
    match &*settings.string("update_channel") {
        BETA => UpdateChannel::Beta,
        _ => UpdateChannel::Stable,
    }
}

impl Display for UpdateChannel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        Display::fmt(match &self {
            UpdateChannel::Stable => STABLE,
            UpdateChannel::Beta => BETA,
        }, f)
    }
}
