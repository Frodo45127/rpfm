//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Per-client session state and lifecycle.
//!
//! Each WebSocket connection (and each MCP client) is wrapped in a [`Session`]
//! managed by a [`SessionManager`]. Sessions are isolated: open packs in one
//! session aren't visible from another, and each one owns a dedicated
//! background thread (see [`crate::background_thread`]) that processes its
//! commands serially, with its own settings. Sessions start with the settings
//! in the settings file, and clients owning their own settings (the UI) replace
//! them with a `session.configure` request.
//!
//! ## Lifecycle
//!
//! 1. **Create.** A new session gets a unique [`SessionId`] from the manager
//!    plus a fresh background thread of its own.
//! 2. **Connect / disconnect.** Clients increment [`Session::connect`] on
//!    attach and [`Session::disconnect`] on detach. The connection count is
//!    what the timeout logic watches.
//! 3. **Reconnect.** A client can pass its previous `session_id` back on the
//!    next WebSocket handshake to adopt the same session and recover its
//!    in-memory state. See [`SessionManager::get_or_create_session`].
//! 4. **Timeout.** When the connection count drops to zero, the session
//!    enters a [`DEFAULT_SESSION_TIMEOUT_SECS`]-long grace period. Reconnects
//!    cancel the timeout; otherwise the cleanup task removes the session and
//!    its background thread exits.
//! 5. **Empty manager → process exit.** When the last session is removed the
//!    server process terminates, so no orphaned backend lingers in the
//!    background.
//!
//! ## The MCP session
//!
//! MCP clients share one session ([`SessionManager::mcp_session`]), whatever
//! their protocol version: newer MCP versions have no sessions of their own,
//! and the older ones have no disconnect signal the server can rely on. The
//! MCP session is never counted as connected; instead, the periodic cleanup
//! task removes it once no request has been sent through it for
//! [`DEFAULT_SESSION_TIMEOUT_SECS`]. The next MCP request creates a new one.

use tokio::sync::mpsc::{error::SendError, unbounded_channel, UnboundedReceiver, UnboundedSender};
use tokio::time::{Duration, Instant};

use std::collections::{BTreeSet, HashMap};
use std::sync::{Arc, Mutex, RwLock, atomic::{AtomicBool, AtomicU32, AtomicUsize, Ordering}};

use rpfm_ipc::api::{ApiError, Done, Request, RpcOutcome, RpcRequest, RpcResponse};
use rpfm_ipc::api::diagnostics::RunDiagnostics;
use rpfm_ipc::api::jobs::JobStarted;
use rpfm_ipc::api::session::Configure;
use rpfm_ipc::helpers::SessionInfo;
use rpfm_ipc::settings::Settings;
use rpfm_telemetry::{error, info};

use crate::api;
use crate::background_thread;
use crate::apply_telemetry_settings;
use crate::jobs::{self, JobRegistry};

/// Error messages for session communication.
pub const SESSION_SENDER_ERROR: &str = "Error in session communication system. Sender failed to send message.";

/// Default session timeout in seconds (5 minutes).
pub const DEFAULT_SESSION_TIMEOUT_SECS: u64 = 300;

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// Unique identifier for a session.
pub type SessionId = u64;

/// A message for a session's background thread, with where to send its response.
#[derive(Debug)]
pub enum SessionMessage {

    /// A request.
    Api(RpcRequest, UnboundedSender<RpcResponse>),

    /// A request that runs as a job, with the ID of its job.
    Job(u64, RpcRequest),

    /// Stops the background thread.
    Exit,
}

/// Manages all active sessions.
///
/// Provides thread-safe access to create, retrieve, and remove sessions.
/// Sessions persist for a configurable timeout after all clients disconnect.
pub struct SessionManager {

    /// Map of session IDs to managed sessions.
    sessions: Mutex<HashMap<SessionId, ManagedSession>>,

    /// Counter for generating unique session IDs.
    next_id: Mutex<SessionId>,

    /// Session timeout duration.
    timeout: Duration,
}

/// Internal state for a managed session.
struct ManagedSession {

    /// The session itself.
    session: Arc<Session>,

    /// When the last client disconnected (None if clients are connected).
    disconnected_at: Option<Instant>,
}

/// A session represents a single client's connection state.
///
/// Each session has its own background thread for processing commands,
/// ensuring complete isolation between clients.
pub struct Session {

    /// Unique identifier for this session.
    id: SessionId,

    /// Whether this is the session of the MCP clients, removed by inactivity instead of by connection count.
    is_mcp: bool,

    /// Instant of the last command sent to this session's background thread.
    ///
    /// Only updated by [`Session::call`], so it reflects actual work, not
    /// transport-level pings.
    last_activity: Mutex<Instant>,

