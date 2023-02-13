//---------------------------------------------------------------------------//
// Copyright (c) 2017-2023 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Module containing tests for decoding/encoding `ESF` files.

use std::io::{BufReader, BufWriter, Write};
use std::fs::File;

use crate::files::*;

use super::ESF;

#[test]
fn test_encode_esf_caab() {
    let path_1 = "../test_files/test_decode_esf_caab.esf";
    let path_2 = "../test_files/test_encode_esf_caab.esf";
    let mut reader = BufReader::new(File::open(path_1).unwrap());
    let mut writer = BufWriter::new(File::create(path_2).unwrap());

    let mut data = ESF::decode(&mut reader, &None).unwrap();
    let mut after = vec![];
    data.encode(&mut after, &None).unwrap();
    writer.write_all(&after).unwrap();

    let mut reader_2 = BufReader::new(File::open(path_2).unwrap());
    let data_2 = ESF::decode(&mut reader_2, &None).unwrap();

    // We have to compare the decoded files due to weird padding issues in cauleb128-encoded fields.
    assert_eq!(data, data_2);
}
