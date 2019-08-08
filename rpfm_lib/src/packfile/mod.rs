//---------------------------------------------------------------------------//
// Copyright (c) 2017-2019 Ismael Gutiérrez González. All rights reserved.
// 
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
// 
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

/*!
Module with all the code to interact with PackFiles.

This module contains all the code related with PackFiles. If you want to do anything with a PackFile,
this is the place you have to come.

Also, something to take into account. RPFM supports PackFile compression/decompression and decryption,
and that is handled automagically by RPFM. All the data you'll ever see will be decompressed/decrypted,
so you don't have to worry about that.
!*/

use bitflags::bitflags;

use std::{fmt, fmt::Display};
use std::fs::{DirBuilder, File};
use std::io::{prelude::*, BufReader, BufWriter, SeekFrom, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use rpfm_error::{ErrorKind, Result};

use crate::GAME_SELECTED;
use crate::SETTINGS;
use crate::SUPPORTED_GAMES;
use crate::common::{*, decoder::Decoder, encoder::Encoder};
use crate::packfile::compression::*;
use crate::packfile::crypto::*;
use crate::packfile::packedfile::*;

mod compression;
mod crypto;
pub mod packedfile;

#[cfg(test)]
mod packfile_test;

/// These consts are used for dealing with Time-related operations.
const WINDOWS_TICK: i64 = 10_000_000;
const SEC_TO_UNIX_EPOCH: i64 = 11_644_473_600;

/// These are the different Preamble/Id the PackFiles can have.
const PFH5_PREAMBLE: &str = "PFH5"; // PFH5
const PFH4_PREAMBLE: &str = "PFH4"; // PFH4
const PFH3_PREAMBLE: &str = "PFH3"; // PFH3
const PFH0_PREAMBLE: &str = "PFH0"; // PFH0

/// This is the list of ***Reserved PackedFile Names***. They're packedfile names used by RPFM for special porpouses.
pub const RESERVED_PACKED_FILE_NAMES: [&str; 1] = ["frodos_biggest_secret.rpfm-notes"];

/// These are the types the PackFiles can have.
const FILE_TYPE_BOOT: u32 = 0;
const FILE_TYPE_RELEASE: u32 = 1;
const FILE_TYPE_PATCH: u32 = 2;
const FILE_TYPE_MOD: u32 = 3;
const FILE_TYPE_MOVIE: u32 = 4;
bitflags! {

    /// This represents the bitmasks a PackFile can have applied to his type.
    ///
    /// Keep in mind that this lib supports decoding PackFiles with any of these flags enabled, 
    /// but it only supports enconding for the `HAS_INDEX_WITH_TIMESTAMPS` flag.
    pub struct PFHFlags: u32 {
        
        /// Used to specify that the header of the PackFile is extended by 20 bytes. Used in Arena.
        const HAS_EXTENDED_HEADER       = 0b0000_0001_0000_0000;
        
        /// Used to specify that the PackedFile Index is encrypted. Used in Arena.
        const HAS_ENCRYPTED_INDEX       = 0b0000_0000_1000_0000;
    
        /// Used to specify that the PackedFile Index contains a timestamp of every PackFile.
        const HAS_INDEX_WITH_TIMESTAMPS = 0b0000_0000_0100_0000;
        
        /// Used to specify that the PackedFile's data is encrypted. Seen in `music.pack` PackFiles and in Arena.
        const HAS_ENCRYPTED_DATA        = 0b0000_0000_0001_0000;
    }
}

//---------------------------------------------------------------------------//
//                              Enum & Structs
//---------------------------------------------------------------------------//

/// This `Struct` stores the data of the PackFile in memory, along with some extra data needed to manipulate the PackFile.
#[derive(Debug, Clone, PartialEq)]
pub struct PackFile {
    
    /// The path of the PackFile on disk, if exists. If not, then this should be empty.
    file_path: PathBuf,

    /// The version of the PackFile.
    pfh_version: PFHVersion,

    /// The type of the PackFile.
    pfh_file_type: PFHFileType,

    /// The bitmasks applied to the PackFile.
    bitmask: PFHFlags,

    /// The timestamp of the last time the PackFile was saved.
    timestamp: i64,

    /// The list of PackFiles this PackFile requires to be loaded before himself when starting the game.
    ///
    /// In other places, we refer to this as the `Dependency List`.
    pack_files: Vec<String>,

    /// The list of PackedFiles this PackFile contains.
    packed_files: Vec<PackedFile>,

    /// Notes added to the PackFile. Exclusive of this lib.
    notes: Option<String>,
}

/// This struct is a reduced version of the `PackFile` one, used to pass just the needed data to an UI.
///
/// Don't create this one manually. Get it `From` the `PackFile` one, and use it as you need it.
#[derive(Debug)]
pub struct PackFileInfo {

    /// The path of the PackFile on disk, if exists. If not, then this should be empty.
    pub file_path: PathBuf,

    /// The version of the PackFile.
    pub pfh_version: PFHVersion,

    /// The type of the PackFile.
    pub pfh_file_type: PFHFileType,

    /// The bitmasks applied to the PackFile.
    pub bitmask: PFHFlags,

    /// The current state of the compression inside the PackFile.
    pub compression_state: CompressionState,

    /// The timestamp of the last time the PackFile was saved.
    pub timestamp: i64,
}


/// This enum represents the **Version** of a PackFile.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PFHVersion {
    
    /// Used in Warhammer 2, Three Kingdoms and Arena.
    PFH5,

    /// Used in Warhammer 1, Attila, Rome 2, and Thrones of Brittania.
    PFH4,

    /// Used in Shogun 2.
    PFH3,

    /// Used in Napoleon and Empire.
    PFH0
}

/// This enum represents the **Type** of a PackFile.
///
/// The types here are sorted in the same order they'll load when the game starts. 
/// The number in their docs is their numeric value when read from a PackFile.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PFHFileType {

    /// **(0)**: Used in CA PackFiles, not useful for modding.
    Boot,

    /// **(1)**: Used in CA PackFiles, not useful for modding.
    Release,

    /// **(2)**: Used in CA PackFiles, not useful for modding.
    Patch,

    /// **(3)**: Used for mods. PackFiles of this type are only loaded in the game if they are enabled in the Mod Manager/Launcher.
    Mod,

    /// **(4)** Used in CA PackFiles and for some special mods. Unlike `Mod` PackFiles, these ones always get loaded.
    Movie,

    /// Wildcard for any type that doesn't fit in any of the other categories. The type's value is stored in the Variant.
    Other(u32),
}

/// This enum represents the type of a path in a PackFile.
///
/// Keep in mind that, in the lib we don't have a reliable way to determine if a path is a file or a folder if their path conflicts.
/// For example, if we have the folder "x/y/z/" and the file "x/y/z", and we ask the lib what type of path it's, we'll default to a file.
#[derive(Debug, Clone)]
pub enum PathType {

    /// Used for PackedFile paths. Contains the path of the PackedFile.
    File(Vec<String>),

    /// Used for folder paths. Contains the path of the folder.
    Folder(Vec<String>),

    /// Used for the PackFile itself.
    PackFile,

    /// Used for any other situation. Usually, if this is used, there is a detection problem somewhere else.
    None,
}

/// This enum indicates the current state of the compression in the current PackFile.
///
/// Despite compression being per-packedfile, we only support applying it to the full PackFile for now.
/// Also, compression is only supported by `PFHVersion::PFH5` PackFiles.
#[derive(Debug, Clone)]
pub enum CompressionState {

    /// All the PackedFiles in the PackFile are compressed.
    Enabled,

    /// Some of the PackedFiles in the PackFile are compressed.
    Partial,

    /// None of the files in the PackFile are compressed.
    Disabled,
}

//---------------------------------------------------------------------------//
//                             Enum Implementations
//---------------------------------------------------------------------------//

/// Implementation of `PFHFileType`.
impl PFHFileType {

    /// This function returns the PackFile's **Type** in `u32` format. To know what value corresponds with what type, check their definition's comment.
    pub fn get_value(self) -> u32 {
        match self {
            PFHFileType::Boot => FILE_TYPE_BOOT,
            PFHFileType::Release => FILE_TYPE_RELEASE,
            PFHFileType::Patch => FILE_TYPE_PATCH,
            PFHFileType::Mod => FILE_TYPE_MOD,
            PFHFileType::Movie => FILE_TYPE_MOVIE,
            PFHFileType::Other(value) => value
        }
    }

