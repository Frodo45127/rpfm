//---------------------------------------------------------------------------//
// Copyright (c) 2017-2024 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Module containing tests for decoding/encoding `Cs2Parsed` files.

use std::io::{BufReader, BufWriter, Write};
use std::fs::File;

use crate::binary::ReadBytes;
use crate::files::*;

use super::Cs2Parsed;

#[test]
fn test_encode_cs2_parsed() {
    let path_1 = "../test_files/test_wall.cs2.parsed";
    let path_2 = "../test_files/test_encode_wall.cs2.parsed";
    let mut reader = BufReader::new(File::open(path_1).unwrap());

    let decodeable_extra_data = DecodeableExtraData::default();

    let data_len = reader.len().unwrap();
    let before = reader.read_slice(data_len as usize, true).unwrap();
    let mut data = Cs2Parsed::decode(&mut reader, &Some(decodeable_extra_data)).unwrap();
    let mut after = vec![];

    data.encode(&mut after, &None).unwrap();

    let mut writer = BufWriter::new(File::create(path_2).unwrap());
    writer.write_all(&after).unwrap();

    assert_eq!(before, after);
}

#[test]
fn test_encode_cs2_corner_parsed() {
    let path_1 = "../test_files/test_wall_corner.cs2.parsed";
    let path_2 = "../test_files/test_encode_wall_corner.cs2.parsed";
    let mut reader = BufReader::new(File::open(path_1).unwrap());

    let decodeable_extra_data = DecodeableExtraData::default();

    let data_len = reader.len().unwrap();
    let before = reader.read_slice(data_len as usize, true).unwrap();
    let mut data = Cs2Parsed::decode(&mut reader, &Some(decodeable_extra_data)).unwrap();
    let mut after = vec![];

    data.encode(&mut after, &None).unwrap();

    let mut writer = BufWriter::new(File::create(path_2).unwrap());
    writer.write_all(&after).unwrap();

    assert_eq!(before, after);
}
