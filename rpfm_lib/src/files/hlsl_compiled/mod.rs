//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! HLSL compiled shader file format support (FASTBIN0/DXBC).
//!
//! This module handles `.hlsl_compiled` files which contain compiled DirectX shaders
//! (DXBC format) wrapped in a FASTBIN0 container with metadata.
//!
//! # File Format
//!
//! HLSL compiled files use the `FASTBIN0` signature and contain:
//! - Serialization version
//! - Shader metadata (API, source file, name, type, shader model, UUID)
//! - Raw DXBC (DirectX Bytecode) shader data
//!
//! # Shader Metadata
//!
//! The wrapper format stores comprehensive shader compilation metadata:
//! - **API**: Target graphics API (e.g., "dx11")
//! - **Source**: Source `.hlsl` file path
//! - **Shader Name**: Entry point function name
//! - **Shader Type**: Vertex, pixel, compute, etc.
//! - **Model**: Shader model version (e.g., "vs_5_0", "ps_5_0")
//! - **UUID**: Unique identifier for this shader compilation
//!
//! # DXBC Data
//!
//! The actual shader bytecode is stored as raw DXBC format, which can be processed
//! by DirectX shader tools or executed by the graphics driver.
//!
//! # Versioning
//!
//! Currently only version 1 of the FASTBIN0 format is supported.
//!
//! # Usage
//!
//! ```ignore
//! use rpfm_lib::files::hlsl_compiled::HlslCompiled;
//! use rpfm_lib::files::Decodeable;
//!
//! // Decode a compiled shader
//! let shader = HlslCompiled::decode(&mut data, &None)?;
//!
//! // Access metadata
//! println!("Shader: {} ({})", shader.shader_name(), shader.shader_type());
//! println!("Model: {}", shader.model_long());
//!
//! // Access raw DXBC data
//! let dxbc_bytes = shader.data();
//! ```

use getset::*;
use serde_derive::{Serialize, Deserialize};

use crate::binary::{ReadBytes, WriteBytes};
use crate::error::{Result, RLibError};
use crate::files::{DecodeableExtraData, Decodeable, EncodeableExtraData, Encodeable};
use crate::utils::check_size_mismatch;

/// File extension for HLSL compiled shader files.
pub const EXTENSION: &str = ".hlsl_compiled";

/// FASTBIN0 file signature.
///
/// Identifies this as a FASTBIN0 container format (ASCII: "FASTBIN0").
pub const SIGNATURE: &[u8; 8] = &[0x46, 0x41, 0x53, 0x54, 0x42, 0x49, 0x4E, 0x30];

mod v1;

#[cfg(test)] mod hlsl_compiled_test;

//---------------------------------------------------------------------------//
//                              Enum & Structs
//---------------------------------------------------------------------------//

/// Represents a compiled HLSL shader with metadata.
///
/// This structure wraps a DXBC shader with comprehensive metadata about its
/// compilation, including source file, shader model, and unique identifiers.
#[derive(Default, PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct HlslCompiled {
    /// FASTBIN0 serialization version (currently always 1).
    serialise_version: u16,

    /// Target graphics API (e.g., "dx11", "dx12").
    api: String,

    /// Path to the source `.hlsl` file.
    source: String,

    /// Shader entry point function name.
    shader_name: String,

    /// Shader type (e.g., "VertexShader", "PixelShader", "ComputeShader").
    shader_type: String,

    /// Full shader model string (e.g., "vs_5_0", "ps_5_0").
    model_long: String,

    /// Unknown string field (purpose not yet identified).
    no_idea_1: String,

    /// Unique identifier for this shader compilation.
    uuid: String,

    /// Unknown u32 field (purpose not yet identified).
    no_idea_2: u32,

    /// Short shader model identifier.
    model_short: String,

    /// Unknown u16 field (purpose not yet identified).
    no_idea_3: u16,

    /// Unknown u32 field (purpose not yet identified).
    no_idea_4: u32,

    /// Raw DXBC (DirectX Bytecode) shader data.
    ///
    /// This is the compiled shader bytecode that can be executed by DirectX-compatible
    /// graphics drivers. The format is standard DXBC and can be analyzed with DirectX
    /// shader tools.
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
