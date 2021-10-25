//---------------------------------------------------------------------------//
// Copyright (c) 2017-2020 Ismael Gutiérrez González. All rights reserved.
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

use rayon::prelude::*;
use serde_derive::{Serialize, Deserialize};

use std::convert::TryFrom;

use rpfm_error::{Error, Result};

use crate::common::{decoder::Decoder, encoder::Encoder};
use crate::packfile::{PackFileInfo, PathType};
use crate::packfile::packedfile::{PackedFile, PackedFileInfo};

pub const EXTENSION: &str = ".animpack";

//---------------------------------------------------------------------------//
//                              Enum & Structs
//---------------------------------------------------------------------------//

/// This holds an entire AnimPack PackedFile decoded in memory.
#[derive(PartialEq, Clone, Debug, Default, Serialize, Deserialize)]
pub struct AnimPack {
    packed_files: Vec<AnimPacked>,
}

/// This holds a PackedFile from inside an AnimPack in memory.
#[derive(PartialEq, Clone, Debug, Default, Serialize, Deserialize)]
pub struct AnimPacked {
    path: Vec<String>,
    data: Vec<u8>,
}

//---------------------------------------------------------------------------//
//                           Implementation of AnimPack
//---------------------------------------------------------------------------//

/// Implementation of `AnimPack`.
impl AnimPack {

    pub fn new() -> Self {
        Self::default()
    }

    /// This function creates a `AnimPack` from a `&[u8]`.
    pub fn read(packed_file_data: &[u8]) -> Result<Self> {
        let mut anim_packeds = vec![];
        let mut index = 0;

        let file_count = packed_file_data.decode_packedfile_integer_i32(index, &mut index)?;

        for _ in 0..file_count {
            let path = packed_file_data.decode_packedfile_string_u8(index, &mut index)?.split('/').map(|x| x.to_owned()).collect::<Vec<String>>();
            let byte_count = packed_file_data.decode_packedfile_integer_i32(index, &mut index)?;
            let data = packed_file_data.get_bytes_checked(index,  byte_count as usize)?.to_vec();
            index += byte_count as usize;

            anim_packeds.push(AnimPacked {
                path,
                data,
            });
        }

        // If we've reached this, we've successfully decoded the entire AnimPack.
        Ok(Self {
            packed_files: anim_packeds,
        })
    }

    /// This function takes an `AnimPack` and encodes it to `Vec<u8>`.
    pub fn save(&self) -> Vec<u8> {
        let mut data = vec![];
        data.encode_integer_i32(self.packed_files.len() as i32);

        for packed_file in &self.packed_files {
            data.encode_packedfile_string_u8(&packed_file.path.join("/"));
            data.encode_integer_i32(packed_file.data.len() as i32);
            data.extend_from_slice(&packed_file.data);
        }

        data
    }

    /// This function returns the entire list of paths contained within the provided AnimPack.
    pub fn get_file_list(&self) -> Vec<String> {
        self.packed_files.iter()
            .map(|x| x.path.join("/"))
            .collect()
    }

    pub fn get_anim_packed_paths_all(&self) -> Vec<Vec<String>> {
        self.packed_files.iter().map(|x| x.path.to_vec()).collect()
    }

    pub fn add_packed_files(&mut self, packed_files: &[PackedFile]) -> Result<Vec<PathType>> {
        let mut success_paths = vec![];
        for packed_file in packed_files {
            match self.packed_files.iter_mut().find(|x| x.path == packed_file.get_path()) {
                Some(file) => {
                    if let Ok(data) = packed_file.get_raw_data() {
                        file.data = data;
                        success_paths.push(PathType::File(packed_file.get_path().to_vec()));
                    }
                }
                None => {
                    if let Ok(anim_packed) = AnimPacked::try_from(packed_file) {
                        self.packed_files.push(anim_packed);
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

        let packed_file_info = self.packed_files.iter().map(From::from).collect();
        (pack_file_info, packed_file_info)
    }

    pub fn get_anim_packed_as_packed_files(&self, path_types: &[PathType]) -> Vec<PackedFile> {
        let paths = self.get_file_paths_from_path_types(path_types);
        self.get_packed_files_by_paths(paths.iter().map(|x| &**x).collect())
    }

    /// This function returns a copy of all `PackedFiles` in the provided `PackFile`.
    pub fn get_packed_files_all(&self) -> Vec<PackedFile> {
        self.packed_files.iter().map(From::from).collect()
    }

    /// This function returns a copy of all the `PackedFiles` starting with the provided path.
    pub fn get_packed_files_by_path_start(&self, path: &[String]) -> Vec<PackedFile> {
        self.packed_files.par_iter().filter(|x| x.get_ref_path().starts_with(path) && !path.is_empty() && x.get_ref_path().len() > path.len()).map(From::from).collect()
    }

    /// This function returns a copy of all the `PackedFiles` in the provided paths.
    pub fn get_packed_files_by_paths(&self, paths: Vec<&[String]>) -> Vec<PackedFile> {
        self.packed_files.par_iter().filter(|x| paths.contains(&x.get_ref_path())).map(From::from).collect()
    }

    /// This function returns a reference of the paths of all the `PackedFiles` in the provided `PackFile` under the provided path.
    pub fn get_ref_packed_files_paths_by_path_start(&self, path: &[String]) -> Vec<&[String]> {
        self.packed_files.par_iter().map(|x| x.get_ref_path()).filter(|x| x.starts_with(path) && !path.is_empty() && x.len() > path.len()).collect()
    }

    /// This function removes, if exists, a `PackedFile` with the provided path from the `PackFile`.
    pub fn remove_packed_file_by_path_types(&mut self, path_types: &[PathType]) {
        let paths = self.get_file_paths_from_path_types(path_types).iter().map(|x| x.to_vec()).collect::<Vec<Vec<String>>>();
        for path in &paths {
            if let Some(position) = self.packed_files.par_iter().position_any(|x| x.get_ref_path() == path) {
                self.packed_files.remove(position);
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
    }
}
