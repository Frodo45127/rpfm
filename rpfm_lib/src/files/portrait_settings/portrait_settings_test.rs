//---------------------------------------------------------------------------//
// Copyright (c) 2017-2024 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Module containing tests for decoding/encoding `PortraitSettings` files.

use std::io::{BufReader, BufWriter, Write};
use std::fs::File;

use crate::binary::ReadBytes;
use crate::files::*;

use super::PortraitSettings;
/*
#[test]
fn test_encode_portrait_settings_v1() {
    let path_1 = "../test_files/test_decode_portrait_settings_v1.bin";
    let path_2 = "../test_files/test_encode_portrait_settings_v1.bin";
    let mut reader = BufReader::new(File::open(path_1).unwrap());

    let data_len = reader.len().unwrap();
    let before = reader.read_slice(data_len as usize, true).unwrap();
    let mut data = PortraitSettings::decode(&mut reader, &None).unwrap();

    let mut after = vec![];
    data.encode(&mut after, &None).unwrap();

    let mut writer = BufWriter::new(File::create(path_2).unwrap());
    writer.write_all(&after).unwrap();

    assert_eq!(before, after);
}*/

#[test]
fn test_encode_portrait_settings_v4() {
    let path_1 = "../test_files/test_decode_portrait_settings_v4.bin";
    let path_2 = "../test_files/test_encode_portrait_settings_v4.bin";
    let mut reader = BufReader::new(File::open(path_1).unwrap());

    let data_len = reader.len().unwrap();
    let before = reader.read_slice(data_len as usize, true).unwrap();
    let mut data = PortraitSettings::decode(&mut reader, &None).unwrap();

    let mut after = vec![];
    data.encode(&mut after, &None).unwrap();

    let mut writer = BufWriter::new(File::create(path_2).unwrap());
    writer.write_all(&after).unwrap();

    assert_eq!(before, after);
}
