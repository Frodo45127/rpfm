//---------------------------------------------------------------------------//
// Copyright (c) 2017-2024 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

use getset::*;
use serde_derive::{Serialize, Deserialize};

use crate::error::Result;
use crate::binary::{ReadBytes, WriteBytes};
use crate::files::{DecodeableExtraData, Decodeable, EncodeableExtraData, Encodeable};
use crate::files::bmd::common::*;
use crate::utils::check_size_mismatch;

pub const EXTENSION: &str = ".cs2.parsed";

#[cfg(test)] mod cs2_parsed_test;

//---------------------------------------------------------------------------//
//                              Enum & Structs
//---------------------------------------------------------------------------//

#[derive(PartialEq, Clone, Debug, Default, Getters, Setters, Serialize, Deserialize)]
pub struct Cs2Parsed {
    version: u32,
    str_1: String,
    matrix_1: Transform4x4,
    int_1: i32,
    int_2: u32,
    str_2: String,
    str_3: String,
    matrix_2: Transform4x4,
    int_3: i32,
    pieces: Vec<Piece>,
    f_6: f32,
}


#[derive(PartialEq, Clone, Debug, Default, Getters, Setters, Serialize, Deserialize)]
pub struct Piece {
    key: String,
    i_1: u32,
    collision_outlines: Vec<CollisionOutline>,
    orange_thingies: Vec<Vec<OrangeThingy>>,
    platforms: Vec<Platform>,
    i_2: i32,
    m_1: Transform3x4,
    pipes: Vec<Pipe>,
    ef_lines: Vec<EFLine>,
    docking_lines: Vec<DockingLine>,
    f_1: f32,
    f_2: f32,
    f_3: f32,
    f_4: f32,
    f_5: f32,
}

#[derive(PartialEq, Clone, Debug, Default, Getters, Setters, Serialize, Deserialize)]
pub struct CollisionOutline {
    key: String,
    line: Outline3d,
    uk_1: u32,
}

#[derive(PartialEq, Clone, Debug, Default, Getters, Setters, Serialize, Deserialize)]
pub struct OrangeThingy {
    f_1: f32,
    f_2: f32,
    u_1: u32,
}

#[derive(PartialEq, Clone, Debug, Default, Getters, Setters, Serialize, Deserialize)]
pub struct Platform {
    f_1: f32,
    f_2: f32,
    f_3: f32,
    line: Outline3d,
    b_1: bool,
    b_2: bool,
    b_3: bool,
}


#[derive(PartialEq, Clone, Debug, Default, Getters, Setters, Serialize, Deserialize)]
pub struct Pipe {
    key: String,
    line: Outline3d,
    uk_1: u32,
}

#[derive(PartialEq, Clone, Debug, Default, Getters, Setters, Serialize, Deserialize)]
pub struct EFLine {
    key: String,
    uk_1: u32,
    f_1: f32,
    f_2: f32,
    f_3: f32,
    f_4: f32,
    f_5: f32,
    f_6: f32,
    f_7: f32,
    f_8: f32,
    f_9: f32,
    uk_2: u32,
}

#[derive(PartialEq, Clone, Debug, Default, Getters, Setters, Serialize, Deserialize)]
pub struct DockingLine {
    key: String,
    f_0: f32,
    f_1: f32,
    f_2: f32,
    f_3: f32,
    f_4: f32,
    f_5: f32,
}

//---------------------------------------------------------------------------//
//                           Implementation of Cs2Parsed
//---------------------------------------------------------------------------//

impl Decodeable for Cs2Parsed {

