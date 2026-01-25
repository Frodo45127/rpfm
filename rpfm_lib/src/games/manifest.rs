//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Game manifest file parsing for vanilla PackFile discovery.
//!
//! This module handles parsing of the `manifest.txt` file found in Total War game
//! data directories. The manifest lists all official game files with their sizes,
//! allowing RPFM to identify which PackFiles are vanilla (from Creative Assembly)
//! versus user-created mods.
//!
//! # Manifest File Format
//!
//! The `manifest.txt` is a tab-delimited file in the game's `/data` directory:
//! ```text
//! data/local_en.pack<TAB>12345678
//! data/units.pack<TAB>98765432<TAB>1
//! ```
//!
//! Columns:
//! 1. **Relative path** - File path relative to `/data` directory
//! 2. **Size** - File size in bytes
//! 3. **Base game flag** (optional, newer games only) - `1` if base game, `0` if DLC
//!
//! # Usage
//!
//! The manifest is primarily used to:
//! - Identify vanilla PackFiles for loading as game data
//! - Distinguish between base game and DLC content
//! - Validate file integrity (via size checking)
//! - Filter out mod files when building dependency trees
//!
//! # Fallback Behavior
//!
//! Not all Total War games have manifest files (Empire, Napoleon don't). For these games,
//! RPFM falls back to hardcoded lists in the game's install data.
//!
//! # Example
//!
//! ```ignore
//! use rpfm_lib::games::manifest::Manifest;
//! use rpfm_lib::games::supported_games::{SupportedGames, KEY_WARHAMMER_3};
//! use std::path::Path;
//!
//! # fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let games = SupportedGames::default();
//! let game = games.game(&KEY_WARHAMMER_3).unwrap();
//! let game_path = Path::new("/path/to/game");
//!
//! // Read manifest from game
//! let manifest = Manifest::read_from_game_path(game, game_path)?;
//!
//! // Check if a file is listed in manifest
//! let is_vanilla = manifest.is_path_in_manifest(Path::new("data/local_en.pack"));
//! # Ok(())
//! # }
//! ```

use csv::ReaderBuilder;
use getset::*;
use serde_derive::Deserialize;

use std::fs::canonicalize;
use std::path::{Path, PathBuf};

use crate::error::{RLibError, Result};
use super::GameInfo;

/// Name of the manifest file in game data directories
const MANIFEST_FILE_NAME: &str = "manifest.txt";

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// Complete parsed manifest file from a game's data directory.
///
/// Contains all entries from `manifest.txt`, representing the full list of
/// official game files.
///
/// # Structure
///
/// This is a wrapper around a vector of [`ManifestEntry`] items, one per line
/// in the manifest file.
#[derive(Deserialize)]
pub struct Manifest(pub Vec<ManifestEntry>);

/// Single file entry from a game manifest.
///
/// Represents one line in `manifest.txt`, describing a single game file.
///
/// # Format Compatibility
///
/// Different Total War games use slightly different manifest formats:
/// - **Older games**: 2 columns (path, size)
/// - **Newer games**: 3 columns (path, size, base_game_flag)
///
/// The parser handles both formats automatically.
#[derive(Default, Getters, Deserialize)]
#[getset(get = "pub")]
pub struct ManifestEntry {

    /// File path relative to the game's `/data` directory.
    ///
    /// Example: `"local_en.pack"` or `"boot.pack"`
    relative_path: String,

    /// File size in bytes.
    ///
    /// Can be used for file integrity validation.
    size: u64,

    /// Base game vs DLC flag (newer games only).
    ///
    /// - `Some(1)`: File is part of base game (always present)
    /// - `Some(0)`: File is from DLC (may be missing)
    /// - `None`: Game doesn't use this field (older manifest format)
    belongs_to_base_game: Option<u8>,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//


/// Implementation of `Manifest`.
impl Manifest {

    /// Reads and parses the manifest file for a specific game installation.
    ///
    /// This is a convenience wrapper that constructs the manifest path from game
    /// information and reads it.
    ///
    /// # Arguments
    ///
    /// * `game` - Game configuration containing path information
    /// * `game_path` - Root directory of the game installation
    ///
    /// # Returns
    ///
    /// Returns the parsed [`Manifest`] containing all file entries.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The game's data path cannot be determined
    /// - The manifest file doesn't exist
    /// - The manifest file is malformed or cannot be parsed
    ///
    /// # Example
    ///
    /// ```ignore
    /// # use rpfm_lib::games::manifest::Manifest;
    /// # use rpfm_lib::games::supported_games::{SupportedGames, KEY_WARHAMMER_3};
    /// # use std::path::Path;
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let games = SupportedGames::default();
    /// # let game = games.game(&KEY_WARHAMMER_3).unwrap();
    /// let game_path = Path::new("/path/to/game");
    /// let manifest = Manifest::read_from_game_path(game, game_path)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn read_from_game_path(game: &GameInfo, game_path: &Path) -> Result<Self> {
        let manifest_path = game.data_path(game_path)?.join(MANIFEST_FILE_NAME);
        Self::read(&manifest_path)
    }

