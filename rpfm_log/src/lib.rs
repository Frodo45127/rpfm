//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Crash reporting and structured logging with Sentry integration.
//!
//! This module provides comprehensive error tracking and logging capabilities:
//! - Local crash report generation with backtraces
//! - Remote error reporting via Sentry
//! - Structured runtime logging (info, warning, error levels)
//! - Automatic panic handling and session tracking
//!
//! # Overview
//!
//! The logging system is heavily inspired by the `human-panic` crate but provides more
//! configurability and integration with Sentry for production error tracking.
//!
//! # Features
//!
//! ## Local Crash Reports
//!
//! When a panic occurs, a detailed crash report is saved locally as a TOML file containing:
//! - Program name and version
//! - Build type (debug/release)
//! - Operating system information
//! - Panic message and location
//! - Full backtrace
//!
//! ## Sentry Integration
//!
//! In release builds, crashes and events are automatically uploaded to Sentry for:
//! - Centralized error tracking
//! - Session health monitoring
//! - Breadcrumb trails
//! - Custom event uploads with attachments
//!
//! ## Runtime Logging
//!
//! Standard logging macros are available throughout the application:
//! - [`info!`]: Informational messages (verbose mode only)
//! - [`warn!`]: Warning messages
//! - [`error!`]: Error messages
//!
//! # Initialization
//!
//! The logger **must** be initialized at program startup by calling [`Logger::init()`].
//! Without initialization, none of the logging features will work.
//!
//! # Example
//!
//! ```no_run
//! use rpfm_log::{Logger, info, warn, error};
//! use std::path::Path;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Initialize the logger
//! let _sentry_guard = Logger::init(
//!     Path::new("logs/crash_reports"),
//!     true,  // verbose mode
//!     true,  // set global logger
//!     Some("rpfm@5.0.0".into())
//! )?;
//!
//! // Use logging throughout the application
//! info!("Application started");
//! warn!("Configuration file not found, using defaults");
//!
//! // The guard ensures Sentry is properly shut down on drop
//! # Ok(())
//! # }
//! ```
//!
//! # Note
//!
//! The Sentry integration is only active in release builds. Debug builds will still
//! generate local crash reports but won't upload to Sentry.

use backtrace::Backtrace;
pub use log::{error, info, warn};
use ron::ser::PrettyConfig;
pub use sentry::{ClientInitGuard, ClientOptions, end_session, end_session_with_status, Envelope, integrations::{log::SentryLogger, tracing::SentryLayer}, protocol::*, release_name, self, SessionMode};
use serde_derive::Serialize;
use simplelog::{ColorChoice, CombinedLogger, LevelFilter, SharedLogger, TermLogger, TerminalMode};

use std::borrow::Cow;
use std::fs::{DirBuilder, File};
use std::io::{BufWriter, Write};
use std::{panic, panic::PanicHookInfo};
use std::path::Path;
use std::sync::{Arc, LazyLock, RwLock};
use std::time::{SystemTime, UNIX_EPOCH};

/// Current version of the crate from Cargo.toml.
const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Sentry DSN (Data Source Name) for error reporting.
///
/// This must be set before calling [`Logger::init()`] for Sentry integration to work.
/// The DSN is provided by Sentry when creating a project.
pub static SENTRY_DSN: LazyLock<Arc<RwLock<String>>> = LazyLock::new(|| Arc::new(RwLock::new(String::new())));

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// Error type for the logging crate.
#[derive(Debug, thiserror::Error)]
pub enum LogError {

