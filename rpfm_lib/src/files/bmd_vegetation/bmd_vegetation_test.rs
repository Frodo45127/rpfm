//---------------------------------------------------------------------------//
// Copyright (c) 2017-2024 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Module containing tests for decoding/encoding `BmdVegetation` files.

use std::io::{BufReader, BufWriter, Write};
use std::fs::File;

use crate::binary::ReadBytes;
use crate::files::*;

use super::BmdVegetation;

#[test]
fn test_encode_bmd_vegetation_v2() {
    let path_1 = "../test_files/fastbin/test_prefab_with_vegetation.bmd.vegetation";
    let path_2 = "../test_files/fastbin/encode_test_prefab_with_vegetation.bmd.vegetation";
    let mut reader = BufReader::new(File::open(path_1).unwrap());

    let decodeable_extra_data = DecodeableExtraData::default();

    let data_len = reader.len().unwrap();
    let before = reader.read_slice(data_len as usize, true).unwrap();
    let mut data = BmdVegetation::decode(&mut reader, &Some(decodeable_extra_data)).unwrap();
    let mut after = vec![];
    data.encode(&mut after, &None).unwrap();

    let mut writer = BufWriter::new(File::create(path_2).unwrap());
    writer.write_all(&after).unwrap();

    assert_eq!(before, after);
}
/*
#[test]
fn test_encode_bmd_vegetation_to_layer() {
    let path_1 = "../test_files/fastbin/test_prefab_with_vegetation.bmd.vegetation";
    let path_2 = "../test_files/fastbin/test_prefab_with_vegetation.layer";
    let mut reader = BufReader::new(File::open(path_1).unwrap());

    let decodeable_extra_data = DecodeableExtraData::default();
    let data = BmdVegetation::decode(&mut reader, &Some(decodeable_extra_data)).unwrap();

    let layer = data.to_layer().unwrap();
    dbg!(&layer);
    let mut writer = BufWriter::new(File::create(path_2).unwrap());
    writer.write_all(layer.as_bytes()).unwrap();
}*/

/*
#[test]
fn test_mass_decode() {
    let folder_path = "/home/frodo45127/Proyectos/rpfm_test_files2/prefabs/";
    let paths = files_from_subdir(&PathBuf::from(folder_path), true).unwrap();
    let mut failures = 0;
    let mut heigh_modes = HashSet::new();
    let decodeable_extra_data = Some(DecodeableExtraData::default());
    for path in &paths {
        //println!("{}", path.to_string_lossy());

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
                println!("\t{}", error);
                failures += 1;
            }
        }
    }

    println!("Total errors: {}", failures);
    dbg!(heigh_modes);
}
*/
