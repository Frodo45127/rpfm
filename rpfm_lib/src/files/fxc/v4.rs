//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

use crate::binary::ReadBytes;
use crate::error::Result;

use super::*;

/// Offset of the total size of a DXBC blob from its start.
const DXBC_SIZE_OFFSET: usize = 24;

//---------------------------------------------------------------------------//
//                           Implementation of Fxc
//---------------------------------------------------------------------------//

impl Fxc {

    pub(crate) fn read_v4<R: ReadBytes>(&mut self, data: &mut R, _extra_data: &Option<DecodeableExtraData>) -> Result<()> {
        let define_count = data.read_u32()?;
        self.defines = Vec::with_capacity(define_count as usize);
        for _ in 0..define_count {
            let name = read_nul_terminated(data)?;
            let value = read_nul_terminated(data)?;
            self.defines.push(FxcDefine { name, value });
        }

        self.source_timestamp = data.read_u32()?;

        let include_count = data.read_u32()?;
        self.includes = Vec::with_capacity(include_count as usize);
        for _ in 0..include_count {
            let path = data.read_sized_string_u8()?;
            let hash = data.read_u32()?;
            self.includes.push(FxcInclude { path, hash });
        }

        // The rest of the file is the shader, which says its own size.
        let shader_size = data.len()? - data.stream_position()?;
        self.shader = data.read_slice(shader_size as usize, false)?;
        let declared_size = self.shader.get(DXBC_SIZE_OFFSET..DXBC_SIZE_OFFSET + 4)
            .map(|bytes| u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]));

        if !self.shader.starts_with(b"DXBC") || declared_size != Some(self.shader.len() as u32) {
            return Err(RLibError::DecodingFxcInvalidShader);
        }

        Ok(())
    }

    pub(crate) fn write_v4<W: WriteBytes>(&mut self, buffer: &mut W, _extra_data: &Option<EncodeableExtraData>) -> Result<()> {
        buffer.write_u32(self.defines.len() as u32)?;
        for define in &self.defines {
            write_nul_terminated(buffer, &define.name)?;
            write_nul_terminated(buffer, &define.value)?;
        }

        buffer.write_u32(self.source_timestamp)?;

        buffer.write_u32(self.includes.len() as u32)?;
        for include in &self.includes {
            buffer.write_sized_string_u8(&include.path)?;
            buffer.write_u32(include.hash)?;
        }

        buffer.write_all(&self.shader)?;

        Ok(())
    }
}

/// Reads a string stored as a u32 size and that many bytes, the last one a NUL that isn't part of the string.
fn read_nul_terminated<R: ReadBytes>(data: &mut R) -> Result<String> {
    let mut string = data.read_sized_string_u8_u32()?;
    if string.ends_with('\0') {
        string.pop();
    }

    Ok(string)
}

/// Writes a string as a u32 size and that many bytes, NUL included.
fn write_nul_terminated<W: WriteBytes>(buffer: &mut W, string: &str) -> Result<()> {
    buffer.write_u32(string.len() as u32 + 1)?;
    buffer.write_string_u8_0terminated(string)
}