    /// Sender to communicate with this session's background thread.
    sender: UnboundedSender<SessionMessage>,

    /// Number of active connections using this session.
    connection_count: AtomicU32,

    /// Whether this session has been marked for shutdown.
    shutdown_requested: AtomicBool,

    /// Names of the pack files currently open in this session.
    pack_names: RwLock<Vec<String>>,

    /// Jobs of this session.
    jobs: Arc<JobRegistry>,

    /// Settings the session runs with. Behind an `Arc` so requests can hold them without copying.
    settings: RwLock<Arc<Settings>>,

    /// Amount of requests sent to the background thread that it hasn't started yet.
    queued: AtomicUsize,

    /// The last diagnostics check requested, so a new one can replace it.
    diagnostics_check: Mutex<Option<DiagnosticsCheck>>,
}

/// A diagnostics check requested to a session.
#[derive(Debug)]
struct DiagnosticsCheck {

    /// ID of the job of the check.
    job: u64,

    /// Paths the check checks. If empty, it checks everything.
    paths: Vec<String>,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

impl Session {

    /// Create a new session with its own background thread.
    ///
    /// `is_mcp` marks the session of the MCP clients, which is removed by
    /// inactivity instead of by connection count (see [`Session::is_mcp`]).
    pub fn new(id: SessionId, is_mcp: bool) -> Arc<Self> {
        let (sender, receiver) = unbounded_channel();

        let session = Arc::new(Self {
            id,
            is_mcp,
            last_activity: Mutex::new(Instant::now()),
            sender,
            connection_count: AtomicU32::new(0),
            shutdown_requested: AtomicBool::new(false),
            pack_names: RwLock::new(Vec::new()),
            jobs: Arc::new(JobRegistry::default()),
            settings: RwLock::new(Arc::new(Settings::init(false))),
            queued: AtomicUsize::new(0),
            diagnostics_check: Mutex::new(None),
        });

        // Requests do heavy blocking work, so they run on their own thread instead of stalling a worker of the async runtime.
        let session_clone = session.clone();
        let spawned = std::thread::Builder::new().name(format!("session-{id}")).spawn(move || {
            info!("Session {} background thread starting...", id);
            background_thread::background_loop(receiver, session_clone);
            info!("Session {} background thread terminated.", id);
        });

        if let Err(error) = spawned {
            error!("Session {}: failed to start its background thread: {}", id, error);
        }

        session
    }

    /// Get the session ID.
    pub fn id(&self) -> SessionId {
        self.id
    }

    /// Whether this is the session of the MCP clients.
    pub fn is_mcp(&self) -> bool {
        self.is_mcp
    }

    /// Record that a command was just sent to this session's background thread.
    fn touch(&self) {
        *self.last_activity.lock().unwrap() = Instant::now();
    }

    /// Get the instant of the last command sent to this session.
    pub fn last_activity(&self) -> Instant {
        *self.last_activity.lock().unwrap()
    }

    /// Increment the connection count.
    pub fn connect(&self) {
        self.connection_count.fetch_add(1, Ordering::SeqCst);
    }

    /// Decrement the connection count.
    pub fn disconnect(&self) {
        self.connection_count.fetch_sub(1, Ordering::SeqCst);
    }

    /// Get the current connection count.
    pub fn connection_count(&self) -> u32 {
        self.connection_count.load(Ordering::SeqCst)
    }

    /// Check if shutdown has been requested.
    pub fn is_shutdown_requested(&self) -> bool {
        self.shutdown_requested.load(Ordering::SeqCst)
    }

    /// Get the pack names for this session.
    pub fn pack_names(&self) -> Vec<String> {
        self.pack_names.read().unwrap().clone()
    }

    /// Add a pack name to this session.
    pub fn add_pack_name(&self, name: &str) {
        let mut names = self.pack_names.write().unwrap();
        if !names.contains(&name.to_string()) {
            names.push(name.to_string());
        }
    }

    /// Remove a pack name from this session.
    pub fn remove_pack_name(&self, name: &str) {
        let mut names = self.pack_names.write().unwrap();
        names.retain(|n| n != name);
    }

    /// Shutdown this session by stopping its background thread.
    pub fn shutdown(&self) {
        info!("Session {} shutting down...", self.id);

        if self.shutdown_requested.swap(true, Ordering::SeqCst) {
            info!("Session {} already marked for shutdown before...", self.id);
            return;
        }

        // Ignore errors, as the channel may be already closed.
        let _ = self.sender.send(SessionMessage::Exit);
    }

