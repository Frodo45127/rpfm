//---------------------------------------------------------------------------//
// Copyright (c) 2017-2024 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Module containing tests for compression, so we don't break it.

use std::fs::File;
use std::io::{BufReader, BufWriter, Write};

use super::*;

#[test]
fn test_compression_lzma1() {
    let path_1 = "../test_files/test_compression_original.bmd";
    let path_2 = "../test_files/test_compression_compressed_lzma1.bmd";
    let path_3 = "../test_files/test_compression_decompressed_lzma1.bmd";
    let path_4 = "../test_files/test_compression_recompressed_lzma1.bmd";

    // This decompress and compress a file multiple times, and checks if the file is still the same.
    let mut reader = BufReader::new(File::open(path_1).unwrap());
    let mut before_nocomp = vec![];
    reader.read_to_end(&mut before_nocomp).unwrap();

    let mut writer_1 = BufWriter::new(File::create(path_2).unwrap());
    let before_comp = before_nocomp.compress(CompressionFormat::Lzma1).unwrap();
    writer_1.write_all(&before_comp).unwrap();

    let mut writer_2 = BufWriter::new(File::create(path_3).unwrap());
    let after_nocomp = before_comp.as_slice().decompress().unwrap();
    writer_2.write_all(&after_nocomp).unwrap();

    let mut writer_3 = BufWriter::new(File::create(path_4).unwrap());
    let after_comp = after_nocomp.compress(CompressionFormat::Lzma1).unwrap();
    writer_3.write_all(&after_comp).unwrap();

    assert_eq!(before_nocomp, after_nocomp);
    assert_eq!(before_comp, after_comp);
}

#[test]
fn test_compression_lz4() {
    let path_1 = "../test_files/test_compression_original.bmd";
    let path_2 = "../test_files/test_compression_compressed_lz4.bmd";
    let path_3 = "../test_files/test_compression_decompressed_lz4.bmd";
    let path_4 = "../test_files/test_compression_recompressed_lz4.bmd";

    // This decompress and compress a file multiple times, and checks if the file is still the same.
    let mut reader = BufReader::new(File::open(path_1).unwrap());
    let mut before_nocomp = vec![];
    reader.read_to_end(&mut before_nocomp).unwrap();

    let mut writer_1 = BufWriter::new(File::create(path_2).unwrap());
    let before_comp = before_nocomp.compress(CompressionFormat::Lz4).unwrap();
    writer_1.write_all(&before_comp).unwrap();

    let mut writer_2 = BufWriter::new(File::create(path_3).unwrap());
    let after_nocomp = before_comp.as_slice().decompress().unwrap();
    writer_2.write_all(&after_nocomp).unwrap();

    let mut writer_3 = BufWriter::new(File::create(path_4).unwrap());
    let after_comp = after_nocomp.compress(CompressionFormat::Lz4).unwrap();
    writer_3.write_all(&after_comp).unwrap();

    assert_eq!(before_nocomp, after_nocomp);
    assert_eq!(before_comp, after_comp);
}

#[test]
fn test_compression_zstd() {
    let path_1 = "../test_files/test_compression_original.bmd";
    let path_2 = "../test_files/test_compression_compressed_zstd.bmd";
    let path_3 = "../test_files/test_compression_decompressed_zstd.bmd";
    let path_4 = "../test_files/test_compression_recompressed_zstd.bmd";

    // This decompress and compress a file multiple times, and checks if the file is still the same.
    let mut reader = BufReader::new(File::open(path_1).unwrap());
    let mut before_nocomp = vec![];
    reader.read_to_end(&mut before_nocomp).unwrap();

    let mut writer_1 = BufWriter::new(File::create(path_2).unwrap());
    let before_comp = before_nocomp.compress(CompressionFormat::Zstd).unwrap();
    writer_1.write_all(&before_comp).unwrap();

    let mut writer_2 = BufWriter::new(File::create(path_3).unwrap());
    let after_nocomp = before_comp.as_slice().decompress().unwrap();
    writer_2.write_all(&after_nocomp).unwrap();

    let mut writer_3 = BufWriter::new(File::create(path_4).unwrap());
    let after_comp = after_nocomp.compress(CompressionFormat::Zstd).unwrap();
    writer_3.write_all(&after_comp).unwrap();

    assert_eq!(before_nocomp, after_nocomp);
    assert_eq!(before_comp, after_comp);
}
