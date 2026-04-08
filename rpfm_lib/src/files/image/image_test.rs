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

use super::Image;

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
