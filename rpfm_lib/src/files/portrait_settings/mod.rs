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
//! tipically at the bottom left of the screen (may vary from game to game), when in battle or in campaign.
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

//#[cfg(test)] mod portrait_settings_test;

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
    camera_settings_head: CameraSetting,

    /// Settings for the body camera. Optional.
    camera_settings_body: Option<CameraSetting>,

    /// Variants? Need more info about this.
    variants: Vec<Variant>
}

/// This represents a Camera setting of a Portrait.
#[derive(PartialEq, Clone, Debug, Default, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
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

/// This represents a generic variant of a Portrait.
#[derive(PartialEq, Eq, Clone, Debug, Default, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
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

