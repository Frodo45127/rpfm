//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
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
//                           Implementation of TreeList
//---------------------------------------------------------------------------//

/// Versions 1 to 3, used by Attila: no scale, and version 2 has an extra byte per tree.
impl TreeList {

    pub(crate) fn read_v1_to_v3<R: ReadBytes>(&mut self, data: &mut R, _extra_data: &Option<DecodeableExtraData>) -> Result<()> {
        for _ in 0..data.read_u32()? {
            let mut vector = BattleTreeItemVector {
                key: data.read_sized_string_u8()?,
                ..Default::default()
            };

            for _ in 0..data.read_u32()? {
                let mut item = BattleTreeItem {
                    x: data.read_f32()?,
                    y: data.read_f32()?,
                    z: data.read_f32()?,
                    rotation: data.read_u8()?,
                    flags: data.read_u8()?,
                    ..Default::default()
                };

                if self.serialise_version == 2 {
                    item.uk_1 = data.read_u8()?;
                }

                vector.value.push(item);
            }

            self.tree_list.push(vector);
        }

        Ok(())
    }

    pub(crate) fn write_v1_to_v3<W: WriteBytes>(&mut self, buffer: &mut W, _extra_data: &Option<EncodeableExtraData>) -> Result<()> {
        buffer.write_u32(self.tree_list.len() as u32)?;

        for vector in &self.tree_list {
            buffer.write_sized_string_u8(&vector.key)?;
            buffer.write_u32(vector.value.len() as u32)?;
            for item in &vector.value {
                buffer.write_f32(item.x)?;
                buffer.write_f32(item.y)?;
                buffer.write_f32(item.z)?;
                buffer.write_u8(item.rotation)?;
                buffer.write_u8(item.flags)?;

                if self.serialise_version == 2 {
                    buffer.write_u8(item.uk_1)?;
                }
            }
        }

        Ok(())
    }
}
