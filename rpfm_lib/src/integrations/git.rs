//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Basic Git repository management for schema updates and version control.
//!
//! This module provides minimal Git functionality needed by RPFM to:
//! - Clone and update remote repositories (primarily for schema updates)
//! - Check for updates without downloading
//! - Handle branch switching and stashing
//!
//! # Limitations
//!
//! This is **not** a full Git client. It provides only the essential operations needed
//! for RPFM's schema update system. For comprehensive Git operations, use a dedicated Git client.
//!
//! # Main Use Case
//!
//! RPFM uses this integration to keep schemas synchronized with the official schema repository.
//! The typical workflow is:
//!
//! 1. Check if updates are available ([`GitIntegration::check_update()`])
//! 2. If available, download updates ([`GitIntegration::update_repo()`])
//! 3. Schema files are now up-to-date and can be loaded
//!
//! # Example
//!
//! ```no_run
//! use rpfm_lib::integrations::git::{GitIntegration, GitResponse};
//! use std::path::Path;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let git = GitIntegration::new(
//!     Path::new("schemas"),
//!     "https://github.com/Frodo45127/rpfm-schemas",
//!     "master",
//!     "origin"
//! );
//!
//! match git.check_update()? {
//!     GitResponse::NewUpdate => {
//!         println!("Update available, downloading...");
//!         git.update_repo()?;
//!     }
//!     GitResponse::NoUpdate => println!("Already up to date"),
//!     GitResponse::NoLocalFiles => {
//!         println!("No local copy, cloning...");
//!         git.update_repo()?;
//!     }
//!     GitResponse::Diverged => println!("Local changes conflict with remote"),
//! }
//! # Ok(())
//! # }
//! ```

use git2::{Reference, ReferenceFormat, Repository, Signature, StashFlags, build::CheckoutBuilder};
use serde::{Deserialize, Serialize};

use std::fs::{DirBuilder, File};
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use std::process::Command as SystemCommand;

use crate::error::{RLibError, Result};

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// Configuration for Git repository operations.
///
/// This struct holds all the information needed to perform fetch/pull operations
/// on a Git repository. Create an instance with [`GitIntegration::new()`] and use it
/// to check for or download updates.
#[derive(Debug)]
pub struct GitIntegration {

    /// Local filesystem path where the repository is (or will be) cloned.
    local_path: PathBuf,

    /// Remote repository URL (e.g., `"https://github.com/user/repo"`).
    url: String,

    /// Branch name to track (e.g., `"master"`, `"main"`).
    branch: String,

    /// Remote name (typically `"origin"`).
    remote: String,
}

/// Result of checking for repository updates.
///
/// Returned by [`GitIntegration::check_update()`] to indicate the repository's state
/// relative to the remote.
#[derive(Debug, Serialize, Deserialize)]
pub enum GitResponse {
    /// A new update is available on the remote.
    NewUpdate,

    /// The local repository is up-to-date with the remote.
    NoUpdate,

    /// No local copy of the repository exists (needs cloning).
    NoLocalFiles,

    /// Local and remote branches have diverged (conflicting changes).
    Diverged,
}

//---------------------------------------------------------------------------//
//                       Enum & Structs Implementations
//---------------------------------------------------------------------------//

impl GitIntegration {

    /// Creates a new Git integration configuration.
    ///
    /// # Arguments
    ///
    /// * `local_path` - Local directory for the repository
    /// * `url` - Remote repository URL
    /// * `branch` - Branch name to track
    /// * `remote` - Remote name (usually `"origin"`)
    ///
    /// # Returns
    ///
    /// Returns a configured [`GitIntegration`] instance ready for operations.
    pub fn new(local_path: &Path, url: &str, branch: &str, remote: &str) -> Self {
        Self {
            local_path: local_path.to_owned(),
            url: url.to_owned(),
            branch: branch.to_owned(),
            remote: remote.to_owned(),
        }
    }

    /// Initializes a new Git repository at the configured local path.
    ///
    /// Creates a `.git` directory and initializes Git metadata. Use this to create
    /// a new repository from scratch.
    ///
    /// # Returns
    ///
    /// Returns the initialized repository handle, or an error if initialization fails.
    pub fn init(&self) -> Result<Repository> {
        Repository::init(&self.local_path).map_err(From::from)
    }

    /// Creates or replaces a `.gitignore` file in the repository.
    ///
    /// # Arguments
    ///
    /// * `contents` - The complete contents of the `.gitignore` file
    ///
    /// # Returns
    ///
    /// Returns [`Ok`] if the file was written successfully, or an error if file I/O fails.
    ///
    /// # Note
    ///
    /// This will overwrite any existing `.gitignore` file.
    pub fn add_gitignore(&self, contents: &str) -> Result<()> {
        let mut file = BufWriter::new(File::create(self.local_path.join(".gitignore"))?);
        file.write_all(contents.as_bytes()).map_err(From::from)
    }