    /// This function returns the PackFile's Type corresponding to the provided value.
    pub fn get_type(value: u32) -> Self {
        match value {
            FILE_TYPE_BOOT => PFHFileType::Boot,
            FILE_TYPE_RELEASE => PFHFileType::Release,
            FILE_TYPE_PATCH => PFHFileType::Patch,
            FILE_TYPE_MOD => PFHFileType::Mod,
            FILE_TYPE_MOVIE => PFHFileType::Movie,
            _ => PFHFileType::Other(value),
        }
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
            PFHFileType::Movie => write!(f, "Movie"),
            PFHFileType::Other(version) => write!(f, "Other: {}", version),
        }
    }
}

/// Implementation of `PFHVersion`.
impl PFHVersion {

    /// This function returns the PackFile's *Id/Preamble* (his 4 first bytes) as a `&str`.
    pub fn get_value(&self) -> &str {
        match *self {
            PFHVersion::PFH5 => PFH5_PREAMBLE,
            PFHVersion::PFH4 => PFH4_PREAMBLE,
            PFHVersion::PFH3 => PFH3_PREAMBLE,
            PFHVersion::PFH0 => PFH0_PREAMBLE,
        }
    }

    /// This function returns the PackFile's `PFHVersion` corresponding to the provided value, or an error if the provided value is not a valid `PFHVersion`.
    pub fn get_version(value: &str) -> Result<Self> {
        match value {
            PFH5_PREAMBLE => Ok(PFHVersion::PFH5),
            PFH4_PREAMBLE => Ok(PFHVersion::PFH4),
            PFH3_PREAMBLE => Ok(PFHVersion::PFH3),
            PFH0_PREAMBLE => Ok(PFHVersion::PFH0),
            _ => Err(ErrorKind::PackFileIsNotAPackFile)?,
        }
    }
}

/// Display implementation of `PFHVersion`.
impl Display for PFHVersion {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            PFHVersion::PFH5 => write!(f, "PFH5"),
            PFHVersion::PFH4 => write!(f, "PFH4"),
            PFHVersion::PFH3 => write!(f, "PFH3"),
            PFHVersion::PFH0 => write!(f, "PFH0"),
        }
    }
}

/// Implementation of `PathType`.
impl PathType {

    /// This function removes duplicates and collisioned items from the provided list of `PathType`.
    ///
    /// This means, if you have an item of type `PackFile` it removes the rest of the items.
    /// If you have a file and a folder containing the file, it removes the file. And so on.
    pub fn dedup(path_types: &[Self]) -> Vec<Self> {
        let mut item_types_clean = vec![];
        for item_type_to_add in path_types {
            match item_type_to_add {

                // If it's a file, we have to check both, if the file is duplicate and if there is a folder containing it.
                PathType::File(ref path_to_add) => {
                    let mut add_type = true;
                    for item_type in path_types {
                        
                        // Skip the current file from checks.
                        if let PathType::File(ref path) = item_type {
                            if path == path_to_add { continue; }
                        }

                        // If the other one is a folder that contains it, dont add it.
                        else if let PathType::Folder(ref path) = item_type {
                            if path_to_add.starts_with(path) { 
                                add_type = false;
                                break;
                            }
                        }
                    }
                    if add_type { item_types_clean.push(item_type_to_add.clone()); }
                }

                // If it's a folder, we have to check if there is already another folder containing it.
                PathType::Folder(ref path_to_add) => {
                    let mut add_type = true;
                    for item_type in path_types {

                        // If the other one is a folder that contains it, dont add it.
                        if let PathType::Folder(ref path) = item_type {
                            if path == path_to_add { continue; }
                            if path_to_add.starts_with(path) { 
                                add_type = false;
                                break;
                            }
                        }
                    }
                    if add_type { item_types_clean.push(item_type_to_add.clone()); }
                }

                // If we got the PackFile, remove everything.
                PathType::PackFile => {
                    item_types_clean.clear();
                    item_types_clean.push(item_type_to_add.clone());
                    break;
                }

                // If we receive one of these... better start praying.
                PathType::None => unimplemented!(),
            }   
        }
        item_types_clean
    }
}

//---------------------------------------------------------------------------//
//                           Structs Implementations
//---------------------------------------------------------------------------//

/// Implementation of `PackFile`.
impl PackFile {

    /// This function creates a new empty `PackFile`. This is used for creating a *dummy* PackFile we'll later populate.
    pub fn new() -> Self {
        Self {
            file_path: PathBuf::new(),
            pfh_version: PFHVersion::PFH5,
            pfh_file_type: PFHFileType::Mod,
            bitmask: PFHFlags::empty(),
            timestamp: 0,

            pack_files: vec![],
            packed_files: vec![],

            notes: None
        }
    }

    /// This function creates a new empty `PackFile` with a name and a specific `PFHVersion`.
    pub fn new_with_name(file_name: &str, pfh_version: PFHVersion) -> Self {
        Self {
            file_path: PathBuf::from(file_name),
            pfh_version,
            pfh_file_type: PFHFileType::Mod,
            bitmask: PFHFlags::empty(),
            timestamp: 0,

            pack_files: vec![],
            packed_files: vec![],

            notes: None,
        }
    }

    /// This function returns a list of reserved PackedFile names, used by RPFM for special porpouses.
    pub fn get_reserved_packed_file_names() -> Vec<Vec<String>> {
        RESERVED_PACKED_FILE_NAMES.iter().map(|x| vec![x.to_string()]).collect()
    }

    /// This function returns the `PackFile List` of the provided `PackFile`.
    pub fn get_packfiles_list(&self) -> &[String] {
        &self.pack_files
    }

    /// This function replaces the `PackFile List` of our `PackFile` with the provided one.
    pub fn set_packfiles_list(&mut self, pack_files: &[String]) {
        self.pack_files = pack_files.to_vec();
    }

    /// This function retuns the list of PackedFiles inside a `PackFile`.
    pub fn get_packedfiles_list(&self) -> Vec<Vec<String>> {
        self.packed_files.iter().map(|x| x.get_path().to_vec()).collect()
    }

    /// This function adds a `PackedFiles` to an existing `PackFile`.
    ///
    /// This function returns the paths of the `PackedFile` which got added succesfully. Also, if you set `overwrite` to `true`, 
    /// in case of conflict, the `PackedFile` is overwritten. If set to false, it'll be renamed instead.
    pub fn add_packed_file(&mut self, packed_file: &PackedFile, overwrite: bool) -> Result<Vec<String>> {

        // If we hit a reserved name, stop. Don't add anything.
        if packed_file.get_path() == RESERVED_PACKED_FILE_NAMES { return Err(ErrorKind::ReservedFiles)? }

        // Get his path, and update his `PackFile` name.
        let mut packed_file = packed_file.clone();
        let mut destination_path = packed_file.get_path().to_vec();
        packed_file.set_packfile_name(&self.get_file_name());
        match self.packed_files.iter().position(|x| x.get_path() == packed_file.get_path()) {

            // Here is were the fun starts. If there is a conflict, we act depending on the `overwrite` value.
            Some(index) => {
                if overwrite { self.packed_files[index] = packed_file }
                else {
                    let reserved_names = Self::get_reserved_packed_file_names();
                    let name_current = destination_path.last().unwrap().to_owned(); 
                    let name_splitted = name_current.split('.').collect::<Vec<&str>>();
                    let name = name_splitted[0];
                    let extension = if name_splitted.len() > 1 { name_splitted[1..].join(".") } else { "".to_owned() };
                    for number in 0.. {
                        let name = if extension.is_empty() { format!("{}_{}", name, number) } else { format!("{}_{}.{}", name, number, extension) };
                        *destination_path.last_mut().unwrap() = name;
                        if !self.packedfile_exists(&destination_path) && !reserved_names.contains(&destination_path) {
                            break;
                        }
                    }
                }
            }

            // If there is no conflict, just add the `PackedFile`.
            None => self.packed_files.push(packed_file),
        }  
        Ok(destination_path)
    }

    /// This function is used to add a file from disk to a `PackFile`, turning it into a `PackedFile`.
    ///
    /// In case of conflict, if overwrite is set to true, the current `PackedFile` in the conflicting path
    /// will be overwritten with the new one. If set to false, the new `PackFile` will be called `xxxx_1.extension`.
    pub fn add_from_file(
        &mut self,
        path_as_file: &PathBuf,
        path_as_packed_file: Vec<String>,
        overwrite: bool,
    ) -> Result<Vec<String>> {
        let packed_file = PackedFile::read_from_path(path_as_file, path_as_packed_file)?;
        self.add_packed_file(&packed_file, overwrite)
    }


