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
//               Implementation of TerrainStencilTriangle
//---------------------------------------------------------------------------//

impl TerrainStencilTriangle {

    pub(crate) fn read_v2<R: ReadBytes>(&mut self, data: &mut R, extra_data: &Option<DecodeableExtraData>) -> Result<()> {
        self.position_0 = Point3d::decode(data, extra_data)?;
        self.position_1 = Point3d::decode(data, extra_data)?;
        self.position_2 = Point3d::decode(data, extra_data)?;
        self.height_mode = data.read_sized_string_u8()?;

        Ok(())
    }

    pub(crate) fn write_v2<W: WriteBytes>(&mut self, buffer: &mut W, extra_data: &Option<EncodeableExtraData>) -> Result<()> {
        self.position_0.encode(buffer, extra_data)?;
        self.position_1.encode(buffer, extra_data)?;
        self.position_2.encode(buffer, extra_data)?;

        buffer.write_sized_string_u8(&self.height_mode)?;

        Ok(())
    }
}

