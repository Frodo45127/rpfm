//---------------------------------------------------------------------------//
// Copyright (c) 2017-2023 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Module containing tests for decoding/encoding `GroupFormations` files.

use std::io::{BufReader, BufWriter, Write};
use std::fs::File;

use crate::binary::ReadBytes;
use crate::files::*;
use crate::games::supported_games::*;

use super::GroupFormations;

/*
#[test]
fn test_encode_group_formation_three_kingdoms() {

    let path_1 = "../test_files/test_decode_group_formations_3k.bin";
    let path_2 = "../test_files/test_encode_group_formations_3k.bin";
    let mut reader = BufReader::new(File::open(path_1).unwrap());

    let mut extra_data = DecodeableExtraData::default();
    extra_data.game_key = Some(KEY_THREE_KINGDOMS);

    let data_len = reader.len().unwrap();
    let before = reader.read_slice(data_len as usize, true).unwrap();
    let mut data = GroupFormations::decode(&mut reader, &Some(extra_data)).unwrap();

    let mut extra_data = EncodeableExtraData::default();
    extra_data.game_key = Some(KEY_THREE_KINGDOMS);

    let mut after = vec![];
    data.encode(&mut after, &Some(extra_data)).unwrap();

    let mut writer = BufWriter::new(File::create(path_2).unwrap());
    writer.write_all(&after).unwrap();

    assert_eq!(before, after);
}*/

#[test]
fn test_encode_group_formation_rome_2() {

    let path_1 = "../test_files/test_decode_group_formations_rom2.bin";
    let path_2 = "../test_files/test_encode_group_formations_rom2.bin";
    let mut reader = BufReader::new(File::open(path_1).unwrap());

    let mut extra_data = DecodeableExtraData::default();
    extra_data.game_key = Some(KEY_ROME_2);


    let data_len = reader.len().unwrap();
    let before = reader.read_slice(data_len as usize, true).unwrap();
    let mut data = GroupFormations::decode(&mut reader, &Some(extra_data)).unwrap();

    let mut extra_data = EncodeableExtraData::default();
    extra_data.game_key = Some(KEY_ROME_2);

    let mut after = vec![];
    data.encode(&mut after, &Some(extra_data)).unwrap();

    let mut writer = BufWriter::new(File::create(path_2).unwrap());
    writer.write_all(&after).unwrap();

    assert_eq!(before, after);
}

#[test]
fn test_encode_group_formation_shogun_2() {

    let path_1 = "../test_files/test_decode_group_formations_sho2.bin";
    let path_2 = "../test_files/test_encode_group_formations_sho2.bin";
    let mut reader = BufReader::new(File::open(path_1).unwrap());

    let mut extra_data = DecodeableExtraData::default();
    extra_data.game_key = Some(KEY_SHOGUN_2);


    let data_len = reader.len().unwrap();
    let before = reader.read_slice(data_len as usize, true).unwrap();
    let mut data = GroupFormations::decode(&mut reader, &Some(extra_data)).unwrap();

    let mut extra_data = EncodeableExtraData::default();
    extra_data.game_key = Some(KEY_SHOGUN_2);

    let mut after = vec![];
    data.encode(&mut after, &Some(extra_data)).unwrap();

    let mut writer = BufWriter::new(File::create(path_2).unwrap());
    writer.write_all(&after).unwrap();

    assert_eq!(before, after);
}
