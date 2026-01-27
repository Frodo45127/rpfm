//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! # RPFM IPC - Inter-Process Communication Protocol
//!
//! This crate defines the IPC protocol used for communication between the RPFM frontend
//! and the backend server (`rpfm_server`). It provides type-safe message definitions that ensure
//! consistent communication between the two processes.
//!
//! ## Protocol Overview
//!
//! The communication follows a request-response pattern over WebSocket connections:
//!
//! 1. The frontend creates a [`messages::Message<Command>`] with a unique ID
//! 2. The message is serialized to JSON and sent over WebSocket to the server
//! 3. The server processes the command and sends back a [`messages::Message<Response>`]
//! 4. The frontend matches the response ID to the original request
//!
//! This ID correlation mechanism enables asynchronous, non-blocking communication where multiple
//! requests can be in flight simultaneously.
//!
//! ## Modules
//!
//! - [`messages`]: Core protocol definitions including [`messages::Command`], [`messages::Response`],
//!   and the [`messages::Message`] wrapper.
//! - [`helpers`]: Data structures for marshalling complex data between UI and server, including
//!   [`helpers::ContainerInfo`], [`helpers::RFileInfo`], and [`helpers::DataSource`].
//!
//! ## Usage
//!
//! This crate is not intended for standalone use. It serves as a shared dependency between
//! `rpfm_server` and `rpfm_ui`, providing the common language they need to communicate.
//!
//! ### Example Message Flow
//!
//! ```ignore
//! // Frontend creates a command
//! let command = Message {
//!     id: 1,
//!     data: Command::OpenPackFiles(vec![PathBuf::from("/path/to/pack.pack")]),
//! };
//!
//! // Serialize and send over WebSocket...
//!
//! // Server responds with matching ID
//! let response = Message {
//!     id: 1,  // Same ID as the request
//!     data: Response::ContainerInfo(container_info),
//! };
//! ```

pub mod helpers;
pub mod messages;

/// Settings key for the base path where MyMods are stored.
pub const MYMOD_BASE_PATH: &str = "mymods_base_path";

/// Settings key for the secondary path (used for additional content paths).
pub const SECONDARY_PATH: &str = "secondary_path";
