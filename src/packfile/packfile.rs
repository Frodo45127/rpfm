// In this file are all the Structs and Impls required to decode and encode the PackFiles.
extern crate chrono;
extern crate failure;

use self::chrono::{
    NaiveDateTime, Utc
};
use std::path::PathBuf;
use std::io::prelude::*;
use std::io::{ BufReader, BufWriter, Read, Write, SeekFrom };
use std::fs::File;
use failure::Error;

use common::coding_helpers::*;
use settings::*;

/// `PackFile`: This stores the data of the entire PackFile in memory ('cause fuck lazy-loading),
/// along with some extra data needed to manipulate the PackFile.
/// It stores the PackFile divided in 3 structs:
/// - extra_data: extra data that we need to manipulate the PackFile.
/// - header: header of the PackFile, decoded.
/// - data: data of the PackFile, decoded.
/// - packed_file_indexes: in case of Read-Only situations, like adding PackedFiles from another PackFile,
///   we can use this vector to store the indexes of the data, instead of the data per-se.
#[derive(Clone, Debug)]
pub struct PackFile {
    pub extra_data: PackFileExtraData,
    pub header: PackFileHeader,
    pub data: PackFileData,
    pub packed_file_indexes: Vec<u64>,
}

/// `PackFileExtraData`: This struct stores some extra data we need to manipulate the PackFiles:
/// - file_name: name of the PackFile.
/// - file_path: current full path of the PackFile in the FileSystem.
/// - is_modified: true if we have changed the PackFile in any way.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PackFileExtraData {
    pub file_name: String,
    pub file_path: PathBuf,
    pub is_modified: bool,
}

/// `PackFileHeader`: This struct stores all the info we can get from the header of the PackFile:
/// - id: ID of the PackFile, like a version.
/// - pack_file_type: type of the PackFile (mod, movie,...).
/// - pack_file_count: amount of files in the PackFile index, at the start of the data (dependencies).
/// - pack_file_index_size: size in bytes of the PackFile Index of the file (the first part of the data, if exists).
/// - packed_file_count: amount of PackedFiles stored inside the PackFile.
/// - packed_file_index_size: size in bytes of the PackedFile Index of the file (the first part of the data).
/// - creation_time: turns out this is the epoch date of the creation of the PackFile.
///
/// NOTE: to understand the "pack_file_type":
/// - 0 => "Boot",
/// - 1 => "Release",
/// - 2 => "Patch",
/// - 3 => "Mod",
/// - 4 => "Movie",
/// - Any other type => Special types we don't want to edit, only to read.
#[derive(Clone, Debug)]
pub struct PackFileHeader {
    pub id: String,
    pub pack_file_type: u32,
    pub pack_file_count: u32,
    pub pack_file_index_size: u32,
    pub packed_file_count: u32,
    pub packed_file_index_size: u32,
    pub creation_time: NaiveDateTime,
}

/// `PackFileData`: This struct stores all the data from the PackFile outside the header:
/// - pack_files: a list of PackFiles our PackFile is meant to overwrite (I guess).
/// - packed_files: a list of the PackedFiles contained inside our PackFile.
/// - empty_folders: a list of every empty folder we have in the PackFile.
#[derive(Clone, Debug)]
pub struct PackFileData {
    pub pack_files: Vec<String>,
    pub packed_files: Vec<PackedFile>,
    pub empty_folders: Vec<Vec<String>>
}

/// `PackedFile`: This struct stores the data of a PackedFile:
/// - size: size of the data.
/// - path: path of the PackedFile inside the PackFile.
/// - data: the data of the PackedFile.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PackedFile {
    pub size: u32,
    pub path: Vec<String>,
    pub data: Vec<u8>,
}

/// Implementation of "PackFile".
impl PackFile {

    /// This function creates a new empty "PackFile". This is used for creating a "dummy" PackFile.
    pub fn new() -> Self {
        Self {
            extra_data: PackFileExtraData::new(),
            header: PackFileHeader::new("PFH5"),
            data: PackFileData::new(),
            packed_file_indexes: vec![],
        }
    }

    /// This function creates a new empty "PackFile" with a name.
    pub fn new_with_name(file_name: String, packfile_id: &str) -> Self {
        Self {
            extra_data: PackFileExtraData::new_with_name(file_name),
            header: PackFileHeader::new(packfile_id),
            data: PackFileData::new(),
            packed_file_indexes: vec![],
        }
    }

