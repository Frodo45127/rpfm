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
//                    Implementation of DeploymentRegion
//---------------------------------------------------------------------------//

impl DeploymentRegion {

    pub(crate) fn read_v1<R: ReadBytes>(&mut self, data: &mut R, extra_data: &Option<DecodeableExtraData>) -> Result<()> {
        for _ in 0..data.read_u32()? {
            self.boundary_list.push(Boundary::decode(data, extra_data)?);
        }

        self.orientation = data.read_f32()?;
        self.snap_facing = data.read_bool()?;
        self.id = data.read_u32()?;

        Ok(())
    }

    pub(crate) fn write_v1<W: WriteBytes>(&mut self, buffer: &mut W, extra_data: &Option<EncodeableExtraData>) -> Result<()> {
        buffer.write_u32(self.boundary_list.len() as u32)?;
        for boundary in &mut self.boundary_list {
            boundary.encode(buffer, extra_data)?;
        }

        buffer.write_f32(self.orientation)?;
        buffer.write_bool(self.snap_facing)?;
        buffer.write_u32(self.id)?;

        Ok(())
    }
}
