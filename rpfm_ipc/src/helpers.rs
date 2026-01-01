//---------------------------------------------------------------------------//
// Copyright (c) 2017-2024 I&smael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Module with all the code that can be considered "backend" of the UI, but should not be on the libs.

use getset::Getters;
use rayon::prelude::*;
use serde_derive::{Serialize, Deserialize};

use std::fmt::{self, Display};

use rpfm_extensions::dependencies::Dependencies;
use rpfm_extensions::search::{GlobalSearch, SearchSource};

use rpfm_lib::compression::CompressionFormat;
use rpfm_lib::games::{*, pfh_file_type::PFHFileType, pfh_version::PFHVersion};
use rpfm_lib::files::{animpack::*, Container, ContainerPath, db::*, FileType, pack::*, RFile, text::TextFormat, video::*};

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This struct is a reduced version of the `PackFile` one, used to pass just the needed data to an UI.
///
/// Don't create this one manually. Get it `From` the `PackFile` one, and use it as you need it.
#[derive(Clone, Debug, Default, Getters, Serialize, Deserialize)]
#[getset(get = "pub")]
pub struct ContainerInfo {

    /// The name of the PackFile's file, if exists. If not, then this should be empty.
    file_name: String,

    /// The path of the PackFile on disk, if exists. If not, then this should be empty.
    file_path: String,

    /// The version of the PackFile.
    pfh_version: PFHVersion,

    /// The type of the PackFile.
    pfh_file_type: PFHFileType,

    /// The bitmasks applied to the PackFile.
    bitmask: PFHFlags,

    /// If the container needs to be compress on save. None for no compression.
    compress: CompressionFormat,

    /// The timestamp of the last time the PackFile was saved.
    timestamp: u64,
}

/// This struct represents the detailed info about the `PackedFile` we can provide to whoever request it.
#[derive(Clone, Debug, Default, Getters, Serialize, Deserialize)]
#[getset(get = "pub")]
pub struct RFileInfo {

    /// This is the path of the `PackedFile`.
    path: String,

    /// This is the name of the `Container` this file belongs to.
    container_name: Option<String>,

    /// This is the ***Last Modified*** time.
    timestamp: Option<u64>,

    file_type: FileType,

    // If the `PackedFile` is compressed or not.
    //is_compressed: bool,

    // If the `PackedFile` is encrypted or not.
    //is_encrypted: bool,

    // If the `PackedFile` has been cached or not.
    //is_cached: bool,

    // The type of the cached `PackedFile`.
    //cached_type: String,
}

/// This struct represents the detailed info about the `PackedFile` we can provide to whoever request it.
#[derive(Clone, Debug, Default, Getters, Serialize, Deserialize)]
#[getset(get = "pub")]
pub struct VideoInfo {

    /// Format of the video file
    format: SupportedFormats,

    /// Version number.
    version: u16,

    /// Codec FourCC (usually 'VP80').
    codec_four_cc: String,

    /// Width of the video in pixels.
    width: u16,

    /// Height of the video in pixels.
    height: u16,

    /// Number of frames on the video.
    num_frames: u32,

    /// Framerate of the video.
    framerate: f32,
}

/// This struct contains the minimal data needed (mainly paths), to know what we have loaded in out dependencies.
///
/// NOTE: As this is intended to be a "Just use it and discard it" struct, we allow public members to make operations
/// where we can move out of here faster.
#[derive(Debug, Clone, Default, Getters, Serialize, Deserialize)]
#[getset(get = "pub")]
pub struct DependenciesInfo {

    /// Full PackedFile-like paths of each asskit-only table.
    pub asskit_tables: Vec<RFileInfo>,

    /// Full list of vanilla PackedFile paths.
    pub vanilla_packed_files: Vec<RFileInfo>,

    /// Full list of parent PackedFile paths.
    pub parent_packed_files: Vec<RFileInfo>,
}

/// This enum represents the source of the data in the view.
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, Ord, PartialOrd, Serialize, Deserialize)]
pub enum DataSource {

    /// This means the data is from somewhere in our PackFile.
    PackFile,

    /// This means the data is from one of the game files.
    GameFiles,

    /// This means the data comes from a parent PackFile.
    ParentFiles,

    /// This means the data comes from the AssKit files.
    AssKitFiles,

    /// This means the data comes from an external file.
    ExternalFile,
}

/// This enum contains the data needed to create a new PackedFile.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum NewFile {

    /// Name of the file.
    AnimPack(String),

    /// Name of the file, Name of the Table, Version of the Table.
    DB(String, String, i32),

    /// Name of the Table.
    Loc(String),

    /// Name of the file, version of the file, and a list of entries that must be cloned from existing values in vanilla files (from, to).
    PortraitSettings(String, u32, Vec<(String, String)>),

    /// Name of the file and its format.
    Text(String, TextFormat),

    /// Name of the file.
    VMD(String),

    /// Name of the file.
    WSModel(String),
}

/// This enum controls the possible responses from the server when checking for an update.
#[derive(Debug, serde_derive::Serialize, serde_derive::Deserialize)]
pub enum APIResponse {

    /// This means a beta update was found.
    NewBetaUpdate(String),

    /// This means a major stable update was found.
    NewStableUpdate(String),

    /// This means a minor stable update was found.
    NewUpdateHotfix(String),

    /// This means no update was found.
    NoUpdate,

