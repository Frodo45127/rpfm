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

use xz2::read::XzDecoder;
use xz2::stream::Stream;

use std::env::temp_dir;
use std::fs::File;
use std::io::prelude::*;
use std::io::{BufReader, Read, SeekFrom};

use std::process::Command;
use std::u64;

use rpfm_error::{Error, ErrorKind, Result};
use rpfm_common::encoder::Encoder;
use rpfm_common::decoder::Decoder;
use crate::SETTINGS;
use crate::settings::ZIP_PATH;

/// This function decompress the data of a PackedFile, returning the decompressed data.
pub fn decompress_data(data: &[u8]) -> Result<Vec<u8>> {
    if !data.is_empty() {
        if data.len() >= 9 {

            // CA Tweaks their headers to remove 4 bytes per PackedFile, while losing +4GB File Compression Support.
            // We need to fix their headers so the normal LZMA lib can read them.
            let mut fixed_data: Vec<u8> = vec![];
            fixed_data.extend_from_slice(&data[4..8]);
            fixed_data.push(0);
            fixed_data.extend_from_slice(&data[0..4]);
            fixed_data.extend_from_slice(&[0; 4]);
            fixed_data.extend_from_slice(&data[9..]);

            // Vanilla compressed files are LZMA Alone (or legacy) level 3 compressed files, reproducible by compressing them
            // with default settings with 7-Zip. This should do the trick to get them decoded.
            let stream = Stream::new_lzma_decoder(u64::MAX).map_err(|_| Error::from(ErrorKind::PackedFileDataCouldNotBeDecompressed))?;
            let mut encoder = XzDecoder::new_stream(&*fixed_data, stream);
            let mut compress_data = vec![];
            match encoder.read_to_end(&mut compress_data) {
                Ok(_) => Ok(compress_data),
                Err(_) => Err(ErrorKind::PackedFileDataCouldNotBeDecompressed.into())
            }
        }
        else { Err(ErrorKind::PackedFileDataCouldNotBeDecompressed.into()) }
    }
    else { Ok(vec![]) }
}

/// This function compress the data of a PackedFile, returning the compressed data.
///
/// Now, some explanation: CA uses Non-Streamed LZMA1 (or LZMA Alone) compressed files.
/// Xz, the `standard` linux lib to deal with LZMA files has a fucking exception for
/// Non-Streamed LZMA1 files. So we can decode from it, but not encode to it.
/// So we do it the hard way: write the uncompressed file to disk, call 7z, compress it
/// to 7z LZMA1 Level 3 format, read the compressed file, and remove the 7z part.
/// Sadly, this means we have to ship 7z with RPFM. But hey, we're not the ones doing a
/// fucking exception to a known format because we don't want to support the original format.
pub fn compress_data(data: &[u8]) -> Result<Vec<u8>> {

    match SETTINGS.read().unwrap().paths.get(ZIP_PATH) {
        Some(Some(zip_path)) => {

            // Prepare both paths, uncompressed and compressed.
            let mut uncompressed_path = temp_dir();
            let mut compressed_path = temp_dir();
            uncompressed_path.push("frodo_best_waifu");
            compressed_path.push("frodo_bestest_waifu.7z");

            // Get the data into the uncompressed file, and launch 7z.
            File::create(&uncompressed_path)?.write_all(data)?;
            Command::new(zip_path).arg("a").arg("-m0=lzma").arg("-mx=3").arg(&compressed_path).arg(&uncompressed_path).output()?;

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
            fixed_data.encode_integer_i32(data.len() as i32);
            fixed_data.extend_from_slice(&[0x5D, 0x00, 0x00, 0x40, 0x00]);
            fixed_data.append(&mut compressed_data);

            Ok(fixed_data)
        }
        _ => Err(ErrorKind::ZipFolderNotFound.into())
    }
}
