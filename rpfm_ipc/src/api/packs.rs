//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Methods about the open packs.

use serde::{Deserialize, Serialize};

use rpfm_lib::compression::CompressionFormat;
use rpfm_lib::games::{pfh_file_type::PFHFileType, pfh_version::PFHVersion};

use crate::messages::OperationalMode;

use super::Request;

/// Short description of an open pack.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PackSummary {

    /// Key identifying the pack in every method that works on it.
    pub key: String,

    /// File name of the pack.
    pub name: String,

    /// Path of the pack on disk. For packs never saved, only their name.
    pub path: String,

    /// Type of the pack.
    pub pack_type: PFHFileType,

    /// Amount of files in the pack.
    pub file_count: usize,
}

/// `pack.info`: returns the details of an open pack.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GetPackInfo {

    /// Key of the pack.
    pub pack: String,
}

/// Details of an open pack.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PackDetails {

    /// Short description of the pack.
    #[serde(flatten)]
    pub summary: PackSummary,

    /// Version of the pack format.
    pub version: PFHVersion,

    /// Compression format of the files of the pack.
    pub compression: CompressionFormat,

    /// If the index of the pack is encrypted.
    pub index_encrypted: bool,

    /// If the data of the pack is encrypted.
    pub data_encrypted: bool,

    /// If the index of the pack includes the timestamp of each file.
    pub index_includes_timestamp: bool,

    /// Packs this pack depends on, loaded as parent files.
    pub dependencies: Vec<PackDependency>,

    /// If the pack is in normal or MyMod mode.
    pub operational_mode: OperationalMode,
}

/// A pack another pack depends on.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PackDependency {

    /// If the dependency is loaded.
    pub enabled: bool,

    /// File name of the pack.
    pub name: String,
}

impl Request for GetPackInfo {
    const METHOD: &'static str = "pack.info";
    type Response = PackDetails;
}
