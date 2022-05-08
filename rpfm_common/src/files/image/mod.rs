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
Module with all the code to interact with Image PackedFiles.

Images... we really just get their that to memory. Nothing more.
!*/

use crate::error::Result;

use crate::schema::Schema;

use crate::files::{Decodeable, FileType};

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
#[derive(Default, PartialEq, Clone, Debug)]
pub struct Image {

    /// The raw_data of the image.
    data: Vec<u8>,
}

//---------------------------------------------------------------------------//
//                           Implementation of Image
//---------------------------------------------------------------------------//

/// Implementation of `Image`.
impl Image {

    /// This function returns the data the provided `Image`.
    pub fn get_data(&self) -> &[u8] {
        &self.data
    }
}

/// Implementation of Decodeable for `Image` PackedFile Type.
impl Decodeable for Image {
    fn file_type(&self) -> FileType {
        FileType::Image
    }

    /// This function creates a `Image` from a `Vec<u8>`.
    fn decode(packed_file_data: &[u8], _extra_data: Option<(&Schema, &str, bool)>) -> Result<Self> {
        Ok(Self {
            data: packed_file_data.to_vec(),
        })
    }
}
