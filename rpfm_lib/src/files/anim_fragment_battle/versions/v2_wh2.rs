//---------------------------------------------------------------------------//
// Copyright (c) 2017-2023 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! This is a module to read/write binary Anim Fragment files, v2 for Warhammer 2.
//!
//! For internal use only.

use crate::error::Result;
use crate::binary::{ReadBytes, WriteBytes};
use crate::files::anim_fragment_battle::*;

//---------------------------------------------------------------------------//
//                            Implementation
//---------------------------------------------------------------------------//

impl AnimFragmentBattle {

    pub fn read_v2_wh2<R: ReadBytes>(&mut self, data: &mut R) -> Result<()> {
        self.skeleton_name = data.read_sized_string_u8()?;
        self.mount_table_name = data.read_sized_string_u8()?;

        self.min_id = data.read_u32()?;
        self.max_id = data.read_u32()?;

        let entry_count = data.read_u32()?;
        for _ in 0..entry_count {
            let animation_id = data.read_u32()?;
            let slot_id = data.read_u32()?;
            let filename = data.read_sized_string_u8()?;
            let metadata = data.read_sized_string_u8()?;
            let metadata_sound = data.read_sized_string_u8()?;
            let skeleton_type = data.read_sized_string_u8()?;
            let blend_in_time = data.read_f32()?;
            let selection_weight = data.read_f32()?;
            let uk_3 = data.read_u32()?;
            let weapon_bone = WeaponBone::from_bits_truncate(data.read_u32()?);
            let uk_4 = data.read_sized_string_u8()?;
            let single_frame_variant = data.read_bool()?;

            self.entries.push(Entry {
                animation_id,
                slot_id,
                filename,
                metadata,
                metadata_sound,
                skeleton_type,
                blend_in_time,
                selection_weight,
                uk_3,
                weapon_bone,
                uk_4,
                single_frame_variant,
                ..Default::default()
            });
        }

        Ok(())
    }

    pub fn write_v2_wh2<W: WriteBytes>(&self, buffer: &mut W) -> Result<()> {
        buffer.write_sized_string_u8(&self.skeleton_name)?;
        buffer.write_sized_string_u8(&self.mount_table_name)?;
        buffer.write_u32(self.min_id)?;
        buffer.write_u32(self.max_id)?;

        buffer.write_u32(self.entries.len() as u32)?;
        for entry in &self.entries {
            buffer.write_u32(entry.animation_id)?;
            buffer.write_u32(entry.slot_id)?;
            buffer.write_sized_string_u8(&entry.filename)?;
            buffer.write_sized_string_u8(&entry.metadata)?;
            buffer.write_sized_string_u8(&entry.metadata_sound)?;
            buffer.write_sized_string_u8(&entry.skeleton_type)?;
            buffer.write_f32(entry.blend_in_time)?;
            buffer.write_f32(entry.selection_weight)?;
            buffer.write_u32(entry.uk_3)?;
            buffer.write_u32(entry.weapon_bone.bits())?;
            buffer.write_sized_string_u8(&entry.uk_4)?;
            buffer.write_bool(entry.single_frame_variant)?;
        }

        Ok(())
    }
}

