// In this file are all the Structs and Impls required to decode and encode the PackFiles.
// For now we only support common TW: Warhammer 2 PackFiles (not loc files, those are different).
extern crate chrono;
extern crate failure;

use self::chrono::{
    NaiveDateTime, Utc
};
use std::path::PathBuf;
use std::io::BufReader;
use std::io::Read;
use std::fs::File;
use self::failure::Error;

use common::coding_helpers::*;
use common::coding_helpers;

/// Struct PackFile: This stores the data of the entire PackFile in memory ('cause fuck lazy-loading),
/// along with some extra data needed to manipulate the PackFile.
/// It stores the PackFile divided in 3 structs:
/// - pack_file_extra_data: extra data that we need to manipulate the PackFile.
/// - pack_file_header: header of the PackFile, decoded.
/// - pack_file_data: data of the PackFile, decoded.
#[derive(Clone, Debug)]
pub struct PackFile {
    pub pack_file_extra_data: PackFileExtraData,
    pub pack_file_header: PackFileHeader,
    pub pack_file_data: PackFileData,
}

/// Struct PackFileExtraData: This struct stores some extra data we need to manipulate the PackFiles:
/// - file_name: name of the PackFile.
/// - file_path: current full path of the PackFile in the FileSystem.
/// - correlation_data: Vector with all the paths that are already in the TreeView. Useful for checking.
#[derive(Clone, Debug)]
pub struct PackFileExtraData {
    pub file_name: String,
    pub file_path: PathBuf,
    pub is_modified: bool,
}


/// Struct PackFileHeader: This struct stores all the info we can get from the header of the PackFile:
/// - pack_file_id: ID of the PackFile, like a version.
/// - pack_file_type: type of the PackFile (mod, movie,...).
/// - pack_file_count: amount of files in the PackFile index, at the start of the data (dependencies).
/// - pack_file_index_size: size in bytes of the PackFile Index of the file (the first part of the data, if exists).
/// - packed_file_count: amount of PackedFiles stored inside the PackFile.
/// - packed_file_index_size: size in bytes of the PackedFile Index of the file (the first part of the data).
/// - packed_file_creation_time: turns out this is the epoch date of the creation of the PackFile.
///
/// NOTE: to understand the "pack_file_type":
/// - 0 => "Boot",
/// - 1 => "Release",
/// - 2 => "Patch",
/// - 3 => "Mod",
/// - 4 => "Movie",
#[derive(Clone, Debug)]
pub struct PackFileHeader {
    pub pack_file_id: String,
    pub pack_file_type: u32,
    pub pack_file_count: u32,
    pub pack_file_index_size: u32,
    pub packed_file_count: u32,
    pub packed_file_index_size: u32,
    pub packed_file_creation_time: NaiveDateTime,
}

/// Struct PackFileData: This struct stores all the PackedFiles inside the PackFile in a vector.
#[derive(Clone, Debug)]
pub struct PackFileData {
    pub pack_files: Vec<String>,
    pub packed_files: Vec<PackedFile>,
}

/// Struct PackedFile: This struct stores the data of a PackedFile:
/// - packed_file_size: size of the data.
/// - packed_file_path: path of the PackedFile inside the PackFile.
/// - packed_file_data: the data of the PackedFile. Temporal, until we implement PackedFileTypes.
#[derive(Clone, Debug)]
pub struct PackedFile {
    pub packed_file_size: u32,
    pub packed_file_path: Vec<String>,
    pub packed_file_data: Vec<u8>,
}

/// Implementation of "PackFile"
impl PackFile {

    /// This function creates a new empty "PackFile". This is used for creating a "dummy" PackFile.
    pub fn new() -> PackFile {
        let pack_file_extra_data = PackFileExtraData::new();
        let pack_file_header = PackFileHeader::new("PFH5");
        let pack_file_data = PackFileData::new();

        PackFile {
            pack_file_extra_data,
            pack_file_header,
            pack_file_data,
        }
    }

