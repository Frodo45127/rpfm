//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Tests for the session module.

use std::sync::Arc;

use super::session::*;

#[tokio::test]
async fn test_get_mcp_session_returns_created_session() {
    let manager = SessionManager::default();
    let session = manager.create_mcp_session();

    let found = manager.get_mcp_session(session.id());

    assert!(found.is_some_and(|found| Arc::ptr_eq(&found, &session)));
}

#[tokio::test]
async fn test_get_mcp_session_ignores_websocket_sessions() {
    let manager = SessionManager::default();
    let session = manager.create_session();

    assert!(manager.get_mcp_session(session.id()).is_none());
}

#[tokio::test]
async fn test_get_mcp_session_ignores_removed_sessions() {
    let manager = SessionManager::default();
    let session = manager.create_mcp_session();

    manager.remove_session(session.id());

    assert!(manager.get_mcp_session(session.id()).is_none());
}

#[tokio::test]
async fn test_mcp_sessions_are_isolated() {
    let manager = SessionManager::default();

    let first = manager.create_mcp_session();
    let second = manager.create_mcp_session();

    assert_ne!(first.id(), second.id());
}

#[tokio::test]
async fn test_mcp_global_session_is_reused_and_unlisted() {
    let manager = SessionManager::default();

    let first = manager.mcp_global_session();
    let second = manager.mcp_global_session();

    assert!(Arc::ptr_eq(&first, &second));
    assert_eq!(manager.session_count(), 0);
    assert!(manager.get_mcp_session(first.id()).is_none());
}