    /// This function is used to add a `PackedFile` from one `PackFile` into another. 
    ///
    /// It's a ***Copy from another PackFile*** kind of function. It returns the paths 
    /// of whatever got added to our `PackFile`, and the paths that failed.
    pub fn add_from_packfile(
        &mut self,
        source: &Self,
        path_type: &PathType,
        overwrite: bool,
    ) -> (Vec<Vec<String>>, Vec<Vec<String>>) {

        // Keep the PathTypes added so we can return them to the UI easely.
        let mut paths_ok = vec![];
        let mut paths_err = vec![];
        match path_type {

            // If the `PathType` is a file, we just get the `PackedFile` and add it to our `PackFile`.
            PathType::File(path) => {
                if let Some(packed_file) = source.get_ref_packed_file_by_path(path) {
                    match self.add_packed_file(&packed_file, overwrite) {
                        Ok(path) => paths_ok.push(path),
                        Err(_) => paths_err.push(path.to_vec()),
                    }
                }
            }

            // If the `PathType` is a folder we just replicate the file behavior in a loop.
            PathType::Folder(ref path) => {
                for packed_file in source.get_ref_packed_files_by_path_start(path) {
                    match self.add_packed_file(&packed_file, overwrite) {
                        Ok(path) => paths_ok.push(path),
                        Err(_) => paths_err.push(packed_file.get_path().to_vec()),
                    }
                }
            }

            // If we want to add an entire PackedFile, just repeat the process with all the `PackedFiles`.
            PathType::PackFile => {
                for packed_file in source.get_ref_all_packed_files() {
                    match self.add_packed_file(&packed_file, overwrite) {
                        Ok(path) => paths_ok.push(path),
                        Err(_) =>  paths_err.push(packed_file.get_path().to_vec()),
                    }
                }
            },

            // In any other case, there is a problem somewhere. Otherwise, this is unreachable.
            _ => unreachable!()
        }
        (paths_ok, paths_err)
    }


    /// This function returns the name of the PackFile. If it's empty, it's an in-memory only PackFile. 
    pub fn get_file_name(&self) -> String {
        match self.file_path.file_name() {
            Some(s) => s.to_string_lossy().to_string(),
            None => String::new()
        }
    }

    /// This function returns the path of the PackFile. If it's empty, it's an in-memory only PackFile. 
    pub fn get_file_path(&self) -> &PathBuf {
        &self.file_path
    }

    /// This function changes the path of the PackFile.
    ///
    /// This can fail if you pass it an empty path.
    pub fn set_file_path(&mut self, path: &Path) -> Result<()> {
        if path.components().count() == 0 { return Err(ErrorKind::EmptyInput)? }
        self.file_path = path.to_path_buf();

        // We have to change the name of the PackFile in all his `PackedFiles` too.
        let file_name = self.get_file_name();
        self.packed_files.iter_mut().for_each(|x| x.set_packfile_name(&file_name));
        Ok(())
    }    

    /// This function returns the current compression state of the provided `PackFile`.
    ///
    /// To get more info about the different compression states, check the `CompressionState` enum.
    pub fn get_compression_state(&self) -> CompressionState {
        let mut has_files_compressed = false;
        let mut has_files_uncompressed = false;
        for packed_file in &self.packed_files {
            let is_compressed = packed_file.get_compression_state();
            if !has_files_compressed && is_compressed {
                has_files_compressed = true;
            }

            if !has_files_uncompressed && !is_compressed {
                has_files_uncompressed = true;
            }

            if has_files_uncompressed && has_files_compressed {
                break;
            }
        }
        if has_files_compressed && has_files_uncompressed { CompressionState::Partial }
        else if has_files_compressed { CompressionState::Enabled }
        else { CompressionState::Disabled }
    }

    /// This function returns if the `PackFile` is editable or not.
    ///
    /// By *if is editable or not* I mean *If you can save it or not*. The conditions under which a PackFile is not editable are:
    /// - All PackFiles with extended header or encrypted parts are not editable.
    /// - All PackFiles of type `Mod` or `Movie` are editable. 
    /// - If you say CA PackFiles are not editable:
    ///   - All PackFiles of type `Boot`, `Release` or `Patch` are not editable.
    /// - If you say CA PackFiles are editable:
    ///   - All PackFiles of type `Boot`, `Release` or `Patch` are editable.
    pub fn is_editable(&self, is_editing_of_ca_packfiles_allowed: bool) -> bool {

        // If it's this very specific type, don't save under any circunstance.
        if let PFHFileType::Other(_) = self.pfh_file_type { false }

        // If ANY of these bitmask is detected in the PackFile, disable all saving.
        else if self.bitmask.contains(PFHFlags::HAS_ENCRYPTED_DATA) || 
            self.bitmask.contains(PFHFlags::HAS_ENCRYPTED_INDEX) || 
            self.bitmask.contains(PFHFlags::HAS_EXTENDED_HEADER) { false }
        else {
            self.pfh_file_type == PFHFileType::Mod || 
            self.pfh_file_type == PFHFileType::Movie ||
            (is_editing_of_ca_packfiles_allowed && self.pfh_file_type.get_value() <= 2)
        }
    }

    /// This function returns a reference to the `PackedFile` with the provided path, if exists.
    pub fn get_ref_packed_file_by_path(&self, path: &[String]) -> Option<&PackedFile> {
        self.packed_files.iter().find(|x| x.get_path() == path)
    }

    /// This function returns a mutable reference to the `PackedFile` with the provided path, if exists.
    pub fn get_ref_mut_packed_file_by_path(&mut self, path: &[String]) -> Option<&mut PackedFile> {
        self.packed_files.iter_mut().find(|x| x.get_path() == path)
    }

    /// This function returns a reference of all the `PackedFiles` starting with the provided path.
    pub fn get_ref_packed_files_by_path_start(&self, path: &[String]) -> Vec<&PackedFile> {
        self.packed_files.iter().filter(|x| x.get_path().starts_with(path) && !path.is_empty() && x.get_path().len() > path.len()).collect()
    }

    /// This function returns a mutable reference of all the `PackedFiles` starting with the provided path.
    pub fn get_ref_mut_packed_files_by_path_start(&mut self, path: &[String]) -> Vec<&mut PackedFile> {
        self.packed_files.iter_mut().filter(|x| x.get_path().starts_with(path) && !path.is_empty() && x.get_path().len() > path.len()).collect()
    }
    
    /// This function returns a reference of all the `PackedFiles` ending with the provided path.
    pub fn get_ref_packed_files_by_path_end(&self, path: &[String]) -> Vec<&PackedFile> {
        self.packed_files.iter().filter(|x| x.get_path().ends_with(path) && !path.is_empty()).collect()
    }

    /// This function returns a mutable reference of all the `PackedFiles` ending with the provided path.
    pub fn get_ref_mut_packed_files_by_path_end(&mut self, path: &[String]) -> Vec<&mut PackedFile> {
        self.packed_files.iter_mut().filter(|x| x.get_path().ends_with(path) && !path.is_empty()).collect()
    }

    /// This function returns a copy of all `PackedFiles` in the provided `PackFile`.
    pub fn get_all_packed_files(&self) -> Vec<PackedFile> {
        self.packed_files.clone()
    }

    /// This function returns a reference of all the `PackedFiles` in the provided `PackFile`.
    pub fn get_ref_all_packed_files(&self) -> Vec<&PackedFile> {
        self.packed_files.iter().collect()
    }

    /// This function returns a mutable reference of all the `PackedFiles` in the provided `PackFile`.
    pub fn get_ref_mut_all_packed_files(&mut self) -> Vec<&mut PackedFile> {
        self.packed_files.iter_mut().collect()
    }

    /// This function removes, if exists, a `PackedFile` with the provided path from the `PackFile`.
    pub fn remove_packed_file_by_path(&mut self, path: &[String]) {
        if let Some(position) = self.packed_files.iter().position(|x| x.get_path() == path) {
            self.packed_files.remove(position);
        }
    }

    /// This function removes, if exists, all `PackedFile` starting with the provided path from the `PackFile`.
    pub fn remove_packed_files_by_path_start(&mut self, path: &[String]) {
        let positions: Vec<usize> = self.packed_files.iter()
            .enumerate()
            .filter(|x| x.1.get_path().starts_with(path) && !path.is_empty() && x.1.get_path().len() > path.len())
            .map(|x| x.0)
            .collect();
        for position in positions.iter().rev() {
            self.packed_files.remove(*position);
        }
    }

    /// This function removes, if exists, all `PackedFile` ending with the provided path from the `PackFile`.
    pub fn remove_packed_files_by_path_end(&mut self, path: &[String]) {
        let positions: Vec<usize> = self.packed_files.iter()
            .enumerate()
            .filter(|x| x.1.get_path().ends_with(path) && !path.is_empty())
            .map(|x| x.0)
            .collect();
        for position in positions.iter().rev() {
            self.packed_files.remove(*position);
        }
    }

