//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Animation table file format support.
//!
//! This module handles animation table files (`*_tables.bin`) which define animation
//! sets and their associated fragments for unit skeletons in Total War games. These
//! files act as indices that map skeleton types to their animation fragment files.
//!
//! # File Format
//!
//! Animation tables use a binary format (version 2) containing:
//! - List of animation table entries
//! - Each entry maps a skeleton type to animation fragments
//! - Fragment references with metadata
//!
//! # File Naming Convention
//!
//! Animation table files must end with `_tables.bin` to be recognized by RPFM.
//! This is a library-specific requirement for disambiguation, not a game limitation.
//!
//! Common naming patterns:
//! - `humanoid01_tables.bin` - Human skeleton animations
//! - `cavalry_tables.bin` - Mounted unit animations
//! - `monster_tables.bin` - Large creature animations
//!
//! # File Organization
//!
//! Animation tables are stored in:
//! ```text
//! animations/animation_tables/{skeleton_type}_tables.bin
//! ```
//!
//! # Structure
//!
//! Each table contains entries that define:
//! - Animation table name (logical identifier)
//! - Skeleton type (which skeleton this applies to)
//! - Mount table reference (for mounted units)
//! - List of animation fragments with metadata
//!
//! # Supported Versions
//!
//! Currently only version 2 is supported, used in modern Total War games
//! (Warhammer 2, Three Kingdoms, Warhammer 3, etc.).
//!
//! # Usage
//!
//! ```ignore
//! use rpfm_lib::files::anims_table::AnimsTable;
//! use rpfm_lib::files::Decodeable;
//!
//! // Decode an animation table
//! let table = AnimsTable::decode(&mut data, &None)?;
//!
//! // Access entries
//! for entry in table.entries() {
//!     println!("Table: {} for skeleton: {}",
//!         entry.table_name(),
//!         entry.skeleton_type()
//!     );
//!
//!     // List fragments
//!     for fragment in entry.fragments() {
//!         println!("  Fragment: {}", fragment.name());
//!     }
//! }
//! ```

use getset::{Getters, Setters};
use serde_derive::{Serialize, Deserialize};

use crate::binary::{ReadBytes, WriteBytes};
use crate::error::{RLibError, Result};
use crate::files::{DecodeableExtraData, Decodeable, EncodeableExtraData, Encodeable};
use crate::utils::check_size_mismatch;

/// Base directory path for animation files.
pub const BASE_PATH: &str = "animations/";

/// File extension pattern for animation table files.
///
/// To differentiate animation tables from other `.bin` files, RPFM only recognizes
/// files ending in `_tables.bin` as AnimsTable files.
///
/// **Note**: This is a library-specific requirement for disambiguation, not a game
/// limitation. The game itself doesn't require this specific naming pattern.
pub const EXTENSION: &str = "_tables.bin";

mod versions;

#[cfg(test)] mod anims_table_test;

//---------------------------------------------------------------------------//
//                              Enum & Structs
//---------------------------------------------------------------------------//

/// Represents an animation table file (`*_tables.bin`).
///
/// Animation tables serve as indices mapping skeleton types to their available animation
/// fragment files. Each table contains one or more entries defining animation sets for
/// different skeleton configurations.
///
/// # Fields
///
/// - `version`: File format version (currently only version 2 is supported)
/// - `entries`: List of animation table entries, each mapping a skeleton to fragments
///
/// # Version Support
///
/// - **Version 2**: Current format used in Warhammer 2, Three Kingdoms, Warhammer 3
///
/// # Example
///
/// ```ignore
/// // Create a new animation table
/// let mut table = AnimsTable::default();
/// table.set_version(2);
///
/// // Add an entry for humanoid skeletons
/// let mut entry = Entry::default();
/// entry.set_table_name("humanoid01_animations".to_string());
/// entry.set_skeleton_type("humanoid01".to_string());
/// table.entries_mut().push(entry);
/// ```
#[derive(PartialEq, Clone, Debug, Default, Getters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", set = "pub")]
pub struct AnimsTable {
    /// File format version number.
    ///
    /// Only version 2 is currently supported.
    version: u32,

    /// List of animation table entries.
    ///
    /// Each entry defines animations for a specific skeleton type and configuration.
    entries: Vec<Entry>,
}

