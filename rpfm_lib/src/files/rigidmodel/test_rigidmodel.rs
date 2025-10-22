//---------------------------------------------------------------------------//
// Copyright (c) 2017-2024 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Module containing tests for decoding/encoding `RigidModel` files.

use std::io::{BufReader, BufWriter, Write};
use std::fs::File;

use crate::binary::ReadBytes;
//use crate::files::rigidmodel::MaterialType;
use crate::files::*;

use super::RigidModel;

#[test]
fn test_encode_rigidmodel_v8() {
    let path_1 = "../test_files/test_decode_rigidmodel_v8.rigid_model_v2";
    let path_2 = "../test_files/test_encode_rigidmodel_v8.rigid_model_v2";
    let mut reader = BufReader::new(File::open(path_1).unwrap());

    let data_len = reader.len().unwrap();
    let before = reader.read_slice(data_len as usize, true).unwrap();
    let mut data = RigidModel::decode(&mut reader, &None).unwrap();

    let mut after = vec![];
    data.encode(&mut after, &None).unwrap();

    let mut writer = BufWriter::new(File::create(path_2).unwrap());
    writer.write_all(&after).unwrap();

    assert_eq!(before, after);
}

#[test]
fn test_encode_rigidmodel_v6() {
    let path_1 = "../test_files/test_decode_rigidmodel_v6.rigid_model_v2";
    let path_2 = "../test_files/test_encode_rigidmodel_v6.rigid_model_v2";
    let mut reader = BufReader::new(File::open(path_1).unwrap());

    let data_len = reader.len().unwrap();
    let before = reader.read_slice(data_len as usize, true).unwrap();
    let mut data = RigidModel::decode(&mut reader, &None).unwrap();

    let mut after = vec![];
    data.encode(&mut after, &None).unwrap();

    let mut writer = BufWriter::new(File::create(path_2).unwrap());
    writer.write_all(&after).unwrap();

    //let mut json_writer = BufWriter::new(File::create(PathBuf::from("../test_files/test_encode_rigidmodel_v6.rigid_model_v2.json")).unwrap());
    //json_writer.write_all(serde_json::to_string_pretty(&data).unwrap().as_bytes()).unwrap();
    assert_eq!(before, after);
}

#[test]
fn test_encode_rigidmodel_wh3() {
    let paths = files_from_subdir(&PathBuf::from("/home/frodo45127/Proyectos/rpfm_test_files2/wh3_rigis/"), true).unwrap();
    let path_2 = "../test_files/test_encode_rigidmodel_recursive.rigid_model_v2";
    for path in &paths {
        if path.to_string_lossy().ends_with(".rigid_model_v2") {
            let mut reader = BufReader::new(File::open(path).unwrap());
            let decodeable_extra_data = DecodeableExtraData::default();

            let data_len = reader.len().unwrap();
            let before = reader.read_slice(data_len as usize, true).unwrap();
            dbg!(&path);
            let mut data = RigidModel::decode(&mut reader, &Some(decodeable_extra_data)).unwrap();

            let mut after = vec![];
            data.encode(&mut after, &None).unwrap();

            let mut writer = BufWriter::new(File::create(path_2).unwrap());
            writer.write_all(&after).unwrap();

            assert_eq!(before, after);
        }
    }
}

/*
#[test]
fn test_converter() {
    let path_1 = "../attila_mesh.rigid_model_v2";
    let path_2 = "../mesh.rigid_model_v2";
    let mut reader = BufReader::new(File::open(path_1).unwrap());

    let data_len = reader.len().unwrap();
    let before = reader.read_slice(data_len as usize, true).unwrap();
    let mut data = RigidModel::decode(&mut reader, &None).unwrap();


    for lod in data.lods_mut() {
        let mut new_meshes = vec![];
        for mesh in lod.mesh_blocks() {
            match mesh.mesh().material_type() {

                MaterialType::WeightedTextureBlend => {
                    let mut new_mesh = mesh.clone();

                    new_mesh.mesh_mut().set_material_type(MaterialType::RsTerrain);
                    new_meshes.push(new_mesh);
                },
                MaterialType::ProjectedDecalV4 => {
                    let mut new_mesh = mesh.clone();

                    for vertex in new_mesh.mesh_mut().vertices_mut() {
                        vertex.normal.w = 1.0;
                    }

                    new_mesh.mesh_mut().set_material_type(MaterialType::RsTerrain);
                    new_meshes.push(new_mesh);
                },
                _ => new_meshes.push(mesh.clone()),
            }
        }

        lod.set_mesh_blocks(new_meshes);
    }


    let mut after = vec![];
    data.encode(&mut after, &None).unwrap();

    let mut writer = BufWriter::new(File::create(path_2).unwrap());
    writer.write_all(&after).unwrap();

    //let mut json_writer = BufWriter::new(File::create(PathBuf::from("../test_files/test_encode_rigidmodel_v6_w.rigid_model_v2.json")).unwrap());
    //json_writer.write_all(serde_json::to_string_pretty(&data).unwrap().as_bytes()).unwrap();
}
*/
