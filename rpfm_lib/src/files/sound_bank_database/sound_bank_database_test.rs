//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

use std::io::{BufReader, BufWriter, Write};
use std::fs::File;

use crate::binary::ReadBytes;
use crate::files::*;

use super::SoundBankDatabase;

/*
#[test]
fn test_encode_sound_bank_database_empire() {
    let path_1 = "../test_files/test_decode_sound_bank_database_emp";
    let path_2 = "../test_files/test_encode_sound_bank_database_emp";
    let mut reader = BufReader::new(File::open(path_1).unwrap());

    let mut extra_data = DecodeableExtraData::default();
    extra_data.game_key = Some(KEY_EMPIRE);

    let data_len = reader.len().unwrap();
    let before = reader.read_slice(data_len as usize, true).unwrap();
    let mut data = SoundBankDatabase::decode(&mut reader, &Some(extra_data)).unwrap();

    let mut extra_data = EncodeableExtraData::default();
    extra_data.game_key = Some(KEY_EMPIRE);
    let mut after = vec![];
    data.encode(&mut after, &Some(extra_data)).unwrap();

    let mut writer = BufWriter::new(File::create(path_2).unwrap());
    writer.write_all(&after).unwrap();

    assert_eq!(before, after);
}

#[test]
fn test_encode_sound_bank_database_napoleon() {
    let path_1 = "../test_files/test_decode_sound_bank_database_nap";
    let path_2 = "../test_files/test_encode_sound_bank_database_nap";
    let mut reader = BufReader::new(File::open(path_1).unwrap());

    let mut extra_data = DecodeableExtraData::default();
    extra_data.game_key = Some(KEY_NAPOLEON);

    let data_len = reader.len().unwrap();
    let before = reader.read_slice(data_len as usize, true).unwrap();
    let mut data = SoundBankDatabase::decode(&mut reader, &Some(extra_data)).unwrap();

    let mut extra_data = EncodeableExtraData::default();
    extra_data.game_key = Some(KEY_NAPOLEON);
    let mut after = vec![];
    data.encode(&mut after, &Some(extra_data)).unwrap();

    let mut writer = BufWriter::new(File::create(path_2).unwrap());
    writer.write_all(&after).unwrap();

    assert_eq!(before, after);
}*/

#[test]
fn test_encode_sound_bank_database_shogun_2() {
    let path_1 = "../test_files/test_decode_sound_bank_database_sho2";
    let path_2 = "../test_files/test_encode_sound_bank_database_sho2";
    let mut reader = BufReader::new(File::open(path_1).unwrap());

    let games = SupportedGames::default();
    let game = games.game(KEY_SHOGUN_2).unwrap();

    let mut extra_data = DecodeableExtraData::default();
    extra_data.game_info = Some(game);

    let data_len = reader.len().unwrap();
    let before = reader.read_slice(data_len as usize, true).unwrap();
    let mut data = SoundBankDatabase::decode(&mut reader, &Some(extra_data)).unwrap();

    let extra_data = EncodeableExtraData::new_from_game_info(game);

    let mut after = vec![];
    data.encode(&mut after, &Some(extra_data)).unwrap();

    let mut writer = BufWriter::new(File::create(path_2).unwrap());
    writer.write_all(&after).unwrap();

    assert_eq!(before, after);
}
