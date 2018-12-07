// In this file are all the Structs and Impls required to decode and encode the PackFiles.
// NOTE: Arena support was implemented thanks to the work of "Trolldemorted" here: https://github.com/TotalWarArena-Modding/twa_pack_lib
extern crate bitflags;

use std::num::Wrapping;
use std::path::PathBuf;
use std::io::prelude::*;
use std::io::{ BufReader, BufWriter, Read, Write, SeekFrom };
use std::fs::File;
use std::sync::{Arc, Mutex};

use crate::common::*;
use crate::common::coding_helpers::*;
use crate::error::{ErrorKind, Result};

/// These consts are used for dealing with Time-related operations.
const WINDOWS_TICK: i64 = 10000000;
const SEC_TO_UNIX_EPOCH: i64 = 11644473600;

/// These are the different Preamble/Id the PackFiles can have.
const PFH5_PREAMBLE: &str = "PFH5"; // PFH5
const PFH4_PREAMBLE: &str = "PFH4"; // PFH4
const PFH3_PREAMBLE: &str = "PFH3"; // PFH3
const PFH2_PREAMBLE: &str = "PFH2"; // PFH2
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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PFHVersion {
    PFH5,
    PFH4,
    PFH3,
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

/// This `Struct` stores the data of a PackedFile.
///
/// If contains:
/// - `timestamp`: the '*Last Modified Date*' of the PackedFile, encoded in `u32`.
/// - `path`: path of the PackedFile inside the PackFile.
/// - `data`: the data of the PackedFile.
#[derive(Clone, Debug)]
pub struct PackedFile {
    pub timestamp: i64,
    pub path: Vec<String>,
    data: PackedFileData,
}

/// This enum represents the data of a PackedFile.
///
/// - `OnMemory`: the data is loaded to memory and the variant holds it.
/// - `OnDisk`: the data is not loaded to memory and the variant holds the file, position and size of the data on the disk. Also, it has a tuple with a true if it's encrypted, and the PFHVersion of his PackFile.
#[derive(Clone, Debug)]
pub enum PackedFileData {
    OnMemory(Vec<u8>),
    OnDisk(Arc<Mutex<BufReader<File>>>, u64, u32, (bool, PFHVersion)),
} 

/// Implementation of PFHFileType.
impl PFHFileType {

    /// This function returns the PackFile's **Type** in `u32` format. To know what value corresponds with what type, check their definition's comment.
    pub fn get_value(&self) -> u32 {
        match *self {
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
        }
    }

    /// This function returns the PackFile's Version or an Error if the version is invalid.
    pub fn get_version(version: &str) -> Result<Self> {
        match version {
            PFH5_PREAMBLE => Ok(PFHVersion::PFH5),
            PFH4_PREAMBLE => Ok(PFHVersion::PFH4),
            PFH3_PREAMBLE => Ok(PFHVersion::PFH3),
            PFH2_PREAMBLE => Err(ErrorKind::PackFileNotSupported)?,
            PFH0_PREAMBLE => Err(ErrorKind::PackFileNotSupported)?,
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
            bitmask: self.bitmask.clone(),
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
        let pack_file_len = pack_file.get_ref().metadata()?.len();
        if pack_file_len < 8 { return Err(ErrorKind::PackFileHeaderNotComplete)? }

        // Create a little buffer to read the basic data from the header of the PackFile.
        let mut buffer = vec![0; 8];
        pack_file.read(&mut buffer)?;

        // Start populating our decoded PackFile Struct.
        pack_file_decoded.file_path = file_path;
        pack_file_decoded.pfh_version = PFHVersion::get_version(&decode_string_u8(&buffer[..4])?)?; 
        pack_file_decoded.pfh_file_type = PFHFileType::get_type(decode_integer_u32(&buffer[4..8])? & 15);
        pack_file_decoded.bitmask = PFHFlags::from_bits_truncate(decode_integer_u32(&buffer[4..8])? & !15);

        // Depending on the data we got, prepare to read the header and ensure we have all the bytes we need.
        match pack_file_decoded.pfh_version {
            PFHVersion::PFH5 | PFHVersion::PFH4 => {
                if pack_file_decoded.bitmask.contains(PFHFlags::HAS_EXTENDED_HEADER) && pack_file_len < 48 { return Err(ErrorKind::PackFileHeaderNotComplete)? }
                else if !pack_file_decoded.bitmask.contains(PFHFlags::HAS_EXTENDED_HEADER) && pack_file_len < 28 { return Err(ErrorKind::PackFileHeaderNotComplete)? }
                
                if pack_file_decoded.bitmask.contains(PFHFlags::HAS_EXTENDED_HEADER) { buffer = vec![0; 48]; }
                else { buffer = vec![0; 28]; }
            }

            PFHVersion::PFH3 => buffer = vec![0; 32],
        }

        // Restore the cursor of the BufReader to 0, so we can read the full header in one go. The first 8 bytes are
        // already decoded but, for the sake of clarity in the positions of the rest of the header stuff, we do this.
        pack_file.seek(SeekFrom::Start(0))?;

        // We try to read the rest of the header.
        pack_file.read(&mut buffer)?;

        // Fill the default header with the current PackFile values.
        let pack_file_count = decode_integer_u32(&buffer[8..12])?;
        let pack_file_index_size = decode_integer_u32(&buffer[12..16])?;
        let packed_file_count = decode_integer_u32(&buffer[16..20])?;
        let packed_file_index_size = decode_integer_u32(&buffer[20..24])?;

        // The creation time is a bit of an asshole. Depending on the PackFile Version/Id/Preamble, it uses a type, another or it doesn't exists.
        // Keep in mind that we store his raw value. If you want his legible value, you have to convert it yourself.
        pack_file_decoded.timestamp = match pack_file_decoded.pfh_version {
            PFHVersion::PFH5 | PFHVersion::PFH4 => decode_integer_u32(&buffer[24..28])? as i64,
            PFHVersion::PFH3 => (decode_integer_i64(&buffer[24..32])? / WINDOWS_TICK) - SEC_TO_UNIX_EPOCH,
        };

        // Ensure the PackFile has all the data needed for the index.
        let mut pack_file_data_position = (buffer.len() as u32 + pack_file_index_size + packed_file_index_size) as u64;
        if pack_file_len < pack_file_data_position { return Err(ErrorKind::PackFileIndexesNotComplete)? }

        // Create the buffers for the indexes data.
        let mut pack_file_index = vec![0; pack_file_index_size as usize];
        let mut packed_file_index = vec![0; packed_file_index_size as usize];

        // Get the data from both indexes to their buffers.
        pack_file.read_exact(&mut pack_file_index)?;
        pack_file.read_exact(&mut packed_file_index)?;

        // Read the PackFile Index.
        let mut pack_file_index_position: usize = 0;

        // First, we decode every entry in the PackFile index and store it. The process is simple:
        // we get his name char by char until hitting 0u8, then save it and start getting the next
        // PackFile's name.
        for _ in 0..pack_file_count {
            let mut pack_file_name = String::new();
            loop {
                let character = pack_file_index[pack_file_index_position];
                pack_file_index_position += 1;
                if character == 0 { break; }
                pack_file_name.push(character as char);
            }

            pack_file_decoded.pack_files.push(pack_file_name);
        }

        // Depending on the version of the PackFile and his bitmask, the PackedFile index has one format or another.
        let packed_file_index_path_offset = match pack_file_decoded.pfh_version {
            PFHVersion::PFH5 => {

                // If it has the extended header bit, is an Arena PackFile.
                if pack_file_decoded.bitmask.contains(PFHFlags::HAS_EXTENDED_HEADER) {

                    // If it has the last modified date of the PackedFiles, we default to 8 (Arena). Otherwise, we default to 4.
                    if pack_file_decoded.bitmask.contains(PFHFlags::HAS_INDEX_WITH_TIMESTAMPS) { 8 } else { 4 }
                }

                // Otherwise, it's a Warhammer 2 PackFile, so we default to 9 (extra and separation byte). Otherwise, we default to 5 (0 between size and path, Warhammer 2).
                else if pack_file_decoded.bitmask.contains(PFHFlags::HAS_INDEX_WITH_TIMESTAMPS) { 9 } else { 5 }
            }
            // If it has the last modified date of the PackedFiles, we default to 8 (Arena). Otherwise, we default to 4.
            PFHVersion::PFH4 => if pack_file_decoded.bitmask.contains(PFHFlags::HAS_INDEX_WITH_TIMESTAMPS) { 8 } else { 4 }

            // If it has the last modified date of the PackedFiles, we default to 12. Otherwise, we default to 4.
            PFHVersion::PFH3 => if pack_file_decoded.bitmask.contains(PFHFlags::HAS_INDEX_WITH_TIMESTAMPS) { 12 } else { 4 }
        };

        // Offset for the loop to get the PackFiles from the PackFile index.
        let mut packed_file_index_position: usize = 0;

        // NOTE: Code from here is based in the twa_pack_lib made by "Trolldemorted" here: https://github.com/TotalWarArena-Modding/twa_pack_lib
        // It's here because it's better (for me) to have all the PackFile's decoding logic together, integrated in RPFM,
        // instead of using a lib to load the data for only one game.
        // Feel free to correct anything if it's wrong, because this for me is almost black magic.
        if pack_file_decoded.bitmask.contains(PFHFlags::HAS_ENCRYPTED_INDEX) {

            // These PackedFiles have their data always starting in multiples of 8.
            let pack_file = Arc::new(Mutex::new(pack_file));
            if pack_file_decoded.bitmask.contains(PFHFlags::HAS_ENCRYPTED_DATA) {
                pack_file_data_position = if (pack_file_data_position % 8) > 0 { pack_file_data_position + 8 - (pack_file_data_position % 8) } else { pack_file_data_position };
            }
            for packed_files_after_this_one in (0..packed_file_count).rev() {

                // Get his size.
                let mut encrypted_size = decode_integer_u32(&packed_file_index[packed_file_index_position..(packed_file_index_position + 4)])?;
                let size = decrypt_index_item_file_length(encrypted_size, packed_files_after_this_one as u32, &mut packed_file_index_position);

                // If we have the last modified date of the PackedFiles, get it.
                let timestamp = if pack_file_decoded.bitmask.contains(PFHFlags::HAS_INDEX_WITH_TIMESTAMPS) {
                    let encrypted_timestamp = match pack_file_decoded.pfh_version {
                        PFHVersion::PFH5 | PFHVersion::PFH4 => decode_integer_u32(&packed_file_index[packed_file_index_position..(packed_file_index_position + 4)])? as i64,
                        PFHVersion::PFH3 => (decode_integer_i64(&packed_file_index[packed_file_index_position..(packed_file_index_position + 8)])? / WINDOWS_TICK) - SEC_TO_UNIX_EPOCH,
                    };
                    decrypt_index_item_file_length(encrypted_timestamp as u32, packed_files_after_this_one as u32, &mut packed_file_index_position)
                } else { 0 };

                // Get the decrypted path.
                let path = decrypt_index_item_filename(&packed_file_index[packed_file_index_position..], size as u8, &mut packed_file_index_position);
                let path = path.split('\\').map(|x| x.to_owned()).collect::<Vec<String>>();

                // Once we are done, we add the PackedFile to the PackFileData.
                let packed_file = PackedFile::read2(
                    timestamp as i64,
                    path,
                    PackedFileData::OnDisk(
                        pack_file.clone(),
                        pack_file_data_position,
                        size,
                        (pack_file_decoded.bitmask.contains(PFHFlags::HAS_ENCRYPTED_DATA), pack_file_decoded.pfh_version.clone())
                    )
                );
                pack_file_decoded.packed_files.push(packed_file);
                if pack_file_decoded.bitmask.contains(PFHFlags::HAS_ENCRYPTED_DATA) {
                    let padding = 8 - (size % 8);
                    let padded_size = if padding < 8 { size + padding } else { size };
                    pack_file_data_position += padded_size as u64;
                }

                else { pack_file_data_position += size as u64; }
            }
        }

        // Otherwise, we decode it as a normal PackFile.
        else {
            let pack_file = Arc::new(Mutex::new(pack_file));
            for _ in 0..packed_file_count {

                // Get his size.
                let size = decode_integer_u32(&packed_file_index[packed_file_index_position..packed_file_index_position + 4])?;

                // If we have the last modified date of the PackedFiles in the Index, get it.
                let timestamp = if pack_file_decoded.bitmask.contains(PFHFlags::HAS_INDEX_WITH_TIMESTAMPS) {
                    match pack_file_decoded.pfh_version {
                        PFHVersion::PFH5 | PFHVersion::PFH4 => decode_integer_u32(&packed_file_index[(packed_file_index_position + 4)..(packed_file_index_position + 8)])? as i64,
                        PFHVersion::PFH3 => (decode_integer_i64(&packed_file_index[(packed_file_index_position + 4)..(packed_file_index_position + 12)])? / WINDOWS_TICK) - SEC_TO_UNIX_EPOCH,
                    }
                } else { 0 };

                // Get his path and update the position of the index.
                packed_file_index_position += packed_file_index_path_offset;
                
                // Create a little buffer to hold the characters until we get a complete name.
                let mut path = String::new();
                loop {
                    let character = packed_file_index[packed_file_index_position];
                    packed_file_index_position += 1;
                    if character == 0 { break; }
                    path.push(character as char);
                }

                let path = path.split('\\').map(|x| x.to_owned()).collect::<Vec<String>>();

                // Once we are done, we add the PackedFile to the PackFileData.
                let packed_file = PackedFile::read2(
                    timestamp,
                    path, 
                    PackedFileData::OnDisk(
                        pack_file.clone(), 
                        pack_file_data_position, 
                        size, 
                        (pack_file_decoded.bitmask.contains(PFHFlags::HAS_ENCRYPTED_DATA), pack_file_decoded.pfh_version.clone())
                    )
                );
                pack_file_decoded.packed_files.push(packed_file);
                pack_file_data_position += size as u64;
            }

            // If at this point we have not reached the end of the PackFile, there is something wrong with it.
            if pack_file_data_position != pack_file_len { return Err(ErrorKind::PackFileSizeIsNotWhatWeExpect(pack_file_len, pack_file_data_position))? }
        }

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
        
        // We ensure that all the data is loaded before attemting to save.
        for packed_file in &mut self.packed_files { packed_file.load_data()?; }

        // To minimize Ram usage, first we encode the indexes.
        let mut pack_file_index = vec![];
        let mut packed_file_index = vec![];

        for pack_file in &self.pack_files {
            pack_file_index.extend_from_slice(pack_file.as_bytes());
            pack_file_index.push(0);
        }

        for packed_file in &self.packed_files {
            packed_file_index.extend_from_slice(&encode_integer_u32(packed_file.get_size()));

            // Depending on the version of the PackFile and his bitmask, the PackedFile index has one format or another.
            match self.pfh_version {
                PFHVersion::PFH5 => {
                    if self.bitmask.contains(PFHFlags::HAS_INDEX_WITH_TIMESTAMPS) { packed_file_index.extend_from_slice(&encode_integer_u32(packed_file.timestamp as u32)); }
                    if !self.bitmask.contains(PFHFlags::HAS_EXTENDED_HEADER) { packed_file_index.push(0); }
                }
                PFHVersion::PFH4 => {
                    if self.bitmask.contains(PFHFlags::HAS_INDEX_WITH_TIMESTAMPS) { packed_file_index.extend_from_slice(&encode_integer_u32(packed_file.timestamp as u32)); }
                }
                PFHVersion::PFH3 => {
                    if self.bitmask.contains(PFHFlags::HAS_INDEX_WITH_TIMESTAMPS) { packed_file_index.extend_from_slice(&encode_integer_i64(packed_file.timestamp)); }
                }
            }

            packed_file_index.append(&mut packed_file.path.join("\\").as_bytes().to_vec());
            packed_file_index.push(0);
        }

        // Create the file to save to, and save the header and the indexes.
        let mut file = BufWriter::new(File::create(&self.file_path)?);

        // Write the entire header.
        file.write(&encode_string_u8(&self.pfh_version.get_value()))?;
        file.write(&encode_integer_u32(self.bitmask.bits | self.pfh_file_type.get_value()))?;
        file.write(&encode_integer_u32(self.pack_files.len() as u32))?;
        file.write(&encode_integer_u32(pack_file_index.len() as u32))?;
        file.write(&encode_integer_u32(self.packed_files.len() as u32))?;
        file.write(&encode_integer_u32(packed_file_index.len() as u32))?;

        // Update the creation time, then save it.
        self.timestamp = get_current_time();
        match self.pfh_version {
            PFHVersion::PFH5 | PFHVersion::PFH4 => file.write(&encode_integer_u32(self.timestamp as u32))?,
            PFHVersion::PFH3 => file.write(&encode_integer_i64((self.timestamp + SEC_TO_UNIX_EPOCH) * WINDOWS_TICK))?,
        };

        // Write the indexes and the data of the PackedFiles. No need to keep the data, as it has been preloaded before.
        file.write(&pack_file_index)?;
        file.write(&packed_file_index)?;
        for packed_file in &self.packed_files { file.write(&(packed_file.get_data()?))?; }

        // If nothing has failed, return success.
        Ok(())
    }
}

/// Implementation of `PackedFile`.
impl PackedFile {

    /// This function receive all the info of a PackedFile and creates a `PackedFile` with it.
    pub fn read(timestamp: i64, path: Vec<String>, data: Vec<u8>) -> Self {
        Self {
            timestamp,
            path,
            data: PackedFileData::OnMemory(data),
        }
    }

    /// This function receive all the info of a PackedFile and creates a `PackedFile` with it.
    pub fn read2(timestamp: i64, path: Vec<String>, data: PackedFileData) -> Self {
        Self {
            timestamp,
            path,
            data,
        }
    }

    /// This function loads the data from the disk if it's not loaded yet.
    pub fn load_data(&mut self) -> Result<()> {
        let data_on_memory = if let PackedFileData::OnDisk(ref file, position, size, (is_encrypted, pack_file_version)) = self.data {
            if is_encrypted {
                match pack_file_version {
                    PFHVersion::PFH5 => {
                        let padding = 8 - (size % 8);
                        let padded_size = if padding < 8 { size + padding } else { size };
                        let mut data = vec![0; padded_size as usize];
                        file.lock().unwrap().seek(SeekFrom::Start(position))?;
                        file.lock().unwrap().read_exact(&mut data)?;
                        PackedFileData::OnMemory(decrypt_file(&data, size as usize, false))
                    }

                    PFHVersion::PFH4 | _ => {
                        let mut data = vec![0; size as usize];
                        file.lock().unwrap().seek(SeekFrom::Start(position))?;
                        file.lock().unwrap().read_exact(&mut data)?;
                        PackedFileData::OnMemory(decrypt_file2(&data, false))
                    }
                }
            }
            else {
                let mut data = vec![0; size as usize];
                file.lock().unwrap().seek(SeekFrom::Start(position))?;
                file.lock().unwrap().read_exact(&mut data)?;
                PackedFileData::OnMemory(data)
            }
        } else { return Ok(()) };
        self.data = data_on_memory;
        Ok(())
    }

    /// This function reads the data from the disk if it's not loaded yet, and return it. This does not store the data in memory.
    pub fn get_data(&self) -> Result<Vec<u8>> {
        match self.data {
            PackedFileData::OnMemory(ref data) => return Ok(data.to_vec()),
            PackedFileData::OnDisk(ref file, position, size, (is_encrypted, pack_file_version)) => {
                if is_encrypted {
                    match pack_file_version {
                        PFHVersion::PFH5 => {
                            let padding = 8 - (size % 8);
                            let padded_size = if padding < 8 { size + padding } else { size };
                            let mut data = vec![0; padded_size as usize];
                            file.lock().unwrap().seek(SeekFrom::Start(position))?;
                            file.lock().unwrap().read_exact(&mut data)?;
                            Ok(decrypt_file(&data, size as usize, false))
                        }

                        PFHVersion::PFH4 | _ => {
                            let mut data = vec![0; size as usize];
                            file.lock().unwrap().seek(SeekFrom::Start(position))?;
                            file.lock().unwrap().read_exact(&mut data)?;
                            Ok(decrypt_file2(&data, false))
                        }
                    }
                }
                else {
                    let mut data = vec![0; size as usize];
                    file.lock().unwrap().seek(SeekFrom::Start(position))?;
                    file.lock().unwrap().read_exact(&mut data)?;
                    Ok(data)
                }
            }
        }
    }

    /// This function reads the data from the disk if it's not loaded yet (or from memory otherwise), and keep it in memory for faster access.
    pub fn get_data_and_keep_it(&mut self) -> Result<Vec<u8>> {
        let data = match self.data {
            PackedFileData::OnMemory(ref data) => return Ok(data.to_vec()),
            PackedFileData::OnDisk(ref file, position, size, (is_encrypted, pack_file_version)) => {
                if is_encrypted {
                    match pack_file_version {
                        PFHVersion::PFH5 => {
                            let padding = 8 - (size % 8);
                            let padded_size = if padding < 8 { size + padding } else { size };
                            let mut data = vec![0; padded_size as usize];
                            file.lock().unwrap().seek(SeekFrom::Start(position))?;
                            file.lock().unwrap().read_exact(&mut data)?;
                            decrypt_file(&data, size as usize, false)
                        }

                        PFHVersion::PFH4 | _ => {
                            let mut data = vec![0; size as usize];
                            file.lock().unwrap().seek(SeekFrom::Start(position))?;
                            file.lock().unwrap().read_exact(&mut data)?;
                            decrypt_file2(&data, false)
                        }
                    }
                }
                else {
                    let mut data = vec![0; size as usize];
                    file.lock().unwrap().seek(SeekFrom::Start(position))?;
                    file.lock().unwrap().read_exact(&mut data)?;
                    data
                }
            }
        };

        self.data = PackedFileData::OnMemory(data.clone());
        Ok(data)
    }

    /// This function loads the data from the disk if it's not loaded yet.
    pub fn set_data(&mut self, data: Vec<u8>) {
        self.data = PackedFileData::OnMemory(data);
    }

    /// This function returns the size of the data of the PackedFile.
    pub fn get_size(&self) -> u32 {
        match self.data {
            PackedFileData::OnMemory(ref data) => data.len() as u32,
            PackedFileData::OnDisk(_, _, size, (_,_)) => size,
        }
    }

}

//-----------------------------------------------------------------------------------------------//
//                     Decryption Functions, copied from tw_pack_lib
//-----------------------------------------------------------------------------------------------//

// NOTE: The reason all these functions are here is because the `twa_pack_lib` doesn't make them public.

// Decryption key.
// static KEY: &str = "L2{B3dPL7L*v&+Q3ZsusUhy[BGQn(Uq$f>JQdnvdlf{-K:>OssVDr#TlYU|13B}r";
static INDEX_KEY: &str = "#:AhppdV-!PEfz&}[]Nv?6w4guU%dF5.fq:n*-qGuhBJJBm&?2tPy!geW/+k#pG?";

/// Function to get the byte we want from the key above. I guess...
fn get_key_at(pos: usize) -> u8 {
    INDEX_KEY.as_bytes()[pos % INDEX_KEY.len()]
}

/// This function decrypts the size of a PackedFile. Requires:
/// - 'ciphertext': the encrypted size of the PackedFile, read directly as LittleEndian::u32.
/// - 'packed_files_after_this_one': the amount of items after this one in the Index.
/// - 'offset': offset to know in what position of the index we should continue decoding the next entry.
fn decrypt_index_item_file_length(ciphertext: u32, packed_files_after_this_one: u32, offset: &mut usize) -> u32 {

    // Decrypt the size of the PackedFile by xoring it. No idea where the 0x15091984 came from.
    let decrypted_size = !packed_files_after_this_one ^ ciphertext ^ 0xE10B73F4;

    // Increase the offset.
    *offset += 4;

    // Return the decrypted value.
    decrypted_size
}

/// This function decrypts the path of a PackedFile. Requires:
/// - 'ciphertext': the encrypted data of the PackedFile, read from the begining of the encrypted path.
/// - 'decrypted_size': the decrypted size of the PackedFile.
/// - 'offset': offset to know in what position of the index we should continue decoding the next entry.
fn decrypt_index_item_filename(ciphertext: &[u8], decrypted_size: u8, offset: &mut usize) -> String {

    // Create a string to hold the decrypted path.
    let mut path: String = String::new();

    // Create the index for the loop.
    let mut index = 0;

    // Loop through all the characters in the path...
    loop {

        // Get the character by xoring it.
        let character = ciphertext[index] ^ !decrypted_size ^ get_key_at(index);

        // Increase the index for the next cycle.
        index += 1;

        // If the character is 0, we reached the end of the entry, so break the loop.
        if character == 0 { break; }

        // If the character is valid, push it to the path.
        path.push(character as char);
    }

    // Increase the offset.
    *offset += index;

    // Once we finish, return the path
    path
}

// Key needed to decrypt files.
static FILE_KEY: Wrapping<u64> = Wrapping(0x8FEB2A6740A6920E);

// Don't make me try to explain this. Is magic for me.
pub fn decrypt_file(ciphertext: &[u8], length: usize, verbose: bool) -> Vec<u8> {
    let mut plaintext = Vec::with_capacity(ciphertext.len());
    let padded_length = ciphertext.len() + 7 & !7;
    assert!(padded_length % 8 == 0);
    assert!(padded_length < ciphertext.len() + 8);
    let mut edi: u32 = 0;
    let mut esi = 0;
    let mut eax;
    let mut edx;
        for _ in 0..padded_length/8 {
        // push 0x8FEB2A67
        // push 0x40A6920E
        // mov eax, edi
        // not eax
        // push 0
        // push eax
        // call multiply
        let prod = (FILE_KEY * Wrapping((!edi) as u64)).0;
        if verbose {
            println!("prod: {:X}", prod);
        }
        eax = prod as u32;
        edx = (prod >> 32) as u32;
        if verbose {
            println!("eax: {:X}", eax);
            println!("edx: {:X}", edx);
        }

        // xor eax, [ebx+esi]
        eax ^= decode_integer_u32(&ciphertext[esi..esi + 4]).unwrap();
        if verbose {
            println!("eax: {:X}", eax);
        }

        // add edi, 8
        edi += 8;

        // xor edx, [ebx+esi+4]
        let _edx = decode_integer_u32(&ciphertext[esi + 4..esi + 8]).unwrap();
        if verbose {
            println!("_edx {:X}", _edx);
        }
        edx ^= _edx;
        if verbose {
            println!("edx {:X}", edx);
        }

        // mov [esi], eax
        plaintext.append(&mut encode_integer_u32(eax));

        // mov [esi+4], edx
        if verbose {
            println!("{:X}", edx);
        }
        plaintext.append(&mut encode_integer_u32(edx));
        esi += 8;
    }
    plaintext.truncate(length);
    plaintext
}

// Don't make me try to explain this. Is magic for me.
pub fn decrypt_file2(ciphertext: &[u8], verbose: bool) -> Vec<u8> {
    let mut plaintext = Vec::with_capacity(ciphertext.len());
    let mut edi: u32 = 0;
    let mut esi = 0;
    let mut eax;
    let mut edx;
    for _ in 0..ciphertext.len()/8 {
        // push 0x8FEB2A67
        // push 0x40A6920E
        // mov eax, edi
        // not eax
        // push 0
        // push eax
        // call multiply
        let prod = (FILE_KEY * Wrapping((!edi) as u64)).0;
        if verbose {
            println!("prod: {:X}", prod);
        }
        eax = prod as u32;
        edx = (prod >> 32) as u32;
        if verbose {
            println!("eax: {:X}", eax);
            println!("edx: {:X}", edx);
        }

        // xor eax, [ebx+esi]
        eax ^= decode_integer_u32(&ciphertext[esi..esi + 4]).unwrap();
        if verbose {
            println!("eax: {:X}", eax);
        }

        // add edi, 8
        edi += 8;

        // xor edx, [ebx+esi+4]
        let _edx = decode_integer_u32(&ciphertext[esi + 4..esi + 8]).unwrap();
        if verbose {
            println!("_edx {:X}", _edx);
        }
        edx ^= _edx;
        if verbose {
            println!("edx {:X}", edx);
        }

        // mov [esi], eax
        plaintext.append(&mut encode_integer_u32(eax));

        // mov [esi+4], edx
        if verbose {
            println!("{:X}", edx);
        }
        plaintext.append(&mut encode_integer_u32(edx));
        esi += 8;
    }
    plaintext
}
