//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

/*!
This module defines the code used for thread communication.
!*/

use qt_core::QEventLoop;

use anyhow::{Result, anyhow};
use crossbeam::channel::{Receiver, Sender, unbounded};
use futures::{SinkExt, StreamExt};
use tokio::sync::mpsc::{unbounded_channel, UnboundedSender, UnboundedReceiver};
use tokio_tungstenite::{connect_async_with_config, tungstenite::protocol::{Message as WsMessage, WebSocketConfig}};

use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::{Arc, RwLock};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

pub use rpfm_ipc::messages::{Command, Response, Message as IpcMessage};
use rpfm_ipc::api::{ApiError, Request, RpcOutcome, RpcRequest, RpcResponse};
use rpfm_ipc::api::jobs::{JobStarted, JobState, WaitForJob};
use rpfm_ipc::api::session::Configure;

use rpfm_telemetry::*;

use crate::CENTRAL_COMMAND;
use crate::settings_ui::backend::{mark_settings_changed, take_changed_settings};

pub mod server;

/// Atomic counter for generating unique message IDs.
static MESSAGE_ID_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Global variable to hold the current session ID we are connected to.
pub static CURRENT_SESSION_ID: std::sync::LazyLock<Arc<RwLock<Option<u64>>>> = std::sync::LazyLock::new(|| Arc::new(RwLock::new(None)));

/// Global variable to hold the session ID to reconnect to. When set, the WebSocket loop will
/// disconnect and reconnect to the specified session.
pub static RECONNECT_SESSION_ID: std::sync::LazyLock<Arc<RwLock<Option<u64>>>> = std::sync::LazyLock::new(|| Arc::new(RwLock::new(None)));

/// Global flag to signal the WebSocket loop that a reconnection is requested.
pub static RECONNECT_REQUESTED: AtomicBool = AtomicBool::new(false);

/// Global flag to indicate that the WebSocket has successfully reconnected.
pub static RECONNECT_COMPLETE: AtomicBool = AtomicBool::new(false);

/// Global flag to signal that the application is shutting down.
pub static SHUTDOWN_REQUESTED: AtomicBool = AtomicBool::new(false);

/// This const is the standard message in case of message communication error. If this happens, crash the program.
pub const THREADS_COMMUNICATION_ERROR: &str = "Error in thread communication system. Response received: ";
pub const THREADS_SENDER_ERROR: &str = "Error in thread communication system. Sender failed to send message.";

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This struct contains the senders and receivers necessary to communicate both, backend and frontend threads.
///
/// You can use them by using the send/recv functions implemented for it.
pub struct CentralCommand<T: Send + Sync + Debug> {
    sender: UnboundedSender<Outgoing<T>>,
    try_lock: AtomicBool,
}

/// A message for the server, with where to send its response.
pub enum Outgoing<T> {

    /// A command of the legacy protocol.
    Legacy(Box<IpcMessage<Command>>, Sender<T>),

    /// A request of the version 2 API.
    Api(RpcRequest, Sender<RpcResponse>),
}

//-------------------------------------------------------------------------------//
//                              Implementations
//-------------------------------------------------------------------------------//

/// Default implementation of `CentralCommand`.
impl<T: Send + Sync + Debug> Default for CentralCommand<T> {
    fn default() -> Self {
        let (sender, _) = unbounded_channel();
        let try_lock = AtomicBool::new(false);
        Self {
            sender,
            try_lock,
        }
    }
}

impl<T: Send + Sync + Debug> CentralCommand<T> {

    /// This function initializes a new central command, and returns the sender to send messages to it.
    ///
    /// Use it to replace the default one on runtime.
    pub fn init() -> (Self, UnboundedReceiver<Outgoing<T>>) {
        let (sender, receiver) = unbounded_channel();
        let try_lock = AtomicBool::new(false);
        (Self {
            sender,
            try_lock,
        }, receiver)
    }
}

/// Implementation of `CentralCommand`.
impl<T: Send + Sync + Debug + for<'a> serde::Deserialize<'a>> CentralCommand<T> {

