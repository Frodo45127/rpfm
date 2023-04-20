//---------------------------------------------------------------------------//
// Copyright (c) 2017-2023 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! This is a module to read/write binary Portrait Settings files.
//!
//! Portrait settings are files containing information about the small portrait each unit uses,
//! tipically at the bottom left of the screen (may vary from game to game) or in the Character Screen in campaign.
//!
//! TODO: add format info.

use getset::*;
use serde_derive::{Serialize, Deserialize};

use crate::error::{Result, RLibError};
use crate::binary::{ReadBytes, WriteBytes};
use crate::files::{DecodeableExtraData, Decodeable, EncodeableExtraData, Encodeable};
use crate::utils::check_size_mismatch;

/// Extension used by PortraitSettings.
pub const EXTENSION: &str = ".bin";

mod versions;

#[cfg(test)] mod portrait_settings_test;

//---------------------------------------------------------------------------//
//                              Enum & Structs
//---------------------------------------------------------------------------//

/// This represents an entire PortraitSettings decoded in memory.
#[derive(PartialEq, Clone, Debug, Default, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct PortraitSettings {

    /// Version of the PortraitSettings.
    version: u32,

    /// Entries on the PortraitSettings.
    entries: Vec<Entry>,
}

/// This represents a generic Portrait Settings Entry.
#[derive(PartialEq, Clone, Debug, Default, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct Entry {

    /// Id of the entry. Points to an art set key.
    id: String,

    /// Settings for the head camera.
    ///
    /// This is the porthole camera you see in campaign, at the bottom left.
    camera_settings_head: CameraSetting,

    /// Settings for the body camera. Optional.
    ///
    /// This is the camera used for displaying the full body of the character in their details window in campaign.
    /// This is only needed for characters. Regular units do not have access to the characters window, so it's not needed for them.
    camera_settings_body: Option<CameraSetting>,

    /// Variants? Need more info about this.
    variants: Vec<Variant>
}

/// This represents a Camera setting of a Portrait.
///
/// Note that the camera has an auto-level feature, so the camera may autorotate to compensate vertical rotation (pitch)
/// greater than 90/-90 degrees.
#[derive(PartialEq, Clone, Debug, Default, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct CameraSetting {

    /// Distance from the character to the camera.
    z: f32,

    /// Vertical displacement of the camera.
    y: f32,

    /// Rotation angle of the camera, sideways. In degrees.
    yaw: f32,

    /// Rotation angle of the camera, vertically. In degrees.
    pitch: f32,

    /// Field of View.
    fov: f32,

    /// Skeleton node that the camera will use as default focus point.
    ///
    /// Optional. If provided, all displacementes/rotations are relative to this point.
    skeleton_node: String,
}

/// This represents a generic variant of a Portrait.
#[derive(PartialEq, Eq, Clone, Debug, Default, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct Variant {

    /// Variant Filename. Points to the column of the same name in the Variants table.
    filename: String,

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

        let mut settings = Self::default();
        settings.version = version;

        match version {
            //1 => settings.read_v1(data)?,
            4 => settings.read_v4(data)?,
            _ => Err(RLibError::DecodingPortraitSettingUnsupportedVersion(version as usize))?,
        }

        // Trigger an error if there's left data on the source.
        check_size_mismatch(data.stream_position()? as usize, data.len()? as usize)?;

        Ok(settings)
    }
}

impl Encodeable for PortraitSettings {

    fn encode<W: WriteBytes>(&mut self, buffer: &mut W, _extra_data: &Option<EncodeableExtraData>) -> Result<()> {
        buffer.write_u32(self.version)?;

        match self.version {
            //1 => self.write_v1(buffer)?,
            4 => self.write_v4(buffer)?,
            _ => unimplemented!()
        }

        Ok(())
    }
}


impl PortraitSettings {

    pub fn from_json(data: &str) -> Result<Self> {
        serde_json::from_str(data).map_err(From::from)
    }

    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string_pretty(&self).map_err(From::from)
    }
}
