//---------------------------------------------------------------------------//
// Copyright (c) 2017-2023 Ismael Gutiérrez González. All rights reserved.
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
//! | File Type            | Decoding Supported | Encoding Supported |
//! | -------------------- | ------------------ | ------------------ |
//! | [`AnimFragment`]     | Yes                | Yes                |
//! | [`AnimPack`]         | Yes                | Yes                |
//! | [`AnimsTable`]       | Yes                | Yes                |
//! | [`DB`]               | Yes                | Yes                |
//! | [`ESF`]              | Limited            | Limited            |
//! | [`Image`]            | Yes                | Yes                |
//! | [`Loc`]              | Yes                | Yes                |
//! | [`MatchedCombat`]    | Yes                | Yes                |
//! | [`Pack`]             | Yes                | Yes                |
//! | [`PortraitSettings`] | No                 | No                 |
//! | [`RigidModel`]       | No                 | No                 |
//! | [`Text`]             | Yes                | Yes                |
//! | [`UIC`]              | No                 | No                 |
//! | [`UnitVariant`]      | Yes                | Yes                |
//! | [`Video`]            | Yes                | Yes                |
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
//! [`DB`]: crate::files::db::DB
//! [`ESF`]: crate::files::esf::ESF
//! [`Image`]: crate::files::image::Image
//! [`Loc`]: crate::files::loc::Loc
//! [`MatchedCombat`]: crate::files::matched_combat::MatchedCombat
//! [`Pack`]: crate::files::pack::Pack
//! [`PortraitSettings`]: crate::files::portrait_settings::PortraitSettings
//! [`RigidModel`]: crate::files::rigidmodel::RigidModel
//! [`Text`]: crate::files::text::Text
//! [`UIC`]: crate::files::uic::UIC
//! [`UnitVariant`]: crate::files::unit_variant::UnitVariant
//! [`Unknown`]: crate::files::unknown::Unknown
//! [`Video`]: crate::files::video::Video


use csv::{QuoteStyle, ReaderBuilder, WriterBuilder};
use getset::*;
#[cfg(feature = "integration_log")] use log::warn;
use rayon::prelude::*;
use serde_derive::{Serialize, Deserialize};

use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use std::{fmt, fmt::{Debug, Display}};
use std::fs::{DirBuilder, File};
use std::io::{BufReader, Cursor, Read, Seek, SeekFrom, BufWriter, Write};
use std::path::{Path, PathBuf};

use crate::binary::{ReadBytes, WriteBytes};
use crate::compression::Decompressible;
use crate::encryption::Decryptable;
use crate::error::{Result, RLibError};
use crate::games::{GameInfo, pfh_version::PFHVersion};
use crate::{REGEX_DB, REGEX_PORTRAIT_SETTINGS};
use crate::schema::{Schema, Definition};
use crate::utils::*;

use self::anim_fragment::AnimFragment;
use self::animpack::AnimPack;
use self::anims_table::AnimsTable;
use self::atlas::Atlas;
use self::audio::Audio;
use self::bmd::Bmd;
use self::db::DB;
use self::esf::ESF;
use self::image::Image;
use self::loc::Loc;
use self::matched_combat::MatchedCombat;
use self::pack::{Pack, RESERVED_NAME_SETTINGS, RESERVED_NAME_NOTES};
use self::portrait_settings::PortraitSettings;
use self::rigidmodel::RigidModel;
use self::soundbank::SoundBank;
use self::text::Text;
use self::uic::UIC;
use self::unit_variant::UnitVariant;
use self::unknown::Unknown;
use self::video::Video;

pub mod anim_fragment;
pub mod animpack;
pub mod anims_table;
pub mod atlas;
pub mod audio;
pub mod bmd;
pub mod bmd_vegetation;
pub mod cs2_parsed;
pub mod db;
pub mod esf;
pub mod image;
pub mod loc;
pub mod matched_combat;
pub mod pack;
pub mod portrait_settings;
pub mod rigidmodel;
pub mod soundbank;
pub mod table;
pub mod text;
pub mod uic;
pub mod unit_variant;
pub mod unknown;
pub mod video;

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
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RFile {

    /// Path of the file, either within a [`Container`] or in the FileSystem.
    ///
    /// It may be an empty string if the file exists only in memory.
    path: String,

    /// Last modified date of the file. Optional.
    timestamp: Option<u64>,

    /// The type of this file.
    file_type: FileType,

    /// Name of the container this [`RFile`] is from, if it's in a contanier.
    container_name: Option<String>,

    /// Inner data of the file.
    ///
    /// Internal only. Users should use the [`RFile`] methods instead of using this directly.
    data: RFileInnerData,
}

/// This enum contains the data of each [`RFile`].
///
/// This is internal only.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
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
#[derive(Clone, Debug, PartialEq, Getters, Serialize, Deserialize)]
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
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum RFileDecoded {
    Anim(Unknown),
    AnimFragment(AnimFragment),
    AnimPack(AnimPack),
    AnimsTable(AnimsTable),
    Atlas(Atlas),
    Audio(Audio),
    BMD(Bmd),
    DB(DB),
    ESF(ESF),
    GroupFormations(Unknown),
    Image(Image),
    Loc(Loc),
    MatchedCombat(MatchedCombat),
    Pack(Pack),
    PortraitSettings(PortraitSettings),
    RigidModel(RigidModel),
    SoundBank(SoundBank),
    Text(Text),
    UIC(UIC),
    UnitVariant(UnitVariant),
    Unknown(Unknown),
    Video(Video),
}

/// This enum specifies the known types of files we can find in a Total War game.
///
/// This list is not exhaustive and it may get bigger in the future as more files are added.
///
/// For each file info, please check their dedicated submodule if exists.
#[derive(Clone, Copy, Debug, Default, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum FileType {
    Anim,
    AnimFragment,
    AnimPack,
    AnimsTable,
    Atlas,
    Audio,
    BMD,
    DB,
    ESF,
    GroupFormations,
    Image,
    Loc,
    MatchedCombat,
    Pack,
    PortraitSettings,
    RigidModel,
    SoundBank,
    Text,
    UIC,
    UnitVariant,
    Video,

    #[default]
    Unknown,
}

