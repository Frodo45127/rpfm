//---------------------------------------------------------------------------//
// Copyright (c) 2017-2023 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

use csv::ReaderBuilder;
use getset::*;
use serde_derive::Deserialize;

use std::path::{Path, PathBuf};

use crate::error::{RLibError, Result};
use super::GameInfo;

const MANIFEST_FILE_NAME: &str = "manifest.txt";

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This struct represents the entire **Manifest.txt** from the /data folder.
///
/// Private for now, because I see no public use for this.
#[derive(Deserialize)]
pub struct Manifest(pub Vec<ManifestEntry>);

/// This struct represents a Manifest Entry.
#[derive(Default, Getters, Deserialize)]
#[getset(get = "pub")]
pub struct ManifestEntry {

    /// The path of the file, relative to /data.
    relative_path: String,

    /// The size in bytes of the file.
    size: u64,

    /// If the file comes with the base game (1), or with one of its dlc (0). Not in all games.
    belongs_to_base_game: Option<u8>,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//


/// Implementation of `Manifest`.
impl Manifest {

    /// This function returns a parsed version of the `manifest.txt` of the Game Selected, if exists and is parsable.
    pub fn read_from_game_path(game: &GameInfo, game_path: &Path) -> Result<Self> {
        let manifest_path = game.data_path(game_path)?.join(MANIFEST_FILE_NAME);
        Self::read(&manifest_path)
    }

    /// This function returns a parsed version of the `manifest.txt` in the folder you provided, if exists and is parsable.
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

    pub fn is_path_in_manifest(&self, path: &Path) -> bool {
        let insensitivized_path = path.to_str().unwrap().to_lowercase().replace('\\', "/");
        self.0.iter().any(|x| insensitivized_path.ends_with(&x.relative_path.to_lowercase()))
    }
}

impl ManifestEntry {

    /// This function returns if a path from a manifest entry can be use (optional files can be missing depending on dlcs used by the user).
    pub fn path_from_manifest_entry(&self, path: PathBuf) -> Option<PathBuf> {
        match self.belongs_to_base_game() {
            Some(value) => {
                if *value == 1 || path.is_file() {
                    Some(path)
                } else {
                    None
                }
            },
            None => Some(path),
        }
    }
}