    /// This function removes, if exists, all `PackedFile` of the provided types from the `PackFile`.
    pub fn remove_packed_files_by_type(&mut self, item_types: &[PathType]) -> Vec<PathType> {

        // We need to "clean" the selected path list to ensure we don't pass stuff already deleted.
        let item_types_clean = PathType::dedup(item_types);

        // Now we do some bitwise magic to get what type of selection combination we have.
        let mut contents: u8 = 0;
        for item_type in &item_types_clean {
            match item_type {
                PathType::File(_) => contents |= 1,
                PathType::Folder(_) => contents |= 2,
                PathType::PackFile => contents |= 4,
                PathType::None => contents |= 8,
            }
        }
        
        // Then we act, depending on the combination of items.
        match contents {

            // Any combination of files and folders.
            1 | 2 | 3 => {
                for item_type in &item_types_clean {
                    match item_type {
                        PathType::File(path) => self.remove_packed_file_by_path(path),
                        PathType::Folder(path) => self.remove_packed_files_by_path_start(path),
                        _ => unreachable!(),
                    } 
                }
            },

            // If the `PackFile` is selected, just delete everything.
            4 | 5 | 6 | 7 => self.remove_all_packedfiles(),

            // No paths selected, none selected, invalid path selected, or invalid value. 
            0 | 8..=255 => {},
        }

        // Return the list of deleted items so the caller can have a clean list to know what was really removed from the `PackFile`.
        item_types_clean
    }

    /// This function extracts, if exists, a `PackedFile` with the provided path from the `PackFile`.
    ///
    /// The destination path is always `destination_path/packfile_name/path_to_packedfile/packed_file`.
    pub fn extract_packed_file_by_path(&self, path: &[String], destination_path: &Path) -> Result<()> {
        match self.get_ref_packed_file_by_path(path) {
            Some(packed_file) => {

                // We get his internal path without his name.
                let mut internal_path = packed_file.get_path().to_vec();
                let file_name = internal_path.pop().unwrap();

                // Then, we join his internal path with his destination path, so we have his almost-full path (his final path without his name).
                // This way we can create the entire folder structure up to the file itself.
                let mut current_path = destination_path.to_path_buf().join(internal_path.iter().collect::<PathBuf>());
                DirBuilder::new().recursive(true).create(&current_path)?;

                // Finish the path and try to save the file to disk.
                current_path.push(&file_name);
                let mut file = BufWriter::new(File::create(&current_path)?);
                if file.write_all(&packed_file.get_data()?).is_err() {
                    return Err(ErrorKind::ExtractError(path.to_vec()))?;
                }
                Ok(())
            }
            None => Err(ErrorKind::PackedFileNotFound)?
        }
    }

    /// This function extract, if exists, all `PackedFile` of the provided types from the `PackFile` to disk.
    ///
    /// As this can fail for some files, and work for others, we return `Ok(amount_files_extracted)` only if all files were extracted correctly.
    /// If any of them failed, we return `Error` with a list of the paths that failed to get extracted.
    pub fn extract_packed_files_by_type(
        &self,
        item_types: &[PathType],
        extracted_path: &PathBuf,
    ) -> Result<u32> {

        // These variables are here to keep track of what we have extracted and what files failed.
        let mut files_extracted = 0;
        let mut error_files = vec![];

        // We need to "clean" the selected path list to ensure we don't pass stuff already extracted.
        let item_types_clean = PathType::dedup(item_types);

        // Now we do some bitwise magic to get what type of selection combination we have.
        let mut contents: u8 = 0;
        for item_type in &item_types_clean {
            match item_type {
                PathType::File(_) => contents |= 1,
                PathType::Folder(_) => contents |= 2,
                PathType::PackFile => contents |= 4,
                PathType::None => contents |= 8,
            }
        }

        // Then we act, depending on the combination of items.
        match contents {

            // Any combination of files and folders.
            1 | 2 | 3 => {

                // For folders we check each PackedFile to see if it starts with the folder's path (it's in the folder).
                // There should be no duplicates here thanks to the filters from before.
                for item_type in &item_types_clean {
                    match item_type {

                        // For individual `PackedFiles`, we extract them one by one.
                        PathType::File(path) => {
                            match self.extract_packed_file_by_path(path, extracted_path) {
                                Ok(_) => files_extracted += 1,
                                Err(_) => error_files.push(format!("{:?}", path)),
                            }
                        },

                        PathType::Folder(path) => {
                            for packed_file in self.get_ref_packed_files_by_path_start(path) {
                                match self.extract_packed_file_by_path(packed_file.get_path(), extracted_path) {
                                    Ok(_) => files_extracted += 1,
                                    Err(_) => error_files.push(format!("{:?}", path)),
                                }
                            }
                        },

                        _ => unreachable!(),
                    } 
                }            
            },

            // If the `PackFile` is selected, just extract it and everything will get extracted with it.
            4 | 5 | 6 | 7 => {

                // For each PackedFile we have, just extracted in the folder we got, under the PackFile's folder.
                for packed_file in self.get_ref_all_packed_files() {
                    match self.extract_packed_file_by_path(packed_file.get_path(), extracted_path) {
                        Ok(_) => files_extracted += 1,
                        Err(_) => error_files.push(format!("{:?}", packed_file.get_path())),
                    }
                }
            },

            // No paths selected, none selected, invalid path selected, or invalid value. 
            0 | 8..=255 => return Err(ErrorKind::NonExistantFile)?,
        }

        // If there is any error in the list, report it.
        if !error_files.is_empty() {
            let error_files_string = error_files.iter().map(|x| format!("<li>{}</li>", x)).collect::<Vec<String>>();
            return Err(ErrorKind::ExtractError(error_files_string))?
        }

        // If we reach this, return the amount of extracted files.
        Ok(files_extracted)
    }

    /// This function enables/disables compression in all `PackedFiles` inside the `PackFile`. Partial compression is not supported.
    pub fn toggle_compression(&mut self, enable: bool) {
        self.packed_files.iter_mut().for_each(|x| x.set_should_be_compressed(enable));
    }

    /// This function returns the notes contained within the provided `PackFile`.
    pub fn get_notes(&self) -> &Option<String> {
        &self.notes
    }

    /// This function saves your notes within the provided `PackFile`.
    pub fn set_notes(&mut self, notes: &Option<String>) {
        self.notes = notes.clone();
    }

    /// This function returns the timestamp of the provided `PackFile`.
    pub fn get_timestamp(&self) -> i64 {
        self.timestamp
    }

    /// This function sets the timestamp of the provided `PackFile`.
    pub fn set_timestamp(&mut self, timestamp: i64) {
        self.timestamp = timestamp;
    }

    /// This function returns the `PFHVersion` of the provided `PackFile`.
    pub fn get_pfh_version(&self) -> PFHVersion {
        self.pfh_version
    }

    /// This function sets the `PFHVersion` of the provided `PackFile`.
    pub fn set_pfh_version(&mut self, pfh_version: PFHVersion) {
        self.pfh_version = pfh_version;
    }

    /// This function returns the `PFHFileType` of the provided `PackFile`.
    pub fn get_pfh_file_type(&self) -> PFHFileType {
        self.pfh_file_type
    }

    /// This function sets the `PFHFileType` of the provided `PackFile`.
    pub fn set_pfh_file_type(&mut self, pfh_file_type: PFHFileType) {
        self.pfh_file_type = pfh_file_type;
    }

    /// This function returns the `Bitmask` of the provided `PackFile`.
    pub fn get_bitmask(&self) -> PFHFlags {
        self.bitmask
    }

    /// This function sets the `Bitmask` of the provided `PackFile`.
    pub fn set_bitmask(&mut self, bitmask: PFHFlags) {
        self.bitmask = bitmask;
    }

    /// This function remove all `PackedFiles` from a `PackFile`.
    pub fn remove_all_packedfiles(&mut self) {
        self.packed_files = vec![];
    }

    /// This function checks if a `PackedFile` with a certain path exists in a `PackFile`.
    pub fn packedfile_exists(&self, path: &[String]) -> bool {
        self.packed_files.iter().any(|x| x.get_path() == path)
    }

    /// This function checks if a folder with `PackedFiles` in it exists in a `PackFile`.
    pub fn folder_exists(&self, path: &[String]) -> bool {
        self.packed_files.iter().any(|x| x.get_path().starts_with(path) && !path.is_empty() && x.get_path().len() > path.len())
    }

