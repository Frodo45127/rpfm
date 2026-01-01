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

impl TreeList {

    pub(crate) fn read_v4<R: ReadBytes>(&mut self, data: &mut R, _extra_data: &Option<DecodeableExtraData>) -> Result<()> {
        for _ in 0..data.read_u32()? {
            let mut vector = BattleTreeItemVector::default();
            vector.key = data.read_sized_string_u8()?;

            for _ in 0..data.read_u32()? {
                let mut item = BattleTreeItem::default();
                item.x = data.read_f32()?;
                item.y = data.read_f32()?;
                item.z = data.read_f32()?;
                item.rotation = data.read_u8()?;
                item.scale = data.read_f32()?;
                item.flags = data.read_u8()?;

                vector.value.push(item);
            }

            self.tree_list.push(vector);
        }

        Ok(())
    }

    pub(crate) fn write_v4<W: WriteBytes>(&mut self, buffer: &mut W, _extra_data: &Option<EncodeableExtraData>) -> Result<()> {
        buffer.write_u32(self.tree_list.len() as u32)?;

        for vector in &self.tree_list {
            buffer.write_sized_string_u8(&vector.key)?;
            buffer.write_u32(vector.value.len() as u32)?;
            for item in &vector.value {
                buffer.write_f32(item.x)?;
                buffer.write_f32(item.y)?;
                buffer.write_f32(item.z)?;
                buffer.write_u8(item.rotation)?;
                buffer.write_f32(item.scale)?;
                buffer.write_u8(item.flags)?;
            }
        }

        Ok(())
    }
}
