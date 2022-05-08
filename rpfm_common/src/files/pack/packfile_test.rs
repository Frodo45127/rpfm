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

use std::path::PathBuf;

use super::Pack;
/*
#[test]
fn test_decode_pfh6() {
    assert_eq!(PackFile::read(&PathBuf::from("../test_files/PFH6_test.pack"), false).is_ok(), true);
}

#[test]
fn test_decode_pfh5() {
    assert_eq!(PackFile::read(&PathBuf::from("../test_files/PFH5_test.pack"), false).is_ok(), true);
}

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

#[test]
fn test_encode_pfh5() {

	// Both PackFiles are not *exactly* the same. We have to reset their timestamp and give them the same path.
	let mut pack_file_base = PackFile::read(&PathBuf::from("../test_files/PFH5_test.pack"), false).unwrap();
	pack_file_base.set_file_path(&PathBuf::from("../test_files/PFH5_test_encode.pack")).unwrap();
	let mut pack_file_new = pack_file_base.clone();
	pack_file_new.save(Some(PathBuf::from("../test_files/PFH5_test_encode.pack"))).unwrap();

	let mut pack_file_new = PackFile::read(&PathBuf::from("../test_files/PFH5_test_encode.pack"), false).unwrap();
	pack_file_base.set_timestamp(0);
	pack_file_new.set_timestamp(0);

	assert_eq!(pack_file_base, pack_file_new);
}

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
