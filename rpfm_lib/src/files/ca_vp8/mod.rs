//---------------------------------------------------------------------------//
// Copyright (c) 2017-2022 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//!Module with all the code to interact with CA_VP8 PackedFiles.
//!
//!This is a video format which is basically VP8 with custom changes by CA.
//!
//!Research and initial implementation for this was done by John Sirett here:
//!- <https://gitlab.com/johnsirett/ca_vp8-reverse>
//!
//!As such, the read/save functions here (and only those functions) are an exception
//!to the MIT license above and are under the CC-SA 4.0 license, available here:
//!- <https://creativecommons.org/licenses/by-sa/4.0/>

use std::io::SeekFrom;
use getset::*;
use crate::error::{RLibError, Result};
use crate::files::DecodeableExtraData;
use fraction::GenericFraction;

use crate::binary::{ReadBytes, WriteBytes};
use crate::files::{Decodeable, Encodeable};

/// Extensions used by CA_VP8 PackedFiles.
pub const EXTENSION: &str = ".ca_vp8";

/// Signature/Magic Numbers/Whatever of a IVF video file.
pub const SIGNATURE_IVF: &str = "DKIF";

/// Signature/Magic Numbers/Whatever of a CAMV video file.
pub const SIGNATURE_CAMV: &str = "CAMV";

/// Key frame marker of a frame in IVF format.
pub const KEY_FRAME_MARKER: &[u8; 3] = &[0x9D, 0x01, 0x2A];

/// Length of the header of a CAMV video.
const HEADER_LENGTH_CAMV_V0: u16 = 0x20;
const HEADER_LENGTH_CAMV_V1: u16 = 0x29;

/// Length of the header of a IVF video.
const HEADER_LENGTH_IVF: u16 = 32;

//---------------------------------------------------------------------------//
//                              Enum & Structs
//---------------------------------------------------------------------------//

/// This holds an entire CA_VP8 PackedFile decoded in memory.
#[derive(PartialEq, Clone, Debug, Getters, Setters)]
pub struct CaVp8 {

    /// Format of the file.
    format: SupportedFormats,

    /// Version of the file.
    version: i16,

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

    /// Frame Table of the video.
    frame_table: Vec<Frame>,

    /// Raw frame data of the video.
    frame_data: Vec<u8>,
}

/// This enum contains the list of formats RPFM supports.
#[derive(PartialEq, Clone, Copy, Debug)]
pub enum SupportedFormats {

    /// Used by CA.
    Camv,

    /// VP8 IVF standard format.
    Ivf,
}

/// This enum represents the data to locate and get an specific frame from a video.
#[derive(PartialEq, Clone, Copy, Debug, Getters, Setters)]
pub struct Frame {

    /// Offset on the data where the frame begins.
    offset: u32,

    /// Size of the frame.
    size: u32,
}

//---------------------------------------------------------------------------//
//                           Implementation of CaVp8
//---------------------------------------------------------------------------//

impl Decodeable for CaVp8 {

    fn decode<R: ReadBytes>(data: &mut R, _extra_data: Option<DecodeableExtraData>) -> Result<Self> {
        match &*data.read_string_u8(4)? {
            SIGNATURE_IVF => Self::read_ivf(data),
            SIGNATURE_CAMV => Self::read_camv(data),
            _ => Err(RLibError::DecodingCAVP8UnsupportedFormat)
        }
    }
}

impl Encodeable for CaVp8 {
    fn encode<W: WriteBytes>(&mut self, buffer: &mut W) -> Result<()> {
        match self.format {
            SupportedFormats::Camv => self.save_camv(buffer),
            SupportedFormats::Ivf => self.save_ivf(buffer),
        }
    }
}

/// Implementation of `CaVp8`.
impl CaVp8 {
/*
    /// This function returns if the provided data corresponds to a video or not.
    pub fn is_video(data: &[u8]) -> bool {
        match data.read_string_u8(4) {
            Ok(signature) => signature == SIGNATURE_IVF || signature == SIGNATURE_CAMV,
            Err(_) => false,
        }
    }
*/
    /// This function creates a `CaVp8` from a `Vec<u8>` containing a video of CAMV format.
    ///
    /// NOTE: this takes a whole vector, not a reference. The reason is this vector can by enormous and this way
    /// we can avoid duplicates.
    fn read_camv<R: ReadBytes>(packed_file_data: &mut R) -> Result<Self> {
        let format = SupportedFormats::Camv;

        let version = packed_file_data.read_i16()?;
        let _header_len = packed_file_data.read_u16()?;
        let codec_four_cc = packed_file_data.read_string_u8(4)?;
        let width = packed_file_data.read_u16()?;
        let height = packed_file_data.read_u16()?;
        let ms_per_frame = packed_file_data.read_f32()?;
        let _mystery_u32 = packed_file_data.read_u32()?;
        let _num_frames_copy = packed_file_data.read_u32()?;
        let offset_frame_table = packed_file_data.read_u32()?;
        let num_frames = packed_file_data.read_u32()?;
        let _largest_frame = packed_file_data.read_u32()?;

        // From here on, it's frame data, then the frame table.
        packed_file_data.seek(SeekFrom::Start(offset_frame_table as u64))?;

        // Brace yourself, wonky workaround incoming!
        // There are some files that, for unknown reasons, have 13 bytes instead of 9 in the frame table.
        // I have no freaking idea what's the logic behind 9/13 bytes, so we go with the ghetto solution:
        // - Frames / 13. If the remainder is 0, we have groups of 13. If not, groups of 9.
        let len = packed_file_data.len()?;
        let curr_pos = packed_file_data.stream_position()?;

        let bells = len - curr_pos / 13 == num_frames as u64 && len - curr_pos % 13 == 0;
        let mut frame_offset = 0;
        let mut frame_table = vec![];
        let mut frame_data = vec![];

        for _ in 0..num_frames {
            let frame_offset_real = packed_file_data.read_u32()?;
            let frame = Frame {
                offset: frame_offset,
                size: packed_file_data.read_u32()?,
            };
            if bells {
                let _unknown_data = packed_file_data.read_u32()?;
            }
            let _flags = packed_file_data.read_u8()?;

            frame_offset += frame.size;
            frame_table.push(frame);

            let frame_offset_real_end = frame_offset_real + frame.size;
            if frame_offset_real_end as u64 > packed_file_data.len()? {
                return Err(RLibError::DecodingCAVP8IncorrectOrUnknownFrameSize);
            }

            frame_data.extend_from_slice(&packed_file_data.read_slice((frame_offset_real_end - frame_offset_real) as usize, false)?);
        }

        Ok(Self {
            format,
            version,
            codec_four_cc,
            width,
            height,
            num_frames,
            framerate: 1_000f32 / ms_per_frame,
            frame_table,
            frame_data,
        })
    }

