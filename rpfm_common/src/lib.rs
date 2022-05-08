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
//! For information about ann specific file, please check their modules under the [`files`] module.


// Reexports for ease of managing dependencies.
pub use rpfm_macros;
pub use rpfm_logging;

pub mod compression;
pub mod decoder;
pub mod encoder;
pub mod encryption;
pub mod error;
pub mod files;
pub mod games;
pub mod integrations;
pub mod schema;
pub mod sqlite;
pub mod utils;

// This tells the compiler to only compile these mods when testing. It's just to make sure
// the encoders and decoders don't break between updates.
#[cfg(test)]
mod decoder_test;

#[cfg(test)]
mod encoder_test;
