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

use std::fs::File;
use std::io::{prelude::*, BufReader, BufWriter, SeekFrom, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use rpfm_error::{ErrorKind, Result};

use crate::common::*;
use crate::common::coding_helpers::*;
use crate::packfile::compression::*;
use crate::packfile::crypto::*;
use crate::packfile::packedfile::*;

mod compression;
mod crypto;
pub mod packedfile;

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
#[derive(Debug)]
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
    pub fn list_packed_files(&self) -> Vec<Vec<String>> {
        self.packed_files.iter().map(|x| x.get_path().to_vec()).collect()
    }

    /// This function adds one or more `PackedFiles` to an existing `PackFile`. In case of conflict, the PackedFiles are overwritten.
    ///
    /// This function is not a "do and forget" type of function. It returns the paths of the PackedFiles which got added succesfully,
    /// and it's up to you to check it against the paths you provided to ensure all the PackedFiles got added correctly.
    pub fn add_packed_files(&mut self, packed_files: &[PackedFile]) -> Vec<Vec<String>> {
        let reserved_packed_file_names = Self::get_reserved_packed_file_names();
        let mut new_paths = vec![];
        for packed_file in packed_files {

            // If it's one of the reserved paths, ignore the file.
            if reserved_packed_file_names.contains(&packed_file.get_path().to_vec()) { continue; }
            new_paths.push(packed_file.get_path().to_vec());
            match self.packed_files.iter().position(|x| x.get_path() == packed_file.get_path()) {
                Some(index) => self.packed_files[index] = packed_file.clone(),
                None => self.packed_files.push(packed_file.clone()),
            }           
        }
        new_paths
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
        else if self.bitmask.contains(PFHFlags::HAS_ENCRYPTED_DATA) || self.bitmask.contains(PFHFlags::HAS_ENCRYPTED_INDEX) || self.bitmask.contains(PFHFlags::HAS_EXTENDED_HEADER) { false }

        // These types are always editable.
        else if self.pfh_file_type == PFHFileType::Mod || self.pfh_file_type == PFHFileType::Movie { true }

        // If the "Allow Editing of CA PackFiles" is enabled, these types are also enabled.
        else if is_editing_of_ca_packfiles_allowed && self.pfh_file_type.get_value() <= 2 { true }

        // Otherwise, always return false.
        else { false }
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

    /// This function allows you to change the path of a `PackedFile` inside a `PackFile`. It there is already a file using the new Path, it gets overwritten.
    ///
    /// This can fail if you pass it an empty path, so make sure you check the result.
    pub fn move_packedfile(&mut self, source_path: &[String], destination_path: &[String]) -> Result<()> {
        if destination_path.len() == 0 { return Err(ErrorKind::EmptyInput)? }
        
        // If there is already a `PackedFile` with that name, and the source exists (it wont fail later for not finding it), we remove it.
        let source_exists = self.packedfile_exists(source_path);
        let destination_exists = self.packedfile_exists(destination_path);
        if source_exists && destination_exists {
            self.remove_packed_file_by_path(destination_path);
        }

        if let Some(packed_file) = self.get_ref_mut_packed_file_by_path(source_path) {
            packed_file.set_path(destination_path)?;
        }
        Ok(())
    }

    /// This function checks if a `PackedFile` with a certain path exists in a `PackFile`.
    pub fn packedfile_exists(&self, path: &[String]) -> bool {
        self.packed_files.iter().find(|x| x.get_path() == path).is_some()
    }

    /// This function checks if a folder with `PackedFiles` in it exists in a `PackFile`.
    pub fn folder_exists(&self, path: &[String]) -> bool {
        self.packed_files.iter().find(|x| x.get_path().starts_with(path) && !path.is_empty() && x.get_path().len() > path.len()).is_some()
    }

    /// This function reads the content of a PackFile into a `PackFile` struct.
    pub fn read(
        file_path: PathBuf,
        use_lazy_loading: bool
    ) -> Result<Self> {

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
        pack_file_decoded.file_path = file_path;
        pack_file_decoded.pfh_version = PFHVersion::get_version(&decode_string_u8(&buffer[..4])?)?; 
        pack_file_decoded.pfh_file_type = PFHFileType::get_type(decode_integer_u32(&buffer[4..8])? & 15);
        pack_file_decoded.bitmask = PFHFlags::from_bits_truncate(decode_integer_u32(&buffer[4..8])? & !15);

        // Read the data about the indexes to use it later.
        let pack_file_count = decode_integer_u32(&buffer[8..12])?;
        let pack_file_index_size = decode_integer_u32(&buffer[12..16])?;
        let packed_file_count = decode_integer_u32(&buffer[16..20])?;
        let packed_file_index_size = decode_integer_u32(&buffer[20..24])?;

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
            PFHVersion::PFH5 | PFHVersion::PFH4 => decode_integer_u32(&buffer[24..28])? as i64,
            PFHVersion::PFH3 => (decode_integer_i64(&buffer[24..32])? / WINDOWS_TICK) - SEC_TO_UNIX_EPOCH,
            PFHVersion::PFH0 => 0
        };

        // Ensure the PackFile has all the data needed for the index. If the PackFile's data is encrypted 
        // and the PackFile is PFH5, due to how the encryption works, the data should start in a multiple of 8.
        let mut data_position = (buffer.len() as u32 + pack_file_index_size + packed_file_index_size) as u64;
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
            let pack_file_name = decode_string_u8_0terminated(&pack_file_index[pack_file_index_position..], &mut pack_file_index_position)?;
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
                let encrypted_size = decode_integer_u32(&packed_file_index[index_position..(index_position + 4)])?;
                decrypt_index_item_file_length(encrypted_size, packed_files_to_decode as u32)
            } else {
                decode_integer_u32(&packed_file_index[index_position..index_position + 4])?
            };

            // If we have the last modified date of the PackedFiles in the Index, get it. Otherwise, default to 0,
            // so we have something to write in case we want to enable them for our PackFile.
            let timestamp = if pack_file_decoded.bitmask.contains(PFHFlags::HAS_INDEX_WITH_TIMESTAMPS) {
                match pack_file_decoded.pfh_version {
                    PFHVersion::PFH5 | PFHVersion::PFH4 => {
                        let timestamp = decode_integer_u32(&packed_file_index[(index_position + 4)..(index_position + 8)])? as i64;
                        if pack_file_decoded.bitmask.contains(PFHFlags::HAS_ENCRYPTED_INDEX) {
                            decrypt_index_item_file_length(timestamp as u32, packed_files_to_decode as u32) as i64
                        } else { timestamp }
                    }

                    // We haven't found a single encrypted PFH3/PFH0 PackFile to test, so always assume these are unencrypted. Also, PFH0 doesn't seem to have a timestamp.
                    PFHVersion::PFH3 => (decode_integer_i64(&packed_file_index[(index_position + 4)..(index_position + 12)])? / WINDOWS_TICK) - SEC_TO_UNIX_EPOCH,
                    PFHVersion::PFH0 => 0,
                }
            } else { 0 };

            // Update his offset, and get his compression data if it has it.
            index_position += packed_file_index_path_offset;
            let is_compressed = if let PFHVersion::PFH5 = pack_file_decoded.pfh_version {
                if let Ok(true) = decode_bool(packed_file_index[(index_position - 1)]) { true } 
                else { false }
            } else { false };
            
            // Get his path. Like the PackFile index, it's a StringU8 terminated in 00. We get it and split it in folders for easy use.
            let path = if pack_file_decoded.bitmask.contains(PFHFlags::HAS_ENCRYPTED_INDEX) {
                decrypt_index_item_filename(&packed_file_index[index_position..], size as u8, &mut index_position)
            }
            else { decode_string_u8_0terminated(&packed_file_index[index_position..], &mut index_position)? };
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
            if packed_file.get_path() == &["frodos_biggest_secret.rpfm-notes"] {
                if let Ok(data) = packed_file.get_data() {
                    if let Ok(data) = decode_string_u8(&data) {
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
                data_position += padded_size as u64;
            }
            else { data_position += size as u64; }
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
    pub fn save(&mut self) -> Result<()> {

        // Before everything else, add the file for the notes if we have them. We'll remove it later, after the file has been saved.
        if let Some(data) = &self.notes {
            self.packed_files.push(PackedFile::read_from_vec(vec!["frodos_biggest_secret.rpfm-notes".to_owned()], self.get_file_name(), 0, false, encode_string_u8(&data)));
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
            packed_file_index.extend_from_slice(&encode_integer_u32(packed_file.get_size()));

            // Depending on the version of the PackFile and his bitmask, the PackedFile index has one format or another.
            // In PFH5 case, we don't support saving encrypted PackFiles for Arena. So we'll default to Warhammer 2 format.
            match self.pfh_version {
                PFHVersion::PFH5 => {
                    if self.bitmask.contains(PFHFlags::HAS_INDEX_WITH_TIMESTAMPS) { packed_file_index.extend_from_slice(&encode_integer_u32(packed_file.get_timestamp() as u32)); }
                    if packed_file.get_should_be_compressed() { packed_file_index.push(1); } else { packed_file_index.push(0); } 
                }
                PFHVersion::PFH4 => {
                    if self.bitmask.contains(PFHFlags::HAS_INDEX_WITH_TIMESTAMPS) { packed_file_index.extend_from_slice(&encode_integer_u32(packed_file.get_timestamp() as u32)); }
                }
                PFHVersion::PFH3 => {
                    if self.bitmask.contains(PFHFlags::HAS_INDEX_WITH_TIMESTAMPS) { packed_file_index.extend_from_slice(&encode_integer_i64(packed_file.get_timestamp())); }
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
        file.write_all(&encode_string_u8(&self.pfh_version.get_value()))?;
        file.write_all(&encode_integer_u32(self.bitmask.bits | self.pfh_file_type.get_value()))?;
        file.write_all(&encode_integer_u32(self.pack_files.len() as u32))?;
        file.write_all(&encode_integer_u32(pack_file_index.len() as u32))?;
        file.write_all(&encode_integer_u32(self.packed_files.len() as u32))?;
        file.write_all(&encode_integer_u32(packed_file_index.len() as u32))?;

        // Update the creation time, then save it. PFH0 files don't have timestamp in the headers.
        self.timestamp = get_current_time();
        match self.pfh_version {
            PFHVersion::PFH5 | PFHVersion::PFH4 => file.write_all(&encode_integer_u32(self.timestamp as u32))?,
            PFHVersion::PFH3 => file.write_all(&encode_integer_i64((self.timestamp + SEC_TO_UNIX_EPOCH) * WINDOWS_TICK))?,
            PFHVersion::PFH0 => {}
        };

        // Write the indexes and the data of the PackedFiles. No need to keep the data, as it has been preloaded before.
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
