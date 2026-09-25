//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Submission of translations to the Translation Hub, and the GitHub sign-in it needs.
//!
//! The GitHub token is kept in the OS keyring and never leaves the server: frontends only
//! see the sign-in state and the account's login.

use anyhow::{anyhow, Result};
use keyring::{Entry, Error as KeyringError};

use std::fs;

use rpfm_extensions::translator::PackTranslation;
use rpfm_extensions::translator::hub::SubmissionResult;

use rpfm_ipc::messages::GitHubSignInState;
use rpfm_ipc::settings_keys::GITHUB_LOGIN;

use rpfm_lib::error::RLibError;
use rpfm_lib::games::{TRANSLATIONS_REPO_NAME, TRANSLATIONS_REPO_OWNER};
use rpfm_lib::integrations::github::{self, DeviceCode, DeviceFlowPoll, GitHubClient, PUBLIC_REPO_SCOPE};

use crate::GITHUB_OAUTH_CLIENT_ID;
use crate::settings::{mutate_settings, translations_local_path, SETTINGS};

/// Service name of RPFM's entries in the OS keyring.
const KEYRING_SERVICE: &str = "rpfm";

/// Name of the keyring entry holding the GitHub token.
const KEYRING_GITHUB_TOKEN: &str = "github_token";

/// Environment variable to submit to another repository (`owner/name`) instead of the real hub. Debug builds only.
#[cfg(debug_assertions)]
const HUB_OVERRIDE_VAR: &str = "RPFM_TRANSLATION_HUB";

//-------------------------------------------------------------------------------//
//                             Enums & Structs
//-------------------------------------------------------------------------------//

/// Result of trying to submit a translation.
pub enum SubmitOutcome {

    /// The translation was submitted.
    Submitted(SubmissionResult),

    /// The user has to sign in to GitHub first, because they never did or GitHub rejected their sign-in.
    SignInRequired,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

/// Start a GitHub sign-in with the device flow.
///
/// # Errors
///
/// Returns an error if this build has no GitHub client ID, or the request fails.
pub fn sign_in_start() -> Result<DeviceCode> {
    if GITHUB_OAUTH_CLIENT_ID.is_empty() {
        return Err(anyhow!("This build of RPFM can't sign in to GitHub, as it was built without a GitHub client ID."));
    }

    github::request_device_code(GITHUB_OAUTH_CLIENT_ID, PUBLIC_REPO_SCOPE).map_err(From::from)
}

/// Check a GitHub sign-in. Once approved, keeps the token in the OS keyring and the account's login in the settings.
///
/// # Arguments
///
/// * `device_code` - Device code of the sign-in, from [`sign_in_start`].
///
/// # Errors
///
/// Returns an error if the request fails, or the token can't be stored in the keyring.
pub fn sign_in_poll(device_code: &str) -> Result<GitHubSignInState> {
    Ok(match github::poll_device_flow(GITHUB_OAUTH_CLIENT_ID, device_code)? {
        DeviceFlowPoll::Pending => GitHubSignInState::Pending,
        DeviceFlowPoll::SlowDown(interval) => GitHubSignInState::SlowDown(interval),
        DeviceFlowPoll::Expired => GitHubSignInState::Expired,
        DeviceFlowPoll::Denied => GitHubSignInState::Denied,
        DeviceFlowPoll::Granted(token) => {
            let login = GitHubClient::new(&token)?.user_login()?;
            keyring_entry()?.set_password(&token).map_err(keyring_error)?;
            mutate_settings(|settings| settings.set_string(GITHUB_LOGIN, &login))?;
            GitHubSignInState::SignedIn(login)
        },
    })
}

/// Login of the GitHub account the user is signed in as.
///
/// # Returns
///
/// The login, or `None` if not signed in.
///
/// # Errors
///
/// Returns an error if the keyring can't be read.
pub fn account() -> Result<Option<String>> {
    if token()?.is_none() {
        return Ok(None);
    }

    let login = SETTINGS.read().unwrap().string(GITHUB_LOGIN);
    Ok(Some(login))
}

/// Sign out of GitHub, deleting the stored token and login.
///
/// # Errors
///
/// Returns an error if the keyring or the settings can't be written.
pub fn sign_out() -> Result<()> {
    match keyring_entry()?.delete_credential() {
        Ok(()) | Err(KeyringError::NoEntry) => {},
        Err(error) => return Err(keyring_error(error)),
    }

    mutate_settings(|settings| settings.set_string(GITHUB_LOGIN, ""))?;
    Ok(())
}

/// Submit a saved translation to the Translation Hub, or update its open pull request.
///
/// If GitHub rejects the stored token, the user is signed out so the next attempt asks them to sign in again.
///
/// # Arguments
///
/// * `game_key` - Key of the game the translation belongs to.
/// * `pack_name` - Name of the pack the translation is for.
/// * `src_lang` - Source language code.
/// * `language` - Target language code.
///
/// # Errors
///
/// Returns an error if the translation can't be loaded, or the submission fails for any reason other than the sign-in.
pub fn submit(game_key: &str, pack_name: &str, src_lang: &str, language: &str) -> Result<SubmitOutcome> {
    let Some(token) = token()? else {
        return Ok(SubmitOutcome::SignInRequired);
    };

    // Submit exactly what's saved locally, not whatever version of it the hub may have.
    let local_path = translations_local_path()?;
    let translation = PackTranslation::load(std::slice::from_ref(&local_path), pack_name, game_key, src_lang, language)?;
    let content = fs::read_to_string(local_path.join(translation.relative_path(game_key)))?;

    let (hub_owner, hub_name) = hub_repository();
    let client = GitHubClient::new(&token)?;
    match translation.submit_to_hub(&client, &hub_owner, &hub_name, game_key, &content) {
        Ok(result) => Ok(SubmitOutcome::Submitted(result)),
        Err(RLibError::GitHubUnauthorized) => {
            sign_out()?;
            Ok(SubmitOutcome::SignInRequired)
        },
        Err(error) => Err(error.into()),
    }
}

/// Stored GitHub token, if the user is signed in.
fn token() -> Result<Option<String>> {
    match keyring_entry()?.get_password() {
        Ok(token) => Ok(Some(token)),
        Err(KeyringError::NoEntry) => Ok(None),
        Err(error) => Err(keyring_error(error)),
    }
}

/// Keyring entry holding the GitHub token.
fn keyring_entry() -> Result<Entry> {
    Entry::new(KEYRING_SERVICE, KEYRING_GITHUB_TOKEN).map_err(keyring_error)
}

/// Explain keyring errors, as they're usually about the system's keyring being unavailable.
fn keyring_error(error: KeyringError) -> anyhow::Error {
    anyhow!("Couldn't access the system's keyring, where RPFM stores your GitHub sign-in: {error}")
}

/// Owner and name of the repository translations are submitted to.
///
/// Debug builds can submit to another repository, set in `RPFM_TRANSLATION_HUB` as `owner/name`, to test without touching the real hub.
fn hub_repository() -> (String, String) {
    #[cfg(debug_assertions)]
    if let Ok(value) = std::env::var(HUB_OVERRIDE_VAR) {
        if let Some((owner, name)) = value.split_once('/') {
            return (owner.to_owned(), name.to_owned());
        }
    }

    (TRANSLATIONS_REPO_OWNER.to_owned(), TRANSLATIONS_REPO_NAME.to_owned())
}
