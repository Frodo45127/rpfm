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

use crate::error::{Error, ErrorKind, Result};
use crate::common::coding_helpers::encode_integer_i32;

/// This function decompress the data of a PackedFile, returning the decompressed data.
pub fn decompress_data(data: &[u8]) -> Result<Vec<u8>> {
    if data.len() >= 9 {
        let mut fixed_data: Vec<u8> = vec![];
        fixed_data.extend_from_slice(&data[4..8]);
        fixed_data.push(0);
        fixed_data.extend_from_slice(&data[0..4]);
        fixed_data.extend_from_slice(&vec![0; 4]);
        fixed_data.extend_from_slice(&data[9..]);
        lzma::decompress(&fixed_data).map_err(|_| Error::from(ErrorKind::PackedFileDataCouldNotBeDecompressed))
    }
    else { Err(ErrorKind::PackedFileDataCouldNotBeDecompressed)? }
}

// This function compress the data of a PackedFile, returning the compressed data.
pub fn compress_data(data: &[u8]) -> Result<Vec<u8>> {

    // Same as with the decompression. Once compressed, we need to change the header.
    let mut compress_data = lzma::compress(data, 6).unwrap();

    // Remove XZ Header and Footer. These numbers are magic....
    compress_data.drain(..31);
    compress_data.drain((compress_data.len() - 33)..);

    let mut fixed_data = encode_integer_i32(data.len() as i32);
    fixed_data.extend_from_slice(&vec![0x5D, 0x00, 0x00, 0x40, 0x00, 0x00]);
    fixed_data.append(&mut compress_data);
    Ok(fixed_data)
}