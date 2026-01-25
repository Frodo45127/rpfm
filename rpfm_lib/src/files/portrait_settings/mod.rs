//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Portrait settings for unit and character portraits.
//!
//! This module handles binary portrait settings files that define camera positions
//! and rendering parameters for unit portraits in Total War games. These portraits
//! appear in various places:
//! - Unit cards at the bottom of the battle/campaign UI
//! - Character details windows in campaign
//! - Diplomacy screens
//!
//! # Supported Versions
//!
//! - **Version 1**: Used in Warhammer 2, Warhammer 1, Thrones of Britannia, and Attila
//! - **Version 4**: Used in Warhammer 3
//!
//! # File Location
//!
//! Portrait settings files are typically found at `ui/portraits/` within game packs.

use getset::*;
use serde_derive::{Serialize, Deserialize};

use crate::error::{Result, RLibError};
use crate::binary::{ReadBytes, WriteBytes};
use crate::files::{DecodeableExtraData, Decodeable, EncodeableExtraData, Encodeable};
use crate::utils::check_size_mismatch;

/// Extension used by portrait settings files.
pub const EXTENSION: &str = ".bin";

mod versions;

#[cfg(test)] mod portrait_settings_test;

//---------------------------------------------------------------------------//
//                              Enum & Structs
//---------------------------------------------------------------------------//

/// Portrait settings file containing camera configurations for unit portraits.
///
/// Each entry in this file corresponds to an art set and defines how the camera
/// should be positioned when rendering that unit's portrait.
#[derive(PartialEq, Clone, Debug, Default, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct PortraitSettings {

    /// Format version of this file (1 or 4).
    version: u32,

    /// Portrait entries, one per art set.
    entries: Vec<Entry>,
}

/// A portrait entry defining camera settings for a specific art set.
///
/// Each entry links an art set ID to camera configurations for rendering
/// head and optionally full-body portraits.
#[derive(PartialEq, Clone, Debug, Default, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct Entry {

    /// Art set key this entry applies to.
    ///
    /// References a key in the art set tables (e.g., `land_units_tables`).
    id: String,

    /// Camera settings for the head/portrait view.
    ///
    /// This is the porthole camera used for unit cards in the bottom-left UI area.
    camera_settings_head: CameraSetting,

    /// Camera settings for the full-body view (optional).
    ///
    /// Used in character detail windows in campaign. Only needed for characters
    /// and heroes; regular units don't require body camera settings.
    camera_settings_body: Option<CameraSetting>,

    /// Texture variants for this portrait.
    ///
    /// Allows different textures to be used based on conditions like season,
    /// character level, or faction role.
    variants: Vec<Variant>
}

/// Camera positioning and field-of-view settings for a portrait.
///
/// Defines how the camera is positioned relative to the character model when
/// rendering a portrait. The camera has an auto-level feature that compensates
/// for vertical rotation (pitch) exceeding 90/-90 degrees.
#[derive(PartialEq, Clone, Debug, Default, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct CameraSetting {

    /// Distance from the character along the Z axis (depth).
    z: f32,

    /// Vertical displacement of the camera (height offset).
    y: f32,

    /// Horizontal rotation angle in degrees (left/right).
    yaw: f32,

    /// Vertical rotation angle in degrees (up/down).
    pitch: f32,

    /// Camera distance. Only used in version 1.
    distance: f32,

    /// Spherical coordinate theta. Only used in version 1.
    theta: f32,

    /// Spherical coordinate phi. Only used in version 1.
    phi: f32,

    /// Field of view angle in degrees.
    fov: f32,

    /// Skeleton bone to use as the camera focus point.
    ///
    /// If specified, all camera offsets and rotations are relative to this bone's
    /// position. Common values include head or chest bones.
    skeleton_node: String,
}

/// A texture variant for a portrait entry.
///
/// Variants allow different portrait textures to be used based on game conditions
/// such as season, character level, age, or faction role.
#[derive(PartialEq, Eq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct Variant {

    /// Variant identifier matching the `variant_filename` column in variants tables.
    filename: String,

    /// Path to the diffuse (color) texture for this variant.
    file_diffuse: String,

    /// Path to first mask texture (purpose unknown).
    file_mask_1: String,

    /// Path to second mask texture (purpose unknown).
    file_mask_2: String,

    /// Path to third mask texture (purpose unknown).
    file_mask_3: String,

    /// Season when this variant applies. Only used in version 1.
    season: String,

    /// Character level threshold. Only used in version 1.
    level: i32,

    /// Character age threshold. Only used in version 1.
    age: i32,

    /// Whether this variant is for politicians. Only used in version 1.
    politician: bool,

    /// Whether this variant is for faction leaders. Only used in version 1.
    faction_leader: bool,
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
            1 => settings.read_v1(data)?,
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
            1 => self.write_v1(buffer)?,
            4 => self.write_v4(buffer)?,
            _ => unimplemented!()
        }

        Ok(())
    }
}


impl PortraitSettings {

    /// Deserializes portrait settings from a JSON string.
    pub fn from_json(data: &str) -> Result<Self> {
        serde_json::from_str(data).map_err(From::from)
    }

    /// Serializes this portrait settings to a pretty-printed JSON string.
    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string_pretty(&self).map_err(From::from)
    }
}

impl Default for Variant {
    fn default() -> Self {
        Self {
            filename: Default::default(),
            file_diffuse: Default::default(),
            file_mask_1: Default::default(),
            file_mask_2: Default::default(),
            file_mask_3: Default::default(),
            season: "none".to_owned(),
            level: Default::default(),
            age: Default::default(),
            politician: Default::default(),
            faction_leader: Default::default(),
        }
    }
}