    /// Switches the repository to a different branch.
    ///
    /// # Arguments
    ///
    /// * `repo` - The repository to operate on
    /// * `refs` - Full reference name (e.g., `"refs/heads/master"`)
    ///
    /// # Returns
    ///
    /// Returns [`Ok`] if the checkout succeeds, or an error if the operation fails.
    pub fn checkout_branch(&self, repo: &Repository, refs: &str) -> Result<()> {
        let head = repo.head().unwrap();
        let oid = head.target().unwrap();
        let commit = repo.find_commit(oid)?;
        let branch_name = refs.splitn(3, '/').collect::<Vec<_>>()[2].to_owned();
        let _ = repo.branch(&branch_name, &commit, false);

        let branch_object = repo.revparse_single(refs)?;
        repo.checkout_tree(&branch_object, None)?;
        repo.set_head(refs)?;
        Ok(())
    }

    /// Checks if updates are available without downloading them.
    ///
    /// This function fetches metadata from the remote and compares it with the local
    /// repository state to determine if new commits are available. Local changes are
    /// temporarily stashed and the branch is restored after checking.
    ///
    /// # Returns
    ///
    /// Returns a [`GitResponse`] indicating:
    /// - [`GitResponse::NewUpdate`]: Remote has new commits
    /// - [`GitResponse::NoUpdate`]: Already up-to-date
    /// - [`GitResponse::NoLocalFiles`]: Repository hasn't been cloned yet
    /// - [`GitResponse::Diverged`]: Local and remote branches have conflicting changes
    ///
    /// # Behavior
    ///
    /// 1. Stashes any local changes
    /// 2. Switches to the tracked branch if needed
    /// 3. Fetches remote metadata
    /// 4. Compares local and remote states
    /// 5. Restores original branch and unstashes changes
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
        let stash_id = repo.stash_save(&signature, &format!("Stashed changes before checking for updates from branch {current_branch_name}"), Some(StashFlags::INCLUDE_UNTRACKED));

        // In case we're not in master, checkout the master branch.
        if current_branch_name != master_refname {
            self.checkout_branch(&repo, &master_refname)?;
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
            self.checkout_branch(&repo, &current_branch_name)?;
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

    /// Downloads and applies updates from the remote repository.
    ///
    /// This function performs a full update of the repository:
    /// - If the repository doesn't exist locally, it clones it
    /// - If it exists, it pulls the latest changes from the tracked branch
    /// - Handles diverged branches by re-cloning if necessary
    ///
    /// # Returns
    ///
    /// Returns [`Ok`] if the update succeeds, or an error if:
    /// - The repository cannot be cloned
    /// - There are no updates available (returns [`RLibError::GitErrorNoUpdatesAvailable`])
    /// - The download fails
    ///
    /// # Behavior
    ///
    /// 1. Opens or clones the repository
    /// 2. Stashes local changes
    /// 3. Switches to the tracked branch
    /// 4. Fetches and merges remote changes (fast-forward when possible)
    /// 5. If branches have diverged, re-clones the repository from scratch
    /// 6. Restores original branch and unstashes changes
    ///
    /// # Platform-Specific Behavior
    ///
    /// On Windows, this function removes read-only flags before deleting directories
    /// to avoid permission errors.
    pub fn update_repo(&self) -> Result<()> {
        let mut new_repo = false;
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
                    Ok(repo) => {
                        new_repo = true;
                        repo
                    },
                    Err(_) => return Err(RLibError::GitErrorDownloadFromRepo(self.url.to_owned())),
                }
            }
        };

        // Just in case there are loose changes, stash them.
        // Ignore a fail on this, as it's possible we don't have contents to stash.
        let current_branch_name = Reference::normalize_name(repo.head()?.name().unwrap(), ReferenceFormat::ALLOW_ONELEVEL)?.to_lowercase();
        let master_refname = format!("refs/heads/{}", self.branch);

        let signature = Signature::now("RPFM Updater", "-")?;
        let stash_id = repo.stash_save(&signature, &format!("Stashed changes before update from branch {current_branch_name}"), Some(StashFlags::INCLUDE_UNTRACKED));

        // In case we're not in master, checkout the master branch.
        if current_branch_name != master_refname {
            self.checkout_branch(&repo, &master_refname)?;
        }

        // If we just cloned a new repo and changed branches, return.
        if new_repo {
            return Ok(());
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
                self.checkout_branch(&repo, &current_branch_name)?;
            }
            if stash_id.is_ok() {
                let _ = repo.stash_pop(0, None);
            }
            Err(RLibError::GitErrorNoUpdatesAvailable(self.url.to_owned()))
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
                self.checkout_branch(&repo, &current_branch_name)?;
            }
            if stash_id.is_ok() {
                let _ = repo.stash_pop(0, None);
            }

            Err(RLibError::GitErrorDownloadFromRepo(self.url.to_owned()))
        }
    }
}
