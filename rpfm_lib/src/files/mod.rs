//---------------------------------------------------------------------------//
// Copyright (c) 2017-2022 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! This module contains the definition of RFile, the file abstraction used by this lib to decode/encode files.
//!
//! # Known file types
//!
//! | File Type         | Decoding Supported | Encoding Supported |
//! | ----------------- | ------------------ | ------------------ |
//! | [`AnimFragment`]  | Yes                | Yes                |
//! | [`AnimPack`]      | Yes                | Yes                |
//! | [`AnimsTable`]    | Yes                | Yes                |
//! | [`CAVP8`]         | Yes                | Yes                |
//! | [`DB`]            | Yes                | Yes                |
//! | [`ESF`]           | Limited            | Limited            |
//! | [`Image`]         | Yes                | Yes                |
//! | [`Loc`]           | Yes                | Yes                |
//! | [`MatchedCombat`] | Yes                | Yes                |
//! | [`Pack`]          | Yes                | Yes                |
//! | [`RigidModel`]    | No                 | No                 |
//! | [`Text`]          | Yes                | Yes                |
//! | [`UIC`]           | No                 | No                 |
//! | [`UnitVariant`]   | Yes                | Yes                |
//!
//! There is an additional type: [`Unknown`]. This type is used as a wildcard,
//! so you can get the raw data of any non-supported file type and manipulate it yourself in a safe way.
//!
//! For more information about specific file types, including their binary format spec, please
//! **check their respective documentation**.
//!
//! [`AnimFragment`]: crate::files::anim_fragment::AnimFragment
//! [`AnimPack`]: crate::files::animpack::AnimPack
//! [`AnimsTable`]: crate::files::anims_table::AnimsTable
//! [`CAVP8`]: crate::files::ca_vp8::CaVp8
//! [`DB`]: crate::files::db::DB
//! [`ESF`]: crate::files::esf::ESF
//! [`Image`]: crate::files::image::Image
//! [`Loc`]: crate::files::loc::Loc
//! [`MatchedCombat`]: crate::files::matched_combat::MatchedCombat
//! [`Pack`]: crate::files::pack::Pack
//! [`RigidModel`]: crate::files::rigidmodel::RigidModel
//! [`Text`]: crate::files::text::Text
//! [`UIC`]: crate::files::uic::UIC
//! [`UnitVariant`]: crate::files::unit_variant::UnitVariant
//! [`Unknown`]: crate::files::unknown::Unknown

#[cfg(feature = "integration_sqlite")] use r2d2::Pool;
#[cfg(feature = "integration_sqlite")] use r2d2_sqlite::SqliteConnectionManager;

use getset::*;
use rayon::prelude::*;
use regex::Regex;

use std::cmp::Ordering;
use std::collections::HashMap;
use std::fmt::Debug;
use std::fs::File;
use std::io::{BufReader, Cursor, Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};

use crate::binary::{ReadBytes, WriteBytes};
use crate::compression::Decompressible;
use crate::encryption::Decryptable;
use crate::error::{Result, RLibError};
use crate::games::pfh_version::PFHVersion;
use crate::schema::Schema;
use crate::utils::*;

use self::anim_fragment::AnimFragment;
use self::animpack::AnimPack;
use self::anims_table::AnimsTable;
use self::ca_vp8::CaVp8;
use self::db::DB;
use self::esf::ESF;
use self::image::Image;
use self::loc::Loc;
use self::matched_combat::MatchedCombat;
use self::pack::Pack;
use self::rigidmodel::RigidModel;
use self::text::Text;
use self::uic::UIC;
use self::unit_variant::UnitVariant;
use self::unknown::Unknown;

pub mod anim_fragment;
pub mod animpack;
pub mod anims_table;
pub mod ca_vp8;
pub mod db;
pub mod esf;
pub mod image;
pub mod loc;
pub mod matched_combat;
pub mod pack;
pub mod rigidmodel;
pub mod table;
pub mod text;
pub mod uic;
pub mod unit_variant;
pub mod unknown;

#[cfg(test)] mod rfile_test;

//---------------------------------------------------------------------------//
//                              Enum & Structs
//---------------------------------------------------------------------------//

/// This struct represents an individual file, including the metadata associated with it.
///
/// It can represent both, a file within a Pack (or anything implementing [Container] really), or a single
/// file on disk.
///
/// It supports Lazy-Loading to reduce RAM usage.
#[derive(Clone, Debug, PartialEq)]
pub struct RFile {

    /// Path of the file, either within a [`Container`] or in the FileSystem.
    ///
    /// It may be an empty string if the file exists only in memory.
    path: String,

    /// Last modified date of the file. Optional.
    timestamp: Option<u64>,

    /// The type of this file.
    file_type: FileType,

    /// Inner data of the file.
    ///
    /// Internal only. Users should use the [`RFile`] methods instead of using this directly.
    data: RFileInnerData,
}

/// This enum contains the data of each [`RFile`].
///
/// This is internal only.
#[derive(Clone, Debug, PartialEq)]
enum RFileInnerData {

    /// This variant represents a file whose data has been loaded to memory and decoded.
    Decoded(Box<RFileDecoded>),

    /// This variant represents a file whose data has been loaded to memory, but it hasn't been decoded.
    Cached(Vec<u8>),

    /// This variant represents a file whose data hasn't been loaded to memory yet.
    OnDisk(OnDisk)
}

/// This struct represents a file on disk, which data has not been loaded to memory yet.
///
/// This may be a file directly on disk, or one inside another file (like inside a [Container]).
///
/// This is internal only. Users should not use it directly.
#[derive(Clone, Debug, PartialEq, Getters)]
struct OnDisk {

    /// Path of the file on disk where the data is.
    ///
    /// This may be a singular file or a file containing it
    path: String,

    /// Last modified date of the file that contains the data.
    ///
    /// This is used to both, get the last modified data into the file's metadata
    /// and to check if the file has been manipulated since we created the OnDisk of it.
    timestamp: u64,

    /// Offset of the start of the file's data.
    ///
    /// `0` if the whole file is the data we want.
    start: u64,

    /// Size in bytes of the file's data.
    size: u64,

    /// Is the data compressed?.
    is_compressed: bool,

    /// Is the data encrypted? And if so, with which format?.
    is_encrypted: Option<PFHVersion>,
}

/// This enum allow us to store any kind of decoded file type on a common place.
#[derive(Clone, Debug, PartialEq)]
pub enum RFileDecoded {
    Anim(Unknown),
    AnimFragment(AnimFragment),
    AnimPack(AnimPack),
    AnimsTable(AnimsTable),
    CaVp8(CaVp8),
    CEO(ESF),
    DB(DB),
    ESF(ESF),
    GroupFormations(Unknown),
    Image(Image),
    Loc(Loc),
    MatchedCombat(MatchedCombat),
    Pack(Pack),
    RigidModel(RigidModel),
    Save(ESF),
    Text(Text),
    UIC(UIC),
    UnitVariant(UnitVariant),
    Unknown(Unknown),
}

/// This enum specifies the known types of files we can find in a Total War game.
///
/// This list is not exhaustive and it may get bigger in the future as more files are added.
///
/// For each file info, please check their dedicated submodule if exists.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FileType {
    Anim,
    AnimFragment,
    AnimPack,
    AnimsTable,
    CaVp8,
    CEO,
    DB,
    ESF,
    GroupFormations,
    Image,
    Loc,
    MatchedCombat,
    Pack,
    RigidModel,
    Save,
    Text,
    UIC,
    UnitVariant,
    Unknown,
}

/// This enum represents a ***Path*** inside a [Container].
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ContainerPath {

    /// This variant represents the path of a single file.
    File(String),

    /// This variant represents the path of a single folder.
    ///
    /// If this is empty, it represents the root of the container.
    Folder(String),
}

