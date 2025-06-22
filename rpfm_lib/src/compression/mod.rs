//---------------------------------------------------------------------------//
// Copyright (c) 2017-2024 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! This module contains the code to compress/decompress data for Total War games.
//!
//! The traits [`Compressible`] and [`Decompressible`] within this module contain functions to compress and decompress
//! data from/to CA's different supported compression formats. Implementations of these two traits for &[[`u8`]] are provided within this module.
//!
//! Also, a couple of things to take into account:
//! * Due to an game bug, compressing tables tends to cause crashes when starting for some people. This bug seems to have been fixed in WH3, but all other games before WH3
//!   may still suffer from it, so unless manually forced to, this lib will not compress tables in those games. Tables will only be compressed in WH3 and newer games.
//!
//! * Compressed files are **only supported on PFH5 Packs** (Since Total War: Warhammer 2).

use lz4_flex::frame::{FrameDecoder, FrameEncoder};
use lzma_rs::{lzma_compress, lzma_decompress};
use serde_derive::{Serialize, Deserialize};

use std::fmt::Display;
use std::io::{Cursor, Read, Seek, Write};

use crate::binary::{ReadBytes, WriteBytes};
use crate::error::{RLibError, Result};

#[cfg(test)]
mod test;

// LZMA Alone doesn't have a defined magic number, but it always starts with one of these, depending on the compression level.
const MAGIC_NUMBERS_LZMA: [u32; 9] = [
    0x0100005D,
    0x1000005D,
    0x0800005D,
    0x1000005D,
    0x2000005D,
    0x4000005D,
    0x8000005D,
    0x0000005D,
    0x0400005D,
];
const MAGIC_NUMBER_LZ4: u32 = 0x184D2204;
const MAGIC_NUMBER_ZSTD: u32 = 0xfd2fb528;

//---------------------------------------------------------------------------//
//                                  Traits
//---------------------------------------------------------------------------//

/// Internal trait to implement compression over a data type.
pub trait Compressible {

    /// This function compress the data of a file, returning the compressed data.
    fn compress(&self, format: CompressionFormat) -> Result<Vec<u8>>;
}

/// Internal trait to implement decompression over a data type.
pub trait Decompressible {

    /// This function decompress the provided data, returning the decompressed data, or an error if the decompression failed.
    ///
    /// Compression format is auto-detected using each format's magic numbers.
    fn decompress(&self) -> Result<Vec<u8>>;
}

/// Compression formats supported by TW Games.
///
/// Not all games support all formats. Check their game info to know what formats each game support.
#[derive(Debug, Copy, Clone, Default, PartialEq, Serialize, Deserialize)]
pub enum CompressionFormat {

    /// Dummy variant to disable compression.
    #[default]None,

    /// Legacy format. Supported by all PFH5 games (all Post-WH2 games).
    ///
    /// Specifically, Total War games use the Non-Streamed LZMA1 format with the following custom header:
    ///
    /// | Bytes | Type  | Data                                                                                |
    /// | ----- | ----- | ----------------------------------------------------------------------------------- |
    /// |  4    | [u32] | Uncompressed size (as u32, max at 4GB).                                             |
    /// |  1    | [u8]  | LZMA model properties (lc, lp, pb) in encoded form... I think. Usually it's `0x5D`. |
    /// |  4    | [u32] | Dictionary size (as u32)... I think. It's usually `[0x00, 0x00, 0x40, 0x00]`.       |
    ///
    /// For reference, a normal Non-Streamed LZMA1 header (from the original spec) contains:
    ///
    /// | Bytes | Type          | Data                                                        |
    /// | ----- | ------------- | ----------------------------------------------------------- |
    /// |  1    | [u8]          | LZMA model properties (lc, lp, pb) in encoded form.         |
    /// |  4    | [u32]         | Dictionary size (32-bit unsigned integer, little-endian).   |
    /// |  8    | [prim@u64]    | Uncompressed size (64-bit unsigned integer, little-endian). |
    ///
    /// This means one has to move the uncompressed size to the correct place in order for a compressed file to be readable,
    /// and one has to remove the uncompressed size and prepend it to the file in order for the game to read the compressed file.
    Lzma1,

    /// New format introduced in WH3 6.2.
    ///
    /// This is a standard Lz4 implementation, with the following tweaks:
    ///
    /// | Bytes | Type      | Data                                          |
    /// | ----- | --------- | --------------------------------------------- |
    /// |  4    | [u32]     | Uncompressed size (as u32, max at 4GB).       |
    /// |  *    | &[[`u8`]] | Lz4 data, starting with the Lz4 Magic Number. |
    Lz4,

    /// New format introduced in WH3 6.2.
    ///
    /// This is a standard Zstd implementation, with the following tweaks:
    ///
    /// | Bytes | Type      | Data                                            |
    /// | ----- | --------- | ----------------------------------------------- |
    /// |  4    | [u32]     | Uncompressed size (as u32, max at 4GB).         |
    /// |  *    | &[[`u8`]] | Zstd data, starting with the Zstd Magic Number. |
    ///
    /// By default the Zstd compression is done with the checksum and content size flags enabled.
    Zstd,
}

//---------------------------------------------------------------------------//
//                              Implementations
//---------------------------------------------------------------------------//

