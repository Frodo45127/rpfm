//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! This module defines sessions for per-client state management.
//!
//! Each client connection gets its own session with isolated state,
//! including its own background thread for processing commands.
//!
//! Sessions persist for a configurable timeout after disconnection,
//! allowing clients to reconnect to the same session.

use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};
use tokio::time::{Duration, Instant};

use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock, atomic::{AtomicBool, AtomicU32, Ordering}};

use rpfm_ipc::helpers::SessionInfo;
use rpfm_ipc::messages::{Command, Response};
use rpfm_log::info;

use crate::background_thread;

/// Error messages for session communication.
pub const SESSION_SENDER_ERROR: &str = "Error in session communication system. Sender failed to send message.";

/// Default session timeout in seconds (5 minutes).
pub const DEFAULT_SESSION_TIMEOUT_SECS: u64 = 300;

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// Unique identifier for a session.
pub type SessionId = u64;

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

    /// Sender to communicate with this session's background thread.
    sender: UnboundedSender<(UnboundedSender<Response>, Command)>,

    /// Number of active connections using this session.
    connection_count: AtomicU32,

    /// Whether this session has been marked for shutdown.
    shutdown_requested: AtomicBool,

    /// Names of the pack files currently open in this session.
    pack_names: RwLock<Vec<String>>,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

impl Session {

    /// Create a new session with its own background thread.
    pub fn new(id: SessionId) -> Arc<Self> {
        let (sender, receiver) = unbounded_channel();

        let session = Arc::new(Self {
            id,
            sender,
            connection_count: AtomicU32::new(0),
            shutdown_requested: AtomicBool::new(false),
            pack_names: RwLock::new(Vec::new()),
        });

        // Spawn a dedicated background thread for this session.
        let session_clone = session.clone();
        tokio::spawn(async move {
            info!("Session {} background thread starting...", id);
            background_thread::background_loop(receiver, session_clone).await;
            info!("Session {} background thread terminated.", id);
        });

        session
    }

    /// Get the session ID.
    pub fn id(&self) -> SessionId {
        self.id
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

    /// Shutdown this session by sending an Exit command.
    pub fn shutdown(&self) {
        info!("Session {} shutting down...", self.id);

        if self.shutdown_requested.swap(true, Ordering::SeqCst) {
            info!("Session {} already marked for shutdown before...", self.id);
            return;
        }

        // Send exit command - ignore errors if channel is already closed.
        let (sender_back, _) = unbounded_channel();
        let _ = self.sender.send((sender_back, Command::Exit));
    }

    /// Send a command to this session's background thread.
    ///
    /// Returns a receiver to get the response.
    pub fn send(&self, command: Command) -> UnboundedReceiver<Response> {
        let (sender_back, receiver_back) = unbounded_channel();
        if let Err(error) = self.sender.send((sender_back, command)) {
            panic!("{SESSION_SENDER_ERROR}: {error}");
        }
        receiver_back
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
        let id = {
            let mut next_id = self.next_id.lock().unwrap();
            let id = *next_id;
            *next_id += 1;
            id
        };

        let session = Session::new(id);
        session.connect();

        self.sessions.lock().unwrap().insert(id, ManagedSession {
            session: session.clone(),
            disconnected_at: None,
        });

        info!("Created new session with ID: {}", id);
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
            info!("Session {} timeout check triggered (cleanup handled by manager)", id);
            manager.remove_session(id);

            // Check if this was the last session and shutdown the server if so.
            if manager.session_count() == 0 {
                info!("No more active sessions, shutting down server...");
                std::process::exit(0);
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
                if let Some(disconnected_at) = managed.disconnected_at {
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

/// Helper function to receive a response from a session.
///
/// This is async and will wait for the response.
pub async fn recv_response(receiver: &mut UnboundedReceiver<Response>) -> Response {
    match receiver.recv().await {
        Some(response) => response,
        None => panic!("Session response channel closed unexpectedly"),
    }
}