    /// Returns the jobs of this session.
    pub fn jobs(&self) -> &Arc<JobRegistry> {
        &self.jobs
    }

    /// Returns the settings of this session.
    pub fn settings(&self) -> Arc<Settings> {
        self.settings.read().unwrap().clone()
    }

    /// Replaces the settings of this session, applying their telemetry choices to the whole process.
    fn configure(&self, request: RpcRequest) -> RpcResponse {
        let result = api::parse_params::<Configure>(request.params).and_then(|configure| {
            apply_telemetry_settings(&configure.settings);
            *self.settings.write().unwrap() = Arc::new(configure.settings);
            serde_json::to_value(Done {}).map_err(|error| ApiError::Internal(error.to_string()))
        });

        RpcResponse::new(request.id, result)
    }

    /// Send a request to this session's background thread.
    ///
    /// Configuration and job control requests are answered without waiting for the background thread,
    /// and jobs are answered right away with their ID, before they run.
    ///
    /// Returns a receiver to get the response.
    pub fn call(&self, request: RpcRequest) -> UnboundedReceiver<RpcResponse> {
        self.touch();
        let (sender_back, receiver_back) = unbounded_channel();

        if request.method == Configure::METHOD {
            let _ = sender_back.send(self.configure(request));
            return receiver_back;
        }

        if jobs::is_job_control_method(&request.method) {
            jobs::handle_request(self.jobs.clone(), request, sender_back);
            return receiver_back;
        }

        if api::is_stateless_method(&request.method) {
            let settings = self.settings();
            tokio::spawn(async move {
                let id = request.id;
                let response = tokio::task::spawn_blocking(move || api::dispatch_stateless(request, &settings)).await
                    .unwrap_or_else(|error| RpcResponse::new(id, Err(ApiError::Internal(format!("The background task failed: {error}")))));

                let _ = sender_back.send(response);
            });
            return receiver_back;
        }

        if api::is_job_method(&request.method) {
            let job = self.jobs.create(&request.method);
            let id = request.id;
            let request = if request.method == RunDiagnostics::METHOD {
                self.replace_diagnostics_check(job, request)
            } else {
                request
            };

            if let Err(error) = self.enqueue(SessionMessage::Job(job, request)) {
                let message = format!("{SESSION_SENDER_ERROR}: {error}");
                info!("{message}");
                self.jobs.finish(job, RpcOutcome::Error(ApiError::Internal(message).into()));
            }

            let started = serde_json::to_value(JobStarted { job }).map_err(|error| ApiError::Internal(error.to_string()));
            let _ = sender_back.send(RpcResponse::new(id, started));
            return receiver_back;
        }

        if let Err(error) = self.enqueue(SessionMessage::Api(request, sender_back)) {
            let message = format!("{SESSION_SENDER_ERROR}: {error}");
            info!("{message}");
            if let SessionMessage::Api(request, sender_back) = error.0 {
                let _ = sender_back.send(RpcResponse::new(request.id, Err(ApiError::Internal(message))));
            }
        }
        receiver_back
    }

    /// Sends a request to the background thread, counting it as queued until the thread starts it.
    fn enqueue(&self, message: SessionMessage) -> Result<(), SendError<SessionMessage>> {
        self.queued.fetch_add(1, Ordering::SeqCst);
        self.sender.send(message).inspect_err(|_| { self.queued.fetch_sub(1, Ordering::SeqCst); })
    }

    /// Marks a request sent to the background thread as started.
    pub fn dequeued(&self) {
        self.queued.fetch_sub(1, Ordering::SeqCst);
    }

    /// Returns if there are requests waiting for the background thread.
    pub fn has_queued_requests(&self) -> bool {
        self.queued.load(Ordering::SeqCst) > 0
    }

    /// Makes a new diagnostics check replace the last one, if it hasn't ended yet.
    ///
    /// The new check also checks the paths the replaced one would have checked. A queued replaced check is
    /// cancelled, and a running one is cancelled once it stops for the new one (see [`Self::requeue_diagnostics_check`]).
    ///
    /// # Arguments
    ///
    /// * `job` - Job of the new check.
    /// * `request` - Request of the new check.
    ///
    /// # Returns
    ///
    /// The request of the new check, with the paths of the replaced one added.
    fn replace_diagnostics_check(&self, job: u64, mut request: RpcRequest) -> RpcRequest {
        let Ok(mut params) = api::parse_params::<RunDiagnostics>(request.params.clone()) else {
            return request;
        };

        let mut last_check = self.diagnostics_check.lock().unwrap();
        let pending = last_check.take().filter(|check| self.jobs.status(check.job).is_some_and(|status| !status.state.has_ended()));
        if let Some(pending) = pending {

            // No paths means everything, so a full check covers any other.
            params.paths = if pending.paths.is_empty() || params.paths.is_empty() {
                vec![]
            } else {
                pending.paths.into_iter().chain(params.paths).collect::<BTreeSet<_>>().into_iter().collect()
            };

            let _ = self.jobs.cancel(pending.job);
            if let Ok(params) = serde_json::to_value(&params) {
                request.params = params;
            }
        }

        *last_check = Some(DiagnosticsCheck { job, paths: params.paths });
        request
    }