/// This is a generic struct to easily pass additional data to a [Decodeable::decode] method.
///
/// To know what you need to provide to each file type, please check their documentation.
#[derive(Clone, Default, Getters, Setters)]
#[getset(get = "pub", set = "pub")]
pub struct DecodeableExtraData<'a> {

    //-----------------------//
    // Configuration toggles //
    //-----------------------//

    /// For [Container] implementors, if they should use LazyLoading for their files.
    lazy_load: bool,

    /// If the data was encrypted (all data that reach the decode functions should be already decrypted).
    is_encrypted: bool,

    /// If the decoder should return incomplete data on failure (only for tables).
    return_incomplete: bool,

    /// Schema for the decoder to use. Mainly for tables.
    schema: Option<&'a Schema>,

    //----------------------------//
    // OnDisk-related config data //
    //----------------------------//

    /// Path of a file on disk, if any.
    disk_file_path: Option<&'a str>,

    /// Offset of a file on disk where the data we're interested on starts.
    disk_file_offset: u64,

    /// Timestamp of a file on disk.
    timestamp: u64,

    //----------------------------//
    // Table-related config data  //
    //----------------------------//

    /// Name of the folder that contains a table fragment.
    table_name: Option<&'a str>,

    /// SQLite Database Pool. For allowing connections to the database.
    #[cfg(feature = "integration_sqlite")]
    pool: Option<&'a Pool<SqliteConnectionManager>>,

    //------------------------------//
    // General-purpouse config data //
    //------------------------------//

    /// Name of the file we're trying to decode.
    file_name: Option<&'a str>,

    /// Size of the data in a file, either on disk or in memory.
    data_size: u64,
}

/// This is a generic struct to easily pass additional data to a [Encodeable::encode] method.
///
/// To know what you need to provide to each file type, please check their documentation.
#[derive(Clone, Default, Getters, Setters)]
#[getset(get = "pub", set = "pub")]
pub struct EncodeableExtraData<'a> {

    //-----------------------//
    // Configuration toggles //
    //-----------------------//

    /// If we're running the encode method on test mode.
    test_mode: bool,

    /// Only for tables. If we should add a GUID to its header or not.
    table_has_guid: bool,

    /// Only for tables. If we should regenerate the GUID of the table (if it even has one) or keep the current one.
    regenerate_table_guid: bool,

    //-----------------------//
    // Optional config data  //
    //-----------------------//

    /// Path of 7z.exe. Used for compressing.
    sevenzip_path: Option<&'a Path>,

    /// SQLite Database Pool. For allowing connections to the database.
    #[cfg(feature = "integration_sqlite")]
    pool: Option<&'a Pool<SqliteConnectionManager>>,
}

//---------------------------------------------------------------------------//
//                           Trait Definitions
//---------------------------------------------------------------------------//

/// A generic trait to implement decoding logic from anything implementing [ReadBytes](crate::binary::ReadBytes)
/// into structured types.
pub trait Decodeable: Send + Sync {

    /// This method provides a generic and expandable way to decode anything implementing [ReadBytes](crate::binary::ReadBytes)
    /// into the implementor's structure.
    ///
    /// The parameter `extra_data` contains arguments that can be used to provide additional data needed for the decoding process.
    fn decode<R: ReadBytes>(data: &mut R, extra_data: &Option<DecodeableExtraData>) -> Result<Self> where Self: Sized;
}

/// A generic trait to implement encoding logic from structured types into anything implementing [WriteBytes](crate::binary::WriteBytes).
pub trait Encodeable: Send + Sync {

    /// This method provides a generic and expandable way to encode any implementor's structure into anything
    /// implementing [WriteBytes](crate::binary::WriteBytes)
    ///
    /// The parameter `extra_data` contains arguments that can be used to provide additional data needed for the encoding process.
    fn encode<W: WriteBytes>(&mut self, buffer: &mut W, extra_data: &Option<EncodeableExtraData>) -> Result<()>;
}

/// An interface to easily work with container-like files.
///
/// This trait allow any implementor to provide methods to manipulate them like [RFile] Containers.
pub trait Container {

    /// This method allow us to insert an [RFile] within a Container, replacing any old [RFile]
    /// with the same path, in case it already existed one.
    ///
    /// Returns the [ContainerPath] of the inserted [RFile].
    fn insert(&mut self, file: RFile) -> Result<ContainerPath> {
        let path = file.path_in_container();
        let path_raw = file.path_in_container_raw();
        self.files_mut().insert(path_raw.to_owned(), file);
        Ok(path)
    }

    /// This method allow us to remove any [RFile] matching the provided [ContainerPath] from a Container.
    ///
    /// An special situation is passing `ContainerPath::Folder("")`. This represents the root of the container,
    /// meaning passing this will delete all the RFiles within the container.
    ///
    /// Returns the list of removed [ContainerPath], always using the [File](ContainerPath::File) variant.
    fn remove(&mut self, path: &ContainerPath) -> Vec<ContainerPath> {
        match path {
            ContainerPath::File(path) => {
                self.files_mut().remove(path);
                return vec![ContainerPath::File(path.to_owned())];
            },
            ContainerPath::Folder(path) => {

                // If the path is empty, we mean the root of the container, including everything on it.
                if path.is_empty() {
                    self.files_mut().clear();
                    return vec![ContainerPath::Folder(path.to_string())];
                }

                // Otherwise, it's a normal folder.
                else {
                    let paths_to_remove = self.files().par_iter()
                        .filter_map(|(key, _)| if key.starts_with(path) { Some(key.to_owned()) } else { None }).collect::<Vec<String>>();

                    paths_to_remove.iter().for_each(|path| {
                        self.files_mut().remove(path);
                    });
                    return paths_to_remove.par_iter().map(|path| ContainerPath::File(path.to_string())).collect();
                }
            }
        }
    }

    /// This method returns the path on disk of the provided Container.
    ///
    /// Implementors should return `""` if the provided Container is not from a disk file.
    fn disk_file_path(&self) -> &str;

    /// This method returns the offset of the data of this Container in its disk file.
    ///
    /// Implementors should return `0` if the provided Container is not within another Container.
    fn disk_file_offset(&self) -> u64;

    /// This method returns a reference to the RFiles inside the provided Container.
    fn files(&self) -> &HashMap<String, RFile>;

    /// This method returns a mutable reference to the RFiles inside the provided Container.
    fn files_mut(&mut self) -> &mut HashMap<String, RFile>;

    /// This method returns a reference to the RFiles inside the provided Container that match the provided [ContainerPath].
    ///
    /// An special situation is passing `ContainerPath::Folder("")`. This represents the root of the container,
    /// meaning passing this will return all RFiles within the container.
    fn files_by_path(&self, path: &ContainerPath) -> Vec<&RFile> {
        match path {
            ContainerPath::File(path) => {
                match self.files().get(path) {
                    Some(file) => vec![file],
                    None => vec![],
                }
            },
            ContainerPath::Folder(path) => {

                // If the path is empty, get everything.
                if path.is_empty() {
                    self.files().values().collect()
                }

                // Otherwise, only get the files under our folder.
                else {
                    self.files().par_iter()
                        .filter_map(|(key, file)|
                            if key.starts_with(path) { Some(file) } else { None }
                        ).collect::<Vec<&RFile>>()
                }
            },
        }
    }

    /// This method returns the list of [ContainerPath] corresponding to RFiles within the provided Container.
    fn paths(&self) -> Vec<ContainerPath> {
        self.files().par_iter().map(|(path, _)| ContainerPath::File(path.to_owned())).collect()
    }

    /// This method returns the list of paths (as [&str]) corresponding to RFiles within the provided Container.
    fn paths_raw(&self) -> Vec<&str> {
        self.files()
            .par_iter()
            .map(|(path, _)| &**path)
            .collect()
    }

    /// This method returns the `Last modified date` stored on the provided Container, in seconds.
    ///
    /// A default implementation that returns `0` is provided for Container types that don't support internal timestamps.
    ///
    /// Implementors should return `0` if the Container doesn't have a file on disk yet.
    fn internal_timestamp(&self) -> u64 {
       0
    }

