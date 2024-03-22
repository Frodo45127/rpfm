//---------------------------------------------------------------------------//
// Copyright (c) 2017-2024 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

use crate::binary::ReadBytes;
use crate::error::Result;
use crate::files::Decodeable;

use super::*;

//---------------------------------------------------------------------------//
//                           Implementation of Building
//---------------------------------------------------------------------------//

impl Building {

    pub(crate) fn read_v11<R: ReadBytes>(&mut self, data: &mut R, extra_data: &Option<DecodeableExtraData>) -> Result<()> {
        self.building_id = data.read_sized_string_u8()?;
        self.parent_id = data.read_i32()?;
        self.building_key = data.read_sized_string_u8()?;
        self.position_type = data.read_sized_string_u8()?;
        self.transform = Transform3x4::decode(data, extra_data)?;
        self.properties = Properties::decode(data, extra_data)?;
        self.height_mode = data.read_sized_string_u8()?;
        self.uid = data.read_u64()?;

        Ok(())
    }

    pub(crate) fn write_v11<W: WriteBytes>(&mut self, buffer: &mut W, extra_data: &Option<EncodeableExtraData>) -> Result<()> {
        buffer.write_sized_string_u8(&self.building_id)?;
        buffer.write_i32(self.parent_id)?;
        buffer.write_sized_string_u8(&self.building_key)?;
        buffer.write_sized_string_u8(&self.position_type)?;

        self.transform.encode(buffer, extra_data)?;
        self.properties.encode(buffer, extra_data)?;

        buffer.write_sized_string_u8(&self.height_mode)?;
        buffer.write_u64(self.uid)?;

        Ok(())
    }
}
