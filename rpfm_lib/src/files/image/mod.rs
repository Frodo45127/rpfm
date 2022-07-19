//---------------------------------------------------------------------------//
// Copyright (c) 2017-2022 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! This is a dummy module to read/write images.
//!
//! Read support just stores the raw data of the image, so you can pass it to another
//! lib/program to read it. Write support just writes that data back to the source.
//!
//! Supported extensions are:
//! - `.jpg`
//! - `.jpeg`
//! - `.tga`
//! - `.png`
//! - `.dds`

use getset::*;
use serde_derive::{Serialize, Deserialize};

use crate::binary::{ReadBytes, WriteBytes};
use crate::error::Result;
use crate::files::{DecodeableExtraData, Decodeable, EncodeableExtraData, Encodeable};

/// Extensions used by Images.
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

/// This represents an entire Image File decoded in memory.
#[derive(Default, PartialEq, Clone, Debug, Getters, Serialize, Deserialize)]
#[getset(get = "pub")]
pub struct Image {
    data: Vec<u8>,
}

//---------------------------------------------------------------------------//
//                           Implementation of Image
//---------------------------------------------------------------------------//

impl Decodeable for Image {

    fn decode<R: ReadBytes>(data: &mut R, _extra_data: &Option<DecodeableExtraData>) -> Result<Self> {
        let len = data.len()?;
        let data = data.read_slice(len as usize, false)?;
        Ok(Self {
            data,
        })
    }
}

impl Encodeable for Image {

    fn encode<W: WriteBytes>(&mut self, buffer: &mut W, _extra_data: &Option<EncodeableExtraData>) -> Result<()> {
        buffer.write_all(&self.data).map_err(From::from)
    }
}