    /// This method returns the `Last modified date` the filesystem reports for the container file, in seconds.
    ///
    /// Implementors should return `0` if the Container doesn't have a file on disk yet.
    fn local_timestamp(&self) -> u64;
}

// TODO: Implement "possible types" logic, to have some flexibility when opening files.

//----------------------------------------------------------------//
//                        Implementations
//----------------------------------------------------------------//

impl RFile {

    /// This function creates a RFile from a lazy-loaded file inside a Container.
    ///
    /// About the parameters:
    ///
    /// TBD
    pub fn new_from_container<C: Container>(
        container: &C,
        size: u64,
        is_compressed: bool,
        is_encrypted: Option<PFHVersion>,
        data_pos: u64,
        file_timestamp: u64,
        path_in_container: &str,
    ) -> Result<Self> {
        let on_disk = OnDisk {
            path: container.disk_file_path().to_owned(),
            timestamp: container.local_timestamp(),
            start: container.disk_file_offset() + data_pos,
            size,
            is_compressed,
            is_encrypted,
        };

        let mut rfile = Self {
            path: path_in_container.to_owned(),
            timestamp: if file_timestamp == 0 { None } else { Some(file_timestamp) },
            file_type: FileType::Unknown,
            data: RFileInnerData::OnDisk(on_disk)
        };

        rfile.guess_file_type()?;
        Ok(rfile)
    }

    /// This function creates a RFile from an disk file.
    ///
    /// This may fail if the file doesn't exist or errors out when trying to be read for metadata.
    pub fn new_from_file(path: &str) -> Result<Self> {
        let path_checked = PathBuf::from(path);
        if !path_checked.is_file() {
            return Err(RLibError::FileNotFound(path.to_owned()));
        }

        let mut file = File::open(path)?;
        let on_disk = OnDisk {
            path: path.to_owned(),
            timestamp: last_modified_time_from_file(&file)?,
            start: 0,
            size: file.len()?,
            is_compressed: false,
            is_encrypted: None,
        };


        let mut rfile = Self {
            path: path.to_owned(),
            timestamp: Some(on_disk.timestamp),
            file_type: FileType::Unknown,
            data: RFileInnerData::OnDisk(on_disk)
        };

        rfile.guess_file_type()?;
        Ok(rfile)
    }

    /// This function creates a RFile from raw data on memory.
    pub fn new_from_vec(data: &[u8], file_type: FileType, timestamp: u64, path: &str) -> Result<Self> {
        let mut rfile = Self {
            path: path.to_owned(),
            timestamp: if timestamp == 0 { None } else { Some(timestamp) },
            file_type,
            data: RFileInnerData::Cached(data.to_vec())
        };

        rfile.guess_file_type()?;
        Ok(rfile)
    }

    /// This function creates a RFile from an RFileDecoded on memory.
    pub fn new_from_decoded(data: &RFileDecoded, file_type: FileType, timestamp: u64, path: &str) -> Result<Self> {
        let mut rfile = Self {
            path: path.to_owned(),
            timestamp: if timestamp == 0 { None } else { Some(timestamp) },
            file_type,
            data: RFileInnerData::Decoded(Box::new(data.clone()))
        };

        rfile.guess_file_type()?;
        Ok(rfile)
    }

