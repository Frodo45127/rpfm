//---------------------------------------------------------------------------//
// Copyright (c) 2017-2023 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! This is a module to read/write binary Anim Fragment files, v2 for Three Kingdoms.
//!
//! For internal use only.

use crate::error::Result;
use crate::binary::{ReadBytes, WriteBytes};
use crate::files::anim_fragment_battle::*;

//---------------------------------------------------------------------------//
//                            Implementation
//---------------------------------------------------------------------------//

impl AnimFragmentBattle {

    pub fn read_v2_3k<R: ReadBytes>(&mut self, data: &mut R) -> Result<()> {
        self.table_name = data.read_sized_string_u8()?;
        self.mount_table_name = data.read_sized_string_u8()?;
        self.unmount_table_name = data.read_sized_string_u8()?;
        self.skeleton_name = data.read_sized_string_u8()?;
        self.is_simple_flight = data.read_bool()?;
        self.is_new_cavalry_tech = data.read_bool()?;

        let entry_count = data.read_u32()?;
        for _ in 0..entry_count {
            let animation_id = data.read_u32()?;
            let blend_in_time = data.read_f32()?;
            let selection_weight = data.read_f32()?;
            let weapon_bone = WeaponBone::from_bits_truncate(data.read_u32()?);
            let single_frame_variant = data.read_bool()?;
            let refs_count = data.read_u32()?;

            let mut anim_refs = vec![];
            for _ in 0..refs_count {
                let file_path = data.read_sized_string_u8()?;
                let meta_file_path = data.read_sized_string_u8()?;
                let snd_file_path = data.read_sized_string_u8()?;

                anim_refs.push(AnimRef {
                    file_path,
                    meta_file_path,
                    snd_file_path,
                });
            }

            self.entries.push(Entry {
                animation_id,
                blend_in_time,
                selection_weight,
                weapon_bone,
                single_frame_variant,
                anim_refs,
                ..Default::default()
            });
        }

        Ok(())
    }

    pub fn write_v2_3k<W: WriteBytes>(&self, buffer: &mut W) -> Result<()> {
        buffer.write_sized_string_u8(&self.table_name)?;
        buffer.write_sized_string_u8(&self.mount_table_name)?;
        buffer.write_sized_string_u8(&self.unmount_table_name)?;
        buffer.write_sized_string_u8(&self.skeleton_name)?;
        buffer.write_bool(self.is_simple_flight)?;
        buffer.write_bool(self.is_new_cavalry_tech)?;

        buffer.write_u32(self.entries.len() as u32)?;
        for entry in &self.entries {
            buffer.write_u32(entry.animation_id)?;
            buffer.write_f32(entry.blend_in_time)?;
            buffer.write_f32(entry.selection_weight)?;
            buffer.write_u32(entry.weapon_bone.bits())?;
            buffer.write_bool(entry.single_frame_variant)?;
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

