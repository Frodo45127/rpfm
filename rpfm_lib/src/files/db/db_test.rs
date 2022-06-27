//---------------------------------------------------------------------------//
// Copyright (c) 2017-2022 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Module containing tests for decoding/encoding `DB` files.

use crate::files::table::TableData;
use std::io::{BufReader, BufWriter, Write};
use std::fs::File;

use crate::binary::ReadBytes;
use crate::files::*;
use crate::files::table::Table;

use super::DB;
/*
#[test]
fn test_generate_table() {
    let definition = DB::test_definition();
    let mut table: DB = From::from(Table::new(&definition, "test_decode_db", false));
    let row_1 = table.new_row(None, None);
    let row_2 = table.new_row(None, None);
    let row_3 = table.new_row(None, None);

    let table_data = TableData::Local(vec![row_1, row_2, row_3]);
    table.table.set_table_data(table_data);

    let mut after = vec![];
    table.encode(&mut after, None).unwrap();
    let mut writer = BufWriter::new(File::create("../test_files/test_decode_db").unwrap());
    writer.write_all(&after).unwrap();
    panic!();
}*/


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
    let mut data = DB::decode(&mut reader, Some(decodeable_extra_data)).unwrap();

    let mut after = vec![];
    data.encode(&mut after, None).unwrap();

    let mut writer = BufWriter::new(File::create(path_2).unwrap());
    writer.write_all(&after).unwrap();

    assert_eq!(before, after);
}

#[test]
fn test_encode_db_sqlite() {
    let pool = crate::integrations::sqlite::init_database().unwrap();

    let path_1 = "../test_files/test_decode_db";
    let path_2 = "../test_files/test_encode_db_sqlite";
    let mut reader = BufReader::new(File::open(path_1).unwrap());

    let mut schema = Schema::default();
    schema.add_definition("test_decode_db", &DB::test_definition());

    let mut decodeable_extra_data = DecodeableExtraData::default();
    decodeable_extra_data.file_name = Some("test_decode_db");
    decodeable_extra_data.table_name = Some("test_decode_db");
    decodeable_extra_data.pool = Some(&pool);
    decodeable_extra_data.schema = Some(&schema);

    let data_len = reader.len().unwrap();
    let before = reader.read_slice(data_len as usize, true).unwrap();
    let mut data = DB::decode(&mut reader, Some(decodeable_extra_data.clone())).unwrap();

    let mut after = vec![];
    data.encode(&mut after, Some(decodeable_extra_data)).unwrap();

    let mut writer = BufWriter::new(File::create(path_2).unwrap());
    writer.write_all(&after).unwrap();

    assert_eq!(before, after);
}

