//---------------------------------------------------------------------------//
// Copyright (c) 2017-2024 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! CA_VP8 are a custom version of a VP8 video by CA. These files contain only video data, no audio.
//!
//! Within this module are functions to convert these files into IVF files, readable by tools such
//! FFMpeg, VLC or MPV.
//!
//! These files can usually be found under the movies folder, with the extension `.ca_vp8`. This format
//! is versioned through a `version` number in the file's header. This lib supports has support for reading
//! and writing the versions 0 and 1.
//!
//! # CA_VP8 Structure
//!
//! ## Header
//! ### V1
//!
//! | Bytes | Type     | Data                       |
//! | ----- | -------- | -------------------------- |
//! | 4     | StringU8 | Signature of the file.     |
//! | 4     | [u32]    | Version of the file.       |
//! | 4     | [u32]    | Length of the header.      |
//! | 4     | StringU8 | FourCC of the video.       |
//! | 2     | [u16]    | Width of the video.        |
//! | 2     | [u16]    | Heigth of the video.       |
//! | 4     | [f32]    | Milliseconds per frame.    |
//! | 4     | [u32]    | Unknown.                   |
//! | 4     | [u32]    | Number of frames - 1.      |
//! | 4     | [u32]    | Offset of the frame table. |
//! | 4     | [u32]    | Number of frames.          |
//! | 4     | [u32]    | Largest frame.             |
//! | 1     | [u8]     | Unknown value.             |
//!
//! ### V0
//!
//! | Bytes | Type     | Data                       |
//! | ----- | -------- | -------------------------- |
//! | 4     | StringU8 | Signature of the file.     |
//! | 4     | [u32]    | Version of the file.       |
//! | 4     | [u32]    | Length of the header - 8.  |
//! | 4     | StringU8 | FourCC of the video.       |
//! | 2     | [u16]    | Width of the video.        |
//! | 2     | [u16]    | Heigth of the video.       |
//! | 4     | [f32]    | Milliseconds per frame.    |
//! | 4     | [u32]    | Unknown.                   |
//! | 4     | [u32]    | Number of frames.          |
//! | 4     | [u32]    | Offset of the frame table. |
//! | 4     | [u32]    | Number of frames.          |
//! | 4     | [u32]    | Largest frame.             |
//!
//! ## Frames Data
//!
//! This is valid for versions 0 and 1.
//!
//! | Bytes                                | Type                                         | Data                                                           |
//! | ------------------------------------ | -------------------------------------------- | -------------------------------------------------------------- |
//! | Frame table's offset - header length | &\[[u8]\]                                    | Frames data, concatenated.                                     |
//! | Until the end of the file            | &\[[Frame Table Entry](#frame-table-entry)\] | List of entries with each frame metadata (position, size,...). |
//!
//! ## Frame Table Entry
//!
//! This is valid for versions 0 and 1.
//!
//! | Bytes      | Type      | Data                                             |
//! | ---------- | --------- | ------------------------------------------------ |
//! | 4          | [u32]     | Offset of the frame from the start of the file.  |
//! | 4          | [u32]     | Size in bytes of the frame's data.               |
//! | 4          | [u32]     | Optional. Unknown value. Only present sometimes. |
//! | 1          | [bool]    | Is the frame a key frame?                        |
//!
//!
//! Credits for this module:
//! - Research and initial implementation for this was done by **John Sirett** here:
//!   - <https://gitlab.com/johnsirett/ca_vp8-reverse>
//!
//! As such, the read/save functions for CaVp8 and Ivf in the submodules of this module (and only those functions)
//! are an exception to the MIT license above and are under the CC-SA 4.0 license, available here:
//! - <https://creativecommons.org/licenses/by-sa/4.0/>

use getset::*;
use serde_derive::{Serialize, Deserialize};

use crate::binary::{ReadBytes, WriteBytes};
use crate::error::{RLibError, Result};
use crate::files::{DecodeableExtraData, Decodeable, EncodeableExtraData, Encodeable};

/// Extensions used by CaVp8 Files.
pub const EXTENSION: &str = ".ca_vp8";

/// Signature/Magic Numbers/Whatever of a IVF video file.
const SIGNATURE_IVF: &str = "DKIF";

/// Signature/Magic Numbers/Whatever of a CaVp8 video file.
const SIGNATURE_CAVP8: &str = "CAMV";

mod ca_vp8;
mod ivf;

#[cfg(test)] mod ca_vp8_test;
#[cfg(test)] mod ivf_test;

//---------------------------------------------------------------------------//
//                              Enum & Structs
//---------------------------------------------------------------------------//

/// This represents an entire CaVp8 File decoded in memory.
#[derive(PartialEq, Clone, Debug, Getters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", set = "pub")]
pub struct Video {

    /// Format of the video file
    format: SupportedFormats,

    /// Version number.
    version: u16,

    /// Codec FourCC (usually 'VP80').
    codec_four_cc: String,

    /// Width of the video in pixels.
    width: u16,

    /// Height of the video in pixels.
    height: u16,

    /// Number of frames on the video.
    num_frames: u32,

    /// Framerate of the video.
    framerate: f32,

    /// Extra unknown data at the end of some headers.
    extra_data: Option<(u8, u32, u32)>,

    /// Frame Table of the video.
    frame_table: Vec<Frame>,

    /// Raw frame data of the video.
    frame_data: Vec<u8>,
}

/// This struct contains the information needed to locate an specific frame from a video within the raw frame data.
#[derive(PartialEq, Eq, Clone, Copy, Debug, Getters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", set = "pub")]
pub struct Frame {

    /// Offset on the data where the frame begins.
    offset: u32,

    /// Size of the frame.
    size: u32,

    /// If the frame is a key frame.
    is_key_frame: bool,
}

/// This enum contains the list of formats this lib supports.
#[derive(PartialEq, Eq, Clone, Copy, Debug, Default, Serialize, Deserialize)]
pub enum SupportedFormats {

    /// Used by CA in CaVp8 files.
    #[default]
    CaVp8,

    /// VP8 IVF standard format.
    Ivf,
}

//---------------------------------------------------------------------------//
//                              Implementation
//---------------------------------------------------------------------------//

impl Decodeable for Video {

    fn decode<R: ReadBytes>(data: &mut R, _extra_data: &Option<DecodeableExtraData>) -> Result<Self> {
        match &*data.read_string_u8(4)? {
            SIGNATURE_IVF => Self::read_ivf(data),
            SIGNATURE_CAVP8 => Self::read_cavp8(data),
            _ => Err(RLibError::DecodingCAVP8UnsupportedFormat)
        }
    }
}

impl Encodeable for Video {
    fn encode<W: WriteBytes>(&mut self, buffer: &mut W, _extra_data: &Option<EncodeableExtraData>) -> Result<()> {
        match self.format {
            SupportedFormats::CaVp8 => self.save_cavp8(buffer),
            SupportedFormats::Ivf => self.save_ivf(buffer),
        }
    }
}
