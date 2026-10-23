//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Tests for serializing responses for MCP clients.

use rpfm_ipc::helpers::{DependenciesInfo, RFileInfo};
use rpfm_ipc::messages::Response;
use rpfm_lib::compression::CompressionFormat;

use super::server_mcp::response_to_json;

fn sample_info() -> DependenciesInfo {
    DependenciesInfo {
        asskit_tables: vec![RFileInfo::default(); 2],
        vanilla_packed_files: vec![RFileInfo::default(); 3],
        parent_packed_files: vec![RFileInfo::default(); 1],
    }
}

/// Dependencies info sent after a game change is replaced by its file counts.
#[test]
fn compression_format_dependencies_info_is_summarized() {
    let response = Response::CompressionFormatDependenciesInfo(CompressionFormat::Lz4, Some(sample_info()));

    let json = response_to_json(&response).unwrap();

    assert_eq!(json, r#"{"CompressionFormatDependenciesInfo":["Lz4",{"asskit_tables_count":2,"vanilla_packed_files_count":3,"parent_packed_files_count":1}]}"#);
}

/// A game change without a dependencies rebuild keeps its `null` info.
#[test]
fn compression_format_without_dependencies_info_is_kept() {
    let response = Response::CompressionFormatDependenciesInfo(CompressionFormat::None, None);

    let json = response_to_json(&response).unwrap();

    assert_eq!(json, serde_json::to_string(&response).unwrap());
}

/// Dependencies info sent after a rebuild is replaced by its file counts.
#[test]
fn dependencies_info_is_summarized() {
    let response = Response::DependenciesInfo(sample_info());

    let json = response_to_json(&response).unwrap();

    assert_eq!(json, r#"{"DependenciesInfo":{"asskit_tables_count":2,"vanilla_packed_files_count":3,"parent_packed_files_count":1}}"#);
}

/// Other responses serialize unchanged.
#[test]
fn other_responses_are_unchanged() {
    let response = Response::String("warhammer_3".to_owned());

    let json = response_to_json(&response).unwrap();

    assert_eq!(json, serde_json::to_string(&response).unwrap());
}
