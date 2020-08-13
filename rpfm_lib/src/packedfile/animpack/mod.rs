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

use serde_derive::{Serialize, Deserialize};

use rpfm_error::Result;

use crate::common::{decoder::Decoder, encoder::Encoder};
use crate::packfile::PackFile;
use crate::packfile::packedfile::PackedFile;

pub const EXTENSION: &str = ".animpack";

pub const DEFAULT_PATH: [&str; 3] = ["animations", "animation_tables", "animation_tables.animpack"];

//---------------------------------------------------------------------------//
//                              Enum & Structs
//---------------------------------------------------------------------------//

/// This holds an entire AnimPack PackedFile decoded in memory.
#[derive(PartialEq, Clone, Debug, Default, Serialize, Deserialize)]
pub struct AnimPack {
    packed_files: Vec<AnimPacked>,
}

/// This holds a PackedFile from inside an AnimPack in memory.
#[derive(PartialEq, Clone, Debug, Serialize, Deserialize)]
pub struct AnimPacked {
    path: Vec<String>,
    data: Vec<u8>,
}

//---------------------------------------------------------------------------//
//                           Implementation of AnimPack
//---------------------------------------------------------------------------//

/// Implementation of `AnimPack`.
impl AnimPack {

    /// This function creates a valid AnimPack. With `valid` I mean with one file inside. The game crashes otherwise.
    pub fn new() -> Self {
        Self {
            packed_files: vec![AnimPacked {
                path: vec!["yuri".to_owned(), "zahard".to_owned(), "best".to_owned(), "waifu".to_owned()],
                data: vec![],
            }],
        }
    }

    /// This function creates a `AnimPack` from a `&[u8]`.
    pub fn read(packed_file_data: &[u8]) -> Result<Self> {
        let mut anim_packeds = vec![];
        let mut index = 0;

        let file_count = packed_file_data.decode_packedfile_integer_i32(index, &mut index)?;

        for _ in 0..file_count {
            let path = packed_file_data.decode_packedfile_string_u8(index, &mut index)?.split('/').map(|x| x.to_owned()).collect::<Vec<String>>();
            let byte_count = packed_file_data.decode_packedfile_integer_i32(index, &mut index)?;
            let data = packed_file_data[index..index + byte_count as usize].to_vec();
            index += byte_count as usize;

            anim_packeds.push(AnimPacked {
                path,
                data,
            });
        }

        // If we've reached this, we've succesfully decoded the entire AnimPack.
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

    /// This function unpacks the entire AnimPack into the current PackFile.
    pub fn unpack(&self, pack_file: &mut PackFile) -> Result<Vec<Vec<String>>> {
        let packed_files = self.packed_files.iter()
            .map(From::from)
            .collect::<Vec<PackedFile>>();
        let packed_files = packed_files.iter().collect::<Vec<&PackedFile>>();
        pack_file.add_packed_files(&packed_files, true)
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

