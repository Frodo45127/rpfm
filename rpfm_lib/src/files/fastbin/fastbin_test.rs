//---------------------------------------------------------------------------//
// Copyright (c) 2017-2023 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Module containing tests for decoding/encoding `FastBin` files.

use std::io::{BufReader, BufWriter, Write};
use std::fs::File;

use crate::binary::ReadBytes;
use crate::files::*;

use super::FastBin;

#[test]
fn test_encode_fastbin_prefab() {
    let path_1 = "../test_files/test_decode.bmd";
    let path_2 = "../test_files/test_encode.bmd";
    let mut reader = BufReader::new(File::open(path_1).unwrap());

    let decodeable_extra_data = DecodeableExtraData::default();

    let data_len = reader.len().unwrap();
    let before = reader.read_slice(data_len as usize, true).unwrap();
    let mut data = FastBin::decode(&mut reader, &Some(decodeable_extra_data)).unwrap();

    let mut after = vec![];
    data.encode(&mut after, &None).unwrap();

    let mut writer = BufWriter::new(File::create(path_2).unwrap());
    writer.write_all(&after).unwrap();

    assert_eq!(before, after);
}

#[test]
fn test_encode_fastbin_map_data() {
    let path_1 = "../test_files/fastbin/bmd_data.bin";
    let path_2 = "../test_files/fastbin/encode_bmd_data.bin";
    let mut reader = BufReader::new(File::open(path_1).unwrap());

    let decodeable_extra_data = DecodeableExtraData::default();

    let data_len = reader.len().unwrap();
    let before = reader.read_slice(data_len as usize, true).unwrap();
    let mut data = FastBin::decode(&mut reader, &Some(decodeable_extra_data)).unwrap();

    let mut after = vec![];
    data.encode(&mut after, &None).unwrap();

    let mut writer = BufWriter::new(File::create(path_2).unwrap());
    writer.write_all(&after).unwrap();

    assert_eq!(before, after);
}

#[test]
fn test_encode_fastbin_map_nogo_data() {
    let path_1 = "../test_files/fastbin/bmd_nogo_data.bin";
    let path_2 = "../test_files/fastbin/encode_bmd_nogo_data.bin";
    let mut reader = BufReader::new(File::open(path_1).unwrap());

    let decodeable_extra_data = DecodeableExtraData::default();

    let data_len = reader.len().unwrap();
    let before = reader.read_slice(data_len as usize, true).unwrap();
    let mut data = FastBin::decode(&mut reader, &Some(decodeable_extra_data)).unwrap();

    let mut after = vec![];
    data.encode(&mut after, &None).unwrap();

    let mut writer = BufWriter::new(File::create(path_2).unwrap());
    writer.write_all(&after).unwrap();

    assert_eq!(before, after);
}

#[test]
fn test_encode_fastbin_map_catchment() {
    let path_1 = "../test_files/fastbin/catchment__bmd.bin";
    let path_2 = "../test_files/fastbin/encode_catchment__bmd.bin";
    let mut reader = BufReader::new(File::open(path_1).unwrap());

    let decodeable_extra_data = DecodeableExtraData::default();

    let data_len = reader.len().unwrap();
    let before = reader.read_slice(data_len as usize, true).unwrap();
    let mut data = FastBin::decode(&mut reader, &Some(decodeable_extra_data)).unwrap();

    let mut after = vec![];
    data.encode(&mut after, &None).unwrap();

    let mut writer = BufWriter::new(File::create(path_2).unwrap());
    writer.write_all(&after).unwrap();

    assert_eq!(before, after);
}