/// This enum represents a ***Path*** inside a [Container].
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
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
#[derive(Clone, Debug, Default, Getters, Setters)]
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

    //------------------------------//
    // General-purpouse config data //
    //------------------------------//

    /// Key of the game.
    game_key: Option<&'a str>,

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

    /// If we want to set any date or timestamp to 0.
    nullify_dates: bool,

    /// Only for tables. If we should add a GUID to its header or not.
    table_has_guid: bool,

    /// Only for tables. If we should regenerate the GUID of the table (if it even has one) or keep the current one.
    regenerate_table_guid: bool,

    //-----------------------//
    // Optional config data  //
    //-----------------------//

    /// Key of the game.
    game_key: Option<&'a str>,

    /// Path of 7z.exe. Used for compressing.
    sevenzip_path: Option<PathBuf>,
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

    /// This method allow us to extract anything on a [ContainerPath] from a Container to disk, replacing any old file
    /// with the same path, in case it already existed one.
    ///
    /// If `keep_container_path_structure` is true, the folder structure the file in question has within the container will be replicated on disk.
    ///
    /// The case-insensitive option only works when extracting folders. Individual file extractions are always case sensitive.
    ///
    /// If a schema is provided, this function will try to extract any DB/Loc file as a TSV. If it fails to decode them, it'll extract them as binary files.
    fn extract(&mut self, container_path: ContainerPath, destination_path: &Path, keep_container_path_structure: bool, schema: &Option<Schema>, case_insensitive: bool, extra_data: &Option<EncodeableExtraData>) -> Result<Vec<PathBuf>> {
        let mut extracted_paths = vec![];
        match container_path {
            ContainerPath::File(mut container_path) => {
                if container_path.starts_with('/') {
                    container_path.remove(0);
                }

                let destination_path = if keep_container_path_structure {
                    destination_path.to_owned().join(&container_path)
                } else {
                    destination_path.to_owned()
                };

                let mut destination_folder = destination_path.to_owned();
                destination_folder.pop();
                DirBuilder::new().recursive(true).create(&destination_folder)?;

                let rfile = self.files_mut().get_mut(&container_path).ok_or_else(|| RLibError::FileNotFound(container_path.to_string()))?;

                // If we want to extract as tsv and we got a db/loc, export to tsv.
                if let Some(schema) = schema {
                    if rfile.file_type() == FileType::DB || rfile.file_type() == FileType::Loc {
                        let mut destination_path_tsv = destination_path.to_owned();

                        // Make sure to NOT replace the extension if there is one, only append to it.
                        match destination_path_tsv.extension() {
                            Some(extension) => {
                                let extension = format!("{}.tsv", extension.to_string_lossy());
                                destination_path_tsv.set_extension(extension)
                            },
                            None => destination_path_tsv.set_extension("tsv"),
                        };

                        let result = rfile.tsv_export_to_path(&destination_path_tsv, schema);

                        // If it fails to extract as tsv, extract as binary.
                        if result.is_err() {

                            #[cfg(feature = "integration_log")] {
                                warn!("File with path {} failed to extract as TSV. Extracting it as binary.", rfile.path_in_container_raw());
                            }

                            extracted_paths.push(destination_path.to_owned());
                            let mut file = BufWriter::new(File::create(&destination_path)?);
                            let data = rfile.encode(extra_data, false, false, true)?.unwrap();
                            file.write_all(&data)?;
                        } else {
                            extracted_paths.push(destination_path_tsv);
                            result?;
                        }
                    } else {
                        extracted_paths.push(destination_path.to_owned());
                        let mut file = BufWriter::new(File::create(&destination_path)?);
                        let data = rfile.encode(extra_data, false, false, true)?.unwrap();
                        file.write_all(&data)?;
                    }
                }

                // Otherwise, just write the binary data to disk.
                else {
                    extracted_paths.push(destination_path.to_owned());
                    let mut file = BufWriter::new(File::create(&destination_path)?);
                    let data = rfile.encode(extra_data, false, false, true)?.unwrap();
                    file.write_all(&data)?;
                }
            }
            ContainerPath::Folder(mut container_path) => {
                if container_path.starts_with('/') {
                    container_path.remove(0);
                }

                let mut rfiles = self.files_by_path_mut(&ContainerPath::Folder(container_path.clone()), case_insensitive);
                for rfile in &mut rfiles {
                    let container_path = rfile.path_in_container_raw();
                    let destination_path = if keep_container_path_structure {
                        destination_path.to_owned().join(container_path)
                    } else {
                        destination_path.to_owned()
                    };

                    let mut destination_folder = destination_path.to_owned();
                    destination_folder.pop();
                    DirBuilder::new().recursive(true).create(&destination_folder)?;

                    // If we want to extract as tsv and we got a db/loc, export to tsv.
                    if let Some(schema) = schema {
                        if rfile.file_type() == FileType::DB || rfile.file_type() == FileType::Loc {
                            let mut destination_path_tsv = destination_path.to_owned();

                            // Make sure to NOT replace the extension if there is one, only append to it.
                            match destination_path_tsv.extension() {
                                Some(extension) => {
                                    let extension = format!("{}.tsv", extension.to_string_lossy());
                                    destination_path_tsv.set_extension(extension)
                                },
                                None => destination_path_tsv.set_extension("tsv"),
                            };

                            let result = rfile.tsv_export_to_path(&destination_path_tsv, schema);

                            // If it fails to extract as tsv, extract as binary.
                            if result.is_err() {

                                #[cfg(feature = "integration_log")] {
                                    warn!("File with path {} failed to extract as TSV. Extracting it as binary.", rfile.path_in_container_raw());
                                }

                                extracted_paths.push(destination_path.to_owned());
                                let mut file = BufWriter::new(File::create(&destination_path)?);
                                let data = rfile.encode(extra_data, false, false, true)?.unwrap();
                                file.write_all(&data)?;
                            } else {
                                extracted_paths.push(destination_path_tsv);
                                result?;
                            }
                        } else {
                            extracted_paths.push(destination_path.to_owned());
                            let mut file = BufWriter::new(File::create(&destination_path)?);
                            let data = rfile.encode(extra_data, false, false, true)?.unwrap();
                            file.write_all(&data)?;
                        }
                    }

                    // Otherwise, just write the binary data to disk.
                    else {
                        extracted_paths.push(destination_path.to_owned());
                        let mut file = BufWriter::new(File::create(&destination_path)?);
                        let data = rfile.encode(extra_data, false, false, true)?.unwrap();
                        file.write_all(&data)?;
                    }

                }

                // If we're extracting the whole container, also extract any relevant metadata file associated with it.
                if container_path.is_empty() {
                    extracted_paths.append(&mut self.extract_metadata(destination_path)?);
                }
            }
        }

        Ok(extracted_paths)
    }

    /// This method allows us to extract the metadata associated to the provided container as `.json` files.
    ///
    /// Default implementation does nothing.
    fn extract_metadata(&mut self, _destination_path: &Path) -> Result<Vec<PathBuf>> {
        Ok(vec![])
    }

    /// This method allow us to insert an [RFile] within a Container, replacing any old [RFile]
    /// with the same path, in case it already existed one.
    ///
    /// Returns the [ContainerPath] of the inserted [RFile].
    fn insert(&mut self, file: RFile) -> Result<Option<ContainerPath>> {
        let path = file.path_in_container();
        let path_raw = file.path_in_container_raw();

        self.paths_cache_insert_path(path_raw);
        self.files_mut().insert(path_raw.to_owned(), file);
        Ok(Some(path))
    }

    /// This method allow us to insert a file from disk into an specific path within a Container,
    /// replacing any old [RFile] with the same path, in case it already existed one.
    ///
    /// If a [Schema](crate::schema::Schema) is provided, this function will attempt to import any tsv files it finds into binary files.
    /// If it fails to convert a file, it'll import it as a normal file instead.
    ///
    /// Returns the [ContainerPath] of the inserted [RFile].
    fn insert_file(&mut self, source_path: &Path, container_path_folder: &str, schema: &Option<Schema>) -> Result<Option<ContainerPath>> {
        let mut container_path_folder = container_path_folder.replace('\\', "/");
        if container_path_folder.starts_with('/') {
            container_path_folder.remove(0);
        }

        if container_path_folder.ends_with('/') || container_path_folder.is_empty() {
            let trimmed_path = source_path.file_name()
                .ok_or_else(|| RLibError::PathMissingFileName(source_path.to_string_lossy().to_string()))?
                .to_string_lossy().to_string();
            container_path_folder = container_path_folder.to_owned() + &trimmed_path;
        }

        // If tsv import is enabled, try to import the file to binary before adding it to the Container.
        let mut tsv_imported = false;
        let mut rfile = match schema {
            Some(schema) => {
                match source_path.extension() {
                    Some(extension) => {
                        if extension.to_string_lossy() == "tsv" {
                            tsv_imported = true;
                            let rfile = RFile::tsv_import_from_path(source_path, schema);
                            if let Err(_error) = rfile {

                                #[cfg(feature = "integration_log")] {
                                    warn!("File with path {} failed to import as TSV. Importing it as binary. Error was: {}", &source_path.to_string_lossy(), _error);
                                }

                                tsv_imported = false;
                                RFile::new_from_file_path(source_path)
                            } else {
                                rfile
                            }
                        } else {
                            RFile::new_from_file_path(source_path)
                        }
                    }
                    None => {
                        RFile::new_from_file_path(source_path)
                    }
                }
            }
            None => {
                RFile::new_from_file_path(source_path)
            }
        }?;

        if !tsv_imported {
            rfile.set_path_in_container_raw(&container_path_folder);
        }

        // Make sure to guess the file type before inserting it.
        rfile.guess_file_type()?;

        self.insert(rfile)
    }

    /// This method allow us to insert an entire folder from disk, including subfolders and files,
    /// into an specific path within a Container, replacing any old [RFile] in a path collision.
    ///
    /// By default it doesn't insert the folder itself, but its contents. If you want to include the folder, pass include_base_folder as true.
    ///
    /// If a [Schema](crate::schema::Schema) is provided, this function will attempt to import any tsv files it finds into binary files.
    /// If it fails to convert a file, it'll import it as a normal file instead.
    ///
    /// If ignored paths are provided, paths that match them (as in relative path with the Container as root) will not be included in the Container.
    ///
    /// Returns the list of [ContainerPath] inserted.
    fn insert_folder(&mut self, source_path: &Path, container_path_folder: &str, ignored_paths: &Option<Vec<&str>>, schema: &Option<Schema>, include_base_folder: bool) -> Result<Vec<ContainerPath>> {
        let mut container_path_folder = container_path_folder.replace('\\', "/");
        if !container_path_folder.is_empty() && !container_path_folder.ends_with('/') {
            container_path_folder.push('/');
        }

        if container_path_folder.starts_with('/') {
            container_path_folder.remove(0);
        }

        let mut source_path_without_base_folder = source_path.to_path_buf();
        source_path_without_base_folder.pop();

        let file_paths = files_from_subdir(source_path, true)?;
        let mut inserted_paths = Vec::with_capacity(file_paths.len());
        for file_path in file_paths {
            let trimmed_path = if include_base_folder {
                file_path.strip_prefix(&source_path_without_base_folder)?
            } else {
                file_path.strip_prefix(source_path)?
            }.to_string_lossy().replace('\\', "/");

            let file_container_path = container_path_folder.to_owned() + &trimmed_path;

            if let Some(ignored_paths) = ignored_paths {
                if ignored_paths.iter().any(|x| trimmed_path.starts_with(x)) {
                    continue;
                }
            }

            // If tsv import is enabled, try to import the file to binary before adding it to the Container.
            let mut tsv_imported = false;
            let mut rfile = match schema {
                Some(schema) => {
                    match file_path.extension() {
                        Some(extension) => {
                            if extension.to_string_lossy() == "tsv" {
                                tsv_imported = true;
                                let rfile = RFile::tsv_import_from_path(&file_path, schema);
                                if let Err(_error) = rfile {

                                    #[cfg(feature = "integration_log")] {
                                        warn!("File with path {} failed to import as TSV. Importing it as binary. Error was: {}", &file_path.to_string_lossy(), _error);
                                    }

                                    tsv_imported = false;
                                    RFile::new_from_file_path(&file_path)
                                } else {
                                    rfile
                                }
                            } else {
                                RFile::new_from_file_path(&file_path)
                            }
                        }
                        None => {
                            RFile::new_from_file_path(&file_path)
                        }
                    }
                }
                None => {
                    RFile::new_from_file_path(&file_path)
                }
            }?;

            if !tsv_imported {
                rfile.set_path_in_container_raw(&file_container_path);
            }

            // Make sure to guess the file type before inserting it.
            rfile.guess_file_type()?;

            if let Some(path) = self.insert(rfile)? {
                inserted_paths.push(path);
            }
        }

        Ok(inserted_paths)
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
                let mut path = path.to_owned();
                if path.starts_with('/') {
                    path.remove(0);
                }

                self.paths_cache_remove_path(&path);
                self.files_mut().remove(&path);
                vec![ContainerPath::File(path.to_owned())]
            },
            ContainerPath::Folder(path) => {
                let mut path = path.to_owned();
                if path.starts_with('/') {
                    path.remove(0);
                }

                // If the path is empty, we mean the root of the container, including everything on it.
                if path.is_empty() {
                    self.files_mut().clear();
                    vec![ContainerPath::Folder(String::new()); 1]
                }

                // Otherwise, it's a normal folder.
                else {
                    let mut path_full = path.to_owned();
                    path_full.push('/');

                    let paths_to_remove = self.files().par_iter()
                        .filter_map(|(key, _)| {

                            // Make sure to only pick folders, not files matching folder names or partial folder matches!
                            if key.starts_with(&path_full) {
                                Some(key.to_owned())
                            } else {
                                None
                            }
                        }).collect::<Vec<String>>();

                    paths_to_remove.iter().for_each(|path| {
                        self.paths_cache_remove_path(path);
                        self.files_mut().remove(path);
                    });

                    // Fix for when we try to delete empty folders.
                    if paths_to_remove.is_empty() {
                        vec![ContainerPath::Folder(path); 1]
                    } else {
                        paths_to_remove.par_iter().map(|path| ContainerPath::File(path.to_string())).collect()
                    }
                }
            }
        }
    }

    /// This method returns the path on disk of the provided Container.
    ///
    /// Implementors should return `""` if the provided Container is not from a disk file.
    fn disk_file_path(&self) -> &str;

    /// This method returns the file name on disk of the provided Container.
    fn disk_file_name(&self) -> String {
       PathBuf::from(self.disk_file_path()).file_name().unwrap_or_default().to_string_lossy().to_string()
    }

    /// This method returns the offset of the data of this Container in its disk file.
    ///
    /// Implementors should return `0` if the provided Container is not within another Container.
    fn disk_file_offset(&self) -> u64;

    /// This function checks if a file with a certain path exists in the provided Container.
    fn has_file(&self, path: &str) -> bool {
        self.files().get(path).is_some()
    }

    /// This function checks if a folder with files in it exists in the provided Container.
    fn has_folder(&self, path: &str) -> bool {
        if path.is_empty() {
           false
        } else {

            // Make sure we don't trigger false positives due to similarly started files/folders.
            let path = if path.ends_with('/') {
                path.to_string()
            } else {
                let mut path = path.to_string();
                path.push('/');
                path
            };

            self.files().keys().any(|x| x.starts_with(&path) && x.len() > path.len())
        }
    }

    /// This method returns a reference to a RFile in the Container, if the file exists.
    fn file(&self, path: &str, case_insensitive: bool) -> Option<&RFile> {
        if case_insensitive {
            let lower = path.to_lowercase();
            self.paths_cache().get(&lower).map(|paths| self.files().get(&paths[0])).flatten()
        } else {
            self.files().get(path)
        }
    }

    /// This method returns a mutable reference to a RFile in the Container, if the file exists.
    fn file_mut(&mut self, path: &str, case_insensitive: bool) -> Option<&mut RFile> {
        if case_insensitive {
            let lower = path.to_lowercase();
            self.paths_cache().get(&lower).cloned().map(|paths| self.files_mut().get_mut(&paths[0])).flatten()
        } else {
            self.files_mut().get_mut(path)
        }
    }

    /// This method returns a reference to the RFiles inside the provided Container.
    fn files(&self) -> &HashMap<String, RFile>;

    /// This method returns a mutable reference to the RFiles inside the provided Container.
    fn files_mut(&mut self) -> &mut HashMap<String, RFile>;

    /// This method returns a reference to the RFiles inside the provided Container of the provided FileTypes.
    fn files_by_type(&self, file_types: &[FileType]) -> Vec<&RFile> {
        self.files().par_iter().filter(|(_, file)| file_types.contains(&file.file_type)).map(|(_, file)| file).collect()
    }

    /// This method returns a mutable reference to the RFiles inside the provided Container of the provided FileTypes.
    fn files_by_type_mut(&mut self, file_types: &[FileType]) -> Vec<&mut RFile> {
        self.files_mut().par_iter_mut().filter(|(_, file)| file_types.contains(&file.file_type)).map(|(_, file)| file).collect()
    }

    /// This method returns a reference to the RFiles inside the provided Container that match the provided [ContainerPath].
    ///
    /// An special situation is passing `ContainerPath::Folder("")`. This represents the root of the container,
    /// meaning passing this will return all RFiles within the container.
    fn files_by_path(&self, path: &ContainerPath, case_insensitive: bool) -> Vec<&RFile> {
        match path {
            ContainerPath::File(path) => self.file(path, case_insensitive).map(|file| vec![file]).unwrap_or(vec![]),
            ContainerPath::Folder(path) => {

                // If the path is empty, get everything.
                if path.is_empty() {
                    self.files().values().collect()
                }

                // Otherwise, only get the files under our folder.
                else {
                    self.files().par_iter()
                        .filter_map(|(key, file)|
                            if case_insensitive {
                                if starts_with_case_insensitive(key, path) { Some(file) } else { None }
                            } else if key.starts_with(path) {
                                Some(file)
                            } else {
                                None
                            }
                        ).collect::<Vec<&RFile>>()
                }
            },
        }
    }

    /// This method returns a mutable reference to the RFiles inside the provided Container that match the provided [ContainerPath].
    ///
    /// An special situation is passing `ContainerPath::Folder("")`. This represents the root of the container,
    /// meaning passing this will return all RFiles within the container.
    fn files_by_path_mut(&mut self, path: &ContainerPath, case_insensitive: bool) -> Vec<&mut RFile> {
        match path {
            ContainerPath::File(path) => self.file_mut(path, case_insensitive).map(|file| vec![file]).unwrap_or(vec![]),
            ContainerPath::Folder(path) => {

                // If the path is empty, get everything.
                if path.is_empty() {
                    self.files_mut().values_mut().collect()
                }

                // Otherwise, only get the files under our folder.
                else {
                    self.files_mut().par_iter_mut()
                        .filter_map(|(key, file)|
                            if case_insensitive {
                                if starts_with_case_insensitive(key, path) { Some(file) } else { None }
                            } else if key.starts_with(path) {
                                Some(file)
                            } else {
                                None
                            }
                        ).collect::<Vec<&mut RFile>>()
                }
            },
        }
    }

    /// This method returns a reference to the RFiles inside the provided Container that match one of the provided [ContainerPath].
    fn files_by_paths(&self, paths: &[ContainerPath], case_insensitive: bool) -> Vec<&RFile> {
        paths.iter()
            .flat_map(|path| self.files_by_path(path, case_insensitive))
            .collect()
    }

    /// This method returns a mutable reference to the RFiles inside the provided Container that match the provided [ContainerPath].
    ///
    /// Use this instead of [files_by_path_mut](Self::files_by_path_mut) if you need to get mutable references to multiple files on different [ContainerPath].
    fn files_by_paths_mut(&mut self, paths: &[ContainerPath], case_insensitive: bool) -> Vec<&mut RFile> {
        self.files_mut()
            .iter_mut()
            .filter(|(file_path, _)| {
                paths.iter().any(|path| {
                    match path {
                        ContainerPath::File(path) => {
                            if case_insensitive {
                                caseless::canonical_caseless_match_str(file_path, path)
                            } else {
                                file_path == &path
                            }
                        }
                        ContainerPath::Folder(path) => {
                            if case_insensitive {
                                starts_with_case_insensitive(file_path, path)
                            } else {
                                file_path.starts_with(path)
                            }
                        }
                    }
                })
            })
            .map(|(_, file)| file)
            .collect()
    }

    /// This method returns a reference to the RFiles inside the provided Container that match the provided [ContainerPath]
    /// and are of one of the provided [FileType].
    fn files_by_type_and_paths(&self, file_types: &[FileType], paths: &[ContainerPath], case_insensitive: bool) -> Vec<&RFile> {
        paths.iter()
            .flat_map(|path| self.files_by_path(path, case_insensitive)
                .into_iter()
                .filter(|file| file_types.contains(&file.file_type()))
                .collect::<Vec<_>>()
            ).collect()
    }

    /// This method returns a mutable reference to the RFiles inside the provided Container that match the provided [ContainerPath]
    /// and are of one of the provided [FileType].
    fn files_by_type_and_paths_mut(&mut self, file_types: &[FileType], paths: &[ContainerPath], case_insensitive: bool) -> Vec<&mut RFile> {
        self.files_by_paths_mut(paths, case_insensitive).into_iter().filter(|file| file_types.contains(&file.file_type())).collect()
    }

    /// This method generate the paths cache of the container.
    fn paths_cache_generate(&mut self) {
        self.paths_cache_mut().clear();

        let mut cache: HashMap<String, Vec<String>> = HashMap::new();
        self.files().keys().for_each(|path| {
            let lower = path.to_lowercase();
            match cache.get_mut(&lower) {
                Some(paths) => paths.push(path.to_owned()),
                None => { cache.insert(lower, vec![path.to_owned()]); },
            }
        });

        *self.paths_cache_mut() = cache;
    }

    /// This method adds a path to the paths cache.
    fn paths_cache_insert_path(&mut self, path: &str) {
        let path_lower = path.to_lowercase();
        match self.paths_cache_mut().get_mut(&path_lower) {
            Some(paths) => if paths.iter().all(|x| x != path) {
                paths.push(path.to_owned());
            }
            None => { self.paths_cache_mut().insert(path_lower, vec![path.to_owned()]); }
        }
    }

    /// This method removes a path from the paths cache.
    fn paths_cache_remove_path(&mut self, path: &str) {
        let path_lower = path.to_lowercase();
        match self.paths_cache_mut().get_mut(&path_lower) {
            Some(paths) => {
                match paths.iter().position(|x| x == path) {
                    Some(pos) => {
                        paths.remove(pos);
                        if paths.is_empty() {
                            self.paths_cache_mut().remove(&path_lower);
                        }
                    },
                    #[cfg(feature = "integration_log")]None => { warn!("remove_path received a valid path, but we don't have casing equivalence for it. This is a bug. {}, {}", path_lower, path); },
                    #[cfg(not(feature = "integration_log"))]None => { dbg!("remove_path received a valid path, but we don't have casing equivalence for it. This is a bug. {}, {}", path_lower, path); },
                }
            }
            #[cfg(feature = "integration_log")] None => { warn!("remove_path received an invalid path. This is a bug. {}, {}", path_lower, path); },
            #[cfg(not(feature = "integration_log"))]None => { dbg!("remove_path received an invalid path. This is a bug. {}, {}", path_lower, path); },
        }
    }

    /// This method returns the cache of paths (lowecased -> cased variants) conntained within the Container.
    ///
    /// Please keep in mind if you manipulate the file list in any way, you NEED to update this cache too.
    fn paths_cache(&self) -> &HashMap<String, Vec<String>>;

    /// This method returns the cache of paths (lowecased -> cased variants) conntained within the Container.
    ///
    /// Please keep in mind if you manipulate the file list in any way, you NEED to update this cache too.
    fn paths_cache_mut(&mut self) -> &mut HashMap<String, Vec<String>>;

    /// This method returns the list of folders conntained within the Container.
    fn paths_folders_raw(&self) -> HashSet<String> {
        self.files()
            .par_iter()
            .filter_map(|(path, _)| {
                let file_path_split = path.split('/').collect::<Vec<&str>>();
                let folder_path_len = file_path_split.len() - 1;
                if folder_path_len == 0 {
                    None
                } else {

                    let mut paths = Vec::with_capacity(folder_path_len);

                    for (index, folder) in file_path_split.iter().enumerate() {
                        if index < path.len() - 1 && !folder.is_empty() {
                            paths.push(file_path_split[0..=index].join("/"))
                        }
                    }

                    Some(paths)
                }
            })
            .flatten()
            .collect::<HashSet<String>>()
    }

    /// This method returns the list of [ContainerPath] corresponding to RFiles within the provided Container.
    fn paths(&self) -> Vec<ContainerPath> {
        self.files()
            .par_iter()
            .map(|(path, _)| ContainerPath::File(path.to_owned()))
            .collect()
    }

    /// This method returns the list of paths (as [&str]) corresponding to RFiles within the provided Container.
    fn paths_raw(&self) -> Vec<&str> {
        self.files()
            .par_iter()
            .map(|(path, _)| &**path)
            .collect()
    }

    /// This function returns the list of paths (as [String]) corresponding to RFiles that match the provided [ContainerPath].
    fn paths_raw_from_container_path(&self, path: &ContainerPath) -> Vec<String> {
        match path {
            ContainerPath::File(path) => vec![path.to_owned(); 1],
            ContainerPath::Folder(path) => {

                // If the path is empty, get everything.
                if path.is_empty() {
                    self.paths_raw().iter().map(|x| x.to_string()).collect()
                }

                // Otherwise, only get the paths under our folder.
                else {
                    self.files().par_iter()
                        .filter_map(|(key, file)|
                            if key.starts_with(path) {
                                Some(file.path_in_container_raw().to_owned())
                            } else {
                                None
                            }
                        ).collect::<Vec<String>>()
                }
            },
        }
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

    /// This function preloads to memory any lazy-loaded RFile within this container.
    fn preload(&mut self) -> Result<()> {
        self.files_mut()
            .into_par_iter()
            .try_for_each(|(_, rfile)| rfile.encode(&None, false, true, false).map(|_| ()))
    }

    /// This function allows you to *move* multiple RFiles or folders of RFiles from one folder to another.
    ///
    /// It returns a list with all the new [ContainerPath].
    fn move_paths(&mut self, in_out_paths: &[(ContainerPath, ContainerPath)]) -> Result<Vec<(ContainerPath, ContainerPath)>> {
        let mut successes = vec![];
        for (source_path, destination_path) in in_out_paths {
            successes.append(&mut self.move_path(source_path, destination_path)?);
        }

        Ok(successes)
    }

    /// This function allows you to *move* any RFile or folder of RFiles from one folder to another.
    ///
    /// It returns a list with all the new [ContainerPath].
    fn move_path(&mut self, source_path: &ContainerPath, destination_path: &ContainerPath) -> Result<Vec<(ContainerPath, ContainerPath)>> {
        match source_path {
            ContainerPath::File(source_path) => match destination_path {
                ContainerPath::File(destination_path) => {
                    if destination_path.is_empty() {
                        return Err(RLibError::EmptyDestiny);
                    }

                    self.paths_cache_remove_path(source_path);
                    let mut moved = self
                        .files_mut()
                        .remove(source_path)
                        .ok_or_else(|| RLibError::FileNotFound(source_path.to_string()))?;

                    moved.set_path_in_container_raw(destination_path);

                    self.insert(moved).map(|x| match x {
                        Some(x) => vec![(ContainerPath::File(source_path.to_string()), x); 1],
                        None => Vec::with_capacity(0)
                    })
                },
                ContainerPath::Folder(_) => unreachable!("move_path_1"),
            },
            ContainerPath::Folder(source_path) => match destination_path {
                ContainerPath::File(_) => unreachable!("move_path_2"),
                ContainerPath::Folder(destination_path) => {
                    if destination_path.is_empty() {
                        return Err(RLibError::EmptyDestiny);
                    }

                    // Fix to avoid false positives.
                    let mut source_path_end = source_path.to_owned();
                    if !source_path_end.ends_with('/') {
                        source_path_end.push('/');
                    }

                    let moved_paths = self.files()
                        .par_iter()
                        .filter_map(|(path, _)| if path.starts_with(&source_path_end) { Some(path.to_owned()) } else { None })
                        .collect::<Vec<_>>();

                    let moved = moved_paths.iter()
                        .filter_map(|x| {
                            self.paths_cache_remove_path(x);
                            self.files_mut().remove(x)
                        })
                        .collect::<Vec<_>>();

                    let mut new_paths = Vec::with_capacity(moved.len());
                    for mut moved in moved {
                        let old_path = moved.path_in_container();
                        let new_path = moved.path_in_container_raw().replacen(source_path, destination_path, 1);
                        moved.set_path_in_container_raw(&new_path);

                        if let Some(new_path) = self.insert(moved)? {
                            new_paths.push((old_path, new_path));
                        }
                    }

                    Ok(new_paths)
                },
            },
        }
    }

    /// This function removes all not-in-memory-already Files from the Container.
    ///
    /// Used for removing possibly corrupted RFiles from the Container in order to sanitize it.
    ///
    /// BE CAREFUL WITH USING THIS. IT MAY (PROBABLY WILL) CAUSE DATA LOSSES.
    fn clean_undecoded(&mut self) {
        self.files_mut().retain(|_, file| file.decoded().is_ok() || file.cached().is_ok());
    }
}

