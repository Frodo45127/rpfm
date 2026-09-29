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

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use std::path::PathBuf;

use rpfm_lib::compression::CompressionFormat;
use rpfm_lib::games::{pfh_file_type::PFHFileType, pfh_version::PFHVersion};

use crate::messages::OperationalMode;

use super::{Done, Request};

/// Short description of an open pack.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct PackSummary {

    /// Key identifying the pack in every method that works on it.
    pub key: String,

    /// File name of the pack.
    pub name: String,

    /// Path of the pack on disk. For packs never saved, only their name.
    pub path: String,

    /// Type of the pack: `Boot`, `Release`, `Patch`, `Mod` or `Movie`.
    #[schemars(with = "String")]
    pub pack_type: PFHFileType,

    /// Amount of files in the pack.
    pub file_count: usize,
}

/// `pack.info`: returns the details of an open pack.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct GetPackInfo {

    /// Key of the pack.
    pub pack: String,
}

/// Details of an open pack.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct PackDetails {

    /// Short description of the pack.
    #[serde(flatten)]
    pub summary: PackSummary,

    /// Version of the pack format, like `PFH5`.
    #[schemars(with = "String")]
    pub version: PFHVersion,

    /// Compression format of the files of the pack, like `None` or `Lz4`.
    #[schemars(with = "String")]
    pub compression: CompressionFormat,

    /// If the index of the pack is encrypted.
    pub index_encrypted: bool,

    /// If the data of the pack is encrypted.
    pub data_encrypted: bool,

    /// If the index of the pack includes the timestamp of each file.
    pub index_includes_timestamp: bool,

    /// Packs this pack depends on, loaded as parent files.
    pub dependencies: Vec<PackDependency>,

    /// If the pack is in normal mode (`"Normal"`) or MyMod mode (`{"MyMod": [game_folder, mod_name]}`).
    #[schemars(with = "serde_json::Value")]
    pub operational_mode: OperationalMode,
}

/// A pack another pack depends on.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
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

/// `pack.new`: creates a new empty pack. It has no path on disk until it's saved with a path.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct NewPack {}

/// `pack.open`: opens one or more packs from disk, merged into a single one.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct OpenPack {

    /// Paths of the packs on disk. The first one gives the pack its key.
    pub paths: Vec<PathBuf>,

    /// If file data is read from disk only when needed. Defaults to the server's setting.
    #[serde(default)]
    pub lazy_loading: Option<bool>,
}

/// `pack.open_vanilla`: opens all the vanilla packs of the selected game, merged into a single one.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct OpenVanillaPacks {}

/// `pack.close`: closes an open pack, discarding unsaved changes.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ClosePack {

    /// Key of the pack.
    pub pack: String,
}

/// `pack.close_all`: closes all open packs, discarding unsaved changes.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct CloseAllPacks {}

/// `pack.save`: saves an open pack to disk.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct SavePack {

    /// Key of the pack.
    pub pack: String,

    /// Path to save the pack to. If not set, the pack is saved to its current path.
    #[serde(default)]
    pub path: Option<PathBuf>,

    /// If files that failed to decode are removed before saving. Use it if saving normally fails.
    #[serde(default)]
    pub clean: bool,

    /// If tables keep their GUID instead of getting a new one. Defaults to the server's setting.
    #[serde(default)]
    pub disable_uuid_regeneration: Option<bool>,

    /// If packs of the vanilla types (not `Mod` or `Movie`) can be saved. Defaults to the server's setting.
    #[serde(default)]
    pub allow_editing_ca_packs: Option<bool>,
}

/// `pack.update`: changes properties of an open pack. Only the fields set are changed.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct UpdatePack {

    /// Key of the pack.
    pub pack: String,

    /// New type of the pack: `Boot`, `Release`, `Patch`, `Mod` or `Movie`.
    #[serde(default)]
    #[schemars(with = "Option<String>")]
    pub pack_type: Option<PFHFileType>,

    /// New compression format of the pack, like `None` or `Lz4`. If the game doesn't support it, a supported one is used.
    #[serde(default)]
    #[schemars(with = "Option<String>")]
    pub compression: Option<CompressionFormat>,

    /// If the index of the pack is encrypted. Only PFH4 and newer packs support encryption.
    #[serde(default)]
    pub index_encrypted: Option<bool>,

    /// If the data of the pack is encrypted. Only PFH4 and newer packs support encryption.
    #[serde(default)]
    pub data_encrypted: Option<bool>,

    /// If the index of the pack includes the timestamp of each file.
    #[serde(default)]
    pub index_includes_timestamp: Option<bool>,

    /// New list of packs this pack depends on.
    #[serde(default)]
    pub dependencies: Option<Vec<PackDependency>>,

    /// New mode of the pack: `"Normal"`, or `{"MyMod": [game_folder, mod_name]}`.
    #[serde(default)]
    #[schemars(with = "Option<serde_json::Value>")]
    pub operational_mode: Option<OperationalMode>,
}

impl Request for NewPack {
    const METHOD: &'static str = "pack.new";
    type Response = PackSummary;
}

impl Request for OpenPack {
    const METHOD: &'static str = "pack.open";
    type Response = PackSummary;
}

impl Request for OpenVanillaPacks {
    const METHOD: &'static str = "pack.open_vanilla";
    type Response = PackSummary;
}

impl Request for ClosePack {
    const METHOD: &'static str = "pack.close";
    type Response = Done;
}

impl Request for CloseAllPacks {
    const METHOD: &'static str = "pack.close_all";
    type Response = Done;
}

impl Request for SavePack {
    const METHOD: &'static str = "pack.save";
    type Response = PackSummary;
}

impl Request for UpdatePack {
    const METHOD: &'static str = "pack.update";
    type Response = PackDetails;
}
