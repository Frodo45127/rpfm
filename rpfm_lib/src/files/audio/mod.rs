//---------------------------------------------------------------------------//
// Copyright (c) 2017-2023 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! This is a module to read/write audio files in a generic way.
//!
//! Supported formats:
//! - .mp3

use getset::*;
use serde_derive::{Serialize, Deserialize};

use crate::binary::{ReadBytes, WriteBytes};
use crate::error::Result;
use crate::files::{DecodeableExtraData, Decodeable, EncodeableExtraData, Encodeable};

/// Extension used by audio files.
///
/// NOTE: .wem files are not yet included because it's not yet supported by any view.
///pub const EXTENSIONS: [&str; 2] = [".mp3", ".wem"];
pub const EXTENSIONS: [&str; 1] = [".mp3"];

//---------------------------------------------------------------------------//
//                              Enum & Structs
//---------------------------------------------------------------------------//

/// This struct contains an unknown file in memory.
#[derive(Clone, Debug, PartialEq, Eq, Getters, Setters, Serialize, Deserialize)]
#[getset(get = "pub")]
pub struct Audio {
    data: Vec<u8>,
}

//---------------------------------------------------------------------------//
//                              Implementations
//---------------------------------------------------------------------------//

impl Decodeable for Audio {

    fn decode<R: ReadBytes>(data: &mut R, _extra_data: &Option<DecodeableExtraData>) -> Result<Self> {
        let len = data.len()?;
        let data = data.read_slice(len as usize, false)?;
        Ok(Self {
            data,
        })
    }
}

impl Encodeable for Audio {

    fn encode<W: WriteBytes>(&mut self, buffer: &mut W, _extra_data: &Option<EncodeableExtraData>) -> Result<()> {
        buffer.write_all(&self.data).map_err(From::from)
    }
}
