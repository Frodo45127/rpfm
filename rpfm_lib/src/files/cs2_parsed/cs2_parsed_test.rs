//---------------------------------------------------------------------------//
// Copyright (c) 2017-2024 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Module containing tests for decoding/encoding `Cs2Parsed` files.

use std::io::{BufReader, BufWriter, Write};
use std::fs::File;

use crate::binary::ReadBytes;
use crate::files::*;

use super::Cs2Parsed;

#[test]
fn test_encode_cs2_parsed() {
    let path_1 = "../test_files/test_wall.cs2.parsed";
    let path_2 = "../test_files/test_encode_wall.cs2.parsed";
    let mut reader = BufReader::new(File::open(path_1).unwrap());

    let decodeable_extra_data = DecodeableExtraData::default();

    let data_len = reader.len().unwrap();
    let before = reader.read_slice(data_len as usize, true).unwrap();
    let mut data = Cs2Parsed::decode(&mut reader, &Some(decodeable_extra_data)).unwrap();
    let mut after = vec![];

    data.encode(&mut after, &None).unwrap();

    let mut writer = BufWriter::new(File::create(path_2).unwrap());
    writer.write_all(&after).unwrap();

    assert_eq!(before, after);
}

#[test]
fn test_encode_cs2_corner_parsed() {
    let path_1 = "../test_files/test_wall_corner.cs2.parsed";
    let path_2 = "../test_files/test_encode_wall_corner.cs2.parsed";
    let mut reader = BufReader::new(File::open(path_1).unwrap());

    let decodeable_extra_data = DecodeableExtraData::default();

    let data_len = reader.len().unwrap();
    let before = reader.read_slice(data_len as usize, true).unwrap();
    let mut data = Cs2Parsed::decode(&mut reader, &Some(decodeable_extra_data)).unwrap();
    let mut after = vec![];

    data.encode(&mut after, &None).unwrap();

    let mut writer = BufWriter::new(File::create(path_2).unwrap());
    writer.write_all(&after).unwrap();

    assert_eq!(before, after);
}

#[test]
fn test_encode_cs2_parsed_v20_3k() {
    let path_1 = "../test_files/test_decode_v20_3k.cs2.parsed";
    let path_2 = "../test_files/test_encode_v20_3k.cs2.parsed";
    let mut reader = BufReader::new(File::open(path_1).unwrap());

    let decodeable_extra_data = DecodeableExtraData::default();

    let data_len = reader.len().unwrap();
    let before = reader.read_slice(data_len as usize, true).unwrap();
    let mut data = Cs2Parsed::decode(&mut reader, &Some(decodeable_extra_data)).unwrap();
    let mut after = vec![];

    data.encode(&mut after, &None).unwrap();

    let mut writer = BufWriter::new(File::create(path_2).unwrap());
    writer.write_all(&after).unwrap();

    assert_eq!(before, after);
}

#[test]
fn test_encode_cs2_parsed_v21() {
    let path_1 = "../test_files/test_decode_v21.cs2.parsed";
    let path_2 = "../test_files/test_encode_v21.cs2.parsed";
    let mut reader = BufReader::new(File::open(path_1).unwrap());

    let decodeable_extra_data = DecodeableExtraData::default();

    let data_len = reader.len().unwrap();
    let before = reader.read_slice(data_len as usize, true).unwrap();
    let mut data = Cs2Parsed::decode(&mut reader, &Some(decodeable_extra_data)).unwrap();
    let mut after = vec![];

    data.encode(&mut after, &None).unwrap();

    let mut writer = BufWriter::new(File::create(path_2).unwrap());
    writer.write_all(&after).unwrap();

    assert_eq!(before, after);
}

/*
#[test]
fn test_assladers_begone() {
    let base_path = "C:/Users/frodo/Desktop/assladers/rigidmodels/buildings";
    let dest_path = PathBuf::from("C:/Users/frodo/Desktop/assladers/rigidmodels/buildings_fixed");
    let paths = files_from_subdir(&PathBuf::from(base_path), true).unwrap();

    for path in &paths {
        if path.file_name().unwrap().to_string_lossy().to_string().ends_with(".cs2.parsed") {
            let mut reader = BufReader::new(File::open(&path).unwrap());

            let decodeable_extra_data = DecodeableExtraData::default();

            let data_len = reader.len().unwrap();
            let before = reader.read_slice(data_len as usize, true).unwrap();
            match Cs2Parsed::decode(&mut reader, &Some(decodeable_extra_data)) {
                Ok(mut data) => {
                    let mut after = vec![];

                    for piece in &mut data.pieces {
                        for destruct in &mut piece.destructs {
                            destruct.pipes.retain(|pipe| !pipe.name.contains("ladder"));
                        }
                    }

                    data.encode(&mut after, &None).unwrap();

                    if before != after {
                        println!("File edited: {}", dest_path.to_string_lossy().to_string());

                        let dest_path_end = path.strip_prefix(base_path).unwrap();
                        let dest_path = dest_path.join(dest_path_end);
                        let mut dest_path_parent = dest_path.to_path_buf();
                        dest_path_parent.pop();

                        DirBuilder::new().recursive(true).create(&dest_path_parent).unwrap();

                        let mut writer = BufWriter::new(File::create(&dest_path).unwrap());
                        writer.write_all(&after).unwrap();
                    }
                }
                Err(error) => {
                    println!("Failed to read file: {}, with error: {}", path.to_string_lossy().to_string(), error);

                }
            }
        }
    }
}
*/
