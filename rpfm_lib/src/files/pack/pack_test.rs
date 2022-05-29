//---------------------------------------------------------------------------//
// Copyright (c) 2017-2022 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

/*!
Module containing test for the `PackFile` module, just to make sure we don't break it... again...
!*/

use std::io::{BufReader, Write, BufWriter};
use std::fs::File;
use std::time::UNIX_EPOCH;

use crate::files::*;

use super::Pack;
/*
#[test]
fn test_decode_pfh6() {
    assert_eq!(PackFile::read(&PathBuf::from("../test_files/PFH6_test.pack"), false).is_ok(), true);
}*/


#[test]
fn test_decode_pfh5() {

    let file = File::open("../test_files/PFH5_test.pack").unwrap();
    let mut decodeable_extra_data = DecodeableExtraData::default();
    decodeable_extra_data.disk_file_path = Some("../test_files/PFH5_test.pack");
    decodeable_extra_data.disk_file_offset = Some(0);
    decodeable_extra_data.timestamp = Some(file.metadata().unwrap().modified().unwrap().duration_since(UNIX_EPOCH).unwrap().as_secs());
    decodeable_extra_data.lazy_load = Some(false);
    decodeable_extra_data.test_mode = true;
    let mut reader = BufReader::new(file);
    let pack = Pack::decode(&mut reader, Some(decodeable_extra_data));
    assert!(pack.is_ok());
}
/*
#[test]
fn test_decode_pfh4() {
	assert_eq!(PackFile::read(&PathBuf::from("../test_files/PFH4_test.pack"), false).is_ok(), true);
}

#[test]
fn test_decode_pfh3() {
	assert_eq!(PackFile::read(&PathBuf::from("../test_files/PFH3_test.pack"), false).is_ok(), true);
}

#[test]
fn test_decode_pfh2() {
	assert_eq!(PackFile::read(&PathBuf::from("../test_files/PFH2_test.pack"), false).is_ok(), true);
}

#[test]
fn test_decode_pfh0() {
    assert_eq!(PackFile::read(&PathBuf::from("../test_files/PFH0_test.pack"), false).is_ok(), true);
}

#[test]
fn test_encode_pfh6() {

    // Both PackFiles are not *exactly* the same. We have to reset their timestamp and give them the same path.
    let mut pack_file_base = PackFile::read(&PathBuf::from("../test_files/PFH6_test.pack"), false).unwrap();
    pack_file_base.set_file_path(&PathBuf::from("../test_files/PFH6_test_encode.pack")).unwrap();
    let mut pack_file_new = pack_file_base.clone();
    pack_file_new.save(Some(PathBuf::from("../test_files/PFH6_test_encode.pack"))).unwrap();

    let mut pack_file_new = PackFile::read(&PathBuf::from("../test_files/PFH6_test_encode.pack"), false).unwrap();
    pack_file_base.set_timestamp(0);
    pack_file_new.set_timestamp(0);

    assert_eq!(pack_file_base, pack_file_new);
}
*/
#[test]
fn test_encode_pfh5() {

    let file = File::open("../test_files/PFH5_test.pack").unwrap();
    let mut decodeable_extra_data = DecodeableExtraData::default();
    decodeable_extra_data.disk_file_path = Some("../test_files/PFH5_test.pack");
    decodeable_extra_data.disk_file_offset = Some(0);
    decodeable_extra_data.timestamp = Some(file.metadata().unwrap().modified().unwrap().duration_since(UNIX_EPOCH).unwrap().as_secs());
    decodeable_extra_data.lazy_load = Some(false);
    decodeable_extra_data.test_mode = true;
    let mut reader = BufReader::new(file);
    let mut pack_1 = Pack::decode(&mut reader, Some(decodeable_extra_data.clone())).unwrap();

    {
        let mut file = BufWriter::new(File::create("../test_files/PFH5_test_encode.pack").unwrap());
        pack_1.encode(&mut file, Some(decodeable_extra_data)).unwrap();
    }

    let mut data_pack_1 = vec![];
    let mut data_pack_2 = vec![];
    let mut pack_1 = BufReader::new(File::open("../test_files/PFH5_test.pack").unwrap());
    let mut pack_2 = BufReader::new(File::open("../test_files/PFH5_test_encode.pack").unwrap());

    pack_1.read_to_end(&mut data_pack_1).unwrap();
    pack_2.read_to_end(&mut data_pack_2).unwrap();

    assert_eq!(data_pack_1, data_pack_2);
}
/*
#[test]
fn test_encode_pfh4() {

	// Both PackFiles are not *exactly* the same. We have to reset their timestamp and give them the same path.
	let mut pack_file_base = PackFile::read(&PathBuf::from("../test_files/PFH4_test.pack"), false).unwrap();
	pack_file_base.set_file_path(&PathBuf::from("../test_files/PFH4_test_encode.pack")).unwrap();
	let mut pack_file_new = pack_file_base.clone();
	pack_file_new.save(Some(PathBuf::from("../test_files/PFH4_test_encode.pack"))).unwrap();

	let mut pack_file_new = PackFile::read(&PathBuf::from("../test_files/PFH4_test_encode.pack"), false).unwrap();
	pack_file_base.set_timestamp(0);
	pack_file_new.set_timestamp(0);

	assert_eq!(pack_file_base, pack_file_new);
}

#[test]
fn test_encode_pfh3() {

	// Both PackFiles are not *exactly* the same. We have to reset their timestamp and give them the same path.
	let mut pack_file_base = PackFile::read(&PathBuf::from("../test_files/PFH3_test.pack"), false).unwrap();
	pack_file_base.set_file_path(&PathBuf::from("../test_files/PFH3_test_encode.pack")).unwrap();
	let mut pack_file_new = pack_file_base.clone();
	pack_file_new.save(Some(PathBuf::from("../test_files/PFH3_test_encode.pack"))).unwrap();

	let mut pack_file_new = PackFile::read(&PathBuf::from("../test_files/PFH3_test_encode.pack"), false).unwrap();
	pack_file_base.set_timestamp(0);
	pack_file_new.set_timestamp(0);

	assert_eq!(pack_file_base, pack_file_new);
}

#[test]
fn test_encode_pfh2() {

	// Both PackFiles are not *exactly* the same. We have to reset their timestamp and give them the same path.
	let mut pack_file_base = PackFile::read(&PathBuf::from("../test_files/PFH2_test.pack"), false).unwrap();
	pack_file_base.set_file_path(&PathBuf::from("../test_files/PFH2_test_encode.pack")).unwrap();
	let mut pack_file_new = pack_file_base.clone();
	pack_file_new.save(Some(PathBuf::from("../test_files/PFH2_test_encode.pack"))).unwrap();

	let mut pack_file_new = PackFile::read(&PathBuf::from("../test_files/PFH2_test_encode.pack"), false).unwrap();
	pack_file_base.set_timestamp(0);
	pack_file_new.set_timestamp(0);

	assert_eq!(pack_file_base, pack_file_new);
}


#[test]
fn test_encode_pfh0() {

	// Both PackFiles are not *exactly* the same. We have to reset their timestamp and give them the same path.
	let mut pack_file_base = PackFile::read(&PathBuf::from("../test_files/PFH0_test.pack"), false).unwrap();
	pack_file_base.set_file_path(&PathBuf::from("../test_files/PFH0_test_encode.pack")).unwrap();
	let mut pack_file_new = pack_file_base.clone();
	pack_file_new.save(Some(PathBuf::from("../test_files/PFH0_test_encode.pack"))).unwrap();

	let mut pack_file_new = PackFile::read(&PathBuf::from("../test_files/PFH0_test_encode.pack"), false).unwrap();
	pack_file_base.set_timestamp(0);
	pack_file_new.set_timestamp(0);

	assert_eq!(pack_file_base, pack_file_new);
}
*/
