//---------------------------------------------------------------------------//
// Copyright (c) 2017-2023 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

use serde_derive::{Serialize, Deserialize};

use std::{fmt, fmt::Display};

use crate::error::RLibError;

/// These are the types the PackFiles can have.
const FILE_TYPE_BOOT: isize = 0;
const FILE_TYPE_RELEASE: isize = 1;
const FILE_TYPE_PATCH: isize = 2;
const FILE_TYPE_MOD: isize = 3;
const FILE_TYPE_MOVIE: isize = 4;

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This enum represents the **Type** of a PackFile.
///
/// The types here are sorted in the same order they'll load when the game starts.
/// The number in their docs is their numeric value when read from a PackFile.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PFHFileType {

    /// **(0)**: Used in CA PackFiles, not useful for modding.
    Boot = FILE_TYPE_BOOT,

    /// **(1)**: Used in CA PackFiles, not useful for modding.
    Release = FILE_TYPE_RELEASE,

    /// **(2)**: Used in CA PackFiles, not useful for modding.
    Patch = FILE_TYPE_PATCH,

    /// **(3)**: Used for mods. PackFiles of this type are only loaded in the game if they are enabled in the Mod Manager/Launcher.
    Mod = FILE_TYPE_MOD,

    /// **(4)** Used in CA PackFiles and for some special mods. Unlike `Mod` PackFiles, these ones always get loaded.
    Movie = FILE_TYPE_MOVIE
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

/// Implementation of `PFHFileType`.
impl PFHFileType {

    /// This function returns the PackFile's **Type** in `u32` format.
    /// To know what value corresponds with what type, check their definition's comment.
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