    /// This function serves as a generic way for commands to be sent to the backend.
    ///
    /// It returns the receiver which will receive the answers for the command, if any.
    pub fn send(&self, data: Command) -> Receiver<T> {
        let (sender_back, receiver_back) = unbounded();
        let id = MESSAGE_ID_COUNTER.fetch_add(1, Ordering::SeqCst);
        let message = IpcMessage { id, data };
        if self.sender.send(Outgoing::Legacy(Box::new(message), sender_back)).is_err() {
            panic!("{THREADS_SENDER_ERROR}");
        }

        receiver_back
    }

    /// This function sends a request of the version 2 API to the backend.
    ///
    /// It returns the receiver which will receive the response.
    pub fn call<R: Request>(&self, request: &R) -> Receiver<RpcResponse> {
        let (sender_back, receiver_back) = unbounded();
        let id = MESSAGE_ID_COUNTER.fetch_add(1, Ordering::SeqCst);
        match RpcRequest::new(id, request) {
            Ok(request) => if self.sender.send(Outgoing::Api(request, sender_back)).is_err() {
                panic!("{THREADS_SENDER_ERROR}");
            },
            Err(error) => {
                let _ = sender_back.send(RpcResponse::new(id, Err(ApiError::InvalidParams(error.to_string()))));
            }
        }

        receiver_back
    }

    /// This functions serves to receive messages from a generated channel.
    ///
    /// This function does only try once, and it locks the thread. Panics if the response fails.
    pub fn recv(receiver: &Receiver<T>) -> T {
        let response = receiver.recv();
        match response {
            Ok(data) => data,
            Err(_) => panic!("{THREADS_COMMUNICATION_ERROR}{response:?}")
        }
    }

    /// This functions serves to receive messages from a generated channel.
    ///
    /// This function will keep asking for a response, keeping the UI responsive. Use it for heavy tasks.
    ///
    /// NOTE: Beware of other events triggering when this keeps the UI enabled. It can lead to crashes.
    pub fn recv_try(&self, receiver: &Receiver<T>) -> T {
        let event_loop = unsafe { QEventLoop::new_0a() };

        // Lock this function after the first execution, until it gets freed again.
        if !self.try_lock.load(Ordering::SeqCst) {
            self.try_lock.store(true, Ordering::SeqCst);

            loop {

                // Check the response and, in case of error, try again. If the error is "Disconnected", CTD.
                let response = receiver.try_recv();
                match response {
                    Ok(data) => {
                        self.try_lock.store(false, Ordering::SeqCst);
                        return data
                    },
                    Err(error) => if error.is_disconnected() {
                        panic!("{THREADS_COMMUNICATION_ERROR}{response:?}")
                    }
                }
                unsafe { event_loop.process_events(); }
            }
        }

        // If we're locked due to another execution, use recv instead.
        else {
            info!("Race condition avoided? Two items calling recv_try on the same execution crashes.");
            Self::recv(receiver)
        }
    }

    /// Non-panicking variant of [`Self::recv_try`]. Returns `None` if the sender is
    /// already disconnected — used by call sites where a lost backend should fail
    /// gracefully (e.g. the updater) instead of crashing the whole UI.
    pub fn recv_try_checked<U: Debug>(&self, receiver: &Receiver<U>) -> Option<U> {
        let event_loop = unsafe { QEventLoop::new_0a() };

        if !self.try_lock.load(Ordering::SeqCst) {
            self.try_lock.store(true, Ordering::SeqCst);

            loop {
                match receiver.try_recv() {
                    Ok(data) => {
                        self.try_lock.store(false, Ordering::SeqCst);
                        return Some(data);
                    },
                    Err(error) => if error.is_disconnected() {
                        self.try_lock.store(false, Ordering::SeqCst);
                        return None;
                    }
                }
                unsafe { event_loop.process_events(); }
            }
        }

        else {
            info!("Race condition avoided? Two items calling recv_try on the same execution crashes.");
            receiver.recv().ok()
        }
    }
}

