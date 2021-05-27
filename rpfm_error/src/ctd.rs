//---------------------------------------------------------------------------//
// Copyright (c) 2017-2020 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

/*!
Module to log CTDs to a file in RPFM's Config folder.

This module is a custom CTD logging module, heavely inspired in the `human-panic` crate. The reason to not use that crate is because it's not configurable. At all.
But otherwise, feel free to check it out if you need an easy-to-use simple error logger.
!*/

use backtrace::Backtrace;
use directories::ProjectDirs;
use serde_derive::Serialize;
use uuid::Uuid;

use std::fs::File;
use std::io::{BufWriter, Write};
use std::panic::PanicInfo;
use std::path::Path;
use std::panic;

use crate::{ErrorKind, Result, SENTRY_DSN, SENTRY_GUARD, VERSION};

/// This struct contains all the info to write into a `CrashReport` file.
#[derive(Debug, Serialize)]
pub struct CrashReport {

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

/// Implementation of `CrashReport`.
impl CrashReport {

    /// This function initialize the `CrashReport` to log crashes.
    pub fn init() -> Result<()> {

        // Register the CTD logger if we're not in a debug build.
        if !cfg!(debug_assertions) {
            let config_path = match ProjectDirs::from("", "", "rpfm") {
                Some(proj_dirs) => proj_dirs.config_dir().to_path_buf(),
                None => return Err(ErrorKind::IOFolderCannotBeOpened.into())
            };

            panic::set_hook(Box::new(move |info: &panic::PanicInfo| {
                Self::new(info, VERSION).save(&config_path).unwrap();
            }));

            // Sentry guard, for testing sentry reports.
            *SENTRY_GUARD.write().unwrap() = Some(sentry::init((SENTRY_DSN, sentry::ClientOptions {
                release: sentry::release_name!(),
                sample_rate: 1.0,
                ..Default::default()
            })));
        }

        Ok(())
    }

    /// Create a new `CrashReport` from a `Panic`.
    ///
    /// Remember that this creates the `CrashReport` in memory. If you want to save it to disk, you've to do it later.
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

    /// This function tries to save the `CrashReport` to the provided folder.
    pub fn save(&self, path: &Path) -> Result<()> {
        let uuid = Uuid::new_v4().to_hyphenated().to_string();
        let file_path = path.to_path_buf().join(format!("error/error-report-{}.toml", &uuid));
        let mut file = BufWriter::new(File::create(&file_path)?);
        file.write_all(toml::to_string_pretty(&self)?.as_bytes())?;
        Ok(())
    }
}
