//---------------------------------------------------------------------------//
// Copyright (c) 2017-2024 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

/*!
Module to log CTDs and messages within RPFM.

This module is a custom CTD logging module, heavily inspired in the `human-panic` crate.
The reason to not use that crate is because it's not configurable. At all. But otherwise,
feel free to check it out if you need an easy-to-use simple error logger.

Note that these loggers need to be initialized on start by calling `Logger::init()`.
Otherwise, none of them will work.
!*/

use backtrace::Backtrace;
use lazy_static::lazy_static;
pub use log::{error, info, warn};
pub use sentry::{ClientInitGuard, Envelope, integrations::log::SentryLogger, protocol::*, release_name, end_session, end_session_with_status};
use serde_derive::Serialize;
use simplelog::{ColorChoice, CombinedLogger, LevelFilter, SharedLogger, TermLogger, TerminalMode};

use std::borrow::Cow;
use std::fs::{DirBuilder, File};
use std::io::{BufWriter, Write};
use std::{panic, panic::PanicHookInfo};
use std::path::Path;
use std::sync::{Arc, RwLock};

use crate::error::Result;
use crate::utils::current_time;

/// Current version of the crate.
const VERSION: &str = env!("CARGO_PKG_VERSION");

lazy_static! {

    /// This is the DSN needed for Sentry reports to work. Don't change it.
    pub static ref SENTRY_DSN: Arc<RwLock<String>> = Arc::new(RwLock::new(String::new()));
}

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This struct contains all the info to write into a `CrashReport` file.
#[derive(Debug, Serialize)]
pub struct Logger {

    /// Name of the Program. To know what of the programs crashed.
    name: String,

    /// Version of the Program/Lib.
    crate_version: String,

    /// If it happened in a `Debug` or `Release` build.
    build_type: String,

    /// The OS in which the crash happened.
    operating_system: String,

    /// The reason why the crash happened.
    explanation: String,

    /// A backtrace generated when the crash happened.
    backtrace: String,
}

//-------------------------------------------------------------------------------//
//                              Implementations
//-------------------------------------------------------------------------------//

/// Implementation of `Logger`.
impl Logger {

    /// This function initialize the `Logger` to log crashes.
    ///
    /// There are three loggers active:
    /// - Log CTD to files.
    /// - Log CTD to sentry (release only)
    /// - Log execution steps to file/sentry.
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
            log::set_boxed_logger(Box::new(logger))?;
        }

        // Initialize Sentry's guard, for remote reporting. Only for release mode.
        let dsn = if cfg!(debug_assertions) { String::new() } else { SENTRY_DSN.read().unwrap().to_string() };
        let client_options = sentry::ClientOptions {
            release: release.clone(),
            sample_rate: 1.0,
            auto_session_tracking: true,
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

    /// Create a new local Crash Report from a `Panic`.
    ///
    /// Remember that this creates the Crash Report in memory. If you want to save it to disk, you've to do it later.
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

    /// This function tries to save a generated Crash Report to the provided folder.
    pub fn save(&self, path: &Path) -> Result<()> {
        let file_path = path.join(format!("error-report-{}.toml", current_time()?));
        let mut file = BufWriter::new(File::create(file_path)?);
        file.write_all(toml::to_string_pretty(&self)?.as_bytes())?;
        Ok(())
    }

    /// This function uploads a patch to sentry's service.
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
}
