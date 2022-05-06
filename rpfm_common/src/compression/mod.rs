//---------------------------------------------------------------------------//
// Copyright (c) 2017-2022 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

// Here should go all the functions related to the compression/decompression of PackedFiles.

use anyhow::{anyhow, Result};
use xz2::read::XzDecoder;
use xz2::stream::Stream;

use std::env::temp_dir;
use std::fs::File;
use std::io::{BufReader, prelude::*, Read, SeekFrom};
use std::path::Path;
use std::process::Command;
use std::u64;

use crate::{decoder::Decoder, encoder::Encoder};

const DATA_CANNOT_BE_DECOMPRESSED: &str = "This is a compressed file and the decompression failed for some reason. This means this PackedFile cannot be opened in RPFM.";


pub trait Compressible {

    /// This function compress the data of a PackedFile, returning the compressed data.
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

/// This trait allow us to easely decode all kind of data from a `&[u8]`.
pub trait Decompressible {
    /// This function decompress the data of a PackedFile, returning the decompressed data.
    fn decompress(&self) -> Result<Vec<u8>>;
}

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
        let mut footer_offset = vec![0; 4];
        reader.seek(SeekFrom::Start(12))?;
        reader.read_exact(&mut footer_offset)?;
        let compressed_data_length = footer_offset.decode_integer_u32(0)?;

        let mut compressed_data = vec![0; compressed_data_length as usize];
        reader.seek(SeekFrom::Start(32))?;
        reader.read_exact(&mut compressed_data)?;

        let mut fixed_data = vec![];
        fixed_data.encode_integer_i32(self.len() as i32);
        fixed_data.extend_from_slice(&[0x5D, 0x00, 0x00, 0x40, 0x00]);
        fixed_data.append(&mut compressed_data);

        Ok(fixed_data)
    }
}

impl Decompressible for &[u8] {

    fn decompress(&self) -> Result<Vec<u8>> {
        if !self.is_empty() {
            if self.len() >= 9 {

                // CA Tweaks their headers to remove 4 bytes per PackedFile, while losing +4GB File Compression Support.
                // We need to fix their headers so the normal LZMA lib can read them.
                let mut fixed_data: Vec<u8> = vec![];
                fixed_data.extend_from_slice(&self[4..8]);
                fixed_data.push(0);
                fixed_data.extend_from_slice(&self[0..4]);
                fixed_data.extend_from_slice(&[0; 4]);
                fixed_data.extend_from_slice(&self[9..]);

                // Vanilla compressed files are LZMA Alone (or legacy) level 3 compressed files, reproducible by compressing them
                // with default settings with 7-Zip. This should do the trick to get them decoded.
                let stream = Stream::new_lzma_decoder(u64::MAX).map_err(|_| anyhow!(DATA_CANNOT_BE_DECOMPRESSED))?;
                let mut encoder = XzDecoder::new_stream(&*fixed_data, stream);
                let mut compress_data = vec![];
                match encoder.read_to_end(&mut compress_data) {
                    Ok(_) => Ok(compress_data),
                    Err(_) => Err(anyhow!(DATA_CANNOT_BE_DECOMPRESSED))
                }
            }
            else { Err(anyhow!(DATA_CANNOT_BE_DECOMPRESSED)) }
        }
        else { Ok(vec![]) }
    }
}
