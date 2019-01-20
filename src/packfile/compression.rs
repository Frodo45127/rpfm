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

/// This function decompress the data of a PackedFile, returning the decompressed data.
pub fn decompress_data(data: &[u8]) -> Result<Vec<u8>> {
    let mut data = data.to_vec();

    if data.len() >= 9 {
        let mut fixed_data: Vec<u8> = vec![];
        fixed_data.append(&mut data[4..8].to_vec());
        fixed_data.push(0);
        fixed_data.append(&mut data[0..4].to_vec());
        fixed_data.append(&mut vec![0; 4]);
        fixed_data.append(&mut data[9..].to_vec());
        lzma::decompress(&fixed_data).map_err(|_| Error::from(ErrorKind::PackedFileDataCouldNotBeDecompressed))
    }
    else { Err(ErrorKind::PackedFileDataCouldNotBeDecompressed)? }
}
