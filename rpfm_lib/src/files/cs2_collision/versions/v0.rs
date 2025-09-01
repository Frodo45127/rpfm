//---------------------------------------------------------------------------//
// Copyright (c) 2017-2024 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//use std::io::SeekFrom;

use crate::error::Result;
use crate::binary::{ReadBytes, WriteBytes};

use super::*;

//---------------------------------------------------------------------------//
//                       Implementation of Collision3d
//---------------------------------------------------------------------------//

impl Collision3d {

    pub fn read_v0<R: ReadBytes>(&mut self, data: &mut R) -> Result<()> {
dbg!(format!("{:02X}", data.stream_position()?));
        self.name = data.read_sized_string_u16()?;
dbg!(format!("{:02X}", data.stream_position()?));

        self.uk_1 = data.read_i32()?;
dbg!(format!("{:02X}", data.stream_position()?));
        self.uk_2 = data.read_i32()?;
dbg!(format!("{:02X}", data.stream_position()?));
        for _ in 0..data.read_u32()? {
            self.vertices.push(Point3d::decode(data, &None)?);
        }
dbg!(format!("{:02X}", data.stream_position()?));
        for _ in 0..data.read_u32()? {
            let mut collision_triangle = CollisionTriangle::default();

            collision_triangle.face_index = data.read_i32()?;
            collision_triangle.padding = data.read_i8()?;
            collision_triangle.vertex_1 = data.read_i32()?;
            collision_triangle.vertex_2 = data.read_i32()?;
            collision_triangle.vertex_3 = data.read_i32()?;
            collision_triangle.edge_1_vertex_1 = data.read_i32()?;
            collision_triangle.edge_1_vertex_2 = data.read_i32()?;
            collision_triangle.face_index_1 = data.read_i32()?;
            collision_triangle.zero_1 = data.read_i32()?;
            collision_triangle.across_face_index_1 = data.read_i32()?;
            collision_triangle.edge_2_vertex_1 = data.read_i32()?;
            collision_triangle.edge_2_vertex_2 = data.read_i32()?;
            collision_triangle.face_index_2 = data.read_i32()?;
            collision_triangle.zero_2 = data.read_i32()?;
            collision_triangle.across_face_index_2 = data.read_i32()?;
            collision_triangle.edge_3_vertex_1 = data.read_i32()?;
            collision_triangle.edge_3_vertex_2 = data.read_i32()?;
            collision_triangle.face_index_3 = data.read_i32()?;
            collision_triangle.zero_3 = data.read_i32()?;
            collision_triangle.across_face_index_3 = data.read_i32()?;
            collision_triangle.zero_4 = data.read_i32()?;

            self.triangles.push(collision_triangle);
        }
dbg!(format!("{:02X}", data.stream_position()?));
        Ok(())
    }

    pub fn write_v0<W: WriteBytes>(&mut self, buffer: &mut W) -> Result<()> {
        buffer.write_sized_string_u16(&self.name)?;

        buffer.write_i32(self.uk_1)?;
        buffer.write_i32(self.uk_2)?;

        buffer.write_u32(self.vertices.len() as u32)?;
        for vertex in &mut self.vertices {
            vertex.encode(buffer, &None)?;
        }

        buffer.write_u32(self.triangles.len() as u32)?;
        for triangle in &mut self.triangles {
            buffer.write_i32(triangle.face_index)?;
            buffer.write_i8(triangle.padding)?;
            buffer.write_i32(triangle.vertex_1)?;
            buffer.write_i32(triangle.vertex_2)?;
            buffer.write_i32(triangle.vertex_3)?;
            buffer.write_i32(triangle.edge_1_vertex_1)?;
            buffer.write_i32(triangle.edge_1_vertex_2)?;
            buffer.write_i32(triangle.face_index_1)?;
            buffer.write_i32(triangle.zero_1)?;
            buffer.write_i32(triangle.across_face_index_1)?;
            buffer.write_i32(triangle.edge_2_vertex_1)?;
            buffer.write_i32(triangle.edge_2_vertex_2)?;
            buffer.write_i32(triangle.face_index_2)?;
            buffer.write_i32(triangle.zero_2)?;
            buffer.write_i32(triangle.across_face_index_2)?;
            buffer.write_i32(triangle.edge_3_vertex_1)?;
            buffer.write_i32(triangle.edge_3_vertex_2)?;
            buffer.write_i32(triangle.face_index_3)?;
            buffer.write_i32(triangle.zero_3)?;
            buffer.write_i32(triangle.across_face_index_3)?;
            buffer.write_i32(triangle.zero_4)?;
        }

        Ok(())
    }
}
