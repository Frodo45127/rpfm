//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

use std::{fmt, fmt::Display};
use serde_derive::{Serialize, Deserialize};

use crate::error::{RLibError, Result};

/// These are the different Preamble/Id the PackFiles can have.
const PFH6_PREAMBLE: &str = "PFH6"; // PFH6
const PFH5_PREAMBLE: &str = "PFH5"; // PFH5
const PFH4_PREAMBLE: &str = "PFH4"; // PFH4
const PFH3_PREAMBLE: &str = "PFH3"; // PFH3
const PFH2_PREAMBLE: &str = "PFH2"; // PFH2
const PFH0_PREAMBLE: &str = "PFH0"; // PFH0

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This enum represents the **Version** of a PackFile.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PFHVersion {

    /// Used in Troy since patch 1.3.0 for mods.
    PFH6,

    /// Used in Warhammer 2, Three Kingdoms and Arena.
    PFH5,

    /// Used in Warhammer 1, Attila, Rome 2, and Thrones of Brittania.
    PFH4,

    /// Used in Shogun 2.
    PFH3,

    /// Also used in Shogun 2.
    PFH2,

    /// Used in Napoleon and Empire.
    PFH0
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

/// Implementation of `PFHVersion`.
impl PFHVersion {

    /// This function returns the PackFile's *Id/Preamble* (his 4 first bytes) as a `&str`.
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

    /// This function returns the PackFile's `PFHVersion` corresponding to the provided value, or an error if the provided value is not a valid `PFHVersion`.
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
