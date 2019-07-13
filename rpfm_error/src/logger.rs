//---------------------------------------------------------------------------//
// Copyright (c) 2017-2019 Ismael Gutiérrez González. All rights reserved.
// 
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
// 
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

// Here it goes the code needed to produce error files in panic, so I can debug properly the stupid CTDs people has, because sentry fails a lot.
// If you're interested, is inspired in the human-panic crate. The reason to not use that crate is because it's not configurable. At all.
// But otherwise, feel free to check it out if you need an easy-to-use simple error logger.

use failure::Backtrace;
use uuid::Uuid;
use serde_derive::Serialize;

use std::fs::File;
use std::io::{BufWriter, Write};
use std::panic::PanicInfo;

use directories::ProjectDirs;
use crate::{ErrorKind, Result};

/// This struct contains all the info to write into a bug report file.
#[derive(Debug, Serialize)]
pub struct Report {
    name: String,
    crate_version: String,
    build_type: String,
    operating_system: String,
    explanation: String,
    backtrace: String,
}

/// Implementation of Report.
impl Report {

	/// Create a new report. Note that this creates the report in memory. If you want to save it to disk, you've to do it later.
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
			operating_system,
			build_type: if cfg!(debug_assertions) { "Debug" } else { "Release" }.to_owned(),
			explanation,
			backtrace: format!("{:#?}", Backtrace::new()),
		}
	}

	/// Try to write a report to disk.
	/// TODO: Deal with the error here when not finding the folder..
	pub fn save(&self) -> Result<()> {
		let uuid = Uuid::new_v4().to_hyphenated().to_string();
		if let Some(proj_dirs) = ProjectDirs::from("", "",  "Rusted PackFile Manager") {
    		let file_path = proj_dirs.config_dir().join(format!("error/error-report-{}.toml", &uuid));
			let mut file = BufWriter::new(File::create(&file_path)?);
			file.write_all(toml::to_string_pretty(&self)?.as_bytes())?;
			Ok(())
    	}
    	else {
    		Err(ErrorKind::IOFolderCannotBeOpened)?
    	}
	}
}
