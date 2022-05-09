//---------------------------------------------------------------------------//
// Copyright (c) 2017-2022 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

use std::{fmt, fmt::Display};

/// These are the types the PackFiles can have.
const FILE_TYPE_BOOT: u32 = 0;
const FILE_TYPE_RELEASE: u32 = 1;
const FILE_TYPE_PATCH: u32 = 2;
const FILE_TYPE_MOD: u32 = 3;
const FILE_TYPE_MOVIE: u32 = 4;

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This enum represents the **Type** of a PackFile.
///
/// The types here are sorted in the same order they'll load when the game starts.
/// The number in their docs is their numeric value when read from a PackFile.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PFHFileType {

    /// **(0)**: Used in CA PackFiles, not useful for modding.
    Boot,

    /// **(1)**: Used in CA PackFiles, not useful for modding.
    Release,

    /// **(2)**: Used in CA PackFiles, not useful for modding.
    Patch,

    /// **(3)**: Used for mods. PackFiles of this type are only loaded in the game if they are enabled in the Mod Manager/Launcher.
    Mod,

    /// **(4)** Used in CA PackFiles and for some special mods. Unlike `Mod` PackFiles, these ones always get loaded.
    Movie,

    /// Wildcard for any type that doesn't fit in any of the other categories. The type's value is stored in the Variant.
    Other(u32),
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

/// Implementation of `PFHFileType`.
impl PFHFileType {

    /// This function returns the PackFile's **Type** in `u32` format. To know what value corresponds with what type, check their definition's comment.
    pub fn value(self) -> u32 {
        match self {
            PFHFileType::Boot => FILE_TYPE_BOOT,
            PFHFileType::Release => FILE_TYPE_RELEASE,
            PFHFileType::Patch => FILE_TYPE_PATCH,
            PFHFileType::Mod => FILE_TYPE_MOD,
            PFHFileType::Movie => FILE_TYPE_MOVIE,
            PFHFileType::Other(value) => value
        }
    }

    /// This function returns the PackFile's Type corresponding to the provided value.
    pub fn file_type(value: u32) -> Self {
        match value {
            FILE_TYPE_BOOT => PFHFileType::Boot,
            FILE_TYPE_RELEASE => PFHFileType::Release,
            FILE_TYPE_PATCH => PFHFileType::Patch,
            FILE_TYPE_MOD => PFHFileType::Mod,
            FILE_TYPE_MOVIE => PFHFileType::Movie,
            _ => PFHFileType::Other(value),
        }
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
            PFHFileType::Movie => write!(f, "Movie"),
            PFHFileType::Other(version) => write!(f, "Other: {}", version),
        }
    }
}

/// Implementation of trait `Default` for `PFHFileType`.
impl Default for PFHFileType {
    fn default() -> Self {
        Self::Mod
    }
}
