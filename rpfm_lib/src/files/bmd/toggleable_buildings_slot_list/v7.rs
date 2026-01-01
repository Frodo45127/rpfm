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
//               Implementation of ToggleableBuildingsSlotList
//---------------------------------------------------------------------------//

impl ToggleableBuildingsSlotList {

    pub(crate) fn read_v7<R: ReadBytes>(&mut self, data: &mut R, extra_data: &Option<DecodeableExtraData>) -> Result<()> {
        for _ in 0..data.read_u32()? {
            self.toggleable_buildings_slots.push(ToggleableBuildingsSlot::decode(data, extra_data)?);
        }

        Ok(())
    }

    pub(crate) fn write_v7<W: WriteBytes>(&mut self, buffer: &mut W, extra_data: &Option<EncodeableExtraData>) -> Result<()> {
        buffer.write_u32(self.toggleable_buildings_slots.len() as u32)?;
        for togg in &mut self.toggleable_buildings_slots {
            togg.encode(buffer, extra_data)?;
        }

        Ok(())
    }
}
