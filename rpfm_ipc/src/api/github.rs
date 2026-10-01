//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Methods to sign in to GitHub, needed to submit translations to the Translation Hub.
//!
//! The sign-in uses GitHub's device flow: the user enters a code on GitHub while the client polls for the result.
//! The token is kept by the server in the OS keyring, and never sent to clients.

use serde::{Deserialize, Serialize};

use rpfm_lib::integrations::github::DeviceCode;

use super::{Done, Request};

/// `github.sign_in_start`: starts a GitHub sign-in, returning the code the user has to enter on GitHub.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct StartGitHubSignIn {}

/// `github.sign_in_poll`: checks if the user approved a sign-in. Once approved, the server keeps the sign-in.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PollGitHubSignIn {

    /// Device code of the sign-in, from `github.sign_in_start`.
    pub device_code: String,
}

/// State of a GitHub sign-in.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum GitHubSignInState {

    /// The user hasn't approved the sign-in yet.
    Pending,

    /// Polling too fast. Contains the new minimum seconds between polls.
    SlowDown(u64),

    /// Signed in. Contains the account's login.
    SignedIn(String),

    /// The code expired before the user approved it.
    Expired,

    /// The user rejected the sign-in.
    Denied,
}

/// `github.account`: returns the GitHub account the user is signed in as.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct GetGitHubAccount {}

/// GitHub account the user is signed in as.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct GitHubAccount {

    /// Login of the account, or `None` if the user isn't signed in.
    pub login: Option<String>,
}

/// `github.sign_out`: signs out of GitHub, deleting the stored sign-in.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SignOutOfGitHub {}

impl Request for StartGitHubSignIn {
    const METHOD: &'static str = "github.sign_in_start";
    type Response = DeviceCode;
}

impl Request for PollGitHubSignIn {
    const METHOD: &'static str = "github.sign_in_poll";
    type Response = GitHubSignInState;
}

impl Request for GetGitHubAccount {
    const METHOD: &'static str = "github.account";
    type Response = GitHubAccount;
}

impl Request for SignOutOfGitHub {
    const METHOD: &'static str = "github.sign_out";
    type Response = Done;
}