    /// Queues again a diagnostics check that stopped to let other requests run, or cancels it if a newer one replaced it.
    ///
    /// # Arguments
    ///
    /// * `job` - Job of the check.
    /// * `request` - Request of the check.
    pub fn requeue_diagnostics_check(&self, job: u64, request: RpcRequest) {
        let last_check = self.diagnostics_check.lock().unwrap();
        if last_check.as_ref().is_some_and(|check| check.job == job) && self.jobs.requeue(job) {
            if self.enqueue(SessionMessage::Job(job, request)).is_err() {
                self.jobs.finish_cancelled(job);
            }
        } else {
            self.jobs.finish_cancelled(job);
        }
    }
}

impl Default for SessionManager {
    fn default() -> Self {
        Self {
            sessions: Mutex::new(HashMap::new()),
            next_id: Mutex::new(1),
            timeout: Duration::from_secs(DEFAULT_SESSION_TIMEOUT_SECS),
        }
    }
}

impl SessionManager {

    /// Create a new session and return a reference to it.
    pub fn create_session(&self) -> Arc<Session> {
        self.create_session_internal(false)
    }

    /// Returns the session shared by all MCP clients, creating it if there is none.
    ///
    /// It's never counted as connected. Instead, [`SessionManager::cleanup_expired_sessions`] removes it
    /// once no request has been sent through it for [`DEFAULT_SESSION_TIMEOUT_SECS`].
    pub fn mcp_session(&self) -> Arc<Session> {
        let mut sessions = self.sessions.lock().unwrap();
        let existing = sessions.values()
            .find(|managed| managed.session.is_mcp() && !managed.session.is_shutdown_requested())
            .map(|managed| managed.session.clone());

        existing.unwrap_or_else(|| self.insert_session(&mut sessions, true))
    }

    fn create_session_internal(&self, is_mcp: bool) -> Arc<Session> {
        let mut sessions = self.sessions.lock().unwrap();
        self.insert_session(&mut sessions, is_mcp)
    }

    /// Creates a session and adds it to the provided sessions, which must be the locked [`SessionManager::sessions`].
    fn insert_session(&self, sessions: &mut HashMap<SessionId, ManagedSession>, is_mcp: bool) -> Arc<Session> {
        let id = {
            let mut next_id = self.next_id.lock().unwrap();
            let id = *next_id;
            *next_id += 1;
            id
        };

        let session = Session::new(id, is_mcp);

        // The MCP session starts disconnected: its lifetime is governed by
        // inactivity, not by the connection count.
        if !is_mcp {
            session.connect();
        }

        sessions.insert(id, ManagedSession {
            session: session.clone(),
            disconnected_at: None,
        });

        info!("Created new {} session with ID: {}", if is_mcp { "MCP" } else { "WebSocket" }, id);
        session
    }

    /// Get an existing session by ID, or create a new one if the ID doesn't exist.
    ///
    /// If `session_id` is `Some`, attempts to retrieve that session.
    /// If the session doesn't exist or `session_id` is `None`, creates a new session.
    ///
    /// Returns the session and whether it was newly created.
    pub fn get_or_create_session(&self, session_id: Option<SessionId>) -> (Arc<Session>, bool) {
        if let Some(id) = session_id {
            let mut sessions = self.sessions.lock().unwrap();
            if let Some(managed) = sessions.get_mut(&id) {

                // Check if the session is still valid (not shut down).
                if !managed.session.is_shutdown_requested() {
                    managed.session.connect();
                    managed.disconnected_at = None;
                    info!("Client reconnected to existing session {}", id);
                    return (managed.session.clone(), false);
                }
            }
        }

        // Either no session_id provided, or session not found/invalid.
        // Create a new session.
        (self.create_session(), true)
    }

    /// Get a session by ID without incrementing the connection count.
    pub fn get_session(&self, id: SessionId) -> Option<Arc<Session>> {
        let sessions = self.sessions.lock().unwrap();
        sessions.get(&id).map(|m| m.session.clone())
    }

