// In this file are all the Structs and Impls required to decode and encode the PackFiles.
// NOTE: Arena support was implemented thanks to the work of "Trolldemorted" here: https://github.com/TotalWarArena-Modding/twa_pack_lib
use std::num::Wrapping;
use std::path::PathBuf;
use std::io::prelude::*;
use std::io::{ BufReader, BufWriter, Read, Write, SeekFrom };
use std::fs::File;

use common::*;
use common::coding_helpers::*;
use error::{ErrorKind, Result};

/// This `Struct` stores the data of the PackFile in memory, along with some extra data needed to manipulate the PackFile.
///
/// It stores the PackFile divided in:
/// - `extra_data`: extra data that we need to manipulate the PackFile.
/// - `header`: header of the PackFile, decoded.
/// - `data`: data of the PackFile (index + data), decoded.
/// - `packed_file_indexes`: in case of Read-Only situations, like adding PackedFiles from another PackFile,
///   we can use this vector to store the indexes of the data, instead of the data per-se.
#[derive(Clone, Debug)]
pub struct PackFile {
    pub extra_data: PackFileExtraData,
    pub header: PackFileHeader,
    pub data: PackFileData,
    pub packed_file_indexes: Vec<u64>,
}

/// This `Struct` stores some extra data we need to manipulate the PackFiles.
///
/// The data stored is:
/// - `file_name`: name of the PackFile.
/// - `file_path`: current full path of the PackFile in the FileSystem.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PackFileExtraData {
    pub file_name: String,
    pub file_path: PathBuf,
}

/// This `Struct` stores all the info we can get from the header of the PackFile.
///
/// It contains the followind fields, all in 4 byte packs:
/// - `id`: ID of the PackFile, like a version. Normally it's `PFHX`.
/// - `pack_file_type`: type of the PackFile (mod, movie,...).
/// - `pack_file_count`: amount of entries in the PackFile index, at the start of the data (dependencies).
/// - `pack_file_index_size`: size in bytes of the entire PackFile Index (the first part of the data, if exists).
/// - `packed_file_count`: amount of PackedFiles stored inside the PackFile.
/// - `packed_file_index_size`: size in bytes of the entire PackedFile Index.
/// - `creation_time`: timestamp of when the PackFile was created, encoded in u32.
///
/// These fields are only used in "extended" `PFH4` and `PFH5` headers:
/// - `unknown_data_1`: bytes 0->4 of the extension, encoded in u32. Unknown use.
/// - `unknown_data_2`: bytes 4->8 of the extension, encoded in u32. Unknown use.
/// - `unknown_data_3`: bytes 8->12 of the extension, encoded in u32. Unknown use.
/// - `signature_position`: position of the signature at the end on the PackFile.
/// - `unknown_data_4`: bytes 16->20 of the extension, encoded in u32. Unknown use. Maybe this is part of the `signature_position` so it supports >4GB PackFiles?
///
/// These four variables are not directly related to the header, but are decoded from it:
/// - `data_is_encrypted`: true if the data of the PackedFiles is encrypted. Seen in `music.pack` in Attila, Rome 2 and Arena.
/// - `index_includes_timestamp`: true if the last modified date of each PackedFile is included in the index.
/// - `index_is_encrypted`: true if the PackedFile index is encrypted.
/// - `header_is_extended`: mysterious value found in Arena PackFiles. Can be usefull to identify them.
///
/// NOTE: to understand the `pack_file_type`, because it's quite complex:
/// - `0` => `Boot`,
/// - `1` => `Release`,
/// - `2` => `Patch`,
/// - `3` => `Mod`,
/// - `4` => `Movie`,
/// - Any other type => Special types we can't read/write properly, yet.
///
/// Also, a bitmask can be applied to that field:
/// - `16` => PackedFiles data is encrypted. 
/// - `64` => PackedFile index has a timestamp (last modification date) of each PackedFile just after his size.
/// - `128` => PackedFile index is encrypted (Only in Arena).
/// - `256` => Header is extended by 20 bytes, and bytes 12-16 of that extension are the signature position. Also, this in a PFH5 means the Indexes have a PFH4 structure. It's in every Arena PackFile (Only in Arena).
///
/// So, when getting the type, we first have to check his bitmasks and see what does it have.
/// NOTE: Currently we only support saving a PackFile if it doesn't have `data_is_encrypted`, `index_is_encrypted` or `header_is_extended` enabled.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PackFileHeader {
    pub id: String,
    pub pack_file_type: u32,
    pub pack_file_count: u32,
    pub pack_file_index_size: u32,
    pub packed_file_count: u32,
    pub packed_file_index_size: u32,
    pub creation_time: u32,

    pub unknown_data_1: u32,
    pub unknown_data_2: u32,
    pub unknown_data_3: u32,
    pub signature_position: u32,
    pub unknown_data_4: u32,

    pub data_is_encrypted: bool,
    pub index_includes_timestamp: bool,
    pub index_is_encrypted: bool,
    pub header_is_extended: bool,
}

