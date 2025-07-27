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
//                       Implementation of Cs2Parsed
//---------------------------------------------------------------------------//

impl Cs2Parsed {

    pub fn read_v11<R: ReadBytes>(&mut self, data: &mut R) -> Result<()> {
        self.bounding_box = Cube::decode(data, &None)?;
        self.ui_flag.name = data.read_sized_string_u16()?;
        self.ui_flag.transform = Transform4x4::decode(data, &None)?;

        self.int_1 = data.read_i32()?;

        // Pieces
        for _ in 0..data.read_u32()? {
            let mut piece = Piece::default();

            // Tech node.
            piece.name = data.read_sized_string_u16()?;
            piece.node_name = data.read_sized_string_u16()?;
            piece.node_transform = Transform4x4::decode(data, &None)?;
            piece.int_3 = data.read_i32()?;

            for _ in 0..data.read_u32()? {
                let mut destruct = Destruct::default();
                destruct.name = data.read_sized_string_u16()?;
                destruct.index = data.read_u32()?;

                destruct.collision_3d.read_v11(data)?;
                destruct.windows = data.read_i32()?;
                destruct.doors = data.read_i32()?;
                destruct.gates = data.read_i32()?;

                // Collision outlines?
                for _ in 0..data.read_u32()? {
                    destruct.collision_outlines.push(CollisionOutline {
                        name: data.read_sized_string_u16()?,
                        vertices: Outline3d::decode(data, &None)?,
                        uk_1: data.read_u32()?,
                    });
                }

                // Pipes.
                for _ in 0..data.read_u32()? {
                    destruct.pipes.push(Pipe {
                        name: data.read_sized_string_u16()?,
                        line: Outline3d::decode(data, &None)?,
                        line_type: PipeType::try_from(data.read_i32()?)?,
                    });
                }

                // This is the weird orange line in terry?
                for _ in 0..data.read_u32()? {
                    let mut thingies = vec![];
                    for _ in 0..data.read_u32()? {
                        thingies.push(OrangeThingy {
                            vertex: Point2d::decode(data, &None)?,
                            vertex_type: data.read_u32()?,
                        });
                    }

                    destruct.orange_thingies.push(thingies);
                }

                // Platforms.
                for _ in 0..data.read_u32()? {
                    destruct.platforms.push(Platform {
                        normal: Point3d::decode(data, &None)?,
                        vertices: Outline3d::decode(data, &None)?,
                        flag_1: data.read_bool()?,
                        flag_2: data.read_bool()?,
                        flag_3: data.read_bool()?,
                    });
                }

                destruct.uk_2 = data.read_i32()?;
                destruct.bounding_box = Cube::decode(data, &None)?;
                destruct.cannon_emitters = data.read_i32()?;

                for _ in 0..data.read_u32()? {
                    destruct.projectile_emitters.push(ProjectileEmitter {
                        name: data.read_sized_string_u16()?,
                        transform: Transform4x4::decode(data, &None)?,
                    });
                }

                destruct.docking_points = data.read_i32()?;

                for _ in 0..data.read_u32()? {
                    destruct.soft_collisions.push(SoftCollisions {
                        name: data.read_sized_string_u16()?,
                        transform: Transform4x4::decode(data, &None)?,
                        uk_1: data.read_i16()?,
                        point_1: Point2d::decode(data, &None)?,
                    });
                }

                destruct.uk_7 = data.read_i32()?;

                // Destructible pillars.
                for _ in 0..data.read_u32()? {
                    destruct.file_refs.push(FileRef {
                        key: data.read_sized_string_u16()?,
                        name: data.read_sized_string_u16()?,
                        transform: Transform4x4::decode(data, &None)?,
                        uk_1: data.read_i16()?
                    });
                }

                // EF lines.
                for _ in 0..data.read_u32()? {
                    destruct.ef_lines.push(EFLine {
                        name: data.read_sized_string_u16()?,
                        action: EFLineType::try_from(data.read_i32()?)?,
                        start: Point3d::decode(data, &None)?,
                        end: Point3d::decode(data, &None)?,
                        direction: Point3d::decode(data, &None)?,
                        parent_index: data.read_u32()?
                    });
                }

                destruct.f_1 = data.read_f32()?;

                piece.destructs.push(destruct);
            }

            piece.f_6 = data.read_f32()?;

            self.pieces.push(piece);
        }

        Ok(())
    }

