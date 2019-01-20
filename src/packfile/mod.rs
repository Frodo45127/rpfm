// In this file are all the Structs and Impls required to decode and encode the PackFiles.

use bitflags::bitflags;

use std::path::PathBuf;
use std::io::prelude::*;
use std::io::{ BufReader, BufWriter, Read, Write, SeekFrom };
use std::fs::File;
use std::sync::{Arc, Mutex};

use crate::common::*;
use crate::common::coding_helpers::*;
use crate::error::{ErrorKind, Result};
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

/// These are the types the PackFiles can have.
const FILE_TYPE_BOOT: u32 = 0;
const FILE_TYPE_RELEASE: u32 = 1;
const FILE_TYPE_PATCH: u32 = 2;
const FILE_TYPE_MOD: u32 = 3;
const FILE_TYPE_MOVIE: u32 = 4;
bitflags! {

    /// This represents the bitmasks a PackFile can have applied to his type.
    ///
    /// The possible bitmasks are:
    /// - `HAS_EXTENDED_HEADER`: Used to specify that the header of the PackFile is extended by 20 bytes. Used in Arena.
    /// - `HAS_ENCRYPTED_INDEX`: Used to specify that the PackedFile Index is encrypted. Used in Arena.
    /// - `HAS_INDEX_WITH_TIMESTAMPS`: Used to specify that the PackedFile Index contains a timestamp of evey PackFile.
    /// - `HAS_ENCRYPTED_DATA`: Used to specify that the PackedFile's data is encrypted. Seen in `music.pack` PackFiles and in Arena.
    pub struct PFHFlags: u32 {
        const HAS_EXTENDED_HEADER       = 0b0000_0001_0000_0000;
        const HAS_ENCRYPTED_INDEX       = 0b0000_0000_1000_0000;
        const HAS_INDEX_WITH_TIMESTAMPS = 0b0000_0000_0100_0000;
        const HAS_ENCRYPTED_DATA        = 0b0000_0000_0001_0000;
    }
}

/// This enum represents the **Version** of a PackFile.
///
/// The possible values are:
/// - `PFH5`: Used in Warhammer 2 and Arena.
/// - `PFH4`: Used in Warhammer 1, Attila, Rome 2, and Thrones of Brittania.
/// - `PFH3`: Used in Shogun 2.
/// - `PFH0`: Used in Napoleon and Empire.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PFHVersion {
    PFH5,
    PFH4,
    PFH3,
    PFH0
}

/// This enum represents the **Type** of a PackFile. 
///
/// The possible types are, in the order they'll load when the game starts (their numeric value is the number besides them):
/// - `Boot` **(0)**: Used in CA PackFiles, not useful for modding.
/// - `Release` **(1)**: Used in CA PackFiles, not useful for modding.
/// - `Patch` **(2)**: Used in CA PackFiles, not useful for modding.
/// - `Mod` **(3)**: Used for mods. PackFiles of this type are only loaded in the game if they are enabled in the Mod Manager/Launcher.
/// - `Movie` **(4)**: Used in CA PackFiles and for some special mods. Unlike `Mod` PackFiles, these ones always get loaded.
/// - `Other(u32)`: Wildcard for any type that doesn't fit in any of the other categories. The type's value is stored in the Variant.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PFHFileType {
    Boot,
    Release,
    Patch,
    Mod,
    Movie,
    Other(u32),
}

/// This `Struct` stores the data of the PackFile in memory, along with some extra data needed to manipulate the PackFile.
///
/// It stores the following data from the header:
/// - `file_path`: the path of the PackFile on disk.
/// - `pfh_version`: the version/id of the PackFile. Usually it's PFHX.
/// - `pfh_file_type`: the type of the PackFile.
/// - `bitmask`: the bitmasks applied to this PackFile.
/// - `timestamp`: that `Last Modified Date` of the PackFile. It's usually that or all zeros.
///
/// And the following data from the *data* part of the PackFile:
/// - `pack_files`: the list of PackFiles in the PackFile Index.
/// - `packed_files`: the list of PackedFiles inside this PackFile.
/// - `empty_folders`: the list of empty folder in the PackFile.
#[derive(Debug)]
pub struct PackFile {
    pub file_path: PathBuf,
    pub pfh_version: PFHVersion,
    pub pfh_file_type: PFHFileType,
    pub bitmask: PFHFlags,
    pub timestamp: i64,

    pub pack_files: Vec<String>,
    pub packed_files: Vec<PackedFile>,
    pub empty_folders: Vec<Vec<String>>
}

/// This `Struct` is a reduced version of the `PackFile` Struct, used to pass data to the UI.
#[derive(Debug)]
pub struct PackFileUIData {
    pub file_path: PathBuf,
    pub pfh_version: PFHVersion,
    pub pfh_file_type: PFHFileType,
    pub bitmask: PFHFlags,
    pub timestamp: i64,
}

/// Implementation of PFHFileType.
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

    /// This function returns the PackFile's Type or an Error if the Type is invalid.
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

