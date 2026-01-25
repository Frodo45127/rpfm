//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! DAT audio configuration file format support.
//!
//! DAT files (`.dat`) are audio configuration files used in Total War games to define
//! sound events, parameters, and enumerations. These files work in conjunction with
//! Wwise sound banks to configure audio playback behavior.
//!
//! # File Format
//!
//! DAT files are binary files containing six data blocks:
//! - **Event 0**: Event parameters (name-value pairs with float values)
//! - **Block 1**: Event enumerations (name with list of enumeration values)
//! - **Block 2**: Event enumerations (name with list of enumeration values)
//! - **Voice Events**: Voice event enumerations (name with list of voice event values)
//! - **Block 4**: Event enumeration list (simple string list)
//! - **Block 5**: Event enumeration list (simple string list)
//!
//! # Purpose
//!
//! DAT files define audio event configurations including:
//! - Sound event parameters and their default values
//! - Available enumeration values for event properties
//! - Groupings and categorizations of audio events
//!
//! # Usage
//!
//! ```rust,ignore
//! use rpfm_lib::files::dat::Dat;
//! use rpfm_lib::files::Decodeable;
//!
//! // Decode from binary data
//! let dat = Dat::decode(&mut data, &None)?;
//!
//! // Access event parameters
//! for (event_name, value) in dat.event_0() {
//!     println!("Parameter '{}' = {}", event_name, value);
//! }
//!
//! // Access voice events
//! for (voice_event, values) in dat.voice_events() {
//!     println!("Voice event '{}' has {} values", voice_event, values.len());
//! }
//! ```
//!
//! # File Location
//!
//! These files are typically found at:
//! - `sound/*.dat`
//! - `audio/wwisedata/*.dat`

use getset::*;
use serde_derive::{Serialize, Deserialize};

use crate::binary::{ReadBytes, WriteBytes};
use crate::error::Result;
use crate::files::{Decodeable, EncodeableExtraData, Encodeable};
use crate::utils::*;

use super::DecodeableExtraData;

/// File extension for DAT audio configuration files.
pub const EXTENSION: &str = ".dat";

//#[cfg(test)] mod test_dat;

//---------------------------------------------------------------------------//
//                              Enum & Structs
//---------------------------------------------------------------------------//

/// Represents a DAT audio configuration file decoded in memory.
///
/// Contains six blocks of audio event configuration data including parameters,
/// enumerations, and event lists used to configure Wwise sound playback.
///
/// # Structure
///
/// The file is organized into six sequential blocks, each serving a different purpose
/// in the audio configuration system. The exact semantic meaning of each block may
/// vary between Total War games.
///
/// # Fields
///
/// * `event_0` - Event parameters with float values (e.g., volume, pitch defaults)
/// * `block_1` - Event enumerations with multiple values (e.g., sound categories)
/// * `block_2` - Event enumerations with multiple values (e.g., sound categories)
/// * `voice_events` - Voice event enumerations with associated values
/// * `block_4` - Simple event enumeration list
/// * `block_5` - Simple event enumeration list
///
/// # Examples
///
/// ```rust,ignore
/// let dat = Dat::decode(&mut data, &None)?;
///
/// // Iterate through event parameters
/// for (name, value) in dat.event_0() {
///     println!("Parameter: {} = {}", name, value);
/// }
///
/// // Iterate through voice events
/// for (voice_event, voice_values) in dat.voice_events() {
///     for value in voice_values {
///         println!("  {} -> {}", voice_event, value);
///     }
/// }
/// ```
#[derive(Default, PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct Dat {
    /// Event parameters with floating-point values.
    ///
    /// Maps event parameter names to their default float values.
    /// Typically used for parameters like volume, pitch, or other numeric settings.
    event_0: Vec<(String, f32)>,

    /// Event enumerations with string lists.
    ///
    /// Maps enumeration names to lists of possible string values.
    /// Used for categorical audio properties.
    block_1: Vec<(String, Vec<String>)>,

    /// Event enumerations with string lists.
    ///
    /// Maps enumeration names to lists of possible string values.
    /// Similar to block_1, potentially for different enumeration categories.
    block_2: Vec<(String, Vec<String>)>,

    /// Voice event enumerations with string lists.
    ///
    /// Maps voice event names to lists of possible voice event values.
    /// Used for categorizing and organizing voice-related audio events.
    voice_events: Vec<(String, Vec<String>)>,

    /// Simple event enumeration list.
    ///
    /// List of event enumeration names without associated values.
    /// FIXME: Possible wrong implementation.
    block_4: Vec<String>,

    /// Simple event enumeration list.
    ///
    /// List of event enumeration names without associated values.
    /// Similar to block_4, potentially for a different category.
    block_5: Vec<String>,
}

