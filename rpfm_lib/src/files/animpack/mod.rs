//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! AnimPack container file format support.
//!
//! AnimPacks (`.animpack` files) are container files that bundle animation-related game
//! data into a single archive. They are primarily used to organize and distribute animation
//! assets for units and characters in Total War games.
//!
//! # File Format
//!
//! AnimPacks use a simple binary format with a file count header followed by a list of
//! embedded files. Each embedded file includes its path, size, and raw data.
//!
//! ```text
//! [u32] file_count
//! For each file:
//!   [u8 + string] file_path (with backslashes)
//!   [u32] file_size
//!   [bytes] file_data
//! ```
//!
//! # Contained File Types
//!
//! AnimPacks typically contain:
//! - [`AnimsTable`] - Animation table indices
//! - [`AnimFragmentBattle`] - Animation fragments
//! - [`MatchedCombat`] - Matched combat definitions
//! - Other animation-related binary files
//!
//! [`AnimsTable`]: crate::files::anims_table::AnimsTable
//! [`AnimFragmentBattle`]: crate::files::anim_fragment_battle::AnimFragmentBattle
//! [`MatchedCombat`]: crate::files::matched_combat::MatchedCombat
//!
//! # File Location
//!
//! AnimPacks are usually found in:
//! ```text
//! animations/*.animpack
//! ```
//!
//! # Usage
//!
//! ```ignore
//! use rpfm_lib::files::animpack::AnimPack;
//! use rpfm_lib::files::{Container, Decodeable};
//!
//! // Decode an AnimPack from disk
//! let mut extra_data = DecodeableExtraData::default();
//! extra_data.set_disk_file_path(Some("animations/unit.animpack"));
//! extra_data.set_data_size(file_size);
//! extra_data.set_timestamp(timestamp);
//!
//! let animpack = AnimPack::decode(&mut reader, &Some(extra_data))?;
//!
//! // Access contained files
//! for (path, file) in animpack.files() {
//!     println!("File: {}", path);
//! }
//!
//! // Extract a specific file
//! if let Some(file) = animpack.file_by_path("battle/animations/humanoid01.bin") {
//!     let data = file.encode(&None, false, false, true)?;
//! }
//! ```
//!
//! # Version Support
//!
//! Complete support for all known AnimPack versions across Total War games.

use serde_derive::{Serialize, Deserialize};

use std::collections::HashMap;
use std::path::PathBuf;
use std::str::FromStr;

use crate::binary::{ReadBytes, WriteBytes};
use crate::error::Result;
use crate::files::*;

/// File extension for AnimPack files.
///
/// AnimPacks use the `.animpack` extension to distinguish them from other
/// container formats like PackFiles (`.pack`).
pub const EXTENSION: &str = ".animpack";

#[cfg(test)] mod animpack_test;

//---------------------------------------------------------------------------//
//                              Enum & Structs
//---------------------------------------------------------------------------//

/// Represents an AnimPack file decoded in memory.
///
/// AnimPacks are container files that bundle animation-related assets into a single
/// archive. This struct holds the complete AnimPack structure including metadata and
/// all contained files.
///
/// # Fields
///
/// - `disk_file_path`: Path to the AnimPack file on disk (empty if in-memory only)
/// - `disk_file_offset`: Byte offset within the disk file (0 if standalone file)
/// - `local_timestamp`: Last modified timestamp for change detection
/// - `paths`: Lowercase path lookup cache for case-insensitive file searches
/// - `files`: Map of file paths to their [`RFile`] data
///
/// # Binary Format
///
/// | Bytes          | Type                         | Data                                    |
/// | -------------- | ---------------------------- | --------------------------------------- |
/// | 4              | [u32]                        | File Count                              |
/// | X * File Count | [File](#file-structure) List | List of files inside the AnimPack File  |
///
/// ## File Structure
///
/// | Bytes       | Type           | Data                  |
/// | ----------- | -------------- | --------------------- |
/// | *           | Sized StringU8 | File Path             |
/// | 4           | [u32]          | File Length in bytes  |
/// | File Length | &\[[u8]\]      | File Data             |
///
/// # Container Implementation
///
/// AnimPack implements the [`Container`] trait, providing:
/// - File extraction and insertion
/// - Case-insensitive path lookup via paths cache
/// - Lazy loading support (when unencrypted)
/// - Timestamp-based change detection
///
/// # Lazy Loading
///
/// When `lazy_load` is enabled in [`DecodeableExtraData`], file data is not read
/// immediately but loaded on-demand. This reduces memory usage for large AnimPacks.
/// Lazy loading requires:
/// - Valid `disk_file_path` to a file on disk
/// - Unencrypted data (encrypted files are always fully loaded)
///
/// # Example
///
/// ```ignore
/// use rpfm_lib::files::animpack::AnimPack;
/// use rpfm_lib::files::Container;
///
/// // Create a new empty AnimPack
/// let mut animpack = AnimPack::default();
///
/// // Add a file
/// animpack.insert(rfile, "battle/animations/unit.bin")?;
///
/// // Look up a file (case-insensitive)
/// if let Some(file) = animpack.file_by_path("BATTLE/animations/UNIT.bin") {
///     println!("Found file!");
/// }
/// ```
#[derive(PartialEq, Clone, Debug, Default, Serialize, Deserialize)]
pub struct AnimPack {

