//---------------------------------------------------------------------------//
// Copyright (c) 2017-2020 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

/*!
Module with all the code to interact with Updates and the Main Program Update Checker.
!*/

use itertools::Itertools;
use self_update::{Download, get_target, Move, backends::github::ReleaseList, cargo_crate_version, update::Release};
use serde_derive::{Serialize, Deserialize};
use tempfile::Builder;

use std::env::{current_dir, current_exe};
use std::fs::{DirBuilder, File};

use rpfm_error::{Error, ErrorKind, Result};

use crate::common::get_files_from_subdir;
use crate::SETTINGS;

const UPDATE_EXTENSION: &str = "zip";
const REPO_OWNER: &str = "Frodo45127";
const REPO_NAME: &str = "rpfm_test_updater";

const UPDATE_FOLDER_PREFIX: &str = "updates";

pub const STABLE: &str = "Stable";
pub const BETA: &str = "Beta";

//---------------------------------------------------------------------------//
//                              Enum & Structs
//---------------------------------------------------------------------------//

/// This enum controls the channels through where RPFM will try to update.
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum UpdateChannel {
    Stable,
    Beta
}

/// This enum controls the possible responses from the server when checking for RPFM updates.
#[derive(Debug, Serialize, Deserialize)]
pub enum APIResponse {

    /// This means a beta update was found.
    SuccessNewBetaUpdate(String),

    /// This means a major stable update was found.
    SuccessNewStableUpdate(String),

    /// This means a minor stable update was found.
    SuccessNewUpdateHotfix(String),

    /// This means no update was found.
    SuccessNoUpdate,

    /// This means don't know if there was an update or not, because the version we got was invalid.
    SuccessUnknownVersion,

    /// This means there was an error when checking for updates.
    Error,
}

//---------------------------------------------------------------------------//
//                              Functions
//---------------------------------------------------------------------------//

/// This function takes care of updating RPFM itself when a new version comes out.
pub fn update_main_program() -> Result<()> {
    let update_channel = get_update_channel();
    let last_release = get_last_release(update_channel)?;

    // Get the download for our architecture.
    let asset = last_release.asset_for(&get_target()).ok_or_else(|| Error::from(ErrorKind::NoUpdateForYourArchitecture))?;
    let tmp_dir = Builder::new()
        .prefix(UPDATE_FOLDER_PREFIX)
        .tempdir_in(current_dir()?)?;

    DirBuilder::new().recursive(true).create(&tmp_dir)?;

    // Nested stuff, because this seems to have problems with creating his own files before using them.
    {
        let tmp_zip_path = tmp_dir.path().join(&asset.name);
        let tmp_zip = File::create(&tmp_zip_path)?;

        Download::from_url(&asset.download_url)
            .set_header(reqwest::header::ACCEPT, "application/octet-stream".parse().unwrap())
            .download_to(&tmp_zip).unwrap();

        // self_update extractor doesn't work. It fails on every-single-test I did. So we use another one.
        let tmp_zip = File::open(&tmp_zip_path)?;
        zip_extract::extract(tmp_zip, &tmp_dir.path().to_path_buf(), true).map_err(|_| Error::from(ErrorKind::ErrorExtractingUpdate))?;
    }

    let mut dest_base_path = current_exe()?;
    dest_base_path.pop();

    for updated_file in &get_files_from_subdir(&tmp_dir.path())? {

        // Ignore the downloaded ZIP.
        if let Some(extension) = updated_file.extension() {
            if let Some(extension) = extension.to_str() {
                if extension == UPDATE_EXTENSION {
                    continue;
                }
            }
        }

        let mut tmp_file = updated_file.to_path_buf();
        tmp_file.set_file_name(&format!("{}_replacement_tmp", updated_file.file_name().unwrap().to_str().unwrap()));

        // Fix for files in folders: we have to get the destination path with the folders included.
        let tmp_file_relative = updated_file.strip_prefix(tmp_dir.path()).unwrap();
        let dest_file = dest_base_path.join(&tmp_file_relative);

        // Make sure the destination folder actually exists, or this will fail.
        let mut dest_folder = dest_base_path.join(&tmp_file_relative);
        dest_folder.pop();
        DirBuilder::new().recursive(true).create(&dest_folder)?;

        Move::from_source(&updated_file)
            .replace_using_temp(&tmp_file)
            .to_dest(&dest_file)?;
    }

    Ok(())
}

