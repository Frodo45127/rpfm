// In this file are all the Structs and Impls required to decode and encode the PackFiles.
// For now we only support common TW: Warhammer 2 PackFiles (not loc files, those are different).

extern crate byteorder;

use ::common;
use std::u32;

use self::byteorder::{
    ReadBytesExt, BigEndian
};

// Struct PackFile: This stores the data of the entire PackFile in memory ('cause fuck lazy-loading),
// along with some extra data needed to manipulate the PackFile.
// It stores the PackFile divided in 3 structs:
// - pack_file_extra_data: extra data that we need to manipulate the PackFile.
// - pack_file_header: header of the PackFile, decoded.
// - pack_file_data: data of the PackFile, decoded.
#[derive(Clone)]
pub struct PackFile {
    pub pack_file_extra_data: PackFileExtraData,
    pub pack_file_header: PackFileHeader,
    pub pack_file_data: PackFileData,
}

// Struct PackFileExtraData: This struct stores some extra data we need to manipulate the PackFiles:
// - file_name: name of the PackFile.
// - file_path: current path of the PackFile in the FileSystem.
// - correlation_data: Vector with all the paths that are already in the TreeView. Useful for checking.
#[derive(Clone)]
pub struct PackFileExtraData {
    pub file_name: String,
    pub file_path: String,
    pub correlation_data: Vec<Vec<String>>,
}


// Struct PackFileHeader: This struct stores all the info we can get from the header of the PackFile:
// - pack_file_id: ID of the PackFile, like a version.
// - pack_file_type: type of the PackFile (mod, movie,...).
// - version: no idea. It was checked in PFM, so we save it, just in case.
// - replaced_pack_file_name: no idea. It was checked in PFM, so we save it, just in case.
// - file_count: amount of files stored inside the PackFile.
// - index_size: size in bytes of the Index of the file (the first part of the data).
// - unknown_data: no idea. It was checked in PFM, so we save it, just in case.
//
// NOTE: to understand the "pack_file_type":
// 0 => "Boot",
// 1 => "Release",
// 2 => "Patch",
// 3 => "Mod",
// 4 => "Movie",
#[derive(Clone)]
pub struct PackFileHeader {
    pub pack_file_id: String,
    pub pack_file_type: u32,
    pub version: u32,
    pub replaced_pack_file_name: u32,
    pub file_count: u32,
    pub index_size: u32,
    pub unknown_data: u32,
}

// Struct PackFileData: This struct stores all the PackedFiles inside the PackFile in a vector.
#[derive(Clone)]
pub struct PackFileData {
    pub packed_files: Vec<PackedFile>,
}

// Struct PackedFile: This struct stores the data of a PackedFile:
// - packed_file_size: size of the data.
// - packed_file_path: path of the PackedFile inside the PackFile.
// - packed_file_data: the data of the PackedFile. Temporal, until we implement PackedFileTypes.
#[derive(Clone)]
pub struct PackedFile {
    pub packed_file_size: u32,
    pub packed_file_path: Vec<String>,
    pub packed_file_data: Vec<u8>,
}

// Implementation of "PackFileExtraData"
impl  PackFileExtraData {

    // This function creates an empty PackFileExtraData.
    pub fn new() -> PackFileExtraData {
        let file_name = String::new();
        let file_path = String::new();
        let correlation_data = vec![vec![]];
        PackFileExtraData {
            file_name,
            file_path,
            correlation_data,
        }
    }

    // This function creates a PackFileExtraData with just a name.
    pub fn new_with_name(file_name: String) -> PackFileExtraData {
        let file_path = String::new();
        let correlation_data = vec![vec![]];
        PackFileExtraData {
            file_name,
            file_path,
            correlation_data,
        }
    }

    // This function creates a PackFileExtraData with a name and a path.
    pub fn new_from_file(file_name: String, file_path: String) -> PackFileExtraData {
        let correlation_data = vec![vec![]];
        PackFileExtraData {
            file_name,
            file_path,
            correlation_data,
        }
    }
}

// Implementation of "PackFile"
impl PackFile {

    // This function creates a new empty "PackFile".
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

    // This function creates a new empty "PackFile" with a name.
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

    // This function reads the content of a PackFile and returns an struct PackFile with all the
    // contents of the PackFile decoded.
    // It requires:
    // - pack_file_buffered: a Vec<u8> with the entire PackFile encoded inside it.
    // - file_name: a String with the name of the PackFile.
    // - file_path: a String with the path of the PackFile.
    pub fn read(pack_file_buffered: Vec<u8>, file_name: String, file_path: String) -> PackFile {

        // We save the "Extra data" of the packfile
        let pack_file_extra_data = PackFileExtraData::new_from_file(file_name, file_path);

        // Then we split the PackFile encoded data into Header and Data and decode them.
        let header = &pack_file_buffered[0..28];
        let data = &pack_file_buffered[28..];
        let pack_file_header = PackFileHeader::read(header);
        let pack_file_data = PackFileData::read(data, pack_file_header.file_count, pack_file_header.index_size);

        // And return the PackFile decoded.
        PackFile {
            pack_file_extra_data,
            pack_file_header,
            pack_file_data,
        }
    }