    /// This function allows you to change the path of a `PackedFile` inside a `PackFile`.
    ///
    /// By default this append a `_number` to the file name in case of collision. If you want it to overwrite instead,
    /// pass `overwrite` as `true`. This can fail if you pass it an empty or reserved path, so make sure you check the result.
    ///
    /// We return the final destination path of the PackedFile, if it worked, or an error.
    pub fn move_packedfile(
        &mut self, 
        source_path: &[String], 
        destination_path: &[String],
        overwrite: bool,
    ) -> Result<Vec<String>> {

        // First, ensure we can move between the paths.
        let reserved_names = Self::get_reserved_packed_file_names();
        if destination_path.is_empty() { return Err(ErrorKind::EmptyInput)? }
        if source_path == destination_path { return Err(ErrorKind::PathsAreEqual)? }
        if reserved_names.contains(&destination_path.to_vec()) { return Err(ErrorKind::ReservedFiles)? }
        
        // We may need to modify his destination path if we're not overwriting so...
        let mut destination_path = destination_path.to_vec();

        // First, we check if BOTH, the source and destination, exist.
        let source_exists = self.packedfile_exists(source_path);
        let destination_exists = self.packedfile_exists(&destination_path);
        
        // If both exists, we do some name resolving:
        // - If we want to overwrite the destination file, we simply remove it.
        // - If not, we check until we find a free path using "_X". This also takes into account extensions, so "m.loc" will become "m_1.loc".
        if source_exists && destination_exists {
            if overwrite { self.remove_packed_file_by_path(&destination_path); }
            else {
                let name_current = destination_path.last().unwrap().to_owned(); 
                let name_splitted = name_current.split('.').collect::<Vec<&str>>();
                let name = name_splitted[0];
                let extension = if name_splitted.len() > 1 { name_splitted[1..].join(".") } else { "".to_owned() };
                for number in 0.. {
                    let name = if extension.is_empty() { format!("{}_{}", name, number) } else { format!("{}_{}.{}", name, number, extension) };
                    *destination_path.last_mut().unwrap() = name;
                    if !self.packedfile_exists(&destination_path) && !reserved_names.contains(&destination_path) {
                        break;
                    }
                }
            }
        }

        // Then just change the path of the `PackedFile` if exists. Return error if it doesn't.
        match self.get_ref_mut_packed_file_by_path(source_path) {
            Some(packed_file) => {
                packed_file.set_path(&destination_path)?; 
                Ok(destination_path) 
            },
            None => Err(ErrorKind::PackedFileNotFound)?
        }
    }

    /// This function allows you to change the name of a folder inside a `PackFile`.
    ///
    /// By default this append a `_number` to the file names in case of collision. If you want it to overwrite instead,
    /// pass `overwrite` as `true`. This can fail if you pass it an empty or reserved path, so make sure you check the result.
    ///
    /// We return the list of source/final paths of each moved PackedFile, if it worked, or an error.
    pub fn move_folder(
        &mut self, 
        source_path: &[String], 
        destination_path: &[String],
        overwrite: bool,
    ) -> Result<Vec<(Vec<String>, Vec<String>)>> {

        // First, ensure we can move between the paths.
        if source_path.is_empty() || destination_path.is_empty() { return Err(ErrorKind::EmptyInput)? }
        if source_path == destination_path { return Err(ErrorKind::PathsAreEqual)? }

        // Next... just get all the PackedFiles to move, and move them one by one.
        let mut successes = vec![];
        for packed_file_current_path in self.get_ref_packed_files_by_path_start(source_path).iter().map(|x| x.get_path().to_vec()).collect::<Vec<Vec<String>>>() {
            let new_path = packed_file_current_path.to_vec().splice(..source_path.len(), destination_path.iter().cloned()).collect::<Vec<String>>();
            if let Ok(new_path) = self.move_packedfile(&packed_file_current_path, &new_path, overwrite) {
                successes.push((packed_file_current_path, new_path))
            }
        }
        
        Ok(successes)
    }

    /// This function is used to rename one or more `PackedFile`/Folder inside a `PackFile`.
    ///
    /// This doesn't stop on failure. Instead, it continues. Then we return the list of paths that errored out.
    /// If `overwrite` is set to `true`, in case of destination `PackedFile` already existing, it'll be overwritten.
    /// If set to `false`, the file will be renamed to 'xxx_1', or the first number available. Extensions are taken 
    /// into account when doing this, so 'x.loc' will become 'x_1.loc'.
    pub fn rename_packedfiles(
        &mut self, 
        renaming_data: &[(PathType, String)], 
        overwrite: bool
    ) -> Vec<(PathType, String)> {

        let mut successes = vec![];
        for (item_type, new_name) in renaming_data {

            // Skip items with empty new names.
            if new_name.is_empty() { continue; }

            // We only allow to rename files and folders.
            match item_type {
                PathType::File(ref path) => {
                    let mut new_path = path.to_vec();
                    *new_path.last_mut().unwrap() = new_name.to_owned();
                    if let Ok(destination_path) = self.move_packedfile(path, &new_path, overwrite) {
                        successes.push((item_type.clone(), destination_path.last().unwrap().to_owned()));
                    }
                }
                
                PathType::Folder(ref path) => {
                    let mut new_path = path.to_vec();
                    *new_path.last_mut().unwrap() = new_name.to_owned();
                    if let Ok(result) = self.move_folder(path, &new_path, overwrite) {
                        result.iter().map(|x| (PathType::File(x.0.to_vec()), new_path.last().unwrap().to_owned())).for_each(|x| successes.push(x));
                    }
                }

                // PackFiles and errors are skipped.
                PathType::PackFile | PathType::None => continue,
            }
        }

        // Return the list of successes.
        successes
    }

    /// This function loads to memory the vanilla (made by CA) dependencies of a `PackFile`.
    fn load_vanilla_dependency_packfiles(packed_files: &mut Vec<PackedFile>) {

        // Get all the paths we need.
        let main_db_pack_paths = get_game_selected_db_pack_path(&*GAME_SELECTED.lock().unwrap());
        let main_loc_pack_paths = get_game_selected_loc_pack_path(&*GAME_SELECTED.lock().unwrap());

        // Get all the DB Tables from the main DB `PackFiles`, if it's configured.
        if let Some(paths) = main_db_pack_paths {
            if let Ok(pack_file) = PackFile::open_packfiles(&paths, true, false, false) {
                for packed_file in pack_file.get_ref_packed_files_by_path_start(&["db".to_owned()]) {

                    // Clone the PackedFile, and add it to the list.
                    let mut packed_file = packed_file.clone();
                    if let Ok(_) = packed_file.load_data() {
                        packed_files.push(packed_file);
                    }
                }
            }
        }

        // Get all the Loc PackedFiles from the main Loc `PackFiles`, if it's configured.
        if let Some(paths) = main_loc_pack_paths {
             if let Ok(pack_file) = PackFile::open_packfiles(&paths, true, false, false) {
                for packed_file in pack_file.get_ref_packed_files_by_path_end(&[".loc".to_owned()]) {

                    // Clone the PackedFile, and add it to the list.
                    let mut packed_file = packed_file.clone();
                    if let Ok(_) = packed_file.load_data() {
                        packed_files.push(packed_file);
                    }
                }
            }
        }
    }

    /// This function loads a `PackFile` as dependency, loading all his dependencies in the process.
    fn load_single_dependency_packfile(        
        packed_files: &mut Vec<PackedFile>, 
        packfile_name: &String, 
        already_loaded_dependencies: &mut Vec<String>,
        data_paths: &Option<Vec<PathBuf>>,
        contents_paths: &Option<Vec<PathBuf>>,
    ) {

        // First we load the content `PackFiles`.
        if let Some(ref paths) = contents_paths {
            if let Some(path) = paths.iter().find(|x| x.file_name().unwrap().to_string_lossy().to_string() == *packfile_name) {
                if let Ok(pack_file) = PackFile::open_packfiles(&[path.to_path_buf()], true, false, false) {

                    // Add the current `PackFile` to the done list, so we don't get into cyclic dependencies.
                    already_loaded_dependencies.push(packfile_name.to_owned());
                    pack_file.get_packfiles_list().iter().for_each(|x| Self::load_single_dependency_packfile(packed_files, x, already_loaded_dependencies, data_paths, contents_paths));
                    for packed_file in pack_file.get_ref_packed_files_by_path_start(&["db".to_owned()]) {
                        
                        // Clone the PackedFile, and add it to the list.
                        let mut packed_file = packed_file.clone();
                        if let Ok(_) = packed_file.load_data() {
                            packed_files.push(packed_file);
                        }
                    }

                    for packed_file in pack_file.get_ref_packed_files_by_path_end(&["loc".to_owned()]) {
                        
                        // Clone the PackedFile, and add it to the list.
                        let mut packed_file = packed_file.clone();
                        if let Ok(_) = packed_file.load_data() {
                            packed_files.push(packed_file);
                        }
                    }
                }
            }
        }

        // Then we load the data `PackFiles`.
        if let Some(ref paths) = data_paths {
            if let Some(path) = paths.iter().find(|x| x.file_name().unwrap().to_string_lossy().to_string() == *packfile_name) {
                if let Ok(pack_file) = PackFile::open_packfiles(&[path.to_path_buf()], true, false, false) {

                    // Add the current `PackFile` to the done list, so we don't get into cyclic dependencies.
                    already_loaded_dependencies.push(packfile_name.to_owned());
                    pack_file.get_packfiles_list().iter().for_each(|x| Self::load_single_dependency_packfile(packed_files, x, already_loaded_dependencies, data_paths, contents_paths));
                    for packed_file in pack_file.get_ref_packed_files_by_path_start(&["db".to_owned()]) {
                        
                        // Clone the PackedFile, and add it to the list.
                        let mut packed_file = packed_file.clone();
                        if let Ok(_) = packed_file.load_data() {
                            packed_files.push(packed_file);
                        }
                    }

                    for packed_file in pack_file.get_ref_packed_files_by_path_end(&["loc".to_owned()]) {
                        
                        // Clone the PackedFile, and add it to the list.
                        let mut packed_file = packed_file.clone();
                        if let Ok(_) = packed_file.load_data() {
                            packed_files.push(packed_file);
                        }
                    }
                }
            }
        }
    }

