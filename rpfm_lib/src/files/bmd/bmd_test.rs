//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Module containing tests for decoding/encoding `Bmd` files.

use std::io::{BufReader, BufWriter, Write};
use std::fs::File;

use crate::binary::ReadBytes;
use crate::files::*;

use super::Bmd;

#[test]
fn test_encode_bmd_prefab() {
    let path_1 = "../test_files/test_decode.bmd";
    let path_2 = "../test_files/test_encode.bmd";
    let mut reader = BufReader::new(File::open(path_1).unwrap());

    let decodeable_extra_data = DecodeableExtraData::default();

    let data_len = reader.len().unwrap();
    let before = reader.read_slice(data_len as usize, true).unwrap();
    let mut data = Bmd::decode(&mut reader, &Some(decodeable_extra_data)).unwrap();

    let mut after = vec![];
    data.encode(&mut after, &None).unwrap();

    let mut writer = BufWriter::new(File::create(path_2).unwrap());
    writer.write_all(&after).unwrap();

    assert_eq!(before, after);
}

#[test]
fn test_encode_bmd_map_data_v23() {
    let path_1 = "../test_files/fastbin/v23_bmd_data.bin";
    let path_2 = "../test_files/fastbin/v23_encode_bmd_data.bin";
    let mut reader = BufReader::new(File::open(path_1).unwrap());

    let decodeable_extra_data = DecodeableExtraData::default();

    let data_len = reader.len().unwrap();
    let before = reader.read_slice(data_len as usize, true).unwrap();
    let mut data = Bmd::decode(&mut reader, &Some(decodeable_extra_data)).unwrap();
    let mut after = vec![];
    data.encode(&mut after, &None).unwrap();

    let mut writer = BufWriter::new(File::create(path_2).unwrap());
    writer.write_all(&after).unwrap();

    assert_eq!(before, after);
}

#[test]
fn test_encode_bmd_map_data_v23_old() {
    let path_1 = "../test_files/fastbin/v23_old_bmd_data.bin";
    let path_2 = "../test_files/fastbin/v23_encode_old_bmd_data.bin";
    let mut reader = BufReader::new(File::open(path_1).unwrap());

    let decodeable_extra_data = DecodeableExtraData::default();

    let data_len = reader.len().unwrap();
    let before = reader.read_slice(data_len as usize, true).unwrap();
    let mut data = Bmd::decode(&mut reader, &Some(decodeable_extra_data)).unwrap();
    let mut after = vec![];
    data.encode(&mut after, &None).unwrap();

    let mut writer = BufWriter::new(File::create(path_2).unwrap());
    writer.write_all(&after).unwrap();

    assert_eq!(before, after);
}

#[test]
fn test_encode_bmd_map_data_v24() {
    let path_1 = "../test_files/fastbin/v24_bmd_data.bin";
    let path_2 = "../test_files/fastbin/v24_encode_bmd_data.bin";
    let mut reader = BufReader::new(File::open(path_1).unwrap());

    let decodeable_extra_data = DecodeableExtraData::default();

    let data_len = reader.len().unwrap();
    let before = reader.read_slice(data_len as usize, true).unwrap();
    let mut data = Bmd::decode(&mut reader, &Some(decodeable_extra_data)).unwrap();
    let mut after = vec![];
    data.encode(&mut after, &None).unwrap();

    let mut writer = BufWriter::new(File::create(path_2).unwrap());
    writer.write_all(&after).unwrap();

    assert_eq!(before, after);
}

#[test]
fn test_encode_bmd_map_data_v27() {
    let path_1 = "../test_files/fastbin/v27_bmd_data.bin";
    let path_2 = "../test_files/fastbin/v27_encode_bmd_data.bin";
    let mut reader = BufReader::new(File::open(path_1).unwrap());

    let decodeable_extra_data = DecodeableExtraData::default();

    let data_len = reader.len().unwrap();
    let before = reader.read_slice(data_len as usize, true).unwrap();
    let mut data = Bmd::decode(&mut reader, &Some(decodeable_extra_data)).unwrap();
    let mut after = vec![];
    data.encode(&mut after, &None).unwrap();

    let mut writer = BufWriter::new(File::create(path_2).unwrap());
    writer.write_all(&after).unwrap();

    assert_eq!(before, after);
}

#[test]
fn test_encode_bmd_map_nogo_data() {
    let path_1 = "../test_files/fastbin/bmd_nogo_data.bin";
    let path_2 = "../test_files/fastbin/encode_bmd_nogo_data.bin";
    let mut reader = BufReader::new(File::open(path_1).unwrap());

    let decodeable_extra_data = DecodeableExtraData::default();

    let data_len = reader.len().unwrap();
    let before = reader.read_slice(data_len as usize, true).unwrap();
    let mut data = Bmd::decode(&mut reader, &Some(decodeable_extra_data)).unwrap();

    let mut after = vec![];
    data.encode(&mut after, &None).unwrap();

    let mut writer = BufWriter::new(File::create(path_2).unwrap());
    writer.write_all(&after).unwrap();

    assert_eq!(before, after);
}

#[test]
fn test_encode_bmd_map_catchment() {
    let path_1 = "../test_files/fastbin/catchment__bmd.bin";
    let path_2 = "../test_files/fastbin/encode_catchment__bmd.bin";
    let mut reader = BufReader::new(File::open(path_1).unwrap());

    let decodeable_extra_data = DecodeableExtraData::default();

    let data_len = reader.len().unwrap();
    let before = reader.read_slice(data_len as usize, true).unwrap();
    let mut data = Bmd::decode(&mut reader, &Some(decodeable_extra_data)).unwrap();

    let mut after = vec![];
    data.encode(&mut after, &None).unwrap();

    let mut writer = BufWriter::new(File::create(path_2).unwrap());
    writer.write_all(&after).unwrap();

    assert_eq!(before, after);
}

#[test]
fn test_encode_bmd_to_layer() {
    let path_1 = "../test_files/fastbin/prefabs/suerto_lzd_cliff_block.bmd";
    let mut reader = BufReader::new(File::open(path_1).unwrap());

    let decodeable_extra_data = DecodeableExtraData::default();
    let data = Bmd::decode(&mut reader, &Some(decodeable_extra_data)).unwrap();

    data.export_prefab_to_raw_data("test", None, &PathBuf::from("../test_files/fastbin/prefabs")).unwrap();
}

/*
#[test]
fn test_mass_decode() {
    let folder_path = "/home/frodo45127/Proyectos/rpfm_test_files2/prefabs/";
    let paths = files_from_subdir(&PathBuf::from(folder_path), true).unwrap();
    let mut failures = 0;
    let mut heigh_modes = HashSet::new();
    let decodeable_extra_data = Some(DecodeableExtraData::default());
    for path in &paths {
        if path.extension().unwrap() == "bmd" {
            let mut reader = BufReader::new(File::open(path).unwrap());
            match Bmd::decode(&mut reader, &decodeable_extra_data) {
                Ok(data) => {
                    for building in data.battlefield_building_list().buildings() {
                        heigh_modes.insert(building.height_mode().to_owned());
                    }

                    for prop in data.prop_list().props() {
                        heigh_modes.insert(prop.height_mode().to_owned());
                    }
                },
                Err(error) => {
                    println!("\t{}:", error);
                    println!("\t\t - {}", path.to_string_lossy());
                    failures += 1;
                    //break;
                }
            }
        }
    }

    println!("Total errors: {}", failures);
    dbg!(heigh_modes);
}
*/
