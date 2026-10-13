//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

use serde_derive::{Serialize, Deserialize};

use crate::binary::{ReadBytes, WriteBytes};
use crate::error::{Result, RLibError};
use crate::files::{Decodeable, EncodeableExtraData, Encodeable};

use super::*;

//---------------------------------------------------------------------------//
//                              Enum & Structs
//---------------------------------------------------------------------------//

/// A prop as Attila stores it: with its key, and its data in a marker of version 1 or 2.
///
/// Version 1 only has the first of the four flag bytes. Flags are kept as bytes, as some files have values other than 0 and 1 in them.
#[derive(Default, PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct PropMarker {

    /// Path of the model of the prop.
    key: String,

    /// Version of the marker.
    serialise_version: u16,

    /// Transform of the prop.
    transform: Transform4x4,

    /// If the prop is clamped to the surface below it.
    clamp_to_surface: bool,

    /// Type of the prop.
    prop_type: u32,

    /// If the prop is allowed outside the playable area.
    allow_in_outfield: bool,

    /// Parallax of the decal of the prop.
    decal_parallax: f32,

    /// Scale of the decal of the prop.
    decal_scale: Point3d,

    /// If the decal of the prop is tiled.
    decal_tiled: u8,

    /// If the prop is fauna. Version 2 only.
    is_fauna: u8,

    /// If the prop is shown with snow. Version 2 only.
    snow: u8,

    /// If the prop is shown when destroyed. Version 2 only.
    destruction: u8,
}

//---------------------------------------------------------------------------//
//                        Implementation of PropMarker
//---------------------------------------------------------------------------//

impl Decodeable for PropMarker {

    fn decode<R: ReadBytes>(data: &mut R, extra_data: &Option<DecodeableExtraData>) -> Result<Self> {
        let mut marker = Self {
            key: data.read_sized_string_u8()?,
            serialise_version: data.read_u16()?,
            ..Default::default()
        };

        if !matches!(marker.serialise_version, 1 | 2) {
            return Err(RLibError::DecodingFastBinUnsupportedVersion(String::from("PropMarker"), marker.serialise_version));
        }

        marker.transform = Transform4x4::decode(data, extra_data)?;
        marker.clamp_to_surface = data.read_bool()?;
        marker.prop_type = data.read_u32()?;
        marker.allow_in_outfield = data.read_bool()?;
        marker.decal_parallax = data.read_f32()?;
        marker.decal_scale = Point3d::decode(data, extra_data)?;
        marker.decal_tiled = data.read_u8()?;

        if marker.serialise_version == 2 {
            marker.is_fauna = data.read_u8()?;
            marker.snow = data.read_u8()?;
            marker.destruction = data.read_u8()?;
        }

        Ok(marker)
    }
}

impl Encodeable for PropMarker {

    fn encode<W: WriteBytes>(&mut self, buffer: &mut W, extra_data: &Option<EncodeableExtraData>) -> Result<()> {
        if !matches!(self.serialise_version, 1 | 2) {
            return Err(RLibError::EncodingFastBinUnsupportedVersion(String::from("PropMarker"), self.serialise_version));
        }

        buffer.write_sized_string_u8(&self.key)?;
        buffer.write_u16(self.serialise_version)?;
        self.transform.encode(buffer, extra_data)?;
        buffer.write_bool(self.clamp_to_surface)?;
        buffer.write_u32(self.prop_type)?;
        buffer.write_bool(self.allow_in_outfield)?;
        buffer.write_f32(self.decal_parallax)?;
        self.decal_scale.encode(buffer, extra_data)?;
        buffer.write_u8(self.decal_tiled)?;

        if self.serialise_version == 2 {
            buffer.write_u8(self.is_fauna)?;
            buffer.write_u8(self.snow)?;
            buffer.write_u8(self.destruction)?;
        }

        Ok(())
    }
}
