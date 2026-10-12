//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

use rpfm_lib::files::esf::ESF;

use super::*;
use crate::startpos::esf_nav::child_record;

/// Builds a record from its name and its only group.
fn record(name: &str, group: Vec<NodeType>) -> NodeType {
    let mut record = RecordNode::default();
    record.set_name(name.to_owned());
    record.set_record_flags(RecordNodeFlags::IS_RECORD_NODE);
    record.set_children(vec![group]);
    NodeType::Record(Box::new(record))
}

/// A startpos whose episodic restrictions have a leaf and one script variable.
fn layout() -> StartposLayout {
    let variables = record(SCRIPT_VARIABLES, vec![record("old", vec![u32_node(BOOL_TYPE), bool_node(true)])]);
    let restrictions = record("EPISODIC_RESTRICTIONS", vec![bool_node(false), variables]);
    let model = record("CAMPAIGN_MODEL", vec![restrictions]);
    let env = record("CAMPAIGN_ENV", vec![model]);
    let mut startpos = ESF::default();
    startpos.set_root_node(record("CAMPAIGN_STARTPOS", vec![env]));
    StartposLayout::from_esf(&startpos).unwrap()
}

#[test]
fn test_variables_are_written_in_order_with_their_type_codes() {
    let variables = vec![
        ("flag".to_owned(), LuaSavedValue::Bool(false)),
        ("count".to_owned(), LuaSavedValue::Integer(-1)),
        ("text".to_owned(), LuaSavedValue::String("a".to_owned())),
    ];

    let restrictions = write_episodic_restrictions(&layout(), Some(&variables)).unwrap();

    let written = child_record(&restrictions, SCRIPT_VARIABLES).unwrap();
    assert_eq!(written.children(), &vec![vec![
        record("flag", vec![u32_node(BOOL_TYPE), bool_node(false)]),
        record("count", vec![u32_node(INTEGER_TYPE), i32_node(-1)]),
        record("text", vec![u32_node(STRING_TYPE), NodeType::Ascii("a".to_owned())]),
    ]]);
    assert_eq!(restrictions.children()[0][0], bool_node(false));
}

#[test]
fn test_layout_variables_are_kept_when_the_scripts_did_not_run() {
    let restrictions = write_episodic_restrictions(&layout(), None).unwrap();

    let written = child_record(&restrictions, SCRIPT_VARIABLES).unwrap();
    assert_eq!(written.children(), &vec![vec![record("old", vec![u32_node(BOOL_TYPE), bool_node(true)])]]);
}
