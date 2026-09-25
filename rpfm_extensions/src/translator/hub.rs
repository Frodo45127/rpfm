//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Submission of translations to the Translation Hub.
//!
//! Builds, from a saved [`PackTranslation`], everything needed to submit it to the hub as a pull
//! request: where its file goes, which outdated file it replaces, the branch it's pushed to, and the
//! texts of the commit and the pull request. Then submits it through a [`GitHubClient`].

use getset::Getters;
use serde_derive::{Deserialize, Serialize};

use std::thread;
use std::time::{Duration, Instant};

use rpfm_lib::error::{RLibError, Result};
use rpfm_lib::integrations::github::{GitHubClient, NewPullRequest, TreeChange};

use super::{DEFAULT_SRC_LANG, PackTranslation};

/// How long to wait for a newly created fork to become usable.
const FORK_TIMEOUT: Duration = Duration::from_secs(60);

/// Time between checks for a newly created fork.
const FORK_POLL_INTERVAL: Duration = Duration::from_secs(2);

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// Line counts of a translation, ignoring lines removed from the pack.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Getters, Serialize, Deserialize)]
#[getset(get = "pub")]
pub struct TranslationStats {

    /// Lines in the pack.
    total: usize,

    /// Lines with an up-to-date translation.
    translated: usize,

    /// Lines that still need translating, because they're new or their source text changed.
    pending: usize,

    /// Up-to-date lines translated automatically and not reviewed yet.
    auto_translated: usize,
}

/// Everything needed to submit a translation to the Translation Hub as a pull request.
#[derive(Clone, Debug, PartialEq, Eq, Getters)]
#[getset(get = "pub")]
pub struct HubSubmission {

    /// Path of the translation's file in the hub.
    file_path: String,

    /// Path of the same translation in the other format version, to delete from the hub if it's there.
    replaced_path: Option<String>,

    /// Branch the submission is pushed to. It's the same for every submission of the same translation.
    branch: String,

    /// Message of the submission's commit.
    commit_message: String,

    /// Title of the pull request.
    title: String,

    /// Description of the pull request, in Markdown.
    body: String,
}

/// Result of submitting a translation to the Translation Hub.
#[derive(Clone, Debug, PartialEq, Eq, Getters, Serialize, Deserialize)]
#[getset(get = "pub")]
pub struct SubmissionResult {

    /// Web page of the pull request.
    url: String,

    /// Whether a new pull request was opened. `false` means an open one was updated.
    created: bool,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

impl PackTranslation {

    /// Path of this translation's file, relative to a translations folder or the hub's root.
    ///
    /// # Arguments
    ///
    /// * `game_key` - Key of the game the translation belongs to.
    ///
    /// # Returns
    ///
    /// `{game_key}/{pack_name}/{file_name}`, with `/` separators so it's also valid as a repository path.
    pub fn relative_path(&self, game_key: &str) -> String {
        format!("{}/{}/{}", game_key, self.pack_name, self.file_name())
    }

    /// Line counts of this translation, ignoring lines removed from the pack.
    pub fn stats(&self) -> TranslationStats {
        self.translations.values()
            .filter(|tr| !tr.rem)
            .fold(TranslationStats::default(), |mut stats, tr| {
                stats.total += 1;
                if tr.retr {
                    stats.pending += 1;
                } else {
                    stats.translated += 1;
                    if tr.aut {
                        stats.auto_translated += 1;
                    }
                }

                stats
            })
    }

    /// Build the submission of this translation to the Translation Hub.
    ///
    /// # Arguments
    ///
    /// * `game_key` - Key of the game the translation belongs to.
    ///
    /// # Returns
    ///
    /// The paths, branch and texts of the submission.
    pub fn hub_submission(&self, game_key: &str) -> HubSubmission {
        let folder = format!("{}/{}", game_key, self.pack_name);

        // Mirrors `save`: only EN-sourced translations have files in both formats, and `{language}.json` is always the EN one.
        let replaced_path = if self.src_lang.eq_ignore_ascii_case(DEFAULT_SRC_LANG) {
            let other_file_name = if self.version == 0 {
                format!("{}-{}.json", self.src_lang, self.language)
            } else {
                format!("{}.json", self.language)
            };

            Some(format!("{folder}/{other_file_name}"))
        } else {
            None
        };

        let branch = ["rpfm", game_key, &self.pack_name, &format!("{}-{}", self.src_lang, self.language)]
            .iter()
            .map(|segment| branch_segment(segment))
            .collect::<Vec<_>>()
            .join("/");

        let languages = format!("{} -> {}", self.src_lang, self.language);
        let stats = self.stats();
        let authors = if self.authors.is_empty() {
            "Not specified".to_owned()
        } else {
            self.authors.join(", ")
        };

        let body = format!("Translation submitted from RPFM's Translator.

- Game: `{game_key}`
- Pack: `{pack}`
- Languages: `{languages}`
- Format version: `{version}`
- Authors: {authors}
- Lines: {translated} of {total} translated, {pending} pending, {auto} of the translated ones auto-translated and not reviewed.
",
            pack = self.pack_name,
            version = self.version,
            translated = stats.translated,
            total = stats.total,
            pending = stats.pending,
            auto = stats.auto_translated,
        );

        HubSubmission {
            file_path: self.relative_path(game_key),
            replaced_path,
            branch,
            commit_message: format!("Update {} translation ({languages}) [{game_key}]", self.pack_name),
            title: format!("[{game_key}] Translation for {} ({languages})", self.pack_name),
            body,
        }
    }

