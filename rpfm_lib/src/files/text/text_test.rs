//---------------------------------------------------------------------------//
// Copyright (c) 2017-2022 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Module containing tests for decoding/encoding `Text` files.

use std::io::{BufReader, BufWriter, Write};
use std::fs::File;

use crate::binary::ReadBytes;
use crate::files::*;

use super::Text;

#[test]
fn test_encode_iso_8859_1() {
    let path_1 = "../test_files/test_decode_iso_8859_1.html";
    let path_2 = "../test_files/test_encode_iso_8859_1.html";
    let mut reader = BufReader::new(File::open(path_1).unwrap());

    let mut decodeable_extra_data = DecodeableExtraData::default();
    decodeable_extra_data.file_name = Some("test_decode_iso_8859_1.html");

    let data_len = reader.len().unwrap();
    let before = reader.read_slice(data_len as usize, true).unwrap();
    let mut data = Text::decode(&mut reader, &Some(decodeable_extra_data)).unwrap();

    let mut after = vec![];
    data.encode(&mut after, &None).unwrap();

    let mut writer = BufWriter::new(File::create(path_2).unwrap());
    writer.write_all(&after).unwrap();

    assert_eq!(before, after);
}

#[test]
fn test_encode_utf8_bom() {
    let path_1 = "../test_files/test_decode_utf8_with_bom.html";
    let path_2 = "../test_files/test_encode_utf8_with_bom.html";
    let mut reader = BufReader::new(File::open(path_1).unwrap());

    let mut decodeable_extra_data = DecodeableExtraData::default();
    decodeable_extra_data.file_name = Some("test_decode_utf8_with_bom.html");

    let data_len = reader.len().unwrap();
    let before = reader.read_slice(data_len as usize, true).unwrap();
    let mut data = Text::decode(&mut reader, &Some(decodeable_extra_data)).unwrap();

    let mut after = vec![];
    data.encode(&mut after, &None).unwrap();

    let mut writer = BufWriter::new(File::create(path_2).unwrap());
    writer.write_all(&after).unwrap();

    assert_eq!(before, after);
}

#[test]
fn test_encode_utf8() {
    let path_1 = "../test_files/test_decode_utf8.html";
    let path_2 = "../test_files/test_encode_utf8.html";
    let mut reader = BufReader::new(File::open(path_1).unwrap());

    let mut decodeable_extra_data = DecodeableExtraData::default();
    decodeable_extra_data.file_name = Some("test_decode_utf8.html");

    let data_len = reader.len().unwrap();
    let before = reader.read_slice(data_len as usize, true).unwrap();
    let mut data = Text::decode(&mut reader, &Some(decodeable_extra_data)).unwrap();

    let mut after = vec![];
    data.encode(&mut after, &None).unwrap();

    let mut writer = BufWriter::new(File::create(path_2).unwrap());
    writer.write_all(&after).unwrap();

    assert_eq!(before, after);
}

#[test]
fn test_encode_utf16_le() {
    let path_1 = "../test_files/test_decode_utf16_le.html";
    let path_2 = "../test_files/test_encode_utf16_le.html";
    let mut reader = BufReader::new(File::open(path_1).unwrap());

    let mut decodeable_extra_data = DecodeableExtraData::default();
    decodeable_extra_data.file_name = Some("test_decode_utf16_le.html");

    let data_len = reader.len().unwrap();
    let before = reader.read_slice(data_len as usize, true).unwrap();
    let mut data = Text::decode(&mut reader, &Some(decodeable_extra_data)).unwrap();

    let mut after = vec![];
    data.encode(&mut after, &None).unwrap();

    let mut writer = BufWriter::new(File::create(path_2).unwrap());
    writer.write_all(&after).unwrap();

    assert_eq!(before, after);
}
