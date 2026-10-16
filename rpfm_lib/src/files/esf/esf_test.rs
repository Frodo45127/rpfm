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
use std::io::Cursor;
use std::fs::File;

use crate::error::Result;
use crate::files::*;

use super::ESF;
use super::{BoolNode, ESFSignature, F32Node, I32Node, NodeType, RecordNode, RecordNodeFlags, U32Node};

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

/// Zero padding after the string tables is accepted, anything else is not.
#[test]
fn trailing_padding_after_the_tables_is_accepted() {
    let zero_padding = [0u8; 64];
    let junk = [0xAB; 64];

    for signature in [ESFSignature::CAAB, ESFSignature::CBAB] {
        let bytes = encode(&sample_esf(signature));

        // Real files can carry zero padding after the last section.
        assert!(
            decode_with_tail(&bytes, &zero_padding).is_ok(),
            "{signature} with zero padding after the tables should decode"
        );

        // Anything else still means the last section parsed to the wrong place.
        assert!(
            decode_with_tail(&bytes, &junk).is_err(),
            "{signature} with junk after the tables should fail"
        );
    }
}

/// Builds a record node out of its parts.
fn record(name: &str, version: u8, children: Vec<Vec<NodeType>>) -> NodeType {
    let mut node = RecordNode::default();
    node.set_record_flags(RecordNodeFlags::IS_RECORD_NODE);
    node.set_version(version);
    node.set_name(name.to_owned());
    node.set_children(children);

    NodeType::Record(Box::new(node))
}

/// Builds a small ESF tree that fills all three string tables.
fn sample_esf(signature: ESFSignature) -> ESF {
    let mut esf = ESF::default();
    esf.set_signature(signature);
    esf.set_unknown_1(0);
    esf.set_creation_date(1764961050);

    esf.set_root_node(record("ROOT", 1, vec![vec![
        NodeType::Ascii("hello".to_owned()),
        NodeType::Utf16("unicode".to_owned()),
        NodeType::U32(U32Node { value: 42, optimized: true }),
        NodeType::Bool(BoolNode { value: true, optimized: true }),
        NodeType::F32(F32Node { value: 1.5, optimized: false }),
        NodeType::U8Array(vec![1, 2, 3, 4, 5]),
        record("INNER", 2, vec![vec![
            NodeType::I32(I32Node { value: -7, optimized: false }),
            NodeType::Ascii("world".to_owned()),
        ]]),
    ]]));

    esf
}

/// Encodes an ESF into its bytes, keeping its signature.
fn encode(esf: &ESF) -> Vec<u8> {
    let mut esf = esf.clone();
    let mut bytes = vec![];
    esf.encode(&mut bytes, &None).unwrap();

    bytes
}

/// Decodes an ESF from `bytes`, with `tail` appended right after them.
fn decode_with_tail(bytes: &[u8], tail: &[u8]) -> Result<ESF> {
    let mut data = bytes.to_vec();
    data.extend_from_slice(tail);

    ESF::decode(&mut Cursor::new(data), &None)
}
