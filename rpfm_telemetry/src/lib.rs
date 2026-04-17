//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Logging, crash reporting and anonymous action telemetry for RPFM.
//!
//! This crate bundles three related concerns that share the same Sentry
//! lifecycle:
//!
//! - **Structured logging**: re-exports `log`'s `info!`/`warn!`/`error!`
//!   macros so every crate can emit log lines without pulling Sentry.
//! - **Crash reporting**: [`Logger::init`] installs panic hooks, writes
//!   local crash reports and wires up Sentry for release builds.
//! - **Action telemetry**: a lightweight action counter that aggregates
//!   anonymous usage data and ships it to Sentry on graceful shutdown.
//!
//! Libraries (`rpfm_lib`, `rpfm_extensions`, `rpfm_ui_common`, ...) depend
//! only on the plain `log` crate. The executables (`rpfm_ui`, `rpfm_server`)
//! depend on this crate to wire up the full stack.

pub use log::{debug, error, info, trace, warn};

mod actions;
mod logger;

pub use actions::*;
pub use logger::*;
