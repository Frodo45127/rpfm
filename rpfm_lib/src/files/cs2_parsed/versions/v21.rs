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

    pub fn read_v21<R: ReadBytes>(&mut self, data: &mut R) -> Result<()> {
        self.str_1 = data.read_sized_string_u8()?;
        self.matrix_1 = Transform4x4::decode(data, &None)?;
        self.int_1 = data.read_i32()?;

        // Pieces
        for _ in 0..data.read_u32()? {
            let mut piece = Piece::default();

            piece.str_2 = data.read_sized_string_u8()?;
            piece.str_3 = data.read_sized_string_u8()?;
            piece.matrix_2 = Transform4x4::decode(data, &None)?;
            piece.int_3 = data.read_i32()?;
            piece.int_4 = data.read_i32()?;

            // Pieces
            for _ in 0..data.read_u32()? {
                let mut destruct = Destruct::default();
                destruct.key = data.read_sized_string_u16()?;
                destruct.i_1 = data.read_u32()?;

                // Collision outlines?
                for _ in 0..data.read_u32()? {
                    destruct.collision_outlines.push(CollisionOutline {
                        key: data.read_sized_string_u16()?,
                        line: Outline3d::decode(data, &None)?,
                        uk_1: data.read_u32()?,
                    });
                }

                // Pipes.
                for _ in 0..data.read_u32()? {
                    destruct.pipes.push(Pipe {
                        key: data.read_sized_string_u16()?,
                        line: Outline3d::decode(data, &None)?,
                        uk_1: data.read_u32()?,
                    });

                }

                // This is the weird orange line in terry?
                for _ in 0..data.read_u32()? {
                    let mut thingies = vec![];
                    for _ in 0..data.read_u32()? {
                        thingies.push(OrangeThingy {
                            f_1: data.read_f32()?,
                            f_2: data.read_f32()?,
                            u_1: data.read_u32()?,
                        });
                    }

                    destruct.orange_thingies.push(thingies);
                }

                // Platforms.
                for _ in 0..data.read_u32()? {
                    destruct.platforms.push(Platform {
                        f_1: data.read_f32()?,
                        f_2: data.read_f32()?,
                        f_3: data.read_f32()?,
                        line: Outline3d::decode(data, &None)?,
                        b_1: data.read_bool()?,
                        b_2: data.read_bool()?,
                        b_3: data.read_bool()?,
                    });
                }

                destruct.i_2 = data.read_u8()? as i32;
                destruct.m_1 = Transform3x4::decode(data, &None)?;

                // EF lines.
                for _ in 0..data.read_u32()? {
                    destruct.ef_lines.push(EFLine {
                        key: data.read_sized_string_u16()?,
                        uk_1: data.read_u32()?,
                        f_1: data.read_f32()?,
                        f_2: data.read_f32()?,
                        f_3: data.read_f32()?,
                        f_4: data.read_f32()?,
                        f_5: data.read_f32()?,
                        f_6: data.read_f32()?,
                        f_7: data.read_f32()?,
                        f_8: data.read_f32()?,
                        f_9: data.read_f32()?,
                        uk_2: data.read_u32()?,
                    });
                }

                // Docking lines.
                for _ in 0..data.read_u32()? {
                    destruct.docking_lines.push(DockingLine {
                        key: data.read_sized_string_u16()?,
                        f_0: data.read_f32()?,
                        f_1: data.read_f32()?,
                        f_2: data.read_f32()?,
                        f_3: data.read_f32()?,
                        f_4: data.read_f32()?,
                        f_5: data.read_f32()?
                    });
                }

                destruct.f_1 = data.read_f32()?;
                destruct.f_2 = data.read_f32()?;
                destruct.f_3 = data.read_f32()?;
                destruct.f_4 = data.read_f32()?;
                destruct.f_5 = data.read_f32()?;

                piece.destructs.push(destruct);
            }

            piece.f_6 = data.read_f32()?;
            self.pieces.push(piece);

        }

        Ok(())
    }

    pub fn write_v21<W: WriteBytes>(&mut self, buffer: &mut W) -> Result<()> {
        buffer.write_sized_string_u8(&self.str_1)?;

        self.matrix_1.encode(buffer, &None)?;

        buffer.write_i32(self.int_1)?;
        buffer.write_u32(self.pieces.len() as u32)?;
        for piece in &mut self.pieces {
            buffer.write_sized_string_u8(&piece.str_2)?;
            buffer.write_sized_string_u8(&piece.str_3)?;

            piece.matrix_2.encode(buffer, &None)?;

            buffer.write_i32(piece.int_3)?;
            buffer.write_i32(piece.int_4)?;
            buffer.write_u32(piece.destructs.len() as u32)?;
            for destruct in &mut piece.destructs {
                buffer.write_sized_string_u16(&destruct.key)?;
                buffer.write_u32(destruct.i_1)?;

                buffer.write_u32(destruct.collision_outlines.len() as u32)?;
                for outline in &mut destruct.collision_outlines {
                    buffer.write_sized_string_u16(&outline.key)?;
                    outline.line.encode(buffer, &None)?;
                    buffer.write_u32(outline.uk_1)?;
                }

                buffer.write_u32(destruct.pipes.len() as u32)?;
                for pipe in &mut destruct.pipes {
                    buffer.write_sized_string_u16(&pipe.key)?;
                    pipe.line.encode(buffer, &None)?;
                    buffer.write_u32(pipe.uk_1)?;
                }

                buffer.write_u32(destruct.orange_thingies.len() as u32)?;
                for orange_thingies in &destruct.orange_thingies {

                    buffer.write_u32(orange_thingies.len() as u32)?;
                    for orange_thingy in orange_thingies.iter() {
                        buffer.write_f32(orange_thingy.f_1)?;
                        buffer.write_f32(orange_thingy.f_2)?;
                        buffer.write_u32(orange_thingy.u_1)?;
                    }
                }

                buffer.write_u32(destruct.platforms.len() as u32)?;
                for platform in &mut destruct.platforms {
                    buffer.write_f32(platform.f_1)?;
                    buffer.write_f32(platform.f_2)?;
                    buffer.write_f32(platform.f_3)?;

                    platform.line.encode(buffer, &None)?;

                    buffer.write_bool(platform.b_1)?;
                    buffer.write_bool(platform.b_2)?;
                    buffer.write_bool(platform.b_3)?;
                }

                buffer.write_u8(destruct.i_2 as u8)?;
                destruct.m_1.encode(buffer, &None)?;

                buffer.write_u32(destruct.ef_lines.len() as u32)?;
                for ef_line in &mut destruct.ef_lines {
                    buffer.write_sized_string_u16(&ef_line.key)?;
                    buffer.write_u32(ef_line.uk_1)?;

                    buffer.write_f32(ef_line.f_1)?;
                    buffer.write_f32(ef_line.f_2)?;
                    buffer.write_f32(ef_line.f_3)?;
                    buffer.write_f32(ef_line.f_4)?;
                    buffer.write_f32(ef_line.f_5)?;
                    buffer.write_f32(ef_line.f_6)?;
                    buffer.write_f32(ef_line.f_7)?;
                    buffer.write_f32(ef_line.f_8)?;
                    buffer.write_f32(ef_line.f_9)?;

                    buffer.write_u32(ef_line.uk_2)?;
                }

                buffer.write_u32(destruct.docking_lines.len() as u32)?;
                for docking_line in &mut destruct.docking_lines {
                    buffer.write_sized_string_u16(&docking_line.key)?;
                    buffer.write_f32(docking_line.f_0)?;

                    buffer.write_f32(docking_line.f_1)?;
                    buffer.write_f32(docking_line.f_2)?;
                    buffer.write_f32(docking_line.f_3)?;
                    buffer.write_f32(docking_line.f_4)?;
                    buffer.write_f32(docking_line.f_5)?;
                }

                buffer.write_f32(destruct.f_1)?;
                buffer.write_f32(destruct.f_2)?;
                buffer.write_f32(destruct.f_3)?;
                buffer.write_f32(destruct.f_4)?;
                buffer.write_f32(destruct.f_5)?;
            }

            buffer.write_f32(piece.f_6)?;
        }

        Ok(())
    }
}

