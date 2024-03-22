//---------------------------------------------------------------------------//
// Copyright (c) 2017-2024 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! This module contains the CaVp8 functions that are specific to CaVp8 formatted video files.
//!
//! All the functions here are internal, so they should be either private or public only within this crate.

use std::io::SeekFrom;

use crate::binary::{ReadBytes, WriteBytes};
use crate::error::{RLibError, Result};
use crate::utils::check_size_mismatch;

use super::*;

const HEADER_LENGTH_CAVP8_V0: u16 = 40;
const HEADER_LENGTH_CAVP8_V1: u16 = 40;

//---------------------------------------------------------------------------//
//                              Implementation
//---------------------------------------------------------------------------//

impl Video {

    /// This function creates a `CaVp8` from a readable source of data containing a video of CaVp8 format.
    ///
    /// NOTE: this function expects the data cursor to be at `start + 4`, meaning just after the signature.
    pub(crate) fn read_cavp8<R: ReadBytes>(data: &mut R) -> Result<Self> {
        let format = SupportedFormats::CaVp8;

        let version = data.read_u16()?;
        let mut header_len = data.read_u16()?;
        let codec_four_cc = data.read_string_u8(4)?;
        let width = data.read_u16()?;
        let height = data.read_u16()?;
        let ms_per_frame = data.read_f32()?;
        let _mystery_u32 = data.read_u32()?;        // No idea, always 1.
        let num_frames_minus_1 = data.read_u32()?;  // Same as num_frames, but sometimes is num_frames - 1. When it's the same, there are 9 extra bytes in the header.
        let offset_frame_table = data.read_u32()?;
        let num_frames = data.read_u32()?;
        let _largest_frame_size = data.read_u32()?; // Largest frame's size, in bytes. Recalculated on save.

        // If both frame counts are the same, we get some extra data in the header.
        let extra_data = if num_frames_minus_1 == num_frames {
            let mystery_u8 = data.read_u8()?;               // No idea, always 0.
            let mystery_u32_1 = data.read_u32()?;           // No idea, always a very big number.
            let mystery_u32_2 = data.read_u32()?;           // No idea, always a very big number, but smaller than the one above.
            Some((mystery_u8, mystery_u32_1, mystery_u32_2))
        } else {
            None
        };

        // Check the header has been read correctly. We need to add 8 bytes to the header lenght to conpensate for the height first bytes of the file.
        header_len += 8;
        check_size_mismatch(data.stream_position()? as usize, header_len as usize)?;

        // Store the frame data for later access.
        let frame_data_len = offset_frame_table as u64 - data.stream_position()?;
        let frame_data = data.read_slice(frame_data_len as usize, false)?;

        // Brace yourself, wonky workaround incoming!
        // There are some files that, for unknown reasons, have 13 bytes instead of 9 in the frame table.
        // I have no freaking idea what's the logic behind 9/13 bytes, so we go with the ghetto solution:
        // - Frames / 13. If the remainder is 0, we have groups of 13. If not, groups of 9.
        let data_len = data.len()?;
        let frame_table_len = data_len - data.stream_position()?;
        let bells = frame_table_len / 13 == num_frames as u64 && frame_table_len % 13 == 0;

        let mut frame_offset = 0;
        let mut frame_table = Vec::with_capacity(num_frames as usize);
        let mut frame_table_decoded = vec![];

        for _ in 0..num_frames {
            let frame_offset_real = data.read_u32()?;
            let frame_size = data.read_u32()?;
            if bells {
                let _unknown_data = data.read_u32()?;
            }
            let frame_is_key_frame = data.read_bool()?;
            let frame = Frame {
                offset: frame_offset,
                size: frame_size,
                is_key_frame: frame_is_key_frame,
            };

            frame_offset += frame.size;
            frame_table.push(frame);

            let frame_offset_real_end = frame_offset_real + frame.size;
            if frame_offset_real_end as u64 > data.len()? {
                return Err(RLibError::DecodingCAVP8IncorrectOrUnknownFrameSize);
            }

            let x = data.stream_position()?;
            data.seek(SeekFrom::Start(frame_offset_real as u64))?;
            frame_table_decoded.extend_from_slice(&data.read_slice(frame.size as usize, false)?);
            data.seek(SeekFrom::Start(x))?;
        }

        // Check we decoded the full file correctly.
        check_size_mismatch(data.stream_position()? as usize, data_len as usize)?;

        Ok(Self {
            format,
            version,
            codec_four_cc,
            width,
            height,
            num_frames,
            extra_data,
            framerate: 1_000f32 / ms_per_frame,
            frame_table,
            frame_data,
        })
    }

    /// This function writes a `CaVp8` into a buffer in the `CaVp8` format.
    pub(crate) fn save_cavp8<W: WriteBytes>(&self, buffer: &mut W) -> Result<()> {

        let header_lenght = if self.version == 0 {
            if self.extra_data.is_some() {
                HEADER_LENGTH_CAVP8_V0 + 9
            } else {
                HEADER_LENGTH_CAVP8_V0
            }
        } else {
            if self.extra_data.is_some() {
                HEADER_LENGTH_CAVP8_V1 + 9
            } else {
                HEADER_LENGTH_CAVP8_V1
            }
        };


        let header_lenght_broken = if self.version == 0 { header_lenght - 8 } else { header_lenght - 8 };

        buffer.write_string_u8(SIGNATURE_CAVP8)?;
        buffer.write_u16(self.version)?;
        buffer.write_u16(header_lenght_broken)?;
        buffer.write_string_u8(&self.codec_four_cc)?;
        buffer.write_u16(self.width)?;
        buffer.write_u16(self.height)?;

        buffer.write_f32(1_000f32 / self.framerate)?;
        buffer.write_u32(1)?; // _mystery_u32: I don't actually know what this is.

        if self.extra_data.is_some() || self.num_frames == 0 {
            buffer.write_u32(self.num_frames)?;
        } else {
            buffer.write_u32(self.num_frames - 1)?;
        }

        buffer.write_u32(header_lenght as u32 + self.frame_data.len() as u32)?;
        buffer.write_u32(self.num_frames)?;
        buffer.write_u32(self.frame_table.iter().map(|x| x.size).max().unwrap())?;

        if let Some(extra_data) = self.extra_data {
            buffer.write_u8(extra_data.0)?;
            buffer.write_u32(extra_data.1)?;
            buffer.write_u32(extra_data.2)?;
        }

        // Frame data and table.
        buffer.write_all(&self.frame_data)?;

        let mut offset = if self.extra_data.is_some() {
            header_lenght_broken as u32
        } else {
            header_lenght as u32
        };

        for frame in &self.frame_table {
            buffer.write_u32(offset)?;
            buffer.write_u32(frame.size)?;
            buffer.write_bool(frame.is_key_frame)?;
            offset += frame.size;
        }

        Ok(())
    }
}
