//---------------------------------------------------------------------------//
// Copyright (c) 2017-2020 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Logging module for the CLI tool.
//!
//! Any logging helper should be here.

use simplelog::{ColorChoice, CombinedLogger, LevelFilter, TerminalMode, TermLogger, WriteLogger};

use std::fs::File;

use rpfm_error::ctd::CrashReport;
use rpfm_error::Result;

use rpfm_lib::settings::get_config_path;

//---------------------------------------------------------------------------//
//                          Logging helpers
//---------------------------------------------------------------------------//

/// This function initialize the logging stuff. To be used at the start of the program.
pub fn initialize_logs() -> Result<()> {

    // In Release Builds, initiallize the logger, so we get messages in the terminal and recorded to disk.
    // Simplelog does not work properly with custom terminals, like the one in Sublime Text. Remember that.
    if !cfg!(debug_assertions) {
        CrashReport::init()?;
        CombinedLogger::init(
            vec![
                TermLogger::new(LevelFilter::Info, simplelog::Config::default(), TerminalMode::Mixed, ColorChoice::Auto),
                WriteLogger::new(LevelFilter::Info, simplelog::Config::default(), File::create(get_config_path()?.join("rpfm_cli.log"))?),
            ]
        )?;
    }


    Ok(())
}
