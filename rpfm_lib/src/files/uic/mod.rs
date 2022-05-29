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
Module with all the code to interact with UI Components.

UI Components are binary files that form the ui of TW Games.
They have no extension (mostly), and I heard they're a pain in the ass to work with.
!*/

use crate::files::DecodeableExtraData;
use crate::error::Result;

use crate::binary::{ReadBytes, WriteBytes};
use crate::files::{Decodeable, Encodeable};

const SIGNATURE: &str = "Version";
const VERSION_SIZE: usize = 3;

/// Size of the header of an UIC PackedFile.
pub const HEADER_SIZE: usize = 10;

pub const EXTENSION: &str = ".cml";

//---------------------------------------------------------------------------//
//                              Enum & Structs
//---------------------------------------------------------------------------//

/// This holds an entire UI Component decoded in memory.
#[derive(PartialEq, Clone, Debug, Default)]
pub struct UIC {
    version: u32,
}

//---------------------------------------------------------------------------//
//                           Implementation of Text
//---------------------------------------------------------------------------//

/// Implementation of `UIC`.
impl UIC {
    /*
    pub fn is_ui_component(data: &[u8]) -> bool {
        match data.decode_string_u8(0, 7) {
            Ok(signature) => signature == SIGNATURE,
            Err(_) => false,
        }
    }*/

    /// This function tries to read the header of an UIC PackedFile from raw data.
    fn read_header<R: ReadBytes>(data: &mut R) -> Result<u32> {
        let _signature = data.read_string_u8(SIGNATURE.len())?;
        let version = data.read_string_u8(VERSION_SIZE)?.parse::<u32>()?;

        Ok(version)
    }
}


impl Decodeable for UIC {

    fn decode<R: ReadBytes>(data: &mut R, _extra_data: Option<DecodeableExtraData>) -> Result<Self> {
        let version = Self::read_header(data)?;

        // If we've reached this, we've successfully decoded the entire UI.
        Ok(Self {
            version,
        })
    }
}

impl Encodeable for UIC {
    fn encode<W: WriteBytes>(&mut self, buffer: &mut W, _extra_data: Option<DecodeableExtraData>) -> Result<()> {
        buffer.write_string_u8(SIGNATURE)?;
        buffer.write_u32(self.version)?;

        Ok(())
    }
}
