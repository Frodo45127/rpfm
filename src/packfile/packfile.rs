// In this file are all the Structs and Impls required to decode and encode the PackFiles.
use tw_pack_lib::{PackedFile, PFHVersion, PFHFileType, PFHFlags, parse_pack, build_pack_from_memory};

use std::path::PathBuf;
use std::fs::File;
use std::sync::Arc;
use std::time::SystemTime;

use common::*;
use error::Result;

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
    pub pfh_file_type: PFHFileType,
    pub bitmask: PFHFlags,
    pub timestamp: u32,

    pub pack_files: Vec<String>,
    pub packed_files: Vec<PackedFileView>,
    pub empty_folders: Vec<Vec<String>>
}

#[derive(Debug)]
pub struct PackFileUIData {
    pub file_path: PathBuf,
    pub pfh_version: PFHVersion,
    pub pfh_file_type: PFHFileType,
    pub bitmask: PFHFlags,
    pub timestamp: u32,
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

    /// This function reads the content of a PackFile and returns a `PackFileView` with the contents of the PackFile decoded.
    ///
    /// It requires:
    /// - `file_path`: a `PathBuf` with the path of the PackFile.
    /// - `lazy_loading`: if the PackFile should be lazy-loaded. This should be disabled for Warhammer 2 when opening from /data due to a bug in TWeaK.
    pub fn read(file_path: PathBuf, lazy_loading: bool) -> Result<Self> {
        
        let now = SystemTime::now();
        let pack_file = File::open(&file_path)?;
        let parsed_pack = parse_pack(pack_file, lazy_loading)?;
        let mut packed_files = vec![];
        for packed_file in parsed_pack.into_iter() {
            packed_files.push(PackedFileView::from(packed_file))
        }
        println!("parsed pack in: {:?}. {:?}", now.elapsed().unwrap(), &parsed_pack.get_file_type());
        Ok(Self {
            file_path,
            pfh_version: parsed_pack.get_version(),
            bitmask: parsed_pack.get_bitmask(),
            pfh_file_type: parsed_pack.get_file_type(),
            timestamp: parsed_pack.get_timestamp(),
            
            pack_files: parsed_pack.get_pack_file_index(),
            packed_files,
            empty_folders: vec![]
        })
    }

    /// This function takes a decoded `PackFile` and tries to encode it and write it on disk.
    ///
    /// It requires:
    /// - `&mut self`: the `PackFile` we are trying to save.
    /// - `mut file`: a `BufWriter` of the PackFile we are trying to write to.
    pub fn save(&mut self, save_as_another_file: bool) -> Result<()> {

        // If we are overwriting our own file (with a normal "Save"), ensure that all the data is loaded to memory before saving.
        if !save_as_another_file { 
            for packed_file in &self.packed_files {
                packed_file.get_data()?;
            }
        }

        let mut packed_files: Vec<&PackedFile> = Vec::with_capacity(self.packed_files.len());
        self.packed_files.iter_mut().for_each(|x| packed_files.push(From::from(x)));
        
        // We try to create the file
        let mut file = File::create(&self.file_path)?;
        build_pack_from_memory(&mut packed_files, &mut file, self.pfh_version, self.bitmask, self.pfh_file_type, get_current_time(), &self.pack_files).map_err(From::from)
    }
    
    /// This function checks if a `PackedFile` exists in a `PackFile`.
    ///
    /// It requires:
    /// - `&self`: a `PackFileView` to check for the `PackedFile`.
    /// - `path`: the path of the `PackedFile` we want to check.
    pub fn packedfile_exists(&self, path: &[String]) -> bool {
        for packed_file in &self.packed_files {
            if packed_file.path == path {
                return true;
            }
        }
        false
    }

    /// This function checks if a `PackedFile` exists in a `PackFile`.
    ///
    /// It requires:
    /// - `&self`: a `PackFileData` to check for the `PackedFile`.
    /// - `path`: the path of the folder we want to check.
    pub fn folder_exists(&self, path: &[String]) -> bool {

        // starts_with() triggers a false positive if the path is empty, so we have to check that first.
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
}

impl PackedFileView {

    /// This function serves to create a new `PackedFileView` from existing data.
    ///
    /// It requires:
    /// - `timestamp`: last modified time of the PackedFile, in `u32`. Optional.
    /// - `path`: the path of the PackedFile.
    /// - `data`: the raw data of the PackedFile.
    pub fn new(timestamp: Option<u32>, path: Vec<String>, data: Vec<u8>) -> Self {
        PackedFileView {
            timestamp,
            packed_file: PackedFile::new(timestamp, path.join("\\"), data),
            path: path
        }
    }

    /// Getter for the PackedFile's Data.
    pub fn get_data(&self) -> Result<Arc<Vec<u8>>> {
        Ok(self.packed_file.get_data()?)
    }

    /// Setter for the PackedFile's Data.
    pub fn set_data(&mut self, data: Arc<Vec<u8>>){
        self.packed_file.set_data(data);
    }
}

/// Implementeations to quickly convert from PackedFile (PackedFile in tw_pack_lib) to PackedFileView (PackedFile in RPFM).
impl From<PackedFile> for PackedFileView {
    fn from(packed_file: PackedFile) -> PackedFileView {
        PackedFileView {
            timestamp: packed_file.timestamp,
            path: packed_file.path.split("\\").map(|x| x.to_owned()).collect(),
            packed_file: packed_file
        }
    }
}

/// Implementation to quickly convert from PackedFileView (PackedFile in RPFM) to PackedFile (PackedFile in tw_pack_lib).
impl<'a> From<&'a mut PackedFileView> for &'a PackedFile {
    fn from(packed_file_view: &mut PackedFileView) -> &PackedFile {
        packed_file_view.packed_file.timestamp = packed_file_view.timestamp;
        packed_file_view.packed_file.path = packed_file_view.path.join("\\");
        &packed_file_view.packed_file
    }
}

