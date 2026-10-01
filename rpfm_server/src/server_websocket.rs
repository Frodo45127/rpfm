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
use tokio::sync::{broadcast, mpsc};

use std::sync::Arc;

use rpfm_ipc::api::{ApiError, Done, JOB_UPDATED_NOTIFICATION, JSONRPC_VERSION, Request, RpcNotification, RpcRequest, RpcResponse};
use rpfm_ipc::api::session::Disconnect;
use rpfm_ipc::messages::{Command, Message as IpcMessage, Response};
use rpfm_telemetry::{info, warn};

use crate::session::{DEFAULT_SESSION_TIMEOUT_SECS, Session, SessionId, SessionManager, recv_response};

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

    /// A response or notification of the legacy protocol.
    Legacy(Box<IpcMessage<Response>>),

    /// A response of the version 2 API.
    Api(RpcResponse),

    /// A notification of the version 2 API.
    Notification(RpcNotification),
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
    let session_connected_msg = IpcMessage {
        id: 0, // Special ID for connection message
        data: Response::SessionConnected(session_id),
    };
    if let Ok(json) = serde_json::to_string(&session_connected_msg) {
        let _ = sink.send(Message::Text(json.into())).await;
    }

    // Task to send responses back to the client.
    let sender_task = tokio::spawn(async move {
        while let Some(outgoing) = rx.recv().await {
            let response_msg = match outgoing {
                Outgoing::Legacy(response_msg) => *response_msg,
                Outgoing::Api(response) => {
                    let json = serde_json::to_string(&response)
                        .unwrap_or_else(|error| serde_json::to_string(&RpcResponse::new(response.id, Err(ApiError::Internal(format!("Serialization error: {error}"))))).unwrap_or_default());

                    if sink.send(Message::Text(json.into())).await.is_err() {
                        break;
                    }
                    continue;
                }
                Outgoing::Notification(notification) => {
                    if let Ok(json) = serde_json::to_string(&notification) {
                        if sink.send(Message::Text(json.into())).await.is_err() {
                            break;
                        }
                    }
                    continue;
                }
            };

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

    // Loop to receive commands from the client.
    while let Some(msg) = receiver.next().await {
        if let Ok(msg) = msg {
            match msg {
                Message::Text(t) => {

                    // Messages with a "jsonrpc" key are version 2 requests. Inside JSON strings its quotes
                    // would be escaped, so only a key can match this.
                    if t.contains("\"jsonrpc\"") {
                        if let Ok(request) = serde_json::from_str::<RpcRequest>(&t) {

                            // Disconnecting needs the session manager, so it's answered here, before cleaning up.
                            if request.method == Disconnect::METHOD {
                                let done = serde_json::to_value(Done {}).map_err(|error| ApiError::Internal(error.to_string()));
                                let _ = tx.send(Outgoing::Api(RpcResponse::new(request.id, done)));
                                graceful_disconnect = true;
                                break;
                            }

                            handle_api_request(request, &session, &tx);
                            continue;
                        }
                    }

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
                                let _ = tx.send(Outgoing::Legacy(Box::new(response_msg)));
                                graceful_disconnect = true;
                                break;
                            }

                            // Enqueue the command before spawning, so commands run in the order they arrived.
                            let mut receiver = session.send(msg.data);
                            let tx = tx.clone();
                            tokio::spawn(async move {
                                let response = recv_response(&mut receiver).await;
                                let response_msg = IpcMessage {
                                    id: msg.id,
                                    data: response,
                                };
                                let _ = tx.send(Outgoing::Legacy(Box::new(response_msg)));
                            });
                        }
                        Err(error) => {
                            warn!("Session {}: Deserialization error: {}", session.id(), error);

                            // Try to extract the message ID from the malformed message so we can
                            // send an error response back to the client.
                            if let Some(id) = serde_json::from_str::<serde_json::Value>(&t)
                                .ok()
                                .and_then(|v| v.get("id")?.as_u64()) {
                                let error_msg = IpcMessage {
                                    id,
                                    data: Response::Error(format!("Server failed to deserialize command: {}", error)),
                                };
                                let _ = tx.send(Outgoing::Legacy(Box::new(error_msg)));
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
    jobs_forward_task.abort();

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

/// Sends a version 2 API request to the session's background thread, and its response to the client when it's done.
fn handle_api_request(request: RpcRequest, session: &Arc<Session>, tx: &mpsc::UnboundedSender<Outgoing>) {
    let id = request.id;
    if request.jsonrpc != JSONRPC_VERSION {
        let _ = tx.send(Outgoing::Api(RpcResponse::invalid_request(id, format!("Unsupported JSON-RPC version: {}", request.jsonrpc))));
        return;
    }

    // Enqueue the request before spawning, so requests run in the order they arrived.
    let mut receiver = session.call(request);
    let tx = tx.clone();
    tokio::spawn(async move {
        let response = receiver.recv().await
            .unwrap_or_else(|| RpcResponse::new(id, Err(ApiError::Internal("Session response channel closed unexpectedly".to_owned()))));

        let _ = tx.send(Outgoing::Api(response));
    });
}
