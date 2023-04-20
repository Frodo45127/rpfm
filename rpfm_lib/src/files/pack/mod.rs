//---------------------------------------------------------------------------//
// Copyright (c) 2017-2023 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Packs are a container-type file, used to contain Total War game files.

use bitflags::bitflags;
use getset::*;
use rayon::prelude::*;
use serde_derive::{Serialize, Deserialize};
use serde_json::{from_slice, to_string_pretty};
use itertools::Itertools;

use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs::File;
use std::io::{BufReader, BufWriter, Cursor, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::str::FromStr;

use crate::binary::{ReadBytes, WriteBytes};
use crate::compression::Compressible;
use crate::error::{RLibError, Result};
use crate::files::{Container, ContainerPath, Decodeable, DecodeableExtraData, Encodeable, EncodeableExtraData, FileType, Loc, RFile, RFileDecoded, table::DecodedData};
use crate::games::{GameInfo, pfh_file_type::PFHFileType, pfh_version::PFHVersion};
use crate::notes::Note;
use crate::utils::{current_time, last_modified_time_from_file};

#[cfg(test)]
mod pack_test;
mod pack_versions;

/// Extension used by Packs.
pub const EXTENSION: &str = ".pack";

/// Special Preamble/Id prefixing steam workshop files, for some reason.
const MFH_PREAMBLE: &str = "MFH"; // Weird format of some packs downloaded from Steam.

/// Path where Terry-generated map files end up.
const TERRY_MAP_PATH: &str = "terrain/tiles/battle/_assembly_kit";

/// This one is the name of the main BMD data file used by maps exported from Terry.
const DEFAULT_BMD_DATA: &str = "bmd_data.bin";

/// These three hints are necessary for the map patching function.
const FORT_PERIMETER_HINT: &[u8; 18] = b"AIH_FORT_PERIMETER";
const DEFENSIVE_HILL_HINT: &[u8; 18] = b"AIH_DEFENSIVE_HILL";
const SIEGE_AREA_NODE_HINT: &[u8; 19] = b"AIH_SIEGE_AREA_NODE";

pub const RESERVED_NAME_DEPENDENCIES_MANAGER: &str = "dependencies_manager.rpfm_reserved";
pub const RESERVED_NAME_EXTRA_PACKFILE: &str = "extra_packfile.rpfm_reserved";
pub const RESERVED_NAME_SETTINGS: &str = "settings.rpfm_reserved";
pub const RESERVED_NAME_SETTINGS_EXTRACTED: &str = "settings.rpfm_reserved.json";
pub const RESERVED_NAME_NOTES: &str = "notes.rpfm_reserved";
pub const RESERVED_NAME_NOTES_EXTRACTED: &str = "notes.rpfm_reserved.md";

/// This is the list of ***Reserved File Names***. They're file names used by RPFM for special purposes.
pub const RESERVED_RFILE_NAMES: [&str; 3] = [RESERVED_NAME_EXTRA_PACKFILE, RESERVED_NAME_SETTINGS, RESERVED_NAME_NOTES];

const AUTHORING_TOOL_CA: &str = "CA_TOOL";
const AUTHORING_TOOL_RPFM: &str = "RPFM";
const AUTHORING_TOOL_SIZE: u32 = 8;

bitflags! {

    /// This represents the bitmasks a Pack can have applied to his type.
    ///
    /// Keep in mind that this lib supports decoding Packs with any of these flags enabled,
    /// but it only supports enconding for the `HAS_INDEX_WITH_TIMESTAMPS` flag.
    #[derive(Serialize, Deserialize)]
    pub struct PFHFlags: u32 {

        /// Used to specify that the header of the Pack is extended by 20 bytes. Used in Arena.
        const HAS_EXTENDED_HEADER       = 0b0000_0001_0000_0000;

        /// Used to specify that the File Index is encrypted. Used in Arena.
        const HAS_ENCRYPTED_INDEX       = 0b0000_0000_1000_0000;

        /// Used to specify that the File Index contains a timestamp of every Pack.
        const HAS_INDEX_WITH_TIMESTAMPS = 0b0000_0000_0100_0000;

        /// Used to specify that the File Data is encrypted. Seen in `music.pack` Packs and in Arena.
        const HAS_ENCRYPTED_DATA        = 0b0000_0000_0001_0000;
    }
}

//---------------------------------------------------------------------------//
//                              Enum & Structs
//---------------------------------------------------------------------------//

/// Packs are a container-type file, used for "packing" all game assets into single files, to speed up disk reads.
///
/// Their format has passed through multiple iterations since empire, getting changes on almost all iterations,
/// like timestamps, encryption, compression,...
///
/// # Pack Structure
///
/// | Bytes  | Type                        | Data                                                                       |
/// | ------ | --------------------------- | -------------------------------------------------------------------------- |
/// | 20-384 | [PackHeader]                | Header of the Pack. Lenght depends on Pack version and flags enabled.      |
/// | *      | [Pack Index](#pack-index)   | Index containing the list of Packs this Pack depends on.                   |
/// | *      | [File Index](#file-index)   | Index containing the list of Files this Pack depends on                    |
/// | *      | [File Data](#file-data)     | Data of the files contained in this Pack.                                  |
/// | 256    | Appendix                    | Unknown data at the end of the Pack. Only seen in Arena's encrypted Packs. |
///
/// ## Pack Index
///
/// The Pack Index contains a list of Packs that will be force-loaded before this mod.
///
/// | Bytes | Type                     | Data            |
/// | ----- | ------------------------ | --------------- |
/// | *     | Null-terminated StringU8 | Pack file name. |
///
/// ## File Index
///
/// The File Index contains the metadata of the Files this Pack contains, in the same order their data is, further in the Pack.
///
/// | Bytes | Type                     | Data                                                                                                            |
/// | ----- | ------------------------ | --------------------------------------------------------------------------------------------------------------- |
/// | 4     | u32                      | Size of the file's data, in bytes.                                                                              |
/// | 8     | u64                      | Timestamp of the file, if the header has the HAS_INDEX_WITH_TIMESTAMPS flag enabled. Only in PFH2 and PFH3.     |
/// | 4     | u32                      | Truncated timestamp of the file, if the header has the HAS_INDEX_WITH_TIMESTAMPS flag enabled. Only since PFH4. |
/// | 1     | bool                     | If the file is compressed. Only since PFH5.                                                                     |
/// | *     | Null-terminated StringU8 | File's path within the Pack.                                                                                    |
///
/// ## File Data
///
/// The raw data of the files contained by this Pack, in the same order as their indexes. Not much to explain here.
///
#[derive(Debug, Clone, PartialEq, Getters, MutGetters, Setters, Default, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct Pack {

    /// The path of the Pack on disk, if exists. If not, then this should be empty.
    disk_file_path: String,

    /// The offset on the disk file the data of this Pack starts. Usually 0.
    disk_file_offset: u64,

    /// Timestamp from the moment this Pack was open. To check if the file was edited on disk while we had it open.
    local_timestamp: u64,

    /// If the files in this Pack should be compressed.
    compress: bool,

    /// Header data of this Pack.
    header: PackHeader,

    /// List of Packs this Pack requires to be loaded before himself when starting the game.
    ///
    /// In other places, we refer to this as the `Dependency List`.
    dependencies: Vec<String>,

    /// List of files this Pack contains.
    files: HashMap<String, RFile>,

    /// Notes added to the Pack. Exclusive of this lib.
    notes: PackNotes,

    /// Settings stored in the Pack itself, to be able to share them between installations.
    settings: PackSettings,
}

