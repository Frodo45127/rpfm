//---------------------------------------------------------------------------//
// Copyright (c) 2017-2022 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! This module contains the code to compress/decompress data for Total War games.
//!
//! Total War games use the Non-Streamed LZMA1 format with the following custom header:
//!
//! | Bytes | Type  | Data                                                                                |
//! | ----- | ----- | ----------------------------------------------------------------------------------- |
//! |  4    | [u32] | Uncompressed size (as u32, max at 4GB).                                             |
//! |  1    | [u8]  | LZMA model properties (lc, lp, pb) in encoded form... I think. Usually it's `0x5D`. |
//! |  4    | [u32] | Dictionary size (as u32)... I think. It's usually `[0x00, 0x00, 0x40, 0x00]`.       |
//!
//! For reference, a normal Non-Streamed LZMA1 header (from the original spec) contains:
//!
//! | Bytes | Type  | Data                                                        |
//! | ----- | ----- | ----------------------------------------------------------- |
//! |  1    | [u8]  | LZMA model properties (lc, lp, pb) in encoded form.         |
//! |  4    | [u32] | Dictionary size (32-bit unsigned integer, little-endian).   |
//! |  8    | [prim@u64] | Uncompressed size (64-bit unsigned integer, little-endian). |
//!
//! The traits [`Compressible`] and [`Decompressible`] within this module contain functions to compress/decompress
//! data from/to CA's LZMA1 custom implementation. Implementations of these two traits for &[[`u8`]] are provided within this module.
//!
//! Also, a couple of things to take into account:
//! * **NEVER COMPRESS TABLES**. The games (at least Total War: Warhammer 2) have some kind of issue where
//!   Packs with compressed tables cause crashes on start to random people.
//!
//! * Compressed files are **only supported on PFH5 Packs** (Since Total War: Warhammer 2).

use xz2::{read::XzDecoder, stream::Stream};

use std::env::temp_dir;
use std::fs::File;
use std::io::{BufReader, prelude::*, Read, SeekFrom};
use std::path::Path;
use std::process::Command;
use std::u64;

use crate::binary::{ReadBytes, WriteBytes};
use crate::error::{RLibError, Result};

//---------------------------------------------------------------------------//
//                                  Traits
//---------------------------------------------------------------------------//

/// Internal trait to implement compression over a data type.
pub trait Compressible {

    /// This function compress the data of a file, returning the compressed data.
    ///
    /// Now, some explanation: CA uses Non-Streamed LZMA1 (or LZMA Alone) compressed files.
    /// Xz, the `standard` linux lib to deal with LZMA files has a fucking exception for
    /// Non-Streamed LZMA1 files. So we can decode from it, but not encode to it.
    /// So we do it the hard way: write the uncompressed file to disk, call 7z, compress it
    /// to 7z LZMA1 Level 3 format, read the compressed file, and remove the 7z part.
    /// Sadly, this means we have to ship 7z with RPFM. But hey, we're not the ones doing a
    /// fucking exception to a known format because we don't want to support the original format.
    fn compress(&self, sevenzip_path: &Path) -> Result<Vec<u8>>;
}

/// Internal trait to implement decompression over a data type.
pub trait Decompressible {

    /// This function decompress the provided data, returning the decompressed data, or an error if the decompression failed.
    fn decompress(&self) -> Result<Vec<u8>>;
}

//---------------------------------------------------------------------------//
//                              Implementations
//---------------------------------------------------------------------------//

impl Compressible for [u8] {
    fn compress(&self, sevenzip_path: &Path) -> Result<Vec<u8>> {

        // Prepare both paths, uncompressed and compressed.
        let mut uncompressed_path = temp_dir();
        let mut compressed_path = temp_dir();
        uncompressed_path.push("frodo_best_waifu");
        compressed_path.push("frodo_bestest_waifu.7z");

        // Get the data into the uncompressed file, and launch 7z.
        File::create(&uncompressed_path)?.write_all(self)?;
        Command::new(sevenzip_path).arg("a").arg("-m0=lzma").arg("-mx=3").arg(&compressed_path).arg(&uncompressed_path).output()?;

        // Get the compressed LZMA data (and only that data) from the compressed file. To get it, we know:
        // - The header of a 7z file is 32 bytes.
        // - The bytes 12-16 are the offset of the footer from the end of the header.
        // - We have just one file, so the offset is the exact length of that file.
        // - Then we read the offset from the end of the header. And done.
        let mut reader = BufReader::new(File::open(&compressed_path)?);
        reader.seek(SeekFrom::Start(12))?;
        let compressed_data_length = reader.read_u32()?;

        let mut compressed_data = vec![0; compressed_data_length as usize];
        reader.seek(SeekFrom::Start(32))?;
        reader.read_exact(&mut compressed_data)?;

        let mut fixed_data = vec![];
        fixed_data.write_i32(self.len() as i32)?;
        fixed_data.extend_from_slice(&[0x5D, 0x00, 0x00, 0x40, 0x00]);
        fixed_data.append(&mut compressed_data);

        Ok(fixed_data)
    }
}

impl Decompressible for &[u8] {
    fn decompress(&self) -> Result<Vec<u8>> {
        if self.is_empty() {
            return Ok(vec![]);
        }

        if self.len() < 9 {
            return Err(RLibError::DataCannotBeDecompressed.into());
        }

        // CA Tweaks their headers to remove 4 bytes per file, while losing +4GB File Compression Support.
        // We need to fix their headers so the normal LZMA lib can read them.
        let mut fixed_data: Vec<u8> = vec![];
        fixed_data.extend_from_slice(&self[4..8]);
        fixed_data.push(0);
        fixed_data.extend_from_slice(&self[0..4]);
        fixed_data.extend_from_slice(&[0; 4]);
        fixed_data.extend_from_slice(&self[9..]);

        // Vanilla compressed files are LZMA Alone (or legacy) level 3 compressed files, reproducible by compressing them
        // with default settings with 7-Zip. This should do the trick to get them decoded.
        let stream = Stream::new_lzma_decoder(u64::MAX).map_err(|_| RLibError::DataCannotBeDecompressed)?;
        let mut encoder = XzDecoder::new_stream(&*fixed_data, stream);
        let mut compress_data = vec![];
        encoder.read_to_end(&mut compress_data)?;
        Ok(compress_data)
    }
}
