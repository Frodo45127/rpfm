//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

use axum::{extract::State, routing::get, Json, Router};
use rmcp::transport::streamable_http_server::{session::local::LocalSessionManager, StreamableHttpService};
use tokio::net::TcpListener;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;

use rpfm_ipc::helpers::SessionInfo;
use rpfm_ipc::messages::{Command, Response};
use rpfm_lib::integrations::log::{Logger, SENTRY_DSN, error, info, release_name};

use crate::server_mcp::McpServer;
use crate::session::SessionManager;
use crate::settings::{error_path, init_config_path};
use crate::server_websocket::ws_handler;

pub mod background_thread;
pub mod comms;
pub mod server_mcp;
pub mod server_websocket;
pub mod session;
pub mod settings;
pub mod updater;

use mimalloc::MiMalloc;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

//-------------------------------------------------------------------------------//
//                                  Constants
//-------------------------------------------------------------------------------//

const SENTRY_DSN_KEY: &str = "https://a8bf0a98ed43467d841ec433fb3d75a8:aeb106a185a0439fb7598598e0160ab2@o152833.ingest.sentry.io/1205298";

const DEFAULT_ADDRESS: [u8; 4] = [127, 0, 0, 1];
const DEFAULT_PORT: u16 = 45127;

const ORG_DOMAIN: &str = "com";
const ORG_NAME: &str = "FrodoWazEre";
const APP_NAME: &str = "rpfm";

//-------------------------------------------------------------------------------//
//                                  Functions
//-------------------------------------------------------------------------------//

#[tokio::main]
async fn main() {

    // Setup tracing subscriber for logging, redirecting to stderr to avoid interfering with MCP.
    // TODO: Migrate lib's logging to tracing.
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer().with_writer(std::io::stderr))
        .init();

    // Sentry client guard, so we can reuse it later on and keep it in scope for the entire duration of the program.
    // TODO: See if I can get rid of this global.
    *SENTRY_DSN.write().unwrap() = SENTRY_DSN_KEY.to_owned();
    let guard = Logger::init(&{
        init_config_path().expect("Error while trying to initialize config path. We're fucked.");
        error_path().unwrap_or_else(|_| PathBuf::from("."))
    }, true, false, release_name!()).expect("Failed to initialize logging system.");

    if guard.is_enabled() {
        info!("Sentry Logging support enabled. Starting...");
    } else {
        info!("Sentry Logging support disabled. Starting...");
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
            error!("Failed to bind to address {}: {}", addr, err);
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