    /// This function loads to memory the custom (made by modders) dependencies of a `PackFile`.
    ///
    /// To avoid entering into an infinite loop while calling this recursively, we have to pass the
    /// list of loaded `PackFiles` each time we execute this.
    fn load_custom_dependency_packfiles(
        packed_files: &mut Vec<PackedFile>, 
        pack_file_names: &[String], 
    ) {
        
        let data_packs_paths = get_game_selected_data_packfiles_paths(&*GAME_SELECTED.lock().unwrap());
        let content_packs_paths = get_game_selected_content_packfiles_paths(&*GAME_SELECTED.lock().unwrap());
        let mut loaded_packfiles = vec![];

        pack_file_names.iter().for_each(|x| Self::load_single_dependency_packfile(packed_files, x, &mut loaded_packfiles, &data_packs_paths, &content_packs_paths));
    }

    /// This function loads to memory the dependencies of a `PackFile`. Well.... most of them.
    ///
    /// This function loads to memory all DB and Loc `PackedFiles` from vanilla `PackFiles` and
    /// from any `PackFile` the provided `PackFile` has as a dependency.
    pub fn load_all_dependency_packfiles(dependencies: &[String]) -> Vec<PackedFile> {

        // Create the empty list.
        let mut packed_files = vec![];

        Self::load_vanilla_dependency_packfiles(&mut packed_files);
        Self::load_custom_dependency_packfiles(&mut packed_files, dependencies);

        packed_files
    }

    /// This function allows you to open one or more `PackFiles`.
    ///
    /// The way it works:
    /// - If you open just one `PackFile`, it just calls the `PackFile::read()` function on it.
    /// - If you open multiple `PackFiles`, it merges them into one, taking care of conflicts the same way the game does.
    ///
    /// You can also make it ignore mod PackFiles, so it only open `PackFiles` released by CA, and can choose to lock it, 
    /// so the user cannot save it (avoiding the *"I tried to save and got an out-of-memory error!!!"* problem).
    pub fn open_packfiles(
        packs_paths: &[PathBuf],
        use_lazy_loading: bool,
        ignore_mods: bool,
        lock_packfile: bool
    ) -> Result<Self> {

        // If we just have one `PackFile`, just read it. No fancy logic needed. If you're an asshole and tried to break this
        // by passing it no paths, enjoy the error.
        if packs_paths.is_empty() { Err(ErrorKind::PackFileNoPathProvided)? }
        if packs_paths.len() == 1 { Self::read(&packs_paths[0], use_lazy_loading) }

        // Otherwise, read all of them into a *fake* `PackFile` and take care of the duplicated files like the game will do.
        else {

            // We have to ensure the paths are sorted and valid. Otherwise, this can go to hell.
            let mut packs_paths = packs_paths.iter().filter(|x| x.is_file()).collect::<Vec<&PathBuf>>();
            packs_paths.sort_by_key(|x| x.file_name().unwrap().to_string_lossy().to_string());

            let pfh_version = SUPPORTED_GAMES.get(&**GAME_SELECTED.lock().unwrap()).unwrap().pfh_version;
            let pfh_name = if ignore_mods { GAME_SELECTED.lock().unwrap().to_owned() } else { String::from("merged_mod.pack")};
            let mut pack_file = Self::new_with_name(&pfh_name, pfh_version);

            // Read all the `PackFiles`, one by one, and separate their files by `PFHFileType`.
            let mut boot_files = vec![];
            let mut release_files = vec![];
            let mut patch_files = vec![];
            let mut mod_files = vec![];
            let mut movie_files = vec![];
            for path in packs_paths {
                match Self::read(&path, use_lazy_loading) {
                    Ok(pack) => match pack.get_pfh_file_type() {
                        PFHFileType::Boot => boot_files.append(&mut pack.get_all_packed_files()),
                        PFHFileType::Release => release_files.append(&mut pack.get_all_packed_files()),
                        PFHFileType::Patch => patch_files.append(&mut pack.get_all_packed_files()),
                        PFHFileType::Mod => mod_files.append(&mut pack.get_all_packed_files()),
                        PFHFileType::Movie => movie_files.append(&mut pack.get_all_packed_files()),

                        // If we find an unknown one, return an error.
                        PFHFileType::Other(_) => return Err(ErrorKind::PackFileTypeUknown)?,
                    },
                    Err(error) => return Err(error) 
                }
            }

            // The priority in case of collision is:
            // - Same Type: First to come is the valid one.
            // - Different Type: Last to come is the valid one.
            boot_files.sort_by_key(|x| x.get_path().to_vec());
            boot_files.dedup_by_key(|x| x.get_path().to_vec());

            release_files.sort_by_key(|x| x.get_path().to_vec());
            release_files.dedup_by_key(|x| x.get_path().to_vec());

            patch_files.sort_by_key(|x| x.get_path().to_vec());
            patch_files.dedup_by_key(|x| x.get_path().to_vec());

            mod_files.sort_by_key(|x| x.get_path().to_vec());
            mod_files.dedup_by_key(|x| x.get_path().to_vec());

            movie_files.sort_by_key(|x| x.get_path().to_vec());
            movie_files.dedup_by_key(|x| x.get_path().to_vec());
        
            for packed_file in &boot_files {
                pack_file.add_packed_file(packed_file, true)?;
            }
            
            for packed_file in &release_files {
                pack_file.add_packed_file(packed_file, true)?;
            }
            
            for packed_file in &patch_files {
                pack_file.add_packed_file(packed_file, true)?;
            }
            
            for packed_file in &mod_files {
                pack_file.add_packed_file(packed_file, true)?;
            }
            
            for packed_file in &movie_files {
                pack_file.add_packed_file(packed_file, true)?;
            }

            // Set it as type "Other(200)", so we can easely identify it as fake in other places.
            // Used to lock the CA Files.
            if lock_packfile {
                pack_file.set_pfh_file_type(PFHFileType::Other(200));
            }
        
            // Return the new PackedFiles list.
            Ok(pack_file)
        }
    }