    /// This function decodes an RFile from binary data, optionally caching and returning the decoded RFile.
    ///
    /// About the arguments:
    ///
    /// - `extra_data`: any data needed to decode specific file types. Check each file type for info about what do each file type need.
    /// - `keep_in_cache`: if true, the data will be cached on memory.
    /// - `return_data`: if true, the decoded data will be returned.
    ///
    /// NOTE: Passing `keep_in_cache` and `return_data` at false causes this function to decode the RFile and
    /// immediately drop the resulting data.
    pub fn decode(&mut self, extra_data: &Option<DecodeableExtraData>, keep_in_cache: bool, return_data: bool) -> Result<Option<RFileDecoded>> {
        let mut already_decoded = false;
        let decoded = match &self.data {

            // If the data is already decoded, just return a copy of it.
            RFileInnerData::Decoded(data) => {
                already_decoded = true;
                *data.clone()
            },

            // If the data is on memory but not yet decoded, decode it.
            RFileInnerData::Cached(data) => {

                // Copy the provided extra data (if any), then replace the file-specific stuff.
                let mut extra_data = match extra_data {
                    Some(extra_data) => extra_data.clone(),
                    None => DecodeableExtraData::default(),
                };
                extra_data.file_name = Some(self.path.rsplit_terminator('/').collect::<Vec<_>>()[0]);
                extra_data.data_size = data.len() as u64;

                // Some types require extra data specific for them to be added to the extra data before decoding.
                let mut data = Cursor::new(data);
                match self.file_type {
                    FileType::Anim => RFileDecoded::Anim(Unknown::decode(&mut data, &Some(extra_data))?),
                    FileType::AnimFragment => RFileDecoded::AnimFragment(AnimFragment::decode(&mut data, &Some(extra_data))?),
                    FileType::AnimPack => RFileDecoded::AnimPack(AnimPack::decode(&mut data, &Some(extra_data))?),
                    FileType::AnimsTable => RFileDecoded::AnimsTable(AnimsTable::decode(&mut data, &Some(extra_data))?),
                    FileType::CaVp8 => RFileDecoded::CaVp8(CaVp8::decode(&mut data, &Some(extra_data))?),
                    FileType::CEO => RFileDecoded::CEO(ESF::decode(&mut data, &Some(extra_data))?),
                    FileType::DB => {

                        // This one is tricky. On one side, DB from disk are expected to, either provide a table_name
                        // or be inside a folder with the table_name. On the other side, if it's in a container it has
                        // to be in a folder with a table_name inside "db". So, if we don't receive a name, we "guess" it.
                        if extra_data.table_name.is_none() {
                            let split_path = self.path.rsplitn(3, '/').collect::<Vec<_>>();
                            if split_path.len() < 3 || split_path[2].to_lowercase() != "db" {
                                return Err(RLibError::DecodingDBNotADBTable);
                            }
                            extra_data.table_name = Some(split_path[1]);
                        }
                        RFileDecoded::DB(DB::decode(&mut data, &Some(extra_data))?)
                    },
                    FileType::ESF => RFileDecoded::ESF(ESF::decode(&mut data, &Some(extra_data))?),
                    FileType::GroupFormations => RFileDecoded::GroupFormations(Unknown::decode(&mut data, &Some(extra_data))?),
                    FileType::Image => RFileDecoded::Image(Image::decode(&mut data, &Some(extra_data))?),
                    FileType::Loc => RFileDecoded::Loc(Loc::decode(&mut data, &Some(extra_data))?),
                    FileType::MatchedCombat => RFileDecoded::MatchedCombat(MatchedCombat::decode(&mut data, &Some(extra_data))?),
                    FileType::Pack => RFileDecoded::Pack(Pack::decode(&mut data, &Some(extra_data))?),
                    FileType::RigidModel => RFileDecoded::RigidModel(RigidModel::decode(&mut data, &Some(extra_data))?),
                    FileType::Save => RFileDecoded::Save(ESF::decode(&mut data, &Some(extra_data))?),
                    FileType::Text => RFileDecoded::Text(Text::decode(&mut data, &Some(extra_data))?),
                    FileType::UIC => RFileDecoded::UIC(UIC::decode(&mut data, &Some(extra_data))?),
                    FileType::UnitVariant => RFileDecoded::UnitVariant(UnitVariant::decode(&mut data, &Some(extra_data))?),
                    FileType::Unknown => RFileDecoded::Unknown(Unknown::decode(&mut data, &Some(extra_data))?),
                }
            },

            // If the data is not yet in memory, it depends:
            // - If it's something we can lazy-load and we want to, decode it directly from disk.
            // - If it's not, load it to memory and decode it from there.
            RFileInnerData::OnDisk(data) => {
                match self.file_type {
                    FileType::Anim |
                    FileType::AnimFragment |
                    FileType::AnimsTable |
                    FileType::CaVp8 |
                    FileType::CEO |
                    FileType::DB |
                    FileType::ESF |
                    FileType::GroupFormations |
                    FileType::Image |
                    FileType::Loc |
                    FileType::MatchedCombat |
                    FileType::RigidModel |
                    FileType::Save |
                    FileType::Text |
                    FileType::UIC |
                    FileType::UnitVariant |
                    FileType::Unknown => {

                        // Copy the provided extra data (if any), then replace the file-specific stuff.
                        let raw_data = data.read(data.is_compressed, data.is_encrypted)?;
                        let mut extra_data = match extra_data {
                            Some(extra_data) => extra_data.clone(),
                            None => DecodeableExtraData::default(),
                        };
                        extra_data.file_name = Some(self.path.rsplit_terminator('/').collect::<Vec<_>>()[0]);
                        extra_data.data_size = raw_data.len() as u64;

                        // These are the easy types: just load the data to memory, and decode.
                        let mut data = Cursor::new(raw_data);
                        match self.file_type {
                            FileType::Anim => RFileDecoded::Anim(Unknown::decode(&mut data, &Some(extra_data))?),
                            FileType::AnimFragment => RFileDecoded::AnimFragment(AnimFragment::decode(&mut data, &Some(extra_data))?),
                            FileType::AnimsTable => RFileDecoded::AnimsTable(AnimsTable::decode(&mut data, &Some(extra_data))?),
                            FileType::CaVp8 => RFileDecoded::CaVp8(CaVp8::decode(&mut data, &Some(extra_data))?),
                            FileType::CEO => RFileDecoded::CEO(ESF::decode(&mut data, &Some(extra_data))?),
                            FileType::DB => RFileDecoded::DB(DB::decode(&mut data, &Some(extra_data))?),
                            FileType::ESF => RFileDecoded::ESF(ESF::decode(&mut data, &Some(extra_data))?),
                            FileType::GroupFormations => RFileDecoded::GroupFormations(Unknown::decode(&mut data, &Some(extra_data))?),
                            FileType::Image => RFileDecoded::Image(Image::decode(&mut data, &Some(extra_data))?),
                            FileType::Loc => RFileDecoded::Loc(Loc::decode(&mut data, &Some(extra_data))?),
                            FileType::MatchedCombat => RFileDecoded::MatchedCombat(MatchedCombat::decode(&mut data, &Some(extra_data))?),
                            FileType::RigidModel => RFileDecoded::RigidModel(RigidModel::decode(&mut data, &Some(extra_data))?),
                            FileType::Save => RFileDecoded::Save(ESF::decode(&mut data, &Some(extra_data))?),
                            FileType::Text => RFileDecoded::Text(Text::decode(&mut data, &Some(extra_data))?),
                            FileType::UIC => RFileDecoded::UIC(UIC::decode(&mut data, &Some(extra_data))?),
                            FileType::UnitVariant => RFileDecoded::UnitVariant(UnitVariant::decode(&mut data, &Some(extra_data))?),
                            FileType::Unknown => RFileDecoded::Unknown(Unknown::decode(&mut data, &Some(extra_data))?),

                            FileType::AnimPack |
                            FileType::Pack => unreachable!()
                        }
                    },

                    FileType::AnimPack |
                    FileType::Pack => {

                        // These two require extra data and may require lazy-loading.
                        // For lazy-loading, disable it if we detect encryption.
                        let mut extra_data = match extra_data {
                            Some(extra_data) => extra_data.clone(),
                            None => DecodeableExtraData::default(),
                        };
                        extra_data.lazy_load = !extra_data.is_encrypted && extra_data.lazy_load;
                        extra_data.file_name = Some(self.path.rsplit_terminator('/').collect::<Vec<_>>()[0]);
                        extra_data.data_size = data.size as u64;

                        // If we're lazy-loading we also need extra data to read from disk on-demand.
                        if extra_data.lazy_load {
                            extra_data.disk_file_path = Some(&data.path);
                            extra_data.disk_file_offset = data.start;
                            extra_data.timestamp = last_modified_time_from_file(&File::open(&data.path)?)?;

                            let mut data = data.read_lazily()?;
                            match self.file_type {
                                FileType::AnimPack => RFileDecoded::AnimPack(AnimPack::decode(&mut data, &Some(extra_data))?),
                                FileType::Pack => RFileDecoded::Pack(Pack::decode(&mut data, &Some(extra_data))?),
                                _ => unreachable!()
                            }
                        }

                        // If we're not using lazy-loading, just use the normal method.
                        else {
                            let raw_data = data.read(data.is_compressed, data.is_encrypted)?;
                            let mut data = Cursor::new(raw_data);

                            match self.file_type {
                                FileType::AnimPack => RFileDecoded::AnimPack(AnimPack::decode(&mut data, &Some(extra_data))?),
                                FileType::Pack => RFileDecoded::Pack(Pack::decode(&mut data, &Some(extra_data))?),
                                _ => unreachable!()
                            }
                        }
                    }
                }
            },
        };

        if !already_decoded && keep_in_cache {
            self.data = RFileInnerData::Decoded(Box::new(decoded.clone()));
        }

        if return_data {
            Ok(Some(decoded))
        } else {
            Ok(None)
        }
    }

    /// This function encodes an RFile to binary, optionally caching and returning the data.
    ///
    /// About the arguments:
    /// - `extra_data`: any data needed to encode specific file types. Check each file type for info about what do each file type need.
    /// - `move_decoded_to_cache`: if true, the decoded data will be dropped in favor of undecoded cached data.
    /// - `move_undecoded_to_cache`: if true, the data will be cached on memory.
    /// - `return_data`: if true, the data will be returned.
    pub fn encode(&mut self, extra_data: &Option<EncodeableExtraData>, move_decoded_to_cache: bool, move_undecoded_to_cache: bool, return_data: bool) -> Result<Option<Vec<u8>>> {
        let mut previously_decoded = false;
        let mut already_encoded = false;
        let mut previously_undecoded = false;

        let encoded = match &mut self.data {
            RFileInnerData::Decoded(data) => {
                previously_decoded = true;
                let mut buffer = vec![];
                match &mut **data {
                    RFileDecoded::Anim(data) => data.encode(&mut buffer, extra_data)?,
                    RFileDecoded::AnimFragment(data) => data.encode(&mut buffer, extra_data)?,
                    RFileDecoded::AnimPack(data) => data.encode(&mut buffer, extra_data)?,
                    RFileDecoded::AnimsTable(data) => data.encode(&mut buffer, extra_data)?,
                    RFileDecoded::CaVp8(data) => data.encode(&mut buffer, extra_data)?,
                    RFileDecoded::CEO(data) => data.encode(&mut buffer, extra_data)?,
                    RFileDecoded::DB(data) => data.encode(&mut buffer, extra_data)?,
                    RFileDecoded::ESF(data) => data.encode(&mut buffer, extra_data)?,
                    RFileDecoded::GroupFormations(data) => data.encode(&mut buffer, extra_data)?,
                    RFileDecoded::Image(data) => data.encode(&mut buffer, extra_data)?,
                    RFileDecoded::Loc(data) => data.encode(&mut buffer, extra_data)?,
                    RFileDecoded::MatchedCombat(data) => data.encode(&mut buffer, extra_data)?,
                    RFileDecoded::Pack(data) => data.encode(&mut buffer, extra_data)?,
                    RFileDecoded::RigidModel(data) => data.encode(&mut buffer, extra_data)?,
                    RFileDecoded::Save(data) => data.encode(&mut buffer, extra_data)?,
                    RFileDecoded::Text(data) => data.encode(&mut buffer, extra_data)?,
                    RFileDecoded::UIC(data) => data.encode(&mut buffer, extra_data)?,
                    RFileDecoded::UnitVariant(data) => data.encode(&mut buffer, extra_data)?,
                    RFileDecoded::Unknown(data) => data.encode(&mut buffer, extra_data)?,
                }

                buffer
            },
            RFileInnerData::Cached(data) => {
                already_encoded = true;
                data.to_vec()
            },
            RFileInnerData::OnDisk(data) => {
                previously_undecoded = true;
                data.read(data.is_compressed, data.is_encrypted)?
            },
        };

        // If the RFile was already decoded.
        if previously_decoded {
            if move_decoded_to_cache {
                if return_data {
                    self.data = RFileInnerData::Cached(encoded.to_vec());
                    Ok(Some(encoded))
                } else {
                    self.data = RFileInnerData::Cached(encoded);
                    Ok(None)
                }
            } else if return_data {
                Ok(Some(encoded))
            } else {
                Ok(None)
            }
        }

        // If the RFile was not even loaded.
        else if previously_undecoded {
            if move_undecoded_to_cache {
                if return_data {
                    self.data = RFileInnerData::Cached(encoded.to_vec());
                    Ok(Some(encoded))
                } else {
                    self.data = RFileInnerData::Cached(encoded);
                    Ok(None)
                }
            } else if return_data {
                Ok(Some(encoded))
            } else {
                Ok(None)
            }
        }

        // If the RFile was already encoded and loaded.
        else if already_encoded && return_data {
            Ok(Some(encoded))
        } else {
            Ok(None)
        }
    }

