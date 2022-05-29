//---------------------------------------------------------------------------//
// Copyright (c) 2017-2022 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Module containing tests for decoding/encoding in Ivf format.

use std::io::{BufReader, BufWriter, Write};
use std::fs::File;

use crate::binary::ReadBytes;
use crate::files::*;

use super::*;

#[test]
fn test_decode_ca_vp8_v1_to_ivf_and_back() {

    let file = File::open("../test_files/ca_vp8_v1_decode.ca_vp8").unwrap();
    let mut decodeable_extra_data = DecodeableExtraData::default();
    decodeable_extra_data.disk_file_path = Some("../test_files/ca_vp8_v1_decode.ca_vp8");
    decodeable_extra_data.disk_file_offset = Some(0);
    decodeable_extra_data.timestamp = Some(last_modified_time_from_file(&file).unwrap());
    let mut reader = BufReader::new(file);
    let data_len = reader.len().unwrap();
    let before = reader.read_slice(data_len as usize, true).unwrap();
    let mut data = CaVp8::decode(&mut reader, Some(decodeable_extra_data)).unwrap();

    data.format = SupportedFormats::Ivf;
    let mut ivf = vec![];
    data.encode(&mut ivf, None).unwrap();

    let mut second_data = CaVp8::decode(&mut Cursor::new(ivf), None).unwrap();
    second_data.format = SupportedFormats::CaVp8;

    let mut after = vec![];
    second_data.encode(&mut after, None).unwrap();

    let mut file_out = BufWriter::new(File::create("../test_files/ca_vp8_v1_to_ivf_and_back.ca_vp8").unwrap());
    file_out.write_all(&after).unwrap();

    assert_eq!(before, after);
}

#[test]
fn test_decode_ca_vp8_v0_to_ivf_and_back() {

    let file = File::open("../test_files/ca_vp8_v0_decode.ca_vp8").unwrap();
    let mut decodeable_extra_data = DecodeableExtraData::default();
    decodeable_extra_data.disk_file_path = Some("../test_files/ca_vp8_v0_decode.ca_vp8");
    decodeable_extra_data.disk_file_offset = Some(0);
    decodeable_extra_data.timestamp = Some(last_modified_time_from_file(&file).unwrap());
    let mut reader = BufReader::new(file);
    let data_len = reader.len().unwrap();
    let before = reader.read_slice(data_len as usize, true).unwrap();
    let mut data = CaVp8::decode(&mut reader, Some(decodeable_extra_data)).unwrap();

    data.format = SupportedFormats::Ivf;
    let mut ivf = vec![];
    data.encode(&mut ivf, None).unwrap();

    let mut second_data = CaVp8::decode(&mut Cursor::new(ivf), None).unwrap();
    second_data.format = SupportedFormats::CaVp8;

    let mut after = vec![];
    second_data.encode(&mut after, None).unwrap();

    let mut file_out = BufWriter::new(File::create("../test_files/ca_vp8_v0_to_ivf_and_back.ca_vp8").unwrap());
    file_out.write_all(&after).unwrap();

    assert_eq!(before, after);
}
