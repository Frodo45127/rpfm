//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Module containing tests for decoding/encoding `Image` files.

use std::io::BufReader;
use std::fs::File;

use crate::files::*;

use super::{decode_dds_rgba, Image};

#[test]
fn test_decode_dds_uncompressed_rgba() {
    let path = "../test_files/equip_diffuse.dds";
    let mut reader = BufReader::new(File::open(path).unwrap());

    let mut extra_data = DecodeableExtraData::default();
    extra_data.set_is_dds(true);

    let result = Image::decode(&mut reader, &Some(extra_data));
    match &result {
        Ok(image) => {
            assert!(image.converted_data().is_some(), "DDS should have been converted to PNG");
        }
        Err(e) => {
            panic!("Failed to decode DDS file: {e:?}");
        }
    }
}

#[test]
fn test_decode_dds_rgba_mipmaps() {
    let data = std::fs::read("../test_files/test_decode_att_bc1_mipmaps.dds").unwrap();

    // 32x32 with 6 mipmaps, down to 1x1.
    for (mipmap, size) in [(0, 32), (1, 16), (2, 8), (3, 4), (4, 2), (5, 1)] {
        let image = decode_dds_rgba(&data, mipmap).unwrap();
        assert_eq!(image.dimensions(), (size, size));
    }

    // Past the last mipmap, the last one.
    assert_eq!(decode_dds_rgba(&data, 99).unwrap().dimensions(), (1, 1));

    // Opaque BC1: every pixel has full alpha.
    assert!(decode_dds_rgba(&data, 0).unwrap().pixels().all(|pixel| pixel[3] == 255));
}

#[test]
fn test_decode_dds_rgba_not_dds() {
    assert!(decode_dds_rgba(b"not a dds file", 0).is_err());
}