    /// This function loads the data of an RFile to memory if it's not yet loaded.
    ///
    /// If it has already been loaded either to cache, or for decoding, this does nothing.
    pub fn load(&mut self) -> Result<()> {
       let loaded = match &self.data {
            RFileInnerData::Decoded(_) |
            RFileInnerData::Cached(_) => {
                return Ok(())
            },
            RFileInnerData::OnDisk(data) => {
                data.read(data.is_compressed, data.is_encrypted)?
            },
        };

        self.data = RFileInnerData::Cached(loaded);
        Ok(())
    }

    /// This function returns a copy of the `Last modified date` of this RFile, if any.
    pub fn timestamp(&self) -> Option<u64> {
        self.timestamp.clone()
    }

    /// This function returns a copy of the FileType of this RFile.
    pub fn file_type(&self) -> FileType {
        self.file_type.clone()
    }

    /// This function returns the [ContainerPath] corresponding to this file.
    pub fn path_in_container(&self) -> ContainerPath {
        ContainerPath::File(self.path.to_owned())
    }
    /// This function returns the [ContainerPath] corresponding to this file as an [&str].
    pub fn path_in_container_raw(&self) -> &str {
        &self.path
    }

    /// This function returns if the RFile can be compressed or not.
    pub fn is_compressible(&self) -> bool {
        match self.file_type {
            FileType::DB |
            FileType::Loc => false,
            _ => true
        }
    }

    /// This function guesses the [`FileType`] of the provided RFile and stores it on it for later queries.
    ///
    /// The way it works is: first it tries to guess it by extension (fast), then by full path (not as fast), then by data (slow and it may fail on lazy-loaded files).
    ///
    /// This may fail for some files, so if you doubt set the type manually.
    fn guess_file_type(&mut self) -> Result<()> {

        // First, try with extensions.
        let path = self.path.to_lowercase();

        // TODO: Add autodetection to these, somehow
        //--Anim,
        //--GroupFormations,
        //--UIC,

        if path.ends_with(pack::EXTENSION) {
            self.file_type = FileType::Pack;
        }

        else if path.ends_with(loc::EXTENSION) {
            self.file_type = FileType::Loc;
        }

        else if path.ends_with(rigidmodel::EXTENSION) {
            self.file_type = FileType::RigidModel
        }

        else if path.ends_with(animpack::EXTENSION) {
            self.file_type =  FileType::AnimPack
        }

        else if path.ends_with(ca_vp8::EXTENSION) {
            self.file_type =  FileType::CaVp8;
        }

        else if image::EXTENSIONS.iter().any(|x| path.ends_with(x)) {
            self.file_type =  FileType::Image;
        }

        else if text::EXTENSIONS.iter().any(|(x, _)| path.ends_with(x)) {
            self.file_type = FileType::Text;
        }

        else if path.ends_with(unit_variant::EXTENSION) {
            self.file_type = FileType::UnitVariant
        }

        else if path.ends_with(esf::EXTENSION_CEO) {
            self.file_type = FileType::CEO;
        }

        else if path.ends_with(esf::EXTENSION_ESF) {
            self.file_type = FileType::ESF;
        }

        else if path.ends_with(esf::EXTENSION_SAVE) {
            self.file_type = FileType::Save;
        }

        // If that failed, try types that need to be in a specific path.
        else if path.starts_with(&matched_combat::BASE_PATH) && path.ends_with(matched_combat::EXTENSION) {
            self.file_type = FileType::MatchedCombat;
        }

        else if path.starts_with(&anims_table::BASE_PATH) && path.ends_with(anims_table::EXTENSION) {
            self.file_type =  FileType::AnimsTable;
        }

        else if path.starts_with(&anim_fragment::BASE_PATH) && anim_fragment::EXTENSIONS.iter().any(|x| path.ends_with(*x)) {
            self.file_type = FileType::AnimFragment;
        }

        // If that failed, check if it's in a folder which is known to only have specific files.
        else if Regex::new(r"db/[^/]+_tables/[^/]+$").unwrap().is_match(&path) {
            self.file_type = FileType::DB;
        }

        // If we reach this... we're clueless. Leave it unknown.
        else {
            self.file_type = FileType::Unknown;
        }

        Ok(())
    }
}

impl OnDisk {

    /// This function tries to read and return the raw data of an RFile.
    ///
    /// This returns the data uncompressed and unencrypted.
    fn read(&self, decompress: bool, decrypt: Option<PFHVersion>) -> Result<Vec<u8>> {

        // Date check, to ensure the source file or container hasn't been modified since we got the indexes to read it.
        let mut file = BufReader::new(File::open(&self.path)?);
        let timestamp = last_modified_time_from_file(file.get_ref())?;
        if timestamp != self.timestamp {
            return Err(RLibError::FileSourceChanged.into());
        }

        // Read the data from disk.
        let mut data = vec![0; self.size as usize];
        file.seek(SeekFrom::Start(self.start))?;
        file.read_exact(&mut data)?;

        // If the data is encrypted, decrypt it.
        if decrypt.is_some() {
            data = Cursor::new(data).decrypt(false)?;
        }

        // If the data is compressed. decompress it.
        if decompress {
            data = data.as_slice().decompress()?;
        }

        Ok(data)
    }

    /// This function tries to read and return the raw data of an RFile.
    ///
    /// This returns the data uncompressed and unencrypted.
    fn read_lazily(&self) -> Result<BufReader<File>> {

        // Date check, to ensure the source file or container hasn't been modified since we got the indexes to read it.
        let mut file = BufReader::new(File::open(&self.path)?);
        let timestamp = last_modified_time_from_file(file.get_ref())?;
        if timestamp != self.timestamp {
            return Err(RLibError::FileSourceChanged.into());
        }

        file.seek(SeekFrom::Start(self.start))?;
        Ok(file)
    }
}

impl ContainerPath {

