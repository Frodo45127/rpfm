//---------------------------------------------------------------------------//
// Copyright (c) 2017-2023 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Module containing tests for decoding/encoding `MatchedCombat` files.

use std::io::{BufReader, BufWriter, Write};
use std::fs::File;

use crate::binary::ReadBytes;
use crate::files::*;

use super::MatchedCombat;

#[test]
fn test_encode_matched_combat_v1_trigger_3k() {
    let path_1 = "../test_files/test_decode_matched_combat_3k_trigger.bin";
    let path_2 = "../test_files/test_encode_matched_combat_3k_trigger.bin";
    let mut reader = BufReader::new(File::open(path_1).unwrap());

    let mut extra_data = DecodeableExtraData::default();
    extra_data.game_key = Some("three_kingdoms");

    let data_len = reader.len().unwrap();
    let before = reader.read_slice(data_len as usize, true).unwrap();
    let mut data = MatchedCombat::decode(&mut reader, &Some(extra_data)).unwrap();

    let mut extra_data = EncodeableExtraData::default();
    extra_data.game_key = Some("three_kingdoms");

    let mut after = vec![];
    data.encode(&mut after, &Some(extra_data)).unwrap();

    let mut writer = BufWriter::new(File::create(path_2).unwrap());
    writer.write_all(&after).unwrap();

    assert_eq!(before, after);
}

#[test]
fn test_encode_matched_combat_v1_matched_wh3() {
    let path_1 = "../test_files/test_decode_matched_combat_wh3.bin";
    let path_2 = "../test_files/test_encode_matched_combat_wh3.bin";
    let mut reader = BufReader::new(File::open(path_1).unwrap());

    let mut extra_data = DecodeableExtraData::default();
    extra_data.game_key = Some("warhammer_3");

    let data_len = reader.len().unwrap();
    let before = reader.read_slice(data_len as usize, true).unwrap();
    let mut data = MatchedCombat::decode(&mut reader, &Some(extra_data)).unwrap();

    let mut extra_data = EncodeableExtraData::default();
    extra_data.game_key = Some("warhammer_3");

    let mut after = vec![];
    data.encode(&mut after, &Some(extra_data)).unwrap();

    let mut writer = BufWriter::new(File::create(path_2).unwrap());
    writer.write_all(&after).unwrap();

    assert_eq!(before, after);
}

#[test]
fn test_encode_matched_combat_v3_matched_3k() {
    let path_1 = "../test_files/test_decode_matched_combat_3k_matched.bin";
    let path_2 = "../test_files/test_encode_matched_combat_3k_matched.bin";
    let mut reader = BufReader::new(File::open(path_1).unwrap());

    let mut extra_data = DecodeableExtraData::default();
    extra_data.game_key = Some("three_kingdoms");

    let data_len = reader.len().unwrap();
    let before = reader.read_slice(data_len as usize, true).unwrap();
    let mut data = MatchedCombat::decode(&mut reader, &Some(extra_data)).unwrap();

    let mut extra_data = EncodeableExtraData::default();
    extra_data.game_key = Some("three_kingdoms");

    let mut after = vec![];
    data.encode(&mut after, &Some(extra_data)).unwrap();

    let mut writer = BufWriter::new(File::create(path_2).unwrap());
    writer.write_all(&after).unwrap();

    assert_eq!(before, after);
}