    pub fn write_v11<W: WriteBytes>(&mut self, buffer: &mut W) -> Result<()> {
        self.bounding_box.encode(buffer, &None)?;
        buffer.write_sized_string_u16(&self.ui_flag.name)?;
        self.ui_flag.transform.encode(buffer, &None)?;

        buffer.write_i32(self.int_1)?;
        buffer.write_u32(self.pieces.len() as u32)?;
        for piece in &mut self.pieces {
            buffer.write_sized_string_u16(&piece.name)?;
            buffer.write_sized_string_u16(&piece.node_name)?;

            piece.node_transform.encode(buffer, &None)?;

            buffer.write_i32(piece.int_3)?;
            buffer.write_u32(piece.destructs.len() as u32)?;
            for destruct in &mut piece.destructs {
                buffer.write_sized_string_u16(&destruct.name)?;
                buffer.write_u32(destruct.index)?;

                destruct.collision_3d.write_v11(buffer)?;
                buffer.write_u32(destruct.windows as u32)?;
                buffer.write_u32(destruct.doors as u32)?;
                buffer.write_u32(destruct.gates as u32)?;

                buffer.write_u32(destruct.collision_outlines.len() as u32)?;
                for outline in &mut destruct.collision_outlines {
                    buffer.write_sized_string_u16(&outline.name)?;
                    outline.vertices.encode(buffer, &None)?;
                    buffer.write_u32(outline.uk_1)?;
                }

                buffer.write_u32(destruct.pipes.len() as u32)?;
                for pipe in &mut destruct.pipes {
                    buffer.write_sized_string_u16(&pipe.name)?;
                    pipe.line.encode(buffer, &None)?;
                    buffer.write_i32(pipe.line_type.into())?;
                }

                buffer.write_u32(destruct.orange_thingies.len() as u32)?;
                for orange_thingies in &mut destruct.orange_thingies {

                    buffer.write_u32(orange_thingies.len() as u32)?;
                    for orange_thingy in orange_thingies {
                        orange_thingy.vertex.encode(buffer, &None)?;
                        buffer.write_u32(orange_thingy.vertex_type)?;
                    }
                }

                buffer.write_u32(destruct.platforms.len() as u32)?;
                for platform in &mut destruct.platforms {
                    platform.normal.encode(buffer, &None)?;
                    platform.vertices.encode(buffer, &None)?;

                    buffer.write_bool(platform.flag_1)?;
                    buffer.write_bool(platform.flag_2)?;
                    buffer.write_bool(platform.flag_3)?;
                }

                buffer.write_i32(destruct.uk_2)?;
                destruct.bounding_box.encode(buffer, &None)?;
                buffer.write_i32(destruct.cannon_emitters)?;

                buffer.write_u32(destruct.projectile_emitters.len() as u32)?;
                for emitter in &mut destruct.projectile_emitters {
                    buffer.write_sized_string_u16(&emitter.name)?;
                    emitter.transform.encode(buffer, &None)?;
                }

                buffer.write_i32(destruct.docking_points)?;

                buffer.write_u32(destruct.soft_collisions.len() as u32)?;
                for soft_collision in &mut destruct.soft_collisions {
                    buffer.write_sized_string_u16(&soft_collision.name)?;
                    soft_collision.transform.encode(buffer, &None)?;
                    buffer.write_i16(soft_collision.uk_1)?;
                    soft_collision.point_1.encode(buffer, &None)?;
                }

                buffer.write_i32(destruct.uk_7)?;

                buffer.write_u32(destruct.file_refs.len() as u32)?;
                for file_ref in &mut destruct.file_refs {
                    buffer.write_sized_string_u16(&file_ref.key)?;
                    buffer.write_sized_string_u16(&file_ref.name)?;

                    file_ref.transform.encode(buffer, &None)?;
                    buffer.write_i16(file_ref.uk_1)?;
                }

                buffer.write_u32(destruct.ef_lines.len() as u32)?;
                for ef_line in &mut destruct.ef_lines {
                    buffer.write_sized_string_u16(&ef_line.name)?;
                    buffer.write_i32(ef_line.action.into())?;
                    ef_line.start.encode(buffer, &None)?;
                    ef_line.end.encode(buffer, &None)?;
                    ef_line.direction.encode(buffer, &None)?;
                    buffer.write_u32(ef_line.parent_index)?;
                }

                buffer.write_f32(destruct.f_1)?;
            }

            buffer.write_f32(piece.f_6)?;
        }

        Ok(())
    }
}
