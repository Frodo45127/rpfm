//---------------------------------------------------------------------------//
// Copyright (c) 2017-2022 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

/*!
Module that contains util functions for git integration.
!*/

use git2::{Reference, ReferenceFormat, Repository, Signature, StashFlags, build::CheckoutBuilder};

use std::fs::DirBuilder;
use std::path::{Path, PathBuf};
use std::process::Command as SystemCommand;

use crate::error::{RCommonError, Result};

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// Struct containing the data needed to perform a fetch/pull from a repo.
#[derive(Debug)]
pub struct GitIntegration {

    /// Local Path of the repo.
    local_path: PathBuf,

    /// URL of the repo.
    url: String,

    /// Branch to fetch/pull.
    branch: String,

    /// Remote to fetch/pull from.
    remote: String,
}

/// Possible responses we can get from a fetch/pull.
#[derive(Debug)]
pub enum GitResponse {
    NewUpdate,
    NoUpdate,
    NoLocalFiles,
    Diverged,
}

//---------------------------------------------------------------------------//
//                       Enum & Structs Implementations
//---------------------------------------------------------------------------//

impl GitIntegration {

    /// This function creates a new GitIntegration struct with data for a git operation.
    pub fn new(local_path: &Path, url: &str, branch: &str, remote: &str) -> Self {
        Self {
            local_path: local_path.to_owned(),
            url: url.to_owned(),
            branch: branch.to_owned(),
            remote: remote.to_owned(),
        }
    }

    /// This function tries to initializes a git repo.
    pub fn init(&self) -> Result<Repository> {
        Repository::init(&self.local_path).map_err(From::from)
    }

    /// This function checks if there is a new update for the current repo.
    pub fn check_update(&self) -> Result<GitResponse> {
        let mut repo = match Repository::open(&self.local_path) {
            Ok(repo) => repo,

            // If this fails, it means we either we don´t have the repo downloaded, or we have a folder without the .git folder.
            Err(_) => return Ok(GitResponse::NoLocalFiles),
        };

        // Just in case there are loose changes, stash them.
        // Ignore a fail on this, as it's possible we don't have contents to stash.
        let current_branch_name = Reference::normalize_name(repo.head()?.name().unwrap(), ReferenceFormat::ALLOW_ONELEVEL)?.to_lowercase();
        let master_refname = format!("refs/heads/{}", self.branch);

        let signature = Signature::now("RPFM Updater", "-")?;
        let stash_id = repo.stash_save(&signature, &format!("Stashed changes before checking for updates from branch {}", current_branch_name), Some(StashFlags::INCLUDE_UNTRACKED));

        // In case we're not in master, checkout the master branch.
        if current_branch_name != master_refname {
            repo.set_head(&master_refname)?;
        }

        // Fetch the info of the master branch.
        repo.find_remote(&self.remote)?.fetch(&[&self.branch], None, None)?;
        let analysis = {
            let fetch_head = repo.find_reference("FETCH_HEAD")?;
            let fetch_commit = repo.reference_to_annotated_commit(&fetch_head)?;
            repo.merge_analysis(&[&fetch_commit])?
        };

        // Reset the repo to his original state after the check
        if current_branch_name != master_refname {
            let _ = repo.set_head(&current_branch_name);
        }
        if stash_id.is_ok() {
            let _ = repo.stash_pop(0, None);
        }

        if analysis.0.is_up_to_date() {
            Ok(GitResponse::NoUpdate)
        }

        // If the branch is a fast-forward, or has diverged, ask for an update.
        else if analysis.0.is_fast_forward() || analysis.0.is_normal() || analysis.0.is_none() || analysis.0.is_unborn() {
            Ok(GitResponse::NewUpdate)
        }

        // Otherwise, it means the branches diverged. In this case, return a diverged.
        else {
            Ok(GitResponse::Diverged)
        }
    }

