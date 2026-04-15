//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
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

use super::GroupFormations;

#[test]
fn test_encode_group_formation_pharaoh() {

    let path_1 = "../test_files/test_decode_group_formations_pharaoh.bin";
    let path_2 = "../test_files/test_encode_group_formations_pharaoh.bin";
    let mut reader = BufReader::new(File::open(path_1).unwrap());

    let games = SupportedGames::default();
    let game = games.game(KEY_PHARAOH).unwrap();

    let mut extra_data = DecodeableExtraData::default();
    extra_data.game_info = Some(game);

    let data_len = reader.len().unwrap();
    let before = reader.read_slice(data_len as usize, true).unwrap();
    let mut data = GroupFormations::decode(&mut reader, &Some(extra_data)).unwrap();

    let extra_data = EncodeableExtraData::new_from_game_info(game);

    let mut after = vec![];
    data.encode(&mut after, &Some(extra_data)).unwrap();

    let mut writer = BufWriter::new(File::create(path_2).unwrap());
    writer.write_all(&after).unwrap();

    assert_eq!(before, after);
}

#[test]
fn test_encode_group_formation_warhammer_3() {

    let path_1 = "../test_files/test_decode_group_formations_wh3.bin";
    let path_2 = "../test_files/test_encode_group_formations_wh3.bin";
    let mut reader = BufReader::new(File::open(path_1).unwrap());

    let games = SupportedGames::default();
    let game = games.game(KEY_WARHAMMER_3).unwrap();

    let mut extra_data = DecodeableExtraData::default();
    extra_data.game_info = Some(game);

    let data_len = reader.len().unwrap();
    let before = reader.read_slice(data_len as usize, true).unwrap();
    let mut data = GroupFormations::decode(&mut reader, &Some(extra_data)).unwrap();

    let extra_data = EncodeableExtraData::new_from_game_info(game);

    let mut after = vec![];
    data.encode(&mut after, &Some(extra_data)).unwrap();

    let mut writer = BufWriter::new(File::create(path_2).unwrap());
    writer.write_all(&after).unwrap();

    assert_eq!(before, after);
}

#[test]
fn test_encode_group_formation_troy() {

    let path_1 = "../test_files/test_decode_group_formations_troy.bin";
    let path_2 = "../test_files/test_encode_group_formations_troy.bin";
    let mut reader = BufReader::new(File::open(path_1).unwrap());

    let games = SupportedGames::default();
    let game = games.game(KEY_TROY).unwrap();

    let mut extra_data = DecodeableExtraData::default();
    extra_data.game_info = Some(game);

    let data_len = reader.len().unwrap();
    let before = reader.read_slice(data_len as usize, true).unwrap();
    let mut data = GroupFormations::decode(&mut reader, &Some(extra_data)).unwrap();

    let extra_data = EncodeableExtraData::new_from_game_info(game);

    let mut after = vec![];
    data.encode(&mut after, &Some(extra_data)).unwrap();

    let mut writer = BufWriter::new(File::create(path_2).unwrap());
    writer.write_all(&after).unwrap();

    assert_eq!(before, after);
}

#[test]
fn test_encode_group_formation_three_kingdoms() {

    let path_1 = "../test_files/test_decode_group_formations_3k.bin";
    let path_2 = "../test_files/test_encode_group_formations_3k.bin";
    let mut reader = BufReader::new(File::open(path_1).unwrap());

    let games = SupportedGames::default();
    let game = games.game(KEY_THREE_KINGDOMS).unwrap();

    let mut extra_data = DecodeableExtraData::default();
    extra_data.game_info = Some(game);

    let data_len = reader.len().unwrap();
    let before = reader.read_slice(data_len as usize, true).unwrap();
    let mut data = GroupFormations::decode(&mut reader, &Some(extra_data)).unwrap();

    let extra_data = EncodeableExtraData::new_from_game_info(game);

    let mut after = vec![];
    data.encode(&mut after, &Some(extra_data)).unwrap();

    let mut writer = BufWriter::new(File::create(path_2).unwrap());
    writer.write_all(&after).unwrap();

    assert_eq!(before, after);
}

#[test]
fn test_encode_group_formation_thrones() {

    let path_1 = "../test_files/test_decode_group_formations_tob.bin";
    let path_2 = "../test_files/test_encode_group_formations_tob.bin";
    let mut reader = BufReader::new(File::open(path_1).unwrap());

    let games = SupportedGames::default();
    let game = games.game(KEY_THRONES_OF_BRITANNIA).unwrap();

    let mut extra_data = DecodeableExtraData::default();
    extra_data.game_info = Some(game);

    let data_len = reader.len().unwrap();
    let before = reader.read_slice(data_len as usize, true).unwrap();
    let mut data = GroupFormations::decode(&mut reader, &Some(extra_data)).unwrap();

    let extra_data = EncodeableExtraData::new_from_game_info(game);

    let mut after = vec![];
    data.encode(&mut after, &Some(extra_data)).unwrap();

    let mut writer = BufWriter::new(File::create(path_2).unwrap());
    writer.write_all(&after).unwrap();

    assert_eq!(before, after);
}

#[test]
fn test_encode_group_formation_attila() {

    let path_1 = "../test_files/test_decode_group_formations_att.bin";
    let path_2 = "../test_files/test_encode_group_formations_att.bin";
    let mut reader = BufReader::new(File::open(path_1).unwrap());

    let games = SupportedGames::default();
    let game = games.game(KEY_ATTILA).unwrap();

    let mut extra_data = DecodeableExtraData::default();
    extra_data.game_info = Some(game);

    let data_len = reader.len().unwrap();
    let before = reader.read_slice(data_len as usize, true).unwrap();
    let mut data = GroupFormations::decode(&mut reader, &Some(extra_data)).unwrap();

    let extra_data = EncodeableExtraData::new_from_game_info(game);

    let mut after = vec![];
    data.encode(&mut after, &Some(extra_data)).unwrap();

    let mut writer = BufWriter::new(File::create(path_2).unwrap());
    writer.write_all(&after).unwrap();

    assert_eq!(before, after);
}

#[test]
fn test_encode_group_formation_rome_2() {

    let path_1 = "../test_files/test_decode_group_formations_rom2.bin";
    let path_2 = "../test_files/test_encode_group_formations_rom2.bin";
    let mut reader = BufReader::new(File::open(path_1).unwrap());

    let games = SupportedGames::default();
    let game = games.game(KEY_ROME_2).unwrap();

    let mut extra_data = DecodeableExtraData::default();
    extra_data.game_info = Some(game);

    let data_len = reader.len().unwrap();
    let before = reader.read_slice(data_len as usize, true).unwrap();
    let mut data = GroupFormations::decode(&mut reader, &Some(extra_data)).unwrap();

    let extra_data = EncodeableExtraData::new_from_game_info(game);

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

    let games = SupportedGames::default();
    let game = games.game(KEY_SHOGUN_2).unwrap();

    let mut extra_data = DecodeableExtraData::default();
    extra_data.game_info = Some(game);

    let data_len = reader.len().unwrap();
    let before = reader.read_slice(data_len as usize, true).unwrap();
    let mut data = GroupFormations::decode(&mut reader, &Some(extra_data)).unwrap();

    let extra_data = EncodeableExtraData::new_from_game_info(game);

    let mut after = vec![];
    data.encode(&mut after, &Some(extra_data)).unwrap();

    let mut writer = BufWriter::new(File::create(path_2).unwrap());
    writer.write_all(&after).unwrap();

    assert_eq!(before, after);
}