    /// This function creates a new empty "PackFile" with a name.
    pub fn new_with_name(file_name: String, packfile_id:&str) -> PackFile {
        let pack_file_extra_data = PackFileExtraData::new_with_name(file_name);
        let pack_file_header = PackFileHeader::new(packfile_id);
        let pack_file_data = PackFileData::new();

        PackFile {
            pack_file_extra_data,
            pack_file_header,
            pack_file_data,
        }
    }

    /// This function adds one or more PackedFiles to an existing PackFile.
    /// It requires:
    /// - self: the PackFile we are going to manipulate.
    /// - packed_files: a Vec<PackedFile> we are going to add.
    pub fn add_packedfiles(&mut self, packed_files: &[PackedFile]) {
        for packed_file in packed_files {
            self.pack_file_header.packed_file_count += 1;
            self.pack_file_data.packed_files.push(packed_file.clone());
        }
    }

    /// This function remove a PackedFile from a PackFile.
    /// It requires:
    /// - self: the PackFile we are going to manipulate.
    /// - index: the index of the PackedFile we want to remove from the PackFile.
    pub fn remove_packedfile(&mut self, index: usize) {
        self.pack_file_header.packed_file_count -= 1;
        self.pack_file_data.packed_files.remove(index);
    }

    /// This function remove all PackedFiles from a PackFile.
    /// It requires:
    /// - self: the PackFile we are going to manipulate.
    pub fn remove_all_packedfiles(&mut self) {
        self.pack_file_header.packed_file_count = 0;
        self.pack_file_data.packed_files = vec![];
    }

    /// This function reads the content of a PackFile and returns an struct PackFile with all the
    /// contents of the PackFile decoded.
    /// It requires:
    /// - pack_file_buffered: a Vec<u8> with the entire PackFile encoded inside it.
    /// - file_name: a String with the name of the PackFile.
    /// - file_path: a PathBuf with the path of the PackFile.
    pub fn read(pack_file: &mut BufReader<File>, file_name: String, file_path: PathBuf) -> Result<PackFile, Error> {

        // We save the "Extra data" of the packfile
        let pack_file_extra_data = PackFileExtraData::new_from_file(file_name, file_path);

        // We try to decode the header of the PackFile.
        match PackFileHeader::read(pack_file) {

            // If it works.
            Ok(header) => {

                // We try to decode his data.
                match PackFileData::read(
                    pack_file,
                    &header
                ) {

                    // If it works...
                    Ok(data) => {

                        // We return a fully decoded PackFile.
                        Ok(PackFile {
                            pack_file_extra_data,
                            pack_file_header: header,
                            pack_file_data: data,
                        })
                    },

                    // Otherwise, we return error.
                    Err(error) => Err(error),
                }
            }

            // Otherwise, we return error.
            Err(error) => Err(error),
        }
    }

    /// This function takes a decoded &PackFile and encode it, ready for being wrote in the disk.
    pub fn save(pack_file_decoded: &PackFile) -> Vec<u8> {
        let mut pack_file_data_encoded = PackFileData::save(
            &pack_file_decoded.pack_file_data,
            &pack_file_decoded.pack_file_header.pack_file_id
        );

        // Both index sizes are only needed to open and save the PackFile, so we only recalculate them
        // on save.
        let new_pack_file_index_size = pack_file_data_encoded[0].len() as u32;
        let new_packed_file_index_size = pack_file_data_encoded[1].len() as u32;
        let mut pack_file_header_encoded = PackFileHeader::save(
            &pack_file_decoded.pack_file_header,
            new_pack_file_index_size,
            new_packed_file_index_size
        );

        let mut pack_file_encoded = vec![];
        pack_file_encoded.append(&mut pack_file_header_encoded);
        pack_file_encoded.append(&mut pack_file_data_encoded[0]);
        pack_file_encoded.append(&mut pack_file_data_encoded[1]);
        pack_file_encoded.append(&mut pack_file_data_encoded[2]);
        pack_file_encoded
    }
}

