//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Audio file passthrough handler.
//!
//! This module provides the [`Audio`] type for handling audio files in Total War PackFiles.
//! It acts as a passthrough, storing raw audio data without decoding or encoding the actual
//! audio formats.
//!
//! # Supported Extensions
//!
//! The following audio file extensions are recognized:
//! - **`.mp3`** - MPEG Audio Layer III
//! - **`.wem`** - Wwise Encoded Media (proprietary audio format)
//! - **`.wav`** - Waveform Audio File Format
//!
//! # Limitations
//!
//! This module does not decode or encode audio data. It only provides raw byte access,
//! allowing external audio libraries or tools to process the data. The files are stored
//! and written back in their original binary format.
//!
//! # Use Cases
//!
//! - Extracting audio files from PackFiles for editing in external tools
//! - Re-packing modified audio files
//! - Analyzing audio file metadata
//! - Custom audio processing with specialized libraries
//!
//! # Example
//!
//! ```
//! use rpfm_lib::files::{Decodeable, audio::Audio};
//! use std::io::Cursor;
//!
//! // Read raw audio data
//! let audio_bytes = vec![/* MP3 or WEM data */];
//! let mut reader = Cursor::new(audio_bytes);
//! let audio = Audio::decode(&mut reader, &None).unwrap();
//!
//! // Access raw data for external processing
//! let raw_data = audio.data();
//! ```

use getset::*;
use serde_derive::{Serialize, Deserialize};

use crate::binary::{ReadBytes, WriteBytes};
use crate::error::Result;
use crate::files::{DecodeableExtraData, Decodeable, EncodeableExtraData, Encodeable};

/// Extension used by audio files.
pub const EXTENSIONS: [&str; 3] = [".mp3", ".wem", ".wav"];

//---------------------------------------------------------------------------//
//                              Enum & Structs
//---------------------------------------------------------------------------//

/// Container for raw audio file data.
///
/// Stores audio data in its original binary format without decoding. This allows
/// audio files to be extracted, modified with external tools, and re-packed.
///
/// # Fields
///
/// * `data` - Raw binary audio data (MP3, WEM, or WAV format)
///
/// # Getters
///
/// Fields have public getters via the `getset` crate:
/// - `data()` - Get reference to the raw audio bytes
///
/// # Example
///
/// ```
/// use rpfm_lib::files::{Decodeable, Encodeable, audio::Audio};
/// use std::io::Cursor;
///
/// # let mp3_data = vec![0xFF, 0xFB]; // MP3 header bytes
/// let mut reader = Cursor::new(mp3_data.clone());
/// let audio = Audio::decode(&mut reader, &None).unwrap();
///
/// assert_eq!(audio.data(), &mp3_data);
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Getters, Setters, Serialize, Deserialize)]
#[getset(get = "pub")]
pub struct Audio {
    /// Raw binary audio data in original format.
    data: Vec<u8>,
}

//---------------------------------------------------------------------------//
//                              Implementations
//---------------------------------------------------------------------------//

impl Decodeable for Audio {

    fn decode<R: ReadBytes>(data: &mut R, _extra_data: &Option<DecodeableExtraData>) -> Result<Self> {
        let len = data.len()?;
        let data = data.read_slice(len as usize, false)?;
        Ok(Self {
            data,
        })
    }
}

impl Encodeable for Audio {

    fn encode<W: WriteBytes>(&mut self, buffer: &mut W, _extra_data: &Option<EncodeableExtraData>) -> Result<()> {
        buffer.write_all(&self.data).map_err(From::from)
    }
}
