// Here should go all the functions related to the compresion/decompression of PackedFiles.

use crate::error::{Error, ErrorKind, Result};
/// This function decompress the data of a PackedFile, returning the decompressed data.
pub fn decompress_data(mut data: Vec<u8>) -> Result<Vec<u8>> {

    // CA seems to use a custom implementation of LZMA with a different header than the 7zip/XZ utils implementation.
    // That means we have to fix the header first. If we don't have at least 9 bytes, it's an invalid file.
    if data.len() >= 9 {
        let mut fixed_data: Vec<u8> = vec![];
        fixed_data.append(&mut data[4..8].to_vec());
        fixed_data.push(0);
        fixed_data.append(&mut data[0..4].to_vec());
        fixed_data.append(&mut vec![0; 4]);
        fixed_data.append(&mut data[9..].to_vec());
        lzma::decompress(&mut fixed_data).map_err(|_| Error::from(ErrorKind::PackedFileDataCouldNotBeDecompressed))
    }
    else { Err(ErrorKind::PackedFileDataCouldNotBeDecompressed)? }
}