/// This `Struct` stores all the data from the PackFile outside the header.
///
/// It contains:
/// - `pack_files`: a list of PackFiles our PackFile is meant to overwrite (I guess).
/// - `packed_files`: a list of the PackedFiles contained inside our PackFile.
/// - `empty_folders`: a list of every empty folder we have in the PackFile.
#[derive(Clone, Debug)]
pub struct PackFileData {
    pub pack_files: Vec<String>,
    pub packed_files: Vec<PackedFile>,
    pub empty_folders: Vec<Vec<String>>
}

/// This `Struct` stores the data of a PackedFile.
///
/// If contains:
/// - `size`: size of the data.
/// - `timestamp`: the '*Last Modified Date*' of the PackedFile, encoded in `u32`.
/// - `path`: path of the PackedFile inside the PackFile.
/// - `data`: the data of the PackedFile.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PackedFile {
    pub size: u32,
    pub timestamp: u32,
    pub path: Vec<String>,
    pub data: Vec<u8>,
}

/// Implementation of `PackFile`.
impl PackFile {

    /// This function creates a new empty `PackFile`. This is used for creating a *dummy* PackFile.
    pub fn new() -> Self {
        Self {
            extra_data: PackFileExtraData::new(),
            header: PackFileHeader::new("PFH5"),
            data: PackFileData::new(),
            packed_file_indexes: vec![],
        }
    }

    /// This function creates a new empty `PackFile` with a name and an specific id.
    pub fn new_with_name(file_name: String, packfile_id: &str) -> Self {
        Self {
            extra_data: PackFileExtraData::new_with_name(file_name),
            header: PackFileHeader::new(packfile_id),
            data: PackFileData::new(),
            packed_file_indexes: vec![],
        }
    }

    /// This function replaces the current `PackFile List` with a new one.
    ///
    /// It requires:
    /// - `&mut self`: the PackFile we are going to manipulate.
    /// - `pack_files`: a Vec<String> we are going to use as new list.
    pub fn save_packfiles_list(&mut self, pack_files: Vec<String>) {
        self.header.pack_file_count = pack_files.len() as u32;
        self.data.pack_files = pack_files;
    }

    /// This function adds one or more `PackedFiles` to an existing `PackFile`.
    ///
    /// It requires:
    /// - `&mut self`: the PackFile we are going to manipulate.
    /// - `packed_files`: a Vec<PackedFile> we are going to add.
    pub fn add_packedfiles(&mut self, mut packed_files: Vec<PackedFile>) {
        self.header.packed_file_count += packed_files.len() as u32;
        self.data.packed_files.append(&mut packed_files);
    }

    /// This function returns if the PackFile is editable or not, depending on the type of the PackFile.
    /// Basically, if the PackFile is not one of the known types OR it has any of the `pack_file_type` bitmasks
    /// as true, this'll return false. Use it to disable saving functions for PackFiles we can read but not
    /// save. Also, if the `is_editing_of_ca_packfiles_allowed` argument is false, return false for everything
    /// except types "Mod" and "Movie".
    pub fn is_editable(&self, is_editing_of_ca_packfiles_allowed: bool) -> bool {

        // If ANY of these bitmask is detected in the PackFile, disable all saving.
        // if self.header.mysterious_mask_music || self.header.index_has_extra_u32 || self.header.index_is_encrypted || self.header.mysterious_mask { false }
        if self.header.data_is_encrypted || self.header.index_is_encrypted || self.header.header_is_extended { false }

        // These types are always editable.
        else if self.header.pack_file_type == 3 || self.header.pack_file_type == 4 { true }

        // If the "Allow Editing of CA PackFiles" is enabled, these types are also enabled.
        else if is_editing_of_ca_packfiles_allowed && self.header.pack_file_type <= 2 { true }

        // Otherwise, always return false.
        else { false }
    }

    /// This function removes a PackedFile from a PackFile.
    ///
    /// It requires:
    /// - `&mut self`: the PackFile we are going to manipulate.
    /// - `index`: the index of the PackedFile we want to remove from the PackFile.
    pub fn remove_packedfile(&mut self, index: usize) {
        self.header.packed_file_count -= 1;
        self.data.packed_files.remove(index);
    }

