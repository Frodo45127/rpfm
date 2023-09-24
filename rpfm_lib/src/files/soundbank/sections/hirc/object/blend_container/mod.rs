//---------------------------------------------------------------------------//
// Copyright (c) 2017-2023 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

use getset::*;
use serde_derive::{Serialize, Deserialize};

use crate::binary::{ReadBytes, WriteBytes};
use crate::error::{Result, RLibError};

mod v122;

//---------------------------------------------------------------------------//
//                              Enum & Structs
//---------------------------------------------------------------------------//

#[derive(PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct BlendContainer {
    data: Vec<u8>,
}

//---------------------------------------------------------------------------//
//                              Implementation
//---------------------------------------------------------------------------//

impl BlendContainer {

    pub(crate) fn read<R: ReadBytes>(data: &mut R, version: u32, size: usize) -> Result<Self> {
        match version {
            122 => Self::read_v122(data, size),
            _ => Err(RLibError::SoundBankUnsupportedVersionFound(version, "BlendContainer".to_string())),
        }
    }

    pub(crate) fn write<W: WriteBytes>(&self, buffer: &mut W, version: u32) -> Result<()> {
        match version {
            122 => self.write_v122(buffer),
            _ => Err(RLibError::SoundBankUnsupportedVersionFound(version, "BlendContainer".to_string())),
        }
    }
}
