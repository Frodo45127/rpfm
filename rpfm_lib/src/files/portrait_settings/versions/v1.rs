//---------------------------------------------------------------------------//
// Copyright (c) 2017-2023 Ismael Gutiérrez González. All rights reserved.
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
                distance_1: 0.0,
                distance_body: 0,
            };

            let count = data.read_u32()?;
            let mut variants = vec![];
            for _ in 0..count {
                variants.push(Variant {
                    id: data.read_sized_string_u8()?,
                    file_diffuse: data.read_sized_string_u8()?,
                    file_mask_1: data.read_sized_string_u8()?,
                    file_mask_2: data.read_sized_string_u8()?,
                    file_mask_3: data.read_sized_string_u8()?,
                });
            }

            // None then 0
            dbg!(data.read_sized_string_u8()?);
            dbg!(data.read_f32()?);
            dbg!(data.read_f32()?);
            dbg!(data.read_u16()?);

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
            buffer.write_f32(entry.camera_settings_head.distance_1)?;
            buffer.write_u16(entry.camera_settings_head.distance_body)?;

            match &entry.camera_settings_body {
                Some(camera) => {
                    buffer.write_bool(true)?;
                    buffer.write_f32(camera.distance)?;
                    buffer.write_f32(camera.theta)?;
                    buffer.write_f32(camera.phi)?;
                    buffer.write_f32(camera.fov)?;
                    buffer.write_f32(camera.distance_1)?;
                    buffer.write_u16(camera.distance_body)?;
                },
                None => buffer.write_bool(false)?,
            }

            buffer.write_u32(entry.variants.len() as u32)?;
            for variant in &entry.variants {
                buffer.write_sized_string_u8(&variant.id)?;
                buffer.write_sized_string_u8(&variant.file_diffuse)?;
                buffer.write_sized_string_u8(&variant.file_mask_1)?;
                buffer.write_sized_string_u8(&variant.file_mask_2)?;
                buffer.write_sized_string_u8(&variant.file_mask_3)?;
            }
        }

        Ok(())
    }
}