    /// This function remove all PackedFiles from a PackFile.
    ///
    /// It requires:
    /// - `&mut self`: the PackFile we are going to manipulate.
    pub fn remove_all_packedfiles(&mut self) {
        self.header.packed_file_count = 0;
        self.data.packed_files = vec![];
    }

    /// This function reads the content of a PackFile and returns a `PackFile` with all the contents of the PackFile decoded.
    ///
    /// It requires:
    /// - `&mut pack_file`: a `BufReader` of the PackFile on disk.
    /// - `file_name`: a `String` with the name of the PackFile.
    /// - `file_path`: a `PathBuf` with the path of the PackFile.
    /// - `is_read_only`: if yes, don't load to memory his data. Instead, just get his indexes.
    pub fn read(
        pack_file: &mut BufReader<File>,
        file_name: String,
        file_path: PathBuf,
        is_read_only: bool
    ) -> Result<Self> {

        // We try to decode the header of the PackFile.
        let header = PackFileHeader::read(pack_file)?;

        // We try to decode his data.
        let mut data = PackFileData::read_indexes(pack_file, &header)?;

        // If it's Read-Only...
        if is_read_only {

            // Create the indexes vector.
            let mut packed_file_indexes = vec![];

            // Get the initial index from the position of the BufReader.
            let mut index = pack_file.seek(SeekFrom::Current(0))?;

            // For each PackFile, get his initial position and move the index.
            for packed_file in &data.packed_files {
                packed_file_indexes.push(index);
                index += packed_file.size as u64;
            }

            // Return the PackFilePartial.
            Ok(Self {
                extra_data: PackFileExtraData::new_from_file(file_name, file_path),
                header,
                data,
                packed_file_indexes,
            })
        }

        // Otherwise, we load the entire PackFile.
        else {

            // We try to load his data to memory.
            let _ = data.read_data(pack_file, &header)?;

            // We return a fully decoded PackFile.
            Ok(Self {
                extra_data: PackFileExtraData::new_from_file(file_name, file_path),
                header,
                data,
                packed_file_indexes: vec![],
            })
        }
    }

    /// This function takes a decoded `PackFile` and tries to encode it and write it on disk.
    ///
    /// It requires:
    /// - `&mut self`: the `PackFile` we are trying to save.
    /// - `mut file`: a `BufWriter` of the PackFile we are trying to write to.
    pub fn save(&mut self, mut file: &mut BufWriter<File>) -> Result<()> {

        // If any of the problematic masks in the header is set, return an error.
        if self.header.data_is_encrypted || self.header.index_is_encrypted || self.header.header_is_extended { return Err(ErrorKind::PackFileIsNonEditable)? }

        // For some bizarre reason, if the PackedFiles are not alphabetically sorted they may or may not crash the game for particular people.
        // So, to fix it, we have to sort all the PackedFiles here by path.
        self.data.packed_files.sort_unstable_by(|a, b| a.path.cmp(&b.path));

        // We encode the indexes, as we need their final size to encode complete the header.
        let indexes = self.data.save_indexes(&self.header);

        // We try to write the header.
        self.header.save(&mut file, indexes.0.len() as u32, indexes.1.len() as u32)?;

        // Then, we try to write the indexes to the file.
        file.write(&indexes.0)?;
        file.write(&indexes.1)?;

        // After all that, we try to write all the PackFiles to the file.
        self.data.save_data(&mut file)?;

        // If nothing has failed, return success.
        Ok(())
    }
}

/// Implementation of `PackFileExtraData`.
impl PackFileExtraData {

    /// This function creates an empty `PackFileExtraData`.
    pub fn new() -> Self {
        Self {
            file_name: String::new(),
            file_path: PathBuf::new(),
        }
    }

    /// This function creates a `PackFileExtraData` with just a name.
    pub fn new_with_name(file_name: String) -> Self {
        Self {
            file_name,
            file_path: PathBuf::new(),
        }
    }

    /// This function creates a `PackFileExtraData` with a name and a path.
    pub fn new_from_file(file_name: String, file_path: PathBuf) -> Self {
        Self {
            file_name,
            file_path,
        }
    }
}

/// Implementation of `PackFileHeader`.
impl PackFileHeader {

