//---------------------------------------------------------------------------//
// Copyright (c) 2017-2024 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! This module contains the CaVp8 functions that are specific to Ivf formatted video files.
//!
//! All the functions here are internal, so they should be either private or public only within this crate.

use fraction::GenericFraction;

use crate::binary::{ReadBytes, WriteBytes};
use crate::error::Result;
use crate::utils::*;

use super::*;

const HEADER_LENGTH_IVF: u16 = 32;

/// Key frame marker of a frame in IVF format.
const KEY_FRAME_MARKER: &[u8; 3] = &[0x9D, 0x01, 0x2A];

//---------------------------------------------------------------------------//
//                              Implementation
//---------------------------------------------------------------------------//

impl Video {

    /// This function creates a `CaVp8` from a readable source of data containing a video of Ivf format.
    ///
    /// NOTE: this function expects the data cursor to be at `start + 4`, meaning just after the signature.
    pub(crate) fn read_ivf<R: ReadBytes>(data: &mut R) -> Result<Self> {
        let format = SupportedFormats::Ivf;

        let version = data.read_u16()?;
        let header_len = data.read_u16()?;
        let codec_four_cc = data.read_string_u8(4)?;
        let width = data.read_u16()?;
        let height = data.read_u16()?;
        let timebase_denominator = data.read_u32()?;
        let timebase_numerator = data.read_u32()?;
        let num_frames = data.read_u32()?;
        let _unused = data.read_u32()?;

        // Check we decoded the header correctly.
        check_size_mismatch(data.stream_position()? as usize, header_len as usize)?;

        let mut frame_table = Vec::with_capacity(num_frames as usize);
        let mut frame_data = vec![];
        let mut frame_offset = 0;

        for _ in 0..num_frames {
            let size = data.read_u32()?;
            let _timestamp = data.read_u64()?;

            let frame_raw_data = data.read_slice(size as usize, false)?;
            let is_key_frame = &frame_raw_data[3..6] == KEY_FRAME_MARKER;

            let frame = Frame {
                offset: frame_offset,
                size,
                is_key_frame,
            };

            frame_data.extend_from_slice(&frame_raw_data);

            frame_offset += frame.size;
            frame_table.push(frame);
        }

        // Check we decoded the full file correctly.
        let data_len = data.len()?;
        check_size_mismatch(data.stream_position()? as usize, data_len as usize)?;

        Ok(Self {
            format,
            version,
            codec_four_cc,
            width,
            height,
            num_frames,
            extra_data: None,
            framerate: timebase_denominator as f32 / timebase_numerator as f32,
            frame_table,
            frame_data,
        })
    }

    /// This function writes a `CaVp8` into a buffer in the `Ivf` format.
    pub(crate) fn save_ivf<W: WriteBytes>(&self, buffer: &mut W) -> Result<()> {
        buffer.write_string_u8(SIGNATURE_IVF)?;

        // This is technically incorrect, but it allow us to keep the version of a CaVp8 when converting back and forward.
        buffer.write_u16(self.version)?;
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
