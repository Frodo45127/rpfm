//---------------------------------------------------------------------------//
// Copyright (c) 2017-2019 Ismael Gutiérrez González. All rights reserved.
// 
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
// 
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

// Here should go all the functions related to the compresion/decompression of PackedFiles.

use xz2::read::{XzEncoder, XzDecoder};
use xz2::stream::{LzmaOptions, Stream};

use std::io::Read;
use std::u64;

use crate::error::{Error, ErrorKind, Result};
use crate::common::coding_helpers::encode_integer_i32;

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

            // Vanilla compressed files are LZMA Alone (or legacy) level 3 compressed files, reproducibles by compressing them
            // with default settings with 7-Zip. This should do the trick to get them decoded.
            let stream = Stream::new_lzma_decoder(u64::MAX).map_err(|_| Error::from(ErrorKind::PackedFileDataCouldNotBeDecompressed))?;
            let mut encoder = XzDecoder::new_stream(&*fixed_data, stream);
            let mut compress_data = vec![];
            match encoder.read_to_end(&mut compress_data) {
                Ok(_) => Ok(compress_data),
                Err(_) => Err(ErrorKind::PackedFileDataCouldNotBeDecompressed)?
            }
        }
        else { Err(ErrorKind::PackedFileDataCouldNotBeDecompressed)? }
    }
    else { Ok(vec![]) }
}

// This function compress the data of a PackedFile, returning the compressed data.
pub fn compress_data(data: &[u8]) -> Result<Vec<u8>> {
    let lzma_options = LzmaOptions::new_preset(3).unwrap();
    let stream = Stream::new_lzma_encoder(&lzma_options).unwrap();
    let mut encoder = XzEncoder::new_stream(data, stream);
    let mut compress_data = vec![];
    encoder.read_to_end(&mut compress_data).unwrap();

    // Now we have to fix the damn header and return it.
    compress_data.drain(..13);
    let mut fixed_data = encode_integer_i32(data.len() as i32);
    fixed_data.extend_from_slice(&[0x5D, 0x00, 0x00, 0x40, 0x00]);
    fixed_data.append(&mut compress_data);

    Ok(fixed_data)
}