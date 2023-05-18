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
//                           Implementation of Text
//---------------------------------------------------------------------------//

impl PlayableArea {

    pub(crate) fn read_v2<R: ReadBytes>(&mut self, data: &mut R, extra_data: &Option<DecodeableExtraData>) -> Result<()> {
        self.area = Rectangle::decode(data, extra_data)?;
        self.has_been_set = data.read_bool()?;
        self.valid_location_flags.set_valid_north(data.read_bool()?);
        self.valid_location_flags.set_valid_south(data.read_bool()?);
        self.valid_location_flags.set_valid_east(data.read_bool()?);
        self.valid_location_flags.set_valid_west(data.read_bool()?);

        Ok(())
    }

    pub(crate) fn write_v2<W: WriteBytes>(&mut self, buffer: &mut W, extra_data: &Option<EncodeableExtraData>) -> Result<()> {
        self.area.encode(buffer, extra_data)?;
        buffer.write_bool(self.has_been_set)?;
        buffer.write_bool(*self.valid_location_flags.valid_north())?;
        buffer.write_bool(*self.valid_location_flags.valid_south())?;
        buffer.write_bool(*self.valid_location_flags.valid_east())?;
        buffer.write_bool(*self.valid_location_flags.valid_west())?;

        Ok(())
    }
}