/// Implementation of PFHVersion.
impl PFHVersion {

    /// This function returns the PackFile's **Preamble** or **Id** (his 4 first bytes) in `u32` format.
    pub fn get_value(&self) -> &str {
        match *self {
            PFHVersion::PFH5 => PFH5_PREAMBLE,
            PFHVersion::PFH4 => PFH4_PREAMBLE,
            PFHVersion::PFH3 => PFH3_PREAMBLE,
            PFHVersion::PFH0 => PFH0_PREAMBLE,
        }
    }

    /// This function returns the PackFile's Version or an Error if the version is invalid.
    pub fn get_version(version: &str) -> Result<Self> {
        match version {
            PFH5_PREAMBLE => Ok(PFHVersion::PFH5),
            PFH4_PREAMBLE => Ok(PFHVersion::PFH4),
            PFH3_PREAMBLE => Ok(PFHVersion::PFH3),
            PFH0_PREAMBLE => Ok(PFHVersion::PFH0),
            _ => Err(ErrorKind::PackFileIsNotAPackFile)?,
        }
    }
}


/// Implementation of `PackFile`.
impl PackFile {

    /// This function creates a new empty `PackFile`. This is used for creating a *dummy* PackFile.
    pub fn new() -> Self {
        Self {
            file_path: PathBuf::new(),
            pfh_version: PFHVersion::PFH5,
            pfh_file_type: PFHFileType::Mod,
            bitmask: PFHFlags::empty(),
            timestamp: 0,

            pack_files: vec![],
            packed_files: vec![],
            empty_folders: vec![]
        }
    }

    /// This function creates a new empty `PackFile` with a name and an specific id.
    pub fn new_with_name(file_name: String, pfh_version: PFHVersion) -> Self {
        let mut file_path = PathBuf::new();
        file_path.set_file_name(file_name);
        Self {
            file_path,
            pfh_version,
            bitmask: PFHFlags::empty(),
            pfh_file_type: PFHFileType::Mod,
            timestamp: 0,

            pack_files: vec![],
            packed_files: vec![],
            empty_folders: vec![]
        }
    }

    /// This function replaces the current `PackFile List` with a new one.
    ///
    /// It requires:
    /// - `&mut self`: the PackFile we are going to manipulate.
    /// - `pack_files`: a Vec<String> we are going to use as new list.
    pub fn save_packfiles_list(&mut self, pack_files: Vec<String>) {
        self.pack_files = pack_files;
    }

    /// This function adds one or more `PackedFiles` to an existing `PackFile`.
    ///
    /// It requires:
    /// - `&mut self`: the PackFile we are going to manipulate.
    /// - `packed_files`: a Vec<PackedFile> we are going to add.
    pub fn add_packedfiles(&mut self, mut packed_files: Vec<PackedFile>) {
        self.packed_files.append(&mut packed_files);
    }

    /// This function returns the name of the PackedFile. If it's empty, it's a dummy PackFile. 
    pub fn get_file_name(&self) -> String {
        match self.file_path.file_name() {
            Some(s) => s.to_string_lossy().to_string(),
            None => String::new()
        }
    }

    /// This function copies the data needed by the UI to load a PackFile.
    pub fn create_ui_data(&self) -> PackFileUIData {
        PackFileUIData {
            file_path: self.file_path.to_path_buf(),
            pfh_version: self.pfh_version,
            pfh_file_type: self.pfh_file_type,
            bitmask: self.bitmask,
            timestamp: self.timestamp,
        }
    }

    /// This function returns if the PackFile is editable or not, depending on the type of the PackFile.
    /// Basically, if the PackFile is not one of the known types OR it has any of the `pack_file_type` bitmasks
    /// as true, this'll return false. Use it to disable saving functions for PackFiles we can read but not
    /// save. Also, if the `is_editing_of_ca_packfiles_allowed` argument is false, return false for everything
    /// except types "Mod" and "Movie".
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

    /// This function removes a PackedFile from a PackFile.
    ///
    /// It requires:
    /// - `&mut self`: the PackFile we are going to manipulate.
    /// - `index`: the index of the PackedFile we want to remove from the PackFile.
    pub fn remove_packedfile(&mut self, index: usize) {
        self.packed_files.remove(index);
    }

    /// This function remove all PackedFiles from a PackFile.
    ///
    /// It requires:
    /// - `&mut self`: the PackFile we are going to manipulate.
    pub fn remove_all_packedfiles(&mut self) {
        self.packed_files = vec![];
    }

    /// This function checks if a `PackedFile` exists in a `PackFile`.
    ///
    /// It requires:
    /// - `&self`: a `PackFileData` to check for the `PackedFile`.
    /// - `path`: the path of the `PackedFile` we want to check.
    pub fn packedfile_exists(&self, path: &[String]) -> bool {
        for packed_file in &self.packed_files {
            if packed_file.path == path {
                return true;
            }
        }
        false
    }