    /// This function creates a new `PackFileHeader` for an empty `PackFile`, requiring only an `ID`.
    pub fn new(packfile_id: &str) -> Self {

        // Create and return the Header.
        Self {
            id: packfile_id.to_owned(),
            pack_file_type: 3,
            pack_file_count: 0,
            pack_file_index_size: 0,
            packed_file_count: 0,
            packed_file_index_size: 0,
            creation_time: get_current_time(),

            unknown_data_1: 0,
            unknown_data_2: 0,
            unknown_data_3: 0,
            signature_position: 0,
            unknown_data_4: 0,

            data_is_encrypted: false,
            index_includes_timestamp: false,
            index_is_encrypted: false,
            header_is_extended: false,
        }
    }

    /// This function reads the header of a PackFile and decode it into a `PackFileHeader`.
    fn read(header: &mut BufReader<File>) -> Result<Self> {

        // Create a new default header.
        let mut pack_file_header = Self::new("PFH5");

        // Create a little buffer to read the data from the header.
        let mut buffer = vec![0; 8];

        // We try to read the ID and the Type/Bitmask of the PackFile.
        let bytes = header.read(&mut buffer)?;

        // If we didn't fill the complete buffer, the PackFile is invalid.
        if bytes != 8 { return Err(ErrorKind::PackFileHeaderNotComplete)? }

        // Try to decode his id.
        let id = decode_string_u8(&buffer[..4])?;

        // If the header's first 4 bytes are "PFH5" or "PFH4", it's a valid file, so we read it.
        if id == "PFH5" || id == "PFH4" { pack_file_header.id = id; }

        // If we reach this point, the file is not valid.
        else { return Err(ErrorKind::PackFileNotSupported)? }

        // Get the "base" PackFile Type.
        pack_file_header.pack_file_type = decode_integer_u32(&buffer[4..8])?;

        // Get the bitmasks from the PackFile's Type.
        pack_file_header.data_is_encrypted = if pack_file_header.pack_file_type & 16 != 0 { true } else { false };
        pack_file_header.index_includes_timestamp = if pack_file_header.pack_file_type & 64 != 0 { true } else { false };
        pack_file_header.index_is_encrypted = if pack_file_header.pack_file_type & 128 != 0 { true } else { false };
        pack_file_header.header_is_extended = if pack_file_header.pack_file_type & 256 != 0 { true } else { false };

        // Disable the masks, so we can get the true Type.
        pack_file_header.pack_file_type = pack_file_header.pack_file_type & 15;
        pack_file_header.pack_file_type = pack_file_header.pack_file_type & 63;
        pack_file_header.pack_file_type = pack_file_header.pack_file_type & 127;
        pack_file_header.pack_file_type = pack_file_header.pack_file_type & 255;

        // If it's a "PFH5" or "PFH4"...
        if pack_file_header.id == "PFH5" || pack_file_header.id == "PFH4" {

            // If it has an extended header, his size is 48 bytes.
            if pack_file_header.header_is_extended { buffer = vec![0; 48]; }

            // Otherwise, his size is 28 bytes.
            else { buffer = vec![0; 28]; }
        }

        // Restore the cursor of the BufReader to 0, so we can read the full header in one go. The first 8 bytes are
        // already decoded but, for the sake of clarity in the positions of the rest of the header stuff, we do this.
        header.seek(SeekFrom::Start(0))?;

        // We try to read the rest of the header.
        let bytes = header.read(&mut buffer)?;

        // If it's a "PFH5" or "PFH4"...
        if pack_file_header.id == "PFH5" || pack_file_header.id == "PFH4" {

            // If it has an extended header and his size is not 48, the PackFile doesn't have a complete header.
            if pack_file_header.header_is_extended && bytes != 48 { return Err(ErrorKind::PackFileHeaderNotComplete)? }

            // If it doesn't have an extended header and his size is not 28. the PackFile doesn't have a complete header.
            else if !pack_file_header.header_is_extended && bytes != 28 { return Err(ErrorKind::PackFileHeaderNotComplete)? }
        }

        // Fill the default header with the current PackFile values.
        pack_file_header.pack_file_count = decode_integer_u32(&buffer[8..12])?;
        pack_file_header.pack_file_index_size = decode_integer_u32(&buffer[12..16])?;
        pack_file_header.packed_file_count = decode_integer_u32(&buffer[16..20])?;
        pack_file_header.packed_file_index_size = decode_integer_u32(&buffer[20..24])?;

        // The creation time is an asshole. We need to get his u32 version.
        // To get the full timestamp we need to use:
        // let naive_date_time: NaiveDateTime = NaiveDateTime::from_timestamp(i64::from(decode_integer_u32(&buffer[24..28])?), 0);
        pack_file_header.creation_time = decode_integer_u32(&buffer[24..28])?;

        // If it's a "PFH5" or "PFH4" with an extended header...
        if (pack_file_header.id == "PFH5" || pack_file_header.id == "PFH4") && pack_file_header.header_is_extended { 

            // Fill the default header with the extended header values.
            pack_file_header.unknown_data_1 = decode_integer_u32(&buffer[28..32])?;
            pack_file_header.unknown_data_2 = decode_integer_u32(&buffer[32..36])?;
            pack_file_header.unknown_data_3 = decode_integer_u32(&buffer[36..40])?;
            pack_file_header.signature_position = decode_integer_u32(&buffer[40..44])?;
            pack_file_header.unknown_data_4 = decode_integer_u32(&buffer[44..48])?;
        }

        // Return the header.
        Ok(pack_file_header)
    }

