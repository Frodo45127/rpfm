//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! PackFile format version identification.
//!
//! This module defines the different PackFile format versions used across Total War games.
//! Each version represents a different internal format with varying capabilities and structure.
//!
//! # PackFile Versions
//!
//! Total War games have evolved their PackFile format over time:
//!
//! - **PFH6**: Troy (v1.3.0+)
//! - **PFH5**: Warhammer 2, Warhammer 3, Three Kingdoms, Troy (pre-1.3.0), Pharaoh, Pharaoh Dynasties, Arena
//! - **PFH4**: Warhammer 1, Attila, Rome 2, Thrones of Britannia
//! - **PFH3**: Shogun 2 (post-patch 15)
//! - **PFH2**: Shogun 2 (pre-patch 15, before Fall of the Samurai expansion)
//! - **PFH0**: Napoleon, Empire
//!
//! # Version Identification
//!
//! PackFiles are identified by a 4-byte "preamble" or "magic number" at the start:
//! ```text
//! Offset 0x00: "PFH6" or "PFH5" or "PFH4" or "PFH3" or "PFH2" or "PFH0"
//! ```
//!
//! # Format Differences
//!
//! Different versions support different features:
//! - Compression algorithms (newer versions support more formats)
//! - Header structure and metadata
//! - File entry format
//! - Timestamps
//! - File path encoding
//!
//! # Usage
//!
//! ```ignore
//! use rpfm_lib::games::pfh_version::PFHVersion;
//!
//! // Get version from preamble string
//! let version = PFHVersion::version("PFH5").unwrap();
//! assert_eq!(version, PFHVersion::PFH5);
//!
//! // Get preamble string from version
//! assert_eq!(version.value(), "PFH5");
//! ```

use std::{fmt, fmt::Display};
use serde_derive::{Serialize, Deserialize};

use crate::error::{RLibError, Result};

/// Preamble string for PFH6 format
const PFH6_PREAMBLE: &str = "PFH6";
/// Preamble string for PFH5 format
const PFH5_PREAMBLE: &str = "PFH5";
/// Preamble string for PFH4 format
const PFH4_PREAMBLE: &str = "PFH4";
/// Preamble string for PFH3 format
const PFH3_PREAMBLE: &str = "PFH3";
/// Preamble string for PFH2 format
const PFH2_PREAMBLE: &str = "PFH2";
/// Preamble string for PFH0 format
const PFH0_PREAMBLE: &str = "PFH0";

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// PackFile format version.
///
/// Identifies the internal format version of a PackFile, which determines its
/// structure, capabilities, and which games can read it.
///
/// # Version History
///
/// Each variant represents a different format evolution in Total War's PackFile system.
/// Newer versions generally support more features but may not be readable by older games.
///
/// # Compatibility
///
/// Games can typically read their own version and sometimes older versions, but cannot
/// read newer versions. For maximum compatibility when creating mods, use the version
/// matching the target game.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PFHVersion {

    /// Used in Troy (v1.3.0+).
    PFH6,

    /// Used in Warhammer 2, Warhammer 3, Three Kingdoms, Troy (pre-1.3.0), Pharaoh, Pharaoh Dynasties, Arena.
    PFH5,

    /// Used in Warhammer 1, Attila, Rome 2, Thrones of Britannia.
    PFH4,

    /// Used in Shogun 2.
    PFH3,

    /// Used in Shogun 2 before patch 15 (Fall of the Samurai expansion).
    PFH2,

    /// Used in Napoleon and Empire.
    PFH0
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

/// Implementation of `PFHVersion`.
impl PFHVersion {

    /// Returns the 4-byte preamble string for this format version.
    ///
    /// This is the "magic number" that appears at offset 0x00 in the PackFile header
    /// to identify its format version.
    ///
    /// # Returns
    ///
    /// A 4-character string like `"PFH6"`, `"PFH5"`, etc.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use rpfm_lib::games::pfh_version::PFHVersion;
    ///
    /// assert_eq!(PFHVersion::PFH6.value(), "PFH6");
    /// assert_eq!(PFHVersion::PFH5.value(), "PFH5");
    /// assert_eq!(PFHVersion::PFH0.value(), "PFH0");
    /// ```
    pub fn value(&self) -> &str {
        match *self {
            PFHVersion::PFH6 => PFH6_PREAMBLE,
            PFHVersion::PFH5 => PFH5_PREAMBLE,
            PFHVersion::PFH4 => PFH4_PREAMBLE,
            PFHVersion::PFH3 => PFH3_PREAMBLE,
            PFHVersion::PFH2 => PFH2_PREAMBLE,
            PFHVersion::PFH0 => PFH0_PREAMBLE,
        }
    }

    /// Parses a preamble string into a format version.
    ///
    /// Converts a 4-byte preamble read from a PackFile header into the corresponding
    /// [`PFHVersion`] enum variant.
    ///
    /// # Arguments
    ///
    /// * `value` - The 4-character preamble string (e.g., `"PFH5"`)
    ///
    /// # Returns
    ///
    /// Returns the matching [`PFHVersion`], or an error if the preamble is not recognized.
    ///
    /// # Errors
    ///
    /// Returns [`RLibError::UnknownPFHVersion`] if the preamble doesn't match any known version.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use rpfm_lib::games::pfh_version::PFHVersion;
    ///
    /// let version = PFHVersion::version("PFH5").unwrap();
    /// assert_eq!(version, PFHVersion::PFH5);
    ///
    /// // Invalid preamble returns error
    /// assert!(PFHVersion::version("PFH9").is_err());
    /// ```
    pub fn version(value: &str) -> Result<Self> {
        match value {
            PFH6_PREAMBLE => Ok(PFHVersion::PFH6),
            PFH5_PREAMBLE => Ok(PFHVersion::PFH5),
            PFH4_PREAMBLE => Ok(PFHVersion::PFH4),
            PFH3_PREAMBLE => Ok(PFHVersion::PFH3),
            PFH2_PREAMBLE => Ok(PFHVersion::PFH2),
            PFH0_PREAMBLE => Ok(PFHVersion::PFH0),
            _ => Err(RLibError::UnknownPFHVersion(value.to_owned())),
        }
    }
}

/// Display implementation of `PFHVersion`.
impl Display for PFHVersion {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            PFHVersion::PFH6 => write!(f, "PFH6"),
            PFHVersion::PFH5 => write!(f, "PFH5"),
            PFHVersion::PFH4 => write!(f, "PFH4"),
            PFHVersion::PFH3 => write!(f, "PFH3"),
            PFHVersion::PFH2 => write!(f, "PFH2"),
            PFHVersion::PFH0 => write!(f, "PFH0"),
        }
    }
}

/// Implementation of trait `Default` for `PFHVersion`.
impl Default for PFHVersion {
    fn default() -> Self {
        Self::PFH6
    }
}
