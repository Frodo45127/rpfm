//---------------------------------------------------------------------------//
// Copyright (c) 2017-2022 Ismael Gutiérrez González. All rights reserved.
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
use log::{error, info, warn};
pub use sentry::{ClientInitGuard, Envelope, integrations::log::SentryLogger, protocol::*};
use serde_derive::Serialize;
use simplelog::{ColorChoice, CombinedLogger, LevelFilter, SharedLogger, TermLogger, TerminalMode, WriteLogger};

use std::fs::{DirBuilder, File};
use std::io::{BufWriter, Write};
use std::panic::PanicInfo;
use std::path::Path;
use std::panic;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::error::Result;

pub mod error;

/// Log files to log execution steps and other messages.
const LOG_FILE_CURRENT: &str = "rpfm.log";
const LOG_FILE_1: &str = "rpfm_1.log";
const LOG_FILE_2: &str = "rpfm_2.log";
const LOG_FILE_3: &str = "rpfm_3.log";

/// Current version of the crate.
const VERSION: &str = env!("CARGO_PKG_VERSION");

/// This is the DSN needed for Sentry reports to work. Don't change it.
const SENTRY_DSN: &str = "https://a8bf0a98ed43467d841ec433fb3d75a8@sentry.io/1205298";

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
    pub fn init(logging_path: &Path) -> Result<ClientInitGuard> {
        println!("Initializing Logger.");

        // Make sure the provided folder exists.
        if let Some(parent_folder) = logging_path.parent() {
            DirBuilder::new().recursive(true).create(&parent_folder)?;
        }

        // Rotate the logs so we can keep a few old logs.
        Self::rotate_logs(&logging_path)?;

        // Initialize the combined logger, with a term logger (for runtime logging) and a write logger (for storing on a log file).
        //
        // So, fun fact: this thing has a tendency to crash on boot for no reason. So instead of leaving it crashing, we'll make it optional.
        let mut file_logger_failed = true;
        let mut loggers: Vec<Box<dyn SharedLogger + 'static>> = vec![TermLogger::new(LevelFilter::Info, simplelog::Config::default(), TerminalMode::Mixed, ColorChoice::Auto)];
        if let Ok(write_logger_file) = File::create(logging_path.join(LOG_FILE_CURRENT)) {
            let write_logger = WriteLogger::new(LevelFilter::Info, simplelog::Config::default(), write_logger_file);
            loggers.push(write_logger);
            file_logger_failed = false;
        }

        let combined_logger = CombinedLogger::new(loggers);

        // Initialize Sentry's logger, so anything logged goes to the breadcrumbs too.
        let logger = SentryLogger::with_dest(combined_logger);
        log::set_max_level(log::LevelFilter::Info);
        log::set_boxed_logger(Box::new(logger))?;

        // Initialize Sentry's guard, for remote reporting. Only for release mode.
        let dsn = if cfg!(debug_assertions) { "" } else { SENTRY_DSN };
        let sentry_guard = sentry::init((dsn, sentry::ClientOptions {
            release: sentry::release_name!(),
            sample_rate: 1.0,
            ..Default::default()
        }));

        // Setup the panic hooks to catch panics on all threads, not only the main one.
        let orig_hook = panic::take_hook();
        let logging_path = logging_path.to_owned();
        panic::set_hook(Box::new(move |info: &panic::PanicInfo| {
            info!("Panic detected. Generating backtraces and crash logs...");
            if Self::new(info, VERSION).save(&logging_path).is_err() {
                error!("Failed to generate crash log.");
            }
            orig_hook(info);
            std::process::exit(1);
        }));

        if file_logger_failed {
            warn!("File Logger failed.");
        }

        // Return Sentry's guard, so we can keep it alive until everything explodes, or the user closes the program.
        info!("Logger initialized.");
        Ok(sentry_guard)
    }

    /// Create a new local Crash Report from a `Panic`.
    ///
    /// Remember that this creates the Crash Report in memory. If you want to save it to disk, you've to do it later.
    pub fn new(panic_info: &PanicInfo, version: &str) -> Self {

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
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis();
        let file_path = path.join(format!("error/error-report-{}.toml", &timestamp));
        let mut file = BufWriter::new(File::create(&file_path)?);
        file.write_all(toml::to_string_pretty(&self)?.as_bytes())?;
        Ok(())
    }

    /// This function takes care of rotating the logs used by RPFM, so we can keep a few old logs when starting a new instance.
    fn rotate_logs(config_path: &Path) -> Result<()> {
        let log_path_current = config_path.join(LOG_FILE_CURRENT);
        let log_path_1 = config_path.join(LOG_FILE_1);
        let log_path_2 = config_path.join(LOG_FILE_2);
        let log_path_3 = config_path.join(LOG_FILE_3);

        if log_path_3.is_file() {
            std::fs::remove_file(&log_path_3)?;
        }

        if log_path_2.is_file() {
            std::fs::rename(&log_path_2, log_path_3)?;
        }

        if log_path_1.is_file() {
            std::fs::rename(&log_path_1, log_path_2)?;
        }

        if log_path_current.is_file() {
            std::fs::rename(log_path_current, log_path_1)?;
        }

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
