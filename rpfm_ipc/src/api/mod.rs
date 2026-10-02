//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! The server API: typed methods over JSON-RPC 2.0.
//!
//! Each method is a request struct implementing [`Request`], which ties it to its method name
//! and its response type. Clients send it as an [`RpcRequest`] and get back an [`RpcResponse`]
//! with either the serialized response or an [`RpcError`] built from an [`ApiError`].
//!
//! Methods are grouped by the resource they work on:
//!
//! - [`session`]: state of the session (selected game, schema, dependencies, open packs).
//! - [`packs`]: details of the open packs.
//! - [`files`]: the files of the open packs and the dependencies: listing, reading, writing and moving them.
//! - [`tables`]: reading DB and Loc tables, and their definitions.
//! - [`diagnostics`]: checking the open packs for problems.
//! - [`references`]: following references between tables and Loc files.
//! - [`search`]: searching and replacing text.
//! - [`schema`]: tables, local patches and updates of the schema.
//! - [`notes`]: notes attached to the files of the open packs.
//! - [`tools`]: the optimizer, map packing, startpos and CEO building, animations, glTF export, MyMods, plugin scripts and Lua.
//! - [`translations`]: translations of the texts of the open packs.
//! - [`github`]: signing in to GitHub, to submit translations.
//! - [`updates`]: checking for and downloading updates.
//! - [`jobs`]: following and cancelling methods that run as jobs.
//!
//! Lists are paginated: requests take an `offset` and an optional `limit`, and responses
//! include the `total` amount of items, so clients never get more than they asked for.

use schemars::JsonSchema;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;

pub mod diagnostics;
pub mod files;
pub mod github;
pub mod jobs;
pub mod notes;
pub mod packs;
pub mod references;
pub mod schema;
pub mod search;
pub mod session;
pub mod tables;
pub mod tools;
pub mod translations;
pub mod updates;

/// Version string of the JSON-RPC protocol, sent in every message.
pub const JSONRPC_VERSION: &str = "2.0";

/// Error code for messages that aren't valid JSON-RPC requests.
pub const INVALID_REQUEST_CODE: i64 = -32600;

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// A method of the API.
///
/// Implemented by the request struct of each method, so the compiler knows which response each request gets.
pub trait Request: Serialize + DeserializeOwned {

    /// Name of the method, like `table.rows`.
    const METHOD: &'static str;

    /// Response of the method.
    type Response: Serialize + DeserializeOwned;

    /// If the method runs as a job.
    ///
    /// Jobs answer right away with a [`jobs::JobStarted`], and their response arrives later as the
    /// result of the job, in [`JOB_UPDATED_NOTIFICATION`] notifications and in [`jobs::GetJobStatus`].
    const IS_JOB: bool = false;
}

/// Method of the notification sent to clients whenever a job changes state. Its params are a [`jobs::JobStatus`].
pub const JOB_UPDATED_NOTIFICATION: &str = "job.updated";

/// Response of the methods that return nothing.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct Done {}

/// A JSON-RPC request.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RpcRequest {

    /// Always [`JSONRPC_VERSION`].
    pub jsonrpc: String,

    /// ID the response is sent back with.
    pub id: u64,

    /// Name of the method to call.
    pub method: String,

    /// Parameters of the method: its request struct, serialized.
    #[serde(default)]
    pub params: Value,
}

/// A JSON-RPC response.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RpcResponse {

    /// Always [`JSONRPC_VERSION`].
    pub jsonrpc: String,

    /// ID of the request this responds to.
    pub id: u64,

    /// Result of the request.
    #[serde(flatten)]
    pub outcome: RpcOutcome,
}

/// Result of a JSON-RPC request. Serialized as a `result` or an `error` field of the response.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RpcOutcome {

    /// The response of the method, serialized.
    Result(Value),

    /// Why the request failed.
    Error(RpcError),
}