    /// This function takes a decoded `PackFileHeader` and encodes it, so it can be saved in a PackFile file.
    ///
    /// We need the final size of both indexes for this.
    fn save(
        &mut self, 
        file: &mut BufWriter<File>, 
        pack_file_index_size: u32, 
        packed_file_index_size: u32
    ) -> Result<()> {

        // Complete the PackFile Type using the bitmasks. Currently, we don't really support saving with some of these bitmasks,
        // but to show how it would be done, we left this here. 
        let mut final_type = self.pack_file_type;
        //if self.data_is_encrypted { final_type = final_type | 16; }
        if self.index_includes_timestamp { final_type = final_type | 64; }
        //if self.index_is_encrypted { final_type = final_type | 128; }
        //if self.header_is_extended { final_type = final_type | 256; }

        // Write the entire header.
        file.write(&encode_string_u8(&self.id))?;
        file.write(&encode_integer_u32(final_type))?;
        file.write(&encode_integer_u32(self.pack_file_count))?;
        file.write(&encode_integer_u32(pack_file_index_size))?;
        file.write(&encode_integer_u32(self.packed_file_count))?;
        file.write(&encode_integer_u32(packed_file_index_size))?;

        // Update the creation time, then save it.
        self.creation_time = get_current_time();
        file.write(&encode_integer_u32(self.creation_time))?;

        // Return success.
        Ok(())
    }
}

/// Implementation of `PackFileData`.
impl PackFileData {

