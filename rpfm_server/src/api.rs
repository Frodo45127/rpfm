//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Dispatcher of the version 2 API, translating [`RpcRequest`]s into [`SessionState`] operations.
//!
//! See [`rpfm_ipc::api`] for the methods and their types.

use serde_json::{Map, Value};

use rpfm_ipc::api::{ApiError, Request, RpcRequest, RpcResponse};
use rpfm_ipc::api::files::ListFiles;
use rpfm_ipc::api::packs::GetPackInfo;
use rpfm_ipc::api::session::GetSessionStatus;
use rpfm_ipc::api::tables::{GetTableDefinition, GetTableInfo, GetTableRows};

use crate::state::SessionState;

/// Runs a request on the session's state.
///
/// # Returns
///
/// The response to the request.
pub fn dispatch(state: &mut SessionState, request: RpcRequest) -> RpcResponse {
    let params = request.params;
    let result = match request.method.as_str() {
        GetSessionStatus::METHOD => call(params, |_: GetSessionStatus| Ok(state.session_status())),
        GetPackInfo::METHOD => call(params, |request: GetPackInfo| state.pack_details(&request.pack)),
        ListFiles::METHOD => call(params, |request: ListFiles| state.list_files(&request)),
        GetTableInfo::METHOD => call(params, |request: GetTableInfo| state.table_info(&request.file)),
        GetTableRows::METHOD => call(params, |request: GetTableRows| state.table_rows(&request)),
        GetTableDefinition::METHOD => call(params, |request: GetTableDefinition| state.table_definition(&request)),
        method => Err(ApiError::MethodNotFound(method.to_owned())),
    };

    RpcResponse::new(request.id, result)
}

/// Deserializes the params of a request, runs its operation, and serializes its response.
fn call<R: Request>(params: Value, operation: impl FnOnce(R) -> anyhow::Result<R::Response>) -> Result<Value, ApiError> {

    // Methods without params can be called without the params field, which arrives as null.
    let params = if params.is_null() { Value::Object(Map::new()) } else { params };

    let request = serde_json::from_value::<R>(params).map_err(|error| ApiError::InvalidParams(error.to_string()))?;
    let response = operation(request).map_err(api_error)?;
    serde_json::to_value(response).map_err(|error| ApiError::Internal(error.to_string()))
}

/// Returns the [`ApiError`] an operation failed with, or wraps its message in [`ApiError::Internal`] if it's another error.
fn api_error(error: anyhow::Error) -> ApiError {
    error.downcast::<ApiError>().unwrap_or_else(|error| ApiError::Internal(error.to_string()))
}