    /// This function adds one or more PackedFiles to an existing PackFile.
    /// It requires:
    /// - self: the PackFile we are going to manipulate.
    /// - packed_files: a Vec<PackedFile> we are going to add.
    pub fn add_packedfiles(&mut self, mut packed_files: Vec<PackedFile>) {
        self.header.packed_file_count += packed_files.len() as u32;
        self.data.packed_files.append(&mut packed_files);
    }

    /// This function returns if the PackFile is editable or not, depending on the type of the PackFile.
    /// Basically, if the PackFile is not one of the 5 know types, this'll return false. Use it to disable
    /// saving functions for PackFiles we can read but not save. Like the "boot.pack" from Attila.
    /// Also, if the "Allow edition of CA PackFiles" setting is disabled, return false for everything
    /// except types "Mod" and "Movie".
    pub fn is_editable(&self, settings: &Settings) -> bool {

        // These types are always editable.
        if self.header.pack_file_type == 3 || self.header.pack_file_type == 4 { true }
        else if self.header.pack_file_type <= 2 && settings.allow_editing_of_ca_packfiles { true }
        else { false }
    }

    /// This function removes a PackedFile from a PackFile.
    /// It requires:
    /// - self: the PackFile we are going to manipulate.
    /// - index: the index of the PackedFile we want to remove from the PackFile.
    pub fn remove_packedfile(&mut self, index: usize) {
        self.header.packed_file_count -= 1;
        self.data.packed_files.remove(index);
    }

    /// This function remove all PackedFiles from a PackFile.
    /// It requires:
    /// - self: the PackFile we are going to manipulate.
    pub fn remove_all_packedfiles(&mut self) {
        self.header.packed_file_count = 0;
        self.data.packed_files = vec![];
    }