/// Header of a Pack, containing all the header-related info of said Pack.
///
/// # Header Structure.
///
/// | Bytes | Type                               | Data                                                                                                            |
/// | ----- | ---------------------------------- | --------------------------------------------------------------------------------------------------------------- |
/// | 8     | 00-Padded StringU8                 | Fake Preamble/Id of this Pack. Usually "MFH" and a bunch of 00. Only in old Steam Workshop files.               |
/// | 4     | StringU8                           | Preamble/Id of this Pack. Contains the "version" of this Pack.                                                  |
/// | 4     | u32                                | Pack Type + Bitwised flags for tweaking certain Pack configurations.                                            |
/// | 4     | u32                                | Amount of items in the Pack Index of this Pack.                                                                 |
/// | 4     | u32                                | Lenght in bytes of the Pack Index.                                                                              |
/// | 4     | u32                                | Amount of items in the File Index of this Pack.                                                                 |
/// | 4     | u32                                | Lenght in bytes of the File Index.                                                                              |
/// | 8     | u64                                | Timestamp when this Pack was last edited. Only in PFH2 and PFH3.                                                |
/// | 20    | Vec<u8>                            | Extended header data. Only if HAS_EXTENDED_HEADER flag is set.                                                  |
/// | 280   | [Subheader](#subheader-structure)  | Subheader data. Only since PFH6.                                                                                |
///
/// # Subheader Structure.
///
/// Subheader containing extra metadata for the Pack. Only in PFH6.
///
/// | Bytes | Type               | Data                                                                                         |
/// | ----- | ------------------ | -------------------------------------------------------------------------------------------- |
/// | 4     | u32                | Subheader marker. Marks the begining of the subheader. If missing, there's no subheader.     |
/// | 4     | u32                | Subheader version.                                                                           |
/// | 4     | u32                | Game version this Pack was done for.                                                         |
/// | 4     | u32                | Build number of the game version this Pack was done for.                                     |
/// | 8     | 00-Padded StringU8 | Tool that made this Pack.                                                                    |
/// | 256   | Vec<u8>            | Unused bytes.                                                                                |
///
#[derive(Debug, Clone, PartialEq, Eq, Getters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", set = "pub")]
pub struct PackHeader {

    /// The version of the Pack.
    pfh_version: PFHVersion,

    /// The type of the Pack.
    pfh_file_type: PFHFileType,

    /// The bitmasks applied to the Pack.
    bitmask: PFHFlags,

    /// The timestamp of the last time the Pack was saved.
    internal_timestamp: u64,

    /// Game version this Pack is intended for. This usually triggers the "outdated mod" warning in the launcher if it doesn't match the current exe version.
    game_version: u32,

    /// Build number of the game.
    build_number: u32,

    /// Tool that created the Pack. Max 8 characters, 00-padded.
    authoring_tool: String,

    /// Extra subheader data, in case it's used in the future.
    extra_subheader_data: Vec<u8>,
}

/// This struct hold Pack-specific settings.
///
/// Pack Settings are settings that are baked into a file in the Pack when saving,
/// so they can be shared across multiple instances.
#[derive(Clone, Debug, PartialEq, Eq, Default, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct PackSettings {

    /// For multi-line text.
    settings_text: BTreeMap<String, String>,

    /// For single-line text.
    settings_string: BTreeMap<String, String>,

    /// For bool values.
    settings_bool: BTreeMap<String, bool>,

    /// For integer values.
    settings_number: BTreeMap<String, i32>,
}

/// This struct hold Pack-specific notes, including both, Pack notes and file-specific notes.
///
/// These notes are baked into a file in the Pack when saving, so they can be shared across multiple instances.
#[derive(Clone, Debug, PartialEq, Eq, Default, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct PackNotes {

    /// Pack-specific notes. The're just a markdown text.
    pack_notes: String,

    /// File-specific notes.
    file_notes: HashMap<String, Vec<Note>>,
}