    /// This function reads the content of a PackFile into a `PackFile` struct.
    pub fn read(
        file_path: &PathBuf,
        use_lazy_loading: bool
    ) -> Result<Self> {

        // Check if what we received is even a `PackFile`.
        if !file_path.file_name().unwrap().to_string_lossy().to_string().ends_with(".pack") { Err(ErrorKind::OpenPackFileInvalidExtension)? }

        // Prepare the PackFile to be read and the virtual PackFile to be written.
        let mut pack_file = BufReader::new(File::open(&file_path)?);
        let pack_file_name = file_path.file_name().unwrap().to_string_lossy().to_string();
        let mut pack_file_decoded = Self::new();

        // First, we do some quick checkings to ensure it's a valid PackFile. 
        // 24 is the bare minimum that we need to check how a PackFile should be internally, so any file with less than that is not a valid PackFile.
        let pack_file_len = pack_file.get_ref().metadata()?.len();
        if pack_file_len < 24 { return Err(ErrorKind::PackFileHeaderNotComplete)? }

        // Create a little buffer to read the basic data from the header of the PackFile.
        let mut buffer = vec![0; 24];
        pack_file.read_exact(&mut buffer)?;

        // Start populating our decoded PackFile struct.
        pack_file_decoded.file_path = file_path.to_path_buf();
        pack_file_decoded.pfh_version = PFHVersion::get_version(&buffer.decode_string_u8(0, 4)?)?; 
        pack_file_decoded.pfh_file_type = PFHFileType::get_type(buffer.decode_integer_u32(4)? & 15);
        pack_file_decoded.bitmask = PFHFlags::from_bits_truncate(buffer.decode_integer_u32(4)? & !15);

        // Read the data about the indexes to use it later.
        let pack_file_count = buffer.decode_integer_u32(8)?;
        let pack_file_index_size = buffer.decode_integer_u32(12)?;
        let packed_file_count = buffer.decode_integer_u32(16)?;
        let packed_file_index_size = buffer.decode_integer_u32(20)?;

        // Depending on the data we got, prepare to read the header and ensure we have all the bytes we need.
        match pack_file_decoded.pfh_version {
            PFHVersion::PFH5 | PFHVersion::PFH4 => {
                if (pack_file_decoded.bitmask.contains(PFHFlags::HAS_EXTENDED_HEADER) && pack_file_len < 48) ||
                    (!pack_file_decoded.bitmask.contains(PFHFlags::HAS_EXTENDED_HEADER) && pack_file_len < 28) { return Err(ErrorKind::PackFileHeaderNotComplete)? }
                
                if pack_file_decoded.bitmask.contains(PFHFlags::HAS_EXTENDED_HEADER) { buffer = vec![0; 48]; }
                else { buffer = vec![0; 28]; }
            }

            PFHVersion::PFH3 => buffer = vec![0; 32],
            PFHVersion::PFH0 => buffer = vec![0; 24],
        }

        // Restore the cursor of the BufReader to 0, so we can read the full header in one go. The first 24 bytes are
        // already decoded but, for the sake of clarity in the positions of the rest of the header stuff, we do this.
        pack_file.seek(SeekFrom::Start(0))?;
        pack_file.read_exact(&mut buffer)?;

        // The creation time is a bit of an asshole. Depending on the PackFile Version/Id/Preamble, it uses a type, another or it doesn't exists.
        // Keep in mind that we store his raw value. If you want his legible value, you have to convert it yourself. PFH0 doesn't have it.
        pack_file_decoded.timestamp = match pack_file_decoded.pfh_version {
            PFHVersion::PFH5 | PFHVersion::PFH4 => i64::from(buffer.decode_integer_u32(24)?),
            PFHVersion::PFH3 => (buffer.decode_integer_i64(24)? / WINDOWS_TICK) - SEC_TO_UNIX_EPOCH,
            PFHVersion::PFH0 => 0
        };

        // Ensure the PackFile has all the data needed for the index. If the PackFile's data is encrypted 
        // and the PackFile is PFH5, due to how the encryption works, the data should start in a multiple of 8.
        let mut data_position = u64::from(buffer.len() as u32 + pack_file_index_size + packed_file_index_size);
        if pack_file_decoded.bitmask.contains(PFHFlags::HAS_ENCRYPTED_DATA) && 
            pack_file_decoded.bitmask.contains(PFHFlags::HAS_EXTENDED_HEADER) && 
            pack_file_decoded.pfh_version == PFHVersion::PFH5 {
            data_position = if (data_position % 8) > 0 { data_position + 8 - (data_position % 8) } else { data_position };
        }
        if pack_file_len < data_position { return Err(ErrorKind::PackFileIndexesNotComplete)? }

        // Create the buffers for the indexes data.
        let mut pack_file_index = vec![0; pack_file_index_size as usize];
        let mut packed_file_index = vec![0; packed_file_index_size as usize];

        // Get the data from both indexes to their buffers.
        pack_file.read_exact(&mut pack_file_index)?;
        pack_file.read_exact(&mut packed_file_index)?;

        // Read the PackFile Index.
        let mut pack_file_index_position: usize = 0;

        // First, we decode every entry in the PackFile index and store it. It's encoded in StringU8 terminated in 00,
        // so we just read them char by char until hitting 0, then decode the next one and so on.
        // NOTE: This doesn't deal with encryption, as we haven't seen any encrypted PackFile with data in this index.
        for _ in 0..pack_file_count {
            let pack_file_name = pack_file_index.decode_packedfile_string_u8_0terminated(pack_file_index_position, &mut pack_file_index_position)?;
            pack_file_decoded.pack_files.push(pack_file_name);
        }

        // Depending on the version of the PackFile and his bitmask, the PackedFile index has one format or another.
        let packed_file_index_path_offset = match pack_file_decoded.pfh_version {
            PFHVersion::PFH5 => {

                // If it has the extended header bit, is an Arena PackFile. These ones use a normal PFH4 index format for some reason.
                if pack_file_decoded.bitmask.contains(PFHFlags::HAS_EXTENDED_HEADER) {
                    if pack_file_decoded.bitmask.contains(PFHFlags::HAS_INDEX_WITH_TIMESTAMPS) { 8 } else { 4 }
                }

                // Otherwise, it's a Warhammer 2 PackFile. These ones have 4 bytes for the size, 4 for the timestamp and 1 for the compression.
                else if pack_file_decoded.bitmask.contains(PFHFlags::HAS_INDEX_WITH_TIMESTAMPS) { 9 } else { 5 }
            }

            // If it has the last modified date of the PackedFiles, we default to 8. Otherwise, we default to 4.
            PFHVersion::PFH4 => if pack_file_decoded.bitmask.contains(PFHFlags::HAS_INDEX_WITH_TIMESTAMPS) { 8 } else { 4 }

            // These are like PFH4, but the timestamp has 8 bytes instead of 4.
            PFHVersion::PFH3 => if pack_file_decoded.bitmask.contains(PFHFlags::HAS_INDEX_WITH_TIMESTAMPS) { 12 } else { 4 }

            // There isn't seem to be a bitmask in ANY PFH0 PackFile, so we will assume they didn't even use it back then.
            PFHVersion::PFH0 => 4
        };

        // Prepare the needed stuff to read the PackedFiles.
        let mut index_position: usize = 0;
        let pack_file = Arc::new(Mutex::new(pack_file));
        for packed_files_to_decode in (0..packed_file_count).rev() {

            // Get his size. If it's encrypted, decrypt it first.
            let size = if pack_file_decoded.bitmask.contains(PFHFlags::HAS_ENCRYPTED_INDEX) {
                let encrypted_size = packed_file_index.decode_integer_u32(index_position)?;
                decrypt_index_item_file_length(encrypted_size, packed_files_to_decode as u32)
            } else {
                packed_file_index.decode_integer_u32(index_position)?
            };

            // If we have the last modified date of the PackedFiles in the Index, get it. Otherwise, default to 0,
            // so we have something to write in case we want to enable them for our PackFile.
            let timestamp = if pack_file_decoded.bitmask.contains(PFHFlags::HAS_INDEX_WITH_TIMESTAMPS) {
                match pack_file_decoded.pfh_version {
                    PFHVersion::PFH5 | PFHVersion::PFH4 => {
                        let timestamp = i64::from(packed_file_index.decode_integer_u32(index_position + 4)?);
                        if pack_file_decoded.bitmask.contains(PFHFlags::HAS_ENCRYPTED_INDEX) {
                            i64::from(decrypt_index_item_file_length(timestamp as u32, packed_files_to_decode as u32))
                        } else { timestamp }
                    }

                    // We haven't found a single encrypted PFH3/PFH0 PackFile to test, so always assume these are unencrypted. Also, PFH0 doesn't seem to have a timestamp.
                    PFHVersion::PFH3 => (packed_file_index.decode_integer_i64(index_position + 4)? / WINDOWS_TICK) - SEC_TO_UNIX_EPOCH,
                    PFHVersion::PFH0 => 0,
                }
            } else { 0 };

            // Update his offset, and get his compression data if it has it.
            index_position += packed_file_index_path_offset;
            let is_compressed = if let PFHVersion::PFH5 = pack_file_decoded.pfh_version {
                if let Ok(true) = packed_file_index.decode_bool(index_position - 1) { true } 
                else { false }
            } else { false };
            
            // Get his path. Like the PackFile index, it's a StringU8 terminated in 00. We get it and split it in folders for easy use.
            let path = if pack_file_decoded.bitmask.contains(PFHFlags::HAS_ENCRYPTED_INDEX) {
                decrypt_index_item_filename(&packed_file_index[index_position..], size as u8, &mut index_position)
            }
            else { packed_file_index.decode_packedfile_string_u8_0terminated(index_position, &mut index_position)? };
            let path = path.split('\\').map(|x| x.to_owned()).collect::<Vec<String>>();

            // Once we are done, we create the and add it to the PackedFile list.
            let packed_file = PackedFile::read_from_data(
                path, 
                pack_file_name.to_string(),
                timestamp,
                is_compressed,
                if pack_file_decoded.bitmask.contains(PFHFlags::HAS_ENCRYPTED_DATA) { Some(pack_file_decoded.pfh_version) } else { None },
                PackedFileData::OnDisk(
                    pack_file.clone(), 
                    data_position, 
                    size,
                    is_compressed,
                    if pack_file_decoded.bitmask.contains(PFHFlags::HAS_ENCRYPTED_DATA) { Some(pack_file_decoded.pfh_version) } else { None },
                )
            );

            // If this is a notes PackedFile, save the notes and forget about the PackedFile. Otherwise, save the PackedFile.
            if packed_file.get_path() == ["frodos_biggest_secret.rpfm-notes"] {
                if let Ok(data) = packed_file.get_data() {
                    if let Ok(data) = data.decode_string_u8(0, data.len()) {
                        pack_file_decoded.notes = Some(data);
                    }
                }
            }
            else {
                pack_file_decoded.packed_files.push(packed_file);
            }

            // Then we move our data position. For encrypted files in PFH5 PackFiles (only ARENA) we have to start the next one in a multiple of 8.
            if pack_file_decoded.bitmask.contains(PFHFlags::HAS_ENCRYPTED_DATA) && 
                pack_file_decoded.bitmask.contains(PFHFlags::HAS_EXTENDED_HEADER) &&
                pack_file_decoded.pfh_version == PFHVersion::PFH5 {
                let padding = 8 - (size % 8);
                let padded_size = if padding < 8 { size + padding } else { size };
                data_position += u64::from(padded_size);
            }
            else { data_position += u64::from(size); }
        }

        // If at this point we have not reached the end of the PackFile, there is something wrong with it.
        // NOTE: Arena PackFiles have extra data at the end. If we detect one of those PackFiles, take that into account.
        if pack_file_decoded.pfh_version == PFHVersion::PFH5 && pack_file_decoded.bitmask.contains(PFHFlags::HAS_EXTENDED_HEADER) {
            if data_position + 256 != pack_file_len { return Err(ErrorKind::PackFileSizeIsNotWhatWeExpect(pack_file_len, data_position))? }
        }
        else if data_position != pack_file_len { return Err(ErrorKind::PackFileSizeIsNotWhatWeExpect(pack_file_len, data_position))? }

        // If we disabled lazy-loading, load every PackedFile to memory.
        if !use_lazy_loading { for packed_file in &mut pack_file_decoded.packed_files { packed_file.load_data()?; }}

        // Return our PackFile.
        Ok(pack_file_decoded)
    }

