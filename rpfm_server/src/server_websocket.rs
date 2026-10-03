//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! WebSocket upgrade handler and message multiplexer for the `/ws` endpoint.
//!
//! On upgrade, the handler either reuses an existing [`Session`] (when the
//! client supplies `?session_id=N`) or creates a new one, and greets the client
//! with a `session.connected` notification. From then on the socket carries
//! JSON-RPC 2.0 requests from the client, and their responses and the job
//! notifications back. Each request is queued into the session's background
//! thread in the order it arrived, and its response is sent back with the
//! request's `id` so the client can correlate them.
//!
//! Graceful disconnect (`session.disconnect`) tears the session down
//! immediately and flushes telemetry. Hard disconnects (socket close without
//! that request) leave the session in a 5-minute grace period so the client
//! can reconnect with the same `session_id` and pick up where it left off.
//!
//! [`Session`]: crate::session::Session

use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    extract::{Query, State},
    response::IntoResponse
};
use futures::stream::StreamExt;
use futures::sink::SinkExt;
use serde::Deserialize;
use tokio::sync::{broadcast, mpsc};

use std::sync::Arc;

use rpfm_ipc::api::{ApiError, Done, JOB_UPDATED_NOTIFICATION, JSONRPC_VERSION, Request, RpcNotification, RpcRequest, RpcResponse};
use rpfm_ipc::api::session::{Disconnect, SESSION_CONNECTED_NOTIFICATION, SessionConnected};
use rpfm_telemetry::{info, warn};

use crate::session::{DEFAULT_SESSION_TIMEOUT_SECS, Session, SessionId, SessionManager};

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//


/// Query parameters for WebSocket connection.
#[derive(Debug, Deserialize)]
pub struct WsQueryParams {

    /// Optional session ID to connect to an existing session.
    pub session_id: Option<SessionId>,
}

/// A message for the client.
enum Outgoing {

    /// A response to a request.
    Response(RpcResponse),

    /// A notification.
    Notification(RpcNotification),

