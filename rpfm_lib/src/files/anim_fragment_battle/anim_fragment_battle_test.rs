//---------------------------------------------------------------------------//
// Copyright (c) 2017-2024 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Module containing tests for decoding/encoding `AnimFragment` files.

use std::io::{BufReader, BufWriter, Write};
use std::fs::File;

use crate::binary::ReadBytes;
use crate::files::*;

use super::AnimFragmentBattle;

#[test]
fn test_encode_anim_fragment_3k() {
    let path_1 = "../test_files/test_decode_anim_fragment_3k.bin";
    let path_2 = "../test_files/test_encode_anim_fragment_3k.bin";
    let mut reader = BufReader::new(File::open(path_1).unwrap());

    let games = SupportedGames::default();
    let game = games.game(KEY_THREE_KINGDOMS).unwrap();

    let mut extra_data = DecodeableExtraData::default();
    extra_data.game_info = Some(game);

    let data_len = reader.len().unwrap();
    let before = reader.read_slice(data_len as usize, true).unwrap();
    let mut data = AnimFragmentBattle::decode(&mut reader, &Some(extra_data)).unwrap();

    let extra_data = EncodeableExtraData::new_from_game_info(game);

    let mut after = vec![];
    data.encode(&mut after, &Some(extra_data)).unwrap();

    let mut writer = BufWriter::new(File::create(path_2).unwrap());
    writer.write_all(&after).unwrap();

    assert_eq!(before, after);
}

#[test]
fn test_encode_anim_fragment_wh2() {
    let path_1 = "../test_files/test_decode_anim_fragment_wh2.frg";
    let path_2 = "../test_files/test_encode_anim_fragment_wh2.frg";
    let mut reader = BufReader::new(File::open(path_1).unwrap());

    let games = SupportedGames::default();
    let game = games.game(KEY_WARHAMMER_2).unwrap();

    let mut extra_data = DecodeableExtraData::default();
    extra_data.game_info = Some(game);

    let data_len = reader.len().unwrap();
    let before = reader.read_slice(data_len as usize, true).unwrap();
    let mut data = AnimFragmentBattle::decode(&mut reader, &Some(extra_data)).unwrap();

    let extra_data = EncodeableExtraData::new_from_game_info(game);

    let mut after = vec![];
    data.encode(&mut after, &Some(extra_data)).unwrap();

    let mut writer = BufWriter::new(File::create(path_2).unwrap());
    writer.write_all(&after).unwrap();

    assert_eq!(before, after);
}

#[test]
fn test_encode_anim_fragment_wh3() {
    let path_1 = "../test_files/test_decode_anim_fragment_wh3.bin";
    let path_2 = "../test_files/test_encode_anim_fragment_wh3.bin";
    let mut reader = BufReader::new(File::open(path_1).unwrap());

    let games = SupportedGames::default();
    let game = games.game(KEY_WARHAMMER_3).unwrap();

    let mut extra_data = DecodeableExtraData::default();
    extra_data.game_info = Some(game);

    let data_len = reader.len().unwrap();
    let before = reader.read_slice(data_len as usize, true).unwrap();
    let mut data = AnimFragmentBattle::decode(&mut reader, &Some(extra_data)).unwrap();

    let extra_data = EncodeableExtraData::new_from_game_info(game);

    let mut after = vec![];
    data.encode(&mut after, &Some(extra_data)).unwrap();

    let mut writer = BufWriter::new(File::create(path_2).unwrap());
    writer.write_all(&after).unwrap();

    assert_eq!(before, after);
}