//----------------------------------------------------------------//
//                        Implementations
//----------------------------------------------------------------//

impl RFile {

    /// This function creates a RFile from a lazy-loaded file inside a Container.
    ///
    /// About the parameters:
    /// - `container`: The container this RFile is on.
    /// - `size`: Size in bytes of the RFile.
    /// - `is_compressed`: If the RFile is compressed.
    /// - `is_encrypted`: If the RFile is encrypted.
    /// - `data_pos`: Byte offset of the data from the beginning of the Container.
    /// - `file_timestamp`: Timestamp of this specific file (not of the container, but the file). If it doesn't have one, pass 0.
    /// - `path_in_container`: Path of the RFile in the container.
    ///
    /// NOTE: Remember to call `guess_file_type` after this to properly set the FileType.
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

        let rfile = Self {
            path: path_in_container.to_owned(),
            timestamp: if file_timestamp == 0 { None } else { Some(file_timestamp) },
            file_type: FileType::Unknown,
            container_name: Some(container.disk_file_name()),
            data: RFileInnerData::OnDisk(on_disk)
        };

        Ok(rfile)
    }

    /// This function creates a RFile from a path on disk.
    ///
    /// This may fail if the file doesn't exist or errors out when trying to be read for metadata.
    ///
    /// NOTE: Remember to call `guess_file_type` after this to properly set the FileType.
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


        let rfile = Self {
            path: path.to_owned(),
            timestamp: Some(on_disk.timestamp),
            file_type: FileType::Unknown,
            container_name: None,
            data: RFileInnerData::OnDisk(on_disk)
        };

        Ok(rfile)
    }

    /// This function creates a RFile from a path on disk.
    ///
    /// This may fail if the file doesn't exist or errors out when trying to be read for metadata.
    ///
    /// NOTE: Remember to call `guess_file_type` after this to properly set the FileType.
    pub fn new_from_file_path(path: &Path) -> Result<Self> {
        let path = path.to_string_lossy().to_string();
        Self::new_from_file(&path)
    }

    /// This function creates a RFile from raw data on memory.
    ///
    /// NOTE: Remember to call `guess_file_type` after this to properly set the FileType.
    pub fn new_from_vec(data: &[u8], file_type: FileType, timestamp: u64, path: &str) -> Self {
        Self {
            path: path.to_owned(),
            timestamp: if timestamp == 0 { None } else { Some(timestamp) },
            file_type,
            container_name: None,
            data: RFileInnerData::Cached(data.to_vec())
        }
    }

    /// This function creates a RFile from an RFileDecoded on memory.
    ///
    /// NOTE: Remember to call `guess_file_type` after this to properly set the FileType.
    pub fn new_from_decoded(data: &RFileDecoded, timestamp: u64, path: &str) -> Self {
        Self {
            path: path.to_owned(),
            timestamp: if timestamp == 0 { None } else { Some(timestamp) },
            file_type: FileType::from(data),
            container_name: None,
            data: RFileInnerData::Decoded(Box::new(data.clone()))
        }
    }

    /// This function returns a reference to the cached data of an RFile, if said RFile has been cached. If not, it returns an error.
    ///
    /// Useful for accessing preloaded data.
    pub fn cached(&self) -> Result<&[u8]> {
        match self.data {
            RFileInnerData::Cached(ref data) => Ok(data),
            _ => Err(RLibError::FileNotCached(self.path_in_container_raw().to_string()))
        }
    }

    /// This function returns a mutable reference to the cached data of an RFile, if said RFile has been cached. If not, it returns an error.
    ///
    /// Useful for accessing preloaded data.
    pub fn cached_mut(&mut self) -> Result<&mut Vec<u8>> {
        match self.data {
            RFileInnerData::Cached(ref mut data) => Ok(data),
            _ => Err(RLibError::FileNotCached(self.path_in_container_raw().to_string()))
        }
    }

    /// This function returns a reference to the decoded data of an RFile, if said RFile has been decoded. If not, it returns an error.
    ///
    /// Useful for accessing preloaded data.
    pub fn decoded(&self) -> Result<&RFileDecoded> {
        match self.data {
            RFileInnerData::Decoded(ref data) => Ok(data),
            _ => Err(RLibError::FileNotDecoded(self.path_in_container_raw().to_string()))
        }
    }

    /// This function returns a mutable reference to the decoded data of an RFile, if said RFile has been decoded. If not, it returns an error.
    ///
    /// Useful for accessing preloaded data.
    pub fn decoded_mut(&mut self) -> Result<&mut RFileDecoded> {
        match self.data {
            RFileInnerData::Decoded(ref mut data) => Ok(data),
            _ => Err(RLibError::FileNotDecoded(self.path_in_container_raw().to_string()))
        }
    }

    /// This function replace any data a RFile has with the provided raw data.
    pub fn set_cached(&mut self, data: &[u8]) {
        self.data = RFileInnerData::Cached(data.to_vec());
    }

    /// This function allows to replace the inner decoded data of a RFile with another. It'll fail if the decoded data is not valid for the file's type.
    pub fn set_decoded(&mut self, decoded: RFileDecoded) -> Result<()> {
        match (self.file_type(), &decoded) {
            (FileType::Anim, &RFileDecoded::Anim(_)) |
            (FileType::AnimFragment, &RFileDecoded::AnimFragment(_)) |
            (FileType::AnimPack, &RFileDecoded::AnimPack(_)) |
            (FileType::AnimsTable, &RFileDecoded::AnimsTable(_)) |
            (FileType::Atlas, &RFileDecoded::Atlas(_)) |
            (FileType::Audio, &RFileDecoded::Audio(_)) |
            (FileType::BMD, &RFileDecoded::BMD(_)) |
            (FileType::DB, &RFileDecoded::DB(_)) |
            (FileType::ESF, &RFileDecoded::ESF(_)) |
            (FileType::GroupFormations, &RFileDecoded::GroupFormations(_)) |
            (FileType::Image, &RFileDecoded::Image(_)) |
            (FileType::Loc, &RFileDecoded::Loc(_)) |
            (FileType::MatchedCombat, &RFileDecoded::MatchedCombat(_)) |
            (FileType::Pack, &RFileDecoded::Pack(_)) |
            (FileType::PortraitSettings, &RFileDecoded::PortraitSettings(_)) |
            (FileType::RigidModel, &RFileDecoded::RigidModel(_)) |
            (FileType::SoundBank, &RFileDecoded::SoundBank(_)) |
            (FileType::Text, &RFileDecoded::Text(_)) |
            (FileType::UIC, &RFileDecoded::UIC(_)) |
            (FileType::UnitVariant, &RFileDecoded::UnitVariant(_)) |
            (FileType::Unknown, &RFileDecoded::Unknown(_)) |
            (FileType::Video, &RFileDecoded::Video(_)) => self.data = RFileInnerData::Decoded(Box::new(decoded)),
            _ => return Err(RLibError::DecodedDataDoesNotMatchFileType(self.file_type(), From::from(&decoded)))
        }

        Ok(())
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

                // Microoptimization: don't clone data if we're not going to use it.
                if !return_data {
                    return Ok(None);
                }

                *data.clone()
            },

            // If the data is on memory but not yet decoded, decode it.
            RFileInnerData::Cached(data) => {

                // Copy the provided extra data (if any), then replace the file-specific stuff.
                let mut extra_data = match extra_data {
                    Some(extra_data) => extra_data.clone(),
                    None => DecodeableExtraData::default(),
                };
                extra_data.file_name = self.file_name();
                extra_data.data_size = data.len() as u64;

                // Some types require extra data specific for them to be added to the extra data before decoding.
                let mut data = Cursor::new(data);
                match self.file_type {
                    FileType::Anim => RFileDecoded::Anim(Unknown::decode(&mut data, &Some(extra_data))?),
                    FileType::AnimFragment => RFileDecoded::AnimFragment(AnimFragment::decode(&mut data, &Some(extra_data))?),
                    FileType::AnimPack => RFileDecoded::AnimPack(AnimPack::decode(&mut data, &Some(extra_data))?),
                    FileType::AnimsTable => RFileDecoded::AnimsTable(AnimsTable::decode(&mut data, &Some(extra_data))?),
                    FileType::Atlas => RFileDecoded::Atlas(Atlas::decode(&mut data, &Some(extra_data))?),
                    FileType::Audio => RFileDecoded::Audio(Audio::decode(&mut data, &Some(extra_data))?),
                    FileType::BMD => RFileDecoded::BMD(Bmd::decode(&mut data, &Some(extra_data))?),
                    FileType::DB => {

                        if extra_data.table_name.is_none() {
                            extra_data.table_name = self.db_table_name_from_path();
                        }
                        RFileDecoded::DB(DB::decode(&mut data, &Some(extra_data))?)
                    },
                    FileType::ESF => RFileDecoded::ESF(ESF::decode(&mut data, &Some(extra_data))?),
                    FileType::GroupFormations => RFileDecoded::GroupFormations(Unknown::decode(&mut data, &Some(extra_data))?),
                    FileType::Image => RFileDecoded::Image(Image::decode(&mut data, &Some(extra_data))?),
                    FileType::Loc => RFileDecoded::Loc(Loc::decode(&mut data, &Some(extra_data))?),
                    FileType::MatchedCombat => RFileDecoded::MatchedCombat(MatchedCombat::decode(&mut data, &Some(extra_data))?),
                    FileType::Pack => RFileDecoded::Pack(Pack::decode(&mut data, &Some(extra_data))?),
                    FileType::PortraitSettings => RFileDecoded::PortraitSettings(PortraitSettings::decode(&mut data, &Some(extra_data))?),
                    FileType::RigidModel => RFileDecoded::RigidModel(RigidModel::decode(&mut data, &Some(extra_data))?),
                    FileType::SoundBank => RFileDecoded::SoundBank(SoundBank::decode(&mut data, &Some(extra_data))?),
                    FileType::Text => RFileDecoded::Text(Text::decode(&mut data, &Some(extra_data))?),
                    FileType::UIC => RFileDecoded::UIC(UIC::decode(&mut data, &Some(extra_data))?),
                    FileType::UnitVariant => RFileDecoded::UnitVariant(UnitVariant::decode(&mut data, &Some(extra_data))?),
                    FileType::Unknown => RFileDecoded::Unknown(Unknown::decode(&mut data, &Some(extra_data))?),
                    FileType::Video => RFileDecoded::Video(Video::decode(&mut data, &Some(extra_data))?),
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
                    FileType::Atlas |
                    FileType::Audio |
                    FileType::BMD |
                    FileType::DB |
                    FileType::ESF |
                    FileType::GroupFormations |
                    FileType::Image |
                    FileType::Loc |
                    FileType::MatchedCombat |
                    FileType::PortraitSettings |
                    FileType::RigidModel |
                    FileType::SoundBank |
                    FileType::Text |
                    FileType::UIC |
                    FileType::UnitVariant |
                    FileType::Unknown |
                    FileType::Video => {

                        // Copy the provided extra data (if any), then replace the file-specific stuff.
                        let raw_data = data.read(data.is_compressed, data.is_encrypted)?;
                        let mut extra_data = match extra_data {
                            Some(extra_data) => extra_data.clone(),
                            None => DecodeableExtraData::default(),
                        };

                        extra_data.file_name = self.file_name();
                        extra_data.data_size = raw_data.len() as u64;

                        // These are the easy types: just load the data to memory, and decode.
                        let mut data = Cursor::new(raw_data);
                        match self.file_type {
                            FileType::Anim => RFileDecoded::Anim(Unknown::decode(&mut data, &Some(extra_data))?),
                            FileType::AnimFragment => RFileDecoded::AnimFragment(AnimFragment::decode(&mut data, &Some(extra_data))?),
                            FileType::AnimsTable => RFileDecoded::AnimsTable(AnimsTable::decode(&mut data, &Some(extra_data))?),
                            FileType::Atlas => RFileDecoded::Atlas(Atlas::decode(&mut data, &Some(extra_data))?),
                            FileType::Audio => RFileDecoded::Audio(Audio::decode(&mut data, &Some(extra_data))?),
                            FileType::BMD => RFileDecoded::BMD(Bmd::decode(&mut data, &Some(extra_data))?),
                            FileType::DB => {

                                if extra_data.table_name.is_none() {
                                    extra_data.table_name = self.db_table_name_from_path();
                                }
                                RFileDecoded::DB(DB::decode(&mut data, &Some(extra_data))?)
                            },
                            FileType::ESF => RFileDecoded::ESF(ESF::decode(&mut data, &Some(extra_data))?),
                            FileType::GroupFormations => RFileDecoded::GroupFormations(Unknown::decode(&mut data, &Some(extra_data))?),
                            FileType::Image => RFileDecoded::Image(Image::decode(&mut data, &Some(extra_data))?),
                            FileType::Loc => RFileDecoded::Loc(Loc::decode(&mut data, &Some(extra_data))?),
                            FileType::MatchedCombat => RFileDecoded::MatchedCombat(MatchedCombat::decode(&mut data, &Some(extra_data))?),
                            FileType::PortraitSettings => RFileDecoded::PortraitSettings(PortraitSettings::decode(&mut data, &Some(extra_data))?),
                            FileType::RigidModel => RFileDecoded::RigidModel(RigidModel::decode(&mut data, &Some(extra_data))?),
                            FileType::SoundBank => RFileDecoded::SoundBank(SoundBank::decode(&mut data, &Some(extra_data))?),
                            FileType::Text => RFileDecoded::Text(Text::decode(&mut data, &Some(extra_data))?),
                            FileType::UIC => RFileDecoded::UIC(UIC::decode(&mut data, &Some(extra_data))?),
                            FileType::UnitVariant => RFileDecoded::UnitVariant(UnitVariant::decode(&mut data, &Some(extra_data))?),
                            FileType::Unknown => RFileDecoded::Unknown(Unknown::decode(&mut data, &Some(extra_data))?),
                            FileType::Video => RFileDecoded::Video(Video::decode(&mut data, &Some(extra_data))?),

                            FileType::AnimPack |
                            FileType::Pack => unreachable!("decode")
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
                        extra_data.file_name = self.file_name();
                        extra_data.data_size = data.size;

                        // If we're lazy-loading we also need extra data to read from disk on-demand.
                        if extra_data.lazy_load {
                            extra_data.disk_file_path = Some(&data.path);
                            extra_data.disk_file_offset = data.start;
                            extra_data.timestamp = last_modified_time_from_file(&File::open(&data.path)?)?;

                            let mut data = data.read_lazily()?;
                            match self.file_type {
                                FileType::AnimPack => RFileDecoded::AnimPack(AnimPack::decode(&mut data, &Some(extra_data))?),
                                FileType::Pack => RFileDecoded::Pack(Pack::decode(&mut data, &Some(extra_data))?),
                                _ => unreachable!("decode_2")
                            }
                        }

                        // If we're not using lazy-loading, just use the normal method.
                        else {
                            let raw_data = data.read(data.is_compressed, data.is_encrypted)?;
                            let mut data = Cursor::new(raw_data);

                            match self.file_type {
                                FileType::AnimPack => RFileDecoded::AnimPack(AnimPack::decode(&mut data, &Some(extra_data))?),
                                FileType::Pack => RFileDecoded::Pack(Pack::decode(&mut data, &Some(extra_data))?),
                                _ => unreachable!("decode_3")
                            }
                        }
                    }
                }
            },
        };

        // If we're returning data, clone it. If not, skip the clone.
        if !already_decoded && keep_in_cache && return_data {
            self.data = RFileInnerData::Decoded(Box::new(decoded.clone()));
        } else if !already_decoded && keep_in_cache && !return_data{
            self.data = RFileInnerData::Decoded(Box::new(decoded));
            return Ok(None)
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
                    RFileDecoded::Atlas(data) => data.encode(&mut buffer, extra_data)?,
                    RFileDecoded::Audio(data) => data.encode(&mut buffer, extra_data)?,
                    RFileDecoded::BMD(data) => data.encode(&mut buffer, extra_data)?,
                    RFileDecoded::DB(data) => data.encode(&mut buffer, extra_data)?,
                    RFileDecoded::ESF(data) => data.encode(&mut buffer, extra_data)?,
                    RFileDecoded::GroupFormations(data) => data.encode(&mut buffer, extra_data)?,
                    RFileDecoded::Image(data) => data.encode(&mut buffer, extra_data)?,
                    RFileDecoded::Loc(data) => data.encode(&mut buffer, extra_data)?,
                    RFileDecoded::MatchedCombat(data) => data.encode(&mut buffer, extra_data)?,
                    RFileDecoded::Pack(data) => data.encode(&mut buffer, extra_data)?,
                    RFileDecoded::PortraitSettings(data) => data.encode(&mut buffer, extra_data)?,
                    RFileDecoded::RigidModel(data) => data.encode(&mut buffer, extra_data)?,
                    RFileDecoded::SoundBank(data) => data.encode(&mut buffer, extra_data)?,
                    RFileDecoded::Text(data) => data.encode(&mut buffer, extra_data)?,
                    RFileDecoded::UIC(data) => data.encode(&mut buffer, extra_data)?,
                    RFileDecoded::UnitVariant(data) => data.encode(&mut buffer, extra_data)?,
                    RFileDecoded::Unknown(data) => data.encode(&mut buffer, extra_data)?,
                    RFileDecoded::Video(data) => data.encode(&mut buffer, extra_data)?,
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

        // Piece of code to find text files we do not support yet. Needs enabling the content_inspector crate.
        #[cfg(feature = "enable_content_inspector")]
        if self.file_type() == FileType::Unknown && content_inspector::inspect(&loaded).is_text() {
            dbg!(self.path_in_container_raw());
        }

        self.data = RFileInnerData::Cached(loaded);
        Ok(())
    }

    /// This function returns a copy of the `Last modified date` of this RFile, if any.
    pub fn timestamp(&self) -> Option<u64> {
        self.timestamp
    }

    /// This function returns a copy of the FileType of this RFile.
    pub fn file_type(&self) -> FileType {
        self.file_type
    }

    /// This function returns the file name if this RFile, if it has one.
    pub fn file_name(&self) -> Option<&str> {
        self.path_in_container_raw().split('/').last()
    }

    /// This function returns the file name of the container this RFile originates from, if any.
    pub fn container_name(&self) -> &Option<String> {
        &self.container_name
    }

    /// This function returns the [ContainerPath] corresponding to this file.
    pub fn path_in_container(&self) -> ContainerPath {
        ContainerPath::File(self.path.to_owned())
    }

    /// This function returns the [ContainerPath] corresponding to this file as an [&str].
    pub fn path_in_container_raw(&self) -> &str {
        &self.path
    }

    /// This function returns the [ContainerPath] corresponding to this file as a [Vec] of [&str].
    pub fn path_in_container_split(&self) -> Vec<&str> {
        self.path.split('/').collect()
    }

    /// This function the *table_name* of this file (the folder that contains this file) if this file is a DB table.
    ///
    /// It returns None of the file provided is not a DB Table.
    pub fn db_table_name_from_path(&self) -> Option<&str> {
        let split_path = self.path.split('/').collect::<Vec<_>>();
        if split_path.len() == 3 && split_path[0].to_lowercase() == "db" {
            Some(split_path[1])
        } else {
            None
        }
    }

    /// This function sets the [ContainerPath] of the provided RFile to the provided path..
    pub fn set_path_in_container_raw(&mut self, path: &str) {
        self.path = path.to_owned();
    }

    /// This function returns if the RFile can be compressed or not.
    pub fn is_compressible(&self) -> bool {
        !matches!(self.file_type, FileType::DB | FileType::Loc) && self.file_name() != Some(RESERVED_NAME_SETTINGS) && self.file_name() != Some(RESERVED_NAME_NOTES)
    }

    /// This function guesses the [`FileType`] of the provided RFile and stores it on it for later queries.
    ///
    /// The way it works is: first it tries to guess it by extension (fast), then by full path (not as fast), then by data (slow and it may fail on lazy-loaded files).
    ///
    /// This may fail for some files, so if you doubt set the type manually.
    pub fn guess_file_type(&mut self) -> Result<()> {

        // First, try with extensions.
        let path = self.path.to_lowercase();

        // TODO: Add autodetection to these, somehow
        //--GroupFormations,

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

        else if path.ends_with(video::EXTENSION) {
            self.file_type =  FileType::Video;
        }

        else if audio::EXTENSIONS.iter().any(|x| path.ends_with(x)) {
            self.file_type = FileType::Audio;
        }

        // TODO: detect bin files for maps and tile maps.
        else if bmd::EXTENSIONS.iter().any(|x| path.ends_with(x)) {
            self.file_type = FileType::BMD;
        }

        else if cfg!(feature = "support_soundbank") && path.ends_with(soundbank::EXTENSION) {
            self.file_type =  FileType::SoundBank;
        }

        else if image::EXTENSIONS.iter().any(|x| path.ends_with(x)) {
            self.file_type =  FileType::Image;
        }

        else if cfg!(feature = "support_uic") && path.starts_with(uic::BASE_PATH) && uic::EXTENSIONS.iter().any(|x| path.ends_with(x) || !path.contains('.')) {
            self.file_type =  FileType::UIC;
        }

        else if text::EXTENSIONS.iter().any(|(x, _)| path.ends_with(x)) {
            self.file_type = FileType::Text;
        }

        else if path.ends_with(unit_variant::EXTENSION) {
            self.file_type = FileType::UnitVariant
        }

        else if esf::EXTENSIONS.iter().any(|x| path.ends_with(x)) {
            self.file_type = FileType::ESF;
        }

        // If that failed, try types that need to be in a specific path.
        else if matched_combat::BASE_PATHS.iter().any(|x| path.starts_with(*x)) && path.ends_with(matched_combat::EXTENSION) {
            self.file_type = FileType::MatchedCombat;
        }

        else if path.starts_with(anims_table::BASE_PATH) && path.ends_with(anims_table::EXTENSION) {
            self.file_type =  FileType::AnimsTable;
        }

        else if path.starts_with(anim_fragment::BASE_PATH) && anim_fragment::EXTENSIONS.iter().any(|x| path.ends_with(*x)) {
            self.file_type = FileType::AnimFragment;
        }

        // If that failed, check if it's in a folder which is known to only have specific files.
        // Microoptimization: check the path before using the regex. Regex is very, VERY slow.
        else if path.starts_with("db/") && REGEX_DB.is_match(&path) {
            self.file_type = FileType::DB;
        }

        else if path.ends_with(portrait_settings::EXTENSION) && REGEX_PORTRAIT_SETTINGS.is_match(&path) {
            self.file_type = FileType::PortraitSettings;
        }

        else if path.ends_with(atlas::EXTENSION) {
            self.file_type = FileType::Atlas;
        }

        // If we reach this... we're clueless. Leave it unknown.
        else {
            self.file_type = FileType::Unknown;
        }

        Ok(())
    }

    /// This function allows to import a TSV file on the provided Path into a binary database file.
    ///
    /// It requires the path on disk of the TSV file and the Schema to use.
    pub fn tsv_import_from_path(path: &Path, schema: &Schema) -> Result<Self> {

        // We want the reader to have no quotes, tab as delimiter and custom headers, because otherwise
        // Excel, Libreoffice and all the programs that edit this kind of files break them on save.
        let mut reader = ReaderBuilder::new()
            .delimiter(b'\t')
            .quoting(false)
            .has_headers(true)
            .flexible(true)
            .from_path(path)?;

        // If we successfully load the TSV file into a reader, check the first line to get the column list and order.
        let field_order = reader.headers()?
            .iter()
            .enumerate()
            .map(|(x, y)| (x as u32, y.to_owned()))
            .collect::<HashMap<u32, String>>();

        // Get the record iterator so we can check the metadata from the second row.
        let mut records = reader.records();
        let (table_type, table_version, file_path) = match records.next() {
            Some(Ok(record)) => {
                let metadata = match record.get(0) {
                    Some(metadata) => metadata.split(';').map(|x| x.to_owned()).collect::<Vec<String>>(),
                    None => return Err(RLibError::ImportTSVWrongTypeTable),
                };

                let table_type = match metadata.get(0) {
                    Some(table_type) => {
                        let mut table_type = table_type.to_owned();
                        if table_type.starts_with('#') {
                            table_type.remove(0);
                        }
                        table_type
                    },
                    None => return Err(RLibError::ImportTSVWrongTypeTable),
                };

                let table_version = match metadata.get(1) {
                    Some(table_version) => table_version.parse::<i32>().map_err(|_| RLibError::ImportTSVInvalidVersion)?,
                    None => return Err(RLibError::ImportTSVInvalidVersion),
                };

                let file_path = match metadata.get(2) {
                    Some(file_path) => file_path.replace('\\', "/"),
                    None => return Err(RLibError::ImportTSVInvalidOrMissingPath),
                };

                (table_type, table_version, file_path)
            }
            Some(Err(_)) |
            None => return Err(RLibError::ImportTSVIncorrectRow(1, 0)),
        };

        // Once we get the metadata, we know what kind of file we have. Create it and pass the records.
        let decoded = match &*table_type {
            loc::TSV_NAME_LOC | loc::TSV_NAME_LOC_OLD => {
                let decoded = Loc::tsv_import(records, &field_order)?;
                RFileDecoded::Loc(decoded)
            }

            // Any other name is assumed to be a db table.
            _ => {
                let decoded = DB::tsv_import(records, &field_order, schema, &table_type, table_version)?;
                RFileDecoded::DB(decoded)
            }
        };

        let rfile = RFile::new_from_decoded(&decoded, 0, &file_path);
        Ok(rfile)
    }

    /// This function allows to export a RFile into a TSV file on disk.
    ///
    /// Only supported for DB and Loc files.
    pub fn tsv_export_to_path(&mut self, path: &Path, schema: &Schema) -> Result<()> {

        // Make sure the folder actually exists.
        let mut folder_path = path.to_path_buf();
        folder_path.pop();
        DirBuilder::new().recursive(true).create(&folder_path)?;

        // We want the writer to have no quotes, tab as delimiter and custom headers, because otherwise
        // Excel, Libreoffice and all the programs that edit this kind of files break them on save.
        let mut writer = WriterBuilder::new()
            .delimiter(b'\t')
            .quote_style(QuoteStyle::Never)
            .has_headers(false)
            .flexible(true)
            .from_path(path)?;

        let mut extra_data = DecodeableExtraData::default();
        extra_data.set_schema(Some(schema));

        let extra_data = Some(extra_data);

        // If it fails in decoding, delete the tsv file.
        let file = self.decode(&extra_data, false, true);
        if let Err(error) = file {
            let _ = std::fs::remove_file(path);
            return Err(error);
        }

        let file = match file?.unwrap() {
            RFileDecoded::DB(table) => table.tsv_export(&mut writer, self.path_in_container_raw()),
            RFileDecoded::Loc(table) => table.tsv_export(&mut writer, self.path_in_container_raw()),
            _ => unimplemented!()
        };

        // If the tsv export failed, delete the tsv file.
        if file.is_err() {
            let _ = std::fs::remove_file(path);
        }

        file
    }

    /// This function tries to merge multiple files into one.
    ///
    /// All files must be of the same type and said type must support merging.
    pub fn merge(sources: &[&Self], path: &str) -> Result<Self> {
        if sources.len() <= 1 {
            return Err(RLibError::RFileMergeOnlyOneFileProvided);
        }

        let mut file_types = sources.iter().map(|file| file.file_type()).collect::<Vec<_>>();
        file_types.sort();
        file_types.dedup();

        if file_types.len() > 1 {
            return Err(RLibError::RFileMergeDifferentTypes);
        }

        match file_types[0] {
            FileType::DB => {
                let files = sources.iter().filter_map(|file| if let Ok(RFileDecoded::DB(table)) = file.decoded() { Some(table) } else { None }).collect::<Vec<_>>();
                let data = RFileDecoded::DB(DB::merge(&files)?);
                Ok(Self::new_from_decoded(&data, current_time()?, path))
            },
            FileType::Loc => {
                let files = sources.iter().filter_map(|file| if let Ok(RFileDecoded::Loc(table)) = file.decoded() { Some(table) } else { None }).collect::<Vec<_>>();
                let data = RFileDecoded::Loc(Loc::merge(&files)?);
                Ok(Self::new_from_decoded(&data, current_time()?, path))
            },
            _ => Err(RLibError::RFileMergeNotSupportedForType(file_types[0].to_string())),
        }
    }

    /// This function tries to update a file to a new version of the same file's format.
    ///
    /// Files used by this function are expected to be pre-decoded.
    pub fn update(&mut self, definition: &Option<Definition>) -> Result<()> {
        match self.decoded_mut() {
            Ok(RFileDecoded::DB(file)) => match definition {
                Some(definition) => file.update(definition),
                None => return Err(RLibError::RawTableMissingDefinition),
            }
            _ => return Err(RLibError::FileNotDecoded(self.path_in_container_raw().to_string())),
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
            return Err(RLibError::FileSourceChanged);
        }

        // Read the data from disk.
        let mut data = vec![0; self.size as usize];
        file.seek(SeekFrom::Start(self.start))?;
        file.read_exact(&mut data)?;

        // If the data is encrypted, decrypt it.
        if decrypt.is_some() {
            data = Cursor::new(data).decrypt()?;
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
            return Err(RLibError::FileSourceChanged);
        }

        file.seek(SeekFrom::Start(self.start))?;
        Ok(file)
    }
}