    /// This function removes collided items from the provided list of `ContainerPath`.
    ///
    /// This means, if you have a file and a folder containing the file, it removes the file.
    pub fn dedup(paths: &[Self]) -> Vec<Self> {

        // As this operation can get very expensive very fast, we first check if we have a path containing the root of the container.
        let root = ContainerPath::Folder("".to_string());
        if paths.contains(&root) {
            return vec![root; 1];
        }

        // If we don't have the root of the container, second optimization: check if we have at least one folder.
        // If not, we just need to dedup the file list.
        if paths.par_iter().any(|item| matches!(item, ContainerPath::Folder(_))) {
            let mut paths = paths.to_vec();
            paths.sort();
            paths.dedup();
            return paths;
        }

        // If we reached this point, we have a mix of files and folders, or only folders.
        // In any case, we need to filter them, then dedup the resultant paths.
        let items_to_remove = paths.par_iter().filter(|item_type_to_add| {
            match item_type_to_add {

                // If it's a file, we have to check if there is a folder containing it.
                ContainerPath::File(ref path_to_add) => {
                    paths.par_iter().filter(|x| {
                        !matches!(x, ContainerPath::File(_))
                    }).any(|item_type| {

                        // If the other one is a folder that contains it, dont add it.
                        if let ContainerPath::Folder(ref path) = item_type {
                            path_to_add.starts_with(path)
                        } else { false }
                    })
                }

                // If it's a folder, we have to check if there is already another folder containing it.
                ContainerPath::Folder(ref path_to_add) => {
                    paths.par_iter().filter(|x| {
                        !matches!(x, ContainerPath::File(_))
                    }).any(|item_type| {

                        // If the other one is a folder that contains it, dont add it.
                        if let ContainerPath::Folder(ref path) = item_type {
                            path_to_add.starts_with(path) && path_to_add.len() > path.len()
                        } else { false }
                    })
                }
            }
        }).cloned().collect::<Vec<Self>>();

        let mut paths = paths.to_vec();
        paths.retain(|x| items_to_remove.contains(x));
        paths.sort();
        paths.dedup();
        paths
    }
}

impl Ord for ContainerPath {
    fn cmp(&self, other: &Self) -> Ordering {
        match self {
            ContainerPath::File(a) => match other {
                ContainerPath::File(b) => if a == b {
                    Ordering::Equal
                } else if a > b {
                    Ordering::Greater
                } else {
                    Ordering::Less
                },
                ContainerPath::Folder(_) => Ordering::Less,
            }
            ContainerPath::Folder(a) => match other {
                ContainerPath::File(_) => Ordering::Greater,
                ContainerPath::Folder(b) => if a == b {
                    Ordering::Equal
                } else if a > b {
                    Ordering::Greater
                } else {
                    Ordering::Less
                },
            }
        }
    }
}

