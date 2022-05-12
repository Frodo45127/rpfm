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
//!
//! # TODO: Write some examples.

pub mod binary;
pub mod compression;
pub mod encryption;
pub mod error;
pub mod files;
pub mod games;
pub mod integrations;
pub mod schema;
pub mod sqlite;
pub mod utils;