impl ContainerPath {

    /// This function returns true if the provided [ContainerPath] corresponds to a file.
    pub fn is_file(&self) -> bool {
        matches!(self, ContainerPath::File(_))
    }

    /// This function returns true if the provided [ContainerPath] corresponds to a folder.
    pub fn is_folder(&self) -> bool {
        matches!(self, ContainerPath::Folder(_))
    }

    /// This function returns true if the provided [ContainerPath] corresponds to a root Pack.
    pub fn is_pack(&self) -> bool {
        match self {
            ContainerPath::Folder(path) => path.is_empty(),
            _ => false,
        }
    }

    /// This function returns a reference to the path stored within the provided [ContainerPath].
    pub fn path_raw(&self) -> &str {
        match self {
            Self::File(ref path) => path,
            Self::Folder(ref path) => path,
        }
    }

    /// This function returns the last item of the provided [ContainerPath], if any.
    pub fn name(&self) -> Option<&str> {
        self.path_raw().split('/').last()
    }

    /// This function the *table_name* of this file (the folder that contains this file) if this file is a DB table.
    ///
    /// It returns None of the file provided is not a DB Table.
    pub fn db_table_name_from_path(&self) -> Option<&str> {
        let split_path = self.path_raw().split('/').collect::<Vec<_>>();
        if split_path.len() == 3 && split_path[0].to_lowercase() == "db" {
            Some(split_path[1])
        } else {
            None
        }
    }

