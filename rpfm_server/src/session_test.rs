//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Tests for sessions: settings, the diagnostics queue, and the shared MCP session.

use rpfm_ipc::api::JSONRPC_VERSION;
use rpfm_ipc::api::jobs::JobState;
use rpfm_ipc::settings_keys::MYMOD_BASE_PATH;

use super::*;

#[tokio::test]
async fn configure_replaces_the_settings_of_the_session() {
    let session = Session::new(1, true);
    let mut settings = Settings::default();
    settings.set_block_write(true);
    settings.set_string(MYMOD_BASE_PATH, "/mymods").unwrap();
    let request = RpcRequest::new(7, &Configure { settings }).unwrap();

    let response = session.call(request).recv().await.unwrap();

    assert_eq!(response.id, 7);
    assert!(matches!(response.outcome, RpcOutcome::Result(_)));
    assert_eq!(session.settings().string(MYMOD_BASE_PATH), "/mymods");
}

#[tokio::test]
async fn configure_with_invalid_params_keeps_the_settings() {
    let session = Session::new(1, true);
    let before = session.settings();
    let request = RpcRequest { jsonrpc: JSONRPC_VERSION.to_owned(), id: 1, method: Configure::METHOD.to_owned(), params: serde_json::json!({ "settings": 5 }) };

    let response = session.call(request).recv().await.unwrap();

    assert!(matches!(response.outcome, RpcOutcome::Error(_)));
    assert_eq!(session.settings(), before);
}

/// Builds the request of a diagnostics check of some paths.
fn diagnostics_request(paths: &[&str]) -> RpcRequest {
    let paths = paths.iter().map(|path| path.to_string()).collect();
    RpcRequest::new(1, &RunDiagnostics { paths, ignored_types: vec![], check_assembly_kit_only_references: false }).unwrap()
}

/// Returns the paths of the request of a diagnostics check.
fn diagnostics_paths(request: RpcRequest) -> Vec<String> {
    api::parse_params::<RunDiagnostics>(request.params).unwrap().paths
}

#[tokio::test]
async fn a_new_diagnostics_check_replaces_the_queued_one() {
    let session = Session::new(1, true);
    let first = session.jobs.create(RunDiagnostics::METHOD);
    session.replace_diagnostics_check(first, diagnostics_request(&["db/a"]));

    let second = session.jobs.create(RunDiagnostics::METHOD);
    let request = session.replace_diagnostics_check(second, diagnostics_request(&["db/b", "db/a"]));

    assert_eq!(diagnostics_paths(request), vec!["db/a", "db/b"]);
    assert_eq!(session.jobs.status(first).unwrap().state, JobState::Cancelled);
    assert_eq!(session.jobs.status(second).unwrap().state, JobState::Queued);
}

#[tokio::test]
async fn a_full_diagnostics_check_covers_the_partial_ones() {
    let session = Session::new(1, true);
    let first = session.jobs.create(RunDiagnostics::METHOD);
    session.replace_diagnostics_check(first, diagnostics_request(&[]));

    let second = session.jobs.create(RunDiagnostics::METHOD);
    let request = session.replace_diagnostics_check(second, diagnostics_request(&["db/b"]));

    assert!(diagnostics_paths(request).is_empty());
}

#[tokio::test]
async fn ended_diagnostics_checks_are_not_replaced() {
    let session = Session::new(1, true);
    let first = session.jobs.create(RunDiagnostics::METHOD);
    session.replace_diagnostics_check(first, diagnostics_request(&["db/a"]));
    session.jobs.finish(first, RpcOutcome::Result(serde_json::Value::Null));

    let second = session.jobs.create(RunDiagnostics::METHOD);
    let request = session.replace_diagnostics_check(second, diagnostics_request(&["db/b"]));

    assert_eq!(diagnostics_paths(request), vec!["db/b"]);
}

#[tokio::test]
async fn stopped_diagnostics_checks_run_again_unless_replaced() {
    let session = Session::new(1, true);

    // A replaced check that stops while running is cancelled.
    let replaced = session.jobs.create(RunDiagnostics::METHOD);
    session.replace_diagnostics_check(replaced, diagnostics_request(&[]));
    assert!(session.jobs.start(replaced));
    let newer = session.jobs.create(RunDiagnostics::METHOD);
    session.replace_diagnostics_check(newer, diagnostics_request(&[]));
    session.requeue_diagnostics_check(replaced, diagnostics_request(&[]));
    assert_eq!(session.jobs.status(replaced).unwrap().state, JobState::Cancelled);

    // The last check that stops goes back to the queue, and the background thread runs it to the end.
    assert!(session.jobs.start(newer));
    session.requeue_diagnostics_check(newer, diagnostics_request(&[]));
    let status = session.jobs.wait(newer, Duration::from_secs(10)).await.unwrap();
    assert!(matches!(status.state, JobState::Finished { .. }), "{status:?}");
    assert!(!session.has_queued_requests());
}

/// Builds a manager whose sessions expire as soon as they're idle or disconnected.
fn manager_without_timeout() -> SessionManager {
    SessionManager { timeout: Duration::ZERO, ..SessionManager::default() }
}

#[tokio::test]
async fn mcp_clients_share_one_session() {
    let manager = SessionManager::default();

    let first = manager.mcp_session();
    let second = manager.mcp_session();

    assert_eq!(first.id(), second.id());
    assert_eq!(manager.session_count(), 1);
}

#[tokio::test]
async fn the_idle_mcp_session_is_removed_and_replaced() {
    let manager = manager_without_timeout();
    let first = manager.mcp_session();

    manager.cleanup_expired_sessions();
    let second = manager.mcp_session();

    assert!(first.is_shutdown_requested());
    assert_ne!(first.id(), second.id());
}

#[tokio::test]
async fn the_mcp_session_is_kept_while_it_has_open_packs() {
    let manager = manager_without_timeout();
    let session = manager.mcp_session();
    session.set_pack_name("0b4c5a62-7d3e-4f1a-9c2b-8e6d5f4a3b21", "my_mod.pack");

    manager.cleanup_expired_sessions();

    assert!(!session.is_shutdown_requested());
    assert_eq!(manager.mcp_session().id(), session.id());
}

#[tokio::test]
async fn the_mcp_session_is_kept_while_it_has_unfinished_jobs() {
    let manager = manager_without_timeout();
    let session = manager.mcp_session();
    let job = session.jobs().create(RunDiagnostics::METHOD);

    manager.cleanup_expired_sessions();
    assert!(!session.is_shutdown_requested());

    session.jobs().finish_cancelled(job);
    manager.cleanup_expired_sessions();
    assert!(session.is_shutdown_requested());
}

#[tokio::test]
async fn disconnected_websocket_sessions_expire_unless_reconnected() {
    let manager = manager_without_timeout();
    let kept = manager.create_session();
    let dropped = manager.create_session();
    manager.client_disconnected(kept.id());
    manager.client_disconnected(dropped.id());
    manager.get_or_create_session(Some(kept.id()));

    let websocket_session_removed = manager.cleanup_expired_sessions();

    assert!(websocket_session_removed);
    assert!(!kept.is_shutdown_requested());
    assert!(dropped.is_shutdown_requested());
}

#[tokio::test]
async fn removing_the_mcp_session_is_not_reported_as_a_websocket_removal() {
    let manager = manager_without_timeout();
    manager.mcp_session();

    assert!(!manager.cleanup_expired_sessions());
    assert_eq!(manager.session_count(), 0);
}
