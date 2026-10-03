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
use base64::{Engine, engine::general_purpose::STANDARD};
use crossbeam::channel::{Receiver, Sender, unbounded};
use futures::{SinkExt, StreamExt};
use tokio::sync::mpsc::{unbounded_channel, UnboundedSender, UnboundedReceiver};
use tokio_tungstenite::{connect_async_with_config, tungstenite::protocol::{Message as WsMessage, WebSocketConfig}};

use std::collections::{BTreeSet, HashMap};
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

use rpfm_ipc::api::{ApiError, JOB_UPDATED_NOTIFICATION, Request, RpcNotification, RpcOutcome, RpcRequest, RpcResponse};
use rpfm_ipc::api::jobs::{JobStarted, JobState, JobStatus, WaitForJob};
use rpfm_ipc::api::files::{CopyFiles, FileData, FileRef, FileSource, GetFilesFromAllSources, GetFilesInfo, ListFiles, ReadFile, ReadFormat, WriteFile};
use rpfm_ipc::api::packs::{GetPackInfo, PackDetails, PackSummary, SavePack};
use rpfm_ipc::helpers::{ContainerInfo, DataSource, RFileInfo};

use rpfm_lib::files::{ContainerPath, RFile, RFileDecoded};
use rpfm_ipc::api::session::{Configure, Disconnect, GetSessionStatus, SESSION_CONNECTED_NOTIFICATION, SessionConnected};
use rpfm_ipc::api::packs::OperationalMode;

use rpfm_telemetry::*;

use crate::CENTRAL_COMMAND;
use crate::utils::file_paths;
use crate::settings_ui::backend::{mark_settings_changed, take_changed_settings};

pub mod server;

/// Atomic counter for generating unique message IDs.
static MESSAGE_ID_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Global variable to hold the current session ID we are connected to.
pub static CURRENT_SESSION_ID: std::sync::LazyLock<Arc<RwLock<Option<u64>>>> = std::sync::LazyLock::new(|| Arc::new(RwLock::new(None)));

/// Global variable to hold the session ID to reconnect to. When set, the WebSocket loop will
/// disconnect and reconnect to the specified session.
pub static RECONNECT_SESSION_ID: std::sync::LazyLock<Arc<RwLock<Option<u64>>>> = std::sync::LazyLock::new(|| Arc::new(RwLock::new(None)));

/// Last state the server notified of each job that hasn't ended yet, by job ID.
static JOB_STATES: std::sync::LazyLock<RwLock<HashMap<u64, JobState>>> = std::sync::LazyLock::new(|| RwLock::new(HashMap::new()));

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

/// Sends the requests of the UI to the WebSocket loop, which forwards them to the server.
pub struct CentralCommand {
    sender: UnboundedSender<(RpcRequest, Sender<RpcResponse>)>,
    try_lock: AtomicBool,
}

//-------------------------------------------------------------------------------//
//                              Implementations
//-------------------------------------------------------------------------------//

/// Default implementation of `CentralCommand`.
impl Default for CentralCommand {
    fn default() -> Self {
        let (sender, _) = unbounded_channel();
        let try_lock = AtomicBool::new(false);
        Self {
            sender,
            try_lock,
        }
    }
}

impl CentralCommand {

    /// This function initializes a new central command, and returns the receiver for the WebSocket loop.
    ///
    /// Use it to replace the default one on runtime.
    pub fn init() -> (Self, UnboundedReceiver<(RpcRequest, Sender<RpcResponse>)>) {
        let (sender, receiver) = unbounded_channel();
        let try_lock = AtomicBool::new(false);
        (Self {
            sender,
            try_lock,
        }, receiver)
    }

    /// This function sends a request of the server's API.
    ///
    /// It returns the receiver which will receive the response.
    pub fn call<R: Request>(&self, request: &R) -> Receiver<RpcResponse> {
        let (sender_back, receiver_back) = unbounded();
        let id = MESSAGE_ID_COUNTER.fetch_add(1, Ordering::SeqCst);
        match RpcRequest::new(id, request) {
            Ok(request) => if self.sender.send((request, sender_back)).is_err() {
                panic!("{THREADS_SENDER_ERROR}");
            },
            Err(error) => {
                let _ = sender_back.send(RpcResponse::new(id, Err(ApiError::InvalidParams(error.to_string()))));
            }
        }

        receiver_back
    }

