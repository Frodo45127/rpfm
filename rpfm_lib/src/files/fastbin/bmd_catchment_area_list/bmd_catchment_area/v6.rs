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
//                    Implementation of BmdCatchmentArea
//---------------------------------------------------------------------------//

impl BmdCatchmentArea {

    pub(crate) fn read_v6<R: ReadBytes>(&mut self, data: &mut R, extra_data: &Option<DecodeableExtraData>) -> Result<()> {
        self.name = data.read_sized_string_u8()?;
        self.area = Area::decode(data, extra_data)?;
        self.battle_type = data.read_sized_string_u8()?;
        self.defending_faction_restriction = data.read_sized_string_u8()?;
        self.valid_location_flags = ValidLocationFlags::decode(data, extra_data)?;

        Ok(())
    }

    pub(crate) fn write_v6<W: WriteBytes>(&mut self, buffer: &mut W, extra_data: &Option<EncodeableExtraData>) -> Result<()> {
        buffer.write_sized_string_u8(&self.name)?;

        self.area.encode(buffer, extra_data)?;

        buffer.write_sized_string_u8(&self.battle_type)?;
        buffer.write_sized_string_u8(&self.defending_faction_restriction)?;

        self.valid_location_flags.encode(buffer, extra_data)?;

        Ok(())
    }
}