    /// This function creates a new empty `PackFileData`.
    pub fn new() -> Self {
        Self {
            pack_files: vec![],
            packed_files: vec![],
            empty_folders: vec![],
        }
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

    /// This function is used to check if any *empty folder* has been used for a `PackedFile`, and
    /// remove it from the empty folder list in that case.
    pub fn update_empty_folders(&mut self) {

        // List of folders to remove from the empty list.
        let mut folders_to_remove = vec![];

        // For each empty folder...
        for (index, folder) in self.empty_folders.iter().enumerate() {

            // For each PackedFile...
            for packed_file in &self.packed_files {

                // starts_with fails if the path is empty.
                if !folder.is_empty() {
                    if packed_file.path.starts_with(folder) && packed_file.path.len() > folder.len() {
                        folders_to_remove.push(index);
                        break;
                    }
                }

                // If the path is empty, remove it as it's an error.
                else {
                    folders_to_remove.push(index);
                    break;
                }
            }
        }

        // Remove every folder in the "to remove" list.
        folders_to_remove.iter().rev().for_each(|x| { self.empty_folders.remove(*x); });
    }

    /// This function reads the indexes of a PackFile, and creates a `PackedFileData` with it.
    ///
    /// It requires:
    /// - `data`: the raw data or the PackFile.
    /// - `header`: the header of the `PackFile`, decoded.
    fn read_indexes(
        data: &mut BufReader<File>,
        header: &PackFileHeader,
    ) -> Result<Self> {

        // Create our PackedFileData.
        let mut pack_file_data = Self::new();

        // Create the buffers for the indexes data.
        let mut pack_file_index = vec![0; header.pack_file_index_size as usize];
        let mut packed_file_index = vec![0; header.packed_file_index_size as usize];

        // Get the data from both indexes to their buffers.
        data.read_exact(&mut pack_file_index)?;
        data.read_exact(&mut packed_file_index)?;

        // If it's an Arena PackFile with the index encrypted, we need to decode it in a different way.
        if header.id == "PFH5" && header.index_is_encrypted {

            // NOTE: Code from here is based in the twa_pack_lib made by "Trolldemorted" here: https://github.com/TotalWarArena-Modding/twa_pack_lib
            // It's here because it's better (for me) to have all the PackFile's decoding logic together, integrated in RPFM,
            // instead of using a lib to load the data for only one game.
            // Feel free to correct anything if it's wrong, because this for me is almost black magic.

            // Offset for the loop to get the PackFiles from the PackFile index.
            let mut packed_file_index_offset: usize = 0;

            // For each PackedFile in the index...
            for packed_files_after_this_one in (0..header.packed_file_count).rev() {

                // We create an empty PackedFile.
                let mut packed_file = PackedFile::new();

                // Get his encrypted size.
                let mut encrypted_size = decode_integer_u32(&packed_file_index[packed_file_index_offset..(packed_file_index_offset + 4)])?;

                // Get the decrypted size.
                packed_file.size = decrypt_index_item_file_length(encrypted_size, packed_files_after_this_one as u32, &mut packed_file_index_offset);

                // If we have the last modified date of the PackedFiles, get it.
                if header.index_includes_timestamp { 
                    let timestamp = decode_integer_u32(&packed_file_index[packed_file_index_offset..(packed_file_index_offset + 4)])?;
                    packed_file.timestamp = decrypt_index_item_file_length(timestamp, packed_files_after_this_one as u32, &mut packed_file_index_offset);
                }

                // Get the decrypted path.
                let decrypted_path = decrypt_index_item_filename(&packed_file_index[packed_file_index_offset..], packed_file.size as u8, &mut packed_file_index_offset);

                // Split it and save it.
                packed_file.path = decrypted_path.split('\\').map(|x| x.to_owned()).collect::<Vec<String>>();

                // Once we are done, we add the PackedFile to the PackFileData.
                pack_file_data.packed_files.push(packed_file);
            }
        }

        // Otherwise, we use the normal decoding method.
        else {

            // Offset for the loop to get the PackFiles from the PackFile index.
            let mut pack_file_index_offset: usize = 0;

            // First, we decode every entry in the PackFile index and store it. The process is simple:
            // we get his name char by char until hitting 0u8, then save it and start getting the next
            // PackFile's name.
            for _ in 0..header.pack_file_count {

                // Store his name.
                let mut pack_file_name = String::new();

                // For each byte...
                loop {

                    // Get it.
                    let character = pack_file_index[pack_file_index_offset];

                    // If the byte is 0...
                    if character == 0 {

                        // Add the PackFile to the list, reset the `pack_file_name` and break the loop.
                        pack_file_data.pack_files.push(pack_file_name);
                        pack_file_index_offset += 1;
                        break;

                    // If it's not 0, then we add the character to the current PackFile name.
                    } else {

                        // Get his char value and add it to the String.
                        pack_file_name.push(character as char);
                        pack_file_index_offset += 1;
                    }
                }
            }

            // Offsets for the loop to get the file corresponding to the index entry.
            let mut packed_file_index_offset: usize = 0;

            // We choose the offset. This depends on a lot of conditions.
            let packed_file_index_path_offset: usize =

                // If it's a common PFH5 PackFile (Warhammer 2 & Arena)...
                if header.id == "PFH5" {

                    // If it has the extended header bit, is an Arena PackFile.
                    if header.header_is_extended {

                        // If it has the last modified date of the PackedFiles, we default to 8 (Arena).
                        if header.index_includes_timestamp { 8 }

                        // Otherwise, we default to 4.
                        else { 4 }
                    }

                    // Otherwise, it's a Warhammer 2 PackFile.
                    else {

                        // If it has the last modified date of the PackedFiles, we default to 9 (extra and separation byte).
                        if header.index_includes_timestamp { 9 }

                        // Otherwise, we default to 5 (0 between size and path, Warhammer 2).
                        else { 5 }
                    }
                }

                // If it's a common PFH4 PackFile (Warhammer/Attila/Rome 2).
                else if header.id == "PFH4" {

                    // If it has the last modified date of the PackedFiles, we default to 8.
                    if header.index_includes_timestamp { 8 }

                    // Otherwise, we default to 4 (no space between size and path of PackedFiles).
                    else { 4 }
                }

                // As default, we use 4 (Rome 2).
                else { 4 };

            // For each PackedFile in our PackFile...
            for _ in 0..header.packed_file_count {

                // We create an empty PackedFile.
                let mut packed_file = PackedFile::new();

                // Get his size.
                packed_file.size = decode_integer_u32(&packed_file_index[
                    packed_file_index_offset..packed_file_index_offset + 4
                ])?;

                // If we have the last modified date of the PackedFiles in the Index, get it.
                if header.index_includes_timestamp {

                    // Get his 'Last Modified Date'.
                    packed_file.timestamp = decode_integer_u32(&packed_file_index[
                        (packed_file_index_offset + 4)..(packed_file_index_offset + 8)
                    ])?;
                }

                // Update the index.
                packed_file_index_offset += packed_file_index_path_offset;

                // Create a little buffer to hold the characters until we get a complete name.
                let mut path = String::new();

                // Loop through all the characters in the path...
                loop {

                    // Get the character new character.
                    let character = packed_file_index[packed_file_index_offset];

                    // Increase the index for the next cycle.
                    packed_file_index_offset += 1;

                    // If the character is 0, we reached the end of the entry, so break the loop.
                    if character == 0 { break; }

                    // If the character is valid, push it to the path.
                    path.push(character as char);
                }

                // Split it and save it.
                packed_file.path = path.split('\\').map(|x| x.to_owned()).collect::<Vec<String>>();

                // Once we are done, we add the PackedFile to the PackFileData.
                pack_file_data.packed_files.push(packed_file);
            }
        }

        // If we reach this point, we managed to get the entire PackFile decoded, so we return it.
        Ok(pack_file_data)
    }

    /// This function reads the data of the `PackedFiles` of a PackFile, and adds it to the `PackedFiles` created when decoding the index.
    ///
    /// It requires:
    /// - `&mut self`: the current `PackFileData` where we are storing the `PackedFiles`.
    /// - `data`: the raw data or the PackFile.
    fn read_data(
        &mut self,
        data: &mut BufReader<File>,
        header: &PackFileHeader,
    ) -> Result<()> {

        // If it's a `PFH5` and the PackFile's data is encrypted, his data must start in a divisible by 8, so...
        if header.id == "PFH5" && header.data_is_encrypted { 

            // We get the current position in the file, and skip the bytes until we reach a divisible by 8.
            let position = data.seek(SeekFrom::Current(0))?;
            let padding = 8 - (position % 8);
            data.read(&mut vec![0; padding as usize])?;
        }

        // Now, we get the raw data from the PackFile, and get it into the corresponding PackedFile.
        for packed_file in &mut self.packed_files {

            // If it's a `PFH5` and the PackFile's data is encrypted, we need to decrypt it before storing it.
            // These encryted files always start in a divisible by 8, so we need to account for that.
            if header.id == "PFH5" && header.data_is_encrypted { 

                // Due to how the decrypt function works, every PackedFile must have a divisible by 8 size so we need to calculate the "extra bytes" to add to the PackedFile.
                let padding = 8 - (packed_file.size % 8);

                // Once we got the amount of "extra bytes" needed, we calculate his "final" size.
                let padded_size = if padding < 8 { packed_file.size + padding } else { packed_file.size };

                // Read his data from the PackFile.
                let mut encrypted_data = vec![0; padded_size as usize];
                data.read_exact(&mut encrypted_data)?;

                // Try to decrypt the data of the PackedFile and store it.
                packed_file.data = decrypt_file(&encrypted_data, packed_file.size as usize, false);
            }

            // If it's a `PFH4` and the PackFile's data is encrypted, we need to decrypt it before storing it. 
            else if header.id == "PFH4" && header.data_is_encrypted { 

                // Read his data from the PackFile.
                let mut encrypted_data = vec![0; packed_file.size as usize];
                data.read_exact(&mut encrypted_data)?;

                // Try to decrypt the data of the PackedFile and store it.
                packed_file.data = decrypt_file2(&encrypted_data, false);
            }

            // Otherwise, we just read it.
            else {

                // Prepare his buffer.
                packed_file.data = vec![0; packed_file.size as usize];

                // Read his "size" of bytes into his data.
                data.read_exact(&mut packed_file.data)?;
            }
        }

        // If we reach this point, we managed to get the entire PackFile decoded, so we return it.
        Ok(())
    }

    /// This function encode both indexes from a PackFile and returns them.
    fn save_indexes(&self, header: &PackFileHeader) -> (Vec<u8>, Vec<u8>) {

        // Create the vectors that'll hold the encoded indexes.
        let mut pack_file_index = vec![];
        let mut packed_file_index = vec![];

        // For each PackFile in our PackFile index...
        for pack_file in &self.pack_files {

            // Encode it and push a 0 at the end.
            pack_file_index.extend_from_slice(pack_file.as_bytes());
            pack_file_index.push(0);
        }

        // For each PackedFile in our PackedFile index...
        for packed_file in &self.packed_files {

            // Encode his size.
            packed_file_index.extend_from_slice(&encode_integer_u32(packed_file.size));

            // If it's a common PFH5 PackFile (Warhammer 2 & Arena)...
            if header.id == "PFH5" {

                // If it has the extended header bit, is an Arena PackFile.
                if header.header_is_extended {

                    // If it has the last modified date of the PackedFiles, we add it to the Index (Arena). 
                    // NOTE: This is not actually used, because if an Arena PackFile reaches this point, it means something broke somewhere else.
                    if header.index_includes_timestamp { packed_file_index.extend_from_slice(&encode_integer_u32(packed_file.timestamp)); }
                }

                // Otherwise, it's a Warhammer 2 PackFile.
                else {

                    // If it has the last modified date of the PackedFiles, we add it to the Index (Warhammer 2).
                    if header.index_includes_timestamp {
                        packed_file_index.extend_from_slice(&encode_integer_u32(packed_file.timestamp));
                    }

                    // Then, we add the zero separating numbers from the path (Warhammer 2).
                    packed_file_index.push(0);
                }
            }

            // If it's a common PFH4 PackFile (Warhammer/Attila/Rome 2).
            else if header.id == "PFH4" {

                // If it has the last modified date of the PackedFiles, we add it to the Index (boot.pack of Attila).
                if header.index_includes_timestamp { packed_file_index.extend_from_slice(&encode_integer_u32(packed_file.timestamp)); }
            }

            // For each field in the path...
            for position in 0..packed_file.path.len() {

                // Encode it.
                packed_file_index.extend_from_slice(packed_file.path[position].as_bytes());

                // If it's not the last field...
                if (position + 1) < packed_file.path.len() {

                    // Push a 92 (5C or \).
                    packed_file_index.push(92);
                }
            }

            // Push a 0 at the end of the Path.
            packed_file_index.push(0);
        }

        // We return the encoded indexes.
        (pack_file_index, packed_file_index)
    }

    /// This function writes all the PackedFile's data at the end of the provided file. Keep in mind this doesn't work with ANY kind of encryption.
    fn save_data(&self, file: &mut BufWriter<File>) -> Result<()> {

        // For each PackedFile, just try to write his data to the disk.
        for packed_file in &self.packed_files {
            file.write(&packed_file.data)?;
        }

        // If nothing failed, return success.
        Ok(())
    }
}

/// Implementation of `PackedFile`.
impl PackedFile {

