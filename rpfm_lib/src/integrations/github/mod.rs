//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! GitHub integration.
//!
//! A small blocking client for the GitHub REST API, covering what's needed to submit files to a
//! repository as pull requests:
//!
//! - Signing in with the OAuth device flow, which needs no client secret.
//! - Forking a repository and keeping the fork in sync.
//! - Committing files through the Git Data API, which accepts large files as plain UTF-8.
//! - Finding and opening pull requests.

use getset::Getters;
use reqwest::{StatusCode, Url};
use reqwest::blocking::{Client, RequestBuilder, Response};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use std::time::Duration;

use crate::error::{RLibError, Result};

#[cfg(test)] mod test;

/// Base URL of the GitHub REST API.
const API_URL: &str = "https://api.github.com";

/// Endpoint to start a device flow sign-in.
const DEVICE_CODE_URL: &str = "https://github.com/login/device/code";

/// Endpoint to poll for the token of a device flow sign-in.
const ACCESS_TOKEN_URL: &str = "https://github.com/login/oauth/access_token";

/// Grant type for polling a device flow sign-in.
const DEVICE_GRANT_TYPE: &str = "urn:ietf:params:oauth:grant-type:device_code";

/// GitHub rejects API requests without a user agent.
const USER_AGENT: &str = "rpfm";

/// Seconds added to the polling interval when GitHub asks to slow down without saying by how much.
const SLOW_DOWN_EXTRA_SECONDS: u64 = 5;

/// Scope needed to fork public repositories and open pull requests on them.
pub const PUBLIC_REPO_SCOPE: &str = "public_repo";

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// Codes of a device flow sign-in, to show to the user and to poll for its token.
#[derive(Clone, Debug, PartialEq, Eq, Getters, Serialize, Deserialize)]
#[getset(get = "pub")]
pub struct DeviceCode {

    /// Code used to poll for the token. Not meant for the user.
    device_code: String,

    /// Code the user has to enter on GitHub.
    user_code: String,

    /// Page where the user enters the code.
    verification_uri: String,

    /// Seconds until the codes expire.
    expires_in: u64,

    /// Minimum seconds between polls.
    interval: u64,
}

/// State of a device flow sign-in, as returned when polling for its token.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DeviceFlowPoll {

    /// The user hasn't approved the sign-in yet.
    Pending,

    /// Polling too fast. Contains the new minimum seconds between polls.
    SlowDown(u64),

    /// The user approved the sign-in. Contains the token.
    Granted(String),

    /// The codes expired before the user approved the sign-in.
    Expired,

    /// The user rejected the sign-in.
    Denied,
}

/// A GitHub repository, as far as submitting pull requests is concerned.
#[derive(Clone, Debug, PartialEq, Eq, Getters)]
#[getset(get = "pub")]
pub struct Repository {

    /// Login of the repository's owner.
    owner: String,

    /// Name of the repository.
    name: String,

    /// Default branch of the repository.
    default_branch: String,

    /// Whether the signed-in user can push to the repository.
    can_push: bool,
}

/// A change to a file, to commit with [`GitHubClient::create_tree`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TreeChange {

    /// Path of the file in the repository, with `/` separators.
    pub path: String,

    /// Blob with the file's new contents, or `None` to delete the file.
    pub blob: Option<String>,
}

/// Data to open a pull request with [`GitHubClient::create_pull_request`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NewPullRequest {

    /// Title of the pull request.
    pub title: String,

    /// Description of the pull request, in Markdown.
    pub body: String,

    /// Branch with the changes, as `owner:branch` when it's in a fork.
    pub head: String,

    /// Branch the changes are meant to be merged into.
    pub base: String,
}

/// A pull request.
#[derive(Clone, Debug, PartialEq, Eq, Getters, Deserialize)]
#[getset(get = "pub")]
pub struct PullRequest {

    /// Number of the pull request in its repository.
    number: u64,

    /// Web page of the pull request.
    html_url: String,
}

/// Client for the GitHub REST API, authenticated as a user.
#[derive(Debug)]
pub struct GitHubClient {