    /// This function returns the path of the parent folder of the provided [ContainerPath].
    ///
    /// If the provided [ContainerPath] corresponds to a Container root, the path returned will be the current one.
    pub fn parent_path(&self) -> String {
        match self {
            ContainerPath::File(path) |
            ContainerPath::Folder(path) => {
                if path.is_empty() || (path.chars().count() == 1 && path.starts_with('/')) {
                    path.to_owned()
                } else {
                    let mut path_split = path.split('/').collect::<Vec<_>>();
                    path_split.pop();
                    path_split.join("/")
                }
            },
        }
    }

    /// This function removes collided items from the provided list of [ContainerPath].
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
        if !paths.par_iter().any(|item| matches!(item, ContainerPath::Folder(_))) {
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
                    !paths.par_iter()
                        .any(|item_type| {

                        // If the other one is a folder that contains it, dont add it.
                        item_type.is_folder() && path_to_add.starts_with(item_type.path_raw())
                    })
                }

                // If it's a folder, we have to check if there is already another folder containing it.
                ContainerPath::Folder(ref path_to_add) => {
                    !paths.par_iter()
                        .any(|item_type| {

                        // If the other one is a folder that contains it, dont add it.
                        let path = item_type.path_raw();
                        item_type.is_folder() && path_to_add.starts_with(path) && path_to_add.len() > path.len()
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
                ContainerPath::File(b) => a.cmp(b),
                ContainerPath::Folder(_) => Ordering::Less,
            }
            ContainerPath::Folder(a) => match other {
                ContainerPath::File(_) => Ordering::Greater,
                ContainerPath::Folder(b) => a.cmp(b),
            }
        }
    }
}

