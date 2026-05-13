//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Core library for reading and writing Total War game files.
//!
//! This crate provides comprehensive support for reading, writing, and manipulating file formats
//! used by Creative Assembly in Total War games since Empire: Total War. It forms the foundation
//! of the Rusted PackFile Manager (RPFM) project.
//!
//! # Supported Games
//!
//! - Total War: Pharaoh - Dynasties
//! - Total War: Pharaoh
//! - Total War: Warhammer 3
//! - Total War Saga: Troy
//! - Total War: Three Kingdoms
//! - Total War: Warhammer 2
//! - Total War: Warhammer
//! - Total War Saga: Thrones of Britannia
//! - Total War: Attila
//! - Total War: Rome 2
//! - Total War: Shogun 2
//! - Total War: Napoleon
//! - Total War: Empire
//!
//! # Supported File Formats
//!
//! The library supports 30+ file types including:
//!
//! - **Pack Files** (`.pack`): Container format for all game assets
//! - **Database Tables** (`db/`): Game data with versioned schemas
//! - **Localisation Files** (`.loc`): Translated text strings
//! - **3D Models** (`.rigid_model_v2`): Unit and building meshes
//! - **Animations**: AnimPack, AnimFragment, AnimsTable, MatchedCombat
//! - **Audio**: Sound banks (`.bnk`), sound events, DAT containers
//! - **Images**: DDS textures, atlases
//! - **Maps**: BMD (battle map data), tile databases, vegetation
//! - **Campaign**: ESF (save files), startpos
//! - **UI**: UIC components, portrait settings, fonts
//! - **Scripts**: Lua, XML, and other text formats
//!
//! See the [`files`] module for detailed information on each file type and its support level.
//!
//! # Architecture
//!
//! The library is organized into several key modules:
//!
//! - [`files`]: File format parsers and writers (30+ types)
//! - [`schema`]: Database table schema definitions and versioning
//! - [`binary`]: Low-level binary I/O utilities
//! - [`games`]: Game-specific configuration and version detection
//! - [`compression`]: LZ4, ZStd, and LZMA compression support
//! - [`encryption`]: Pack file encryption/decryption
//! - [`error`]: Error types and result handling
//! - [`integrations`]: Optional Assembly Kit and Git integration
//!
//! # Feature Flags
//!
//! - `integration_assembly_kit`: Enable Assembly Kit raw table parsing
//! - `integration_git`: Enable Git repository operations
//!
//! # Examples
//!
//! ## Reading a Pack File
//!
//! ```ignore
//! use rpfm_lib::files::pack::Pack;
//! use rpfm_lib::games::supported_games::{SupportedGames, KEY_WARHAMMER_3};
//! use std::path::PathBuf;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let path = PathBuf::from("path/to/pack.pack");
//! let games = SupportedGames::default();
//! let game_info = games.game(&KEY_WARHAMMER_3).unwrap();
//! let pack = Pack::read_and_merge(&[path], game_info, true, false, false)?;
//!
//! for file in pack.files().values() {
//!     println!("{}", file.path_in_container_raw());
//! }
//! # Ok(())
//! # }
//! ```
//!
//! ## Working with Database Tables
//!
//! ```ignore
//! use rpfm_lib::files::{db::DB, Decodeable, DecodeableExtraData, RFile, RFileDecoded};
//! use rpfm_lib::schema::Schema;
//! use std::path::Path;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let schema_path = Path::new("path/to/schema.ron");
//! let schema = Schema::load(&schema_path, None)?;
//! let mut extra_data = DecodeableExtraData::default();
//! extra_data.set_schema(Some(&schema));
//!
//! // Decode a DB file (assuming pack is loaded)
//! // let mut file = pack.files_by_path(&path, false).first().unwrap();
//! // file.decode(&Some(extra_data), false, true)?;
//! //
//! // if let Ok(RFileDecoded::DB(db)) = file.decoded() {
//! //     println!("Table {} has {} rows", db.table_name(), db.data().len());
//! // }
//! # Ok(())
//! # }
//! ```
//!
//! # Related Crates
//!
//! - `rpfm_extensions`: Higher-level features (dependencies, diagnostics, search, optimizer)
//! - `rpfm_ui`: Qt-based desktop application

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

/// Regular expression to identify database table file paths.
///
/// Matches paths in the format: `db/{table_name}_tables/{file}`
///
/// # Examples
///
/// - `db/units_tables/core_units.bin` → matches
/// - `db/buildings_tables/data.bin` → matches
/// - `data/units.txt` → does not match
pub static REGEX_DB: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"db/[^/]+_tables/[^/]+$").unwrap());

/// Regular expression to identify CEO database table file paths. These do not use the normal db prefix
/// due to crashes when loading them in mods ingame.
///
/// Matches paths in the format: `ceo_db/{table_name}_tables/{file}`
///
/// # Examples
///
/// - `ceo_db/units_tables/core_units.bin` → matches
/// - `ceo_db/buildings_tables/data.bin` → matches
/// - `data/units.txt` → does not match
pub static REGEX_CEO_DB: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"ceo_db/[^/]+_tables/[^/]+$").unwrap());

/// Regular expression to identify portrait settings file paths.
///
/// Matches any path ending with `portrait_settings` followed by optional characters and `.bin`.
///
/// # Examples
///
/// - `portraits/portrait_settings.bin` → matches
/// - `data/portrait_settings_v2.bin` → matches
/// - `other/settings.bin` → does not match
pub static REGEX_PORTRAIT_SETTINGS: LazyLock<Regex> = LazyLock::new(|| Regex::new(r".*portrait_settings\S*\.bin$").unwrap());
