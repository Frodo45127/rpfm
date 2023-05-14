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

    pub(crate) fn read_v2<R: ReadBytes>(&mut self, data: &mut R, extra_data: &Option<DecodeableExtraData>) -> Result<()> {
        self.building_reference = BuildingReference::decode(data, extra_data)?;
        self.prefab_index = data.read_i32()?;
        self.prefab_building_key = data.read_sized_string_u8()?;

        Ok(())
    }

    pub(crate) fn write_v2<W: WriteBytes>(&mut self, buffer: &mut W, extra_data: &Option<EncodeableExtraData>) -> Result<()> {
        self.building_reference.encode(buffer, extra_data)?;
        buffer.write_i32(self.prefab_index)?;
        buffer.write_sized_string_u8(&self.prefab_building_key)?;

        Ok(())
    }
}
