//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Module containing tests for decoding/encoding the FASTBIN0 list files of Attila.

use std::io::Cursor;

use crate::files::{Decodeable, Encodeable};


use super::*;

/// Decodes and encodes back a sample, returning the decoded file.
fn round_trip<T: Decodeable + Encodeable>(name: &str) -> T {
    let before = std::fs::read(format!("../test_files/fastbin_att/{name}.bin")).unwrap();
    let mut decoded = T::decode(&mut Cursor::new(&before), &None).unwrap();

    let mut after = vec![];
    decoded.encode(&mut after, &None).unwrap();
    assert!(before == after, "{name} isn't written back as it was");

    decoded
}

#[test]
fn test_encode_tree_list_files() {
    for (name, version, has_uk_1) in [("tree_list_v1", 1, false), ("tree_list_v2", 2, true), ("tree_list_v3", 3, false)] {
        let file: TreeListFile = round_trip(name);
        assert_eq!(*file.tree_list().serialise_version(), version);

        let trees = file.tree_list().tree_list();
        assert!(!trees.is_empty());
        assert!(trees.iter().all(|trees| trees.key().ends_with(".rigid_model_v2")));
        assert_eq!(trees.iter().flat_map(|trees| trees.value()).any(|tree| *tree.uk_1() != 0), has_uk_1);
    }
}

#[test]
fn test_encode_grass_list_file() {
    let file: GrassListFile = round_trip("grass_list_v1");
    let grass = file.grass_list().grass_list();

    assert!(!grass.is_empty());
    assert!(grass.iter().all(|grass| !grass.key().is_empty() && !grass.value().is_empty()));
}

#[test]
fn test_encode_civilian_files() {
    let deployments: CivilianDeploymentFile = round_trip("civilian_deployment");
    let deployments = deployments.civilian_deployment_list().civilian_deployment_list();
    assert!(!deployments.is_empty());
    assert!(deployments.iter().all(|deployment| !deployment.area().points().is_empty()));

    let shelters: CivilianShelterFile = round_trip("civilian_shelter");
    assert!(!shelters.civilian_shelter_list().civilian_shelter_list().is_empty());
}

#[test]
fn test_decode_fastbin_list_wrong_signature() {
    let mut bytes = std::fs::read("../test_files/fastbin_att/tree_list_v3.bin").unwrap();
    bytes[0] = b'X';
    assert!(TreeListFile::decode(&mut Cursor::new(&bytes), &None).is_err());
}