    /// HTTP client, with long timeouts so large files can be uploaded.
    client: Client,

    /// Token of the signed-in user.
    token: String,
}

//-------------------------------------------------------------------------------//
//                        Device flow sign-in
//-------------------------------------------------------------------------------//

/// Start a device flow sign-in.
///
/// # Arguments
///
/// * `client_id` - Client ID of the OAuth App signing in.
/// * `scope` - Scopes requested, space-separated.
///
/// # Returns
///
/// The codes to show to the user and to poll for the token.
///
/// # Errors
///
/// Returns an error if the request fails, or if GitHub rejects it (e.g. the OAuth App doesn't have the device flow enabled).
pub fn request_device_code(client_id: &str, scope: &str) -> Result<DeviceCode> {
    let response = http_client()?
        .post(DEVICE_CODE_URL)
        .header("Accept", "application/json")
        .json(&json!({ "client_id": client_id, "scope": scope }))
        .send()?;

    let value = checked_response(response, DEVICE_CODE_URL)?.json::<Value>()?;
    if let Some(error) = value.get("error") {
        return Err(RLibError::GitHubApi(DEVICE_CODE_URL.to_owned(), StatusCode::OK.as_u16(), error_description(&value, error)));
    }

    serde_json::from_value(value).map_err(From::from)
}

/// Poll for the token of a device flow sign-in.
///
/// # Arguments
///
/// * `client_id` - Client ID of the OAuth App signing in.
/// * `device_code` - Device code returned by [`request_device_code`].
///
/// # Returns
///
/// The state of the sign-in, including the token once the user approves it.
///
/// # Errors
///
/// Returns an error if the request fails, or GitHub returns an unexpected error.
pub fn poll_device_flow(client_id: &str, device_code: &str) -> Result<DeviceFlowPoll> {
    let response = http_client()?
        .post(ACCESS_TOKEN_URL)
        .header("Accept", "application/json")
        .json(&json!({ "client_id": client_id, "device_code": device_code, "grant_type": DEVICE_GRANT_TYPE }))
        .send()?;

    let value = checked_response(response, ACCESS_TOKEN_URL)?.json::<Value>()?;
    parse_device_flow_poll(&value)
}