    /// This function reads the content of a PackFile and returns an struct PackFile with all the
    /// contents of the PackFile decoded.
    /// It requires:
    /// - pack_file: a BufReader of the PackFile on disk.
    /// - file_name: a String with the name of the PackFile.
    /// - file_path: a PathBuf with the path of the PackFile.
    /// - is_read_only: if yes, don't load to memory his data. Instead, just get his indexes.
    pub fn read(
        pack_file: &mut BufReader<File>,
        file_name: String,
        file_path: PathBuf,
        is_read_only: bool
    ) -> Result<Self, Error> {

        // We try to decode the header of the PackFile.
        match PackFileHeader::read(pack_file) {

            // If it works.
            Ok(header) => {

                // We try to decode his data.
                match PackFileData::read_indexes(
                    pack_file,
                    &header
                ) {

                    // If it works...
                    Ok(mut data) => {

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
                            match data.read_data(pack_file) {
                                Ok(_) => {

                                    // We return a fully decoded PackFile.
                                    Ok(Self {
                                        extra_data: PackFileExtraData::new_from_file(file_name, file_path),
                                        header,
                                        data,
                                        packed_file_indexes: vec![],
                                    })
                                }

                                // Otherwise, we return error.
                                Err(error) => Err(error),
                            }
                        }
                    },

                    // Otherwise, we return error.
                    Err(error) => Err(error),
                }
            }

            // Otherwise, we return error.
            Err(error) => Err(error),
        }
    }

    /// This function takes a decoded &mut PackFile, and tries to encode it and write it on disk.
    pub fn save(&self, mut file: &mut BufWriter<File>) -> Result<(), Error> {

        // First, we encode the indexes, as we need their final size to encode complete the header.
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

/// Implementation of "PackFileExtraData".
impl PackFileExtraData {

    /// This function creates an empty PackFileExtraData.
    pub fn new() -> Self {
        Self {
            file_name: String::new(),
            file_path: PathBuf::new(),
            is_modified: false,
        }
    }

    /// This function creates a PackFileExtraData with just a name.
    pub fn new_with_name(file_name: String) -> Self {
        Self {
            file_name,
            file_path: PathBuf::new(),
            is_modified: false,
        }
    }

    /// This function creates a PackFileExtraData with a name and a path.
    pub fn new_from_file(file_name: String, file_path: PathBuf) -> Self {
        Self {
            file_name,
            file_path,
            is_modified: false,
        }
    }
}

/// Implementation of "PackFileHeader".
impl PackFileHeader {

    /// This function creates a new PackFileHeader for an empty PackFile, requiring only an ID.
    pub fn new(packfile_id: &str) -> Self {
        Self {
            id: packfile_id.to_owned(),
            pack_file_type: 3,
            pack_file_count: 0,
            pack_file_index_size: 0,
            packed_file_count: 0,
            packed_file_index_size: 0,
            creation_time: Utc::now().naive_utc(),
        }
    }

    /// This function reads the Header of a PackFile and decode it into a PackFileHeader.
    fn read(header: &mut BufReader<File>) -> Result<Self, Error> {

        // Create a new default header.
        let mut pack_file_header = Self::new("PFH5");

        // Create a little buffer to read the data from the header.
        let mut buffer = [0; 28];

        // Check if at least has enough bytes to try to get his header.
        match header.read(&mut buffer) {
            Ok(bytes) => {

                // If we filled the complete buffer, we have the minimum amount of bytes to try to decode it.
                if bytes == 28 {

                    // Check his first 4 headers, to see if they are PackFiles we can read.
                    match decode_string_u8(&buffer[..4]) {
                        Ok(id) => {

                            // If the header's first 4 bytes are "PFH5" or "PFH4", it's a valid file, so we read it.
                            if id == "PFH5" || id == "PFH4" {
                                pack_file_header.id = id;
                            }

                            // If we reach this point, the file is not valid.
                            else {
                                return Err(format_err!("The file is not a supported PackFile.\n\nFor now, we only support:\n - Warhammer 2.\n - Warhammer.\n - Attila."))
                            }
                        }

                        // If we reach this point, there has been a decoding error.
                        Err(error) => return Err(error),
                    }
                }

                // Otherwise, return an error.
                else { return Err(format_err!("The file doesn't even have a full header.")) }
            }

            // If we couldn't read the header, return the error.
            Err(_) => return Err(format_err!("Error while trying to read the header of the PackFile from the disk.")),
        }

        // Fill the default header with the current PackFile values.
        pack_file_header.pack_file_type = decode_integer_u32(&buffer[4..8])?;
        pack_file_header.pack_file_count = decode_integer_u32(&buffer[8..12])?;
        pack_file_header.pack_file_index_size = decode_integer_u32(&buffer[12..16])?;
        pack_file_header.packed_file_count = decode_integer_u32(&buffer[16..20])?;
        pack_file_header.packed_file_index_size = decode_integer_u32(&buffer[20..24])?;
        pack_file_header.creation_time = NaiveDateTime::from_timestamp(i64::from(decode_integer_u32(&buffer[24..28])?), 0);

        // Return the header.
        Ok(pack_file_header)
    }

    /// This function takes a decoded Header and encode it, so it can be saved in a PackFile file.
    /// We need the final size of both indexes for this.
    fn save(&self, file: &mut BufWriter<File>, pack_file_index_size: u32, packed_file_index_size: u32) -> Result<(), Error> {

        file.write(&encode_string_u8(&self.id))?;
        file.write(&encode_integer_u32(self.pack_file_type))?;
        file.write(&encode_integer_u32(self.pack_file_count))?;
        file.write(&encode_integer_u32(pack_file_index_size))?;
        file.write(&encode_integer_u32(self.packed_file_count))?;
        file.write(&encode_integer_u32(packed_file_index_size))?;

        // For some reason this returns a reversed i64. We need to truncate it and reverse it before
        // writing it to the data.
        let mut creation_time = encode_integer_i64(Utc::now().naive_utc().timestamp());
        creation_time.truncate(4);
        creation_time.reverse();
        file.write(&creation_time)?;

        // Return success.
        Ok(())
    }
}

/// Implementation of "PackFileData".
impl PackFileData {

    /// This function creates a new empty "PackFileData".
    pub fn new() -> Self {
        Self {
            pack_files: vec![],
            packed_files: vec![],
            empty_folders: vec![],
        }
    }

    /// This function checks if a PackedFile exists in a PackFile.
    /// It requires:
    /// - self: a PackFileData to check for the PackedFile.
    /// - path: the path of the PackedFile we want to check.
    pub fn packedfile_exists(&self, path: &[String]) -> bool {
        for packed_file in &self.packed_files {
            if packed_file.path == path {
                return true;
            }
        }
        false
    }

    /// This function checks if a folder with PackedFiles exists in a PackFile.
    /// It requires:
    /// - self: a PackFileData to check for the folder.
    /// - path: the path of the folder we want to check.
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

    /// This function is used to check if any "empty folder" has been used for a PackedFile, and
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

    /// This function reads the Data part of a PackFile, and creates a PackedFileData with it.
    /// It requires:
    /// - data: the raw data or the PackFile.
    /// - header: the header of the PackFile.
    fn read_indexes(
        data: &mut BufReader<File>,
        header: &PackFileHeader,
    ) -> Result<Self, Error> {

        // Create our PackedFileData.
        let mut pack_file_data = Self::new();

        // Create the buffers for the indexes data.
        let mut pack_file_index = vec![0; header.pack_file_index_size as usize];
        let mut packed_file_index = vec![0; header.packed_file_index_size as usize];

        // Get the data from both indexes to their buffers.
        data.read_exact(&mut pack_file_index)?;
        data.read_exact(&mut packed_file_index)?;

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

        // PFH5 PackFiles (Warhammer 2) have a 0 separating size and name of the file in the index.
        let packed_file_index_path_offset: usize =

            // If it's a common PFH5 PackFile (Warhammer 2).
            if header.id == "PFH5" && header.pack_file_type <= 4 { 5 }

            // If it's a common PFH4 PackFile (Warhammer & most of Attila).
            else if header.id == "PFH4" && header.pack_file_type <= 4 { 4 }

            // If it's Attila's BootX.
            else if header.id == "PFH4" && header.pack_file_type > 4 { 8 }

            // As default, we use 4 (Attila).
            else { 4 };

        // For each PackedFile in our PackFile...
        for _ in 0..header.packed_file_count {

            // We create an empty PackedFile.
            let mut packed_file = PackedFile::new();

            // Get his size.
            packed_file.size = decode_integer_u32(&packed_file_index[
                packed_file_index_offset..packed_file_index_offset + 4
            ])?;

            // Update the index.
            packed_file_index_offset += packed_file_index_path_offset;

            // Create a little buffer to hold the characters until we get a complete name.
            let mut character_buffer = String::new();

            // For each byte...
            loop {

                // Get it.
                let character = packed_file_index[packed_file_index_offset];

                // If the byte is 0...
                if character == 0 {

                    // Add the PackFile to the list and break the loop.
                    packed_file.path.push(character_buffer);

                    // We move the index to the begining of the next entry.
                    packed_file_index_offset += 1;

                    // And break the loop.
                    break;
                }

                // If the byte is 92 (\ or 5C), we got a folder.
                else if character == 92 {

                    // We add it to the PackedFile's path.
                    packed_file.path.push(character_buffer);

                    // Reset the character buffer.
                    character_buffer = String::new();

                    // We move the index to the begining of the next name.
                    packed_file_index_offset += 1;
                }

                // If it's not 0 nor 92, it's a character from our current file.
                else {

                    // Get his char value and add it to the buffer.
                    character_buffer.push(character as char);
                    packed_file_index_offset += 1;
                }
            }

            // Once we are done, we add the PackedFile to the PackFileData.
            pack_file_data.packed_files.push(packed_file);
        }

        // If we reach this point, we managed to get the entire PackFile decoded, so we return it.
        Ok(pack_file_data)
    }

    /// This function reads the Data part of a PackFile, and creates a PackedFileData with it.
    /// It requires:
    /// - data: the raw data or the PackFile.
    /// - header: the header of the PackFile.
    fn read_data(
        &mut self,
        data: &mut BufReader<File>,
    ) -> Result<(), Error> {

        // Now, we get the raw data from the PackedFiles, and get it into the corresponding PackedFile.
        for packed_file in &mut self.packed_files {

            // Prepare his buffer.
            packed_file.data = vec![0; packed_file.size as usize];

            // Read his "size" of bytes into his data.
            data.read_exact(&mut packed_file.data)?;
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

            // If it's a PFH5 (Warhammer 2), put a 0 between size and path.
            if header.id == "PFH5" { packed_file_index.push(0) };

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

    /// This function writes all the PackedFile's data at the end of the provided file.
    fn save_data(&self, file: &mut BufWriter<File>) -> Result<(), Error> {

        // For each PackedFile, just try to write his data to the disk.
        for packed_file in &self.packed_files {
            file.write(&packed_file.data)?;
        }

        // If nothing failed, return success.
        Ok(())
    }
}

/// Implementation of "PackedFile".
impl PackedFile {

    /// This function creates an empty PackedFile.
    pub fn new() -> Self {
        Self {
            size: 0,
            path: vec![],
            data: vec![],
        }
    }

    /// This function receive all the info of a PackedFile and creates a PackedFile with it.
    pub fn read(size: u32, path: Vec<String>, data: Vec<u8>) -> Self {
        Self {
            size,
            path,
            data,
        }
    }
}
