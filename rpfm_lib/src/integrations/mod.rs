//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Optional integrations with external tools and services.
//!
//! This module provides optional integration features with third-party tools and services.
//! All integrations are feature-gated and must be explicitly enabled to be compiled.
//!
//! # Available Integrations
//!
//! ## Assembly Kit Integration
//!
//! **Feature**: `integration_assembly_kit`
//!
//! Provides functionality for importing and synchronizing table definitions from Creative Assembly's
//! official Assembly Kit tools. This allows RPFM to stay up-to-date with official schema definitions.
//!
//! Key capabilities:
//! - Parsing Assembly Kit table definition XML files
//! - Importing field metadata (types, keys, references, descriptions)
//! - Detecting localisable fields
//! - Automatic schema generation from Assembly Kit data
//!
//! See [`assembly_kit`] module for details.
//!
//! ## Git Integration
//!
//! **Feature**: `integration_git`
//!
//! Enables basic Git repository management for version control of mod files and schemas.
//! This allows RPFM to fetch schema updates from remote repositories and manage local changes.
//!
//! Key capabilities:
//! - Cloning and updating Git repositories
//! - Pulling changes from remotes
//! - Basic repository status queries
//!
//! See `git` module for details.
//!
//! ## Logging and Crash Reporting
//!
//! **Feature**: `integration_log`
//!
//! Provides structured logging and automatic crash report uploading via Sentry.
//! This helps with debugging and collecting error reports from users.
//!
//! Key capabilities:
//! - Structured logging with multiple levels
//! - Automatic crash report generation
//! - Sentry integration for error tracking
//! - User-friendly error reporting
//!
//! See `log` module for details.
//!
//! # Usage
//!
//! Each integration is completely optional. Enable only the features you need in your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! rpfm_lib = { version = "5.0", features = ["integration_assembly_kit", "integration_git"] }
//! ```

#[cfg(feature = "integration_assembly_kit")] pub mod assembly_kit;
#[cfg(feature = "integration_git")] pub mod git;
#[cfg(feature = "integration_log")] pub mod log;