/// This function takes care of checking for new RPFM updates.
///
/// Also, this has a special behavior: If we have a beta version and we have the stable channel selected,
/// it'll pick the newest stable release, even if it's older than our beta. That way we can easely opt-out of betas.
pub fn check_updates_rpfm() -> Result<APIResponse> {
    let update_channel = get_update_channel();
    let last_release = get_last_release(update_channel)?;

    let current_version = cargo_crate_version!().split(".").map(|x| x.parse::<i32>().unwrap_or(0)).collect::<Vec<i32>>();
    let last_version = &last_release.version.split(".").map(|x| x.parse::<i32>().unwrap_or(0)).collect::<Vec<i32>>();

    // Before doing anything else, check if we are going back to stable after a beta, and we are currently in a beta version.
    // In that case, return the last stable as valid.
    if let UpdateChannel::Stable = update_channel {
        if current_version[2] >= 99 {
            return Ok(APIResponse::SuccessNewStableUpdate(format!("v{}", last_version.iter().map(|x| x.to_string()).join("."))));
        }
    }

    // Get the version numbers from our version and from the latest released version, so we can compare them.
    let first = (last_version[0], current_version[0]);
    let second = (last_version[1], current_version[1]);
    let third = (last_version[2], current_version[2]);

    // If this is triggered, there has been a problem parsing the current/remote version.
    if first.0 == 0 && second.0 == 0 && third.0 == 0 || first.1 == 0 && second.1 == 0 && third.1 == 0 {
        Ok(APIResponse::SuccessUnknownVersion)
    }

    // If the current version is different than the last released version...
    else if last_version != &current_version {

        // If the latest released version is lesser than the current version...
        // No update. We are using a newer build than the last build released (dev?).
        if first.0 < first.1 { Ok(APIResponse::SuccessNoUpdate) }

        // If the latest released version is greater than the current version...
        // New major update. No more checks needed.
        else if first.0 > first.1 {
            match update_channel {
                UpdateChannel::Stable => Ok(APIResponse::SuccessNewStableUpdate(format!("v{}", last_version.iter().map(|x| x.to_string()).join(".")))),
                UpdateChannel::Beta => Ok(APIResponse::SuccessNewBetaUpdate(format!("v{}", last_version.iter().map(|x| x.to_string()).join(".")))),
            }
        }

        // If the latest released version the same than the current version, we check the second, then the third number.
        // No update. We are using a newer build than the last build released (dev?).
        else if second.0 < second.1 { Ok(APIResponse::SuccessNoUpdate) }

        // New major update. No more checks needed.
        else if second.0 > second.1 {
            match update_channel {
                UpdateChannel::Stable => Ok(APIResponse::SuccessNewStableUpdate(format!("v{}", last_version.iter().map(|x| x.to_string()).join(".")))),
                UpdateChannel::Beta => Ok(APIResponse::SuccessNewBetaUpdate(format!("v{}", last_version.iter().map(|x| x.to_string()).join(".")))),
            }
        }

        // We check the last number in the versions, and repeat. Scraping the barrel...
        // No update. We are using a newer build than the last build released (dev?).
        else if third.0 < third.1 { Ok(APIResponse::SuccessNoUpdate) }

        // If the latest released version only has the last number higher, is a hotfix.
        else if third.0 > third.1 {
            match update_channel {
                UpdateChannel::Stable => Ok(APIResponse::SuccessNewUpdateHotfix(format!("v{}", last_version.iter().map(|x| x.to_string()).join(".")))),
                UpdateChannel::Beta => Ok(APIResponse::SuccessNewBetaUpdate(format!("v{}", last_version.iter().map(|x| x.to_string()).join(".")))),
            }
        }

        // This means both are the same, and the checks will never reach this place thanks to the parent if.
        else { unreachable!() }
    }
    else {
        Ok(APIResponse::SuccessNoUpdate)
    }
}

/// This function returns the last release available, according to our update channel.
pub fn get_last_release(update_channel: UpdateChannel) -> Result<Release> {
    let releases = ReleaseList::configure()
        .repo_owner(REPO_OWNER)
        .repo_name(REPO_NAME)
        .build()?
        .fetch()?;

    match releases.iter().find(|release| {
        match update_channel {
            UpdateChannel::Stable => release.version.split(".").collect::<Vec<&str>>()[2].parse::<i32>().unwrap_or(0) < 99,
            UpdateChannel::Beta => true
        }
    }) {
        Some(last_release) => Ok(last_release.clone()),
        None => Err(ErrorKind::Generic.into())
    }
}

/// This function returns the currently selected update channel.
pub fn get_update_channel() -> UpdateChannel {
    match &*SETTINGS.read().unwrap().settings_string["update_channel"] {
        BETA => UpdateChannel::Beta,
        _ => UpdateChannel::Stable,
    }
}

/// Implementation of ToString.
impl ToString for UpdateChannel {
    fn to_string(&self) -> String {
        match &self {
            UpdateChannel::Stable => STABLE.to_owned(),
            UpdateChannel::Beta => BETA.to_owned(),
        }
    }
}