    /// Waits for a response while keeping the UI responsive. Use it for heavy tasks.
    ///
    /// Returns `None` if the WebSocket loop dropped the request, like when the connection is lost.
    ///
    /// NOTE: Beware of other events triggering when this keeps the UI enabled. It can lead to crashes.
    pub fn recv_try(&self, receiver: &Receiver<RpcResponse>) -> Option<RpcResponse> {
        let event_loop = unsafe { QEventLoop::new_0a() };

        // Lock this function after the first execution, until it gets freed again.
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

        // If we're locked due to another execution, just wait.
        else {
            info!("Race condition avoided? Two items calling recv_try on the same execution crashes.");
            receiver.recv().ok()
        }
    }
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

/// Returns the last state the server notified of a job, or `None` if it ended or there were no notifications for it yet.
pub fn job_state(job: u64) -> Option<JobState> {
    JOB_STATES.read().unwrap().get(&job).cloned()
}

/// Keeps the state of a job from a notification, forgetting the jobs that ended.
fn store_job_state(status: JobStatus) {
    let mut states = JOB_STATES.write().unwrap();
    if status.state.has_ended() {
        states.remove(&status.job);
    } else {
        states.insert(status.job, status.state);
    }
}

/// Sends a request of the server's API without waiting for its response, which arrives through the returned receiver.
pub fn send_api<R: Request>(request: &R) -> Receiver<RpcResponse> {
    CENTRAL_COMMAND.read().unwrap().call(request)
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
    CENTRAL_COMMAND.read().unwrap().recv_try(&receiver).ok_or_else(|| anyhow!("{THREADS_COMMUNICATION_ERROR}Disconnected"))
}

/// Turns the response of a request into its result. Errors are [`ApiError`]s, so callers can downcast them.
pub fn api_result<T: serde::de::DeserializeOwned>(response: RpcResponse) -> Result<T> {
    match response.outcome {
        RpcOutcome::Result(value) => serde_json::from_value(value).map_err(From::from),
        RpcOutcome::Error(error) => Err(ApiError::from(error).into()),
    }
}

/// Returns the details of an open pack.
pub fn pack_details(pack_key: &str) -> Result<PackDetails> {
    call_api(&GetPackInfo { pack: pack_key.to_owned() })
}

/// Returns the operational mode of an open pack, or the normal one if the pack isn't open.
pub fn pack_operational_mode(pack_key: &str) -> OperationalMode {
    pack_details(pack_key).map(|details| details.operational_mode).unwrap_or_default()
}

/// Saves an open pack, keeping the settings of the session for the rest of the options.
///
/// # Arguments
///
/// * `pack_key` - Key of the pack.
/// * `path` - Path to save the pack to. If `None`, the pack is saved to its current path.
/// * `clean` - If files that failed to decode are removed before saving.
///
/// # Returns
///
/// The info of the pack after saving it.
pub fn save_pack(pack_key: &str, path: Option<PathBuf>, clean: bool) -> Result<ContainerInfo> {
    let request = SavePack { pack: pack_key.to_owned(), path, clean, disable_uuid_regeneration: None, allow_editing_ca_packs: None };
    call_api_async(&request)?;
    pack_details(pack_key).map(|details| ContainerInfo::from(&details))
}

/// Replaces the contents of a file of an open pack with a decoded file, like the one a view edits.
pub fn save_decoded_file(pack_key: &str, path: &str, decoded: &RFileDecoded) -> Result<()> {
    let contents = FileData::Decoded { data: serde_json::to_value(decoded)? };
    call_api_async(&WriteFile { pack: pack_key.to_owned(), path: path.to_owned(), contents }).map(|_| ())
}

/// Returns the bytes of a file.
pub fn raw_file_data(file: FileRef) -> Result<Vec<u8>> {
    match call_api(&ReadFile { file, format: ReadFormat::Raw })?.contents {
        FileData::Raw { base64 } => STANDARD.decode(base64).map_err(From::from),
        _ => Err(anyhow!("The server didn't return the bytes of the file.")),
    }
}

/// Returns the info of files of an open pack: the ones at the provided paths, or all of them. Empty if the pack isn't open.
pub fn files_info(pack_key: &str, paths: Option<Vec<String>>) -> Vec<RFileInfo> {
    call_api(&GetFilesInfo { pack: pack_key.to_owned(), paths }).map(|info| info.files).unwrap_or_default()
}

/// Returns if a file exists in an open pack.
pub fn file_exists(pack_key: &str, path: &str) -> bool {
    !files_info(pack_key, Some(vec![path.to_owned()])).is_empty()
}

/// Returns if a folder exists in an open pack.
pub fn folder_exists(pack_key: &str, path: &str) -> bool {
    let request = ListFiles {
        source: FileSource::Pack(pack_key.to_owned()),
        prefix: format!("{}/", path.trim_end_matches('/')),
        recursive: true,
        file_types: None,
        offset: 0,
        limit: Some(1),
    };

    call_api(&request).is_ok_and(|list| list.total > 0)
}

/// Returns the paths of the files under a folder in the open packs, the parent packs and the game files, sorted.
pub fn file_paths_in_all_sources(folder: &str) -> Vec<String> {
    let sources = [FileSource::ParentFiles, FileSource::GameFiles].into_iter()
        .chain(open_packs().into_iter().map(|pack| FileSource::Pack(pack.key)));

    let paths = sources.flat_map(|source| {
        let request = ListFiles { source, prefix: format!("{}/", folder.trim_end_matches('/')), recursive: true, file_types: None, offset: 0, limit: Some(usize::MAX) };
        call_api(&request).map(|list| list.files).unwrap_or_default()
    });

    paths.map(|file| file.path).collect::<BTreeSet<_>>().into_iter().collect()
}

/// Copies files and folders from any source into an open pack, keeping their paths.
///
/// # Returns
///
/// The paths of the files added to the pack.
pub fn copy_files(source: FileSource, paths: &[ContainerPath], pack: &str) -> Result<Vec<ContainerPath>> {
    let request = CopyFiles { source, paths: paths.iter().map(|path| path.path_raw().to_owned()).collect(), pack: pack.to_owned() };
    Ok(file_paths(call_api(&request)?.added))
}

/// Returns the files and folders found in the open packs, the parent packs and the game files, by source and path.
///
/// # Arguments
///
/// * `paths` - Paths of the files and folders.
/// * `lowercase_paths` - If the returned paths are lowercased.
pub fn files_from_all_sources(paths: Vec<ContainerPath>, lowercase_paths: bool) -> HashMap<DataSource, HashMap<String, RFile>> {
    call_api(&GetFilesFromAllSources { paths, lowercase_paths }).map(|files| files.files).unwrap_or_default()
}

/// Returns the packs open in the session.
pub fn open_packs() -> Vec<PackSummary> {
    call_api(&GetSessionStatus {}).map(|status| status.packs).unwrap_or_default()
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
    let _ = CENTRAL_COMMAND.read().unwrap().call(&Disconnect {});
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
pub async fn websocket_loop(mut receiver: UnboundedReceiver<(RpcRequest, Sender<RpcResponse>)>) {
    let base_url = "ws://localhost:45127/ws";
    let mut current_session_id: Option<u64> = None;

    let mut response_channels: HashMap<u64, Sender<RpcResponse>> = HashMap::new();

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

                        // New request from the UI. The server must have the current settings before running it.
                        Some((request, sender)) = receiver.recv() => {
                            if !send_changed_settings(&mut ws_stream).await {
                                error!("Failed to send the settings over WebSocket.");
                                break;
                            }

                            response_channels.insert(request.id, sender);
                            let json = serde_json::to_string(&request).unwrap();
                            if ws_stream.send(WsMessage::Text(json.into())).await.is_err() {
                                error!("Failed to send message over WebSocket.");
                                break;
                            }
                        }

                        // Response from the server.
                        Some(msg) = ws_stream.next() => {
                            match msg {
                                Ok(WsMessage::Text(text)) => {

                                    // Responses have an ID, notifications don't.
                                    if let Ok(response) = serde_json::from_str::<RpcResponse>(&text) {
                                        if let Some(sender) = response_channels.remove(&response.id) {
                                            let _ = sender.send(response);
                                        } else if let RpcOutcome::Error(error) = response.outcome {
                                            error!("The server rejected a request [ID {}]: {}", response.id, error.message);
                                        }
                                    }

                                    else if let Ok(notification) = serde_json::from_str::<RpcNotification>(&text) {
                                        if notification.method == JOB_UPDATED_NOTIFICATION {
                                            if let Ok(status) = serde_json::from_value::<JobStatus>(notification.params) {
                                                store_job_state(status);
                                            }
                                        }

                                        else if notification.method == SESSION_CONNECTED_NOTIFICATION {
                                            if let Ok(connected) = serde_json::from_value::<SessionConnected>(notification.params) {
                                                info!("Connected to session ID: {}", connected.session_id);
                                                *CURRENT_SESSION_ID.write().unwrap() = Some(connected.session_id);

                                                // A new or adopted session doesn't have our settings yet.
                                                mark_settings_changed();
                                                if !send_changed_settings(&mut ws_stream).await {
                                                    error!("Failed to send the settings over WebSocket.");
                                                    break;
                                                }
                                            }
                                        }
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
