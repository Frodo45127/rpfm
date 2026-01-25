//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! PackFile type classification for load order control.
//!
//! This module defines the different types of PackFiles and their loading behavior
//! in Total War games.
//!
//! # PackFile Types
//!
//! Total War games recognize five PackFile types that control loading order and behavior:
//!
//! 1. **Boot** (0): Core game files, loaded first
//! 2. **Release** (1): Main game data, loaded second
//! 3. **Patch** (2): Official patches and updates, loaded third
//! 4. **Mod** (3): User mods, only loaded if enabled in launcher, loaded fourth
//! 5. **Movie** (4): Cinematic files and special data, always loaded, loaded last
//!
//! # Load Order
//!
//! The game loads PackFiles in the order shown above. Within each type, files are
//! loaded alphabetically. This means:
//! - Boot/Release/Patch files always override each other in type order
//! - Mod files only load if enabled in the game's mod manager
//! - Movie files always load and can override everything else
//!
//! # Usage for Modding
//!
//! For normal mod creation, use [`PFHFileType::Mod`]. This ensures:
//! - Users can enable/disable the mod via the launcher
//! - The mod doesn't interfere with other mod types
//! - Standard load order behavior
//!
//! [`PFHFileType::Movie`] can be used for mods that should always load, but this
//! bypasses the mod manager and should be used sparingly.

use serde_derive::{Serialize, Deserialize};

use std::{fmt, fmt::Display};

use crate::error::RLibError;

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// PackFile type determining load order and behavior.
///
/// This enum represents the classification of a PackFile, which controls when and
/// how the game loads it. Types are listed in load order.
///
/// The numeric values in parentheses are the values stored in the PackFile header.
///
/// # Ordering
///
/// The enum implements [`Ord`] and [`PartialOrd`] based on load order, so
/// `Boot < Release < Patch < Mod < Movie`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[repr(u32)]
pub enum PFHFileType {

    /// Core game boot files **(0)**.
    ///
    /// Contains essential game startup data. Used only by Creative Assembly.
    /// Not useful for modding.
    Boot = 0,

    /// Main game data files **(1)**.
    ///
    /// Contains the base game content. Used only by Creative Assembly.
    /// Not useful for modding.
    Release = 1,

    /// Official patch and update files **(2)**.
    ///
    /// Contains patches and official updates. Used only by Creative Assembly.
    /// Not useful for modding.
    Patch = 2,

    /// User mod files **(3)**.
    ///
    /// Standard type for player-created mods. Only loaded if enabled in the
    /// game's mod manager or launcher. **Use this for normal mod creation.**
    Mod = 3,

    /// Cinematic and always-loaded files **(4)**.
    ///
    /// Used for movies and special data that should always load. Unlike Mod
    /// files, these always load regardless of launcher settings. Can be used
    /// for mods but bypasses mod manager control.
    Movie = 4
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

/// Implementation of `PFHFileType`.
impl PFHFileType {

    /// Returns the numeric value of this PackFile type.
    ///
    /// This is the value stored in the PackFile header to identify its type.
    ///
    /// # Returns
    ///
    /// The type value as `u32` (0-4).
    ///
    /// # Example
    ///
    /// ```ignore
    /// use rpfm_lib::games::pfh_file_type::PFHFileType;
    ///
    /// assert_eq!(PFHFileType::Boot.value(), 0);
    /// assert_eq!(PFHFileType::Mod.value(), 3);
    /// assert_eq!(PFHFileType::Movie.value(), 4);
    /// ```
    pub fn value(&self) -> u32 {
        *self as u32
    }
}

/// Default implementation of `PFHFileType`.
impl Default for PFHFileType {
    fn default() -> Self {
        Self::Mod
    }
}

/// Display implementation of `PFHFileType`.
impl Display for PFHFileType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            PFHFileType::Boot => write!(f, "Boot"),
            PFHFileType::Release => write!(f, "Release"),
            PFHFileType::Patch => write!(f, "Patch"),
            PFHFileType::Mod => write!(f, "Mod"),
            PFHFileType::Movie => write!(f, "Movie")
        }
    }
}

/// TryFrom implementation of `PFHFileType`.
impl TryFrom<u32> for PFHFileType {
    type Error = RLibError;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            x if x == PFHFileType::Boot as u32 => Ok(PFHFileType::Boot),
            x if x == PFHFileType::Release as u32 => Ok(PFHFileType::Release),
            x if x == PFHFileType::Patch as u32 => Ok(PFHFileType::Patch),
            x if x == PFHFileType::Mod as u32 => Ok(PFHFileType::Mod),
            x if x == PFHFileType::Movie as u32 => Ok(PFHFileType::Movie),
            _ => Err(RLibError::UnknownPFHFileType(value.to_string())),
        }
    }
}

impl TryFrom<&str> for PFHFileType {
    type Error = RLibError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "boot" => Ok(PFHFileType::Boot),
            "release" => Ok(PFHFileType::Release),
            "patch" => Ok(PFHFileType::Patch),
            "mod" => Ok(PFHFileType::Mod),
            "movie" => Ok(PFHFileType::Movie),
            _ => Err(RLibError::UnknownPFHFileType(value.to_string())),
        }
    }
}
