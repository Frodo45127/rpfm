//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Module containing tests for decoding/encoding `DB` files.

use std::fs::File;
use std::io::{BufReader, BufWriter, Write};

use crate::binary::ReadBytes;
use crate::files::*;

use super::DB;

#[test]
fn test_encode_db_no_sqlite() {
    let path_1 = "../test_files/test_decode_db";
    let path_2 = "../test_files/test_encode_db_no_sqlite";
    let mut reader = BufReader::new(File::open(path_1).unwrap());

    let mut schema = Schema::default();
    schema.add_definition("test_decode_db", &DB::test_definition());

    let mut decodeable_extra_data = DecodeableExtraData::default();
    decodeable_extra_data.file_name = Some("test_decode_db");
    decodeable_extra_data.table_name = Some("test_decode_db");
    decodeable_extra_data.schema = Some(&schema);

    let data_len = reader.len().unwrap();
    let before = reader.read_slice(data_len as usize, true).unwrap();
    let mut data = DB::decode(&mut reader, &Some(decodeable_extra_data)).unwrap();

    let mut after = vec![];
    data.encode(&mut after, &None).unwrap();

    let mut writer = BufWriter::new(File::create(path_2).unwrap());
    writer.write_all(&after).unwrap();

    assert_eq!(before, after);
}