//---------------------------------------------------------------------------//
//                          Implementation of Dat
//---------------------------------------------------------------------------//

impl Decodeable for Dat {

    fn decode<R: ReadBytes>(data: &mut R, _extra_data: &Option<DecodeableExtraData>) -> Result<Self> {
        let mut decoded = Self::default();
        let data_len = data.len()?;

        for _ in 0..data.read_u32()? {
            let string_size = data.read_u32()?;
            let event_name = data.read_string_u8(string_size as usize)?;
            let event_value = data.read_f32()?;
            decoded.event_0.push((event_name, event_value));
        }

        for _ in 0..data.read_u32()? {
            let string_size = data.read_u32()?;
            let event_enum = data.read_string_u8(string_size as usize)?;

            let mut entries = vec![];
            for _ in 0..data.read_u32()? {
                let string_size = data.read_u32()?;
                let event_enum = data.read_string_u8(string_size as usize)?;
                entries.push(event_enum);
            }
            decoded.block_1.push((event_enum, entries));
        }

        for _ in 0..data.read_u32()? {
            let string_size = data.read_u32()?;
            let event_enum = data.read_string_u8(string_size as usize)?;

            let mut entries = vec![];
            for _ in 0..data.read_u32()? {
                let string_size = data.read_u32()?;
                let event_enum = data.read_string_u8(string_size as usize)?;
                entries.push(event_enum);
            }
            decoded.block_2.push((event_enum, entries));
        }

        for _ in 0..data.read_u32()? {
            let string_size = data.read_u32()?;
            let event_enum = data.read_string_u8(string_size as usize)?;

            let mut entries = vec![];
            for _ in 0..data.read_u32()? {
                let string_size = data.read_u32()?;
                let event_enum = data.read_string_u8(string_size as usize)?;
                entries.push(event_enum);
            }
            decoded.voice_events.push((event_enum, entries));
        }

        for _ in 0..data.read_u32()? {
            let string_size = data.read_u32()?;
            let event_enum = data.read_string_u8(string_size as usize)?;
            decoded.block_4.push(event_enum);
        }

        for _ in 0..data.read_u32()? {
            let string_size = data.read_u32()?;
            let event_enum = data.read_string_u8(string_size as usize)?;
            decoded.block_5.push(event_enum);
        }

        check_size_mismatch(data.stream_position()? as usize, data_len as usize)?;
        Ok(decoded)
    }
}

impl Encodeable for Dat {

    fn encode<W: WriteBytes>(&mut self, buffer: &mut W, _extra_data: &Option<EncodeableExtraData>) -> Result<()> {
        buffer.write_u32(self.event_0.len() as u32)?;
        for entry in self.event_0() {
            buffer.write_u32(entry.0.len() as u32)?;
            buffer.write_string_u8(&entry.0)?;
            buffer.write_f32(entry.1)?;
        }

        buffer.write_u32(self.block_1.len() as u32)?;
        for entry in self.block_1() {
            buffer.write_u32(entry.0.len() as u32)?;
            buffer.write_string_u8(&entry.0)?;

            buffer.write_u32(entry.1.len() as u32)?;
            for subentry in &entry.1 {
                buffer.write_u32(subentry.len() as u32)?;
                buffer.write_string_u8(subentry)?;
            }
        }

        buffer.write_u32(self.block_2.len() as u32)?;
        for entry in self.block_2() {
            buffer.write_u32(entry.0.len() as u32)?;
            buffer.write_string_u8(&entry.0)?;

            buffer.write_u32(entry.1.len() as u32)?;
            for subentry in &entry.1 {
                buffer.write_u32(subentry.len() as u32)?;
                buffer.write_string_u8(subentry)?;
            }
        }

        buffer.write_u32(self.voice_events.len() as u32)?;
        for entry in self.voice_events() {
            buffer.write_u32(entry.0.len() as u32)?;
            buffer.write_string_u8(&entry.0)?;

            buffer.write_u32(entry.1.len() as u32)?;
            for subentry in &entry.1 {
                buffer.write_u32(subentry.len() as u32)?;
                buffer.write_string_u8(subentry)?;
            }
        }

        buffer.write_u32(self.block_4.len() as u32)?;
        for entry in self.block_4() {
            buffer.write_u32(entry.len() as u32)?;
            buffer.write_string_u8(entry)?;
        }

        buffer.write_u32(self.block_5.len() as u32)?;
        for entry in self.block_5() {
            buffer.write_u32(entry.len() as u32)?;
            buffer.write_string_u8(entry)?;
        }

        Ok(())
    }
}