/// Function to send a command to the backend and receive a result. Use it for commands that can fail.
pub fn send_ipc_command_result<T, F>(command: Command, extractor: F) -> Result<T>
where
    F: FnOnce(Response) -> T,
{
    let receiver = CENTRAL_COMMAND.read().unwrap().send(command);
    match CentralCommand::recv(&receiver) {
        Response::Error(error) => Err(anyhow!(error)),
        response => Ok(extractor(response)),
    }
}

/// Function to send a command to the backend and receive a result. Use it for commands that can fail.
///
/// This version of the function is for calls that must keep the ui alive.
#[allow(dead_code)]
pub fn send_ipc_command_result_async<T, F>(command: Command, extractor: F) -> Result<T>
where
    F: FnOnce(Response) -> T,
{
    let receiver = CENTRAL_COMMAND.read().unwrap().send(command);
    match CENTRAL_COMMAND.read().unwrap().recv_try_checked(&receiver) {
        Some(Response::Error(error)) => Err(anyhow!(error)),
        Some(response) => Ok(extractor(response)),
        None => Err(anyhow!("{THREADS_COMMUNICATION_ERROR}Disconnected")),
    }
}

/// Function to send a command to the backend. Use it for commands that can't fail.
pub fn send_ipc_command<T, F>(command: Command, extractor: F) -> T
where
    F: FnOnce(Response) -> T,
{
    let receiver = CENTRAL_COMMAND.read().unwrap().send(command);
    let response = CentralCommand::recv(&receiver);
    extractor(response)
}

/// Function to send a command to the backend. Use it for commands that can't fail.
///
/// This version of the function is for calls that must keep the ui alive.
#[allow(dead_code)]
pub fn send_ipc_command_async<T, F>(command: Command, extractor: F) -> T
where
    F: FnOnce(Response) -> T,
{
    let receiver = CENTRAL_COMMAND.read().unwrap().send(command);
    let response = CENTRAL_COMMAND.read().unwrap().recv_try(&receiver);
    extractor(response)
}

/// Calls a method of the server's API, and waits for its response.
pub fn call_api<R: Request>(request: &R) -> Result<R::Response> {
    debug_assert!(!R::IS_JOB, "{} runs as a job. Use run_job instead.", R::METHOD);
    let receiver = CENTRAL_COMMAND.read().unwrap().call(request);
    match receiver.recv() {
        Ok(response) => api_result(response),
        Err(_) => panic!("{THREADS_COMMUNICATION_ERROR}Disconnected"),
    }
}

/// Calls a method of the server's API, and waits for its response while keeping the UI alive.
pub fn call_api_async<R: Request>(request: &R) -> Result<R::Response> {
    debug_assert!(!R::IS_JOB, "{} runs as a job. Use run_job instead.", R::METHOD);
    api_result(call_api_async_raw(request)?)
}

/// Runs a method of the server's API that runs as a job, and waits for it to end while keeping the UI alive.
pub fn run_job<R: Request>(request: &R) -> Result<R::Response> {
    debug_assert!(R::IS_JOB, "{} doesn't run as a job. Use call_api instead.", R::METHOD);
    let JobStarted { job } = api_result(call_api_async_raw(request)?)?;
    loop {
        let status = call_api_async(&WaitForJob { job, timeout_secs: None })?;
        match status.state {
            JobState::Finished { result } => return serde_json::from_value(result).map_err(From::from),
            JobState::Failed { error } => return Err(ApiError::from(error).into()),
            JobState::Cancelled => return Err(anyhow!("The job was cancelled before it started.")),
            JobState::Queued | JobState::Running { .. } => {}
        }
    }
}

/// Sends a request of the server's API, and waits for its response while keeping the UI alive.
fn call_api_async_raw<R: Request>(request: &R) -> Result<RpcResponse> {
    let receiver = CENTRAL_COMMAND.read().unwrap().call(request);
    CENTRAL_COMMAND.read().unwrap().recv_try_checked(&receiver).ok_or_else(|| anyhow!("{THREADS_COMMUNICATION_ERROR}Disconnected"))
}

