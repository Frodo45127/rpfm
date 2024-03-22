//---------------------------------------------------------------------------//
// Copyright (c) 2017-2024 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! This is a module to read/write binary Portrait Settings files, v4.
//!
//! For internal use only.

use crate::error::Result;
use crate::binary::{ReadBytes, WriteBytes};
use crate::files::portrait_settings::*;

//---------------------------------------------------------------------------//
//                            Implementation
//---------------------------------------------------------------------------//

impl PortraitSettings {

    pub fn read_v4<R: ReadBytes>(&mut self, data: &mut R) -> Result<()> {
        let entries_count = data.read_u32()?;

        for _ in 0..entries_count {
            let id = data.read_sized_string_u8()?;

            let z = data.read_f32()?;
            let y = data.read_f32()?;
            let yaw = data.read_f32()?;
            let pitch = data.read_f32()?;
            let fov = data.read_f32()?;
            let skeleton_node = data.read_sized_string_u8()?;

            let camera_settings_head = CameraSetting {
                z,
                y,
                yaw,
                pitch,
                fov,
                skeleton_node,
            };

            // Body camera is optional, only used by characters.
            let has_body_camera = data.read_bool()?;
            let camera_settings_body = if has_body_camera {
                let z = data.read_f32()?;
                let y = data.read_f32()?;
                let yaw = data.read_f32()?;
                let pitch = data.read_f32()?;
                let fov = data.read_f32()?;
                let skeleton_node = data.read_sized_string_u8()?;

                Some(CameraSetting {
                    z,
                    y,
                    yaw,
                    pitch,
                    fov,
                    skeleton_node,
                })
            } else {
                None
            };

            let count = data.read_u32()?;
            let mut variants = vec![];
            for _ in 0..count {
                variants.push(Variant {
                    filename: data.read_sized_string_u8()?,
                    file_diffuse: data.read_sized_string_u8()?,
                    file_mask_1: data.read_sized_string_u8()?,
                    file_mask_2: data.read_sized_string_u8()?,
                    file_mask_3: data.read_sized_string_u8()?,
                });
            }

            self.entries.push(Entry {
                id,
                camera_settings_head,
                camera_settings_body,
                variants
            });
        }

        Ok(())
    }

    pub fn write_v4<W: WriteBytes>(&self, buffer: &mut W) -> Result<()> {
        buffer.write_u32(self.entries.len() as u32)?;

        for entry in &self.entries {
            buffer.write_sized_string_u8(&entry.id)?;
            buffer.write_f32(entry.camera_settings_head.z)?;
            buffer.write_f32(entry.camera_settings_head.y)?;
            buffer.write_f32(entry.camera_settings_head.yaw)?;
            buffer.write_f32(entry.camera_settings_head.pitch)?;
            buffer.write_f32(entry.camera_settings_head.fov)?;
            buffer.write_sized_string_u8(&entry.camera_settings_head.skeleton_node)?;

            match &entry.camera_settings_body {
                Some(camera) => {
                    buffer.write_bool(true)?;
                    buffer.write_f32(camera.z)?;
                    buffer.write_f32(camera.y)?;
                    buffer.write_f32(camera.yaw)?;
                    buffer.write_f32(camera.pitch)?;
                    buffer.write_f32(camera.fov)?;
                    buffer.write_sized_string_u8(&camera.skeleton_node)?;
                },
                None => buffer.write_bool(false)?,
            }

            buffer.write_u32(entry.variants.len() as u32)?;
            for variant in &entry.variants {
                buffer.write_sized_string_u8(&variant.filename)?;
                buffer.write_sized_string_u8(&variant.file_diffuse)?;
                buffer.write_sized_string_u8(&variant.file_mask_1)?;
                buffer.write_sized_string_u8(&variant.file_mask_2)?;
                buffer.write_sized_string_u8(&variant.file_mask_3)?;
            }
        }

        Ok(())
    }
}

