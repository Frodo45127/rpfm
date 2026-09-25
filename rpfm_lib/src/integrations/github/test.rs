//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Tests for the parts of the GitHub integration that don't need network access.

use super::*;

/// Every documented state of a device flow poll maps to its variant.
#[test]
fn device_flow_poll_states() {
    let parse = |value: Value| parse_device_flow_poll(&value).unwrap();

    assert_eq!(parse(json!({ "access_token": "gho_123", "token_type": "bearer", "scope": "public_repo" })), DeviceFlowPoll::Granted("gho_123".to_owned()));
    assert_eq!(parse(json!({ "error": "authorization_pending" })), DeviceFlowPoll::Pending);
    assert_eq!(parse(json!({ "error": "slow_down", "interval": 10 })), DeviceFlowPoll::SlowDown(10));
    assert_eq!(parse(json!({ "error": "slow_down" })), DeviceFlowPoll::SlowDown(SLOW_DOWN_EXTRA_SECONDS));
    assert_eq!(parse(json!({ "error": "expired_token" })), DeviceFlowPoll::Expired);
    assert_eq!(parse(json!({ "error": "access_denied" })), DeviceFlowPoll::Denied);
}

/// Unknown device flow errors are reported, with GitHub's description.
#[test]
fn device_flow_poll_unknown_error() {
    let error = parse_device_flow_poll(&json!({ "error": "incorrect_client_credentials", "error_description": "The client_id is not valid." })).unwrap_err();
    assert!(matches!(error, RLibError::GitHubApi(_, _, message) if message == "The client_id is not valid."));
}

/// Device codes deserialize from GitHub's response.
#[test]
fn device_code_from_response() {
    let code: DeviceCode = serde_json::from_value(json!({
        "device_code": "3584d83530557fdd1f46af8289938c8ef79f9dc5",
        "user_code": "WDJB-MJHT",
        "verification_uri": "https://github.com/login/device",
        "expires_in": 900,
        "interval": 5,
    })).unwrap();

    assert_eq!(code.user_code(), "WDJB-MJHT");
    assert_eq!(*code.interval(), 5);
}

/// Repositories keep the owner, default branch and push permission, which default to `false` when missing.
#[test]
fn repository_from_response() {
    let repo = parse_repository(&json!({
        "name": "total_war_translation_hub",
        "owner": { "login": "Frodo45127" },
        "default_branch": "master",
        "permissions": { "push": true },
    })).unwrap();

    assert_eq!(repo.owner(), "Frodo45127");
    assert_eq!(repo.name(), "total_war_translation_hub");
    assert_eq!(repo.default_branch(), "master");
    assert!(repo.can_push());

    let anonymous = parse_repository(&json!({ "name": "r", "owner": { "login": "o" }, "default_branch": "main" })).unwrap();
    assert!(!anonymous.can_push());
}

/// Deleted files are sent with a `null` SHA, as the Git Data API expects.
#[test]
fn tree_entries_add_and_delete() {
    let entries = tree_entries(&[
        TreeChange { path: "warhammer_3/mod.pack/EN-SP.json".to_owned(), blob: Some("abc".to_owned()) },
        TreeChange { path: "warhammer_3/mod.pack/SP.json".to_owned(), blob: None },
    ]);

    assert_eq!(entries, json!([
        { "path": "warhammer_3/mod.pack/EN-SP.json", "mode": "100644", "type": "blob", "sha": "abc" },
        { "path": "warhammer_3/mod.pack/SP.json", "mode": "100644", "type": "blob", "sha": null },
    ]));
}

/// Path segments are percent-encoded, so pack names with spaces don't break the URL.
#[test]
fn api_url_encodes_segments() {
    let url = api_url(&["repos", "owner", "repo", "contents", "warhammer_3", "my mod#1.pack"]);
    assert_eq!(url.as_str(), "https://api.github.com/repos/owner/repo/contents/warhammer_3/my%20mod%231.pack");
}
