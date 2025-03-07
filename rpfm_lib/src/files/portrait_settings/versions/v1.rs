//---------------------------------------------------------------------------//
// Copyright (c) 2017-2024 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! This is a module to read/write binary Portrait Settings files, v1.
//!
//! For internal use only.
//!
//! TODO: Finish it, as there are still fields I don't know what they are.

use crate::error::Result;
use crate::binary::{ReadBytes, WriteBytes};
use crate::files::portrait_settings::*;

//---------------------------------------------------------------------------//
//                       Implementation of PortraitSettings
//---------------------------------------------------------------------------//

impl PortraitSettings {

    pub fn read_v1<R: ReadBytes>(&mut self, data: &mut R) -> Result<()> {
        let entries_count = data.read_u32()?;

        for _ in 0..entries_count {
            let id = data.read_sized_string_u8()?;
            let camera_settings_head = CameraSetting {
                distance: data.read_f32()?,
                theta: data.read_f32()?,
                phi: data.read_f32()?,
                fov: data.read_f32()?,
                ..Default::default()
            };

            let mut variants = vec![];
            for _ in 0..data.read_u32()? {
                variants.push(Variant {
                    filename: data.read_sized_string_u8()?,
                    file_diffuse: data.read_sized_string_u8()?,
                    file_mask_1: data.read_sized_string_u8()?,
                    file_mask_2: data.read_sized_string_u8()?,
                    file_mask_3: data.read_sized_string_u8()?,
                    season: data.read_sized_string_u8()?,
                    level: data.read_i32()?,
                    age: data.read_i32()?,
                    politician: data.read_bool()?,
                    faction_leader: data.read_bool()?,
                });
            }

            self.entries.push(Entry {
                id,
                camera_settings_head,
                camera_settings_body: None,
                variants
            });
        }

        Ok(())
    }

    pub fn write_v1<W: WriteBytes>(&self, buffer: &mut W) -> Result<()> {
        buffer.write_u32(self.entries.len() as u32)?;

        for entry in &self.entries {
            buffer.write_sized_string_u8(&entry.id)?;
            buffer.write_f32(entry.camera_settings_head.distance)?;
            buffer.write_f32(entry.camera_settings_head.theta)?;
            buffer.write_f32(entry.camera_settings_head.phi)?;
            buffer.write_f32(entry.camera_settings_head.fov)?;

            buffer.write_u32(entry.variants.len() as u32)?;
            for variant in &entry.variants {
                buffer.write_sized_string_u8(&variant.filename)?;
                buffer.write_sized_string_u8(&variant.file_diffuse)?;
                buffer.write_sized_string_u8(&variant.file_mask_1)?;
                buffer.write_sized_string_u8(&variant.file_mask_2)?;
                buffer.write_sized_string_u8(&variant.file_mask_3)?;
                buffer.write_sized_string_u8(&variant.season)?;
                buffer.write_i32(variant.level)?;
                buffer.write_i32(variant.age)?;
                buffer.write_bool(variant.politician)?;
                buffer.write_bool(variant.faction_leader)?;
            }
        }

        Ok(())
    }
}

