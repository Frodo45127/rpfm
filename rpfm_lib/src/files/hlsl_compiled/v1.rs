//---------------------------------------------------------------------------//
// Copyright (c) 2017-2023 Ismael Gutiérrez González. All rights reserved.
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

//---------------------------------------------------------------------------//
//                     Implementation of HlslCompiled
//---------------------------------------------------------------------------//

impl HlslCompiled {

    pub(crate) fn read_v1<R: ReadBytes>(&mut self, data: &mut R, _extra_data: &Option<DecodeableExtraData>) -> Result<()> {
        self.api = data.read_sized_string_u8()?;
        self.source = data.read_sized_string_u8()?;
        self.shader_name = data.read_sized_string_u8()?;
        self.shader_type = data.read_sized_string_u8()?;
        self.model_long = data.read_sized_string_u8()?;
        self.no_idea_1 = data.read_sized_string_u8()?;
        self.uuid = data.read_sized_string_u8()?;
        self.no_idea_2 = data.read_u32()?;
        self.model_short = data.read_sized_string_u8()?;
        self.no_idea_3 = data.read_u16()?;
        self.no_idea_4 = data.read_u32()?;

        let data_size = data.read_u32()?;
        self.data = data.read_slice(data_size as usize, false)?;

        Ok(())
    }

    pub(crate) fn write_v1<W: WriteBytes>(&mut self, buffer: &mut W, _extra_data: &Option<EncodeableExtraData>) -> Result<()> {
        buffer.write_sized_string_u8(&self.api)?;
        buffer.write_sized_string_u8(&self.source)?;
        buffer.write_sized_string_u8(&self.shader_name)?;
        buffer.write_sized_string_u8(&self.shader_type)?;
        buffer.write_sized_string_u8(&self.model_long)?;
        buffer.write_sized_string_u8(&self.no_idea_1)?;
        buffer.write_sized_string_u8(&self.uuid)?;
        buffer.write_u32(self.no_idea_2)?;
        buffer.write_sized_string_u8(&self.model_short)?;
        buffer.write_u16(self.no_idea_3)?;
        buffer.write_u32(self.no_idea_4)?;

        buffer.write_u32(self.data.len() as u32)?;
        buffer.write_all(&self.data)?;

        Ok(())
    }
}