    /// This function creates a `CaVp8` from a `Vec<u8>` containing a video of IVF format.
    fn read_ivf<R: ReadBytes>(packed_file_data: &mut R) -> Result<Self> {
        let format = SupportedFormats::Ivf;

        let version = packed_file_data.read_i16()?;
        let _header_len = packed_file_data.read_u16()?;
        let codec_four_cc = packed_file_data.read_string_u8(4)?;
        let width = packed_file_data.read_u16()?;
        let height = packed_file_data.read_u16()?;
        let timebase_denominator = packed_file_data.read_u32()?;
        let timebase_numerator = packed_file_data.read_u32()?;
        let num_frames = packed_file_data.read_u32()?;
        let _unused = packed_file_data.read_u32()?;

        let mut frame_table = vec![];
        let mut frame_data = vec![];
        let mut frame_offset = 0;
        for _ in 0..num_frames {
            let size = packed_file_data.read_u32()?;
            let _pts = packed_file_data.read_u64()?;
            let frame = Frame {
                offset: frame_offset,
                size,
            };
            frame_data.extend_from_slice(&packed_file_data.read_slice(frame.size as usize, false)?);

            frame_offset += frame.size;
            frame_table.push(frame);
        }

        Ok(Self {
            format,
            version,
            codec_four_cc,
            width,
            height,
            num_frames,
            framerate: timebase_denominator as f32 / timebase_numerator as f32,
            frame_table,
            frame_data,
        })
    }

    /// This function creates a `CaVp8` from a `Vec<u8>` containing a video of CAMV format.
    fn save_camv<W: WriteBytes>(&self, buffer: &mut W) -> Result<()> {

        let header_lenght = if self.version == 0 { HEADER_LENGTH_CAMV_V0 } else { HEADER_LENGTH_CAMV_V1 };
        let header_lenght_full = if self.version == 0 { HEADER_LENGTH_CAMV_V0 + 8 } else { HEADER_LENGTH_CAMV_V1 } as u32;

        buffer.write_string_u8(SIGNATURE_CAMV)?;
        buffer.write_i16(self.version)?;
        buffer.write_u16(header_lenght)?;
        buffer.write_string_u8(&self.codec_four_cc)?;
        buffer.write_u16(self.width)?;
        buffer.write_u16(self.height)?;

        buffer.write_f32(1_000f32 / self.framerate)?;
        buffer.write_u32(1)?;

        // Not a fucking clue why, but this has to have one less frame.
        buffer.write_u32(self.num_frames - 1)?;

        buffer.write_u32(header_lenght_full + self.frame_table.iter().map(|x| x.size).sum::<u32>())?;
        buffer.write_u32(self.num_frames)?;
        buffer.write_u32(self.frame_table.iter().map(|x| x.size).max().unwrap())?;

        // Final header byte, only in version 1.
        if self.version == 1 {
            buffer.write_u8(0)?;
        }

        // Frame data and table.
        buffer.write_all(&self.frame_data)?;

        let mut offset = 0;
        for frame in &self.frame_table {
            let frame_data = &self.frame_data[offset..(offset + frame.size as usize)];
            let is_key_frame = if &frame_data[3..6] == KEY_FRAME_MARKER { 1 } else { 0 };

            buffer.write_u32(offset as u32 + header_lenght_full)?;
            buffer.write_u32(frame_data.len() as u32)?;
            buffer.write_u8(is_key_frame)?;
            offset += frame.size as usize;
        }

        Ok(())
    }

    /// This function creates a `CaVp8` from a `Vec<u8>` containing a video of CAMV format.
    fn save_ivf<W: WriteBytes>(&self, buffer: &mut W) -> Result<()> {
        buffer.write_string_u8(SIGNATURE_IVF)?;
        buffer.write_i16(0)?;
        buffer.write_u16(HEADER_LENGTH_IVF)?;
        buffer.write_string_u8(&self.codec_four_cc)?;
        buffer.write_u16(self.width)?;
        buffer.write_u16(self.height)?;

        let fraction: GenericFraction<u32> = GenericFraction::from(self.framerate);
        buffer.write_u32(*fraction.numer().unwrap())?;
        buffer.write_u32(*fraction.denom().unwrap())?;
        buffer.write_u32(self.num_frames)?;
        buffer.write_u32(0)?;

        let mut offset = 0;
        for (index, frame) in self.frame_table.iter().enumerate() {
            let frame_data = &self.frame_data[offset..(offset + frame.size as usize)];
            buffer.write_u32(frame_data.len() as u32)?;
            buffer.write_u64(index as u64)?;
            buffer.write_all(frame_data)?;
            offset += frame.size as usize;
        }

        Ok(())
    }
}
