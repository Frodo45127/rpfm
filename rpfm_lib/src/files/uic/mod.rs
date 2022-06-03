//---------------------------------------------------------------------------//
// Copyright (c) 2017-2022 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! This is a module to read/write binary UIC (UI Component) files.
//!
//! UIC files define the layout and functionality of the UI. Binaries until 3k.
//! From there onwards they're in xml format.
//!
//! Unifinished module, do not use.

use crate::binary::{ReadBytes, WriteBytes};
use crate::error::Result;
use crate::files::{DecodeableExtraData, Decodeable, Encodeable};

/// Signature/Magic Numbers/Whatever of an UnitVariant.
const SIGNATURE: &str = "Version";
const VERSION_SIZE: usize = 3;

/// Extension of UIC files in some games (they usually don't have extensions).
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

    /// This function tries to read the header of an UIC from a raw data input.
    fn read_header<R: ReadBytes>(data: &mut R) -> Result<u32> {
        let _signature = data.read_string_u8(SIGNATURE.len())?;
        let version = data.read_string_u8(VERSION_SIZE)?.parse::<u32>()?;

        Ok(version)
    }
}


impl Decodeable for UIC {

    fn decode<R: ReadBytes>(data: &mut R, _extra_data: Option<DecodeableExtraData>) -> Result<Self> {
        let version = Self::read_header(data)?;
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