//---------------------------------------------------------------------------//
//                           Structs Implementations
//---------------------------------------------------------------------------//

impl Container for Pack {

    /// This method allows us to extract the metadata associated to the provided container as `.json` or `.md` files.
    ///
    /// [Pack] implementation extracts the [PackSettings] of the provided Pack and its associated notes.
    fn extract_metadata(&mut self, destination_path: &Path) -> Result<Vec<PathBuf>> {
        let mut paths = vec![];
        let mut data = vec![];
        data.write_all(to_string_pretty(&self.notes)?.as_bytes())?;
        data.extend_from_slice(b"\n"); // Add newline to the end of the file

        let path = destination_path.join(RESERVED_NAME_NOTES_EXTRACTED);
        paths.push(path.to_owned());
        let mut file = BufWriter::new(File::create(path)?);
        file.write_all(&data)?;
        file.flush()?;

        let mut data = vec![];
        data.write_all(to_string_pretty(&self.settings)?.as_bytes())?;
        data.extend_from_slice(b"\n"); // Add newline to the end of the file

        let path = destination_path.join(RESERVED_NAME_SETTINGS_EXTRACTED);
        paths.push(path.to_owned());
        let mut file = BufWriter::new(File::create(path)?);
        file.write_all(&data)?;
        file.flush()?;

        Ok(paths)
    }

    fn insert(&mut self, mut file: RFile) -> Result<Option<ContainerPath>> {

        // Filter out special files, so we only leave the normal files in.
        let path_container = file.path_in_container();
        let path = file.path_in_container_raw();
        if path == RESERVED_NAME_NOTES_EXTRACTED {
            self.notes = PackNotes::load(&file.encode(&None, false, false, true)?.unwrap())?;
            Ok(None)
        } else if path == RESERVED_NAME_SETTINGS_EXTRACTED {
            self.settings = PackSettings::load(&file.encode(&None, false, false, true)?.unwrap())?;
            Ok(None)
        }

        // If it's not filtered out, add it to the Pack.
        else {
            self.files.insert(path.to_owned(), file);
            Ok(Some(path_container))
        }

    }

    fn disk_file_path(&self) -> &str {
       &self.disk_file_path
    }

    fn files(&self) -> &HashMap<String, RFile> {
        &self.files
    }

    fn files_mut(&mut self) -> &mut HashMap<String, RFile> {
        &mut self.files
    }

    fn disk_file_offset(&self) -> u64 {
       self.disk_file_offset
    }

    fn internal_timestamp(&self) -> u64 {
       self.header.internal_timestamp
    }

    fn local_timestamp(&self) -> u64 {
       self.local_timestamp
    }