impl PartialOrd for ContainerPath {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Display for FileType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            FileType::Anim => write!(f, "Anim"),
            FileType::AnimFragment => write!(f, "AnimFragment"),
            FileType::AnimPack => write!(f, "AnimPack"),
            FileType::AnimsTable => write!(f, "AnimsTable"),
            FileType::Atlas => write!(f, "Atlas"),
            FileType::Audio => write!(f, "Audio"),
            FileType::BMD => write!(f, "Battle Map Definition"),
            FileType::DB => write!(f, "DB Table"),
            FileType::ESF => write!(f, "ESF"),
            FileType::GroupFormations => write!(f, "Group Formations"),
            FileType::Image => write!(f, "Image"),
            FileType::Loc => write!(f, "Loc Table"),
            FileType::MatchedCombat => write!(f, "Matched Combat"),
            FileType::Pack => write!(f, "PackFile"),
            FileType::PortraitSettings => write!(f, "Portrait Settings"),
            FileType::RigidModel => write!(f, "RigidModel"),
            FileType::SoundBank => write!(f, "SoundBank"),
            FileType::Text => write!(f, "Text"),
            FileType::UIC => write!(f, "UI Component"),
            FileType::UnitVariant => write!(f, "Unit Variant"),
            FileType::Unknown => write!(f, "Unknown"),
            FileType::Video => write!(f, "Video"),
        }
    }
}