    /// Mark a session as disconnected by a client.
    ///
    /// If no more clients are connected, starts the timeout countdown.
    /// The session will be removed after the timeout unless a client reconnects.
    pub fn client_disconnected(manager: Arc<Self>, id: SessionId) {
        let should_schedule_cleanup = {
            let mut sessions = manager.sessions.lock().unwrap();
            if let Some(managed) = sessions.get_mut(&id) {
                managed.session.disconnect();

                if managed.session.connection_count() == 0 {
                    managed.disconnected_at = Some(Instant::now());
                    info!("Session {} has no active connections, will timeout in {:?}", id, manager.timeout);
                    true
                } else {
                    false
                }
            } else {
                false
            }
        };

        if should_schedule_cleanup {
            Self::schedule_cleanup(manager.clone(), id);
        }
    }

    /// Schedule a cleanup check for a session after the timeout period.
    fn schedule_cleanup(manager: Arc<Self>, id: SessionId) {
        let timeout = manager.timeout;
        let manager = manager.clone();

        tokio::spawn(async move {
            tokio::time::sleep(timeout).await;

            // A client may have reconnected during the grace period. Only
            // remove the session if it is still disconnected, otherwise the
            // scheduled task would tear down a live session.
            let still_disconnected = {
                let sessions = manager.sessions.lock().unwrap();
                sessions.get(&id).is_some_and(|managed| managed.session.connection_count() == 0)
            };

            if still_disconnected {
                info!("Session {} timeout check triggered, removing session", id);
                manager.remove_session(id);

                // Check if this was the last session and shutdown the server if so.
                if manager.session_count() == 0 {
                    info!("No more active sessions, shutting down server...");
                    std::process::exit(0);
                }
            } else {
                info!("Session {} reconnected before timeout check, skipping cleanup", id);
            }
        });
    }

    /// Perform cleanup of expired sessions.
    ///
    /// This should be called periodically or after timeout events.
    pub fn cleanup_expired_sessions(&self) {
        let now = Instant::now();
        let mut to_remove = Vec::new();

        {
            let sessions = self.sessions.lock().unwrap();
            for (id, managed) in sessions.iter() {
                if managed.session.is_mcp() {

                    // MCP clients have no disconnect signal, so their session is removed by inactivity.
                    if now.duration_since(managed.session.last_activity()) >= self.timeout {
                        to_remove.push(*id);
                    }
                } else if let Some(disconnected_at) = managed.disconnected_at {
                    if now.duration_since(disconnected_at) >= self.timeout
                        && managed.session.connection_count() == 0
                    {
                        to_remove.push(*id);
                    }
                }
            }
        }

        for id in to_remove {
            self.remove_session(id);
        }
    }

    /// Remove a session immediately.
    pub fn remove_session(&self, id: SessionId) -> Option<Arc<Session>> {
        let mut sessions = self.sessions.lock().unwrap();
        if let Some(managed) = sessions.remove(&id) {
            info!("Removing session {}", id);
            managed.session.shutdown();
            Some(managed.session)
        } else {
            None
        }
    }

    /// Get the number of active sessions.
    pub fn session_count(&self) -> usize {
        let sessions = self.sessions.lock().unwrap();
        sessions.len()
    }

    /// Get all active session IDs.
    pub fn session_ids(&self) -> Vec<SessionId> {
        let sessions = self.sessions.lock().unwrap();
        sessions.keys().cloned().collect()
    }

    /// Get information about all active sessions.
    ///
    /// Returns a vector of [`SessionInfo`] structs containing session state snapshots
    /// for use by session management tools.
    pub fn get_sessions_info(&self) -> Vec<SessionInfo> {
        let sessions = self.sessions.lock().unwrap();
        let now = Instant::now();

        sessions.values().map(|managed| {
            let timeout_remaining_secs = managed.disconnected_at.map(|disconnected_at| {
                let elapsed = now.duration_since(disconnected_at);
                if elapsed < self.timeout {
                    (self.timeout - elapsed).as_secs()
                } else {
                    0
                }
            });

            SessionInfo::new(
                managed.session.id(),
                managed.session.connection_count(),
                timeout_remaining_secs,
                managed.session.is_shutdown_requested(),
                managed.session.pack_names(),
            )
        }).collect()
    }

    /// Start a background task that periodically cleans up expired sessions.
    pub fn start_cleanup_task(manager: Arc<Self>) {
        let cleanup_interval = manager.timeout / 2; // Check twice per timeout period.

        tokio::spawn(async move {
            loop {
                tokio::time::sleep(cleanup_interval).await;
                manager.cleanup_expired_sessions();
            }
        });
    }
}

#[cfg(test)]
mod tests {
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
}