    /// Submit this translation to the Translation Hub as a pull request, or update its open one.
    ///
    /// The commit goes on top of the hub's current default branch, in the hub itself if the user can push
    /// to it, or in the user's fork otherwise (created if needed). Each submission replaces the previous one
    /// in the translation's branch, so an open pull request always shows a single commit.
    ///
    /// # Arguments
    ///
    /// * `client` - GitHub client, signed in as the submitting user.
    /// * `hub_owner` - Owner of the Translation Hub repository.
    /// * `hub_name` - Name of the Translation Hub repository.
    /// * `game_key` - Key of the game the translation belongs to.
    /// * `content` - Contents of the translation's file, as saved on disk.
    ///
    /// # Returns
    ///
    /// The pull request's page, and whether it was newly opened.
    ///
    /// # Errors
    ///
    /// Returns an error if the hub can't be found, the fork isn't ready in time, or any GitHub request fails.
    pub fn submit_to_hub(&self, client: &GitHubClient, hub_owner: &str, hub_name: &str, game_key: &str, content: &str) -> Result<SubmissionResult> {
        let submission = self.hub_submission(game_key);
        let hub = client.repository(hub_owner, hub_name)?
            .ok_or_else(|| RLibError::GitHubRepositoryNotFound(format!("{hub_owner}/{hub_name}")))?;
        let base_branch = hub.default_branch();

        let (repo_owner, repo_name) = if *hub.can_push() {
            (hub_owner.to_owned(), hub_name.to_owned())
        } else {
            let fork = client.fork(hub_owner, hub_name)?;
            wait_for_fork(client, fork.owner(), fork.name(), fork.default_branch())?;

            // A fork whose branch has its own changes can't be synced. That's fine: forks share
            // objects with their upstream, so the commit can still be based on the hub's head.
            let _ = client.merge_upstream(fork.owner(), fork.name(), fork.default_branch());
            (fork.owner().to_owned(), fork.name().to_owned())
        };

        let base_commit = client.branch_head(hub_owner, hub_name, base_branch)?
            .ok_or_else(|| RLibError::GitHubRepositoryNotFound(format!("{hub_owner}/{hub_name}:{base_branch}")))?;
        let base_tree = client.commit_tree(hub_owner, hub_name, &base_commit)?;

        let blob = client.create_blob(&repo_owner, &repo_name, content)?;
        let mut changes = vec![TreeChange { path: submission.file_path().to_owned(), blob: Some(blob) }];

        // Deleting a file that isn't in the tree fails, so only delete the replaced file if the hub has it.
        if let Some(replaced_path) = submission.replaced_path() {
            if let Some((folder, file_name)) = replaced_path.rsplit_once('/') {
                if client.folder_entries(hub_owner, hub_name, folder, &base_commit)?.iter().any(|entry| entry == file_name) {
                    changes.push(TreeChange { path: replaced_path.to_owned(), blob: None });
                }
            }
        }

        let tree = client.create_tree(&repo_owner, &repo_name, &base_tree, &changes)?;
        let commit = client.create_commit(&repo_owner, &repo_name, submission.commit_message(), &tree, &base_commit)?;
        client.set_branch(&repo_owner, &repo_name, submission.branch(), &commit)?;

        let head = format!("{repo_owner}:{}", submission.branch());
        if let Some(pull) = client.open_pull_request(hub_owner, hub_name, &head, base_branch)? {
            return Ok(SubmissionResult { url: pull.html_url().to_owned(), created: false });
        }

        let pull = client.create_pull_request(hub_owner, hub_name, &NewPullRequest {
            title: submission.title().to_owned(),
            body: submission.body().to_owned(),
            head,
            base: base_branch.to_owned(),
        })?;

        Ok(SubmissionResult { url: pull.html_url().to_owned(), created: true })
    }
}

/// Wait until a fork is usable. GitHub creates forks in the background, so a new one may take a moment.
fn wait_for_fork(client: &GitHubClient, owner: &str, name: &str, branch: &str) -> Result<()> {
    let deadline = Instant::now() + FORK_TIMEOUT;
    while client.branch_head(owner, name, branch)?.is_none() {
        if Instant::now() >= deadline {
            return Err(RLibError::GitHubForkNotReady(format!("{owner}/{name}")));
        }

        thread::sleep(FORK_POLL_INTERVAL);
    }

    Ok(())
}

/// Turn a text into a valid segment of a git branch name.
///
/// Characters other than ASCII letters, digits, `.`, `_` and `-` become `-`, and the result avoids the
/// sequences git forbids in reference names: `..`, a leading or trailing `.`, and a trailing `.lock`.
fn branch_segment(text: &str) -> String {
    let mut segment = String::with_capacity(text.len());
    for character in text.chars() {
        let character = if character.is_ascii_alphanumeric() || matches!(character, '.' | '_' | '-') { character } else { '-' };
        let previous = segment.chars().last();

        // Collapse runs of dashes, and break up `..` so it can't appear.
        if (character == '-' && previous == Some('-')) || (character == '.' && previous == Some('.')) {
            continue;
        }

        segment.push(character);
    }

    let mut segment = segment.trim_matches(|character| character == '-' || character == '.').to_owned();
    if segment.ends_with(".lock") {
        segment.push('_');
    }

    if segment.is_empty() {
        segment.push('_');
    }

    segment
}