    /// This function downloads the latest revision of the current repository.
    pub fn update_repo(&self) -> Result<()> {
        let mut repo = match Repository::open(&self.local_path) {
            Ok(repo) => repo,
            Err(_) => {

                // If it fails to open, it means either we don't have the .git folder, or we don't have a folder at all.
                // In either case, recreate it and redownload the repo. No more steps are needed here.
                // On windows, remove the read-only flags before doing anything else, or this will fail.
                if cfg!(target_os = "windows") {
                    let path = self.local_path.to_string_lossy().to_string() + "\\*.*";
                    let _ = SystemCommand::new("attrib").arg("-r").arg(path).arg("/s").output();
                }
                let _ = std::fs::remove_dir_all(&self.local_path);
                DirBuilder::new().recursive(true).create(&self.local_path)?;
                match Repository::clone(&self.url, &self.local_path) {
                    Ok(_) => return Ok(()),
                    Err(_) => return Err(RCommonError::GitErrorDownloadFromRepo(self.url.to_owned())),
                }
            }
        };

        // Just in case there are loose changes, stash them.
        // Ignore a fail on this, as it's possible we don't have contents to stash.
        let current_branch_name = Reference::normalize_name(repo.head()?.name().unwrap(), ReferenceFormat::ALLOW_ONELEVEL)?.to_lowercase();
        let master_refname = format!("refs/heads/{}", self.branch);

        let signature = Signature::now("RPFM Updater", "-")?;
        let stash_id = repo.stash_save(&signature, &format!("Stashed changes before update from branch {}", current_branch_name), Some(StashFlags::INCLUDE_UNTRACKED));

        // In case we're not in master, checkout the master branch.
        if current_branch_name != master_refname {
            repo.set_head(&master_refname)?;
        }

        // If it worked, now we have to do a pull from master. Sadly, git2-rs does not support pull.
        // Instead, we kinda force a fast-forward. Made in StackOverflow.
        repo.find_remote(&self.remote)?.fetch(&[&self.branch], None, None)?;
        let (analysis, fetch_commit_id) = {
            let fetch_head = repo.find_reference("FETCH_HEAD")?;
            let fetch_commit = repo.reference_to_annotated_commit(&fetch_head)?;
            (repo.merge_analysis(&[&fetch_commit])?, fetch_commit.id())
        };

        // If we're up to date, nothing more is needed.
        if analysis.0.is_up_to_date() {

            // Reset the repo to his original state after the check
            if current_branch_name != master_refname {
                let _ = repo.set_head(&current_branch_name);
            }
            if stash_id.is_ok() {
                let _ = repo.stash_pop(0, None);
            }
            Err(RCommonError::GitErrorNoUpdatesAvailable(self.url.to_owned()))
        }

        // If we can do a fast-forward, we do it. This is the preferred option.
        else if analysis.0.is_fast_forward() {
            let mut reference = repo.find_reference(&master_refname)?;
            reference.set_target(fetch_commit_id, "Fast-Forward")?;
            repo.set_head(&master_refname)?;
            repo.checkout_head(Some(CheckoutBuilder::default().force())).map_err(From::from)
        }

        // If not, we face multiple problems:
        // - If there are uncommitted changes: covered by the stash.
        // - If we're not in the branch: covered by the branch switch.
        // - If the branches diverged: this one... the cleanest way to deal with it should be redownload the repo.
        else if analysis.0.is_normal() || analysis.0.is_none() || analysis.0.is_unborn() {

            // On windows, remove the read-only flags before doing anything else, or this will fail.
            if cfg!(target_os = "windows") {
                let path = self.local_path.to_string_lossy().to_string() + "\\*.*";
                let _ = SystemCommand::new("attrib").arg("-r").arg(path).arg("/s").output();
            }
            let _ = std::fs::remove_dir_all(&self.local_path);
            self.update_repo()
        }
        else {

            // Reset the repo to his original state after the check
            if current_branch_name != master_refname {
                let _ = repo.set_head(&current_branch_name);
            }
            if stash_id.is_ok() {
                let _ = repo.stash_pop(0, None);
            }

            Err(RCommonError::GitErrorDownloadFromRepo(self.url.to_owned()))
        }
    }
}
