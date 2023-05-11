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
//                    Implementation of BuildingLink
//---------------------------------------------------------------------------//

impl BuildingLink {

    pub(crate) fn read_v3<R: ReadBytes>(&mut self, data: &mut R, _extra_data: &Option<DecodeableExtraData>) -> Result<()> {
        self.building_index = data.read_i32()?;
        self.prefab_index = data.read_i32()?;
        self.prefab_building_key = data.read_sized_string_u8()?;
        self.uid = data.read_u64()?;
        self.prefab_uid = data.read_u64()?;

        Ok(())
    }

    pub(crate) fn write_v3<W: WriteBytes>(&mut self, buffer: &mut W, _extra_data: &Option<EncodeableExtraData>) -> Result<()> {
        buffer.write_i32(self.building_index)?;
        buffer.write_i32(self.prefab_index)?;
        buffer.write_sized_string_u8(&self.prefab_building_key)?;
        buffer.write_u64(self.uid)?;
        buffer.write_u64(self.prefab_uid)?;

        Ok(())
    }
}
