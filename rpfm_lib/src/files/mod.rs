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
//! For more information about specific file types, including their binary format spec, please **check their respective modules**.
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

#[cfg(feature = "integration_sqlite")] use r2d2::Pool;
#[cfg(feature = "integration_sqlite")] use r2d2_sqlite::SqliteConnectionManager;

use getset::*;
use rayon::prelude::*;

use std::collections::HashMap;
use std::fmt::Debug;
use std::fs::File;
use std::io::{BufReader, Cursor, Read, Seek, SeekFrom};
use std::path::Path;

use crate::binary::{ReadBytes, WriteBytes};
use crate::compression::Decompressible;
use crate::encryption::Decryptable;
use crate::error::{Result, RLibError};
use crate::games::pfh_version::PFHVersion;
use crate::schema::Schema;
use crate::utils::*;

use self::loc::Loc;
use self::text::Text;

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

    /// Path of the file within a [`Container`]. It may be an empty string if the file is not in one.
    path_in_container: String,

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
    size: u32,

    /// Is the data compressed?.
    is_compressed: bool,

    /// Is the data encrypted? And if so, with which format?.
    is_encrypted: Option<PFHVersion>,
}

/// This enum allow us to store any kind of decoded file type on a common place.
#[derive(Clone, Debug, PartialEq)]
pub enum RFileDecoded {
    Text(Text),
    Loc(Loc)
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
    Text,
    UIC,
    UnitVariant,
    Unknown,
}

/// This enum represents a ***Path*** inside a [Container].
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

    /// Path of a file on disk, if any.
    disk_file_path: Option<&'a str>,

    /// Offset of a file on disk where the data we're interested on starts.
    disk_file_offset: u64,

    /// Timestamp of a file on disk.
    timestamp: u64,

    /// For [Container] implementors, if they should use LazyLoading for their files.
    lazy_load: bool,

    /// If the data was encrypted (all data that reach the decode functions should be already decrypted).
    is_encrypted: bool,

    /// Schema for the decoder to use. Mainly for tables.
    schema: Option<&'a Schema>,

    /// Name of the folder that contains a table fragment.
    table_name: Option<&'a str>,

    /// If the decoder should return incomplete data on failure (only for tables).
    return_incomplete: bool,

    /// Name of the file we're trying to decode.
    file_name: Option<&'a str>,

    /// SQLite Database Pool. For allowing connections to the database.
    #[cfg(feature = "integration_sqlite")]
    pool: Option<&'a Pool<SqliteConnectionManager>>,
}


/// This is a generic struct to easily pass additional data to a [Encodeable::encode] method.
///
/// To know what you need to provide to each file type, please check their documentation.
#[derive(Clone, Default, Getters, Setters)]
#[getset(get = "pub", set = "pub")]
pub struct EncodeableExtraData<'a> {

    /// If we're running the encode method on test mode.
    test_mode: bool,

    /// Name of the folder that contains a table fragment.
    table_name: Option<&'a str>,

    /// Path of 7z.exe. Used for compressing.
    sevenzip_path: Option<&'a Path>,

    /// Name of the file we're trying to encode.
    file_name: Option<&'a str>,

    /// Only for tables. If we should add a GUID to its header or not.
    table_has_guid: bool,

    /// Only for tables. If we should regenerate the GUID of the table (if it even has one) or keep the current one.
    regenerate_table_guid: bool,

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
    fn decode<R: ReadBytes>(data: &mut R, extra_data: Option<DecodeableExtraData>) -> Result<Self> where Self: Sized;
}

/// A generic trait to implement encoding logic from structured types into anything implementing [WriteBytes](crate::binary::WriteBytes).
pub trait Encodeable: Send + Sync {

    /// This method provides a generic and expandable way to encode any implementor's structure into anything
    /// implementing [WriteBytes](crate::binary::WriteBytes)
    ///
    /// The parameter `extra_data` contains arguments that can be used to provide additional data needed for the encoding process.
    fn encode<W: WriteBytes>(&mut self, buffer: &mut W, extra_data: Option<EncodeableExtraData>) -> Result<()>;
}

/// An interface to easily work with container-like files.
///
/// This trait allow any implementor to provide methods to manipulate them like [RFile] Containers.
pub trait Container {