    /// Path to this AnimPack file on disk.
    ///
    /// If the AnimPack has not been saved to disk or exists only in memory,
    /// this is an empty string.
    disk_file_path: String,

    /// Byte offset of this AnimPack within its disk file.
    ///
    /// If the AnimPack is a standalone file (not embedded in another file),
    /// this is 0.
    disk_file_offset: u64,

    /// Last modified timestamp of the disk file in seconds.
    ///
    /// Used to detect external modifications when lazy loading is enabled.
    /// Set to 0 for in-memory AnimPacks.
    local_timestamp: u64,

    /// Case-insensitive path lookup cache.
    ///
    /// Maps lowercase file paths to a list of their original-cased variants.
    /// This enables fast case-insensitive file lookups via [`Container::file_by_path()`].
    paths: HashMap<String, Vec<String>>,

    /// Map of file paths to their data.
    ///
    /// Keys are file paths as stored in the AnimPack (with forward slashes).
    /// Values are [`RFile`] instances containing the file data (cached or lazy-loaded).
    files: HashMap<String, RFile>,
}

//---------------------------------------------------------------------------//
//                           Implementation of AnimPack
//---------------------------------------------------------------------------//

impl Container for AnimPack {

    fn disk_file_path(&self) -> &str {
       &self.disk_file_path
    }

    fn files(&self) -> &HashMap<String, RFile> {
        &self.files
    }

    fn files_mut(&mut self) -> &mut HashMap<String, RFile> {
        &mut self.files
    }

    fn disk_file_offset(&self) -> u64 {
       self.disk_file_offset
    }

    fn paths_cache(&self) -> &HashMap<String, Vec<String>> {
        &self.paths
    }

    fn paths_cache_mut(&mut self) -> &mut HashMap<String, Vec<String>> {
        &mut self.paths
    }

    fn local_timestamp(&self) -> u64 {
        self.local_timestamp
    }
}

impl Decodeable for AnimPack {

    /// Decodes an AnimPack from a binary data source.
    ///
    /// # Parameters
    ///
    /// - `data`: Binary reader implementing [`ReadBytes`]
    /// - `extra_data`: Required decoding context (see below)
    ///
    /// # Required Extra Data Fields
    ///
    /// This implementation requires [`DecodeableExtraData`] with:
    /// - `lazy_load`: Enable lazy loading (ignored if encrypted)
    /// - `is_encrypted`: Whether the AnimPack data is encrypted
    /// - `disk_file_path`: Path to file on disk (required for lazy loading)
    /// - `disk_file_offset`: Offset within disk file (0 for standalone files)
    /// - `data_size`: Total size of AnimPack data in bytes
    /// - `timestamp`: Last modified timestamp in seconds (0 for in-memory)
    ///
    /// # Returns
    ///
    /// - `Ok(AnimPack)`: Successfully decoded AnimPack with all files
    /// - `Err(_)`: I/O error, malformed data, or missing required extra data
    ///
    /// # Lazy Loading Behavior
    ///
    /// When `lazy_load` is true and data is unencrypted:
    /// - File metadata is read immediately
    /// - File data is loaded on-demand when accessed
    /// - Requires valid `disk_file_path` to a file on disk
    ///
    /// When encrypted or lazy loading disabled:
    /// - All file data is read into memory immediately
    ///
    /// # Example
    ///
    /// ```ignore
    /// use std::fs::File;
    /// use std::io::BufReader;
    /// use rpfm_lib::binary::ReadBytes;
    /// use rpfm_lib::files::{Decodeable, DecodeableExtraData, animpack::AnimPack};
    /// use rpfm_lib::utils::last_modified_time_from_file;
    ///
    /// let path = "animations/unit.animpack";
    /// let mut reader = BufReader::new(File::open(path)?);
    ///
    /// let mut extra_data = DecodeableExtraData::default();
    /// extra_data.set_disk_file_path(Some(path));
    /// extra_data.set_data_size(reader.len()?);
    /// extra_data.set_timestamp(last_modified_time_from_file(reader.get_ref())?);
    ///
    /// let animpack = AnimPack::decode(&mut reader, &Some(extra_data))?;
    /// ```
    fn decode<R: ReadBytes>(data: &mut R, extra_data: &Option<DecodeableExtraData>) -> Result<Self> {
        let extra_data = extra_data.as_ref().ok_or(RLibError::DecodingMissingExtraData)?;

        // If we're reading from a file on disk, we require a valid path.
        // If we're reading from a file on memory, we don't need a valid path.
        let disk_file_path = match extra_data.disk_file_path {
            Some(path) => {
                let file_path = PathBuf::from_str(path).map_err(|_|RLibError::DecodingMissingExtraDataField("disk_file_path".to_owned()))?;
                if file_path.is_file() {
                    path.to_owned()
                } else {
                    return Err(RLibError::DecodingMissingExtraData)
                }
            }
            None => String::new()
        };

        let disk_file_offset = extra_data.disk_file_offset;
        let disk_file_size = extra_data.data_size;
        let local_timestamp = extra_data.timestamp;
        let is_encrypted = extra_data.is_encrypted;

        // If we don't have a path, or the file is encrypted, we can't lazy-load.
        let lazy_load = !disk_file_path.is_empty() && !is_encrypted && extra_data.lazy_load;
        let file_count = data.read_u32()?;

        let mut anim_pack = Self {
            disk_file_path,
            disk_file_offset,
            local_timestamp,
            paths: HashMap::new(),
            files: if file_count < 50_000 { HashMap::with_capacity(file_count as usize) } else { HashMap::new() },
        };

        for _ in 0..file_count {
            let path_in_container = data.read_sized_string_u8()?.replace('\\', "/");
            let size = data.read_u32()?;

            // Encrypted files cannot be lazy-loaded. They must be read in-place.
            if !lazy_load || is_encrypted {
                let data = data.read_slice(size as usize, false)?;
                let file = RFile {
                    path: path_in_container.to_owned(),
                    timestamp: None,
                    file_type: FileType::AnimPack,
                    container_name: None,
                    data: RFileInnerData::Cached(data),
                };

                anim_pack.files.insert(path_in_container, file);
            }

            // Unencrypted and files are not read, but lazy-loaded, unless specified otherwise.
            else {
                let data_pos = data.stream_position()? - disk_file_offset;
                let file = RFile::new_from_container(&anim_pack, size as u64, false, None, data_pos, local_timestamp, &path_in_container)?;
                data.seek(SeekFrom::Current(size as i64))?;

                anim_pack.files.insert(path_in_container, file);
            }
        }

        anim_pack.paths_cache_generate();

        anim_pack.files.par_iter_mut().map(|(_, file)| file.guess_file_type()).collect::<Result<()>>()?;

        check_size_mismatch(data.stream_position()? as usize - anim_pack.disk_file_offset as usize, disk_file_size as usize)?;
        Ok(anim_pack)
    }
}