    /// This function checks if a folder with `PackedFiles` exists in a `PackFile`.
    ///
    /// It requires:
    /// - `&elf`: a `PackFileData` to check for the folder.
    /// - `path`: the path of the folder we want to check.
    pub fn folder_exists(&self, path: &[String]) -> bool {

        // If the path is empty, this triggers a false positive, so it needs to be checked here.
        if path.is_empty() { false }
        else {
            for packed_file in &self.packed_files {
                if packed_file.path.starts_with(path) && packed_file.path.len() > path.len() {
                    return true;
                }
            }

            for folder in &self.empty_folders {
                if folder.starts_with(path) { return true; }
            }

            false
        }
    }

    /// This functions serves to update the empty folder list.
    pub fn update_empty_folders(&mut self) {
        let packed_files = &self.packed_files;
        self.empty_folders.retain(|folder| {
            if folder.is_empty() { false }
            else {
                for packed_file in packed_files {
                    if packed_file.path.starts_with(folder) && packed_file.path.len() > folder.len() {
                        return false
                    }
                }
                true
            }
        })
    }

    /// This function reads the content of a PackFile and returns a `PackFile` with all the contents of the PackFile decoded.
    ///
    /// It requires:
    /// - `file_path`: a `PathBuf` with the path of the PackFile.
    /// - `use_lazy_loading`: if yes, don't load to memory his data.
    pub fn read(
        file_path: PathBuf,
        use_lazy_loading: bool
    ) -> Result<Self> {

        // Prepare the PackFile to be read and the virtual PackFile to be written.
        let mut pack_file = BufReader::new(File::open(&file_path)?);
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
        if pack_file_decoded.bitmask.contains(PFHFlags::HAS_ENCRYPTED_DATA) && pack_file_decoded.pfh_version == PFHVersion::PFH5 {
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
                timestamp,
                is_compressed,
                if pack_file_decoded.bitmask.contains(PFHFlags::HAS_ENCRYPTED_DATA) { Some(pack_file_decoded.pfh_version) } else { None },
                PackedFileData::OnDisk(
                    pack_file.clone(), 
                    data_position, 
                    size
                )
            );
            pack_file_decoded.packed_files.push(packed_file);

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

    /// This function takes a decoded `PackFile` and tries to encode it and write it on disk.
    ///
    /// It requires:
    /// - `&mut self`: the `PackFile` we are trying to save.
    pub fn save(&mut self) -> Result<()> {

        // For some bizarre reason, if the PackedFiles are not alphabetically sorted they may or may not crash the game for particular people.
        // So, to fix it, we have to sort all the PackedFiles here by path.
        // NOTE: This sorting has to be CASE INSENSITIVE. This means for "ac", "Ab" and "aa" it'll be "aa", "Ab", "ac".
        self.packed_files.sort_unstable_by(|a, b| a.path.join("\\").to_lowercase().cmp(&b.path.join("\\").to_lowercase()));
        
        // We ensure that all the data is loaded and in his right form (compressed/encrypted) before attempting to save.
        // We need to do this here because we need later on their compressed size.
        for packed_file in &mut self.packed_files { 
            packed_file.load_data()?;

            // Remember: first compress (only PFH5), then encrypt.
            let (data, is_compressed, is_encrypted, should_be_compressed, should_be_encrypted) = packed_file.get_data_and_info_from_memory()?;
            
            // Compression is not yet supported (stupid xz). Uncompress everything.
            if *is_compressed {
                *data = decompress_data(&data)?;
                *is_compressed = false;
                *should_be_compressed = false;
            }

            // Encryption is not yet supported. Unencrypt everything.
            if *is_encrypted { 
                *data = decrypt_packed_file(&data);
                *is_encrypted = false;
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
                    if self.bitmask.contains(PFHFlags::HAS_INDEX_WITH_TIMESTAMPS) { packed_file_index.extend_from_slice(&encode_integer_u32(packed_file.timestamp as u32)); }
                    //if packed_file.is_compressed { packed_file_index.push(1); } else { packed_file_index.push(0); } 
                    packed_file_index.push(0);
                }
                PFHVersion::PFH4 => {
                    if self.bitmask.contains(PFHFlags::HAS_INDEX_WITH_TIMESTAMPS) { packed_file_index.extend_from_slice(&encode_integer_u32(packed_file.timestamp as u32)); }
                }
                PFHVersion::PFH3 => {
                    if self.bitmask.contains(PFHFlags::HAS_INDEX_WITH_TIMESTAMPS) { packed_file_index.extend_from_slice(&encode_integer_i64(packed_file.timestamp)); }
                }

                // This one doesn't have timestamps, so we just skip this step.
                PFHVersion::PFH0 => {}
            }

            packed_file_index.append(&mut packed_file.path.join("\\").as_bytes().to_vec());
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
            let (data,_,_,_,_) = packed_file.get_data_and_info_from_memory()?;
            file.write_all(&data)?; }

        // If nothing has failed, return success.
        Ok(())
    }
}