    // This function takes a decoded &PackFile and encode it, ready for being wrote in the disk.
    pub fn save(pack_file_decoded: &PackFile) -> Vec<u8> {
        let mut pack_file_header_encoded = PackFileHeader::save(&pack_file_decoded.pack_file_header);
        let mut pack_file_data_encoded = PackFileData::save(&pack_file_decoded.pack_file_data);

        let mut pack_file_encoded = vec![];
        pack_file_encoded.append(&mut pack_file_header_encoded);
        pack_file_encoded.append(&mut pack_file_data_encoded);
        pack_file_encoded
    }
}

// Implementation of "PackFileHeader"
impl PackFileHeader {

    // This function creates a new PackFileHeader for an empty PackFile of Warhammer 2.
    pub fn new() -> PackFileHeader {
        let pack_file_id = "PFH5".to_string();
        let pack_file_type = 3 as u32;
        let version = 0 as u32;
        let replaced_pack_file_name = 0 as u32;
        let file_count = 0 as u32;
        let index_size = 0 as u32;
        let unknown_data = 0 as u32;

        PackFileHeader {
            pack_file_id,
            pack_file_type,
            version,
            replaced_pack_file_name,
            file_count,
            index_size,
            unknown_data,
        }
    }

    // This function reads the Header of a PackFile and decode it into a PackFileHeader. We read all
    // this data in packs of 4 bytes, then we put them together, reverse them and read them.
    pub fn read(header: &[u8]) -> PackFileHeader {

        // ID
        let pack_file_id = common::latin1_to_string(&header[0..4]);

        // PackFile Type
        let mut pack_file_type: Vec<u8> = header[4..8].into();
        pack_file_type.reverse();
        let mut pack_file_type = &pack_file_type[0..4];
        let pack_file_type: u32 = pack_file_type.read_u32::<BigEndian>().unwrap();

        // Version
        let mut version: Vec<u8> = header[8..12].into();
        version.reverse();
        let mut version = &version[0..4];
        let version: u32 = version.read_u32::<BigEndian>().unwrap();

        // "replaced_pack_file_name"
        let mut replaced_pack_file_name: Vec<u8> = header[12..16].into();
        replaced_pack_file_name.reverse();
        let mut replaced_pack_file_name = &replaced_pack_file_name[0..4];
        let replaced_pack_file_name: u32 = replaced_pack_file_name.read_u32::<BigEndian>().unwrap();

        // File count
        let mut file_count: Vec<u8> = header[16..20].into();
        file_count.reverse();
        let mut file_count = &file_count[0..4];
        let file_count: u32 = file_count.read_u32::<BigEndian>().unwrap();

        // Index size
        let mut index_size: Vec<u8> = header[20..24].into();
        index_size.reverse();
        let mut index_size = &index_size[0..4];
        let index_size: u32 = index_size.read_u32::<BigEndian>().unwrap();

        // Unknown data
        let mut unknown_data: Vec<u8> = header[24..28].into();
        unknown_data.reverse();
        let mut unknown_data = &unknown_data[0..4];
        let unknown_data: u32 = unknown_data.read_u32::<BigEndian>().unwrap();

        PackFileHeader {
            pack_file_id,
            pack_file_type,
            version,
            replaced_pack_file_name,
            file_count,
            index_size,
            unknown_data,
        }
    }

    // This function takes a decoded Header and encode it, so it can be saved in a PackFile file.
    // We just put all the data in order in a 28 bytes Vec<u8>, and return that Vec<u8>.
    pub fn save(header_decoded: &PackFileHeader) -> Vec<u8> {
        let mut header_encoded = vec![];

        let pack_file_id = &header_decoded.pack_file_id;
        let pack_file_type = common::u32_to_u8_reverse(header_decoded.pack_file_type);
        let version = common::u32_to_u8_reverse(header_decoded.version);
        let replaced_pack_file_name = common::u32_to_u8_reverse(header_decoded.replaced_pack_file_name);
        let file_count = common::u32_to_u8_reverse(header_decoded.file_count);
        let index_size = common::u32_to_u8_reverse(header_decoded.index_size);
        let unknown_data = common::u32_to_u8_reverse(header_decoded.unknown_data);

        header_encoded.extend_from_slice(&pack_file_id.as_bytes());
        header_encoded.extend_from_slice(&pack_file_type);
        header_encoded.extend_from_slice(&version);
        header_encoded.extend_from_slice(&replaced_pack_file_name);
        header_encoded.extend_from_slice(&file_count);
        header_encoded.extend_from_slice(&index_size);
        header_encoded.extend_from_slice(&unknown_data);
        header_encoded
    }
}