    /// Reads and parses a manifest file from a specific path.
    ///
    /// Parses a `manifest.txt` file in tab-delimited format. Handles both 2-column
    /// (older games) and 3-column (newer games) formats automatically.
    ///
    /// # Arguments
    ///
    /// * `manifest_path` - Path to the `manifest.txt` file
    ///
    /// # Returns
    ///
    /// Returns the parsed [`Manifest`] containing all file entries.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The file cannot be opened or read
    /// - The format is invalid (wrong number of columns)
    /// - Field values cannot be parsed (e.g., non-numeric size)
    ///
    /// # Format Details
    ///
    /// Expected format (tab-delimited):
    /// ```text
    /// path/to/file.pack<TAB>1234567
    /// path/to/other.pack<TAB>9876543<TAB>1
    /// ```
    pub fn read(manifest_path: &Path) -> Result<Self> {

        let mut reader = ReaderBuilder::new()
            .delimiter(b'\t')
            .quoting(false)
            .has_headers(false)
            .flexible(true)
            .from_path(manifest_path)?;

        // Due to "flexible" not actually working when doing serde-backed deserialization (took some time to figure this out)
        // the deserialization has to be done manually.
        let mut entries = vec![];
        for record in reader.records() {
            let record = record?;

            // We only know these manifest formats.
            if record.len() != 2 && record.len() != 3 {
                return Err(RLibError::ManifestFileParseError("Mismatch column count".to_owned()));
            } else {
                let mut manifest_entry = ManifestEntry {
                    relative_path: record.get(0).ok_or_else(|| RLibError::ManifestFileParseError("Error reading relative path".to_owned()))?.to_owned(),
                    size: record.get(1).ok_or_else(|| RLibError::ManifestFileParseError("Error reading size".to_owned()))?.parse()?,
                    ..Default::default()
                };

                // In newer games, a third field has been added.
                if record.len() == 3 {
                    manifest_entry.belongs_to_base_game = record.get(2).ok_or_else(|| RLibError::ManifestFileParseError("Error reading if file belongs to the base game".to_owned()))?.parse().ok();
                }
                else {
                    manifest_entry.belongs_to_base_game = None;
                }

                entries.push(manifest_entry);
            }
        }

        let manifest = Self(entries);
        Ok(manifest)
    }

    /// Checks if a file path is listed in the manifest.
    ///
    /// Performs case-insensitive comparison and handles path separator differences
    /// (backslash vs forward slash).
    ///
    /// # Arguments
    ///
    /// * `path` - Path to check (can be absolute or relative)
    ///
    /// # Returns
    ///
    /// Returns `true` if the path ends with any relative path in the manifest.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use rpfm_lib::games::manifest::Manifest;
    /// # use std::path::Path;
    /// # fn example(manifest: &Manifest) {
    /// let is_vanilla = manifest.is_path_in_manifest(
    ///     Path::new("C:/Games/Warhammer3/data/local_en.pack")
    /// );
    /// # }
    /// ```
    pub fn is_path_in_manifest(&self, path: &Path) -> bool {
        let insensitivized_path = path.to_str().unwrap().to_lowercase().replace('\\', "/");
        self.0.iter().any(|x| insensitivized_path.ends_with(&x.relative_path.to_lowercase()))
    }
}

impl ManifestEntry {

    /// Validates and canonicalizes a path based on manifest entry metadata.
    ///
    /// Determines if a file path should be used based on whether it's from the base
    /// game or DLC, and whether the file actually exists on disk.
    ///
    /// # Arguments
    ///
    /// * `path` - File path to validate
    ///
    /// # Returns
    ///
    /// Returns `Some(canonical_path)` if the file should be used:
    /// - Base game files (`belongs_to_base_game == 1`): Always returned
    /// - DLC files (`belongs_to_base_game == 0`): Only if file exists on disk
    /// - Unknown origin (`belongs_to_base_game == None`): Only if file exists
    ///
    /// Returns `None` if the file is a missing DLC file.
    ///
    /// # Path Canonicalization
    ///
    /// If the path is valid, it's canonicalized to an absolute path before returning.
    pub fn path_from_manifest_entry(&self, path: PathBuf) -> Option<PathBuf> {
        match self.belongs_to_base_game() {
            Some(value) => {
                if *value == 1 || path.is_file() {
                    canonicalize(path).ok()
                } else {
                    None
                }
            },
            None => canonicalize(path).ok(),
        }
    }
}