impl Encodeable for AnimPack {

    /// Encodes this AnimPack to a binary data stream.
    ///
    /// # Parameters
    ///
    /// - `buffer`: Binary writer implementing [`WriteBytes`]
    /// - `extra_data`: Encoding options (not used, pass [`None`])
    ///
    /// # Returns
    ///
    /// - `Ok(())`: Successfully encoded AnimPack
    /// - `Err(_)`: I/O error, file too large, or encoding error
    ///
    /// # Encoding Behavior
    ///
    /// - Files are sorted alphabetically by path (case-insensitive)
    /// - Paths use forward slashes (`/`) not backslashes (`\`)
    /// - Each file is encoded inline with its size prefix
    /// - Files larger than 4GB (u32::MAX) return an error
    ///
    /// # Path Format
    ///
    /// **Important**: Encoded paths use forward slashes because animation sets created
    /// by assed tool break if backslashes are used.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use std::fs::File;
    /// use std::io::{BufWriter, Write};
    /// use rpfm_lib::files::{Encodeable, animpack::AnimPack};
    ///
    /// let mut animpack = AnimPack::default();
    /// // ... add files to animpack ...
    ///
    /// let mut encoded = vec![];
    /// animpack.encode(&mut encoded, &None)?;
    ///
    /// // Write to disk
    /// let mut writer = BufWriter::new(File::create("output.animpack")?);
    /// writer.write_all(&encoded)?;
    /// ```
    fn encode<W: WriteBytes>(&mut self, buffer: &mut W, extra_data: &Option<EncodeableExtraData>) -> Result<()> {
        buffer.write_u32(self.files.len() as u32)?;

        // NOTE: This has to use /, not \, because for some reason the animsets made by Assed break if we use \.
        let mut sorted_files = self.files.iter_mut().collect::<Vec<(&String, &mut RFile)>>();
        sorted_files.sort_unstable_by_key(|(path, _)| path.to_lowercase());

        for (path, file) in sorted_files {
            buffer.write_sized_string_u8(path)?;

            let data = file.encode(extra_data, false, false, true)?.unwrap();

            // Error on files too big for the AnimPack.
            if data.len() > u32::MAX as usize {
                return Err(RLibError::DataTooBigForContainer("AnimPack".to_owned(), u32::MAX as u64, data.len(), path.to_owned()));
            }

            buffer.write_u32(data.len() as u32)?;
            buffer.write_all(&data)?;
        }

        Ok(())
    }
}