    fn decode<R: ReadBytes>(data: &mut R, extra_data: &Option<DecodeableExtraData>) -> Result<Self> {
        let mut decoded = Self::default();
        decoded.version = data.read_u32()?;
        decoded.str_1 = data.read_sized_string_u8()?;
        decoded.matrix_1 = Transform4x4::decode(data, extra_data)?;
        decoded.int_1 = data.read_i32()?;

        decoded.int_2 = data.read_u32()?;
        decoded.str_2 = data.read_sized_string_u8()?;
        decoded.str_3 = data.read_sized_string_u8()?;
        decoded.matrix_2 = Transform4x4::decode(data, extra_data)?;
        decoded.int_3 = data.read_i32()?;

        // Pieces
        for _ in 0..data.read_u32()? {
            let mut piece = Piece::default();
            piece.key = data.read_sized_string_u16()?;
            piece.i_1 = data.read_u32()?;

            // Collision outlines?
            for _ in 0..data.read_u32()? {
                piece.collision_outlines.push(CollisionOutline {
                    key: data.read_sized_string_u16()?,
                    line: Outline3d::decode(data, extra_data)?,
                    uk_1: data.read_u32()?,
                });
            }

            // Pipes.
            for _ in 0..data.read_u32()? {
                piece.pipes.push(Pipe {
                    key: data.read_sized_string_u16()?,
                    line: Outline3d::decode(data, extra_data)?,
                    uk_1: data.read_u32()?,
                });

            }

            // This is the weird orange line in terry?
            for index in 0..data.read_u32()? {
                piece.orange_thingies.push(vec![]);

                for _ in 0..data.read_u32()? {
                    piece.orange_thingies[index as usize].push(OrangeThingy {
                        f_1: data.read_f32()?,
                        f_2: data.read_f32()?,
                        u_1: data.read_u32()?,
                    });
                }
            }

            // Platforms.
            for _ in 0..data.read_u32()? {
                piece.platforms.push(Platform {
                    f_1: data.read_f32()?,
                    f_2: data.read_f32()?,
                    f_3: data.read_f32()?,
                    line: Outline3d::decode(data, extra_data)?,
                    b_1: data.read_bool()?,
                    b_2: data.read_bool()?,
                    b_3: data.read_bool()?
                });
            }

            piece.i_2 = data.read_i32()?;
            piece.m_1 = Transform3x4::decode(data, extra_data)?;

            // EF lines.
            for _ in 0..data.read_u32()? {
                piece.ef_lines.push(EFLine {
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
                piece.docking_lines.push(DockingLine {
                    key: data.read_sized_string_u16()?,
                    f_0: data.read_f32()?,
                    f_1: data.read_f32()?,
                    f_2: data.read_f32()?,
                    f_3: data.read_f32()?,
                    f_4: data.read_f32()?,
                    f_5: data.read_f32()?
                });
            }

            piece.f_1 = data.read_f32()?;
            piece.f_2 = data.read_f32()?;
            piece.f_3 = data.read_f32()?;
            piece.f_4 = data.read_f32()?;
            piece.f_5 = data.read_f32()?;

            decoded.pieces.push(piece);
        }

        decoded.f_6 = data.read_f32()?;

        // Trigger an error if there's left data on the source.
        check_size_mismatch(data.stream_position()? as usize, data.len()? as usize)?;

        Ok(decoded)
    }
}

impl Encodeable for Cs2Parsed {

    fn encode<W: WriteBytes>(&mut self, buffer: &mut W, extra_data: &Option<EncodeableExtraData>) -> Result<()> {
        buffer.write_u32(self.version)?;
        buffer.write_sized_string_u8(&self.str_1)?;

        self.matrix_1.encode(buffer, extra_data)?;

        buffer.write_i32(self.int_1)?;
        buffer.write_u32(self.int_2)?;
        buffer.write_sized_string_u8(&self.str_2)?;
        buffer.write_sized_string_u8(&self.str_3)?;

        self.matrix_2.encode(buffer, extra_data)?;

        buffer.write_i32(self.int_3)?;
        buffer.write_u32(self.pieces.len() as u32)?;
        for piece in &mut self.pieces {
            buffer.write_sized_string_u16(&piece.key)?;
            buffer.write_u32(piece.i_1)?;

            buffer.write_u32(piece.collision_outlines.len() as u32)?;
            for outline in &mut piece.collision_outlines {
                buffer.write_sized_string_u16(&outline.key)?;
                outline.line.encode(buffer, extra_data)?;
                buffer.write_u32(outline.uk_1)?;
            }

            buffer.write_u32(piece.pipes.len() as u32)?;
            for pipe in &mut piece.pipes {
                buffer.write_sized_string_u16(&pipe.key)?;
                pipe.line.encode(buffer, extra_data)?;
                buffer.write_u32(pipe.uk_1)?;
            }

            buffer.write_u32(piece.orange_thingies.len() as u32)?;
            for orange_thingies in &piece.orange_thingies {

                buffer.write_u32(orange_thingies.len() as u32)?;
                for orange_thingy in orange_thingies.iter() {
                    buffer.write_f32(orange_thingy.f_1)?;
                    buffer.write_f32(orange_thingy.f_2)?;
                    buffer.write_u32(orange_thingy.u_1)?;
                }
            }

            buffer.write_u32(piece.platforms.len() as u32)?;
            for platform in &mut piece.platforms {
                buffer.write_f32(platform.f_1)?;
                buffer.write_f32(platform.f_2)?;
                buffer.write_f32(platform.f_3)?;

                platform.line.encode(buffer, extra_data)?;

                buffer.write_bool(platform.b_1)?;
                buffer.write_bool(platform.b_2)?;
                buffer.write_bool(platform.b_3)?;
            }

            buffer.write_i32(piece.i_2)?;
            piece.m_1.encode(buffer, extra_data)?;

            buffer.write_u32(piece.ef_lines.len() as u32)?;
            for ef_line in &mut piece.ef_lines {
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

            buffer.write_u32(piece.docking_lines.len() as u32)?;
            for docking_line in &mut piece.docking_lines {
                buffer.write_sized_string_u16(&docking_line.key)?;
                buffer.write_f32(docking_line.f_0)?;

                buffer.write_f32(docking_line.f_1)?;
                buffer.write_f32(docking_line.f_2)?;
                buffer.write_f32(docking_line.f_3)?;
                buffer.write_f32(docking_line.f_4)?;
                buffer.write_f32(docking_line.f_5)?;
            }

            buffer.write_f32(piece.f_1)?;
            buffer.write_f32(piece.f_2)?;
            buffer.write_f32(piece.f_3)?;
            buffer.write_f32(piece.f_4)?;
            buffer.write_f32(piece.f_5)?;
        }

        buffer.write_f32(self.f_6)?;

        Ok(())
    }
}