    /// Closes the connection, once the messages queued before it are sent.
    Close,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

/// WebSocket handler to upgrade the connection and handle messages.
///
/// Accepts an optional `session_id` query parameter to reconnect to an existing session.
/// Example: `ws://localhost:45127/ws?session_id=123`
pub(crate) async fn ws_handler(
    State(session_manager): State<Arc<SessionManager>>,
    Query(params): Query<WsQueryParams>,
    ws: WebSocketUpgrade,
) -> impl IntoResponse {
    ws.max_message_size(usize::MAX)
        .max_frame_size(usize::MAX)
        .on_upgrade(move |socket| handle_socket(socket, session_manager, params.session_id))
}

/// Function to handle a WebSocket connection.
///
/// Each WebSocket connection gets its own session with an isolated background thread.
/// If a session_id is provided and that session exists, the client reconnects to it.
async fn handle_socket(socket: WebSocket, session_manager: Arc<SessionManager>, requested_session_id: Option<SessionId>) {

    // Get or create a session for this client connection.
    let (session, is_new) = session_manager.get_or_create_session(requested_session_id);
    let session_id = session.id();

    if is_new {
        info!("New WebSocket client connected, created session ID: {}", session_id);
    } else {
        info!("WebSocket client reconnected to existing session ID: {}", session_id);
    }

    let (mut sink, mut receiver) = socket.split();
    let (tx, mut rx) = mpsc::unbounded_channel::<Outgoing>();

    // Send the session ID to the client immediately after connection.
    if let Ok(params) = serde_json::to_value(SessionConnected { session_id }) {
        if let Ok(json) = serde_json::to_string(&RpcNotification::new(SESSION_CONNECTED_NOTIFICATION, params)) {
            let _ = sink.send(Message::Text(json.into())).await;
        }
    }

    // Task to send responses back to the client.
    let sender_task = tokio::spawn(async move {
        while let Some(outgoing) = rx.recv().await {
            let json = match outgoing {
                Outgoing::Response(response) => serde_json::to_string(&response)
                    .unwrap_or_else(|error| serde_json::to_string(&RpcResponse::new(response.id, Err(ApiError::Internal(format!("Serialization error: {error}"))))).unwrap_or_default()),
                Outgoing::Notification(notification) => match serde_json::to_string(&notification) {
                    Ok(json) => json,
                    Err(_) => continue,
                },
                Outgoing::Close => {
                    let _ = sink.close().await;
                    break;
                }
            };

            if sink.send(Message::Text(json.into())).await.is_err() {
                break;
            }
        }
    });

    // Task to notify the client of every change of the session's jobs.
    let mut job_updates = session.jobs().subscribe();
    let jobs_tx = tx.clone();
    let jobs_forward_task = tokio::spawn(async move {
        loop {
            match job_updates.recv().await {
                Ok(status) => {
                    if let Ok(params) = serde_json::to_value(&status) {
                        let _ = jobs_tx.send(Outgoing::Notification(RpcNotification::new(JOB_UPDATED_NOTIFICATION, params)));
                    }
                }

                // Clients can always ask for the current state of a job, so missed updates are skipped.
                Err(broadcast::error::RecvError::Lagged(_)) => continue,
                Err(broadcast::error::RecvError::Closed) => break,
            }
        }
    });

    // Track whether the client requested a graceful disconnect.
    let mut graceful_disconnect = false;

    // Loop to receive requests from the client.
    while let Some(msg) = receiver.next().await {
        if let Ok(msg) = msg {
            match msg {
                Message::Text(t) => {

                    let request = match serde_json::from_str::<RpcRequest>(&t) {
                        Ok(request) => request,
                        Err(error) => {
                            warn!("Session {}: Deserialization error: {}", session.id(), error);

                            // Answer with the ID of the malformed request, if it can be found.
                            let id = serde_json::from_str::<serde_json::Value>(&t).ok()
                                .and_then(|value| value.get("id")?.as_u64())
                                .unwrap_or_default();

                            let _ = tx.send(Outgoing::Response(RpcResponse::invalid_request(id, format!("Invalid request: {error}"))));
                            continue;
                        }
                    };

                    // Disconnecting needs the session manager, so it's answered here, before cleaning up.
                    if request.method == Disconnect::METHOD {
                        let done = serde_json::to_value(Done {}).map_err(|error| ApiError::Internal(error.to_string()));
                        let _ = tx.send(Outgoing::Response(RpcResponse::new(request.id, done)));
                        let _ = tx.send(Outgoing::Close);
                        graceful_disconnect = true;
                        break;
                    }

                    handle_api_request(request, &session, &tx);
                }
                Message::Close(_) => {
                    info!("Session {}: Client disconnected", session_id);
                    break;
                }
                _ => {}
            }
        } else {
            info!("Session {}: Client disconnected (error)", session_id);
            break;
        }
    }

    jobs_forward_task.abort();

    // On a graceful disconnect, the sender ends after sending the disconnect's response.
    if graceful_disconnect {
        let _ = sender_task.await;
    } else {
        sender_task.abort();
    }

    // Client requested graceful disconnect - remove session immediately.
    if graceful_disconnect {
        info!("Session {}: Client requested graceful disconnect, removing session immediately", session_id);
        session_manager.remove_session(session_id);
        session_manager.exit_if_no_sessions();
    }

    // Unexpected disconnect - mark session for timeout cleanup.
    else {
        session_manager.client_disconnected(session_id);
        info!("Session {} client disconnected, session will timeout in {} minutes if not reconnected", session_id, DEFAULT_SESSION_TIMEOUT_SECS / 60);
    }
}

/// Sends a request to the session's background thread, and its response to the client when it's done.
fn handle_api_request(request: RpcRequest, session: &Arc<Session>, tx: &mpsc::UnboundedSender<Outgoing>) {
    let id = request.id;
    if request.jsonrpc != JSONRPC_VERSION {
        let _ = tx.send(Outgoing::Response(RpcResponse::invalid_request(id, format!("Unsupported JSON-RPC version: {}", request.jsonrpc))));
        return;
    }

    // Enqueue the request before spawning, so requests run in the order they arrived.
    let mut receiver = session.call(request);
    let tx = tx.clone();
    tokio::spawn(async move {
        let response = receiver.recv().await
            .unwrap_or_else(|| RpcResponse::new(id, Err(ApiError::Internal("Session response channel closed unexpectedly".to_owned()))));

        let _ = tx.send(Outgoing::Response(response));
    });
}
