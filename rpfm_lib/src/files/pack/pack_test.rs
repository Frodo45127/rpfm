//---------------------------------------------------------------------------//
// Copyright (c) 2017-2022 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Module containing tests for decoding/encoding Packs in multiple formats.

use std::io::{BufReader, BufWriter};
use std::fs::File;

use crate::files::*;
use super::Pack;

#[test]
fn test_decode_pfh6() {
    let path = "../test_files/PFH6_test.pack";
    let mut reader = BufReader::new(File::open(path).unwrap());

    let mut decodeable_extra_data = DecodeableExtraData::default();
    decodeable_extra_data.disk_file_path = Some(path);
    decodeable_extra_data.timestamp = last_modified_time_from_file(reader.get_ref()).unwrap();

    let pack = Pack::decode(&mut reader, Some(decodeable_extra_data));
    assert!(pack.is_ok());
}

#[test]
fn test_decode_pfh5() {
    let path = "../test_files/PFH5_test.pack";
    let mut reader = BufReader::new(File::open(path).unwrap());

    let mut decodeable_extra_data = DecodeableExtraData::default();
    decodeable_extra_data.disk_file_path = Some(path);
    decodeable_extra_data.timestamp = last_modified_time_from_file(reader.get_ref()).unwrap();

    let pack = Pack::decode(&mut reader, Some(decodeable_extra_data));
    assert!(pack.is_ok());
}

#[test]
fn test_decode_pfh4() {
    let path = "../test_files/PFH4_test.pack";
    let mut reader = BufReader::new(File::open(path).unwrap());

    let mut decodeable_extra_data = DecodeableExtraData::default();
    decodeable_extra_data.disk_file_path = Some(path);
    decodeable_extra_data.timestamp = last_modified_time_from_file(reader.get_ref()).unwrap();

    let pack = Pack::decode(&mut reader, Some(decodeable_extra_data));
    assert!(pack.is_ok());
}

#[test]
fn test_decode_pfh3() {
    let path = "../test_files/PFH3_test.pack";
    let mut reader = BufReader::new(File::open(path).unwrap());

    let mut decodeable_extra_data = DecodeableExtraData::default();
    decodeable_extra_data.disk_file_path = Some(path);
    decodeable_extra_data.timestamp = last_modified_time_from_file(reader.get_ref()).unwrap();

    let pack = Pack::decode(&mut reader, Some(decodeable_extra_data));
    assert!(pack.is_ok());
}

#[test]
fn test_decode_pfh2() {
    let path = "../test_files/PFH2_test.pack";
    let mut reader = BufReader::new(File::open(path).unwrap());

    let mut decodeable_extra_data = DecodeableExtraData::default();
    decodeable_extra_data.disk_file_path = Some(path);
    decodeable_extra_data.timestamp = last_modified_time_from_file(reader.get_ref()).unwrap();

    let pack = Pack::decode(&mut reader, Some(decodeable_extra_data));
    assert!(pack.is_ok());
}

#[test]
fn test_decode_pfh0() {
    let path = "../test_files/PFH0_test.pack";
    let mut reader = BufReader::new(File::open(path).unwrap());

    let mut decodeable_extra_data = DecodeableExtraData::default();
    decodeable_extra_data.disk_file_path = Some(path);
    decodeable_extra_data.timestamp = last_modified_time_from_file(reader.get_ref()).unwrap();

    let pack = Pack::decode(&mut reader, Some(decodeable_extra_data));
    assert!(pack.is_ok());
}

#[test]
fn test_encode_pfh6() {
    let path_1 = "../test_files/PFH6_test.pack";
    let path_2 = "../test_files/PFH6_test_encode.pack";
    let mut reader = BufReader::new(File::open(path_1).unwrap());

    let mut decodeable_extra_data = DecodeableExtraData::default();
    decodeable_extra_data.disk_file_path = Some(path_1);
    decodeable_extra_data.timestamp = last_modified_time_from_file(reader.get_ref()).unwrap();

    let mut pack = Pack::decode(&mut reader, Some(decodeable_extra_data)).unwrap();
    let mut file = BufWriter::new(File::create(path_2).unwrap());

    let mut encodeable_extra_data = EncodeableExtraData::default();
    encodeable_extra_data.test_mode = true;
    pack.encode(&mut file, Some(encodeable_extra_data)).unwrap();

    let mut data_pack_1 = vec![];
    let mut data_pack_2 = vec![];
    let mut pack_1 = BufReader::new(File::open(path_1).unwrap());
    let mut pack_2 = BufReader::new(File::open(path_2).unwrap());

    pack_1.read_to_end(&mut data_pack_1).unwrap();
    pack_2.read_to_end(&mut data_pack_2).unwrap();

    assert_eq!(data_pack_1, data_pack_2);
}

#[test]
fn test_encode_pfh5() {
    let path_1 = "../test_files/PFH5_test.pack";
    let path_2 = "../test_files/PFH5_test_encode.pack";
    let mut reader = BufReader::new(File::open(path_1).unwrap());

    let mut decodeable_extra_data = DecodeableExtraData::default();
    decodeable_extra_data.disk_file_path = Some(path_1);
    decodeable_extra_data.timestamp = last_modified_time_from_file(reader.get_ref()).unwrap();

    let mut pack = Pack::decode(&mut reader, Some(decodeable_extra_data)).unwrap();
    let mut file = BufWriter::new(File::create(path_2).unwrap());

    let mut encodeable_extra_data = EncodeableExtraData::default();
    encodeable_extra_data.test_mode = true;
    pack.encode(&mut file, Some(encodeable_extra_data)).unwrap();

    let mut data_pack_1 = vec![];
    let mut data_pack_2 = vec![];
    let mut pack_1 = BufReader::new(File::open(path_1).unwrap());
    let mut pack_2 = BufReader::new(File::open(path_2).unwrap());

    pack_1.read_to_end(&mut data_pack_1).unwrap();
    pack_2.read_to_end(&mut data_pack_2).unwrap();

    assert_eq!(data_pack_1, data_pack_2);
}

