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
//! client supplies `?session_id=N`) or creates a new one. From then on the
//! socket carries a stream of JSON-encoded [`IpcMessage<Command>`] frames
//! from the client and [`IpcMessage<Response>`] frames back. Each command
//! is dispatched into the session's dedicated background thread, whose
//! responses are forwarded back over the same socket with the originating
//! request `id` preserved so the client can correlate them.
//!
//! Graceful disconnect (`Command::ClientDisconnecting`) tears the session
//! down immediately and flushes telemetry. Hard disconnects (socket close
//! without that command) leave the session in a 5-minute grace period so
//! the client can reconnect with the same `session_id` and pick up where it
//! left off.
//!
//! [`Session`]: crate::session::Session
//! [`IpcMessage<Command>`]: rpfm_ipc::messages::Message
//! [`IpcMessage<Response>`]: rpfm_ipc::messages::Message

use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    extract::{Query, State},
    response::IntoResponse
};
use futures::stream::StreamExt;
use futures::sink::SinkExt;
use serde::Deserialize;
use tokio::sync::mpsc;

use std::sync::Arc;

use rpfm_ipc::messages::{Command, Message as IpcMessage, Response};
use rpfm_telemetry::{error, info};

use crate::session::{DEFAULT_SESSION_TIMEOUT_SECS, SessionId, SessionManager, recv_response};

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//


/// Query parameters for WebSocket connection.
#[derive(Debug, Deserialize)]
pub struct WsQueryParams {

    /// Optional session ID to connect to an existing session.
    pub session_id: Option<SessionId>,
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
    let (tx, mut rx) = mpsc::unbounded_channel::<IpcMessage<Response>>();

    // Send the session ID to the client immediately after connection.
    let session_connected_msg = IpcMessage {
        id: 0, // Special ID for connection message
        data: Response::SessionConnected(session_id),
    };
    if let Ok(json) = serde_json::to_string(&session_connected_msg) {
        let _ = sink.send(Message::Text(json.into())).await;
    }

    // Task to send responses back to the client.
    let sender_task = tokio::spawn(async move {
        while let Some(response_msg) = rx.recv().await {
            match serde_json::to_string(&response_msg) {
                Ok(json) => {
                    if sink.send(Message::Text(json.into())).await.is_err() {
                        break;
                    }
                }
                Err(error) => {
                    let error_msg = IpcMessage {
                        id: response_msg.id,
                        data: Response::Error(format!("Serialization error: {}", error)),
                    };

                    if let Ok(json) = serde_json::to_string(&error_msg) {
                        let _ = sink.send(Message::Text(json.into())).await;
                    }
                }
            }
        }
    });

    // Track whether the client requested a graceful disconnect.
    let mut graceful_disconnect = false;

    // Loop to receive commands from the client.
    while let Some(msg) = receiver.next().await {
        if let Ok(msg) = msg {
            match msg {
                Message::Text(t) => {
                    // Try to parse the message to check for ClientDisconnecting.
                    match serde_json::from_str::<IpcMessage<Command>>(&t) {
                        Ok(msg) => {
                            info!("Session {}: Received command [ID {}]: {:?}", session.id(), msg.id, msg.data);

                            // Handle ClientDisconnecting specially - it needs access to session_manager.
                            if matches!(msg.data, Command::ClientDisconnecting) {
                                // Send success response before cleanup.
                                let response_msg = IpcMessage {
                                    id: msg.id,
                                    data: Response::Success,
                                };
                                let _ = tx.send(response_msg);
                                graceful_disconnect = true;
                                break;
                            }

                            // Route other commands through the session's background thread.
                            let tx = tx.clone();
                            let session = session.clone();
                            tokio::spawn(async move {
                                let mut receiver = session.send(msg.data);
                                let response = recv_response(&mut receiver).await;
                                let response_msg = IpcMessage {
                                    id: msg.id,
                                    data: response,
                                };
                                let _ = tx.send(response_msg);
                            });
                        }
                        Err(error) => {
                            error!("Session {}: Deserialization error: {}", session.id(), error);

                            // Try to extract the message ID from the malformed message so we can
                            // send an error response back to the client.
                            if let Some(id) = serde_json::from_str::<serde_json::Value>(&t)
                                .ok()
                                .and_then(|v| v.get("id")?.as_u64()) {
                                let error_msg = IpcMessage {
                                    id,
                                    data: Response::Error(format!("Server failed to deserialize command: {}", error)),
                                };
                                let _ = tx.send(error_msg);
                            }

                            // TODO: Handle the error case when the message ID cannot be extracted.
                        }
                    }
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

    sender_task.abort();

    // Client requested graceful disconnect - remove session immediately.
    if graceful_disconnect {
        info!("Session {}: Client requested graceful disconnect, removing session immediately", session_id);
        session_manager.remove_session(session_id);

        // Check if this was the last session and shutdown the server if so.
        if session_manager.session_count() == 0 {
            info!("No more active sessions, shutting down server...");
            rpfm_telemetry::flush("Server Action Telemetry");
            std::process::exit(0);
        }
    }

    // Unexpected disconnect - mark session for timeout cleanup.
    else {
        SessionManager::client_disconnected(session_manager.clone(),session_id);
        info!("Session {} client disconnected, session will timeout in {} minutes if not reconnected", session_id, DEFAULT_SESSION_TIMEOUT_SECS / 60);
    }
}