    /// This function creates an empty `PackedFile`.
    pub fn new() -> Self {

        // Create and return the PackedFile.
        Self {
            size: 0,
            timestamp: 0,
            path: vec![],
            data: vec![],
        }
    }

    /// This function receive all the info of a PackedFile and creates a `PackedFile` with it.
    pub fn read(size: u32, timestamp: u32, path: Vec<String>, data: Vec<u8>) -> Self {
        Self {
            size,
            timestamp,
            path,
            data,
        }
    }
}

//-----------------------------------------------------------------------------------------------//
//                Decryption Functions (for Arena), copied from twa_pack_lib
//-----------------------------------------------------------------------------------------------//

// NOTE: The reason all these functions are here is because the `twa_pack_lib` doesn't make them public.

// Decryption key.
static KEY: &str = "L2{B3dPL7L*v&+Q3ZsusUhy[BGQn(Uq$f>JQdnvdlf{-K:>OssVDr#TlYU|13B}r";

/// Function to get the byte we want from the key above. I guess...
fn get_key_at(pos: usize) -> u8 {
    KEY.as_bytes()[pos % KEY.len()]
}

/// This function decrypts the size of a PackedFile. Requires:
/// - 'ciphertext': the encrypted size of the PackedFile, read directly as LittleEndian::u32.
/// - 'packed_files_after_this_one': the amount of items after this one in the Index.
/// - 'offset': offset to know in what position of the index we should continue decoding the next entry.
fn decrypt_index_item_file_length(ciphertext: u32, packed_files_after_this_one: u32, offset: &mut usize) -> u32 {

    // Decrypt the size of the PackedFile by xoring it. No idea where the 0x15091984 came from.
    let decrypted_size = packed_files_after_this_one ^ ciphertext ^ 0x15091984;

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
        let character = ciphertext[index] ^ decrypted_size ^ get_key_at(index);

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
    assert!(ciphertext.len() % 8 == 0, "ciphertext is not a multiple of 8");
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