/// Implementation of "PackFileExtraData"
impl PackFileExtraData {

    /// This function creates an empty PackFileExtraData.
    pub fn new() -> PackFileExtraData {
        let file_name = String::new();
        let file_path = PathBuf::new();
        let is_modified = false;
        PackFileExtraData {
            file_name,
            file_path,
            is_modified,
        }
    }

    /// This function creates a PackFileExtraData with just a name.
    pub fn new_with_name(file_name: String) -> PackFileExtraData {
        let file_path = PathBuf::new();
        let is_modified = false;
        PackFileExtraData {
            file_name,
            file_path,
            is_modified,
        }
    }

    /// This function creates a PackFileExtraData with a name and a path.
    pub fn new_from_file(file_name: String, file_path: PathBuf) -> PackFileExtraData {
        let is_modified = false;
        PackFileExtraData {
            file_name,
            file_path,
            is_modified,
        }
    }
}

/// Implementation of "PackFileHeader"
impl PackFileHeader {

    /// This function creates a new PackFileHeader for an empty PackFile of Warhammer 2.
    pub fn new(packfile_id: &str) -> PackFileHeader {
        let pack_file_id = packfile_id.to_string();
        let pack_file_type = 3 as u32;
        let pack_file_count = 0 as u32;
        let pack_file_index_size = 0 as u32;
        let packed_file_count = 0 as u32;
        let packed_file_index_size = 0 as u32;
        let packed_file_creation_time = Utc::now().naive_utc();

        PackFileHeader {
            pack_file_id,
            pack_file_type,
            pack_file_count,
            pack_file_index_size,
            packed_file_count,
            packed_file_index_size,
            packed_file_creation_time,
        }
    }

