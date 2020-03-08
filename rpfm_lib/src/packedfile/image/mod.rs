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
Module with all the code to interact with Image PackedFiles.

Images... we really just get their that to memory. Nothing more.
!*/

use serde_derive::{Serialize, Deserialize};

use rpfm_error::Result;

/// Extensions used by Image PackedFiles.
pub const EXTENSIONS: [&str; 5] = [
    ".jpg",
    ".jpeg",
    ".tga",
    ".dds",
    ".png",
];

//---------------------------------------------------------------------------//
//                              Enum & Structs
//---------------------------------------------------------------------------//

/// This holds an entire Image PackedFile decoded in memory.
#[derive(PartialEq, Clone, Debug, Serialize, Deserialize)]
pub struct Image {

    /// The raw_data of the image.
    data: Vec<u8>,
}

//---------------------------------------------------------------------------//
//                           Implementation of Image
//---------------------------------------------------------------------------//

/// Implementation of `Default` for `Image`.
impl Default for Image {
    fn default() -> Self {
        Self {
            data: vec![],
        }
    }
}

/// Implementation of `Image`.
impl Image {

    /// This function creates a new empty `Image`. Akin to `default()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// This function creates a `Image` from a `Vec<u8>`.
    pub fn read(packed_file_data: &[u8]) -> Result<Self> {
        Ok(Self {
            data: packed_file_data.to_vec(),
        })
    }

    /// This function returns the data the provided `Image`.
    pub fn get_data(&self) -> &[u8] {
        &self.data
    }
}
