//---------------------------------------------------------------------------//
// Copyright (c) 2017-2022 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

/*!
Module with all the code to interact with CA_VP8 PackedFiles.

This is a video format which is basically VP8 with custom changes by CA.

Research and initial implementation for this was done by John Sirett here:
- https://gitlab.com/johnsirett/ca_vp8-reverse

As such, the read/save functions here (and only those functions) are an exception
to the MIT license above and are under the CC-SA 4.0 license, available here:
- https://creativecommons.org/licenses/by-sa/4.0/
!*/

use anyhow::{anyhow, Result};
use fraction::GenericFraction;

use rpfm_common::{decoder::Decoder, encoder::Encoder};
use rpfm_macros::*;

use crate::{Decodeable, Encodeable, PackedFileType};

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

const ERROR_NOT_A_CAVP8_OR_IVF: &str = "This PackedFile is neither a CA_VP8 nor IVF file.";
const ERROR_INCORRECT_FRAME_SIZE: &str = "Incorrect/Unknown Frame size.";

//---------------------------------------------------------------------------//
//                              Enum & Structs
//---------------------------------------------------------------------------//

/// This holds an entire CA_VP8 PackedFile decoded in memory.
#[derive(PartialEq, Clone, Debug, GetRef, Set)]
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
#[derive(PartialEq, Clone, Copy, Debug, GetRef, Set)]
pub enum SupportedFormats {

    /// Used by CA.
    Camv,

    /// VP8 IVF standard format.
    Ivf,
}

/// This enum represents the data to locate and get an specific frame from a video.
#[derive(PartialEq, Clone, Copy, Debug, GetRef, Set)]
pub struct Frame {

    /// Offset on the data where the frame begins.
    offset: u32,

    /// Size of the frame.
    size: u32,
}

//---------------------------------------------------------------------------//
//                           Implementation of CaVp8
//---------------------------------------------------------------------------//

/// Implementation of `CaVp8`.
impl CaVp8 {

    /// This function returns if the provided data corresponds to a video or not.
    pub fn is_video(data: &[u8]) -> bool {
        match data.decode_string_u8(0, 4) {
            Ok(signature) => signature == SIGNATURE_IVF || signature == SIGNATURE_CAMV,
            Err(_) => false,
        }
    }

