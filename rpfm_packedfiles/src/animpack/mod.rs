//---------------------------------------------------------------------------//
// Copyright (c) 2017-2022 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

/*!
Module with all the code to interact with AnimPack PackedFiles.

This is a container, containing all the anim tables and related files. For each
file type, check their own module.

AnimPack's structure is very simple:
- File count.
- List of files:
    - File Path.
    - Byte Count.
!*/

use anyhow::Result;
use rayon::prelude::*;

use std::collections::HashMap;

use rpfm_common::{decoder::Decoder, encoder::Encoder, schema::Schema};
use crate::*;

pub const EXTENSION: &str = ".animpack";

//---------------------------------------------------------------------------//
//                              Enum & Structs
//---------------------------------------------------------------------------//

/// This holds an entire AnimPack PackedFile decoded in memory.
#[derive(PartialEq, Clone, Debug, Default)]
pub struct AnimPack {
    files: HashMap<String, AnimPacked>,
}

/// This holds a PackedFile from inside an AnimPack in memory.
#[derive(PartialEq, Clone, Debug, Default)]
pub struct AnimPacked {
    path: String,
    data: Vec<u8>,
}

//---------------------------------------------------------------------------//
//                           Implementation of AnimPack
//---------------------------------------------------------------------------//

/// Implementation of `AnimPack`.
impl AnimPack {
/*
    /// This function returns the entire list of paths contained within the provided AnimPack.
    pub fn get_file_list(&self) -> Vec<String> {
        self.files.iter()
            .map(|x| x.path.join("/"))
            .collect()
    }

    pub fn get_anim_packed_paths_all(&self) -> Vec<Vec<String>> {
        self.files.iter().map(|x| x.path.to_vec()).collect()
    }

    pub fn add_packed_files(&mut self, packed_files: &[PackedFile]) -> Result<Vec<PathType>> {
        let mut success_paths = vec![];
        for packed_file in packed_files {
            match self.files.iter_mut().find(|x| x.path == packed_file.get_path()) {
                Some(file) => {
                    if let Ok(data) = packed_file.get_raw_data() {
                        file.data = data;
                        success_paths.push(PathType::File(packed_file.get_path().to_vec()));
                    }
                }
                None => {
                    if let Ok(anim_packed) = AnimPacked::try_from(packed_file) {
                        self.files.push(anim_packed);
                        success_paths.push(PathType::File(packed_file.get_path().to_vec()));
                    }
                }
            }
        }
        Ok(success_paths)
    }

    pub fn get_as_pack_file_info(&self, path: &[String]) -> (PackFileInfo, Vec<PackedFileInfo>) {
        let pack_file_info = PackFileInfo {
            file_name: path.last().unwrap().to_owned(),
            ..Default::default()
        };

        let packed_file_info = self.files.iter().map(From::from).collect();
        (pack_file_info, packed_file_info)
    }

    pub fn get_anim_packed_as_packed_files(&self, path_types: &[PathType]) -> Vec<PackedFile> {
        let paths = self.get_file_paths_from_path_types(path_types);
        self.get_packed_files_by_paths(paths.iter().map(|x| &**x).collect())
    }

    /// This function returns a copy of all `PackedFiles` in the provided `PackFile`.
    pub fn get_packed_files_all(&self) -> Vec<PackedFile> {
        self.files.iter().map(From::from).collect()
    }

    /// This function returns a copy of all the `PackedFiles` starting with the provided path.
    pub fn get_packed_files_by_path_start(&self, path: &[String]) -> Vec<PackedFile> {
        self.files.par_iter().filter(|x| x.get_ref_path().starts_with(path) && !path.is_empty() && x.get_ref_path().len() > path.len()).map(From::from).collect()
    }

    /// This function returns a copy of all the `PackedFiles` in the provided paths.
    pub fn get_packed_files_by_paths(&self, paths: Vec<&[String]>) -> Vec<PackedFile> {
        self.files.par_iter().filter(|x| paths.contains(&x.get_ref_path())).map(From::from).collect()
    }

    /// This function returns a reference of the paths of all the `PackedFiles` in the provided `PackFile` under the provided path.
    pub fn get_ref_packed_files_paths_by_path_start(&self, path: &[String]) -> Vec<&[String]> {
        self.files.par_iter().map(|x| x.get_ref_path()).filter(|x| x.starts_with(path) && !path.is_empty() && x.len() > path.len()).collect()
    }

    /// This function removes, if exists, a `PackedFile` with the provided path from the `PackFile`.
    pub fn remove_packed_file_by_path_types(&mut self, path_types: &[PathType]) {
        let paths = self.get_file_paths_from_path_types(path_types).iter().map(|x| x.to_vec()).collect::<Vec<Vec<String>>>();
        for path in &paths {
            if let Some(position) = self.files.par_iter().position_any(|x| x.get_ref_path() == path) {
                self.files.remove(position);
            }
        }
    }

    pub fn get_file_paths_from_path_types(&self, path_types: &[PathType]) -> Vec<Vec<String>> {

        // Keep the PathTypes added so we can return them to the UI easily.
        let path_types = PathType::dedup(path_types);

        // As this can get very slow very quickly, we do here some... optimizations.
        // First, we get if there are PackFiles or folders in our list of PathTypes.
        let we_have_packfile = path_types.par_iter().any(|item| {
            matches!(item, PathType::PackFile)
        });

        let we_have_folder = path_types.par_iter().any(|item| {
            matches!(item, PathType::Folder(_))
        });

        // Then, if we have a PackFile,... just import all PackedFiles.
        if we_have_packfile {
            self.get_anim_packed_paths_all()
        }

        // If we only have files, get all the files we have at once, then add them all together.
        else if !we_have_folder {
            path_types.par_iter().filter_map(|x| {
                if let PathType::File(path) = x { Some(path.to_vec()) } else { None }
            }).collect::<Vec<Vec<String>>>()
        }

        // Otherwise, we have a mix of Files and Folders (or folders only).
        // In this case, we get all the individual files, then the ones inside folders.
        // Then we merge them, and add all of them together.
        else {
            let mut paths_files = path_types.par_iter().filter_map(|x| {
                if let PathType::File(path) = x { Some(path.to_vec()) } else { None }
            }).collect::<Vec<Vec<String>>>();

            paths_files.append(&mut path_types.par_iter()
                .filter_map(|x| {
                    if let PathType::Folder(path) = x { Some(path.to_vec()) } else { None }
                })
            .map(|path| self.get_ref_packed_files_paths_by_path_start(&path).iter().map(|x| x.to_vec()).collect::<Vec<Vec<String>>>())
            .flatten()
            .collect::<Vec<Vec<String>>>());
            paths_files
        }
    }
}

/// Implementation of AnimPacked.
impl AnimPacked {
    pub fn get_ref_data(&self) -> &[u8] {
        &self.data
    }

    pub fn get_ref_path(&self) -> &[String] {
        &self.path
    }
}

/// Implementation to create an `AnimPacked` from a `PackedFile`.
impl TryFrom<&PackedFile> for AnimPacked {

    type Error = Error;

    fn try_from(packed_file: &PackedFile) -> Result<Self> {
        let anim_packed = Self {
            path: packed_file.get_path().to_vec(),
            data: packed_file.get_raw_data()?,
        };
        Ok(anim_packed)
    }
}

impl From<&AnimPacked> for PackedFileInfo {
    fn from(anim_packed: &AnimPacked) -> Self {
        let packed_file_info = Self {
            path: anim_packed.get_ref_path().to_vec(),
            ..Default::default()
        };
        packed_file_info
    }*/
}