// Implementation of "PackFileData"
impl PackFileData {

    // This function creates a new empty "PackFileData"
    pub fn new() -> PackFileData {
        let packed_files: Vec<PackedFile> = vec![];

        PackFileData {
            packed_files,
        }
    }

    // This function reads the Data part of a PackFile, get all the files on the PackFile and put
    // them in a Vec<PackedFile>.
    // It requires:
    // - data: the raw data or the PackFile.
    // - file_count: the amount of PackedFiles inside the PackFile. This should come from the header.
    // - index_size: the size of the index of PackedFiles. This should come from the header.
    pub fn read(data: &[u8], file_count: u32, index_size: u32) -> PackFileData {

        let mut packed_files: Vec<PackedFile> = vec![];

        // We split the data into "index" and "data".
        let packed_file_index = &data[..(index_size as usize)];
        let packed_file_data = &data[(index_size as usize)..];

        // Offsets for the loop to get the file corresponding to the index entry.
        let mut packed_file_index_offset: u32 = 0;
        let mut packed_file_data_offset: u32 = 0;

        // Special offsets, to get the size and path of the PackedFiles from the index.
        let mut packed_file_index_file_size_begin_offset: u32 = 0;
        let mut packed_file_index_file_size_path_offset: u32 = 5;

        // We start a loop to decode every PackedFile
        for i in 0..file_count {

            // After the first PackedFile, we update the special offsets, because the first
            // PackedFile has a byte less than the others.
            if i != 0 {
                packed_file_index_file_size_begin_offset = 1;
                packed_file_index_file_size_path_offset = 6;
            }

            // We get the size of the PackedFile (bytes 1 to 4 of the index)
            let mut file_size: Vec<u8> = packed_file_index[
                ((packed_file_index_offset as usize)
                    + packed_file_index_file_size_begin_offset as usize)
                    ..((packed_file_index_offset as usize)
                    + 4
                    + (packed_file_index_file_size_begin_offset as usize))].into();
            file_size.reverse();
            let mut file_size = &file_size[0..4];
            let file_size: u32 = file_size.read_u32::<BigEndian>().unwrap();

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

        PackFileData {
            packed_files,
        }
    }

    // This function takes a decoded Data and encode it, so it can be saved in a PackFile file.
    pub fn save(data_decoded: &PackFileData) -> Vec<u8> {

        let mut pack_file_index = vec![];
        let mut pack_file_data = vec![];

        for i in &data_decoded.packed_files {
            let mut packed_file_encoded = PackedFile::save(i);
            pack_file_index.append(&mut packed_file_encoded.0);
            pack_file_data.append(&mut packed_file_encoded.1);
            pack_file_index.extend_from_slice("\0".as_bytes());
        }

        let mut pack_file_data_encoded = vec![];
        pack_file_data_encoded.append(&mut pack_file_index);
        pack_file_data_encoded.append(&mut pack_file_data);
        pack_file_data_encoded
    }
}

// Implementation of "PackedFile"
impl PackedFile {

    // This function adds a new PackedFile the the PackFile, from his size, path and data.
    // It requires:
    // - packed_file_size: the size in bytes of the data of the PackedFile.
    // - packed_file_path: the path of the PackedFile.
    // - packed_file_data: the data of the PackedFile.
    pub fn add(packed_file_size: u32, packed_file_path: Vec<String>, packed_file_data: Vec<u8>) -> PackedFile {
        PackedFile {
            packed_file_size,
            packed_file_path,
            packed_file_data,
        }
    }

    // This function receive all the info of a PackedFile and creates a PackedFile with it.
    pub fn read(file_size: u32, path: Vec<String>, data: Vec<u8>) -> PackedFile {

        let packed_file_size = file_size;
        let packed_file_path = path;
        let packed_file_data = data;

        PackedFile {
            packed_file_size,
            packed_file_path,
            packed_file_data,
        }
    }

    // This function takes a decoded PackedFile and encode it, so it can be Saved inside a PackFile file.
    pub fn save(packed_file_decoded: &PackedFile) -> (Vec<u8>, Vec<u8>) {

        // We need to return both, the index and the data of the PackedFile, so we get them separated.
        // First, we encode the index.
        let mut packed_file_index_entry: Vec<u8> = vec![];

        // We get the file_size and add a \u{0} to it.
        let file_size_in_bytes = common::u32_to_u8_reverse(packed_file_decoded.packed_file_size);
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
        let packed_file_data_encoded = (packed_file_index_entry, packed_file_data_entry);
        packed_file_data_encoded
    }
}