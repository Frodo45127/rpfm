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

use rpfm_lib::logger::Logger;
use rpfm_error::Result;

//---------------------------------------------------------------------------//
//                          Logging helpers
//---------------------------------------------------------------------------//

/// This function initialize the logging stuff. To be used at the start of the program.
pub fn initialize_logs() -> Result<()> {
    let _ = Logger::init()?;
    Ok(())
}
