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
//                    Implementation of RiverNode
//---------------------------------------------------------------------------//

impl RiverNode {

    pub(crate) fn read_v1<R: ReadBytes>(&mut self, data: &mut R, _extra_data: &Option<DecodeableExtraData>) -> Result<()> {
        self.vertex = Vertex {
            x: data.read_f32()?,
            y: data.read_f32()?,
            z: data.read_f32()?,
        };
        self.width = data.read_f32()?;
        self.flow_speed = data.read_f32()?;

        Ok(())
    }

    pub(crate) fn write_v1<W: WriteBytes>(&mut self, buffer: &mut W, _extra_data: &Option<EncodeableExtraData>) -> Result<()> {
        buffer.write_f32(self.vertex.x)?;
        buffer.write_f32(self.vertex.y)?;
        buffer.write_f32(self.vertex.z)?;
        buffer.write_f32(self.width)?;
        buffer.write_f32(self.flow_speed)?;

        Ok(())
    }
}