#[test]
fn test_encode_pfh4() {
    let path_1 = "../test_files/PFH4_test.pack";
    let path_2 = "../test_files/PFH4_test_encode.pack";
    let mut reader = BufReader::new(File::open(path_1).unwrap());

    let mut decodeable_extra_data = DecodeableExtraData::default();
    decodeable_extra_data.disk_file_path = Some(path_1);
    decodeable_extra_data.timestamp = last_modified_time_from_file(reader.get_ref()).unwrap();

    let mut pack = Pack::decode(&mut reader, Some(decodeable_extra_data)).unwrap();
    let mut file = BufWriter::new(File::create(path_2).unwrap());

    let mut encodeable_extra_data = EncodeableExtraData::default();
    encodeable_extra_data.test_mode = true;
    pack.encode(&mut file, Some(encodeable_extra_data)).unwrap();

    let mut data_pack_1 = vec![];
    let mut data_pack_2 = vec![];
    let mut pack_1 = BufReader::new(File::open(path_1).unwrap());
    let mut pack_2 = BufReader::new(File::open(path_2).unwrap());

    pack_1.read_to_end(&mut data_pack_1).unwrap();
    pack_2.read_to_end(&mut data_pack_2).unwrap();

    assert_eq!(data_pack_1, data_pack_2);
}

#[test]
fn test_encode_pfh3() {
    let path_1 = "../test_files/PFH3_test.pack";
    let path_2 = "../test_files/PFH3_test_encode.pack";
    let mut reader = BufReader::new(File::open(path_1).unwrap());

    let mut decodeable_extra_data = DecodeableExtraData::default();
    decodeable_extra_data.disk_file_path = Some(path_1);
    decodeable_extra_data.timestamp = last_modified_time_from_file(reader.get_ref()).unwrap();

    let mut pack = Pack::decode(&mut reader, Some(decodeable_extra_data)).unwrap();
    let mut file = BufWriter::new(File::create(path_2).unwrap());

    let mut encodeable_extra_data = EncodeableExtraData::default();
    encodeable_extra_data.test_mode = true;
    pack.encode(&mut file, Some(encodeable_extra_data)).unwrap();

    let mut data_pack_1 = vec![];
    let mut data_pack_2 = vec![];
    let mut pack_1 = BufReader::new(File::open(path_1).unwrap());
    let mut pack_2 = BufReader::new(File::open(path_2).unwrap());

    pack_1.read_to_end(&mut data_pack_1).unwrap();
    pack_2.read_to_end(&mut data_pack_2).unwrap();

    assert_eq!(data_pack_1, data_pack_2);
}

#[test]
fn test_encode_pfh2() {

    let path_1 = "../test_files/PFH2_test.pack";
    let path_2 = "../test_files/PFH2_test_encode.pack";
    let mut reader = BufReader::new(File::open(path_1).unwrap());

    let mut decodeable_extra_data = DecodeableExtraData::default();
    decodeable_extra_data.disk_file_path = Some(path_1);
    decodeable_extra_data.timestamp = last_modified_time_from_file(reader.get_ref()).unwrap();

    let mut pack = Pack::decode(&mut reader, Some(decodeable_extra_data)).unwrap();
    let mut file = BufWriter::new(File::create(path_2).unwrap());

    let mut encodeable_extra_data = EncodeableExtraData::default();
    encodeable_extra_data.test_mode = true;
    pack.encode(&mut file, Some(encodeable_extra_data)).unwrap();

    let mut data_pack_1 = vec![];
    let mut data_pack_2 = vec![];
    let mut pack_1 = BufReader::new(File::open(path_1).unwrap());
    let mut pack_2 = BufReader::new(File::open(path_2).unwrap());

    pack_1.read_to_end(&mut data_pack_1).unwrap();
    pack_2.read_to_end(&mut data_pack_2).unwrap();

    assert_eq!(data_pack_1, data_pack_2);
}

#[test]
fn test_encode_pfh0() {

    let path_1 = "../test_files/PFH0_test.pack";
    let path_2 = "../test_files/PFH0_test_encode.pack";
    let mut reader = BufReader::new(File::open(path_1).unwrap());

    let mut decodeable_extra_data = DecodeableExtraData::default();
    decodeable_extra_data.disk_file_path = Some(path_1);
    decodeable_extra_data.timestamp = last_modified_time_from_file(reader.get_ref()).unwrap();

    let mut pack = Pack::decode(&mut reader, Some(decodeable_extra_data)).unwrap();
    let mut file = BufWriter::new(File::create(path_2).unwrap());

    let mut encodeable_extra_data = EncodeableExtraData::default();
    encodeable_extra_data.test_mode = true;
    pack.encode(&mut file, Some(encodeable_extra_data)).unwrap();

    let mut data_pack_1 = vec![];
    let mut data_pack_2 = vec![];
    let mut pack_1 = BufReader::new(File::open(path_1).unwrap());
    let mut pack_2 = BufReader::new(File::open(path_2).unwrap());

    pack_1.read_to_end(&mut data_pack_1).unwrap();
    pack_2.read_to_end(&mut data_pack_2).unwrap();

    assert_eq!(data_pack_1, data_pack_2);
}

