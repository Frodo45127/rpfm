//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! High-level extensions for Total War modding built on top of `rpfm_lib`.
//!
//! This crate provides advanced features that build upon the core file handling
//! capabilities of `rpfm_lib`. While `rpfm_lib` focuses on low-level file format
//! parsing and encoding, this crate implements higher-level modding workflows
//! and analysis tools.
//!
//! # Modules
//!
//! ## Dependencies Management
//!
//! The [`dependencies`] module provides a comprehensive system for managing
//! dependencies between packs and vanilla game files:
//!
//! - Loading and caching vanilla game data for reference lookups
//! - Managing parent mod dependencies with automatic recursive loading
//! - Building reference data for DB table foreign key relationships
//! - Assembly Kit integration for tables not present in game files
//!
//! ## Diagnostics
//!
//! The [`diagnostics`] module implements validation and error checking:
//!
//! - DB/Loc table validation (invalid references, empty keys, duplicates)
//! - Pack-level checks (conflicting files, missing dependencies)
//! - Portrait settings validation
//! - Animation fragment validation
//! - Configurable diagnostic levels (Info, Warning, Error)
//!
//! ## Global Search
//!
//! The [`search`] module provides search and replace functionality across
//! entire packs:
//!
//! - Pattern and regex-based searching
//! - Case-sensitive and case-insensitive modes
//! - Search across multiple file types (DB, Loc, Text, etc.)
//! - Search in vanilla/parent dependencies
//! - Batch replace operations
//!
//! ## Pack Optimizer
//!
//! The [`optimizer`] module helps reduce pack size and improve compatibility:
//!
//! - Remove files identical to vanilla (ITM - Identical To Master)
//! - Remove duplicate and ITM table rows
//! - Clean up unused Portrait Settings entries
//! - Remove unnecessary XML and auxiliary files
//! - Datacore management for `twad_key_deletes` tables
//!
//! ## Translation Support
//!
//! The [`translator`] module assists with mod localization:
//!
//! - Extract translatable strings from packs
//! - Track translation status and changes
//! - Auto-translate from vanilla localisation data
//! - Export/import translation files
//!
//! ## glTF Export
//!
//! The [`gltf`] module provides 3D model export capabilities:
//!
//! - Convert RigidModel files to glTF format
//! - Preserve mesh data, materials, and textures
//! - Support for multiple LOD levels as separate scenes

// Disabled `Clippy` linters, with the reasons why they were disabled.
#![allow(
    clippy::too_many_arguments,             // Disabled because it gets annoying really quick.
    clippy::field_reassign_with_default,    // Disabled because it gets annoying on tests.
    clippy::assigning_clones,
    clippy::type_complexity,
)]

use std::{sync::{mpsc::Sender, Arc, LazyLock, RwLock}, thread::JoinHandle};

pub mod dependencies;
pub mod diagnostics;
pub mod gltf;
pub mod optimizer;
pub mod search;
pub mod translator;

/// Current version of the rpfm_extensions crate.
///
/// Used for versioning the dependencies cache to ensure compatibility.
const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Background thread handle for startpos generation.
///
/// Some games have a bug where the startpos build process deletes a folder that it
/// also requires to exist. This background thread repeatedly recreates the folder
/// to work around the issue. This static holds the thread handles and communication
/// channels for that process.
static START_POS_WORKAROUND_THREAD: LazyLock<Arc<RwLock<Option<Vec<(Sender<bool>, JoinHandle<()>)>>>>> = LazyLock::new(|| Arc::new(RwLock::new(None)));