impl Compressible for [u8] {
    fn compress(&self, format: CompressionFormat) -> Result<Vec<u8>> {
        match format {
            CompressionFormat::None => Ok(self.to_vec()),
            CompressionFormat::Lzma1 => {
                let mut dst = vec![];
                dst.write_i32(self.len() as i32)?;

                let mut compressed_data = vec![];
                let mut src = Cursor::new(self);
                lzma_compress(&mut src, &mut compressed_data).unwrap();

                if compressed_data.len() < 13 {
                    return Err(RLibError::DataCannotBeCompressed);
                }

                dst.extend_from_slice(&compressed_data[..5]);
                dst.extend_from_slice(&compressed_data[13..]);

                Ok(dst)
            },
            CompressionFormat::Lz4 => {
                let mut dst = vec![];
                dst.write_u32(self.len() as u32)?;

                let mut encoder = FrameEncoder::new(&mut dst);
                encoder.write_all(self)?;
                encoder.finish()?;

                Ok(dst)
            },
            CompressionFormat::Zstd => {
                let mut dst = vec![];
                dst.write_u32(self.len() as u32)?;

                let mut encoder = zstd::Encoder::new(&mut dst, 3)?;
                encoder.include_checksum(true)?;
                encoder.include_contentsize(true)?;
                encoder.set_pledged_src_size(Some(self.len() as u64))?;

                let mut src = Cursor::new(self.to_vec());
                std::io::copy(&mut src, &mut encoder)?;
                encoder.finish()?;
                Ok(dst)
            },
        }
    }
}

impl Decompressible for &[u8] {
    fn decompress(&self) -> Result<Vec<u8>> {
        if self.is_empty() {
            return Ok(vec![]);
        }

        // We use the magic numbers to know in what format are the files compressed.
        let mut src = Cursor::new(self);
        let u_size = src.read_u32()?;
        let magic_number = src.read_u32()?;

        let format = if magic_number == MAGIC_NUMBER_ZSTD {
            CompressionFormat::Zstd
        } else if magic_number == MAGIC_NUMBER_LZ4 {
            CompressionFormat::Lz4
        } else if MAGIC_NUMBERS_LZMA.contains(&magic_number) {
            CompressionFormat::Lzma1
        }

        // Special case files marked as compressed but not being compressed. This allows fixing them so they're readable again.
        else {
            CompressionFormat::None
        };

        // Fix the starting position of the file before processing it.
        src.seek_relative(-4)?;

        match format {
            CompressionFormat::None => Ok(self.to_vec()),
            CompressionFormat::Lzma1 => {

                // LZMA1 headers have 13 bytes, but we only have 9 due to using a u32 size.
                if self.len() < 9 {
                    return Err(RLibError::DataCannotBeDecompressed);
                }

                // Unlike other formats, in this one we need to inject the uncompressed size in the file header. Otherwise it won't be a valid lzma file.
                let mut fixed_data: Vec<u8> = Vec::with_capacity(self.len() + 4);
                fixed_data.extend_from_slice(&src.read_slice(5, false)?);
                fixed_data.write_u64(u_size as u64)?;
                src.read_to_end(&mut fixed_data)?;

                // Vanilla compressed files are LZMA Alone (or legacy) level 3 compressed files, reproducible by compressing them
                // with default settings with 7-Zip. This should do the trick to get them decoded.
                let mut dst = Vec::with_capacity(u_size as usize);
                let mut reader = Cursor::new(fixed_data);
                let result = lzma_decompress(&mut reader, &mut dst);

                // Ok, history lesson. That method breaks sometimes due to difference in program's behavior when reading LZMA1 files with uncompressed size set.
                // If that fails, we try passing a unknown size (u64::MAX) instead. This usually deals with the errors.
                if result.is_err() {
                    src.set_position(4);

                    let mut fixed_data = Vec::with_capacity(self.len() + 4);
                    fixed_data.extend_from_slice(&src.read_slice(5, false)?);
                    fixed_data.write_u64(u64::MAX)?;
                    src.read_to_end(&mut fixed_data)?;

                    let mut dst = Vec::with_capacity(u_size as usize);
                    let mut reader = Cursor::new(fixed_data);
                    lzma_decompress(&mut reader, &mut dst)?;

                    Ok(dst)
                } else {
                    Ok(dst)
                }
            },
            CompressionFormat::Lz4 => {
                let mut dst = Vec::with_capacity(u_size as usize);
                let mut reader = FrameDecoder::new(src);
                std::io::copy(&mut reader, &mut dst)?;
                Ok(dst)
            },
            CompressionFormat::Zstd => {
                let mut dst = Vec::with_capacity(u_size as usize);
                zstd::stream::copy_decode(src, &mut dst)?;
                Ok(dst)
            },
        }
    }
}

impl From<&str> for CompressionFormat {
    fn from(value: &str) -> Self {
        match value {
            "Lzma1" => Self::Lzma1,
            "Lz4" => Self::Lz4,
            "Zstd" => Self::Zstd,
            _ => Self::None,
        }
    }
}

impl Display for CompressionFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Lzma1 => write!(f, "Lzma1"),
            Self::Lz4 => write!(f, "Lz4"),
            Self::Zstd => write!(f, "Zstd"),
            Self::None => write!(f, "None"),
        }
    }
}
