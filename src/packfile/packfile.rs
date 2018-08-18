// In this file are all the Structs and Impls required to decode and encode the PackFiles.

use std::path::PathBuf;
use std::fs::File;
use std::sync::Arc;

use error::Result;
use std::time::SystemTime;
use tw_pack_lib::PackedFile;
use tw_pack_lib::PFHVersion;
use tw_pack_lib::PFHFileType;
use tw_pack_lib::PFHFlags;
use tw_pack_lib;

/// This `Struct` stores the data of the PackFile in memory, along with some extra data needed to manipulate the PackFile.
///
/// It stores the PackFile divided in:
/// - `extra_data`: extra data that we need to manipulate the PackFile.
/// - `header`: header of the PackFile, decoded.
/// - `data`: data of the PackFile (index + data), decoded.
/// - `packed_file_indexes`: in case of Read-Only situations, like adding PackedFiles from another PackFile,
///   we can use this vector to store the indexes of the data, instead of the data per-se.
#[derive(Debug)]
pub struct PackFileView {
    pub file_path: PathBuf,
    pub pfh_version: PFHVersion,
    pub bitmask: PFHFlags,
    pub pfh_file_type: PFHFileType,
    pub pack_files: Vec<String>,
    pub timestamp: u32,
    pub packed_files: Vec<PackedFileView>,
    pub empty_folders: Vec<Vec<String>>
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PackFileExtraData {
    pub file_name: String,
    pub file_path: PathBuf,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PackFileHeader {
    pub id: String,
    pub pack_file_type: u32,
    pub pack_file_count: u32,
    pub pack_file_index_size: u32,
    pub packed_file_count: u32,
    pub packed_file_index_size: u32,
    pub creation_time: u32,

    pub data_is_encrypted: bool,
    pub index_includes_timestamp: bool,
    pub index_is_encrypted: bool,
    pub header_is_extended: bool,
}

#[derive(Clone, Debug)]
pub struct PackedFileView {
    packed_file: PackedFile,
    pub timestamp: Option<u32>,
    pub path: Vec<String>
}

/// Implementation of `PackFileView`.
impl PackFileView {

    /// This function creates a new empty `PackFileView`. This is used for creating a *dummy* PackFile.
    pub fn new() -> Self {
        Self {
            file_path: PathBuf::new(),
            pfh_version: PFHVersion::PFH5,
            bitmask: PFHFlags::empty(),
            pfh_file_type: PFHFileType::Mod,
            pack_files: vec![],
            timestamp: 0,
            packed_files: vec![],
            empty_folders: vec![]
        }
    }

    /// This function creates a new empty `PackFile` with a name and an specific id.
    pub fn new_with_name(file_name: String, pfh_version: PFHVersion) -> Self {
        let mut path = PathBuf::new();
        path.set_file_name(file_name);
        Self {
            file_path: path,
            pfh_version: pfh_version,
            bitmask: tw_pack_lib::PFHFlags::empty(),
            pfh_file_type: PFHFileType::Mod,
            pack_files: vec![],
            timestamp: 0,
            packed_files: vec![],
            empty_folders: vec![]
        }
    }

    pub fn create_extra_data(&self) -> PackFileExtraData {
        PackFileExtraData {
            file_name: self.get_file_name(),
            file_path: self.file_path.clone()
        }
    }

    pub fn create_header(&self) -> PackFileHeader {
        PackFileHeader {
            id: match self.pfh_version {
                PFHVersion::PFH4 => "PFH4".to_string(),
                PFHVersion::PFH5 => "PFH5".to_string()
            },
            pack_file_type: self.pfh_file_type.get_value(),
            pack_file_count: 0,
            pack_file_index_size: 0,
            packed_file_count: 0,
            packed_file_index_size: 0,
            creation_time: self.timestamp,

            data_is_encrypted: self.bitmask.contains(PFHFlags::HAS_ENCRYPTED_CONTENT),
            index_includes_timestamp: self.bitmask.contains(PFHFlags::HAS_INDEX_WITH_TIMESTAMPS),
            index_is_encrypted: self.bitmask.contains(PFHFlags::HAS_ENCRYPTED_INDEX),
            header_is_extended: self.bitmask.contains(PFHFlags::HAS_BIG_HEADER),
        }
    }

    pub fn get_file_name(&self) -> String {
        match self.file_path.file_name() {
            Some(s) => s.to_string_lossy().to_string(),
            None => "".to_string()
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
    pub fn add_packedfiles(&mut self, mut packed_files: Vec<PackedFileView>) {
        self.packed_files.append(&mut packed_files);
    }

    /// This function returns if the PackFile is editable or not, depending on the type of the PackFile.
    /// Basically, if the PackFile is not one of the known types OR it has any of the `pack_file_type` bitmasks
    /// as true, this'll return false. Use it to disable saving functions for PackFiles we can read but not
    /// save. Also, if the `is_editing_of_ca_packfiles_allowed` argument is false, return false for everything
    /// except types "Mod" and "Movie".
    pub fn is_editable(&self, is_editing_of_ca_packfiles_allowed: bool) -> bool {

        // If ANY of these bitmask is detected in the PackFile, disable all saving.
        if self.bitmask.contains(PFHFlags::HAS_ENCRYPTED_CONTENT) || self.bitmask.contains(PFHFlags::HAS_ENCRYPTED_INDEX) || self.bitmask.contains(PFHFlags::HAS_BIG_HEADER) { false }

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

    /// This function reads the content of a PackFile and returns a `PackFile` with all the contents of the PackFile decoded.
    ///
    /// It requires:
    /// - `file_path`: a `PathBuf` with the path of the PackFile.
    pub fn read(pack_file_path: PathBuf, load_lazy: bool) -> Result<Self> {
        let pack_file = File::open(&pack_file_path)?;
        let now = SystemTime::now();
        let parsed_pack = tw_pack_lib::parse_pack(pack_file, load_lazy).unwrap();
        let mut packed_files = vec![];
        for packed_file in parsed_pack.into_iter() {
            packed_files.push(PackedFileView::from_packedfile(packed_file))
        }
        println!("parsed pack in: {:?}. {:?}", now.elapsed().unwrap(), &parsed_pack.get_file_type());
        Ok(Self {
                file_path: pack_file_path,
                pfh_version: parsed_pack.get_version(),
                bitmask: parsed_pack.get_bitmask(),
                pfh_file_type: parsed_pack.get_file_type(),
                pack_files: vec![],
                timestamp: parsed_pack.get_timestamp(),
                packed_files: packed_files,
                empty_folders: vec![]
        })
    }

    /// This function takes a decoded `PackFile` and tries to encode it and write it on disk.
    ///
    /// It requires:
    /// - `&mut self`: the `PackFile` we are trying to save.
    /// - `mut file`: a `BufWriter` of the PackFile we are trying to write to.
    pub fn save(&mut self, pack_file_path: &PathBuf) -> Result<()> {

        // ensure every packed file has the correct timestamp and path and is loaded into memory before we (potentially) overwrite the file
        let mut packed_files = Vec::with_capacity(self.packed_files.len());
        for packed_file_view in &mut self.packed_files {
            packed_file_view.packed_file.timestamp = Some(self.timestamp);
            packed_file_view.packed_file.path = packed_file_view.path.join("\\");
            packed_files.push(&packed_file_view.packed_file)
        }

        // We try to create the file
        let mut file = File::create(pack_file_path)?;
        tw_pack_lib::build_pack_from_memory(&packed_files, &mut file, self.pfh_version, self.bitmask, self.pfh_file_type, self.timestamp).map_err(From::from)
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

    pub fn folder_exists(&self, path: &[String]) -> bool {
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

    pub fn update_empty_folders(&mut self) {
        let packed_files = &self.packed_files;
        // For each empty folder...
        self.empty_folders.retain(|folder| {
            assert!(!folder.is_empty());
            for packed_file in packed_files {
                if packed_file.path.starts_with(folder) && packed_file.path.len() > folder.len() {
                    return false
                }
            }
            true
        })
    }
}

impl PackedFileView {
    pub fn new(timestamp: Option<u32>, path: Vec<String>, data: Vec<u8>) -> Self {
        PackedFileView {
            timestamp: timestamp,
            packed_file: PackedFile::new(timestamp, path.join("\\"), data),
            path: path
        }
    }
    pub fn from_packedfile(packed_file: PackedFile) -> Self {
        PackedFileView {
            timestamp: packed_file.timestamp,
            path: packed_file.path.split("\\").map(|x| x.to_owned()).collect(),
            packed_file: packed_file
        }
    }
    pub fn get_data(&self) -> Result<Arc<Vec<u8>>> {
        Ok(self.packed_file.get_data()?)
    }

    pub fn set_data(&mut self, data: Arc<Vec<u8>>){
        self.packed_file.set_data(data);
    }
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
