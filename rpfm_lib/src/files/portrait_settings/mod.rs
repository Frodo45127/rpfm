//---------------------------------------------------------------------------//
// Copyright (c) 2017-2022 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! This is a module to read/write binary Portrait Settings files.

use getset::*;
use serde_derive::{Serialize, Deserialize};

use crate::error::Result;
use crate::binary::{ReadBytes, WriteBytes};
use crate::files::{DecodeableExtraData, Decodeable, EncodeableExtraData, Encodeable};
use crate::utils::check_size_mismatch;

/// Extension used by PortraitSettings.
pub const EXTENSION: &str = ".bin";

//#[cfg(test)] mod portrait_settings_test;

//---------------------------------------------------------------------------//
//                              Enum & Structs
//---------------------------------------------------------------------------//

/// This represents an entire PortraitSettings decoded in memory.
#[derive(PartialEq, Clone, Debug, Default, Getters, Setters, Serialize, Deserialize)]
pub struct PortraitSettings {

    /// Version of the PortraitSettings.
    version: u32,

    /// Entries on the PortraitSettings.
    entries: Vec<Entry>,
}

/// This represents a Portrait Settings Entry.
#[derive(PartialEq, Clone, Debug, Default, Getters, Setters, Serialize, Deserialize)]
pub struct Entry {

    /// Id of the entry. Points to an art set key.
    id: String,

    /// Settings for the head camera.
    camera_settings_head: CameraSetting,

    /// Settings for the body camera. Optional.
    camera_settings_body: Option<CameraSetting>,

    /// Variants? Need more info about this.
    variants: Vec<Variant>
}

/// This represents a Camera setting of a Portrait.
#[derive(PartialEq, Clone, Debug, Default, Getters, Setters, Serialize, Deserialize)]
pub struct CameraSetting {

    /// Distance of the camera from the character.
    distance: f32,

    /// Theta Angle of the camera.
    theta: f32,

    /// Phi Angle of the camera.
    phi: f32,

    /// Field of View of the camera.
    fov: f32,

    /// No clue about this.
    distance_1: f32,

    /// No clue about this.
    distance_body: u16,
}

/// This represents a Variant of a Portrait.
#[derive(PartialEq, Eq, Clone, Debug, Default, Getters, Setters, Serialize, Deserialize)]
pub struct Variant {

    /// No idea what this corresponds to.
    id: String,

    /// Path of the diffuse image of the Variant.
    file_diffuse: String,

    /// No idea. Optional.
    file_mask_1: String,

    /// No idea. Optional.
    file_mask_2: String,

    /// No idea. Optional.
    file_mask_3: String,
}

//---------------------------------------------------------------------------//
//                       Implementation of PortraitSettings
//---------------------------------------------------------------------------//

impl Decodeable for PortraitSettings {

    fn decode<R: ReadBytes>(data: &mut R, _extra_data: &Option<DecodeableExtraData>) -> Result<Self> {
        let version = data.read_u32()?;
        let entries_count = data.read_u32()?;
        let mut entries = vec![];
        for _ in 0..entries_count {
            let id = data.read_sized_string_u8()?;
            let camera_settings_head = CameraSetting {
                distance: data.read_f32()?,
                theta: data.read_f32()?,
                phi: data.read_f32()?,
                fov: data.read_f32()?,
                distance_1: data.read_f32()?,
                distance_body: data.read_u16()?,
            };

            let has_body_camera = data.read_bool()?;
            let camera_settings_body = if has_body_camera {
                Some(CameraSetting {
                    distance: data.read_f32()?,
                    theta: data.read_f32()?,
                    phi: data.read_f32()?,
                    fov: data.read_f32()?,
                    distance_1: data.read_f32()?,
                    distance_body: data.read_u16()?,
                })
            } else {
                None
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

            entries.push(Entry {
                id,
                camera_settings_head,
                camera_settings_body,
                variants
            });
        }


        // Trigger an error if there's left data on the source.
        check_size_mismatch(data.stream_position()? as usize, data.len()? as usize)?;

        Ok(Self {
            version,
            entries
        })
    }
}

impl Encodeable for PortraitSettings {

    fn encode<W: WriteBytes>(&mut self, buffer: &mut W, _extra_data: &Option<EncodeableExtraData>) -> Result<()> {
        buffer.write_u32(self.version)?;
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