/// A JSON-RPC notification: a message from the server that isn't a response to a request.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RpcNotification {

    /// Always [`JSONRPC_VERSION`].
    pub jsonrpc: String,

    /// What the notification is about, like [`JOB_UPDATED_NOTIFICATION`].
    pub method: String,

    /// Data of the notification.
    pub params: Value,
}

/// A JSON-RPC error.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct RpcError {

    /// Numeric code of the error. See [`ApiError::code`].
    pub code: i64,

    /// Human-readable description of the error.
    pub message: String,

    /// The [`ApiError`] this was built from, serialized, so clients can match on it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

/// Errors returned by the API.
#[derive(Debug, Clone, PartialEq, Eq, Error, Serialize, Deserialize)]
#[serde(tag = "kind", content = "details", rename_all = "snake_case")]
pub enum ApiError {

    /// There is no open pack with this key.
    #[error("Pack not found: {0}")]
    PackNotFound(String),

    /// There is no file at this path in its source.
    #[error("File not found: {0}")]
    FileNotFound(String),

    /// The file at this path is not a DB or Loc table.
    #[error("The file {0} is not a table.")]
    NotATable(String),

    /// The operation needs a schema, and there is none loaded for the selected game.
    #[error("There is no Schema for the Game Selected.")]
    SchemaNotLoaded,

    /// The schema has no definition for this table.
    #[error("No definition found for table {0}.")]
    DefinitionNotFound(String),

    /// The file can't be edited, because it's not in an open pack.
    #[error("The file {0} can't be edited: only files in open packs can.")]
    ReadOnly(String),

    /// The operation needs the results of a diagnostics check, and there are none.
    #[error("There are no diagnostics results. Run diagnostics.run first.")]
    DiagnosticsNotRun,

    /// The operation needs the matches of a search, and there are none.
    #[error("There are no search matches. Run search.run first.")]
    SearchNotRun,

    /// The searched value wasn't found.
    #[error("{0} not found.")]
    NotFound(String),

    /// There is no job with this ID, or it ended long ago and was forgotten.
    #[error("There is no job with ID {0}.")]
    JobNotFound(u64),

    /// The operation needs the vanilla dependencies, and they're not loaded.
    #[error("Dependencies cache needs to be regenerated before this.")]
    DependenciesNotLoaded,

    /// There is no method with this name.
    #[error("Method not found: {0}")]
    MethodNotFound(String),

    /// The parameters of the request are not valid for its method.
    #[error("Invalid params: {0}")]
    InvalidParams(String),

    /// Any other error, with its message.
    #[error("{0}")]
    Internal(String),
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

impl RpcRequest {

    /// Builds the JSON-RPC request of a method call.
    ///
    /// # Arguments
    ///
    /// * `id` - ID the response will be sent back with.
    /// * `request` - The method call.
    pub fn new<R: Request>(id: u64, request: &R) -> Result<Self, serde_json::Error> {
        Ok(Self {
            jsonrpc: JSONRPC_VERSION.to_owned(),
            id,
            method: R::METHOD.to_owned(),
            params: serde_json::to_value(request)?,
        })
    }
}

impl RpcResponse {

    /// Builds the JSON-RPC response of a request.
    ///
    /// # Arguments
    ///
    /// * `id` - ID of the request this responds to.
    /// * `result` - The serialized response of the method, or why it failed.
    pub fn new(id: u64, result: Result<Value, ApiError>) -> Self {
        Self {
            jsonrpc: JSONRPC_VERSION.to_owned(),
            id,
            outcome: match result {
                Ok(value) => RpcOutcome::Result(value),
                Err(error) => RpcOutcome::Error(RpcError::from(error)),
            },
        }
    }

    /// Builds the response to a message that isn't a valid JSON-RPC request.
    ///
    /// # Arguments
    ///
    /// * `id` - ID of the request, or 0 if it couldn't be read.
    /// * `message` - Why the request is not valid.
    pub fn invalid_request(id: u64, message: String) -> Self {
        Self {
            jsonrpc: JSONRPC_VERSION.to_owned(),
            id,
            outcome: RpcOutcome::Error(RpcError { code: INVALID_REQUEST_CODE, message, data: None }),
        }
    }

