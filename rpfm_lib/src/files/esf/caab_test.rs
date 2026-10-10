//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

use std::io::Cursor;

use crate::files::{Decodeable, Encodeable};

use super::*;

/// Tests that string arrays survive a CAAB round-trip, including empty ones and ones followed by more nodes.
#[test]
fn test_encode_caab_string_arrays() {
    let mut root = RecordNode::default();
    root.set_name("ROOT".to_owned());
    root.set_record_flags(RecordNodeFlags::IS_RECORD_NODE);
    root.set_children(vec![vec![
        NodeType::Utf16Array(vec![]),
        NodeType::Utf16Array(vec!["first".to_owned(), "second".to_owned()]),
        NodeType::AsciiArray(vec!["third".to_owned()]),
        NodeType::Ascii("after".to_owned()),
    ]]);
    let mut data = ESF::default();
    data.set_signature(ESFSignature::CAAB);
    data.set_root_node(NodeType::Record(Box::new(root)));

    let mut encoded = vec![];
    data.encode(&mut encoded, &None).unwrap();
    let decoded = ESF::decode(&mut Cursor::new(encoded), &None).unwrap();

    assert_eq!(decoded.root_node(), data.root_node());
}