impl From<&str> for FileType {
    fn from(value: &str) -> Self {
        match value {
            "Anim" => FileType::Anim,
            "AnimFragment" => FileType::AnimFragment,
            "AnimPack" => FileType::AnimPack,
            "AnimsTable" => FileType::AnimsTable,
            "Atlas" => FileType::Atlas,
            "Audio" => FileType::Audio,
            "BMD" => FileType::BMD,
            "DB" => FileType::DB,
            "ESF" => FileType::ESF,
            "GroupFormations" => FileType::GroupFormations,
            "Image" => FileType::Image,
            "Loc" => FileType::Loc,
            "MatchedCombat" => FileType::MatchedCombat,
            "Pack" => FileType::Pack,
            "PortraitSettings" => FileType::PortraitSettings,
            "RigidModel" => FileType::RigidModel,
            "SoundBank" => FileType::SoundBank,
            "Text" => FileType::Text,
            "UIC" => FileType::UIC,
            "UnitVariant" => FileType::UnitVariant,
            "Unknown" => FileType::Unknown,
            "Video" => FileType::Video,
            _ => unimplemented!(),
        }
    }
}

impl From<FileType> for String {
    fn from(value: FileType) -> String {
        match value {
            FileType::Anim => "Anim",
            FileType::AnimFragment => "AnimFragment",
            FileType::AnimPack => "AnimPack",
            FileType::AnimsTable => "AnimsTable",
            FileType::Atlas => "Atlas",
            FileType::Audio => "Audio",
            FileType::BMD => "BMD",
            FileType::DB => "DB",
            FileType::ESF => "ESF",
            FileType::GroupFormations => "GroupFormations",
            FileType::Image => "Image",
            FileType::Loc => "Loc",
            FileType::MatchedCombat => "MatchedCombat",
            FileType::Pack => "Pack",
            FileType::PortraitSettings => "PortraitSettings",
            FileType::RigidModel => "RigidModel",
            FileType::SoundBank => "SoundBank",
            FileType::Text => "Text",
            FileType::UIC => "UIC",
            FileType::UnitVariant => "UnitVariant",
            FileType::Unknown => "Unknown",
            FileType::Video => "Video",
        }.to_owned()
    }
}

