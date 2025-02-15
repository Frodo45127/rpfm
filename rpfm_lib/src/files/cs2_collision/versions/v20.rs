//---------------------------------------------------------------------------//
// Copyright (c) 2017-2024 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

use crate::error::Result;
use crate::binary::{ReadBytes, WriteBytes};

use super::*;

//---------------------------------------------------------------------------//
//                       Implementation of Cs2Collision
//---------------------------------------------------------------------------//

impl Cs2Collision {

    pub fn read_v20<R: ReadBytes>(&mut self, data: &mut R) -> Result<()> {
        self.bounding_box = Cube::decode(data, &None)?;

        while data.read_u8().is_ok() {
            data.seek_relative(-1)?;

            let mut collision_3d = Collision3d::default();
            collision_3d.name = data.read_sized_string_u8()?;

            collision_3d.uk_1 = data.read_i32()?;
            collision_3d.uk_2 = data.read_i32()?;

            for _ in 0..data.read_u32()? {
                collision_3d.vertices.push(Point3d::decode(data, &None)?);
            }

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

                collision_3d.triangles.push(collision_triangle);
            }

            collision_3d.zero_4 = data.read_i32()?;
            collision_3d.bounding_box = Cube::decode(data, &None)?;

            self.collisions_3d.push(collision_3d);
        }


        Ok(())
    }

    pub fn write_v20<W: WriteBytes>(&mut self, buffer: &mut W) -> Result<()> {
        self.bounding_box.encode(buffer, &None)?;

        for collision_3d in &mut self.collisions_3d {
            buffer.write_sized_string_u8(&collision_3d.name)?;

            buffer.write_i32(collision_3d.uk_1)?;
            buffer.write_i32(collision_3d.uk_2)?;

            buffer.write_u32(collision_3d.vertices.len() as u32)?;
            for vertex in &mut collision_3d.vertices {
                vertex.encode(buffer, &None)?;
            }

            buffer.write_u32(collision_3d.triangles.len() as u32)?;
            for triangle in &mut collision_3d.triangles {
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

            buffer.write_i32(collision_3d.zero_4)?;
            collision_3d.bounding_box.encode(buffer, &None)?;
        }


        Ok(())
    }
}

