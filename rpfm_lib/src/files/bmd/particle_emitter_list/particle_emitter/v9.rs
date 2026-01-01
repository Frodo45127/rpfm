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

//---------------------------------------------------------------------------//
//                 Implementation of ParticleEmitter
//---------------------------------------------------------------------------//

impl ParticleEmitter {

    pub(crate) fn read_v9<R: ReadBytes>(&mut self, data: &mut R, extra_data: &Option<DecodeableExtraData>) -> Result<()> {
        self.key = data.read_sized_string_u8()?;

        self.transform = Transform3x4::decode(data, extra_data)?;

        self.emission_rate = data.read_f32()?;
        self.instance_name = data.read_sized_string_u8()?;

        self.flags = Flags::decode(data, extra_data)?;

        self.height_mode = data.read_sized_string_u8()?;
        self.pdlc_mask = data.read_u64()?;
        self.autoplay = data.read_bool()?;
        self.visible_in_shroud = data.read_bool()?;
        self.parent_id = data.read_i32()?;

        Ok(())
    }

    pub(crate) fn write_v9<W: WriteBytes>(&mut self, buffer: &mut W, extra_data: &Option<EncodeableExtraData>) -> Result<()> {
        buffer.write_sized_string_u8(&self.key)?;

        self.transform.encode(buffer, extra_data)?;

        buffer.write_f32(self.emission_rate)?;
        buffer.write_sized_string_u8(&self.instance_name)?;

        self.flags.encode(buffer, extra_data)?;

        buffer.write_sized_string_u8(&self.height_mode)?;
        buffer.write_u64(self.pdlc_mask)?;
        buffer.write_bool(self.autoplay)?;
        buffer.write_bool(self.visible_in_shroud)?;
        buffer.write_i32(self.parent_id)?;


        Ok(())
    }
}
