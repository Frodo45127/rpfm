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
        self.subversion = data.read_u32()?;
        self.name = data.read_sized_string_u8()?;
        self.mount_bin = data.read_sized_string_u8()?;
        self.uk_string_1 = data.read_sized_string_u8()?;
        self.skeleton_name = data.read_sized_string_u8()?;
        self.locomotion_graph = data.read_sized_string_u8()?;
        self.uk_string_2 = data.read_sized_string_u8()?;

        let entries_count = data.read_u32()?;

        for _ in 0..entries_count {
            let animation_id = data.read_u32()?;
            let blend_in_time = data.read_f32()?;
            let selection_weight = data.read_f32()?;
            let weapon_bone = WeaponBone::from_bits_truncate(data.read_u32()?);
            let uk_1 = data.read_bool()?;
            let refs_count = data.read_u32()?;

            let mut anim_refs = vec![];
            for _ in 0..refs_count {
                let file_path = data.read_sized_string_u8()?;
                let meta_file_path = data.read_sized_string_u8()?;
                let snd_file_path = data.read_sized_string_u8()?;

                let data = AnimRef {
                    file_path,
                    meta_file_path,
                    snd_file_path,
                };
                anim_refs.push(data);
            }
            let entry = Entry {
                animation_id,
                blend_in_time,
                selection_weight,
                weapon_bone,
                uk_1,
                anim_refs,
                ..Default::default()
            };

            self.entries.push(entry);
        }

        Ok(())
    }

    pub fn write_v4<W: WriteBytes>(&self, buffer: &mut W) -> Result<()> {
        buffer.write_u32(self.subversion)?;
        buffer.write_sized_string_u8(&self.name)?;
        buffer.write_sized_string_u8(&self.mount_bin)?;
        buffer.write_sized_string_u8(&self.uk_string_1)?;
        buffer.write_sized_string_u8(&self.skeleton_name)?;
        buffer.write_sized_string_u8(&self.locomotion_graph)?;
        buffer.write_sized_string_u8(&self.uk_string_2)?;

        buffer.write_u32(self.entries.len() as u32)?;
        for entry in &self.entries {
            buffer.write_u32(entry.animation_id)?;
            buffer.write_f32(entry.blend_in_time)?;
            buffer.write_f32(entry.selection_weight)?;
            buffer.write_u32(entry.weapon_bone.bits())?;
            buffer.write_bool(entry.uk_1)?;
            buffer.write_u32(entry.anim_refs.len() as u32)?;
            for anim_ref in &entry.anim_refs {
                buffer.write_sized_string_u8(&anim_ref.file_path)?;
                buffer.write_sized_string_u8(&anim_ref.meta_file_path)?;
                buffer.write_sized_string_u8(&anim_ref.snd_file_path)?;
            }
        }

        Ok(())
    }
}

