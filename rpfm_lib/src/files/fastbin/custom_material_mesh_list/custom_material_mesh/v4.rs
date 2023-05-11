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

    pub(crate) fn read_v4<R: ReadBytes>(&mut self, data: &mut R, extra_data: &Option<DecodeableExtraData>) -> Result<()> {
        for _ in 0..data.read_u32()? {
            self.vertices.push(Vertex {
                x: data.read_f32()?,
                y: data.read_f32()?,
                z: data.read_f32()?,
            });
        }

        for _ in 0..data.read_u32()? {
            self.indices.push(data.read_u16()?);
        }

        self.material = data.read_sized_string_u8()?;
        self.height_mode = data.read_sized_string_u8()?;
        self.flags = Flags::decode(data, extra_data)?;
        self.transform = Transform::decode(data, extra_data)?;
        self.snow_inside = data.read_bool()?;
        self.snow_outside = data.read_bool()?;
        self.destruction_inside = data.read_bool()?;
        self.destruction_outside = data.read_bool()?;
        self.visible_in_shroud = data.read_bool()?;
        self.visible_without_shroud = data.read_bool()?;

        Ok(())
    }

    pub(crate) fn write_v4<W: WriteBytes>(&mut self, buffer: &mut W, extra_data: &Option<EncodeableExtraData>) -> Result<()> {
        buffer.write_u32(self.vertices.len() as u32)?;
        for vertex in &self.vertices {
            buffer.write_f32(vertex.x)?;
            buffer.write_f32(vertex.y)?;
            buffer.write_f32(vertex.z)?;
        }

        buffer.write_u32(self.indices.len() as u32)?;
        for index in &self.indices {
            buffer.write_u16(*index)?;
        }

        buffer.write_sized_string_u8(&self.material)?;
        buffer.write_sized_string_u8(&self.height_mode)?;

        self.flags.encode(buffer, extra_data)?;
        self.transform.encode(buffer, extra_data)?;

        buffer.write_bool(self.snow_inside)?;
        buffer.write_bool(self.snow_outside)?;
        buffer.write_bool(self.destruction_inside)?;
        buffer.write_bool(self.destruction_outside)?;
        buffer.write_bool(self.visible_in_shroud)?;
        buffer.write_bool(self.visible_without_shroud)?;

        Ok(())
    }
}
