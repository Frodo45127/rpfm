//---------------------------------------------------------------------------//
// Copyright (c) 2017-2024 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! This is a dummy module to read/write files of type unknown.
//!
//! Internally, this just stores the raw data so you can manipulate it at will.

use getset::*;
use serde_derive::{Serialize, Deserialize};

use crate::binary::{ReadBytes, WriteBytes};
use crate::error::Result;
use crate::files::{DecodeableExtraData, Decodeable, EncodeableExtraData, Encodeable};

//---------------------------------------------------------------------------//
//                              Enum & Structs
//---------------------------------------------------------------------------//

/// This struct contains an unknown file in memory.
#[derive(Clone, Debug, PartialEq, Eq, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct Unknown {
    data: Vec<u8>,
}

//---------------------------------------------------------------------------//
//                              Implementations
//---------------------------------------------------------------------------//

impl Decodeable for Unknown {

    fn decode<R: ReadBytes>(data: &mut R, _extra_data: &Option<DecodeableExtraData>) -> Result<Self> {
        let len = data.len()?;
        let data = data.read_slice(len as usize, false)?;
        Ok(Self {
            data,
        })
    }
}

impl Encodeable for Unknown {

    fn encode<W: WriteBytes>(&mut self, buffer: &mut W, _extra_data: &Option<EncodeableExtraData>) -> Result<()> {
        buffer.write_all(&self.data).map_err(From::from)
    }
}