    /// This function allows you to *move* any RFile of folder of RFiles from one folder to another.
    ///
    /// It returns a list with all the new [ContainerPath].
    fn move_path(&mut self, source_path: &ContainerPath, destination_path: &ContainerPath) -> Result<Vec<(ContainerPath, ContainerPath)>> {
        match source_path {
            ContainerPath::File(source_path) => match destination_path {
                ContainerPath::File(destination_path) => {
                    if RESERVED_RFILE_NAMES.contains(&&**destination_path) {
                        return Err(RLibError::ReservedFiles);
                    }

                    if destination_path.is_empty() {
                        return Err(RLibError::EmptyDestiny);
                    }

                    let mut moved = self.files_mut()
                        .remove(source_path)
                        .ok_or_else(|| RLibError::FileNotFound(source_path.to_string()))?;

                    moved.set_path_in_container_raw(destination_path);

                    self.insert(moved).map(|x| match x {
                        Some(x) => vec![(ContainerPath::File(source_path.to_string()), x); 1],
                        None => Vec::with_capacity(0),
                    })
                },
                ContainerPath::Folder(_) => unreachable!("move_path_pack_1"),
            },
            ContainerPath::Folder(source_path) => match destination_path {
                ContainerPath::File(_) => unreachable!("move_path_pack_2"),
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
                        .filter_map(|x| self.files_mut().remove(x))
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
}

impl Decodeable for Pack {

    fn decode<R: ReadBytes>(data: &mut R, extra_data: &Option<DecodeableExtraData>) -> Result<Self> {
        Self::read(data, extra_data)
    }
}

impl Encodeable for Pack {

    fn encode<W: WriteBytes>(&mut self, buffer: &mut W, extra_data: &Option<EncodeableExtraData>) -> Result<()> {
        self.write(buffer, extra_data)
    }
}

/// Implementation of `Pack`.
impl Pack {

    /// This function creates a new empty Pack with a specific PFHVersion.
    pub fn new_with_version(pfh_version: PFHVersion) -> Self {
        let mut pack = Self::default();
        pack.header.pfh_version = pfh_version;
        pack
    }

    /// This function creates a new empty Pack with a name and a specific PFHVersion.
    pub fn new_with_name_and_version(name: &str, pfh_version: PFHVersion) -> Self {
        let mut pack = Self::default();
        pack.header.pfh_version = pfh_version;
        pack.disk_file_path = name.to_owned();
        pack
    }

    /// This function tries to read a `Pack` from raw data.
    ///
    /// If `lazy_load` is false, the data of all the files inside the `Pack` will be preload to memory.
    fn read<R: ReadBytes>(data: &mut R, extra_data: &Option<DecodeableExtraData>) -> Result<Self> {
        let extra_data = extra_data.as_ref().ok_or(RLibError::DecodingMissingExtraData)?;

        // If we're reading from a file on disk, we require a valid path.
        // If we're reading from a file on memory, we don't need a valid path.
        let disk_file_path = match extra_data.disk_file_path {
            Some(path) => {
                let file_path = PathBuf::from_str(path).map_err(|_|RLibError::DecodingMissingExtraDataField("disk_file_path".to_owned()))?;
                if file_path.is_file() {
                    path.to_owned()
                } else {
                    return Err(RLibError::DecodingMissingExtraData)
                }
            }
            None => String::new()
        };

        let disk_file_offset = extra_data.disk_file_offset;
        let disk_file_size = if extra_data.data_size > 0 { extra_data.data_size } else { data.len()? };
        let timestamp = extra_data.timestamp;
        let is_encrypted = extra_data.is_encrypted;

        // If we don't have a path, or the file is encrypted, we can't lazy-load.
        let lazy_load = !disk_file_path.is_empty() && !is_encrypted && extra_data.lazy_load;

        // First, we do some quick checks to ensure it's a valid Pack.
        // A valid Pack, bare and empty, needs at least 24 bytes, regardless of game or type.
        let data_len = disk_file_size;
        if data_len < 24 {
            return Err(RLibError::PackHeaderNotComplete);
        }

        // Check if it has the weird steam-only header, and skip it if found.
        let start = if data.read_string_u8(3)? == MFH_PREAMBLE { 8 } else { 0 };
        data.seek(SeekFrom::Current(-3))?;
        data.seek(SeekFrom::Current(start))?;

        // Create the default Pack and start populating it.
        let mut pack = Self {
            disk_file_path,
            disk_file_offset,
            local_timestamp: timestamp,
            ..Default::default()
        };

        pack.header.pfh_version = PFHVersion::version(&data.read_string_u8(4)?)?;

        let pack_type = data.read_u32()?;
        pack.header.pfh_file_type = PFHFileType::try_from(pack_type & 15)?;
        pack.header.bitmask = PFHFlags::from_bits_truncate(pack_type & !15);

        // Each Pack version has its own read function, to avoid breaking support for older Packs
        // when implementing support for a new Pack version.
        let expected_data_len = match pack.header.pfh_version {
            PFHVersion::PFH6 => pack.read_pfh6(data, extra_data)?,
            PFHVersion::PFH5 => pack.read_pfh5(data, extra_data)?,
            PFHVersion::PFH4 => pack.read_pfh4(data, extra_data)?,
            PFHVersion::PFH3 => pack.read_pfh3(data, extra_data)?,
            PFHVersion::PFH2 => pack.read_pfh2(data, extra_data)?,
            PFHVersion::PFH0 => pack.read_pfh0(data, extra_data)?,
        };

        // Remove the reserved files from the Pack and read them properly.
        if let Some(mut notes) = pack.files.remove(RESERVED_NAME_NOTES) {
            notes.load()?;
            let data = notes.cached()?;

            // Migration logic from 3.X to 4.X notes: iff we detect old notes, we don't fail.
            // We instead generate a new 4.X note and fill the pack message with the old 3.X note.
            match PackNotes::load(data) {
                Ok(notes) => pack.notes = notes,
                Err(_) => {
                    let len = data.len();
                    let mut data = Cursor::new(data);
                    pack.notes = PackNotes::default();
                    pack.notes.pack_notes = data.read_string_u8(len)?;
                }
            }
        }

        if let Some(mut settings) = pack.files.remove(RESERVED_NAME_SETTINGS) {
            settings.load()?;
            let data = settings.cached()?;
            pack.settings = PackSettings::load(data)?;
        }

        // If at this point we have not reached the end of the Pack, there is something wrong with it.
        // NOTE: Arena Packs have extra data at the end. If we detect one of those Packs, take that into account.
        if pack.header.pfh_version == PFHVersion::PFH5 && pack.header.bitmask.contains(PFHFlags::HAS_EXTENDED_HEADER) {
            if expected_data_len + 256 != data_len { return Err(RLibError::DecodingMismatchSizeError(data_len as usize, expected_data_len as usize)) }
        }
        else if expected_data_len != data_len { return Err(RLibError::DecodingMismatchSizeError(data_len as usize, expected_data_len as usize)) }

        // Guess the file's types. Do this here because this can be very slow and here we can do it in paralell.
        pack.files.par_iter_mut().map(|(_, file)| file.guess_file_type()).collect::<Result<()>>()?;

        // If we disabled lazy-loading, load every File to memory.
        if !lazy_load {
            pack.files.par_iter_mut().try_for_each(|(_, file)| file.load())?;
        }

        // Return our Pack.
        Ok(pack)
    }

    /// This function writes a `Pack` into the provided buffer.
    fn write<W: WriteBytes>(&mut self, buffer: &mut W, extra_data: &Option<EncodeableExtraData>) -> Result<()> {
        let test_mode = if let Some(extra_data) = extra_data {
            extra_data.test_mode
        } else {
            false
        };

        if !test_mode {

            // Only do this in non-vanilla files.
            if self.header.pfh_file_type == PFHFileType::Mod || self.header.pfh_file_type == PFHFileType::Movie {

                // Save notes, if needed.
                let mut data = vec![];
                data.write_all(to_string_pretty(&self.notes)?.as_bytes())?;
                let file = RFile::new_from_vec(&data, FileType::Text, 0, RESERVED_NAME_NOTES);
                self.files.insert(RESERVED_NAME_NOTES.to_owned(), file);

                // Saving Pack settings.
                let mut data = vec![];
                data.write_all(to_string_pretty(&self.settings)?.as_bytes())?;
                let file = RFile::new_from_vec(&data, FileType::Text, 0, RESERVED_NAME_SETTINGS);
                self.files.insert(RESERVED_NAME_SETTINGS.to_owned(), file);
            }
        }

        match self.header.pfh_version {
            PFHVersion::PFH6 => self.write_pfh6(buffer, extra_data)?,
            PFHVersion::PFH5 => self.write_pfh5(buffer, extra_data)?,
            PFHVersion::PFH4 => self.write_pfh4(buffer, extra_data)?,
            PFHVersion::PFH3 => self.write_pfh3(buffer, extra_data)?,
            PFHVersion::PFH2 => self.write_pfh2(buffer, extra_data)?,
            PFHVersion::PFH0 => self.write_pfh0(buffer, extra_data)?,
        }

        // Remove again the reserved Files.
        self.remove(&ContainerPath::File(RESERVED_NAME_NOTES.to_owned()));
        self.remove(&ContainerPath::File(RESERVED_NAME_SETTINGS.to_owned()));

        // If nothing has failed, return success.
        Ok(())
    }

    //-----------------------------------------------------------------------//
    //                        Convenience functions
    //-----------------------------------------------------------------------//

    /// This function reads and returns all CA Packs for the provided game merged as one, for easy manipulation.
    ///
    /// This needs a [GameInfo] to get the Packs from, and a game path to search the Packs on.
    pub fn read_and_merge_ca_packs(game: &GameInfo, game_path: &Path) -> Result<Self> {
        let paths = game.ca_packs_paths(game_path)?;
        let mut pack = Self::read_and_merge(&paths, true, true)?;

        // Make sure it's not mod type.
        pack.header_mut().set_pfh_file_type(PFHFileType::Release);
        Ok(pack)
    }

    /// Convenience function to open multiple Packs as one, taking care of overwriting files when needed.
    ///
    /// If this function receives only one path, it works as a normal read_from_disk function. If it receives none, an error will be returned.
    pub fn read_and_merge(pack_paths: &[PathBuf], lazy_load: bool, ignore_mods: bool) -> Result<Self> {
        if pack_paths.is_empty() {
            return Err(RLibError::NoPacksProvided);
        }

        let mut extra_data = DecodeableExtraData {
            lazy_load,
            ..Default::default()
        };

        // If we only got one path, just decode the Pack on it.
        if pack_paths.len() == 1 {
            let mut data = BufReader::new(File::open(&pack_paths[0])
                .map_err(|error| RLibError::IOErrorPath(Box::new(RLibError::IOError(error)), pack_paths[0].to_path_buf()))?);
            let path_str = pack_paths[0].to_string_lossy().replace('\\', "/");

            extra_data.set_disk_file_path(Some(&path_str));
            extra_data.set_timestamp(last_modified_time_from_file(data.get_ref()).unwrap());

            return Self::read(&mut data, &Some(extra_data))
        }

        // Generate a new empty Pack to act as merged one.
        let mut pack_new = Pack::default();
        let mut packs = pack_paths.par_iter()
            .map(|path| {
                let mut data = BufReader::new(File::open(path)
                    .map_err(|error| RLibError::IOErrorPath(Box::new(RLibError::IOError(error)), pack_paths[0].to_path_buf()))?);
                let path_str = path.to_string_lossy().replace('\\', "/");

                let mut extra_data = extra_data.to_owned();
                extra_data.set_disk_file_path(Some(&path_str));
                extra_data.set_timestamp(last_modified_time_from_file(data.get_ref())?);

                Self::read(&mut data, &Some(extra_data))
            }).collect::<Result<Vec<Pack>>>()?;

        // Sort the decoded Packs by name and type, so each type has their own Packs also sorted by name.
        packs.sort_by_key(|pack| pack.disk_file_path.to_owned());
        packs.sort_by_key(|pack| pack.header.pfh_file_type as u8);

        packs.iter_mut()
            .filter(|pack| {
                if let PFHFileType::Mod = pack.header.pfh_file_type {
                    !ignore_mods
                } else { true }
            })
            .for_each(|pack| {
                pack_new.files_mut().extend(pack.files().clone())
            });

        // Fix the dependencies of the merged pack.
        let pack_names = packs.iter().map(|pack| pack.disk_file_name()).collect::<Vec<_>>();
        let mut dependencies = packs.iter()
            .flat_map(|pack| pack.dependencies()
                .iter()
                .filter(|dependency| !pack_names.contains(dependency))
                .cloned()
                .collect::<Vec<_>>())
            .collect::<Vec<_>>();
        dependencies.sort();
        dependencies.dedup();
        pack_new.set_dependencies(dependencies);

        // Fix the pack version.
        pack_new.set_pfh_file_type(packs[0].pfh_file_type());

        Ok(pack_new)
    }

    /// Convenience function to easily save a Pack to disk.
    ///
    /// If a path is provided, the Pack will be saved to that path. Otherwise, it'll use whatever path it had set before.
    pub fn save(&mut self, path: Option<&Path>, game_info: &GameInfo, extra_data: &Option<EncodeableExtraData>) -> Result<()> {
        if let Some(path) = path {
            self.disk_file_path = path.to_string_lossy().to_string();
        }

        // Before truncating the file, make sure we loaded everything to memory.
        self.files.iter_mut().try_for_each(|(_, file)| file.load())?;

        let mut file = BufWriter::new(File::create(&self.disk_file_path)?);
        let extra_data = if extra_data.is_some() {
            extra_data.clone()
        } else {
            Some(EncodeableExtraData::new_from_game_info(game_info))
        };

        self.encode(&mut file, &extra_data)
    }

    //-----------------------------------------------------------------------//
    //                           Getters & Setters
    //-----------------------------------------------------------------------//

    /// This function returns the current PFH Version of the provided Pack.
    pub fn pfh_version(&self) -> PFHVersion {
        *self.header.pfh_version()
    }

    /// This function returns the current PFH File Type of the provided Pack.
    pub fn pfh_file_type(&self) -> PFHFileType {
        *self.header.pfh_file_type()
    }

    /// This function returns the bitmask applied to the provided Pack.
    pub fn bitmask(&self) -> PFHFlags {
        *self.header.bitmask()
    }

    /// This function returns the timestamp of the last time the Pack was saved.
    pub fn internal_timestamp(&self) -> u64 {
        *self.header.internal_timestamp()
    }

    /// This function returns the Game version this Pack is intended for.
    pub fn game_version(&self) -> u32 {
        *self.header.game_version()
    }

    /// This function returns the build number of the game this Pack is intended for.
    pub fn build_number(&self) -> u32 {
        *self.header.build_number()
    }

    /// This function returns the tool that created the Pack. Max 8 characters, 00-padded.
    pub fn authoring_tool(&self) -> &str {
        self.header.authoring_tool()
    }

    /// This function returns the Extra Subheader Data, if any.
    pub fn extra_subheader_data(&self) -> &[u8] {
        self.header.extra_subheader_data()
    }
/*
    /// This function changes the path of the Pack.
    ///
    /// This can fail if you pass it an empty path.
    pub fn set_file_path(&mut self, path: &Path) -> Result<()> {
        if path.components().count() == 0 { return Err(ErrorKind::EmptyInput.into()) }
        self.file_path = path.to_path_buf();

        // We have to change the name of the Pack in all his `Files` too.
        let file_name = self.disk_file_name();
        self.files.iter_mut().for_each(|x| x.get_ref_mut_raw().set_packfile_name(&file_name));
        Ok(())
    }*/

    /// This function sets the current Pack PFH Version to the provided one.
    pub fn set_pfh_version(&mut self, version: PFHVersion) {
        self.header.set_pfh_version(version);
    }

    /// This function sets the current Pack PFH File Type to the provided one.
    pub fn set_pfh_file_type(&mut self, file_type: PFHFileType) {
        self.header.set_pfh_file_type(file_type);
    }

    /// This function sets the current Pack bitmask to the provided one.
    pub fn set_bitmask(&mut self, bitmask: PFHFlags) {
        self.header.set_bitmask(bitmask);
    }

    /// This function sets the current Pack timestamp to the provided one.
    pub fn set_internal_timestamp(&mut self, timestamp: u64) {
        self.header.set_internal_timestamp(timestamp);
    }

    /// This function sets the game version (as in X.Y.Z) this Pack is for.
    pub fn set_game_version(&mut self, game_version: u32) {
        self.header.set_game_version(game_version);
    }

    /// This function sets the build number this Pack is for.
    pub fn set_build_number(&mut self, build_number: u32) {
        self.header.set_build_number(build_number);
    }

    /// This function sets the authoring tool that last edited this Pack.
    pub fn set_authoring_tool(&mut self, authoring_tool: &str) {
        self.header.set_authoring_tool(authoring_tool.to_string());
    }

    /// This function sets the Extra Subheader Data of the Pack.
    pub fn set_extra_subheader_data(&mut self, extra_subheader_data: &[u8]) {
        self.header.set_extra_subheader_data(extra_subheader_data.to_vec());
    }

    //-----------------------------------------------------------------------//
    //                             Util functions
    //-----------------------------------------------------------------------//

    /// This function allows to toggle CA Authoring tool spoofing for this Pack.
    ///
    /// Passing spoof as false will reset the Authoring Tool to the default one.
    pub fn spoof_ca_authoring_tool(&mut self, spoof: bool) {
        if spoof {
            self.header.set_authoring_tool(AUTHORING_TOOL_CA.to_string());
        } else {
            self.header.set_authoring_tool(AUTHORING_TOOL_RPFM.to_string());
        }
    }

    /// This function returns if the Pack is compressible or not.
    pub fn is_compressible(&self) -> bool {
        matches!(self.header.pfh_version, PFHVersion::PFH6 | PFHVersion::PFH5)
    }

    /// This function is used to generate all loc entries missing from a Pack into a missing.loc file.
    pub fn generate_missing_loc_data(&mut self) -> Result<Option<ContainerPath>> {

        let db_tables = self.files_by_type(&[FileType::DB]);
        let loc_tables = self.files_by_type(&[FileType::Loc]);
        let mut missing_trads_file = Loc::new(false);

        let loc_keys_from_memory = loc_tables.par_iter().filter_map(|rfile| {
            if let Ok(RFileDecoded::Loc(table)) = rfile.decoded() {
                Some(table.data(&None).unwrap().iter().filter_map(|x| {
                    if let DecodedData::StringU16(data) = &x[0] {
                        Some(data.to_owned())
                    } else {
                        None
                    }
                }).collect::<HashSet<String>>())
            } else { None }
        }).flatten().collect::<HashSet<String>>();

        let missing_trads_file_table_data = db_tables.par_iter().filter_map(|rfile| {
            if let Ok(RFileDecoded::DB(table)) = rfile.decoded() {
                let definition = table.definition();
                let loc_fields = definition.localised_fields();
                if !loc_fields.is_empty() {
                    let table_data = table.data(&None).unwrap();
                    let table_name = table.table_name_without_tables();

                    // Get the keys, which may be concatenated. We get them IN THE ORDER THEY ARE IN THE BINARY FILE.
                    let localised_order = definition.localised_key_order();
                    let mut new_rows = vec![];

                    for row in table_data.iter() {
                        for loc_field in loc_fields {
                            let key = localised_order.iter().map(|pos| row[*pos as usize].data_to_string()).join("");

                            // Key can be empty due to incomplete schema. Ignore those.
                            if !key.is_empty() {
                                let loc_key = format!("{}_{}_{}", table_name, loc_field.name(), key);

                                if loc_keys_from_memory.get(&*loc_key).is_none() {
                                    let mut new_row = missing_trads_file.new_row();
                                    new_row[0] = DecodedData::StringU16(loc_key);
                                    new_row[1] = DecodedData::StringU16("PLACEHOLDER".to_owned());
                                    new_rows.push(new_row);
                                }
                            }
                        }
                    }

                    return Some(new_rows)
                }
            }
            None
        }).flatten().collect::<Vec<Vec<DecodedData>>>();

        // Save the missing translations to a missing_locs.loc file.
        let _ = missing_trads_file.set_data(&missing_trads_file_table_data);
        if !missing_trads_file_table_data.is_empty() {
            let packed_file = RFile::new_from_decoded(&RFileDecoded::Loc(missing_trads_file), 0,  "text/missing_locs.loc");
            Ok(self.insert(packed_file)?)
        } else {
            Ok(None)
        }
    }

    /// This function is used to patch Warhammer I & II Siege map packs so their AI actually works.
    ///
    /// This also removes the useless xml files left by Terry in the Pack.
    pub fn patch_siege_ai(&mut self) -> Result<(String, Vec<ContainerPath>)> {

        // If there are no files, directly return an error.
        if self.files().is_empty() {
            return Err(RLibError::PatchSiegeAIEmptyPack)
        }

        let mut files_patched = 0;
        let mut files_to_delete: Vec<ContainerPath> = vec![];
        let mut multiple_defensive_hill_hints = false;

        // We only need to change stuff inside the map folder, so we only check the maps in that folder.
        for file in self.files_by_path_mut(&ContainerPath::Folder(TERRY_MAP_PATH.to_owned()), true) {
            let path = file.path_in_container_raw();
            let name = &path[path.rfind('/').unwrap_or(0)..];

            // The files we need to process are `bmd_data.bin` and all the `catchment_` files the map has.
            if name == DEFAULT_BMD_DATA || (name.starts_with("catchment_") && name.ends_with(".bin")) {
                file.load()?;
                let data = file.cached_mut()?;

                // The patching process it's simple. First, we check if there is SiegeAI stuff in the file by checking if there is an Area Node.
                // If we find one, we check if there is a defensive hill hint in the same file, and patch it if there is one.
                if data.windows(19).any(|window: &[u8]|window == SIEGE_AREA_NODE_HINT) {
                    if let Some(index) = data.windows(18).position(|window: &[u8]|window == DEFENSIVE_HILL_HINT) {
                        data.splice(index..index + 18, FORT_PERIMETER_HINT.iter().cloned());
                        files_patched += 1;
                    }

                    // If there is more than one defensive hill in one file, is a valid file, but we want to warn the user about it.
                    if data.windows(18).any(|window: &[u8]|window == DEFENSIVE_HILL_HINT) {
                        multiple_defensive_hill_hints = true;
                    }
                }
            }

            // All xml in this folder are useless, so we mark them all for deletion.
            else if name.ends_with(".xml") {
                files_to_delete.push(ContainerPath::File(file.path_in_container_raw().to_string()));
            }
        }

        // If there are files to delete, we delete them.
        files_to_delete.iter().for_each(|x| { self.remove(x); });

        // If we didn't found any file to patch or delete, return an error.
        if files_patched == 0 && files_to_delete.is_empty() { Err(RLibError::PatchSiegeAINoPatchableFiles) }

        // TODO: make this more.... `fluent`.
        // If we found files to delete, but not to patch, return a message reporting it.
        else if files_patched == 0 {
            Ok((format!("No file suitable for patching has been found.\n{} files deleted.", files_to_delete.len()), files_to_delete))
        }

        // If we found multiple defensive hill hints... it's ok, but we return a warning.
        else if multiple_defensive_hill_hints {

            // The message is different depending on the amount of files deleted.
            if files_to_delete.is_empty() {
                Ok((format!("{files_patched} files patched.\nNo file suitable for deleting has been found.\
                \n\n\
                WARNING: Multiple Defensive Hints have been found and we only patched the first one.\
                 If you are using SiegeAI, you should only have one Defensive Hill in the map (the \
                 one acting as the perimeter of your fort/city/castle). Due to SiegeAI being present, \
                 in the map, normal Defensive Hills will not work anyways, and the only thing they do \
                 is interfere with the patching process. So, if your map doesn't work properly after \
                 patching, delete all the extra Defensive Hill Hints. They are the culprit."), files_to_delete))
            }
            else {
                Ok((format!("{} files patched.\n{} files deleted.\
                \n\n\
                WARNING: Multiple Defensive Hints have been found and we only patched the first one.\
                 If you are using SiegeAI, you should only have one Defensive Hill in the map (the \
                 one acting as the perimeter of your fort/city/castle). Due to SiegeAI being present, \
                 in the map, normal Defensive Hills will not work anyways, and the only thing they do \
                 is interfere with the patching process. So, if your map doesn't work properly after \
                 patching, delete all the extra Defensive Hill Hints. They are the culprit.",
                files_patched, files_to_delete.len()), files_to_delete))
            }
        }

        // If no files to delete were found, but we got files patched, report it.
        else if files_to_delete.is_empty() {
            Ok((format!("{files_patched} files patched.\nNo file suitable for deleting has been found."), files_to_delete))
        }

        // And finally, if we got some files patched and some deleted, report it too.
        else {
            Ok((format!("{} files patched.\n{} files deleted.", files_patched, files_to_delete.len()), files_to_delete))
        }
    }
}

impl PackNotes {

    /// This function tries to load the notes from the current Pack and return them.
    pub fn load(data: &[u8]) -> Result<Self> {
        from_slice(data).map_err(From::from)
    }

    /// This function returns all notes afecting the provided path.
    pub fn notes_by_path(&self, path: &str) -> Vec<Note> {
        let path_lower = path.to_lowercase();
        self.file_notes()
            .iter()
            .filter(|(path, _)| path.is_empty() || path_lower.starts_with(*path) || &&path_lower == path)
            .flat_map(|(_, notes)| notes.to_vec())
            .collect()
    }

    /// This function adds a note for an specific path.
    ///
    /// Note: for DB tables, notes are added for all tables with the same table name instead of specific tables.
    pub fn add_note(&mut self, mut note: Note) -> Note {

        // For tables, share notes between same-type tables.
        let mut path = note.path().to_lowercase();
        if path.starts_with("db/") {
            let mut new_path = path.split('/').collect::<Vec<_>>();
            if new_path.len() == 3 {
                new_path.pop();
            }
            path = new_path.join("/");
        }
        note.set_path(path.to_owned());

        match self.file_notes_mut().get_mut(&path) {
            Some(notes) => {

                // If it already has an id greater than 0, we're trying to replace and existing note if found.
                if *note.id() == 0 {
                    let id = notes.iter().map(|note| note.id()).max().unwrap();
                    note.set_id(*id + 1);
                } else {
                    notes.retain(|x| x.id() != note.id());
                }

                notes.push(note.clone());
                note
            },
            None => {
                let notes = vec![note.clone()];
                self.file_notes_mut().insert(path.to_owned(), notes);
                note
            }
        }
    }

    /// This function deletes a note with the specified path and id.
    pub fn delete_note(&mut self, path: &str, id: u64) {
        let path_lower = path.to_lowercase();

        if let Some(notes) = self.file_notes_mut().get_mut(&path_lower) {
            notes.retain(|note| note.id() != &id);
            if notes.is_empty() {
                self.file_notes_mut().remove(&path_lower);
            }
        }
    }
}

impl PackSettings {

    /// This function tries to load the settings from the current Pack and return them.
    pub fn load(data: &[u8]) -> Result<Self> {
        from_slice(data).map_err(From::from)
    }

    /// This function returns the provided string setting, if found.
    pub fn setting_string(&self, key: &str) -> Option<&String> {
        self.settings_string.get(key)
    }

    /// This function returns the provided text setting (multiline string), if found.
    pub fn setting_text(&self, key: &str) -> Option<&String> {
        self.settings_text.get(key)
    }

    /// This function returns the provided bool setting, if found.
    pub fn setting_bool(&self, key: &str) -> Option<&bool> {
        self.settings_bool.get(key)
    }

    /// This function returns the provided numeric setting, if found.
    pub fn setting_number(&self, key: &str) -> Option<&i32> {
        self.settings_number.get(key)
    }

    /// This function sets the string setting provided with the value you passed.
    ///
    /// If the value already existed, it gets overwritten.
    pub fn set_setting_string(&mut self, key: &str, value: &str) {
        self.settings_string.insert(key.to_owned(), value.to_owned());
    }

    /// This function sets the text (multiline string) setting provided with the value you passed.
    ///
    /// If the value already existed, it gets overwritten.
    pub fn set_setting_text(&mut self, key: &str, value: &str) {
        self.settings_text.insert(key.to_owned(), value.to_owned());
    }

    /// This function sets the bool setting provided with the value you passed.
    ///
    /// If the value already existed, it gets overwritten.
    pub fn set_setting_bool(&mut self, key: &str, value: bool) {
        self.settings_bool.insert(key.to_owned(), value);
    }

    /// This function sets the numeric setting provided with the value you passed.
    ///
    /// If the value already existed, it gets overwritten.
    pub fn set_setting_number(&mut self, key: &str, value: i32) {
        self.settings_number.insert(key.to_owned(), value);
    }

    // TODO: Move this to rpfm_extensions.
    pub fn diagnostics_files_to_ignore(&self) -> Option<Vec<(String, Vec<String>, Vec<String>)>> {
        self.settings_text.get("diagnostics_files_to_ignore").map(|files_to_ignore| {
            let files = files_to_ignore.split('\n').collect::<Vec<&str>>();

            // Ignore commented out rows.
            files.iter().filter_map(|x| {
                if !x.starts_with('#') {
                    let path = x.splitn(3, ';').collect::<Vec<&str>>();
                    if path.len() == 3 {
                        Some((path[0].to_string(), path[1].split(',').filter_map(|y| if !y.is_empty() { Some(y.to_owned()) } else { None }).collect::<Vec<String>>(), path[2].split(',').filter_map(|y| if !y.is_empty() { Some(y.to_owned()) } else { None }).collect::<Vec<String>>()))
                    } else if path.len() == 2 {
                        Some((path[0].to_string(), path[1].split(',').filter_map(|y| if !y.is_empty() { Some(y.to_owned()) } else { None }).collect::<Vec<String>>(), vec![]))
                    } else if path.len() == 1 {
                        Some((path[0].to_string(), vec![], vec![]))
                    } else {
                        None
                    }
                } else {
                    None
                }
            }).collect::<Vec<(String, Vec<String>, Vec<String>)>>()
        })
    }
}

impl Default for PackHeader {
    fn default() -> Self {
        Self {
            pfh_version: Default::default(),
            pfh_file_type: Default::default(),
            bitmask: Default::default(),
            internal_timestamp: Default::default(),
            game_version: Default::default(),
            build_number: Default::default(),
            authoring_tool: AUTHORING_TOOL_RPFM.to_owned(),
            extra_subheader_data: Default::default(),
        }
    }
}

impl Default for PFHFlags {
    fn default() -> Self {
        Self::empty()
    }
}