impl From<&RFileDecoded> for FileType {
    fn from(file: &RFileDecoded) -> Self {
        match file {
            RFileDecoded::Anim(_) => Self::Anim,
            RFileDecoded::AnimFragment(_) => Self::AnimFragment,
            RFileDecoded::AnimPack(_) => Self::AnimPack,
            RFileDecoded::AnimsTable(_) => Self::AnimsTable,
            RFileDecoded::Atlas(_) => Self::Atlas,
            RFileDecoded::Audio(_) => Self::Audio,
            RFileDecoded::BMD(_) => Self::BMD,
            RFileDecoded::DB(_) => Self::DB,
            RFileDecoded::ESF(_) => Self::ESF,
            RFileDecoded::GroupFormations(_) => Self::GroupFormations,
            RFileDecoded::Image(_) => Self::Image,
            RFileDecoded::Loc(_) => Self::Loc,
            RFileDecoded::MatchedCombat(_) => Self::MatchedCombat,
            RFileDecoded::Pack(_) => Self::Pack,
            RFileDecoded::PortraitSettings(_) => Self::PortraitSettings,
            RFileDecoded::RigidModel(_) => Self::RigidModel,
            RFileDecoded::SoundBank(_) => Self::SoundBank,
            RFileDecoded::Text(_) => Self::Text,
            RFileDecoded::UIC(_) => Self::UIC,
            RFileDecoded::UnitVariant(_) => Self::UnitVariant,
            RFileDecoded::Unknown(_) => Self::Unknown,
            RFileDecoded::Video(_) => Self::Video,
        }
    }
}

impl<'a> EncodeableExtraData<'a> {

    /// This functions generates an EncodeableExtraData for a specific game.
    pub fn new_from_game_info(game_info: &'a GameInfo) -> Self {
        let mut extra_data = Self::default();
        extra_data.set_game_key(Some(game_info.key()));
        extra_data.set_table_has_guid(*game_info.db_tables_have_guid());
        extra_data
    }
}
