//---------------------------------------------------------------------------//
// Copyright (c) 2017-2023 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Module containing tests for decoding/encoding RFiles.

use std::io::{BufReader, BufWriter, Write};
use std::fs::File;

use crate::binary::ReadBytes;
use crate::files::*;

#[test]
fn test_encode_rfile() {
    let path_1 = "../test_files/test_decode_rfile.pack";
    let path_2 = "../test_files/test_encode_rfile.pack";

    let mut reader = BufReader::new(File::open(path_1).unwrap());
    let data_len = reader.len().unwrap();
    let before = reader.read_slice(data_len as usize, true).unwrap();

    let mut decodeable_extra_data = DecodeableExtraData::default();
    decodeable_extra_data.lazy_load = true;
    decodeable_extra_data.file_name = Some("test_decode_rfile.pack");

    let mut rfile = RFile::new_from_file(path_1).unwrap();
    rfile.file_type = FileType::Pack;
    let mut decoded = rfile.decode(&Some(decodeable_extra_data.clone()), false, true).unwrap().unwrap();

    match decoded {
        RFileDecoded::Pack(ref mut pack) => {
            for (_, file) in pack.files_mut() {
                file.decode(&Some(decodeable_extra_data.clone()), true, true).unwrap();
            }

            let mut encodeable_extra_data = EncodeableExtraData::default();
            encodeable_extra_data.test_mode = true;

            let mut after = vec![];
            pack.encode(&mut after, &Some(encodeable_extra_data)).unwrap();

            let mut writer = BufWriter::new(File::create(path_2).unwrap());
            writer.write_all(&after).unwrap();

            assert_eq!(before, after);
        }
        _ => panic!("Incorrect file type"),
    }

}