impl Decodeable for AnimPack {

    fn file_type(&self) -> PackedFileType {
        PackedFileType::AnimPack
    }

    fn decode(packed_file_data: &[u8], extra_data: Option<(&Schema, &str, bool)>) -> Result<Self> {
        let mut index = 0;

        let file_count = packed_file_data.decode_packedfile_integer_u32(index, &mut index)?;
        let mut files = if file_count < 500_000 { HashMap::with_capacity(file_count as usize) } else { HashMap::new() };

        for _ in 0..file_count {
            let path = packed_file_data.decode_packedfile_string_u8(index, &mut index)?;
            let byte_count = packed_file_data.decode_packedfile_integer_u32(index, &mut index)? as usize;
            let data = packed_file_data.decode_bytes_checked(index,  byte_count)?.to_vec();
            index += byte_count;

            files.insert(path.to_owned(), AnimPacked{path, data});
        }

        // If we've reached this, we've successfully decoded the entire AnimPack.
        Ok(Self {
            files,
        })
    }
}

impl Encodeable for AnimPack {
    fn encode(&self) -> Vec<u8> {
        let mut data = vec![];
        data.encode_integer_u32(self.files.len() as u32);

        // TODO: check if sorting is needed.
        for packed_file in self.files.values() {
            data.encode_packedfile_string_u8(&packed_file.path);
            data.encode_integer_u32(packed_file.data.len() as u32);
            data.extend_from_slice(&packed_file.data);
        }

        data
    }
}


impl Container for AnimPack {
    type T = AnimPacked;

    fn insert(&mut self, file: Self::T) -> ContainerPath {
        let path = file.path();
        let path_raw = file.path_raw();
        self.files.insert(path_raw.to_owned(), file);
        path
    }

    fn remove(&mut self, path: &ContainerPath) -> Vec<ContainerPath> {
        match path {
            ContainerPath::File(path) => {
                self.files.remove(path);
                return vec![ContainerPath::File(path.to_owned())];
            },
            ContainerPath::Folder(path) => {
                let paths_to_remove = self.files.par_iter()
                    .filter_map(|(key, _)| if key.starts_with(path) { Some(key.to_owned()) } else { None }).collect::<Vec<String>>();

                paths_to_remove.iter().for_each(|path| {
                    self.files.remove(path);
                });
                return paths_to_remove.par_iter().map(|path| ContainerPath::File(path.to_string())).collect();
            },
            ContainerPath::FullContainer => {
                self.files.clear();
                return vec![ContainerPath::FullContainer];
            },
        }
    }

    fn files(&self, path: &ContainerPath) -> Vec<&Self::T> {
        match path {
            ContainerPath::File(path) => {
                match self.files.get(path) {
                    Some(file) => vec![file],
                    None => vec![],
                }
            },
            ContainerPath::Folder(path) => {
                self.files.par_iter()
                    .filter_map(|(key, file)|
                        if key.starts_with(path) { Some(file) } else { None }
                    ).collect::<Vec<&Self::T>>()
            },
            ContainerPath::FullContainer => {
                self.files.values().collect()
            },
        }
    }

    fn paths(&self) -> Vec<ContainerPath> {
        self.files.par_iter().map(|(path, _)| ContainerPath::File(path.to_owned())).collect()
    }

    fn paths_raw(&self) -> Vec<&str> {
        self.files.par_iter().map(|(path, _)| &**path).collect()
    }
}

impl Containerizable for AnimPacked {
    fn data(&self) -> &[u8] {
        &self.data
    }
    fn path(&self) -> ContainerPath {
        ContainerPath::File(self.path.to_owned())
    }
    fn path_raw(&self) -> &str {
        &self.path
    }
}
