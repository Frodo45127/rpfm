//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! # `rpfm_server`
//!
//! Backend process for [Rusted PackFile Manager][rpfm]. Hosts the heavy work
//! that the Qt6 UI ([`rpfm_ui`][ui]) and AI / MCP clients drive remotely:
//! Pack I/O, schema decoding, diagnostics, search, dependencies, optimisation
//! and so on.
//!
//! [rpfm]: https://github.com/Frodo45127/rpfm
//! [ui]: https://crates.io/crates/rpfm_ui
//!
//! ## Architecture
//!
//! The server is built on [`axum`] (HTTP + WebSocket) and [`tokio`]. It binds
//! to `127.0.0.1:45127` by default and exposes three endpoints:
//!
//! | Endpoint    | Method | Purpose                                                                          |
//! |-------------|--------|----------------------------------------------------------------------------------|
//! | `/ws`       | GET    | WebSocket upgrade. Carries the [`rpfm_ipc`] command/response protocol.           |
//! | `/sessions` | GET    | REST: list every active session (used by the UI session picker).                 |
//! | `/mcp`      | *      | MCP `StreamableHttpService` exposing the same surface to AI / MCP clients.       |
//!
//! Every client connection is wrapped in a [`session::Session`] managed by a
//! [`session::SessionManager`]. Each session owns a dedicated background
//! thread (see [`background_thread`]) that processes commands serially against
//! its own in-memory state (open packs, dependency cache, settings cache),
//! so multiple concurrent clients can't step on each other.
//!
//! ## Modules
//!
//! - [`background_thread`] — central command dispatcher; one async loop per session.
//! - [`comms`] — generic mpsc-based request/response abstraction used to talk
//!   to the background thread.
//! - [`server_websocket`] — `/ws` upgrade handler and message multiplexer.
//! - [`server_mcp`] — `/mcp` endpoint: tools, prompts, resources for MCP clients.
//! - [`session`] — `SessionManager`, `Session`, lifecycle and timeout handling.
//! - [`settings`] — JSON-backed settings store with batch-write optimisation.
//! - [`updater`] — self-update checks against GitHub releases.
//!
//! ## Telemetry
//!
//! Logging, panic capture and action telemetry are wired through
//! [`rpfm_telemetry`]. The Sentry guard returned by [`Logger::init`] is held
//! for the process lifetime in [`main`].

use axum::{extract::State, routing::get, Json, Router};
use rmcp::transport::streamable_http_server::{session::local::LocalSessionManager, StreamableHttpService};
use tokio::net::TcpListener;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, Layer};

use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;

use rpfm_ipc::helpers::SessionInfo;
use rpfm_ipc::messages::{Command, Response};
use rpfm_ipc::settings_keys::{ENABLE_CRASH_REPORTS, ENABLE_USAGE_TELEMETRY};

use rpfm_telemetry::{Logger, SentryLayer, SENTRY_DSN, info, release_name, warn};

use crate::server_mcp::McpServer;
use crate::session::SessionManager;
use crate::settings::{error_path, init_config_path, Settings};
use crate::server_websocket::ws_handler;

pub mod background_thread;
pub mod ceo_builder;
pub mod comms;
pub mod server_mcp;
pub mod server_websocket;
pub mod session;
pub mod settings;
pub mod updater;
#[cfg(test)] mod updater_test;

use mimalloc::MiMalloc;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

//-------------------------------------------------------------------------------//
//                                  Constants
//-------------------------------------------------------------------------------//

/// Sentry DSN used for crash reports and action telemetry.
const SENTRY_DSN_KEY: &str = match option_env!("RPFM_SERVER_SENTRY_DSN") {
    Some(dsn) => dsn,
    None => "",
};

/// Default IP address the HTTP server binds to (`127.0.0.1` / loopback).
const DEFAULT_ADDRESS: [u8; 4] = [127, 0, 0, 1];

/// Default TCP port the HTTP server listens on.
const DEFAULT_PORT: u16 = 45127;

/// Organisation domain used to derive the OS-specific config directory
/// (mirrors `QCoreApplication::organizationDomain` on the UI side).
const ORG_DOMAIN: &str = "com";

/// Organisation name used to derive the OS-specific config directory.
const ORG_NAME: &str = "FrodoWazEre";

