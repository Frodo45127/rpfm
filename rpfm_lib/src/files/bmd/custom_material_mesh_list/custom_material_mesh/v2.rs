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
//                    Implementation of CustomMaterialMeshList
//---------------------------------------------------------------------------//

impl CustomMaterialMesh {

    pub(crate) fn read_v2<R: ReadBytes>(&mut self, data: &mut R, extra_data: &Option<DecodeableExtraData>) -> Result<()> {
        for _ in 0..data.read_u32()? {
            self.vertices.push(Point3d::decode(data, extra_data)?);
        }

        for _ in 0..data.read_u32()? {
            self.indices.push(data.read_u16()?);
        }

        self.material = data.read_sized_string_u8()?;
        self.height_mode = data.read_sized_string_u8()?;

        Ok(())
    }

    pub(crate) fn write_v2<W: WriteBytes>(&mut self, buffer: &mut W, extra_data: &Option<EncodeableExtraData>) -> Result<()> {
        buffer.write_u32(self.vertices.len() as u32)?;
        for vertex in &mut self.vertices {
            vertex.encode(buffer, extra_data)?;
        }

        buffer.write_u32(self.indices.len() as u32)?;
        for index in &self.indices {
            buffer.write_u16(*index)?;
        }

        buffer.write_sized_string_u8(&self.material)?;
        buffer.write_sized_string_u8(&self.height_mode)?;

        Ok(())
    }
}