    /// Returns the typed response of a method, or the error the request failed with.
    pub fn into_response<R: Request>(self) -> Result<R::Response, ApiError> {
        match self.outcome {
            RpcOutcome::Result(value) => serde_json::from_value(value).map_err(|error| ApiError::Internal(format!("Invalid response: {error}"))),
            RpcOutcome::Error(error) => Err(ApiError::from(error)),
        }
    }
}

impl RpcNotification {

    /// Builds a notification.
    pub fn new(method: &str, params: Value) -> Self {
        Self {
            jsonrpc: JSONRPC_VERSION.to_owned(),
            method: method.to_owned(),
            params,
        }
    }
}

impl ApiError {

    /// Returns the JSON-RPC code of the error: the standard ones for protocol errors,
    /// and codes in the `-32000..-32099` range, reserved for servers, for the rest.
    pub fn code(&self) -> i64 {
        match self {
            Self::MethodNotFound(_) => -32601,
            Self::InvalidParams(_) => -32602,
            Self::Internal(_) => -32603,
            Self::PackNotFound(_) => -32001,
            Self::FileNotFound(_) => -32002,
            Self::NotATable(_) => -32003,
            Self::SchemaNotLoaded => -32004,
            Self::DependenciesNotLoaded => -32005,
            Self::DefinitionNotFound(_) => -32006,
            Self::ReadOnly(_) => -32007,
            Self::JobNotFound(_) => -32008,
            Self::DiagnosticsNotRun => -32009,
            Self::SearchNotRun => -32010,
            Self::NotFound(_) => -32011,
        }
    }
}

impl From<ApiError> for RpcError {
    fn from(error: ApiError) -> Self {
        Self {
            code: error.code(),
            message: error.to_string(),
            data: serde_json::to_value(&error).ok(),
        }
    }
}

impl From<RpcError> for ApiError {
    fn from(error: RpcError) -> Self {
        error.data
            .and_then(|data| serde_json::from_value(data).ok())
            .unwrap_or(Self::Internal(error.message))
    }
}

/// Default value of fields that are `true` unless set.
fn default_true() -> bool {
    true
}

//-------------------------------------------------------------------------------//
//                                   Tests
//-------------------------------------------------------------------------------//

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;
    use super::session::GetSessionStatus;

    #[test]
    fn request_is_serialized_as_jsonrpc() {
        let request = RpcRequest::new(7, &GetSessionStatus {}).unwrap();

        assert_eq!(serde_json::to_value(&request).unwrap(), json!({"jsonrpc": "2.0", "id": 7, "method": "session.status", "params": {}}));
    }

    #[test]
    fn response_result_is_serialized_as_result_field() {
        let response = RpcResponse::new(3, Ok(json!({"a": 1})));

        assert_eq!(serde_json::to_value(&response).unwrap(), json!({"jsonrpc": "2.0", "id": 3, "result": {"a": 1}}));
    }

    #[test]
    fn response_error_round_trips_to_the_same_api_error() {
        let response = RpcResponse::new(4, Err(ApiError::PackNotFound("my_pack.pack".to_owned())));
        let json = serde_json::to_value(&response).unwrap();

        assert_eq!(json["error"]["code"], json!(-32001));
        assert_eq!(json["error"]["message"], json!("Pack not found: my_pack.pack"));

        let parsed: RpcResponse = serde_json::from_value(json).unwrap();
        assert_eq!(parsed.into_response::<GetSessionStatus>(), Err(ApiError::PackNotFound("my_pack.pack".to_owned())));
    }

    #[test]
    fn error_without_data_becomes_internal() {
        let error = RpcError { code: -32603, message: "boom".to_owned(), data: None };

        assert_eq!(ApiError::from(error), ApiError::Internal("boom".to_owned()));
    }
}