    /// This means don't know if there was an update or not, because the version we got was invalid.
    UnknownVersion,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

impl From<&Pack> for ContainerInfo {
    fn from(pack: &Pack) -> Self {

        // If we have no disk file for the pack, it's a new one.
        let file_name = if pack.disk_file_path().is_empty() {
            "new_file.pack"
        } else {
            pack.disk_file_path().split('/').next_back().unwrap_or("unknown.pack")
        };

        Self {
            file_name: file_name.to_string(),
            file_path: pack.disk_file_path().to_string(),
            pfh_version: *pack.header().pfh_version(),
            pfh_file_type: *pack.header().pfh_file_type(),
            bitmask: *pack.header().bitmask(),
            timestamp: *pack.header().internal_timestamp(),
            compress: pack.compression_format(),
        }
    }
}

/// NOTE: DO NOT USE THIS FOR ANIMPACKS WITHIN PACKS.
///
/// It sets the path and name wrong in those cases.
impl From<&AnimPack> for ContainerInfo {
    fn from(animpack: &AnimPack) -> Self {
        Self {
            file_name: animpack.disk_file_path().split('/').next_back().unwrap_or("unknown.animpack").to_string(),
            file_path: animpack.disk_file_path().to_string(),
            ..Default::default()
        }
    }
}

impl From<&RFileInfo> for ContainerInfo {
    fn from(file_info: &RFileInfo) -> Self {
        Self {
            file_name: ContainerPath::File(file_info.path().to_owned()).name().unwrap_or("unknown").to_string(),
            file_path: file_info.path().to_owned(),
            ..Default::default()
        }
    }
}

impl From<&RFile> for RFileInfo {
    fn from(rfile: &RFile) -> Self {
        //let is_cached = !matches!(rfile.get_ref_decoded(), DecodedPackedFile::Unknown);
        //let cached_type = if let DecodedPackedFile::Unknown = rfile.get_ref_decoded() { "Not Yet Cached".to_owned() }
        //else { format!("{:?}", PackedFileType::from(rfile.get_ref_decoded())) };
        Self {
            path: rfile.path_in_container_raw().to_owned(),
            container_name: rfile.container_name().clone(),
            timestamp: rfile.timestamp(),
            file_type: rfile.file_type(),
            //is_compressed: rfile.get_ref_raw().get_compression_state(),
            //is_encrypted: rfile.get_ref_raw().get_encryption_state(),
            //is_cached,
            //cached_type,
        }
    }
}

impl From<&Video> for VideoInfo {
    fn from(video: &Video) -> Self {
        Self {
            format: *video.format(),
            version: *video.version(),
            codec_four_cc: video.codec_four_cc().to_string(),
            width: *video.width(),
            height: *video.height(),
            num_frames: *video.num_frames(),
            framerate: *video.framerate(),
        }
    }
}

impl DependenciesInfo {
    pub fn new(dependencies: &Dependencies, table_name_logic: &VanillaDBTableNameLogic) -> Self {
        let asskit_tables = dependencies.asskit_only_db_tables().values().map(|table| {
            let table_name = match table_name_logic {
                VanillaDBTableNameLogic::DefaultName(ref name) => name,
                VanillaDBTableNameLogic::FolderName => table.table_name(),
            };

            RFileInfo::from_db(table, table_name)
        }).collect::<Vec<RFileInfo>>();

        let vanilla_packed_files = dependencies.vanilla_loose_files()
            .par_iter()
            .chain(dependencies.vanilla_files().par_iter())
            .map(|(_, value)| From::from(value))
            .collect::<Vec<RFileInfo>>();

        let parent_packed_files = dependencies.parent_files()
            .par_iter()
            .map(|(_, value)| From::from(value))
            .collect::<Vec<RFileInfo>>();

        Self {
            asskit_tables,
            vanilla_packed_files,
            parent_packed_files,
        }
    }
}

impl RFileInfo {

    /// This function returns the PackedFileInfo for all the PackedFiles the current search has searched on.
    pub fn info_from_global_search(global_search: &GlobalSearch, pack: &Pack) -> Vec<Self> {
        let types = global_search.search_on().types_to_search();

        // Only return info of stuff on the local Pack.
        if global_search.source() == &SearchSource::Pack {
            pack.files_by_type(&types).iter().map(|x| From::from(*x)).collect()
        } else {
            vec![]
        }
    }

    pub fn table_name(&self) -> Option<&str> {
        if self.file_type == FileType::DB {
            self.path().split('/').collect::<Vec<_>>().get(1).cloned()
        } else {
            None
        }
    }

    pub fn from_db(db: &DB, table_file_name: &str) -> Self {
        Self {
            path: format!("db/{}/{}", db.table_name(), table_file_name),
            container_name: None,
            timestamp: None,
            file_type: FileType::DB,
        }
    }
}

impl Display for DataSource {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Display::fmt(match self {
            Self::PackFile => "PackFile",
            Self::GameFiles => "GameFiles",
            Self::ParentFiles => "ParentFiles",
            Self::AssKitFiles => "AssKitFiles",
            Self::ExternalFile => "ExternalFile",
        }, f)
    }
}

impl From<&str> for DataSource {
    fn from(value: &str) -> Self {
        match value {
            "PackFile" => Self::PackFile,
            "GameFiles" => Self::GameFiles,
            "ParentFiles" => Self::ParentFiles,
            "AssKitFiles" => Self::AssKitFiles,
            "ExternalFile" => Self::ExternalFile,
            _ => unreachable!("from data source {}", value)
        }
    }
}
