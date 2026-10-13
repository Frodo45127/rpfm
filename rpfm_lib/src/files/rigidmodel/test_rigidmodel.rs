//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
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
use crate::files::rigidmodel::MaterialType;
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
fn test_encode_rigidmodel_v7() {
    let path_1 = "../test_files/test_decode_rigidmodel_v7.rigid_model_v2";
    let path_2 = "../test_files/test_encode_rigidmodel_v7.rigid_model_v2";
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

    assert_eq!(before, after);
}
/*
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
}*/

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

/// Decodes and encodes back a model of Attila, returning the decoded model.
fn round_trip_attila(name: &str) -> RigidModel {
    let before = std::fs::read(format!("../test_files/rigidmodel_att/{name}.rigid_model_v2")).unwrap();
    let mut data = RigidModel::decode(&mut std::io::Cursor::new(&before), &None).unwrap();

    let mut after = vec![];
    data.encode(&mut after, &None).unwrap();
    assert!(before == after, "{name} isn't written back as it was");

    data
}

#[test]
fn test_encode_rigidmodel_attila() {
    let mut names = std::fs::read_dir("../test_files/rigidmodel_att").unwrap()
        .map(|entry| entry.unwrap().path().file_stem().unwrap().to_string_lossy().to_string())
        .collect::<Vec<_>>();
    names.sort();

    assert_eq!(names.len(), 22);
    for name in &names {
        round_trip_attila(name);
    }
}

#[test]
fn test_decode_rigidmodel_attila_rope() {
    let data = round_trip_attila("v6_rope");
    let rope_meshes = data.lods()[0].mesh_blocks().iter()
        .filter(|block| *block.mesh().material_type() == MaterialType::Rope)
        .collect::<Vec<_>>();

    assert!(!rope_meshes.is_empty());
    for block in rope_meshes {
        let rope = block.material().rope();
        assert_eq!(rope.links().len() + 1, rope.nodes().len());
        assert!(block.mesh().vertices().is_empty());
        assert!(!block.mesh().extra_indices().is_empty());
    }
}

#[test]
fn test_decode_rigidmodel_attila_cloth_bone_padding() {
    let data = round_trip_attila("v5_weighted_cloth_collision_shape");
    let bones = data.lods()[0].mesh_blocks().iter()
        .filter(|block| *block.mesh().material_type() == MaterialType::WeightedCloth)
        .flat_map(|block| block.material().cloth_bones())
        .collect::<Vec<_>>();

    assert!(!bones.is_empty());
    assert!(bones.iter().all(|bone| bone.name().starts_with("bn_") || bone.name().starts_with("bone_")));
    assert!(bones.iter().any(|bone| !bone.name_padding().is_empty()));
}

#[test]
fn test_decode_rigidmodel_attila_borrowed_vertices() {
    let data = round_trip_attila("v6_static_point_light");
    let light = data.lods()[0].mesh_blocks().iter()
        .find(|block| *block.mesh().material_type() == MaterialType::StaticPointLight)
        .unwrap();

    assert!(!light.mesh().vertices().is_empty());
    assert!(!light.mesh().indices().is_empty());
}

#[test]
fn test_encode_rigidmodel_rope_link_count_mismatch() {
    let mut data = round_trip_attila("v6_rope");
    let block = data.lods_mut()[0].mesh_blocks_mut().iter_mut()
        .find(|block| *block.mesh().material_type() == MaterialType::Rope)
        .unwrap();
    block.material_mut().rope_mut().links_mut().pop();

    assert!(data.encode(&mut vec![], &None).is_err());
}