/// Turns the response of a request into its result. Errors are [`ApiError`]s, so callers can downcast them.
pub fn api_result<T: serde::de::DeserializeOwned>(response: RpcResponse) -> Result<T> {
    match response.outcome {
        RpcOutcome::Result(value) => serde_json::from_value(value).map_err(From::from),
        RpcOutcome::Error(error) => Err(ApiError::from(error).into()),
    }
}

/// Request a reconnection to a specific session ID.
///
/// This will signal the WebSocket loop to disconnect from the current session and
/// reconnect to the specified session.
pub fn request_reconnect(session_id: u64) {
    RECONNECT_COMPLETE.store(false, Ordering::SeqCst);
    *RECONNECT_SESSION_ID.write().unwrap() = Some(session_id);
    RECONNECT_REQUESTED.store(true, Ordering::SeqCst);
}

/// Mark the application as shutting down and tell the server to tear down the session.
///
/// The flag is set before the command is sent so the WebSocket loop can distinguish
/// the upcoming connection drop from an unexpected disconnect and skip reconnection.
pub fn request_disconnect() {
    SHUTDOWN_REQUESTED.store(true, Ordering::SeqCst);
    let _ = CENTRAL_COMMAND.read().unwrap().send(Command::ClientDisconnecting);
}

/// Wait for the reconnection to complete, processing Qt events to keep the UI responsive.
///
/// Returns true if reconnection completed within the timeout, false otherwise.
pub fn wait_for_reconnect(timeout_ms: u64) -> bool {
    let event_loop = unsafe { QEventLoop::new_0a() };
    let start = std::time::Instant::now();
    let timeout = std::time::Duration::from_millis(timeout_ms);

    while !RECONNECT_COMPLETE.load(Ordering::SeqCst) {
        if start.elapsed() > timeout {
            return false;
        }
        unsafe { event_loop.process_events(); }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    true
}

/// This function is the one that actually handles the WebSocket communication with the server.
pub async fn websocket_loop(mut receiver: UnboundedReceiver<Outgoing<Response>>) {
    let base_url = "ws://localhost:45127/ws";
    let mut current_session_id: Option<u64> = None;

    let mut response_channels = HashMap::new();
    let mut api_channels: HashMap<u64, Sender<RpcResponse>> = HashMap::new();

    loop {
        // Exit cleanly once the UI has requested disconnection — the server tears the
        // connection down hard, so without this we'd loop forever trying to reconnect.
        if SHUTDOWN_REQUESTED.load(Ordering::SeqCst) {
            info!("Shutdown requested, exiting WebSocket loop.");
            return;
        }

        // Check if a reconnection was requested.
        if RECONNECT_REQUESTED.swap(false, Ordering::SeqCst) {
            current_session_id = RECONNECT_SESSION_ID.write().unwrap().take();
            info!("Reconnection requested to session: {:?}", current_session_id);
            // Clear any pending response channels from the old connection.
            response_channels.clear();
            api_channels.clear();
        }

        // Build the URL with optional session ID parameter.
        let url = match current_session_id {
            Some(id) => format!("{}?session_id={}", base_url, id),
            None => base_url.to_string(),
        };
        info!("Connecting to WebSocket server at {}...", url);

        let config = WebSocketConfig::default()
            .max_message_size(Some(67108864 * 2 * 2 * 2 * 2 * 2 * 2 * 2 * 2))   // 16GB
            .max_frame_size(Some(67108864 * 2 * 2 * 2 * 2 * 2 * 2 * 2 * 2));    // 16GB

        match connect_async_with_config(&url, Some(config), false).await {
            Ok((mut ws_stream, _)) => {
                info!("WebSocket connected!");

                // Signal that reconnection is complete.
                RECONNECT_COMPLETE.store(true, Ordering::SeqCst);

                loop {
                    tokio::select! {

                        // Periodically check for reconnection requests (every 100ms).
                        _ = tokio::time::sleep(tokio::time::Duration::from_millis(100)) => {
                            if RECONNECT_REQUESTED.load(Ordering::SeqCst) {
                                info!("Reconnection requested, closing current connection...");
                                let _ = ws_stream.close(None).await;
                                break;
                            }
                        }

                        // New command from the UI. The server must have the current settings before running it.
                        Some(outgoing) = receiver.recv() => {
                            if !send_changed_settings(&mut ws_stream).await {
                                error!("Failed to send the settings over WebSocket.");
                                break;
                            }

                            let json = match outgoing {
                                Outgoing::Legacy(message, sender) => {
                                    response_channels.insert(message.id, sender);
                                    serde_json::to_string(&message).unwrap()
                                }
                                Outgoing::Api(request, sender) => {
                                    api_channels.insert(request.id, sender);
                                    serde_json::to_string(&request).unwrap()
                                }
                            };
                            if ws_stream.send(WsMessage::Text(json.into())).await.is_err() {
                                error!("Failed to send message over WebSocket.");
                                break;
                            }
                        }

                        // Response from the server.
                        Some(msg) = ws_stream.next() => {
                            match msg {
                                Ok(WsMessage::Text(text)) => {

                                    // Version 2 messages. Notifications have no ID, so they don't parse as responses.
                                    if text.contains("\"jsonrpc\"") {
                                        if let Ok(response) = serde_json::from_str::<RpcResponse>(&text) {
                                            if let Some(sender) = api_channels.remove(&response.id) {
                                                let _ = sender.send(response);
                                            } else if let RpcOutcome::Error(error) = response.outcome {
                                                error!("The server rejected a request [ID {}]: {}", response.id, error.message);
                                            }
                                        }
                                        continue;
                                    }

                                    match serde_json::from_str::<IpcMessage<Response>>(&text) {
                                        Ok(msg) => {
                                            // Handle SessionConnected message specially to update current session ID.
                                            if let Response::SessionConnected(session_id) = &msg.data {
                                                info!("Connected to session ID: {}", session_id);
                                                *CURRENT_SESSION_ID.write().unwrap() = Some(*session_id);

                                                // A new or adopted session doesn't have our settings yet.
                                                mark_settings_changed();
                                                if !send_changed_settings(&mut ws_stream).await {
                                                    error!("Failed to send the settings over WebSocket.");
                                                    break;
                                                }
                                                continue;
                                            }

                                            if let Some(sender) = response_channels.remove(&msg.id) {
                                                let _ = sender.send(msg.data);
                                            } else {
                                                error!("Received response [ID {}] but no channel was waiting for it.", msg.id);
                                            }
                                        }
                                        Err(error) => error!("Failed to deserialize response: {}", error),
                                    }
                                }
                                Ok(WsMessage::Close(_)) => {
                                    info!("WebSocket closed by server.");
                                    break;
                                }
                                Err(error) => {
                                    if SHUTDOWN_REQUESTED.load(Ordering::SeqCst) {
                                        info!("WebSocket closed during shutdown.");
                                    } else {
                                        warn!("WebSocket read loop ended: {}", error);
                                    }
                                    break;
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
            Err(error) => {
                info!("Failed to connect to WebSocket server: {}. Retrying in 5 seconds...", error);
                tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
            }
        }
    }
}

/// Sends the settings to the session of the server if they changed since they were last sent.
///
/// Returns false if the message couldn't be sent.
async fn send_changed_settings<S: SinkExt<WsMessage> + Unpin>(ws_stream: &mut S) -> bool {
    let Some(settings) = take_changed_settings() else {
        return true;
    };

    let id = MESSAGE_ID_COUNTER.fetch_add(1, Ordering::SeqCst);
    let json = match RpcRequest::new(id, &Configure { settings }).and_then(|request| serde_json::to_string(&request)) {
        Ok(json) => json,
        Err(error) => {
            error!("Failed to serialize the settings: {error}");
            return true;
        }
    };

    ws_stream.send(WsMessage::Text(json.into())).await.is_ok()
}