/// Turn a response of the device flow token endpoint into the state of the sign-in.
fn parse_device_flow_poll(value: &Value) -> Result<DeviceFlowPoll> {
    if let Some(token) = value.get("access_token").and_then(Value::as_str) {
        return Ok(DeviceFlowPoll::Granted(token.to_owned()));
    }

    let error = value.get("error").cloned().unwrap_or(Value::Null);
    match error.as_str() {
        Some("authorization_pending") => Ok(DeviceFlowPoll::Pending),
        Some("slow_down") => {
            let interval = value.get("interval").and_then(Value::as_u64).unwrap_or(SLOW_DOWN_EXTRA_SECONDS);
            Ok(DeviceFlowPoll::SlowDown(interval))
        },
        Some("expired_token") | Some("token_expired") => Ok(DeviceFlowPoll::Expired),
        Some("access_denied") => Ok(DeviceFlowPoll::Denied),
        _ => Err(RLibError::GitHubApi(ACCESS_TOKEN_URL.to_owned(), StatusCode::OK.as_u16(), error_description(value, &error))),
    }
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

impl GitHubClient {

    /// Create a client authenticated with the provided token.
    ///
    /// # Errors
    ///
    /// Returns an error if the HTTP client can't be built.
    pub fn new(token: &str) -> Result<Self> {
        Ok(Self {
            client: http_client()?,
            token: token.to_owned(),
        })
    }

    /// Login of the signed-in user.
    pub fn user_login(&self) -> Result<String> {
        let value: Value = self.send_json(self.request(reqwest::Method::GET, &["user"]))?;
        value.get("login")
            .and_then(Value::as_str)
            .map(str::to_owned)
            .ok_or_else(|| RLibError::GitHubApi("user".to_owned(), StatusCode::OK.as_u16(), "Missing login.".to_owned()))
    }

    /// Get a repository.
    ///
    /// # Returns
    ///
    /// The repository, or `None` if it doesn't exist or the user can't see it.
    pub fn repository(&self, owner: &str, repo: &str) -> Result<Option<Repository>> {
        let request = self.request(reqwest::Method::GET, &["repos", owner, repo]);
        match self.send_json::<Value>(request) {
            Ok(value) => parse_repository(&value).map(Some),
            Err(RLibError::GitHubApi(_, 404, _)) => Ok(None),
            Err(error) => Err(error),
        }
    }

    /// Fork a repository into the signed-in user's account. Returns the existing fork if there's one already.
    ///
    /// Forks are created asynchronously, so the fork may take a moment before it's usable.
    pub fn fork(&self, owner: &str, repo: &str) -> Result<Repository> {
        let request = self.request(reqwest::Method::POST, &["repos", owner, repo, "forks"])
            .json(&json!({ "default_branch_only": true }));
        parse_repository(&self.send_json(request)?)
    }

    /// Bring a fork's branch up to date with the same branch in the repository it was forked from.
    ///
    /// # Errors
    ///
    /// Returns an error if the branch can't be updated, for example because it has changes the upstream branch doesn't have.
    pub fn merge_upstream(&self, owner: &str, repo: &str, branch: &str) -> Result<()> {
        let request = self.request(reqwest::Method::POST, &["repos", owner, repo, "merge-upstream"])
            .json(&json!({ "branch": branch }));
        self.send(request).map(|_| ())
    }

    /// Commit a branch points to.
    ///
    /// # Returns
    ///
    /// The commit's SHA, or `None` if the branch doesn't exist.
    pub fn branch_head(&self, owner: &str, repo: &str, branch: &str) -> Result<Option<String>> {
        let mut path = vec!["repos", owner, repo, "git", "ref", "heads"];
        path.extend(branch.split('/'));

        match self.send_json::<Value>(self.request(reqwest::Method::GET, &path)) {
            Ok(value) => Ok(value.pointer("/object/sha").and_then(Value::as_str).map(str::to_owned)),
            Err(RLibError::GitHubApi(_, 404, _)) => Ok(None),
            Err(error) => Err(error),
        }
    }

    /// Tree of a commit.
    ///
    /// # Returns
    ///
    /// The SHA of the commit's tree.
    pub fn commit_tree(&self, owner: &str, repo: &str, commit: &str) -> Result<String> {
        let value: Value = self.send_json(self.request(reqwest::Method::GET, &["repos", owner, repo, "git", "commits", commit]))?;
        value.pointer("/tree/sha")
            .and_then(Value::as_str)
            .map(str::to_owned)
            .ok_or_else(|| RLibError::GitHubApi("git/commits".to_owned(), StatusCode::OK.as_u16(), "Missing tree.".to_owned()))
    }

    /// Names of the entries of a folder in a repository.
    ///
    /// # Arguments
    ///
    /// * `owner` - Owner of the repository.
    /// * `repo` - Name of the repository.
    /// * `path` - Path of the folder, with `/` separators.
    /// * `reference` - Branch or commit to read the folder from.
    ///
    /// # Returns
    ///
    /// The names of the files and folders in it, or an empty list if the folder doesn't exist.
    pub fn folder_entries(&self, owner: &str, repo: &str, path: &str, reference: &str) -> Result<Vec<String>> {
        let mut segments = vec!["repos", owner, repo, "contents"];
        segments.extend(path.split('/'));
        let mut url = api_url(&segments);
        url.query_pairs_mut().append_pair("ref", reference);
        let request = self.request_url(reqwest::Method::GET, url);

        match self.send_json::<Value>(request) {
            Ok(Value::Array(entries)) => Ok(entries.iter()
                .filter_map(|entry| entry.get("name").and_then(Value::as_str).map(str::to_owned))
                .collect()),
            Ok(_) => Ok(vec![]),
            Err(RLibError::GitHubApi(_, 404, _)) => Ok(vec![]),
            Err(error) => Err(error),
        }
    }

    /// Upload a file's contents.
    ///
    /// # Returns
    ///
    /// The SHA of the new blob.
    pub fn create_blob(&self, owner: &str, repo: &str, content: &str) -> Result<String> {
        let request = self.request(reqwest::Method::POST, &["repos", owner, repo, "git", "blobs"])
            .json(&json!({ "content": content, "encoding": "utf-8" }));
        sha_from(&self.send_json(request)?, "git/blobs")
    }

    /// Create a tree applying file changes on top of another tree.
    ///
    /// # Arguments
    ///
    /// * `owner` - Owner of the repository.
    /// * `repo` - Name of the repository.
    /// * `base_tree` - Tree the changes are applied to.
    /// * `changes` - Files to add, replace or delete.
    ///
    /// # Returns
    ///
    /// The SHA of the new tree.
    pub fn create_tree(&self, owner: &str, repo: &str, base_tree: &str, changes: &[TreeChange]) -> Result<String> {
        let request = self.request(reqwest::Method::POST, &["repos", owner, repo, "git", "trees"])
            .json(&json!({ "base_tree": base_tree, "tree": tree_entries(changes) }));
        sha_from(&self.send_json(request)?, "git/trees")
    }

    /// Create a commit.
    ///
    /// # Arguments
    ///
    /// * `owner` - Owner of the repository.
    /// * `repo` - Name of the repository.
    /// * `message` - Commit message.
    /// * `tree` - Tree of the commit.
    /// * `parent` - Parent commit.
    ///
    /// # Returns
    ///
    /// The SHA of the new commit.
    pub fn create_commit(&self, owner: &str, repo: &str, message: &str, tree: &str, parent: &str) -> Result<String> {
        let request = self.request(reqwest::Method::POST, &["repos", owner, repo, "git", "commits"])
            .json(&json!({ "message": message, "tree": tree, "parents": [parent] }));
        sha_from(&self.send_json(request)?, "git/commits")
    }

    /// Point a branch to a commit, creating the branch if it doesn't exist, and replacing whatever it had otherwise.
    pub fn set_branch(&self, owner: &str, repo: &str, branch: &str, commit: &str) -> Result<()> {
        let request = if self.branch_head(owner, repo, branch)?.is_some() {
            let mut path = vec!["repos", owner, repo, "git", "refs", "heads"];
            path.extend(branch.split('/'));
            self.request(reqwest::Method::PATCH, &path).json(&json!({ "sha": commit, "force": true }))
        } else {
            self.request(reqwest::Method::POST, &["repos", owner, repo, "git", "refs"])
                .json(&json!({ "ref": format!("refs/heads/{branch}"), "sha": commit }))
        };

        self.send(request).map(|_| ())
    }

    /// Find an open pull request from a branch.
    ///
    /// # Arguments
    ///
    /// * `owner` - Owner of the repository the pull request is in.
    /// * `repo` - Name of the repository the pull request is in.
    /// * `head` - Branch with the changes, as `owner:branch`.
    /// * `base` - Branch the changes are meant to be merged into.
    ///
    /// # Returns
    ///
    /// The pull request, or `None` if there's no open one from that branch.
    pub fn open_pull_request(&self, owner: &str, repo: &str, head: &str, base: &str) -> Result<Option<PullRequest>> {
        let mut url = api_url(&["repos", owner, repo, "pulls"]);
        url.query_pairs_mut().extend_pairs([("state", "open"), ("head", head), ("base", base)]);
        let request = self.request_url(reqwest::Method::GET, url);
        let pulls: Vec<PullRequest> = self.send_json(request)?;
        Ok(pulls.into_iter().next())
    }

    /// Open a pull request.
    pub fn create_pull_request(&self, owner: &str, repo: &str, pull: &NewPullRequest) -> Result<PullRequest> {
        let request = self.request(reqwest::Method::POST, &["repos", owner, repo, "pulls"])
            .json(&json!({ "title": pull.title, "body": pull.body, "head": pull.head, "base": pull.base }));
        self.send_json(request)
    }

    /// Build an authenticated request to an API endpoint.
    fn request(&self, method: reqwest::Method, segments: &[&str]) -> RequestBuilder {
        self.request_url(method, api_url(segments))
    }

    /// Build an authenticated request to an API URL, for endpoints that need query parameters.
    fn request_url(&self, method: reqwest::Method, url: Url) -> RequestBuilder {
        self.client.request(method, url)
            .bearer_auth(&self.token)
            .header("Accept", "application/vnd.github+json")
            .header("X-GitHub-Api-Version", "2022-11-28")
    }

    /// Send a request, turning unsuccessful responses into errors.
    fn send(&self, request: RequestBuilder) -> Result<Response> {
        let response = request.send()?;
        let endpoint = response.url().path().to_owned();
        checked_response(response, &endpoint)
    }

    /// Send a request and deserialize its JSON response.
    fn send_json<T: DeserializeOwned>(&self, request: RequestBuilder) -> Result<T> {
        self.send(request)?.json::<T>().map_err(From::from)
    }
}

//-------------------------------------------------------------------------------//
//                                 Helpers
//-------------------------------------------------------------------------------//

/// HTTP client shared by all requests. Timeouts are long because translation files can be tens of MB.
fn http_client() -> Result<Client> {
    Client::builder()
        .user_agent(USER_AGENT)
        .connect_timeout(Duration::from_secs(30))
        .timeout(Duration::from_secs(600))
        .build()
        .map_err(From::from)
}

/// URL of an API endpoint. Each segment is percent-encoded on its own, so names with spaces or other special characters are safe.
fn api_url(segments: &[&str]) -> Url {
    let mut url = Url::parse(API_URL).expect("API_URL must be a valid URL");
    url.path_segments_mut()
        .expect("API_URL must be a base URL")
        .extend(segments);
    url
}

/// Turn an unsuccessful response into an error, keeping GitHub's message.
fn checked_response(response: Response, endpoint: &str) -> Result<Response> {
    let status = response.status();
    if status.is_success() {
        return Ok(response);
    }

    if status == StatusCode::UNAUTHORIZED {
        return Err(RLibError::GitHubUnauthorized);
    }

    let text = response.text().unwrap_or_default();
    let message = serde_json::from_str::<Value>(&text).ok()
        .and_then(|value| value.get("message").and_then(Value::as_str).map(str::to_owned))
        .unwrap_or(text);

    Err(RLibError::GitHubApi(endpoint.to_owned(), status.as_u16(), message))
}

/// Human-readable description of an OAuth error response.
fn error_description(value: &Value, error: &Value) -> String {
    value.get("error_description")
        .and_then(Value::as_str)
        .map(str::to_owned)
        .unwrap_or_else(|| error.to_string())
}

/// Parse a repository from its API representation.
fn parse_repository(value: &Value) -> Result<Repository> {
    let field = |pointer: &str| value.pointer(pointer).and_then(Value::as_str).map(str::to_owned)
        .ok_or_else(|| RLibError::GitHubApi("repos".to_owned(), StatusCode::OK.as_u16(), format!("Missing {pointer} in repository.")));

    Ok(Repository {
        owner: field("/owner/login")?,
        name: field("/name")?,
        default_branch: field("/default_branch")?,
        can_push: value.pointer("/permissions/push").and_then(Value::as_bool).unwrap_or(false),
    })
}

/// Entries of a tree creation request for the provided changes. A `null` SHA deletes the file.
fn tree_entries(changes: &[TreeChange]) -> Value {
    Value::Array(changes.iter()
        .map(|change| json!({ "path": change.path, "mode": "100644", "type": "blob", "sha": change.blob }))
        .collect())
}

/// SHA of a created Git object.
fn sha_from(value: &Value, endpoint: &str) -> Result<String> {
    value.get("sha")
        .and_then(Value::as_str)
        .map(str::to_owned)
        .ok_or_else(|| RLibError::GitHubApi(endpoint.to_owned(), StatusCode::OK.as_u16(), "Missing sha.".to_owned()))
}
