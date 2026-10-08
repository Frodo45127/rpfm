//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Tests for the JSON-RPC messages of the API, its errors, and how requests reject unknown params.

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

#[test]
fn missing_tables_without_the_suffix_get_a_hint() {
    assert_eq!(ApiError::DefinitionNotFound("units_tables".to_owned()).to_string(), "No definition found for table units_tables.");
    assert!(ApiError::DefinitionNotFound("units".to_owned()).to_string().ends_with("Table names end in `_tables`, like `units_tables`."));
}

#[test]
fn requests_reject_unknown_params() {
    let error = serde_json::from_value::<tables::GetTableRows>(json!({
        "file": { "source": "game_files", "path": "db/units_tables/data" },
        "filter": [],
    })).unwrap_err();

    assert!(error.to_string().contains("unknown field `filter`"));
}

#[test]
fn nested_params_reject_unknown_fields() {
    let error = serde_json::from_value::<files::FileRef>(json!({ "source": "game_files", "path": "a", "paht": "b" })).unwrap_err();

    assert!(error.to_string().contains("unknown field `paht`"));
}

#[test]
fn tagged_params_reject_unknown_fields_but_accept_their_tag() {
    let edit = serde_json::from_value::<tables::RowEdit>(json!({ "op": "delete", "indexes": [1] })).unwrap();
    let error = serde_json::from_value::<tables::RowEdit>(json!({ "op": "delete", "indexes": [1], "index": 2 })).unwrap_err();

    assert_eq!(edit, tables::RowEdit::Delete { indexes: vec![1] });
    assert!(error.to_string().contains("unknown field `index`"));
}