/// Application name used to derive the OS-specific config directory.
const APP_NAME: &str = "rpfm";

//-------------------------------------------------------------------------------//
//                                  Functions
//-------------------------------------------------------------------------------//

/// Process entry point.
///
/// Initialises the Sentry/telemetry guard, primes the telemetry toggles from
/// persisted settings, builds the [`session::SessionManager`], wires the
/// `axum` router (`/ws`, `/sessions`, `/mcp`) and starts the listener on
/// [`DEFAULT_ADDRESS`]:[`DEFAULT_PORT`].
///
/// Returns when the listener stops accepting (typically after every session
/// has been cleaned up — the cleanup task in [`session::SessionManager`]
/// terminates the process when the session set drains).
#[tokio::main]
async fn main() {

    // Sentry client guard, so we can reuse it later on and keep it in scope for the entire duration of the program.
    // Must be initialized before the tracing subscriber so the SentryLayer can capture spans.
    *SENTRY_DSN.write().unwrap() = SENTRY_DSN_KEY.to_owned();
    let guard = Logger::init(&{
        init_config_path().expect("Error while trying to initialize config path. We're fucked.");
        error_path().unwrap_or_else(|_| PathBuf::from("."))
    }, true, false, release_name!()).expect("Failed to initialize logging system.");

    // Setup tracing subscriber for logging, redirecting to stderr to avoid interfering with MCP.
    // The SentryLayer captures tracing spans/events as Sentry breadcrumbs and performance spans.
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer()
            .with_writer(std::io::stderr)
            .with_filter(tracing_subscriber::filter::LevelFilter::INFO))
        .with(SentryLayer::default())
        .init();

    if guard.is_enabled() {
        info!("Sentry logging support for RPFM SERVER enabled. Starting...");
    } else {
        info!("Sentry logging support for RPFM SERVER disabled. Starting...");
    }

    // Read telemetry settings from disk before any sessions spin up so early commands
    // are counted and crash reports respect the user's choice. Background threads will
    // refresh these whenever the settings change.
    if let Ok(settings) = Settings::init(false) {
        rpfm_telemetry::set_usage_telemetry_enabled(settings.bool(ENABLE_USAGE_TELEMETRY));
        rpfm_telemetry::set_crash_reports_enabled(settings.bool(ENABLE_CRASH_REPORTS));
    }

    // Create the session manager to handle per-client sessions,
    // and start the background cleanup task for expired sessions.
    let session_manager: Arc<SessionManager> = Arc::new(SessionManager::default());
    SessionManager::start_cleanup_task(session_manager.clone());

    // Create an MCP service with its own session for MCP clients.
    let sm = session_manager.clone();
    let http_service = StreamableHttpService::new(
        move || {
            let session = sm.create_session();
            Ok(McpServer::new(session))
        },
        LocalSessionManager::default().into(),
        Default::default(),
    );

    // Setup the endpoints for the server.
    let app = Router::new()
        .route("/ws", get(ws_handler))
        .route("/sessions", get(sessions_handler))
        .nest_service("/mcp", http_service)
        .with_state(session_manager);

    let addr = SocketAddr::from((DEFAULT_ADDRESS, DEFAULT_PORT));
    match TcpListener::bind(addr).await {
        Ok(listener) => {
            info!("Listening on {}", addr);
            axum::serve(listener, app).await.unwrap();
        }
        Err(err) => {
            warn!("Failed to bind to address {}: {}\n\nThis usually means you got another copy of the server running. Either use that one, or stop it and try again.", addr, err);
        }
    }
}

/// REST endpoint to get information about all active sessions.
///
/// Returns a JSON array of [`SessionInfo`] objects containing:
/// - `session_id`: Unique session identifier
/// - `connection_count`: Number of active WebSocket connections
/// - `timeout_remaining_secs`: Seconds until session cleanup (if disconnected)
/// - `is_shutting_down`: Whether session is marked for shutdown
///
/// This endpoint is used by the UI's session management dialog to display
/// available sessions and allow users to connect to specific ones.
async fn sessions_handler(State(session_manager): State<Arc<SessionManager>>) -> Json<Vec<SessionInfo>> {
    let sessions = session_manager.get_sessions_info();
    info!("Sessions endpoint queried: {} active session(s)", sessions.len());
    Json(sessions)
}
