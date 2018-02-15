// In this file are all the Structs and Impls required to decode and encode the PackFiles.
// For now we only support common TW: Warhammer 2 PackFiles (not loc files, those are different).
extern crate chrono;
extern crate failure;

use self::chrono::{
    NaiveDateTime, Utc
};
use std::path::PathBuf;

use self::failure::Error;

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

    /// This function creates a new empty "PackFile".
    pub fn new() -> PackFile {
        let pack_file_extra_data = PackFileExtraData::new();
        let pack_file_header = PackFileHeader::new();
        let pack_file_data = PackFileData::new();

        PackFile {
            pack_file_extra_data,
            pack_file_header,
            pack_file_data,
        }
    }

    /// This function creates a new empty "PackFile" with a name.
    pub fn new_with_name(file_name: String) -> PackFile {
        let pack_file_extra_data = PackFileExtraData::new_with_name(file_name);
        let pack_file_header = PackFileHeader::new();
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
    pub fn add_packedfiles(&mut self, packed_files: Vec<PackedFile>) {
        for packed_file in packed_files.iter() {
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
    pub fn read(pack_file_buffered: Vec<u8>, file_name: String, file_path: PathBuf) -> Result<PackFile, Error> {

        // We save the "Extra data" of the packfile
        let pack_file_extra_data = PackFileExtraData::new_from_file(file_name, file_path);

        // Then we split the PackFile encoded data into Header and Data and decode them.
        let header = &pack_file_buffered[0..28];
        let data = &pack_file_buffered[28..];
        match PackFileHeader::read(header) {
            Ok(pack_file_header) => {
                match PackFileData::read(data, pack_file_header.pack_file_count, pack_file_header.pack_file_index_size, pack_file_header.packed_file_count, pack_file_header.packed_file_index_size) {
                    Ok(pack_file_data) => {
                        Ok(PackFile {
                            pack_file_extra_data,
                            pack_file_header,
                            pack_file_data,
                        })
                    },
                    Err(error) => Err(error),
                }
            }
            Err(error) => Err(error),
        }
    }

    /// This function takes a decoded &PackFile and encode it, ready for being wrote in the disk.
    pub fn save(pack_file_decoded: &PackFile) -> Vec<u8> {
        let mut pack_file_data_encoded = PackFileData::save(&pack_file_decoded.pack_file_data);

        // Both index sizes are only needed to open and save the PackFile, so we only recalculate them
        // on save.
        let new_pack_file_index_size = pack_file_data_encoded[0].len() as u32;
        let new_packed_file_index_size = pack_file_data_encoded[1].len() as u32;
        let mut pack_file_header_encoded = PackFileHeader::save(&pack_file_decoded.pack_file_header, new_pack_file_index_size, new_packed_file_index_size);

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
    pub fn new() -> PackFileHeader {
        let pack_file_id = "PFH5".to_string();
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
    pub fn read(header: &[u8]) -> Result<PackFileHeader, Error> {

        let mut pack_file_header = PackFileHeader::new();

        pack_file_header.pack_file_id = coding_helpers::decode_string_u8((&header[0..4]).to_vec())?;
        pack_file_header.pack_file_type = coding_helpers::decode_integer_u32((&header[4..8]).to_vec())?;
        pack_file_header.pack_file_count = coding_helpers::decode_integer_u32((&header[8..12]).to_vec())?;
        pack_file_header.pack_file_index_size = coding_helpers::decode_integer_u32((&header[12..16]).to_vec())?;
        pack_file_header.packed_file_count = coding_helpers::decode_integer_u32((&header[16..20]).to_vec())?;
        pack_file_header.packed_file_index_size = coding_helpers::decode_integer_u32((&header[20..24]).to_vec())?;
        pack_file_header.packed_file_creation_time = NaiveDateTime::from_timestamp(coding_helpers::decode_integer_u32((&header[24..28]).to_vec())? as i64, 0);

        Ok(pack_file_header)
    }

    /// This function takes a decoded Header and encode it, so it can be saved in a PackFile file.
    /// We just put all the data in order in a 28 bytes Vec<u8>, and return that Vec<u8>.
    pub fn save(header_decoded: &PackFileHeader, pack_file_index_size: u32, packed_file_index_size: u32) -> Vec<u8> {
        let mut header_encoded = vec![];

        let mut pack_file_id = coding_helpers::encode_string_u8(header_decoded.pack_file_id.clone());
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
    pub fn new() -> PackFileData {
        let pack_files: Vec<String> = vec![];
        let packed_files: Vec<PackedFile> = vec![];

        PackFileData {
            pack_files,
            packed_files,
        }
    }

    /// This function checks if a PackedFile exists in a PackFile.
    /// It requires:
    /// - self: a PackFileData to check for the PackedFile.
    /// - packed_file_paths: the paths of the PackedFiles we want to check.
    pub fn packedfile_exists(&self, packed_file_path: &Vec<String>) -> bool {
        for packed_file in self.packed_files.iter() {
            if &packed_file.packed_file_path == packed_file_path {
                return true;
            }
        }
        false
    }

    /// This function checks if a folder with PackedFiles exists in a PackFile.
    /// It requires:
    /// - self: a PackFileData to check for the folder.
    /// - packed_file_paths: the path of the folder we want to check.
    pub fn folder_exists(&self, packed_file_path: &Vec<String>) -> bool {
        for packed_file in self.packed_files.iter() {
            if packed_file.packed_file_path.starts_with(&packed_file_path)
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
    /// - pack_file_count: the amount of PackFiles inside the PackFile Index. This should come from the header.
    /// - pack_index_size: the size of the index of PackFiles. This should come from the header.
    /// - packed_file_count: the amount of PackedFiles inside the PackFile. This should come from the header.
    /// - packed_index_size: the size of the index of PackedFiles. This should come from the header.
    pub fn read(
        data: &[u8],
        pack_file_count: u32,
        pack_file_size: u32,
        packed_file_count: u32,
        packed_index_size: u32
    ) -> Result<PackFileData, Error> {

        let mut pack_files: Vec<String> = vec![];
        let mut packed_files: Vec<PackedFile> = vec![];

        // We split the data into "pack_files_index", "index" and "data".
        let pack_file_index = &data[..(pack_file_size as usize)];
        let packed_file_index = &data[(pack_file_size as usize)..((packed_index_size as usize) + (pack_file_size as usize))];
        let packed_file_data = &data[((packed_index_size as usize) + (pack_file_size as usize))..];

        // Offset for the loop to get the PackFiles from the PackFile index.
        let mut pack_file_index_offset: u32 = 0;

        // First, we decode every entry in the PackFile index and store it. The process is simple:
        // we get his name char by char until hitting \u{0}, then save it and start getting the next
        // PackFile name.
        for _ in 0..pack_file_count {

            let mut pack_file_name: String = String::new();
            let mut done = false;
            while !done {
                let c = pack_file_index[pack_file_index_offset as usize] as char;

                // If the byte is \u{0}, the PackFile name is complete. We save it and update the
                // offsets to get the name of the next PackFile.
                if c.escape_unicode().to_string() == ("\\u{0}") {
                    pack_files.push(pack_file_name);
                    pack_file_name = String::new();
                    pack_file_index_offset += 1;
                    done = true;

                // If none of the options before are True, then we add the character to the current
                // PackFile name.
                } else {
                    pack_file_name.push(c);
                    pack_file_index_offset += 1;
                }
            }
        }

        // Offsets for the loop to get the file corresponding to the index entry.
        let mut packed_file_index_offset: u32 = 0;
        let mut packed_file_data_offset: u32 = 0;

        // Special offsets, to get the size and path of the PackedFiles from the index.
        let mut packed_file_index_file_size_begin_offset: u32 = 0;
        let mut packed_file_index_file_size_path_offset: u32 = 5;

        // We start a loop to decode every PackedFile
        for i in 0..packed_file_count {

            // After the first PackedFile, we update the special offsets, because the first
            // PackedFile has a byte less than the others.
            if i != 0 {
                packed_file_index_file_size_begin_offset = 1;
                packed_file_index_file_size_path_offset = 6;
            }

            // We get the size of the PackedFile (bytes 1 to 4 of the index)
            let file_size = coding_helpers::decode_integer_u32(packed_file_index[(
                    (packed_file_index_offset as usize) + packed_file_index_file_size_begin_offset as usize)
                    ..((packed_file_index_offset as usize) + 4 + (packed_file_index_file_size_begin_offset as usize))
                    ].to_vec()
            )?;

            // Then we get the Path, char by char
            let mut packed_file_index_path: Vec<String> = vec![];
            let mut packed_file_index_path_folder: String = String::new();
            let mut done = false;
            while !done {
                let c = packed_file_index[
                    (packed_file_index_offset
                        + packed_file_index_file_size_path_offset) as usize] as char;

                // If the byte is \u{5c} (\), we got a folder. We save it an continue with the next.
                // part of the path.
                if c.escape_unicode().to_string() == ("\\u{5c}") {
                    packed_file_index_path.push(packed_file_index_path_folder);
                    packed_file_index_path_folder = String::new();
                    packed_file_index_offset = packed_file_index_offset + 1;
                }

                // If the byte is \u{0}, the path is complete. We save it and update the offsets to
                // get the data from the next PackedFile.
                else if c.escape_unicode().to_string() == ("\\u{0}") {
                    packed_file_index_path.push(packed_file_index_path_folder);
                    packed_file_index_path_folder = String::new();
                    packed_file_index_offset =
                        packed_file_index_offset
                            + packed_file_index_file_size_path_offset;
                    done = true;

                // If none of the options before are True, then we add the character to the current
                // folder/file name.
                } else {
                    packed_file_index_path_folder.push(c);
                    packed_file_index_offset = packed_file_index_offset + 1;
                }
            }

            // After getting the "index" part of the PackedFile, we save the "data" part into a
            // Vec<u8> and prepare the offset for the next PackedFile.
            let packed_file_data_file_data: Vec<u8> = packed_file_data[(
                packed_file_data_offset as usize)
                ..((packed_file_data_offset as usize)
                + (file_size as usize))].into();
            packed_file_data_offset = packed_file_data_offset + file_size;

            // And finally, we create the PackedFile decoded and we push it to the Vec<PackedFile>.
            packed_files.push(PackedFile::read(file_size, packed_file_index_path, packed_file_data_file_data));
        }

        Ok(PackFileData {
            pack_files,
            packed_files,
        })
    }

    /// This function takes a decoded Data and encode it, so it can be saved in a PackFile file.
    ///
    /// NOTE: We return the stuff in 3 vectors to be able to use it to update the header before saving.
    pub fn save(data_decoded: &PackFileData) -> Vec<Vec<u8>> {
        let mut pack_file_index = vec![];
        let mut packed_file_index = vec![];
        let mut packed_file_data = vec![];

        for i in &data_decoded.pack_files {
            pack_file_index.extend_from_slice( i.as_bytes());
            pack_file_index.extend_from_slice("\0".as_bytes());
        }

        for i in &data_decoded.packed_files {
            let mut packed_file_encoded = PackedFile::save(i);
            packed_file_index.append(&mut packed_file_encoded.0);
            packed_file_data.append(&mut packed_file_encoded.1);
            packed_file_index.extend_from_slice("\0".as_bytes());
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

    /// This function receive all the info of a PackedFile and creates a PackedFile with it.
    pub fn read(packed_file_size: u32, packed_file_path: Vec<String>, packed_file_data: Vec<u8>) -> PackedFile {

        PackedFile {
            packed_file_size,
            packed_file_path,
            packed_file_data,
        }
    }

    /// This function takes a decoded PackedFile and encode it, so it can be Saved inside a PackFile file.
    pub fn save(packed_file_decoded: &PackedFile) -> (Vec<u8>, Vec<u8>) {

        // We need to return both, the index and the data of the PackedFile, so we get them separated.
        // First, we encode the index.
        let mut packed_file_index_entry: Vec<u8> = vec![];

        // We get the file_size and add a \u{0} to it.
        let file_size_in_bytes = coding_helpers::encode_integer_u32(packed_file_decoded.packed_file_size);
        packed_file_index_entry.extend_from_slice(&file_size_in_bytes);
        packed_file_index_entry.extend_from_slice("\0".as_bytes());

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