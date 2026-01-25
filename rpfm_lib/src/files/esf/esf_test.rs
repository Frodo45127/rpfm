//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Unit tests for ESF encoding and decoding.
//!
//! These tests verify round-trip fidelity: decode → encode → decode should
//! produce identical in-memory structures.
//!
//! # Test Strategy
//!
//! The tests compare decoded structures rather than raw binary output because
//! CAULEB128-encoded length fields may have different padding between the
//! original file and re-encoded output. The important invariant is that the
//! *data* is preserved, not the exact byte representation.
//!
//! # Test Files
//!
//! Tests require sample ESF files in `../test_files/`:
//! - `test_decode_esf_caab.esf`: Sample CAAB-format file
//! - `test_decode_esf_cbab.esf`: Sample CBAB-format file

use std::io::{BufReader, BufWriter, Write};
use std::fs::File;

use crate::files::*;

use super::ESF;

/// Tests CAAB format round-trip encoding fidelity.
///
/// Verifies that decoding a CAAB file, re-encoding it, and decoding again
/// produces the same in-memory structure. This ensures no data loss during
/// the encode/decode cycle.
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
    let mut data_2 = ESF::decode(&mut reader_2, &None).unwrap();
    let mut after = vec![];
    data_2.encode(&mut after, &None).unwrap();

    // Compare decoded structures, not binary output, due to CAULEB128 padding variations.
    assert_eq!(data, data_2);
}

/// Tests CBAB format round-trip encoding fidelity.
///
/// Verifies that decoding a CBAB file, re-encoding it, and decoding again
/// produces the same in-memory structure. This ensures no data loss during
/// the encode/decode cycle.
#[test]
fn test_encode_esf_cbab() {
    let path_1 = "../test_files/test_decode_esf_cbab.esf";
    let path_2 = "../test_files/test_encode_esf_cbab.esf";
    let mut reader = BufReader::new(File::open(path_1).unwrap());
    let mut writer = BufWriter::new(File::create(path_2).unwrap());

    let mut data = ESF::decode(&mut reader, &None).unwrap();
    let mut after = vec![];
    data.encode(&mut after, &None).unwrap();
    writer.write_all(&after).unwrap();

    let mut reader_2 = BufReader::new(File::open(path_2).unwrap());
    let mut data_2 = ESF::decode(&mut reader_2, &None).unwrap();
    let mut after = vec![];
    data_2.encode(&mut after, &None).unwrap();

    // Compare decoded structures, not binary output, due to CAULEB128 padding variations.
    assert_eq!(data, data_2);
}
