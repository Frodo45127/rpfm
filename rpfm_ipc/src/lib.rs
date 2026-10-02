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
//! This crate defines the protocol used between RPFM's clients (the UI, MCP clients and scripts) and the backend
//! server (`rpfm_server`), and the settings both sides share.
//!
//! ## Protocol Overview
//!
//! Clients call typed methods of the server over JSON-RPC 2.0, sent as WebSocket text frames:
//!
//! 1. The client sends an [`api::RpcRequest`] with a unique ID, a method name like `table.rows`, and its params.
//! 2. The server answers with an [`api::RpcResponse`] with the same ID, holding the result or an [`api::ApiError`].
//! 3. Long methods run as jobs: they answer right away with a job ID, and report their progress and result in
//!    `job.updated` notifications.
//!
//! Each method is a struct implementing [`api::Request`], which ties it to its name and response type.
//!
//! ## Modules
//!
//! - [`api`]: The methods, grouped by domain, and the JSON-RPC envelope.
//! - [`helpers`]: Data structures shared by several methods, like [`helpers::ContainerInfo`], [`helpers::RFileInfo`]
//!   and [`helpers::DataSource`].
//! - [`settings`]: The settings store and the config folder helpers.
//! - [`settings_keys`]: The keys of the settings.

pub mod api;
pub mod helpers;
pub mod settings;
pub mod settings_keys;