/// Represents a single animation table entry mapping a skeleton to fragments.
///
/// Each entry defines an animation set for a specific skeleton type. Entries can
/// reference mount tables for cavalry/mounted units and contain lists of animation
/// fragments that apply to this skeleton configuration.
///
/// # Fields
///
/// - `table_name`: Logical name of this animation table (e.g., "humanoid01_animations")
/// - `skeleton_type`: Skeleton identifier this entry applies to (e.g., "humanoid01")
/// - `mount_table_name`: Reference to mount table for mounted units (empty for infantry)
/// - `fragments`: List of animation fragment references
/// - `uk_6`: Unknown boolean flag (purpose unclear)
/// - `uk_7`: Unknown boolean flag (purpose unclear)
///
/// # Mount Tables
///
/// For mounted units (cavalry, chariots, monsters with riders), `mount_table_name`
/// references another animation table that defines the mount's animations. Infantry
/// units leave this field empty.
///
/// # Example
///
/// ```ignore
/// // Infantry entry
/// let mut infantry = Entry::default();
/// infantry.set_table_name("hu1_animations".to_string());
/// infantry.set_skeleton_type("humanoid01".to_string());
/// infantry.set_mount_table_name(String::new());
///
/// // Cavalry entry
/// let mut cavalry = Entry::default();
/// cavalry.set_table_name("cav_animations".to_string());
/// cavalry.set_skeleton_type("humanoid01".to_string());
/// cavalry.set_mount_table_name("horse_animations".to_string());
/// ```
#[derive(PartialEq, Clone, Debug, Default, Getters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", set = "pub")]
pub struct Entry {
    /// Logical name of this animation table.
    ///
    /// Used as an identifier for this animation set (e.g., "humanoid01_animations").
    ///
    /// FIXME: This is possibly not table_name, but skeleton_type, and skeleton_type is possibly skeleton_type_cinematic.
    table_name: String,

    /// Skeleton type identifier.
    ///
    /// Specifies which skeleton this entry's animations apply to (e.g., "humanoid01",
    /// "cavalry", "monster").
    skeleton_type: String,

    /// Mount table reference for mounted units.
    ///
    /// For cavalry/mounted units, this references the animation table for the mount
    /// (e.g., "horse_animations"). Empty string for infantry/unmounted units.
    mount_table_name: String,

    /// List of animation fragments for this skeleton.
    ///
    /// Each fragment references an animation file and associated metadata.
    fragments: Vec<Fragment>,

    /// Unknown boolean flag.
    ///
    /// Purpose currently unclear. May be related to animation blending or state flags.
    uk_6: bool,

    /// Unknown boolean flag.
    ///
    /// Purpose currently unclear. May be related to animation blending or state flags.
    uk_7: bool,
}

/// Represents a reference to an animation fragment file.
///
/// Fragments are individual animation files that can be applied to a skeleton.
/// Each fragment reference includes the fragment's name and associated metadata.
///
/// # Fields
///
/// - `name`: Animation fragment file name (without path or extension)
/// - `uk_5`: Unknown 32-bit metadata value (purpose unclear)
///
/// # Fragment Naming
///
/// Fragment names typically follow patterns like:
/// - `hu1_walk_01` - Humanoid walking animation
/// - `cav_charge_02` - Cavalry charge animation
/// - `monster_attack_melee` - Monster melee attack
///
/// The actual fragment files are stored in separate `.bin` or `.frg` files in the
/// animations directory structure.
///
/// # Example
///
/// ```ignore
/// let mut fragment = Fragment::default();
/// fragment.set_name("hu1_idle_breathing_01".to_string());
/// fragment.set_uk_5(0);  // Unknown field
/// ```
#[derive(PartialEq, Clone, Debug, Default, Getters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", set = "pub")]
pub struct Fragment {
    /// Animation fragment file name.
    ///
    /// The name of the fragment file (without path or extension). For example,
    /// "hu1_walk_01" refers to a humanoid walking animation fragment.
    name: String,

    /// Unknown 32-bit metadata value.
    ///
    /// Purpose currently unclear. May be related to fragment priority, blending
    /// weight, or other animation system metadata.
    uk_5: u32,
}

//---------------------------------------------------------------------------//
//                      Implementation of AnimsTable
//---------------------------------------------------------------------------//

impl Decodeable for AnimsTable {

    fn decode<R: ReadBytes>(data: &mut R, _extra_data: &Option<DecodeableExtraData>) -> Result<Self> {
        let mut table = Self::default();
        table.version = data.read_u32()?;

        match table.version {
            2 => table.read_v2(data)?,
            _ => Err(RLibError::DecodingMatchedCombatUnsupportedVersion(table.version as usize))?,
        }

        // If we are not in the last byte, it means we didn't parse the entire file, which means this file is corrupt.
        check_size_mismatch(data.stream_position()? as usize, data.len()? as usize)?;

        Ok(table)
    }
}

impl Encodeable for AnimsTable {

    fn encode<W: WriteBytes>(&mut self, buffer: &mut W, _extra_data: &Option<EncodeableExtraData>) -> Result<()> {
        buffer.write_u32(self.version)?;

        match self.version {
            2 => self.write_v2(buffer)?,
            _ => Err(RLibError::DecodingAnimFragmentUnsupportedVersion(self.version as usize))?,
        };

        Ok(())
    }
}
