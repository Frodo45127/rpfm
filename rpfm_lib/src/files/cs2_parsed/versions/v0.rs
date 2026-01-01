//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
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

    pub fn read_v0<R: ReadBytes>(&mut self, data: &mut R) -> Result<()> {
        let mut piece = Piece::default();
        let mut destruct = Destruct::default();
        destruct.collision_3d.read_v0(data)?;

        for _ in 0..data.read_u32()? {
            destruct.action_vfx.push(Vfx {
                key: data.read_sized_string_u16()?,
                matrix_1: Transform4x4::decode(data, &None)?,
            });
        }

        for _ in 0..data.read_u32()? {
            let mut vec = vec![];
            for _ in 0..data.read_u32()? {
                vec.push(data.read_i16()?);
            }
            destruct.bin_data.push(vec);
        }

        piece.destructs.push(destruct);
        self.pieces.push(piece);

        Ok(())
    }

    pub fn write_v0<W: WriteBytes>(&mut self, buffer: &mut W) -> Result<()> {
        for piece in &mut self.pieces {
            for destruct in &mut piece.destructs {
                destruct.collision_3d.write_v0(buffer)?;

                buffer.write_u32(destruct.action_vfx.len() as u32)?;
                for vfx in &mut destruct.action_vfx {
                    buffer.write_sized_string_u16(&vfx.key)?;
                    vfx.matrix_1.encode(buffer, &None)?;
                }

                buffer.write_u32(destruct.bin_data.len() as u32)?;
                for bin_data in &destruct.bin_data {
                    buffer.write_u32(bin_data.len() as u32)?;
                    for bin_data in bin_data {
                        buffer.write_i16(*bin_data)?;
                    }
                }
            }
        }

        Ok(())
    }
}