    /// This function creates a `CaVp8` from a `Vec<u8>` containing a video of CAMV format.
    ///
    /// NOTE: this takes a whole vector, not a reference. The reason is this vector can by enormous and this way
    /// we can avoid duplicates.
    fn read_camv(packed_file_data: &[u8]) -> Result<Self> {
        let format = SupportedFormats::Camv;

        let mut offset = 4;
        let version = packed_file_data.decode_packedfile_integer_i16(offset, &mut offset)?;
        let _header_len = packed_file_data.decode_packedfile_integer_u16(offset, &mut offset)?;
        let codec_four_cc = packed_file_data.decode_string_u8(offset, 4)?;
        offset += 4;
        let width = packed_file_data.decode_packedfile_integer_u16(offset, &mut offset)?;
        let height = packed_file_data.decode_packedfile_integer_u16(offset, &mut offset)?;
        let ms_per_frame = packed_file_data.decode_packedfile_float_f32(offset, &mut offset)?;
        let _mystery_u32 = packed_file_data.decode_packedfile_integer_u32(offset, &mut offset)?;
        let _num_frames_copy = packed_file_data.decode_packedfile_integer_u32(offset, &mut offset)?;
        let offset_frame_table = packed_file_data.decode_packedfile_integer_u32(offset, &mut offset)?;
        let num_frames = packed_file_data.decode_packedfile_integer_u32(offset, &mut offset)?;
        let _largest_frame = packed_file_data.decode_packedfile_integer_u32(offset, &mut offset)?;

        // From here on, it's frame data, then the frame table.
        offset = offset_frame_table as usize;

        // Brace yourself, wonky workaround incoming!
        // There are some files that, for unknown reasons, have 13 bytes instead of 9 in the frame table.
        // I have no freaking idea what's the logic behind 9/13 bytes, so we go with the ghetto solution:
        // - Frames / 13. If the remainder is 0, we have groups of 13. If not, groups of 9.
        let bells = packed_file_data[offset..].len() / 13 == num_frames as usize && packed_file_data[offset..].len() % 13 == 0;
        let mut frame_offset = 0;
        let mut frame_table = vec![];
        let mut frame_data = vec![];

        for _ in 0..num_frames {
            let frame_offset_real = packed_file_data.decode_packedfile_integer_u32(offset, &mut offset)?;
            let frame = Frame {
                offset: frame_offset,
                size: packed_file_data.decode_packedfile_integer_u32(offset, &mut offset)?,
            };
            if bells {
                let _unknown_data = packed_file_data.decode_packedfile_integer_u32(offset, &mut offset)?;
            }
            let _flags = packed_file_data.decode_packedfile_integer_u8(offset, &mut offset)?;

            frame_offset += frame.size;
            frame_table.push(frame);

            let frame_offset_real_end = frame_offset_real + frame.size;
            if frame_offset_real_end as usize > packed_file_data.len() {
                return Err(anyhow!(ERROR_INCORRECT_FRAME_SIZE));
            }

            frame_data.extend_from_slice(&packed_file_data[frame_offset_real as usize..frame_offset_real_end as usize]);
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
    fn read_ivf(packed_file_data: &[u8]) -> Result<Self> {
        let format = SupportedFormats::Ivf;

        let mut offset = 4;
        let version = packed_file_data.decode_packedfile_integer_i16(offset, &mut offset)?;
        let _header_len = packed_file_data.decode_packedfile_integer_u16(offset, &mut offset)?;
        let codec_four_cc = packed_file_data.decode_string_u8(offset, 4)?;
        offset += 4;
        let width = packed_file_data.decode_packedfile_integer_u16(offset, &mut offset)?;
        let height = packed_file_data.decode_packedfile_integer_u16(offset, &mut offset)?;
        let timebase_denominator = packed_file_data.decode_packedfile_integer_u32(offset, &mut offset)?;
        let timebase_numerator = packed_file_data.decode_packedfile_integer_u32(offset, &mut offset)?;
        let num_frames = packed_file_data.decode_packedfile_integer_u32(offset, &mut offset)?;
        let _unused = packed_file_data.decode_packedfile_integer_u32(offset, &mut offset)?;

        let mut frame_table = vec![];
        let mut frame_data = vec![];
        let mut frame_offset = 0;
        for _ in 0..num_frames {
            let size = packed_file_data.decode_packedfile_integer_u32(offset, &mut offset)?;
            let _pts = packed_file_data.decode_packedfile_integer_u64(offset, &mut offset)?;
            let frame = Frame {
                offset: frame_offset,
                size,
            };
            frame_data.extend_from_slice(&packed_file_data[offset..offset + frame.size as usize]);
            offset += frame.size as usize;
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
    fn save_camv(&self) -> Vec<u8> {

        let header_lenght = if self.version == 0 { HEADER_LENGTH_CAMV_V0 } else { HEADER_LENGTH_CAMV_V1 };
        let header_lenght_full = if self.version == 0 { HEADER_LENGTH_CAMV_V0 + 8 } else { HEADER_LENGTH_CAMV_V1 } as u32;
        let mut packed_file = vec![];
        packed_file.encode_string_u8(SIGNATURE_CAMV);
        packed_file.encode_integer_i16(self.version);
        packed_file.encode_integer_u16(header_lenght);
        packed_file.encode_string_u8(&self.codec_four_cc);
        packed_file.encode_integer_u16(self.width);
        packed_file.encode_integer_u16(self.height);

        packed_file.encode_float_f32(1_000f32 / self.framerate);
        packed_file.encode_integer_u32(1);

        // Not a fucking clue why, but this has to have one less frame.
        packed_file.encode_integer_u32(self.num_frames - 1);

        packed_file.encode_integer_u32(header_lenght_full + self.frame_table.iter().map(|x| x.size).sum::<u32>());
        packed_file.encode_integer_u32(self.num_frames);
        packed_file.encode_integer_u32(self.frame_table.iter().map(|x| x.size).max().unwrap());

        // Final header byte, only in version 1.
        if self.version == 1 {
            packed_file.push(0);
        }

        // Frame data and table.
        packed_file.extend_from_slice(&self.frame_data);

        let mut offset = 0;
        for frame in &self.frame_table {
            let frame_data = &self.frame_data[offset..(offset + frame.size as usize)];
            let is_key_frame = if &frame_data[3..6] == KEY_FRAME_MARKER { 1 } else { 0 };

            packed_file.encode_integer_u32(offset as u32 + header_lenght_full);
            packed_file.encode_integer_u32(frame_data.len() as u32);
            packed_file.push(is_key_frame);
            offset += frame.size as usize;
        }

        packed_file
    }

    /// This function creates a `CaVp8` from a `Vec<u8>` containing a video of CAMV format.
    fn save_ivf(&self) -> Vec<u8> {
        let mut packed_file = vec![];
        packed_file.encode_string_u8(SIGNATURE_IVF);
        packed_file.encode_integer_i16(0);
        packed_file.encode_integer_u16(HEADER_LENGTH_IVF);
        packed_file.encode_string_u8(&self.codec_four_cc);
        packed_file.encode_integer_u16(self.width);
        packed_file.encode_integer_u16(self.height);

        let fraction: GenericFraction<u32> = GenericFraction::from(self.framerate);
        packed_file.encode_integer_u32(*fraction.numer().unwrap());
        packed_file.encode_integer_u32(*fraction.denom().unwrap());
        packed_file.encode_integer_u32(self.num_frames);
        packed_file.encode_integer_u32(0);

        let mut offset = 0;
        for (index, frame) in self.frame_table.iter().enumerate() {
            let frame_data = &self.frame_data[offset..(offset + frame.size as usize)];
            packed_file.encode_integer_u32(frame_data.len() as u32);
            packed_file.encode_integer_u64(index as u64);
            packed_file.extend_from_slice(frame_data);
            offset += frame.size as usize;
        }

        packed_file
    }
}


impl Decodeable for CaVp8 {

    fn file_type(&self) -> PackedFileType {
        PackedFileType::CaVp8
    }

    fn decode(packed_file_data: &[u8]) -> Result<Self> {
        match &*packed_file_data.decode_string_u8(0, 4)? {
            SIGNATURE_IVF => Self::read_ivf(packed_file_data),
            SIGNATURE_CAMV => Self::read_camv(packed_file_data),
            _ => Err(anyhow!(ERROR_NOT_A_CAVP8_OR_IVF))
        }
    }
}

impl Encodeable for CaVp8 {
    fn encode(&self) -> Vec<u8> {
        match self.format {
            SupportedFormats::Camv => self.save_camv(),
            SupportedFormats::Ivf => self.save_ivf(),
        }
    }
}
