//---------------------------------------------------------------------------//
// Copyright (c) 2017-2024 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Module containing tests for decoding/encoding `RigidModel` into/from GLTF files.
//!
//! Currently disabled because the support is not complete yet and some data is lost on the conversion.

/*
use std::io::{BufReader, BufWriter, Write};
use std::fs::File;

use rpfm_lib::binary::ReadBytes;
use rpfm_lib::files::*;

use super::*;

#[test]
fn test_encode_rigidmodel_v8_gltf() {
    let path_1 = "../test_files/test_decode_rigidmodel_v8.rigid_model_v2";
    let path_2 = "../test_files/test_encode_rigidmodel_v8_after_gltf.rigid_model_v2";
    let path_gltf_1 = "../test_files/test_encode_rigidmodel_v8.gltf";
    let path_gltf_2 = "../test_files/test_encode_rigidmodel_v8.bin";

    let mut reader = BufReader::new(File::open(path_1).unwrap());

    let data_len = reader.len().unwrap();
    let before = reader.read_slice(data_len as usize, true).unwrap();
    let data = RigidModel::decode(&mut reader, &None).unwrap();

    // Write the gltf to disk for debugging.
    let gltf = gltf_from_rigid(&data, &mut Dependencies::default()).unwrap();
    let mut writer_gltf = BufWriter::new(File::create(path_gltf_1).unwrap());
    //let mut writer_bin = BufWriter::new(File::create(path_gltf_2).unwrap());
    writer_gltf.write_all(gltf.as_json().to_string_pretty().unwrap().as_bytes()).unwrap();
    //writer_bin.write_all(&gltf.blob.clone().unwrap()).unwrap();

    let mut data_2 = rigid_from_gltf(&gltf).unwrap();
    let mut after = vec![];
    data_2.encode(&mut after, &None).unwrap();

    let mut writer = BufWriter::new(File::create(path_2).unwrap());
    writer.write_all(&after).unwrap();

    assert_eq!(before, after);
}*/
