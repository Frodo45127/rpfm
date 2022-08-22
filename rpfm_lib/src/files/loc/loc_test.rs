//---------------------------------------------------------------------------//
// Copyright (c) 2017-2022 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Module containing tests for decoding/encoding `Loc` files.

use std::io::{BufReader, BufWriter, Write};
use std::fs::File;

use crate::binary::ReadBytes;
use crate::files::*;

use super::Loc;

#[test]
fn test_encode_loc_no_sqlite() {
    let path_1 = "../test_files/test_decode.loc";
    let path_2 = "../test_files/test_encode_no_sqlite.loc";
    let mut reader = BufReader::new(File::open(path_1).unwrap());

    let decodeable_extra_data = DecodeableExtraData::default();

    let data_len = reader.len().unwrap();
    let before = reader.read_slice(data_len as usize, true).unwrap();
    let mut data = Loc::decode(&mut reader, &Some(decodeable_extra_data)).unwrap();

    let mut after = vec![];
    data.encode(&mut after, &None).unwrap();

    let mut writer = BufWriter::new(File::create(path_2).unwrap());
    writer.write_all(&after).unwrap();

    assert_eq!(before, after);
}

#[test]
fn test_encode_loc_sqlite() {
    let pool = crate::integrations::sqlite::init_database().unwrap();

    let path_1 = "../test_files/test_decode.loc";
    let path_2 = "../test_files/test_encode_sqlite.loc";
    let mut reader = BufReader::new(File::open(path_1).unwrap());

    let mut decodeable_extra_data = DecodeableExtraData::default();
    decodeable_extra_data.pool = Some(&pool);

    let data_len = reader.len().unwrap();
    let before = reader.read_slice(data_len as usize, true).unwrap();
    let mut data = Loc::decode(&mut reader, &Some(decodeable_extra_data)).unwrap();

    let mut after = vec![];
    let mut encodeable_extra_data = EncodeableExtraData::default();
    encodeable_extra_data.pool = Some(&pool);
    data.encode(&mut after, &Some(encodeable_extra_data)).unwrap();

    let mut writer = BufWriter::new(File::create(path_2).unwrap());
    writer.write_all(&after).unwrap();

    assert_eq!(before, after);
}