    /// This method allow us to insert an [RFile] within a Container, replacing any old [RFile]
    /// with the same path, in case it already existed one.
    ///
    /// Returns the [ContainerPath] of the inserted [RFile].
    fn insert(&mut self, file: RFile) -> ContainerPath {
        let path = file.path_in_container();
        let path_raw = file.path_in_container_raw();
        self.files_mut().insert(path_raw.to_owned(), file);
        path
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

    /// This method returns the `Last modified date` of the provided Container on disk, in seconds.
    ///
    /// Implementors should return `0` if the Container doesn't have a file on disk yet.
    fn timestamp(&self) -> u64;
}

// TODO: Implement "possible types" logic, to have some flexibility when opening files.


impl RFile {
    pub fn new_from_container<C: Container>(
        container: &C,
        size: u32,
        is_compressed: bool,
        is_encrypted: Option<PFHVersion>,
        data_pos: u64,
        timestamp: u64,
        path_in_container: &str,
    ) -> Self {
        let on_disk = OnDisk {
            path: container.disk_file_path().to_owned(),
            timestamp: container.timestamp(),
            start: container.disk_file_offset() + data_pos,
            size,
            is_compressed,
            is_encrypted,
        };

        Self {
            path_in_container: path_in_container.to_owned(),
            timestamp: if timestamp == 0 { None } else { Some(timestamp) },
            file_type: FileType::Unknown,
            data: RFileInnerData::OnDisk(on_disk)
        }
    }
/*
                // Build the File as a LazyLoaded file by default.
            let on_disk = OnDisk {
                path: self.disk_file_path.to_owned(),
                timestamp: self.timestamp,
                start: self.disk_file_offset + data_pos,
                size,
                is_compressed,
                is_encrypted: if self.header.bitmask.contains(PFHFlags::HAS_ENCRYPTED_DATA) { Some(self.header.pfh_version) } else { None },
            };

            let file: RFile = RFile {
                path: path.to_owned(),
                timestamp: if timestamp == 0 { None } else { Some(timestamp) },
                file_type: FileType::Unknown,
                data: RFileInnerData::OnDisk(on_disk)
            };
*/

    pub fn new_from_vec(data: &[u8], file_type: FileType, timestamp: u64, path_in_container: &str) -> Self {
        Self {
            path_in_container: path_in_container.to_owned(),
            timestamp: if timestamp == 0 { None } else { Some(timestamp) },
            file_type,
            data: RFileInnerData::Cached(data.to_vec())
        }
    }

    pub fn decode_return(&mut self, keep_in_cache: bool) -> Result<RFileDecoded> {
        let mut already_decoded = false;
        let decoded = match &self.data {
            RFileInnerData::Decoded(data) => {
                already_decoded = true;
                *data.clone()
            },
            RFileInnerData::Cached(data) => {
                let mut data = Cursor::new(data);
                self.decode_bytes(&mut data)?
            },
            RFileInnerData::OnDisk(data) => {
                let raw_data = data.read(data.is_compressed, data.is_encrypted)?;
                self.decode_bytes(&mut Cursor::new(raw_data))?
            },
        };

        if !already_decoded && keep_in_cache {
            self.data = RFileInnerData::Decoded(Box::new(decoded.clone()));
        }

        Ok(decoded)
    }

    fn decode_bytes<R: ReadBytes>(&self, data: &mut R) -> Result<RFileDecoded> {
        Ok(match self.file_type() {
            FileType::Loc => RFileDecoded::Loc(Loc::decode(data, None)?),
            FileType::Text => RFileDecoded::Text(Text::decode(data, None)?),

            // TODO: Create unknown type to store their data here, optionally.
            _ => todo!(),
        })
    }

    pub fn encode(&mut self, keep_in_cache: bool, return_data: bool) -> Result<Option<Vec<u8>>> {
        let mut already_encoded = false;
        let encoded = match &mut self.data {
            RFileInnerData::Decoded(data) => {
                match data.as_mut() {
                    RFileDecoded::Text(data) => {
                        let mut buffer = vec![];
                        data.encode(&mut buffer, None)?;
                        buffer
                    }

                    _ => todo!()
                }
            },
            RFileInnerData::Cached(data) => {
                already_encoded = true;
                data.to_vec()
            },
            RFileInnerData::OnDisk(data) => data.read(data.is_compressed, data.is_encrypted)?,
        };

        if !already_encoded && keep_in_cache {
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

    pub fn timestamp(&self) -> Option<u64> {
        self.timestamp.clone()
    }


    pub fn file_type(&self) -> FileType {
        self.file_type.clone()
    }

    pub fn path_in_container(&self) -> ContainerPath {
        ContainerPath::File(self.path_in_container.to_owned())
    }
    pub fn path_in_container_raw(&self) -> &str {
        &self.path_in_container
    }

    pub fn is_compressible(&self) -> bool {
        match self.file_type {
            FileType::DB |
            FileType::Loc => false,
            _ => true
        }
    }
}

impl OnDisk {

    /// This function tries to read and return the raw data of the PackedFile.
    fn read(&self, decompress: bool, decrypt: Option<PFHVersion>) -> Result<Vec<u8>> {

        // Date check, to ensure the PackFile hasn't been modified since we got the indexes to read it.
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
