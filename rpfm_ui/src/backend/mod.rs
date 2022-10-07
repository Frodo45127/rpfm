//---------------------------------------------------------------------------//
// Copyright (c) 2017-2022 I&smael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

/*!
Module with all the code that can be considered "backend" of the UI, but should not be on the libs.
!*/

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

use getset::Getters;

use std::path::PathBuf;

use rpfm_extensions::dependencies::Dependencies;
use rpfm_extensions::search::{GlobalSearch, SearchSource};

use rpfm_lib::games::{*, pfh_file_type::PFHFileType, pfh_version::PFHVersion};
use rpfm_lib::files::{*, Container, FileType, RFile, animpack::*, pack::*};

use crate::GAME_SELECTED;
use crate::packedfile_views::DataSource;

/// This struct is a reduced version of the `PackFile` one, used to pass just the needed data to an UI.
///
/// Don't create this one manually. Get it `From` the `PackFile` one, and use it as you need it.
#[derive(Clone, Debug, Default, Getters)]
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

    /// If the container needs to be compress on save.
    compress: bool,

    /// The timestamp of the last time the PackFile was saved.
    timestamp: u64,
}

/// This struct represents the detailed info about the `PackedFile` we can provide to whoever request it.
#[derive(Clone, Debug, Default, Getters)]
#[getset(get = "pub")]
pub struct RFileInfo {

    /// This is the path of the `PackedFile`.
    path: String,

    /// This is the name of the `PackFile` this file belongs to.
    packfile_name: String,

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

/// This struct contains the minimal data needed (mainly paths), to know what we have loaded in out dependencies.
#[derive(Debug, Clone, Default, Getters)]
#[getset(get = "pub")]
pub struct DependenciesInfo {

    /// Full PackedFile-like paths of each asskit-only table.
    asskit_tables: Vec<RFileInfo>,

    /// Full list of vanilla PackedFile paths.
    vanilla_packed_files: Vec<RFileInfo>,

    /// Full list of parent PackedFile paths.
    parent_packed_files: Vec<RFileInfo>,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

impl From<&Pack> for ContainerInfo {
    fn from(pack: &Pack) -> Self {
        Self {
            file_name: pack.disk_file_path().split("/").last().unwrap_or("unknown.pack").to_string(),
            file_path: pack.disk_file_path().to_string(),
            pfh_version: *pack.header().pfh_version(),
            pfh_file_type: *pack.header().pfh_file_type(),
            bitmask: *pack.header().bitmask(),
            timestamp: *pack.header().internal_timestamp(),
            compress: *pack.compress(),
        }
    }
}

/// TODO: Pretty sure this doesn't work.
/// Yeah, this sets the path wrong.
impl From<&AnimPack> for ContainerInfo {
    fn from(animpack: &AnimPack) -> Self {
        Self {
            file_name: animpack.disk_file_path().split("/").last().unwrap_or("unknown.animpack").to_string(),
            file_path: animpack.disk_file_path().to_string(),
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
            packfile_name: rfile.file_name().unwrap().to_string(),
            timestamp: rfile.timestamp(),
            file_type: rfile.file_type(),
            //is_compressed: rfile.get_ref_raw().get_compression_state(),
            //is_encrypted: rfile.get_ref_raw().get_encryption_state(),
            //is_cached,
            //cached_type,
        }
    }
}

impl From<&Dependencies> for DependenciesInfo {
    fn from(dependencies: &Dependencies) -> Self {
        let table_name_logic = GAME_SELECTED.read().unwrap().vanilla_db_table_name_logic();
        Self {
            asskit_tables: dependencies.asskit_only_db_tables().iter().map(|(_, table)| {
                let table_name = match table_name_logic {
                    VanillaDBTableNameLogic::DefaultName(ref name) => name.to_owned(),
                    VanillaDBTableNameLogic::FolderName => table.table_name().to_owned(),
                };

                RFileInfo::from(&RFile::new_from_decoded(&RFileDecoded::DB(table.clone()), 0, &format!("db/{}/{}", table.table_name(), table_name)))
            }).collect(),
            vanilla_packed_files: dependencies.vanilla_files().values().map(From::from).collect(),
            parent_packed_files: dependencies.parent_files().values().map(From::from).collect(),
        }
    }
}

impl RFileInfo {

    /// This function returns the PackedFileInfo for all the PackedFiles the current search has searched on.
    pub fn info_from_global_search(global_search: &GlobalSearch, pack: &Pack) -> Vec<Self> {
        let mut types = vec![];
        if global_search.search_on_dbs { types.push(FileType::DB); }
        if global_search.search_on_locs { types.push(FileType::Loc); }
        if global_search.search_on_texts { types.push(FileType::Text); }

        // Only return info of stuff on the local Pack.
        if global_search.source == SearchSource::Pack {
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
}
