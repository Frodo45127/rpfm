//---------------------------------------------------------------------------//
// Copyright (c) 2017-2022 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

/*!
Module with all the code to interact with individual PackedFiles.

This module contains all the code related with the interaction with individual PackFiles,
meaning the code that takes care of loading/writing their data from/to disk.

You'll rarely have to touch anything here.
!*/

use anyhow::{anyhow, Result};
use filepath::FilePath;

use std::collections::hash_map::DefaultHasher;
use std::convert::TryFrom;
use std::hash::{Hash, Hasher};
use std::io::prelude::*;
use std::io::{BufReader, Read, SeekFrom};
use std::fs::File;
use std::sync::{Arc, Mutex};

use rpfm_common::{compression::*, schema::Schema, utils::*};
use getset::*;
use rpfm_files::{db::DB, loc::Loc, PackedFileType};

use crate::*;
//use crate::packedfile::animpack::AnimPacked;

//---------------------------------------------------------------------------//
//                              Enum & Structs
//---------------------------------------------------------------------------//

/// This struct represents a `PackedFile` in memory.
#[derive(Clone, Debug, PartialEq)]
pub struct PackedFile {
    raw: RawPackedFile,
    //decoded: DecodedPackedFile,
}

/// This struct represents a `PackedFile` in memory in his raw form.
#[derive(Clone, Debug, PartialEq)]
pub struct RawPackedFile {

    /// The path of the `PackedFile` inside the `PackFile`.
    path: Vec<String>,

    /// Name of the original `PackFile` containing it. To know from where a `PackedFile` came when loading multiple PackFiles as one.
    packfile_name: String,

    /// The '*Last Modified Date*' of the `PackedFile`, encoded in `i64`. Only in PackFiles with the appropriate flag enabled..
    timestamp: i64,

    /// If the data should be compressed when saving it to disk. Only available from `PFHVersion::PFH5` onwards.
    should_be_compressed: bool,

    /// If the data should be encrypted when saving it to disk. If it should, it contains `Some(PFHVersion)`, being `PFHVersion` the one of the game this `PackedFile` is for.
    should_be_encrypted: Option<PFHVersion>,

    /// the data of the PackedFile. Use the getter/setter functions to interact with it.
    data: PackedFileData,
}

/// This enum represents the data of a `PackedFile`, in his current state.
#[derive(Clone, Debug, PartialEq)]
pub enum PackedFileData {

    /// The data is loaded to memory and the variant holds the data and info about the current state of the data (data, is_compressed, is_encrypted).
    OnMemory(Vec<u8>, bool, Option<PFHVersion>),

    /// The data is not loaded to memory and the variant holds the info needed to get the data loaded to memory on demand.
    OnDisk(RawOnDisk),
}

/// This struct contains the stuff needed to read the data of a particular PackedFile from disk.
#[derive(Clone, Debug, PartialEq, Getters)]
pub struct RawOnDisk {

    /// Reader over the PackFile containing the PackedFile.
    //reader: Arc<Mutex<BufReader<File>>>,
    start: u64,
    size: u32,
    is_compressed: bool,
    is_encrypted: Option<PFHVersion>,

    /// Last Modified Date on disk of the PackFile containing this PackedFile.
    last_modified_date_pack: i64,

    // Hash of the PackedFile's data, to ensure we don't grab the wrong data.
    //hash: Arc<Mutex<u64>>,
}

/// This struct contains a "Cached" version of a PackedFile, so we can serialize it and store it.
///
/// This is mostly a 1:1 map of the RawOnDisk with extras.
#[derive(Clone, Debug, Getters, Serialize, Deserialize)]
pub struct CachedPackedFile {
    pack_file_path: String,
    packed_file_path: String,
    data_start: u64,
    data_size: u32,
    is_compressed: bool,
    is_encrypted: Option<PFHVersion>,
    last_modified_date_pack: i64,
}

/// This struct represents the detailed info about the `PackedFile` we can provide to whoever request it.
#[derive(Clone, Debug, Default)]
pub struct PackedFileInfo {

    /// This is the path of the `PackedFile`.
    pub path: Vec<String>,

    /// This is the name of the `PackFile` this file belongs to.
    pub packfile_name: String,

    /// This is the ***Last Modified*** time.
    pub timestamp: i64,

    /// If the `PackedFile` is compressed or not.
    pub is_compressed: bool,

    /// If the `PackedFile` is encrypted or not.
    pub is_encrypted: bool,

    /// If the `PackedFile` has been cached or not.
    pub is_cached: bool,

    /// The type of the cached `PackedFile`.
    pub cached_type: String,
}

//---------------------------------------------------------------------------//
//                       Enum & Structs Implementations
//---------------------------------------------------------------------------//

/// Implementation of `PackedFile`.
impl PackedFile {
/*


    /// This function creates a new empty `PackedFile` of the provided type and path.
    pub fn new_from_type_and_path(
        packed_file_type: PackedFileType,
        path: Vec<String>,
        dependencies: &Dependencies,
    ) -> Result<Self> {

        // Depending on their type, we do different things to prepare the PackedFile and get his data.
        let schema = SCHEMA.read().unwrap();
        let data = match packed_file_type {

            // For locs, we just create them with their last definition.
            PackedFileType::Loc => {
                let definition = match *schema {
                    Some(ref schema) => schema.get_ref_last_definition_loc()?,
                    None => return Err(ErrorKind::SchemaNotFound.into())
                };
                DecodedPackedFile::Loc(Loc::new(definition))
            },

            // For dbs, we create them with their last definition, if we found one, and their table name.
            PackedFileType::DB => {
                let table_name = path.get(1).ok_or_else(|| Error::from(ErrorKind::DBTableIsNotADBTable))?;
                let table_definition = match *schema {
                    Some(ref schema) => schema.get_ref_last_definition_db(table_name, dependencies)?,
                    None => return Err(ErrorKind::SchemaNotFound.into())
                };
                DecodedPackedFile::DB(DB::new(table_name, None, table_definition))
            }

            // TODO: Add Text files here.

            // For anything else, just return `Unknown`.
            _ => DecodedPackedFile::Unknown,
        };

        Ok(Self::new_from_decoded(&data, &path))
    }

    /// This function returns the current compression state of the provided `RawPackedFile`.
    pub fn get_compression_state(&self) -> bool {
        match self.data {
            PackedFileData::OnMemory(_, state, _) => state,
            PackedFileData::OnDisk(ref raw_on_disk) => raw_on_disk.get_compression_state(),
        }
    }

    /// This function returns the current encryption state of the provided `RawPackedFile`.
    pub fn get_encryption_state(&self) -> bool {
        match self.data {
            PackedFileData::OnMemory(_, _, state) => state.is_some(),
            PackedFileData::OnDisk(ref raw_on_disk) => raw_on_disk.get_encryption_state(),
        }
    }

    /// This function sets the path of the provided `RawPackedFile`.
    ///
    /// This can fail if you pass it an empty path, so make sure you check the result.
    ///
    /// ***WARNING***: DON'T USE THIS IF YOUR PACKEDFILE IS INSIDE A PACKFILE. USE THE `move_packedfile` FUNCTION INSTEAD.
    pub fn set_path(&mut self, path: &[String]) -> Result<()> {
        if path.is_empty() { return Err(ErrorKind::EmptyInput.into()) }
        self.path = path.to_vec();
        Ok(())
    }
*/
}
