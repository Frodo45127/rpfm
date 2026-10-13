//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Compiled shader containers (`.fxc`) of the Warscape games.
//!
//! These files, under `fxc/` in the packs, hold one compiled DirectX shader each, with the defines and
//! includes it was compiled with. Newer games use [`HlslCompiled`](crate::files::hlsl_compiled::HlslCompiled) instead.
//!
//! # File Format
//!
//! All values are little endian:
//!
//! ```text
//! [u32]      signature (0x075BCD15)
//! [u32]      version
//! [u32]      define count
//! [...]      defines: name and value, each a u32 size and that many bytes, NUL included
//! [u32]      timestamp of the source
//! [u32]      include count
//! [...]      includes: path (u16 size and that many bytes, no NUL), u32 hash
//! [...]      DXBC shader blob, up to the end of the file
//! ```
//!
//! Only version 4 is known, used by Attila's DX11 shaders.
//!
//! # Usage
//!
//! ```ignore
//! use rpfm_lib::files::fxc::Fxc;
//! use rpfm_lib::files::Decodeable;
//!
//! let fxc = Fxc::decode(&mut data, &None)?;
//! let dxbc = fxc.shader();
//! ```

use getset::*;
use serde_derive::{Serialize, Deserialize};

use crate::binary::{ReadBytes, WriteBytes};
use crate::error::{Result, RLibError};
use crate::files::{DecodeableExtraData, Decodeable, EncodeableExtraData, Encodeable};

/// File extension of compiled shader containers.
pub const EXTENSION: &str = ".fxc";

/// Signature at the start of every fxc file.
pub const SIGNATURE: u32 = 0x075B_CD15;

mod v4;

#[cfg(test)] mod fxc_test;

//---------------------------------------------------------------------------//
//                              Enum & Structs
//---------------------------------------------------------------------------//

/// A compiled shader with the defines and includes it was compiled with.
#[derive(Default, PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct Fxc {

    /// Version of the container format.
    version: u32,

    /// Preprocessor defines the shader was compiled with, in file order.
    defines: Vec<FxcDefine>,

    /// Timestamp of the shader's source file.
    source_timestamp: u32,

    /// Files the shader's source includes, in file order.
    includes: Vec<FxcInclude>,

    /// The compiled shader: a DXBC blob, with its reflection data.
    shader: Vec<u8>,
}

/// A preprocessor define of a compiled shader.
#[derive(Default, PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct FxcDefine {

    /// Name of the define.
    name: String,

    /// Value of the define.
    value: String,
}

/// A file included by the source of a compiled shader.
#[derive(Default, PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct FxcInclude {

    /// Path of the included file.
    path: String,

    /// Hash of the included file.
    hash: u32,
}

//---------------------------------------------------------------------------//
//                              Implementations
//---------------------------------------------------------------------------//

impl Decodeable for Fxc {

    fn decode<R: ReadBytes>(data: &mut R, extra_data: &Option<DecodeableExtraData>) -> Result<Self> {
        let signature = data.read_u32()?;
        if signature != SIGNATURE {
            return Err(RLibError::DecodingFxcUnsupportedSignature(signature));
        }

        let mut fxc = Self {
            version: data.read_u32()?,
            ..Default::default()
        };

        match fxc.version {
            4 => fxc.read_v4(data, extra_data)?,
            _ => return Err(RLibError::DecodingFxcUnsupportedVersion(fxc.version)),
        }

        Ok(fxc)
    }
}

impl Encodeable for Fxc {

    fn encode<W: WriteBytes>(&mut self, buffer: &mut W, extra_data: &Option<EncodeableExtraData>) -> Result<()> {
        buffer.write_u32(SIGNATURE)?;
        buffer.write_u32(self.version)?;

        match self.version {
            4 => self.write_v4(buffer, extra_data)?,
            _ => return Err(RLibError::EncodingFxcUnsupportedVersion(self.version)),
        }

        Ok(())
    }
}