    /// This function reads the Header of a PackFile and decode it into a PackFileHeader. We read all
    /// this data in packs of 4 bytes, and read them in LittleEndian.
    pub fn read(header: &mut BufReader<File>) -> Result<PackFileHeader, Error> {

        // Create a new default header.
        let mut pack_file_header = PackFileHeader::new("PFH5");

        // Create a little buffer to read the data from the header.
        let mut buffer = [0; 28];

        // Check if at least has enough bytes to try to get his header.
        match header.read(&mut buffer) {
            Ok(bytes) => {

                // If we filled the complete buffer, we have the minimum amount of bytes to try to decode it.
                if bytes == 28 {

                    // Check his first 4 headers, to see if they are PackFiles we can read.
                    match decode_string_u8(&buffer[..4]) {
                        Ok(pack_file_id) => {

                            // If the header's first 4 bytes are "PFH5" or "PFH4", it's a valid file, so we read it.
                            if pack_file_id == "PFH5" || pack_file_id == "PFH4" {
                                pack_file_header.pack_file_id = pack_file_id;
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
        pack_file_header.pack_file_type = coding_helpers::decode_integer_u32(&buffer[4..8])?;
        pack_file_header.pack_file_count = coding_helpers::decode_integer_u32(&buffer[8..12])?;
        pack_file_header.pack_file_index_size = coding_helpers::decode_integer_u32(&buffer[12..16])?;
        pack_file_header.packed_file_count = coding_helpers::decode_integer_u32(&buffer[16..20])?;
        pack_file_header.packed_file_index_size = coding_helpers::decode_integer_u32(&buffer[20..24])?;
        pack_file_header.packed_file_creation_time = NaiveDateTime::from_timestamp(i64::from(coding_helpers::decode_integer_u32(&buffer[24..28])?), 0);

        // Return the header.
        Ok(pack_file_header)
    }

    /// This function takes a decoded Header and encode it, so it can be saved in a PackFile file.
    /// We just put all the data in order in a 28 bytes Vec<u8>, and return that Vec<u8>.
    pub fn save(header_decoded: &PackFileHeader, pack_file_index_size: u32, packed_file_index_size: u32) -> Vec<u8> {
        let mut header_encoded = vec![];

        let mut pack_file_id = coding_helpers::encode_string_u8(&header_decoded.pack_file_id);
        let mut pack_file_type = coding_helpers::encode_integer_u32(header_decoded.pack_file_type);
        let mut pack_file_count = coding_helpers::encode_integer_u32(header_decoded.pack_file_count);
        let mut pack_file_index_size = coding_helpers::encode_integer_u32(pack_file_index_size);
        let mut packed_file_count = coding_helpers::encode_integer_u32(header_decoded.packed_file_count);
        let mut packed_file_index_size = coding_helpers::encode_integer_u32(packed_file_index_size);
        let mut packed_file_creation_time = coding_helpers::encode_integer_i64(Utc::now().naive_utc().timestamp());

        // For some reason this returns a reversed i64. We need to truncate it and reverse it before
        // writing it to the data.
        packed_file_creation_time.truncate(4);
        packed_file_creation_time.reverse();

        header_encoded.append(&mut pack_file_id);
        header_encoded.append(&mut pack_file_type);
        header_encoded.append(&mut pack_file_count);
        header_encoded.append(&mut pack_file_index_size);
        header_encoded.append(&mut packed_file_count);
        header_encoded.append(&mut packed_file_index_size);
        header_encoded.append(&mut packed_file_creation_time);
        header_encoded
    }
}

/// Implementation of "PackFileData"
impl PackFileData {

    /// This function creates a new empty "PackFileData"
    pub fn new() -> Self {
        Self {
            pack_files: vec![],
            packed_files: vec![],
        }
    }

    /// This function checks if a PackedFile exists in a PackFile.
    /// It requires:
    /// - self: a PackFileData to check for the PackedFile.
    /// - packed_file_paths: the paths of the PackedFiles we want to check.
    pub fn packedfile_exists(&self, packed_file_path: &[String]) -> bool {
        for packed_file in &self.packed_files {
            if packed_file.packed_file_path == packed_file_path {
                return true;
            }
        }
        false
    }

    /// This function checks if a folder with PackedFiles exists in a PackFile.
    /// It requires:
    /// - self: a PackFileData to check for the folder.
    /// - packed_file_paths: the path of the folder we want to check.
    pub fn folder_exists(&self, packed_file_path: &[String]) -> bool {
        for packed_file in &self.packed_files {
            if packed_file.packed_file_path.starts_with(packed_file_path)
                && packed_file.packed_file_path.len() > packed_file_path.len() {
                return true;
            }
        }
        false
    }

    /// This function reads the Data part of a PackFile, get all the files on the PackFile and put
    /// them in a Vec<PackedFile>.
    /// It requires:
    /// - data: the raw data or the PackFile.
    /// - pack_file_id: ID of the PackFile, so we can decode multiple PackFile Types.
    /// - pack_file_count: the amount of PackFiles inside the PackFile Index. This should come from the header.
    /// - pack_index_size: the size of the index of PackFiles. This should come from the header.
    /// - packed_file_count: the amount of PackedFiles inside the PackFile. This should come from the header.
    /// - packed_index_size: the size of the index of PackedFiles. This should come from the header.
    pub fn read(
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
        let packed_file_index_path_offset: usize = if header.pack_file_id == "PFH5" { 5 } else { 4 };

        // For each PackedFile in our PackFile...
        for _ in 0..header.packed_file_count {

            // We create an empty PackedFile.
            let mut packed_file = PackedFile::new();

            // Get his size.
            packed_file.packed_file_size = decode_integer_u32(&packed_file_index[
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
                    packed_file.packed_file_path.push(character_buffer);

                    // We move the index to the begining of the next entry.
                    packed_file_index_offset += 1;

                    // And break the loop.
                    break;
                }

                // If the byte is 92 (\ or 5C), we got a folder.
                else if character == 92 {

                    // We add it to the PackedFile's path.
                    packed_file.packed_file_path.push(character_buffer);

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

        // Now, we get the raw data from the PackedFiles, and get it into the corresponding PackedFile.
        for packed_file in &mut pack_file_data.packed_files {

            // Prepare his buffer.
            packed_file.packed_file_data = vec![0; packed_file.packed_file_size as usize];

            // Read his "size" of bytes into his data.
            data.read_exact(&mut packed_file.packed_file_data)?;
        }

        // If we reach this point, we managed to get the entire PackFile decoded, so we return it.
        Ok(pack_file_data)
    }

    /// This function takes a decoded Data and encode it, so it can be saved in a PackFile file.
    ///
    /// NOTE: We return the stuff in 3 vectors to be able to use it to update the header before saving.
    pub fn save(data_decoded: &PackFileData, pack_file_id: &str) -> Vec<Vec<u8>> {
        let mut pack_file_index = vec![];
        let mut packed_file_index = vec![];
        let mut packed_file_data = vec![];

        for i in &data_decoded.pack_files {
            pack_file_index.extend_from_slice(i.as_bytes());
            pack_file_index.push(0);
        }

        for i in &data_decoded.packed_files {
            let mut packed_file_encoded = PackedFile::save(i, pack_file_id);
            packed_file_index.append(&mut packed_file_encoded.0);
            packed_file_data.append(&mut packed_file_encoded.1);
            packed_file_index.push(0);
        }

        let mut pack_file_data_encoded: Vec<Vec<u8>> = vec![];
        pack_file_data_encoded.push(pack_file_index);
        pack_file_data_encoded.push(packed_file_index);
        pack_file_data_encoded.push(packed_file_data);
        pack_file_data_encoded
    }
}

/// Implementation of "PackedFile"
impl PackedFile {

    /// This function creates an empty PackedFile.
    pub fn new() -> PackedFile {
        PackedFile {
            packed_file_size: 0,
            packed_file_path: vec![],
            packed_file_data: vec![],
        }
    }

    /// This function receive all the info of a PackedFile and creates a PackedFile with it.
    pub fn read(packed_file_size: u32, packed_file_path: Vec<String>, packed_file_data: Vec<u8>) -> PackedFile {

        PackedFile {
            packed_file_size,
            packed_file_path,
            packed_file_data,
        }
    }

    /// This function takes a decoded PackedFile and encode it, so it can be Saved inside a PackFile file.
    pub fn save(packed_file_decoded: &PackedFile, pack_file_id: &str) -> (Vec<u8>, Vec<u8>) {

        // We need to return both, the index and the data of the PackedFile, so we get them separated.
        // First, we encode the index.
        let mut packed_file_index_entry: Vec<u8> = vec![];

        // We get the file_size.
        let file_size_in_bytes = coding_helpers::encode_integer_u32(packed_file_decoded.packed_file_size);
        packed_file_index_entry.extend_from_slice(&file_size_in_bytes);

        // If it's a PFH5 (Warhammer 2), put a 0 between size and path.
        if pack_file_id == "PFH5" { packed_file_index_entry.push(0) };

        // Then we get the path, turn it into a single String and push it with the rest of the index.
        let mut path = String::new();
        for i in 0..packed_file_decoded.packed_file_path.len() {
            path.push_str(&packed_file_decoded.packed_file_path[i]);
            if (i + 1) < packed_file_decoded.packed_file_path.len() {
                path.push_str("\\");
            }
        }
        let path_in_bytes = path.as_bytes();
        packed_file_index_entry.extend_from_slice(path_in_bytes);

        // Then, we encode the data
        let packed_file_data_entry: Vec<u8> = packed_file_decoded.packed_file_data.to_vec();

        // Finally, we put both together and return them.
        (packed_file_index_entry, packed_file_data_entry)
    }
}
