//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Animation file handler with partial support.
//!
//! This module provides the [`Anim`] type for handling animation files (`.anim`) in
//! Total War PackFiles. Animation files contain skeletal animation data used by the
//! game engine for character and unit animations.
//!
//! # File Format
//!
//! Animation files consist of a header followed by binary animation data. The header
//! contains metadata about the animation:
//! - Version number
//! - Frame rate
//! - Skeleton name
//! - End time (duration)
//! - Bone count
//!
//! # Limited Support
//!
//! Support is currently limited because:
//! - **Header only**: Only the header is parsed into structured fields
//! - **Binary data**: Animation data remains as raw bytes
//! - **No keyframe access**: Individual animation keyframes cannot be accessed or modified
//!
//! This allows basic metadata inspection and preservation of the animation data when
//! re-encoding, but does not enable deep editing of animation curves or keyframes.
//!
//! # Use Cases
//!
//! - Extracting animation metadata (skeleton name, frame rate, duration)
//! - Identifying which skeleton an animation is for
//! - Re-packing animations without modification
//! - Bulk operations on animation files
//!
//! # Example
//!
//! ```no_run
//! use rpfm_lib::files::{Decodeable, anim::Anim};
//! use std::io::Cursor;
//!
//! # let anim_bytes = vec![];
//! let mut reader = Cursor::new(anim_bytes);
//! let anim = Anim::decode(&mut reader, &None).unwrap();
//!
//! // Access metadata
//! println!("Skeleton: {}", anim.skeleton_name());
//! println!("Frame rate: {} fps", anim.frame_rate());
//! println!("Duration: {} seconds", anim.end_time());
//! println!("Bones: {}", anim.bone_count());
//! ```

use getset::*;
use serde_derive::{Serialize, Deserialize};

use crate::binary::{ReadBytes, WriteBytes};
use crate::error::Result;
use crate::files::{DecodeableExtraData, Decodeable, EncodeableExtraData, Encodeable};
use crate::utils::check_size_mismatch;

/// Extension for animation files.
pub const EXTENSION: &str = ".anim";

//mod versions;

#[cfg(test)] mod anim_test;

//---------------------------------------------------------------------------//
//                              Enum & Structs
//---------------------------------------------------------------------------//

/// Partially decoded animation file.
///
/// Contains parsed header metadata and raw binary animation data. The header provides
/// information about the animation's properties, while the actual keyframe data remains
/// in binary form.
///
/// # Fields
///
/// * `version` - Animation file format version
/// * `uk_1` - Unknown field (purpose not yet identified)
/// * `frame_rate` - Animation playback speed in frames per second
/// * `skeleton_name` - Name of the skeleton this animation is for
/// * `end_time` - Animation duration in seconds
/// * `bone_count` - Number of bones animated in this file
/// * `data` - Raw binary animation data (keyframes, curves, etc.)
///
/// # Getters/Setters
///
/// All fields have public getters, mutable getters, and setters via the `getset` crate:
/// - `version()`, `version_mut()`, `set_version()`
/// - `uk_1()`, `uk_1_mut()`, `set_uk_1()`
/// - `frame_rate()`, `frame_rate_mut()`, `set_frame_rate()`
/// - `skeleton_name()`, `skeleton_name_mut()`, `set_skeleton_name()`
/// - `end_time()`, `end_time_mut()`, `set_end_time()`
/// - `bone_count()`, `bone_count_mut()`, `set_bone_count()`
/// - `data()`, `data_mut()`, `set_data()`
///
/// # Example
///
/// ```no_run
/// use rpfm_lib::files::{Decodeable, anim::Anim};
/// use std::io::Cursor;
///
/// # let anim_data = vec![];
/// let mut reader = Cursor::new(anim_data);
/// let anim = Anim::decode(&mut reader, &None).unwrap();
///
/// // Check if animation matches expected skeleton
/// if anim.skeleton_name().contains("humanoid") {
///     println!("Found humanoid animation: {} seconds at {} fps",
///         anim.end_time(), anim.frame_rate());
/// }
/// ```
#[derive(PartialEq, Clone, Debug, Default, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct Anim {
    /// Animation file format version.
    version: u32,

    /// Unknown field (purpose not yet identified).
    uk_1: u32,

    /// Animation playback speed in frames per second.
    frame_rate: f32,

    /// Name of the skeleton this animation targets.
    skeleton_name: String,

    /// Animation duration in seconds.
    end_time: f32,

    /// Number of bones animated in this file.
    bone_count: u32,

    /// Raw binary animation data (keyframes, curves, etc.).
    data: Vec<u8>,
}

//---------------------------------------------------------------------------//
//                          Implementation of Anim
//---------------------------------------------------------------------------//

impl Decodeable for Anim {

    fn decode<R: ReadBytes>(data: &mut R, _extra_data: &Option<DecodeableExtraData>) -> Result<Self> {
        let mut anim = Self::default();
        anim.version = data.read_u32()?;
        anim.uk_1 = data.read_u32()?;
        anim.frame_rate = data.read_f32()?;
        anim.skeleton_name = data.read_sized_string_u8()?;
        anim.end_time = data.read_f32()?;
        anim.bone_count = data.read_u32()?;

        let data_left = data.len()?.checked_sub(data.stream_position()?);
        if let Some(data_left) = data_left {
            anim.data = data.read_slice(data_left as usize, false)?;
        }

        // If we are not in the last byte, it means we didn't parse the entire file, which means this file is corrupt.
        check_size_mismatch(data.stream_position()? as usize, data.len()? as usize)?;

        Ok(anim)
    }
}

impl Encodeable for Anim {

    fn encode<W: WriteBytes>(&mut self, buffer: &mut W, _extra_data: &Option<EncodeableExtraData>) -> Result<()> {
        buffer.write_u32(self.version)?;
        buffer.write_u32(self.uk_1)?;
        buffer.write_f32(self.frame_rate)?;
        buffer.write_sized_string_u8(self.skeleton_name())?;
        buffer.write_f32(self.end_time)?;
        buffer.write_u32(self.bone_count)?;
        buffer.write_all(self.data())?;

        Ok(())
    }
}
