//---------------------------------------------------------------------------//
// Copyright (c) 2017-2024 Ismael Gutiérrez González. All rights reserved.
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
    clippy::too_many_arguments,             // Disabled because it gets annoying really quick.
    clippy::field_reassign_with_default,    // Disabled because it gets annoying on tests.
    clippy::assigning_clones,
    clippy::type_complexity,
    clippy::upper_case_acronyms,
)]

use regex::Regex;

use std::sync::LazyLock;

pub mod binary;
pub mod compression;
pub mod encryption;
pub mod error;
pub mod files;
pub mod games;
pub mod integrations;
pub mod notes;
pub mod schema;
pub mod utils;

#[cfg(test)] mod utils_test;

/// Regex to find if a path belongs to a db table.
pub static REGEX_DB: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"db/[^/]+_tables/[^/]+$").unwrap());

/// Regex to find if a path belongs to a portrait settings file.
pub static REGEX_PORTRAIT_SETTINGS: LazyLock<Regex> = LazyLock::new(|| Regex::new(r".*portrait_settings\S*\.bin$").unwrap());
