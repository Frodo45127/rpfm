//---------------------------------------------------------------------------//
// Copyright (c) 2017-2023 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Module containing tests for decoding/encoding in CaVp8 format.

use std::io::{BufReader, BufWriter, Write};
use std::fs::File;

use crate::binary::ReadBytes;
use crate::files::*;

use super::Video;

#[test]
fn test_decode_ca_vp8_v1() {
    let path = "../test_files/ca_vp8_v1_decode.ca_vp8";
    let mut reader = BufReader::new(File::open(path).unwrap());

    let mut decodeable_extra_data = DecodeableExtraData::default();
    decodeable_extra_data.disk_file_path = Some(path);
    decodeable_extra_data.timestamp = last_modified_time_from_file(reader.get_ref()).unwrap();

    let _ = Video::decode(&mut reader, &Some(decodeable_extra_data)).unwrap();
}

#[test]
fn test_decode_ca_vp8_v1_short() {
    let path = "../test_files/ca_vp8_v1_short_decode.ca_vp8";
    let mut reader = BufReader::new(File::open(path).unwrap());

    let mut decodeable_extra_data = DecodeableExtraData::default();
    decodeable_extra_data.disk_file_path = Some(path);
    decodeable_extra_data.timestamp = last_modified_time_from_file(reader.get_ref()).unwrap();

    let _ = Video::decode(&mut reader, &Some(decodeable_extra_data)).unwrap();
}

#[test]
fn test_decode_ca_vp8_v0() {
    let path = "../test_files/ca_vp8_v0_decode.ca_vp8";
    let mut reader = BufReader::new(File::open(path).unwrap());

    let mut decodeable_extra_data = DecodeableExtraData::default();
    decodeable_extra_data.disk_file_path = Some(path);
    decodeable_extra_data.timestamp = last_modified_time_from_file(reader.get_ref()).unwrap();

    let _ = Video::decode(&mut reader, &Some(decodeable_extra_data)).unwrap();
}

#[test]
fn test_encode_ca_vp8_v1() {
    let path_1 = "../test_files/ca_vp8_v1_decode.ca_vp8";
    let path_2 = "../test_files/ca_vp8_v1_encode.ca_vp8";
    let mut reader = BufReader::new(File::open(path_1).unwrap());

    let mut decodeable_extra_data = DecodeableExtraData::default();
    decodeable_extra_data.disk_file_path = Some(path_1);
    decodeable_extra_data.timestamp = last_modified_time_from_file(reader.get_ref()).unwrap();

    let data_len = reader.len().unwrap();
    let before = reader.read_slice(data_len as usize, true).unwrap();
    let mut data = Video::decode(&mut reader, &Some(decodeable_extra_data)).unwrap();

    let mut after = vec![];
    data.encode(&mut after, &None).unwrap();

    let mut file_out = BufWriter::new(File::create(path_2).unwrap());
    file_out.write_all(&after).unwrap();

    assert_eq!(before, after);
}

#[test]
fn test_encode_ca_vp8_v1_short() {
    let path_1 = "../test_files/ca_vp8_v1_short_decode.ca_vp8";
    let path_2 = "../test_files/ca_vp8_v1_short_encode.ca_vp8";
    let mut reader = BufReader::new(File::open(path_1).unwrap());

    let mut decodeable_extra_data = DecodeableExtraData::default();
    decodeable_extra_data.disk_file_path = Some(path_1);
    decodeable_extra_data.timestamp = last_modified_time_from_file(reader.get_ref()).unwrap();

    let data_len = reader.len().unwrap();
    let before = reader.read_slice(data_len as usize, true).unwrap();
    let mut data = Video::decode(&mut reader, &Some(decodeable_extra_data)).unwrap();

    let mut after = vec![];
    data.encode(&mut after, &None).unwrap();

    let mut file_out = BufWriter::new(File::create(path_2).unwrap());
    file_out.write_all(&after).unwrap();

    assert_eq!(before, after);
}

#[test]
fn test_encode_ca_vp8_v0() {
    let path_1 = "../test_files/ca_vp8_v0_decode.ca_vp8";
    let path_2 = "../test_files/ca_vp8_v0_encode.ca_vp8";
    let mut reader = BufReader::new(File::open(path_1).unwrap());

    let mut decodeable_extra_data = DecodeableExtraData::default();
    decodeable_extra_data.disk_file_path = Some(path_1);
    decodeable_extra_data.timestamp = last_modified_time_from_file(reader.get_ref()).unwrap();

    let data_len = reader.len().unwrap();
    let before = reader.read_slice(data_len as usize, true).unwrap();
    let mut data = Video::decode(&mut reader, &Some(decodeable_extra_data)).unwrap();

    let mut after = vec![];
    data.encode(&mut after, &None).unwrap();

    let mut file_out = BufWriter::new(File::create(path_2).unwrap());
    file_out.write_all(&after).unwrap();

    assert_eq!(before, after);
}
