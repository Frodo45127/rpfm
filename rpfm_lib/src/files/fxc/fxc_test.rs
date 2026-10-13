//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Module containing tests for decoding/encoding `Fxc` files.

use std::io::Cursor;

use crate::files::*;

use super::Fxc;

#[test]
fn test_encode_fxc_v4() {
    let before = std::fs::read("../test_files/test_decode_att.fxc").unwrap();
    let mut data = Fxc::decode(&mut Cursor::new(&before), &None).unwrap();

    assert_eq!(data.defines().len(), 1);
    assert_eq!(data.defines()[0].name(), "SM_3_0");
    assert_eq!(data.defines()[0].value(), "");
    assert_eq!(*data.source_timestamp(), 1394533868);
    assert_eq!(data.includes().len(), 5);
    assert_eq!(data.includes()[0].path(), "FX\\FXConfig.hlsl");
    assert_eq!(*data.includes()[0].hash(), 0x531e_e5ec);
    assert_eq!(data.shader().len(), 920);
    assert!(data.shader().starts_with(b"DXBC"));

    let mut after = vec![];
    data.encode(&mut after, &None).unwrap();
    assert_eq!(before, after);
}

#[test]
fn test_decode_fxc_truncated_shader() {
    let mut bytes = std::fs::read("../test_files/test_decode_att.fxc").unwrap();
    bytes.pop();

    assert!(Fxc::decode(&mut Cursor::new(&bytes), &None).is_err());
}
