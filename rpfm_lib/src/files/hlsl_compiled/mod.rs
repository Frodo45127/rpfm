//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! This is a module to read/write hlsl_compiled files. These are DXBC Shader files, packed with some extra baggage.

use getset::*;
use serde_derive::{Serialize, Deserialize};

use crate::binary::{ReadBytes, WriteBytes};
use crate::error::{Result, RLibError};
use crate::files::{DecodeableExtraData, Decodeable, EncodeableExtraData, Encodeable};
use crate::utils::check_size_mismatch;

pub const EXTENSION: &str = ".hlsl_compiled";

/// FASTBIN0
pub const SIGNATURE: &[u8; 8] = &[0x46, 0x41, 0x53, 0x54, 0x42, 0x49, 0x4E, 0x30];

mod v1;

#[cfg(test)] mod hlsl_compiled_test;

//---------------------------------------------------------------------------//
//                              Enum & Structs
//---------------------------------------------------------------------------//

#[derive(Default, PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct HlslCompiled {
    serialise_version: u16,
    api: String,
    source: String,
    shader_name: String,
    shader_type: String,
    model_long: String,
    no_idea_1: String,
    uuid: String,
    no_idea_2: u32,
    model_short: String,
    no_idea_3: u16,
    no_idea_4: u32,

    /// This contains the raw DXBC data.
    data: Vec<u8>,
}

//---------------------------------------------------------------------------//
//                              Implementations
//---------------------------------------------------------------------------//

impl Decodeable for HlslCompiled {

    fn decode<R: ReadBytes>(data: &mut R, extra_data: &Option<DecodeableExtraData>) -> Result<Self> {
        let signature_bytes = data.read_slice(8, false)?;
        if signature_bytes.as_slice() != SIGNATURE {
            return Err(RLibError::DecodingFastBinUnsupportedSignature(signature_bytes));
        }

        let mut fastbin = Self::default();
        fastbin.serialise_version = data.read_u16()?;

        match fastbin.serialise_version {
            1 => fastbin.read_v1(data, extra_data)?,
            _ => return Err(RLibError::DecodingFastBinUnsupportedVersion(String::from("HlslCompiled"), fastbin.serialise_version)),
        }

        // If we are not in the last byte, it means we didn't parse the entire file, which means this file is corrupt.
        check_size_mismatch(data.stream_position()? as usize, data.len()? as usize)?;

        Ok(fastbin)
    }
}

impl Encodeable for HlslCompiled {

    fn encode<W: WriteBytes>(&mut self, buffer: &mut W, extra_data: &Option<EncodeableExtraData>) -> Result<()> {
        buffer.write_all(SIGNATURE)?;
        buffer.write_u16(self.serialise_version)?;

        match self.serialise_version {
            1 => self.write_v1(buffer, extra_data)?,
            _ => return Err(RLibError::EncodingFastBinUnsupportedVersion(String::from("HlslCompiled"), self.serialise_version)),
        }

        Ok(())
    }
}
