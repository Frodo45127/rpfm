//---------------------------------------------------------------------------//
// Copyright (c) 2017-2024 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! This crate contains certain functionality extensions that, for one reason or another, didn't fit in the main RPFM lib crate.

// Disabled `Clippy` linters, with the reasons why they were disabled.
#![allow(
    clippy::too_many_arguments,             // Disabled because it gets annoying really quick.
    clippy::field_reassign_with_default,    // Disabled because it gets annoying on tests.
    clippy::assigning_clones,
    clippy::type_complexity,
)]

use fancy_regex::Regex;
use lazy_static::lazy_static;

pub mod dependencies;
pub mod diagnostics;
pub mod optimizer;
pub mod search;
pub mod translator;

lazy_static! {

    /// Regex to find if a path belongs to a db table.
    pub static ref REGEX_INVALID_ESCAPES: Regex = Regex::new(r"(?<!\\)\\n|(?<!\\)\\t").unwrap();
}

const VERSION: &str = env!("CARGO_PKG_VERSION");