    /// Wrapper for [`std::io::Error`].
    #[error(transparent)]
    IoError(#[from] std::io::Error),

    /// Wrapper for [`toml::ser::Error`].
    #[error(transparent)]
    TomlSerError(#[from] toml::ser::Error),

    /// Wrapper for [`log::SetLoggerError`].
    #[error(transparent)]
    LogError(#[from] log::SetLoggerError),

    /// Wrapper for [`ron::Error`].
    #[error(transparent)]
    RonError(#[from] ron::Error),

    /// Wrapper for [`std::time::SystemTimeError`].
    #[error(transparent)]
    SystemTimeError(#[from] std::time::SystemTimeError),
}

/// Result type for the logging crate.
pub type Result<T> = std::result::Result<T, LogError>;

/// Crash report data structure.
///
/// Contains all information needed to generate a detailed crash report that can be
/// saved locally as a TOML file or uploaded to Sentry. Created automatically when
/// a panic occurs.
#[derive(Debug, Serialize)]
pub struct Logger {

    /// Name of the program that crashed.
    ///
    /// Taken from the `CARGO_PKG_NAME` environment variable.
    name: String,

    /// Version of the program/library.
    ///
    /// Taken from the `CARGO_PKG_VERSION` environment variable.
    crate_version: String,

    /// Build configuration (Debug or Release).
    ///
    /// Determined at compile time based on debug assertions.
    build_type: String,

    /// Operating system information.
    ///
    /// Includes OS type and version (e.g., "Windows 11", "Ubuntu 22.04").
    operating_system: String,

    /// Panic explanation.
    ///
    /// Contains the panic message and location (file and line number).
    explanation: String,

    /// Full backtrace from the panic.
    ///
    /// Formatted stack trace showing the call chain leading to the panic.
    backtrace: String,
}

//-------------------------------------------------------------------------------//
//                              Implementations
//-------------------------------------------------------------------------------//

/// Implementation of `Logger`.
impl Logger {

    /// Initializes the logging system with crash reporting and Sentry integration.
    ///
    /// This function sets up three logging mechanisms:
    /// 1. **Local crash reports**: Panics are saved as TOML files to disk
    /// 2. **Sentry crash reporting**: Panics are uploaded to Sentry (release builds only)
    /// 3. **Runtime logging**: Structured logging via terminal and Sentry breadcrumbs
    ///
    /// # Arguments
    ///
    /// * `logging_path` - Directory where crash reports will be saved
    /// * `verbose` - If `true`, log `Info` level messages; if `false`, only `Warn` and above
    /// * `set_logger` - If `true`, initialize the global logger (disable for testing)
    /// * `release` - Optional release identifier for Sentry (e.g., `"rpfm@5.0.0"`)
    ///
    /// # Returns
    ///
    /// Returns a [`ClientInitGuard`] that must be kept alive for the duration of the program.
    /// Dropping the guard will shut down Sentry and flush pending events.
    ///
    /// # Panics
    ///
    /// After initialization, any panic in any thread will:
    /// 1. Generate a local crash report in `logging_path`
    /// 2. Upload to Sentry (if in release mode and enabled)
    /// 3. Mark the Sentry session as crashed
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use rpfm_log::Logger;
    /// # use std::path::Path;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let _guard = Logger::init(
    ///     Path::new("crash_reports"),
    ///     true,  // verbose
    ///     true,  // set global logger
    ///     Some("myapp@1.0.0".into())
    /// )?;
    ///
    /// // Logger is now active
    /// // Keep _guard alive until program exit
    /// # Ok(())
    /// # }
    /// ```
    pub fn init(logging_path: &Path, verbose: bool, set_logger: bool, release: Option<Cow<'static, str>>) -> Result<ClientInitGuard> {

        // Make sure the provided folder exists.
        if let Some(parent_folder) = logging_path.parent() {
            DirBuilder::new().recursive(true).create(parent_folder)?;
        }

        let log_level = if verbose {
            LevelFilter::Info
        } else {
            LevelFilter::Warn
        };

        if set_logger {

            // Initialize the combined logger, with a term logger (for runtime logging) and a write logger (for storing on a log file).
            //
            // So, fun fact: this thing has a tendency to crash on boot for no reason. So instead of leaving it crashing, we'll make it optional.
            let loggers: Vec<Box<dyn SharedLogger + 'static>> = vec![TermLogger::new(log_level, simplelog::Config::default(), TerminalMode::Mixed, ColorChoice::Auto)];
            let combined_logger = CombinedLogger::new(loggers);

            // Initialize Sentry's logger, so anything logged goes to the breadcrumbs too.
            let logger = SentryLogger::with_dest(combined_logger);
            log::set_max_level(log_level);
            let _ = log::set_boxed_logger(Box::new(logger));
        }

        // Initialize Sentry's guard, for remote reporting. Only for release mode.
        let dsn = if cfg!(debug_assertions) { String::new() } else { SENTRY_DSN.read().unwrap().to_string() };
        let client_options = ClientOptions {
            release: release.clone(),
            sample_rate: 0.3,
            traces_sample_rate: 0.3,
            enable_logs: true,
            auto_session_tracking: true,
            session_mode: SessionMode::Application,
            ..Default::default()
        };

        let sentry_guard = sentry::init((dsn, client_options));

        // Setup the panic hooks to catch panics on all threads, not only the main one.
        let sentry_enabled = sentry_guard.is_enabled();
        let orig_hook = panic::take_hook();
        let logging_path = logging_path.to_owned();
        panic::set_hook(Box::new(move |info: &PanicHookInfo| {
            warn!("Panic detected. Generating backtraces and crash logs...");

            // Get the data printed into the logs, because I'm tired of this getting "missed" when is a cross-thread crash.
            let data = Self::new(info, VERSION);
            if data.save(&logging_path).is_err() {
                error!("Failed to generate crash log.");
            }

            orig_hook(info);

            // Stop tracking session health before existing.
            if sentry_enabled {
                end_session_with_status(SessionStatus::Crashed)
            }
        }));

        // Return Sentry's guard, so we can keep it alive until everything explodes, or the user closes the program.
        info!("Logger initialized.");
        Ok(sentry_guard)
    }

    /// Creates a crash report from panic information.
    ///
    /// This function extracts all relevant information from a panic and constructs
    /// a structured crash report. The report is created in memory and must be
    /// explicitly saved with [`Logger::save()`].
    ///
    /// # Arguments
    ///
    /// * `panic_info` - Panic hook information provided by the panic handler
    /// * `version` - Version string of the program
    ///
    /// # Returns
    ///
    /// Returns a populated [`Logger`] instance containing the crash report data.
    ///
    /// # Note
    ///
    /// This is typically called automatically by the panic hook installed by [`Logger::init()`].
    pub fn new(panic_info: &PanicHookInfo, version: &str) -> Self {

        let info = os_info::get();
        let operating_system = format!("OS: {}\nVersion: {}", info.os_type(), info.version());

        let mut explanation = String::new();
        if let Some(payload) = panic_info.payload().downcast_ref::<&str>() {
            explanation.push_str(&format!("Cause: {}\n", &payload));
        }

        match panic_info.location() {
            Some(location) => explanation.push_str(&format!("Panic occurred in file '{}' at line {}\n", location.file(), location.line())),
            None => explanation.push_str("Panic location unknown.\n"),
        }

        Self {
            name: env!("CARGO_PKG_NAME").to_owned(),
            crate_version: version.to_owned(),
            build_type: if cfg!(debug_assertions) { "Debug" } else { "Release" }.to_owned(),
            operating_system,
            explanation,
            backtrace: format!("{:#?}", Backtrace::new()),
        }
    }

    /// Saves the crash report to a TOML file.
    ///
    /// The crash report is saved with a timestamped filename in the format
    /// `error-report-{timestamp}.toml`.
    ///
    /// # Arguments
    ///
    /// * `path` - Directory where the crash report file should be saved
    ///
    /// # Returns
    ///
    /// Returns [`Ok`] if the report was saved successfully, or an error if file I/O fails.
    pub fn save(&self, path: &Path) -> Result<()> {
        let current_time = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
        let file_path = path.join(format!("error-report-{}.toml", current_time));
        let mut file = BufWriter::new(File::create(file_path)?);
        file.write_all(toml::to_string_pretty(&self)?.as_bytes())?;
        Ok(())
    }

    /// Sends a custom event to Sentry with optional file attachment.
    ///
    /// This function creates and uploads a Sentry event with a message and optional
    /// data attachment. Useful for manually reporting errors or uploading diagnostic data.
    ///
    /// # Arguments
    ///
    /// * `sentry_guard` - The Sentry client guard (must be active)
    /// * `level` - Severity level (e.g., [`Level::Info`], [`Level::Warning`], [`Level::Error`])
    /// * `message` - Event message/description
    /// * `data` - Optional tuple of `(filename, data_bytes)` to attach to the event
    ///
    /// # Returns
    ///
    /// Returns [`Ok`] if the event was sent (or Sentry is disabled), or an error on failure.
    ///
    /// # Note
    ///
    /// If Sentry is not enabled (debug builds or no DSN), this function does nothing
    /// and returns [`Ok`].
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use rpfm_log::{Logger, Level};
    /// # fn example(sentry_guard: &sentry::ClientInitGuard) -> Result<(), Box<dyn std::error::Error>> {
    /// // Send a simple event
    /// Logger::send_event(sentry_guard, Level::Info, "Schema updated", None)?;
    ///
    /// // Send an event with attachment
    /// let patch_data = b"some patch data";
    /// Logger::send_event(
    ///     sentry_guard,
    ///     Level::Warning,
    ///     "Schema patch failed",
    ///     Some(("patch.json", patch_data))
    /// )?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn send_event(sentry_guard: &ClientInitGuard, level: Level, message: &str, data: Option<(&str, &[u8])>) -> Result<()> {
        if sentry_guard.is_enabled() {
            let mut event = Event::new();
            event.level = level;
            event.message = Some(message.to_string());

            let mut envelope = Envelope::from(event);
            if let Some((filename, buffer)) = data {
                let attatchment = Attachment {
                    buffer: buffer.to_vec(),
                    filename: filename.to_owned(),
                    content_type: Some("application/json".to_owned()),
                    ty: None
                };

                envelope.add_item(EnvelopeItem::Attachment(attatchment));
            }
            sentry_guard.send_envelope(envelope);
        }

        // TODO: Make this fail in case of sentry being not working?
        Ok(())
    }

    /// Uploads schema patches to Sentry for debugging/analysis.
    ///
    /// Serializes the data to RON format and sends it as an informational Sentry event.
    ///
    /// # Arguments
    ///
    /// * `sentry_guard` - The Sentry client guard
    /// * `game_name` - Name of the game the patches are for
    /// * `patches` - The data to upload (must implement `Serialize`)
    pub fn upload_patches(sentry_guard: &ClientInitGuard, game_name: &str, patches: &impl serde::Serialize) -> Result<()> {
        let current_time = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
        let level = Level::Info;
        let message = format!("Schema Patch for: {} - {}.", game_name, current_time);
        let config = PrettyConfig::default();
        let mut data = vec![];
        ron::ser::to_writer_pretty(&mut data, patches, config)?;
        let file_name = "patch.txt";

        Self::send_event(sentry_guard, level, &message, Some((file_name, &data)))
    }

    /// Uploads schema definitions to Sentry for debugging/analysis.
    ///
    /// Serializes the data to RON format and sends it as an informational Sentry event.
    ///
    /// # Arguments
    ///
    /// * `sentry_guard` - The Sentry client guard
    /// * `game_name` - Name of the game the definitions are for
    /// * `definitions` - The data to upload (must implement `Serialize`)
    pub fn upload_definitions(sentry_guard: &ClientInitGuard, game_name: &str, definitions: &impl serde::Serialize) -> Result<()> {
        let current_time = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
        let level = Level::Info;
        let message = format!("Schema Definition for: {} - {}.", game_name, current_time);
        let config = PrettyConfig::default();
        let mut data = vec![];
        ron::ser::to_writer_pretty(&mut data, definitions, config)?;
        let file_name = "definition.txt";

        Self::send_event(sentry_guard, level, &message, Some((file_name, &data)))
    }
}
