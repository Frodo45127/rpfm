//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Module containing tests for decoding/encoding `TileDatabase` files.

use std::io::{BufReader, BufWriter, Write};
use std::fs::File;

use crate::binary::ReadBytes;
use crate::files::*;

use super::TileDatabase;

#[test]
fn test_tile_database_att() {
    let path_1 = "../test_files/test_decode_tile_database.bin";
    let path_2 = "../test_files/test_encode_tile_database.bin";
    let mut reader = BufReader::new(File::open(path_1).unwrap());

    let mut decodeable_extra_data = DecodeableExtraData::default();
    decodeable_extra_data.file_name = Some("test_decode_tile_database.bin");

    let data_len = reader.len().unwrap();
    let before = reader.read_slice(data_len as usize, true).unwrap();
    let mut data = TileDatabase::decode(&mut reader, &Some(decodeable_extra_data)).unwrap();

    let mut after = vec![];
    data.encode(&mut after, &None).unwrap();

    let mut writer = BufWriter::new(File::create(path_2).unwrap());
    writer.write_all(&after).unwrap();

    assert_eq!(before, after);
}

/// Reads the Assembly Kit sample: its settings, and its tiles in file name order.
fn assembly_kit_sample() -> (String, Vec<String>) {
    let dir = "../test_files/tile_database_ak";
    let settings = std::fs::read_to_string(format!("{dir}/_settings.xml")).unwrap();
    let mut paths = std::fs::read_dir(format!("{dir}/TILES")).unwrap().map(|entry| entry.unwrap().path()).collect::<Vec<_>>();
    paths.sort();

    (settings, paths.iter().map(|path| std::fs::read_to_string(path).unwrap()).collect())
}

#[test]
fn test_tile_database_from_assembly_kit() {
    let (settings, tiles) = assembly_kit_sample();
    let tiles = tiles.iter().map(|tile| tile.as_str()).collect::<Vec<_>>();
    let mut database = TileDatabase::from_assembly_kit(&settings, &tiles, None, true).unwrap();

    // Larger tiles first.
    let locations = database.tiles().iter().map(|tile| tile.location().rsplit('\\').next().unwrap().to_owned()).collect::<Vec<_>>();
    assert_eq!(locations, ["8x8", "4x4", "3x1"]);
    assert_eq!(database.tile_sets().last().unwrap().name(), "user");

    // What BOB writes from the same sources.
    let mut encoded = vec![];
    database.encode(&mut encoded, &None).unwrap();
    assert_eq!(encoded, std::fs::read("../test_files/tile_database_ak/expected_tile_database.bin").unwrap());
}

#[test]
fn test_tile_database_from_assembly_kit_order() {
    let (settings, tiles) = assembly_kit_sample();
    let tiles = tiles.iter().map(|tile| tile.as_str()).collect::<Vec<_>>();
    let database = TileDatabase::from_assembly_kit(&settings, &tiles, None, false).unwrap();
    assert!(database.tile_sets().iter().all(|tile_set| tile_set.name() != "user"));

    let mut order = database.tiles().iter().map(|tile| tile.location().to_owned()).collect::<Vec<_>>();
    order.reverse();
    let reversed = TileDatabase::from_assembly_kit(&settings, &tiles, Some(&order), false).unwrap();
    assert_eq!(reversed.tiles().iter().map(|tile| tile.location().to_owned()).collect::<Vec<_>>(), order);

    order.pop();
    assert!(TileDatabase::from_assembly_kit(&settings, &tiles, Some(&order), false).is_err());
    assert!(TileDatabase::from_assembly_kit("<TILE_DATABASE>", &tiles, None, false).is_err());
}
