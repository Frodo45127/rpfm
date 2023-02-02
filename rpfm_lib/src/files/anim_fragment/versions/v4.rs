//---------------------------------------------------------------------------//
// Copyright (c) 2017-2023 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! This is a module to read/write binary Anim Fragment files, v4.
//!
//! For internal use only.

use crate::error::Result;
use crate::binary::{ReadBytes, WriteBytes};
use crate::files::anim_fragment::*;

//---------------------------------------------------------------------------//
//                            Implementation
//---------------------------------------------------------------------------//

impl AnimFragment {

    pub fn read_v4<R: ReadBytes>(&mut self, data: &mut R) -> Result<()> {
        self.uk1 = data.read_u32()?;
        self.skeleton_1 = data.read_sized_string_u8()?;
        self.uk2 = data.read_u32()?;
        self.skeleton_2 = data.read_sized_string_u8()?;
        self.locomotion_graph = data.read_sized_string_u8()?;
        self.uk_string = data.read_sized_string_u8()?;

        let count = data.read_u32()?;
        dbg!(&self);
        dbg!(count);
        for i in 0..count {
            let uk1 = data.read_u32()?; dbg!(uk1);
            let uk2 = data.read_f32()?; dbg!(uk2);
            let uk3 = data.read_f32()?; dbg!(uk3);
            let uk4 = data.read_u32()?; dbg!(uk4);
            let uk5 = data.read_bool()?; dbg!(uk5);
            let count2 = data.read_u32()?; dbg!(count2);

            let mut anim_data = vec![];
            for i2 in 0..count2 {
                let anim = data.read_sized_string_u8()?; dbg!(&anim);
                let anim_meta = data.read_sized_string_u8()?; dbg!(&anim_meta);
                let anim_snd = data.read_sized_string_u8()?; dbg!(&anim_snd);

                let data = AnimData {
                    anim,
                    anim_meta,
                    anim_snd,
                };
                anim_data.push(data);
            }
            let entry = Entry {
                uk1,
                uk2,
                uk3,
                uk4,
                uk5,
                anim_data
            };

            self.entries.push(entry);
        }

        Ok(())
    }

    pub fn write_v4<W: WriteBytes>(&self, buffer: &mut W) -> Result<()> {
        buffer.write_u32(self.uk1)?;
        buffer.write_sized_string_u8(&self.skeleton_1)?;
        buffer.write_u32(self.uk2)?;
        buffer.write_sized_string_u8(&self.skeleton_2)?;
        buffer.write_sized_string_u8(&self.locomotion_graph)?;
        buffer.write_sized_string_u8(&self.uk_string)?;

        buffer.write_u32(self.entries.len() as u32)?;
        for entry in &self.entries {
            buffer.write_u32(entry.uk1)?;
            buffer.write_f32(entry.uk2)?;
            buffer.write_f32(entry.uk3)?;
            buffer.write_u32(entry.uk4)?;
            buffer.write_bool(entry.uk5)?;
            buffer.write_u32(entry.anim_data.len() as u32)?;
            for data in &entry.anim_data {
                buffer.write_sized_string_u8(&data.anim)?;
                buffer.write_sized_string_u8(&data.anim_meta)?;
                buffer.write_sized_string_u8(&data.anim_snd)?;
            }
        }

        Ok(())
    }
}