impl PartialOrd for ContainerPath {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/*
//----------------------------------------------------------------//
// Implementations for `DecodedPackedFile`.
//----------------------------------------------------------------//

/// Implementation of `DecodedPackedFile`.
impl DecodedPackedFile {

    /// This function decodes a `RawPackedFile` into a `DecodedPackedFile`, returning it.
    pub fn decode(raw_packed_file: &mut RawPackedFile) -> Result<Self> {
        match PackedFileType::get_packed_file_type(raw_packed_file, true) {

            PackedFileType::AnimFragment => {
                let schema = SCHEMA.read().unwrap();
                match schema.deref() {
                    Some(schema) => {
                        let data = raw_packed_file.get_data_and_keep_it()?;
                        let packed_file = AnimFragment::read(&data, schema, false)?;
                        Ok(DecodedPackedFile::AnimFragment(packed_file))
                    }
                    None => Err(ErrorKind::SchemaNotFound.into()),
                }
            }

            PackedFileType::AnimPack => {
                let data = raw_packed_file.get_data()?;
                let packed_file = AnimPack::read(&data)?;
                Ok(DecodedPackedFile::AnimPack(packed_file))
            }

            PackedFileType::AnimTable => {
                let schema = SCHEMA.read().unwrap();
                match schema.deref() {
                    Some(schema) => {
                        let data = raw_packed_file.get_data_and_keep_it()?;
                        let packed_file = AnimTable::read(&data, schema, false)?;
                        Ok(DecodedPackedFile::AnimTable(packed_file))
                    }
                    None => Err(ErrorKind::SchemaNotFound.into()),
                }
            }

            PackedFileType::CaVp8 => {
                let data = raw_packed_file.get_data()?;
                let packed_file = CaVp8::read(data)?;
                Ok(DecodedPackedFile::CaVp8(packed_file))
            }

            PackedFileType::ESF => {
                let data = raw_packed_file.get_data()?;
                let packed_file = ESF::read(&data)?;
                Ok(DecodedPackedFile::ESF(packed_file))
            }

            PackedFileType::DB => {
                let schema = SCHEMA.read().unwrap();
                match schema.deref() {
                    Some(schema) => {
                        let data = raw_packed_file.get_data_and_keep_it()?;
                        let name = raw_packed_file.get_path().get(1).ok_or_else(|| Error::from(ErrorKind::DBTableIsNotADBTable))?;
                        let packed_file = DB::read(&data, name, schema, false)?;
                        Ok(DecodedPackedFile::DB(packed_file))
                    }
                    None => Err(ErrorKind::SchemaNotFound.into()),
                }
            }

            PackedFileType::Image => {
                let data = raw_packed_file.get_data_and_keep_it()?;
                let packed_file = Image::read(&data)?;
                Ok(DecodedPackedFile::Image(packed_file))
            }

            PackedFileType::Loc => {
                let schema = SCHEMA.read().unwrap();
                match schema.deref() {
                    Some(schema) => {
                        let data = raw_packed_file.get_data_and_keep_it()?;
                        let packed_file = Loc::read(&data, schema, false)?;
                        Ok(DecodedPackedFile::Loc(packed_file))
                    }
                    None => Err(ErrorKind::SchemaNotFound.into()),
                }
            }

            PackedFileType::MatchedCombat => {
                let schema = SCHEMA.read().unwrap();
                match schema.deref() {
                    Some(schema) => {
                        let data = raw_packed_file.get_data_and_keep_it()?;
                        let packed_file = MatchedCombat::read(&data, schema, false)?;
                        Ok(DecodedPackedFile::MatchedCombat(packed_file))
                    }
                    None => Err(ErrorKind::SchemaNotFound.into()),
                }
            }

            #[cfg(feature = "support_rigidmodel")]
            PackedFileType::RigidModel => {
                let data = raw_packed_file.get_data_and_keep_it()?;
                let packed_file = RigidModel::read(&data);
                Ok(DecodedPackedFile::RigidModel(packed_file))
            }

            PackedFileType::Text(text_type) => {
                let data = raw_packed_file.get_data_and_keep_it()?;
                let mut packed_file = Text::read(&data)?;
                packed_file.set_text_type(text_type);
                Ok(DecodedPackedFile::Text(packed_file))
            }

            #[cfg(feature = "support_uic")]
            PackedFileType::UIC => {
                let schema = SCHEMA.read().unwrap();
                match schema.deref() {
                    Some(schema) => {
                        let data = raw_packed_file.get_data_and_keep_it()?;
                        let packed_file = UIC::read(&data, &schema)?;
                        Ok(DecodedPackedFile::UIC(packed_file))
                    }
                    None => Err(ErrorKind::SchemaNotFound.into()),
                }
            }

            PackedFileType::UnitVariant => {
                let data = raw_packed_file.get_data_and_keep_it()?;
                let packed_file = UnitVariant::read(&data)?;
                Ok(DecodedPackedFile::UnitVariant(packed_file))
            }

            _=> Ok(DecodedPackedFile::Unknown)
        }
    }

    /// This function decodes a `RawPackedFile` into a `DecodedPackedFile`, returning it.
    pub fn decode_no_locks(raw_packed_file: &mut RawPackedFile, schema: &Schema) -> Result<Self> {
        match PackedFileType::get_packed_file_type(raw_packed_file, true) {

            PackedFileType::AnimFragment => {
                let data = raw_packed_file.get_data_and_keep_it()?;
                let packed_file = AnimFragment::read(&data, schema, false)?;
                Ok(DecodedPackedFile::AnimFragment(packed_file))
            }

            PackedFileType::AnimPack => Self::decode(raw_packed_file),

            PackedFileType::AnimTable => {
                let data = raw_packed_file.get_data_and_keep_it()?;
                let packed_file = AnimTable::read(&data, schema, false)?;
                Ok(DecodedPackedFile::AnimTable(packed_file))
            }

            PackedFileType::CaVp8 => Self::decode(raw_packed_file),
            PackedFileType::ESF => Self::decode(raw_packed_file),

            PackedFileType::DB => {
                let data = raw_packed_file.get_data_and_keep_it()?;
                let name = raw_packed_file.get_path().get(1).ok_or_else(|| Error::from(ErrorKind::DBTableIsNotADBTable))?;
                let packed_file = DB::read(&data, name, schema, false)?;
                Ok(DecodedPackedFile::DB(packed_file))
            }

            PackedFileType::Image => Self::decode(raw_packed_file),

            PackedFileType::Loc => {
                let data = raw_packed_file.get_data_and_keep_it()?;
                let packed_file = Loc::read(&data, schema, false)?;
                Ok(DecodedPackedFile::Loc(packed_file))
            }

            PackedFileType::MatchedCombat => {
                let data = raw_packed_file.get_data_and_keep_it()?;
                let packed_file = MatchedCombat::read(&data, schema, false)?;
                Ok(DecodedPackedFile::MatchedCombat(packed_file))
            }

            #[cfg(feature = "support_rigidmodel")]
            PackedFileType::RigidModel => Self::decode(raw_packed_file),

            PackedFileType::Text(_) => Self::decode(raw_packed_file),

            #[cfg(feature = "support_uic")]
            PackedFileType::UIC => {
                let data = raw_packed_file.get_data_and_keep_it()?;
                let packed_file = UIC::read(&data, &schema)?;
                Ok(DecodedPackedFile::UIC(packed_file))
            }

            PackedFileType::UnitVariant => {
                let data = raw_packed_file.get_data_and_keep_it()?;
                let packed_file = UnitVariant::read(&data)?;
                Ok(DecodedPackedFile::UnitVariant(packed_file))
            }
            _=> Ok(DecodedPackedFile::Unknown)
        }
    }

    /// This function encodes a `DecodedPackedFile` into a `Vec<u8>`, returning it.
    ///
    /// Keep in mind this should only work for PackedFiles with saving support.
    pub fn encode(&self) -> Option<Result<Vec<u8>>> {
        match self {
            DecodedPackedFile::AnimFragment(data) => Some(data.save()),
            DecodedPackedFile::AnimPack(data) => Some(Ok(data.save())),
            DecodedPackedFile::AnimTable(data) => Some(data.save()),
            DecodedPackedFile::CaVp8(data) => Some(Ok(data.save())),
            DecodedPackedFile::DB(data) => Some(data.save()),
            DecodedPackedFile::ESF(data) => Some(Ok(data.save())),
            DecodedPackedFile::Loc(data) => Some(data.save()),
            DecodedPackedFile::MatchedCombat(data) => Some(data.save()),

            #[cfg(feature = "support_rigidmodel")]
            DecodedPackedFile::RigidModel(data) => Some(Ok(data.save())),

            DecodedPackedFile::Text(data) => Some(data.save()),

            #[cfg(feature = "support_uic")]
            DecodedPackedFile::UIC(data) => Some(Ok(data.save())),

            DecodedPackedFile::UnitVariant(data) => Some(data.save()),
            _=> None,
        }
    }

    /// This function updates a DB Table to its latest valid version, being the latest valid version the one in the data.pack or equivalent of the game.
    ///
    /// It returns both, old and new versions, or an error.
    pub fn update_table(&mut self, dependencies: &Dependencies) -> Result<(i32, i32)> {
        match self {
            DecodedPackedFile::DB(data) => {
                let dep_db = dependencies.get_db_tables_from_cache(data.get_ref_table_name(), true, false)?;
                if let Some(vanilla_db) = dep_db.iter()
                    .max_by(|x, y| x.get_ref_definition().get_version().cmp(&y.get_ref_definition().get_version())) {

                    let definition_new = vanilla_db.get_definition();
                    let definition_old = data.get_definition();
                    if definition_old != definition_new {
                        data.set_definition(&definition_new);
                        Ok((definition_old.get_version(), definition_new.get_version()))
                    }
                    else {
                        Err(ErrorKind::NoDefinitionUpdateAvailable.into())
                    }
                }
                else { Err(ErrorKind::NoTableInGameFilesToCompare.into()) }
            }
            _ => Err(ErrorKind::DBTableIsNotADBTable.into()),
        }
    }
}

//----------------------------------------------------------------//
// Implementations for `PackedFileType`.
//----------------------------------------------------------------//

/// Display implementation of `PackedFileType`.
impl Display for PackedFileType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            PackedFileType::Anim => write!(f, "Anim"),
            PackedFileType::AnimFragment => write!(f, "AnimFragment"),
            PackedFileType::AnimPack => write!(f, "AnimPack"),
            PackedFileType::AnimTable => write!(f, "AnimTable"),
            PackedFileType::CaVp8 => write!(f, "CA_VP8"),
            PackedFileType::CEO => write!(f, "CEO"),
            PackedFileType::DB => write!(f, "DB Table"),
            PackedFileType::DependencyPackFilesList => write!(f, "Dependency PackFile List"),
            PackedFileType::ESF => write!(f, "ESF"),
            PackedFileType::Image => write!(f, "Image"),
            PackedFileType::GroupFormations => write!(f, "Group Formations"),
            PackedFileType::Loc => write!(f, "Loc Table"),
            PackedFileType::MatchedCombat => write!(f, "Matched Combat"),
            PackedFileType::PackFile => write!(f, "PackFile"),
            PackedFileType::RigidModel => write!(f, "RigidModel"),
            PackedFileType::UIC => write!(f, "UI Component"),
            PackedFileType::UnitVariant => write!(f, "Unit Variant"),
            PackedFileType::Text(text_type) => write!(f, "Text, type: {:?}", text_type),
            PackedFileType::PackFileSettings => write!(f, "PackFile Settings"),
            PackedFileType::Unknown => write!(f, "Unknown"),
        }
    }
}

/// Implementation of `PackedFileType`.
impl PackedFileType {

    /// This function returns the type of the provided `PackedFile` based on the info about them (path, name, extension,...).
    ///
    /// Strict mode also performs a search by checking the data directly if no type was found, but that's very slow. Think twice before using it.
    pub fn get_packed_file_type(packed_file: &RawPackedFile, strict_mode: bool) -> Self {

        // First, try with extensions.
        let path = packed_file.get_path();

        // Reserved PackedFiles.
        if path == [RESERVED_NAME_NOTES] {
            return Self::Text(TextType::Markdown);
        }

        if !path.is_empty() && path.starts_with(&[RESERVED_NAME_EXTRA_PACKFILE.to_owned()]) {
            return Self::PackFile;
        }

        if let Some(packedfile_name) = path.last() {
            let packedfile_name = packedfile_name.to_lowercase();

            if packedfile_name.ends_with(table::loc::EXTENSION) {
                return Self::Loc;
            }

            if packedfile_name.ends_with(rigidmodel::EXTENSION) {
                return Self::RigidModel
            }

            if packedfile_name.ends_with(animpack::EXTENSION) {
                return Self::AnimPack
            }

            if packedfile_name.ends_with(ca_vp8::EXTENSION) {
                return Self::CaVp8;
            }

            if image::EXTENSIONS.iter().any(|x| packedfile_name.ends_with(x)) {
                return Self::Image;
            }

            if let Some((_, text_type)) = text::EXTENSIONS.iter().find(|(x, _)| packedfile_name.ends_with(x)) {
                return Self::Text(*text_type);
            }

            if packedfile_name.ends_with(unit_variant::EXTENSION) {
                return Self::UnitVariant
            }

            if esf::EXTENSIONS.iter().any(|x| packedfile_name.ends_with(*x)) && SETTINGS.read().unwrap().settings_bool["enable_esf_editor"] {
                return Self::ESF;
            }

            // If that failed, try types that need to be in a specific path.
            let path_str = path.iter().map(String::as_str).collect::<Vec<&str>>();
            if path_str.starts_with(&table::matched_combat::BASE_PATH) && packedfile_name.ends_with(table::matched_combat::EXTENSION) {
                return Self::MatchedCombat;
            }

            if path_str.starts_with(&table::animtable::BASE_PATH) && packedfile_name.ends_with(table::animtable::EXTENSION) {
                return Self::AnimTable;
            }

            if path_str.starts_with(&table::anim_fragment::BASE_PATH) && table::anim_fragment::EXTENSIONS.iter().any(|x| packedfile_name.ends_with(*x)) {
                return Self::AnimFragment;
            }

            // If that failed, check if it's in a folder which is known to only have specific files.
            if let Some(folder) = path.get(0) {
                let base_folder = folder.to_lowercase();
                if &base_folder == "db" {
                    return Self::DB;
                }

                if &base_folder == "ui" && (!packedfile_name.contains('.') || packedfile_name.ends_with(uic::EXTENSION)) {
                    return Self::UIC;
                }
            }

            // If nothing worked, then it's simple: if we enabled strict mode, check the data. If not, we don't know.
            // This is very slow when done over a lot of files, so be careful with it.
            if strict_mode {
                let data = packed_file.get_data().unwrap();

                if Text::read(&data).is_ok() {
                    return Self::Text(TextType::Plain);
                }

                if Loc::is_loc(&data) {
                    return Self::Loc;
                }

                if DB::read_header(&data).is_ok() {
                    return Self::DB;
                }

                if CaVp8::is_video(&data) {
                    return Self::CaVp8;
                }

                if UIC::is_ui_component(&data) {
                    return Self::UIC;
                }
            }
        }

        // If we reach this... we're clueless.
        Self::Unknown
    }

    /// This function returns the type of the provided `CachedPackedFile` based on the info about them (path, name, extension,...).
    ///
    /// Strict mode also performs a search by checking the data directly if no type was found, but that's very slow. Think twice before using it.
    pub fn get_cached_packed_file_type(packed_file: &CachedPackedFile, strict_mode: bool) -> Self {

        // First, try with extensions.
        let path = packed_file.get_ref_packed_file_path().to_lowercase();
        if path.ends_with(table::loc::EXTENSION) {
            return Self::Loc;
        }

        if path.ends_with(rigidmodel::EXTENSION) {
            return Self::RigidModel
        }

        if path.ends_with(animpack::EXTENSION) {
            return Self::AnimPack
        }

        if path.ends_with(ca_vp8::EXTENSION) {
            return Self::CaVp8;
        }

        if image::EXTENSIONS.iter().any(|x| path.ends_with(x)) {
            return Self::Image;
        }

        if let Some((_, text_type)) = text::EXTENSIONS.iter().find(|(x, _)| path.ends_with(x)) {
            return Self::Text(*text_type);
        }

        if path.ends_with(unit_variant::EXTENSION) {
            return Self::UnitVariant
        }

        if esf::EXTENSIONS.iter().any(|x| path.ends_with(*x)) && SETTINGS.read().unwrap().settings_bool["enable_esf_editor"] {
            return Self::ESF;
        }

        // If that failed, try types that need to be in a specific path.
        let path_str = path.split('/').collect::<Vec<&str>>();
        if path.ends_with(table::matched_combat::EXTENSION) && path_str.starts_with(&table::matched_combat::BASE_PATH) {
            return Self::MatchedCombat;
        }

        if path.ends_with(table::animtable::EXTENSION) && path_str.starts_with(&table::animtable::BASE_PATH) {
            return Self::AnimTable;
        }

        if path_str.starts_with(&table::anim_fragment::BASE_PATH) && table::anim_fragment::EXTENSIONS.iter().any(|x| path.ends_with(*x)) {
            return Self::AnimFragment;
        }

        // If that failed, check if it's in a folder which is known to only have specific files.
        if let Some(folder) = path_str.get(0) {
            if *folder == "db" {
                return Self::DB;
            }

            if *folder == "ui" && (!path.contains('.') || path.ends_with(uic::EXTENSION)) {
                return Self::UIC;
            }
        }

        // If nothing worked, turn it into a proper PackedFile and try to get the type that way.
        // NOTE: EXTREMELY SLOW!!!!!!
        if strict_mode {
            Self::get_packed_file_type(PackedFile::try_from(packed_file).unwrap().get_ref_raw(), strict_mode);
        }

        // If we reach this... we're clueless.
        Self::Unknown
    }

    /// This function is a less strict version of the one implemented with the `Eq` trait.
    ///
    /// It performs an equality check between both provided types, ignoring the subtypes. This means,
    /// a Text PackedFile with subtype XML and one with subtype LUA will return true, because both are Text PackedFiles.
    pub fn eq_non_strict(self, other: Self) -> bool {
        match self {
            Self::Anim |
            Self::AnimFragment |
            Self::AnimPack |
            Self::AnimTable |
            Self::CaVp8 |
            Self::CEO |
            Self::DB |
            Self::DependencyPackFilesList |
            Self::ESF |
            Self::Image |
            Self::GroupFormations |
            Self::Loc |
            Self::MatchedCombat |
            Self::PackFile |
            Self::RigidModel |
            Self::PackFileSettings |
            Self::UIC |
            Self::UnitVariant |
            Self::Unknown => self == other,
            Self::Text(_) => matches!(other, Self::Text(_)),
        }
    }

    /// This function is a less strict version of the one implemented with the `Eq` trait, adapted to work with slices of types instead of singular types.
    ///
    /// It performs an equality check between both provided types, ignoring the subtypes. This means,
    /// a Text PackedFile with subtype XML and one with subtype LUA will return true, because both are Text PackedFiles.
    pub fn eq_non_strict_slice(self, others: &[Self]) -> bool {
        match self {
            Self::Anim |
            Self::AnimFragment |
            Self::AnimPack |
            Self::AnimTable |
            Self::CaVp8 |
            Self::CEO |
            Self::DB |
            Self::DependencyPackFilesList |
            Self::ESF |
            Self::Image |
            Self::GroupFormations |
            Self::Loc |
            Self::MatchedCombat |
            Self::PackFile |
            Self::RigidModel |
            Self::PackFileSettings |
            Self::UIC |
            Self::UnitVariant |
            Self::Unknown => others.contains(&self),
            Self::Text(_) => others.iter().any(|x| matches!(x, Self::Text(_))),
        }
    }
}

/// From implementation to get the type from a DecodedPackedFile.
impl From<&DecodedPackedFile> for PackedFileType {
    fn from(packed_file: &DecodedPackedFile) -> Self {
        match packed_file {
            DecodedPackedFile::Anim => PackedFileType::Anim,
            DecodedPackedFile::AnimFragment(_) => PackedFileType::AnimFragment,
            DecodedPackedFile::AnimPack(_) => PackedFileType::AnimPack,
            DecodedPackedFile::AnimTable(_) => PackedFileType::AnimTable,
            DecodedPackedFile::CaVp8(_) => PackedFileType::CaVp8,
            DecodedPackedFile::CEO(_) => PackedFileType::CEO,
            DecodedPackedFile::DB(_) => PackedFileType::DB,
            DecodedPackedFile::Image(_) => PackedFileType::Image,
            DecodedPackedFile::GroupFormations => PackedFileType::GroupFormations,
            DecodedPackedFile::Loc(_) => PackedFileType::Loc,
            DecodedPackedFile::MatchedCombat(_) => PackedFileType::MatchedCombat,
            DecodedPackedFile::RigidModel(_) => PackedFileType::RigidModel,
            DecodedPackedFile::ESF(_) => PackedFileType::ESF,
            DecodedPackedFile::Text(text) => PackedFileType::Text(text.get_text_type()),
            DecodedPackedFile::UIC(_) => PackedFileType::UIC,
            DecodedPackedFile::UnitVariant(_) => PackedFileType::UnitVariant,
            DecodedPackedFile::Unknown => PackedFileType::Unknown,
        }
    }
}
*/
