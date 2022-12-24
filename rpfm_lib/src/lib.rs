//---------------------------------------------------------------------------//
// Copyright (c) 2017-2022 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! # Overview
//!
//! This crate provides utilities to read/write multiple types of files used by Creative Assembly (CA)
//! in Total War Games since Empire: Total War.
//!
//! For information about an specific file type (support, docs, specs,...), please check their modules
//! under the [`files`] module.
//!
//! # TODO: Write some examples.

// Disabled `Clippy` linters, with the reasons why they were disabled.
#![allow(
    clippy::too_many_arguments              // Disabled because it gets annoying really quick.
)]

use lazy_static::*;
use regex::Regex;

pub mod binary;
pub mod compression;
pub mod encryption;
pub mod error;
pub mod files;
pub mod games;
pub mod integrations;
pub mod schema;
pub mod tips;
pub mod utils;

lazy_static! {

    /// Regex to find if a path belongs to a db table.
    pub static ref REGEX_DB: Regex = Regex::new(r"db/[^/]+_tables/[^/]+$").unwrap();

    /// Regex to find if a path belongs to a portrait settings file.
    pub static ref REGEX_PORTRAIT_SETTINGS: Regex = Regex::new(r"portrait_settings_\S+.bin$").unwrap();
}