    /// This function tries to save a `PackFile` to a file in the filesystem.
    ///
    /// If no path is passed, the `PackFile` will be saved in his current path. 
    /// If a path is passed as `new_path` the `PackFile` will be saved in that path.
    pub fn save(&mut self, new_path: Option<PathBuf>) -> Result<()> {

        // If any of the problematic masks in the header is set or is one of CA's, return an error.
        if !self.is_editable(*SETTINGS.lock().unwrap().settings_bool.get("is_editing_of_ca_packfiles_allowed").unwrap()) { return Err(ErrorKind::PackFileIsNonEditable)? }

        // If we receive a new path, update it. Otherwise, ensure the file actually exists on disk.
        if let Some(path) = new_path { self.set_file_path(&path)?; }
        else if !self.get_file_path().is_file() { return Err(ErrorKind::PackFileIsNotAFile)? }
        
        // Before everything else, add the file for the notes if we have them. We'll remove it later, after the file has been saved.
        if let Some(note) = &self.notes {
            let mut data = vec![];
            data.encode_string_u8(&note);
            self.packed_files.push(PackedFile::read_from_vec(vec!["frodos_biggest_secret.rpfm-notes".to_owned()], self.get_file_name(), 0, false, data));
        }

        // For some bizarre reason, if the PackedFiles are not alphabetically sorted they may or may not crash the game for particular people.
        // So, to fix it, we have to sort all the PackedFiles here by path.
        // NOTE: This sorting has to be CASE INSENSITIVE. This means for "ac", "Ab" and "aa" it'll be "aa", "Ab", "ac".
        self.packed_files.sort_unstable_by(|a, b| a.get_path().join("\\").to_lowercase().cmp(&b.get_path().join("\\").to_lowercase()));
        
        // We ensure that all the data is loaded and in his right form (compressed/encrypted) before attempting to save.
        // We need to do this here because we need later on their compressed size.
        for packed_file in &mut self.packed_files { 
            packed_file.load_data()?;

            // Remember: first compress (only PFH5), then encrypt.
            let (data, is_compressed, is_encrypted, should_be_compressed, should_be_encrypted) = packed_file.get_data_and_info_from_memory()?;
            
            // If, in any moment, we enabled/disabled the PackFile compression, compress/decompress the PackedFile.
            if *should_be_compressed && !*is_compressed {
                *data = compress_data(&data)?;
                *is_compressed = true;
            }
            else if !*should_be_compressed && *is_compressed {
                *data = decompress_data(&data)?;
                *is_compressed = false;
            }

            // Encryption is not yet supported. Unencrypt everything.
            if is_encrypted.is_some() { 
                *data = decrypt_packed_file(&data);
                *is_encrypted = None;
                *should_be_encrypted = None;
            }
        }

        // First we encode the indexes and the data (just in case we compressed it).
        let mut pack_file_index = vec![];
        let mut packed_file_index = vec![];

        for pack_file in &self.pack_files {
            pack_file_index.extend_from_slice(pack_file.as_bytes());
            pack_file_index.push(0);
        }

        for packed_file in &self.packed_files {
            packed_file_index.encode_integer_u32(packed_file.get_size());

            // Depending on the version of the PackFile and his bitmask, the PackedFile index has one format or another.
            // In PFH5 case, we don't support saving encrypted PackFiles for Arena. So we'll default to Warhammer 2 format.
            match self.pfh_version {
                PFHVersion::PFH5 => {
                    if self.bitmask.contains(PFHFlags::HAS_INDEX_WITH_TIMESTAMPS) { packed_file_index.encode_integer_u32(packed_file.get_timestamp() as u32); }
                    if packed_file.get_should_be_compressed() { packed_file_index.push(1); } else { packed_file_index.push(0); } 
                }
                PFHVersion::PFH4 => {
                    if self.bitmask.contains(PFHFlags::HAS_INDEX_WITH_TIMESTAMPS) { packed_file_index.encode_integer_u32(packed_file.get_timestamp() as u32); }
                }
                PFHVersion::PFH3 => {
                    if self.bitmask.contains(PFHFlags::HAS_INDEX_WITH_TIMESTAMPS) { packed_file_index.encode_integer_i64(packed_file.get_timestamp()); }
                }

                // This one doesn't have timestamps, so we just skip this step.
                PFHVersion::PFH0 => {}
            }

            packed_file_index.append(&mut packed_file.get_path().join("\\").as_bytes().to_vec());
            packed_file_index.push(0);
        }

        // Create the file to save to, and save the header and the indexes.
        let mut file = BufWriter::new(File::create(&self.file_path)?);

        // Write the entire header.
        let mut header = vec![];
        header.encode_string_u8(&self.pfh_version.get_value());
        header.encode_integer_u32(self.bitmask.bits | self.pfh_file_type.get_value());
        header.encode_integer_u32(self.pack_files.len() as u32);
        header.encode_integer_u32(pack_file_index.len() as u32);
        header.encode_integer_u32(self.packed_files.len() as u32);
        header.encode_integer_u32(packed_file_index.len() as u32);

        // Update the creation time, then save it. PFH0 files don't have timestamp in the headers.
        self.timestamp = get_current_time();
        match self.pfh_version {
            PFHVersion::PFH5 | PFHVersion::PFH4 => header.encode_integer_u32(self.timestamp as u32),
            PFHVersion::PFH3 => header.encode_integer_i64((self.timestamp + SEC_TO_UNIX_EPOCH) * WINDOWS_TICK),
            PFHVersion::PFH0 => {}
        };

        // Write the indexes and the data of the PackedFiles. No need to keep the data, as it has been preloaded before.
        file.write_all(&header)?;
        file.write_all(&pack_file_index)?;
        file.write_all(&packed_file_index)?;
        for packed_file in &mut self.packed_files { 
            let data = packed_file.get_data()?;
            file.write_all(&data)?;
        }

        // Remove again the notes PackedFile, as that one is stored separated from the rest.
        self.remove_packed_file_by_path(&["frodos_biggest_secret.rpfm-notes".to_owned()]);

        // If nothing has failed, return success.
        Ok(())
    }
}

/// Implementaion of trait `Default` for `PackFile`.
impl Default for PackFile {

    /// This function creates a new empty `PackFile`.
    ///
    /// In reality, this just calls the `new()` function. It's just here for completeness.
    fn default() -> Self {
        Self::new()
    }
}

/// Implementation to create a `PackFileInfo` from a `PackFile`.
impl From<PackFile> for PackFileInfo {
    fn from(packfile: PackFile) -> Self {
        Self {
            file_path: packfile.file_path.to_path_buf(),
            pfh_version: packfile.pfh_version,
            pfh_file_type: packfile.pfh_file_type,
            bitmask: packfile.bitmask,
            timestamp: packfile.timestamp,
            compression_state: packfile.get_compression_state(),
        }
    }
}


