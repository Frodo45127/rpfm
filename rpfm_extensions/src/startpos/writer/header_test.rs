//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

use super::*;
use crate::startpos::writer::u32_node;

#[test]
fn test_full_names_join_the_localised_names() {
    let texts = HashMap::from([
        ("names_name_1".to_owned(), "Alfred".to_owned()),
        ("names_name_2".to_owned(), "of Wessex".to_owned()),
        ("names_name_3".to_owned(), String::new()),
    ]);
    let names = |ids: &[&str]| ids.iter().map(|id| id.to_string()).collect::<Vec<_>>();

    assert_eq!(full_name(&texts, &names(&["1", "", "", ""])), "Alfred");
    assert_eq!(full_name(&texts, &names(&["1", "2", "3", "4"])), "Alfred of Wessex");
}

#[test]
fn test_estates_count_by_owner() {
    let record = |name: &str, children: Vec<Vec<NodeType>>| {
        let mut record = RecordNode::default();
        record.set_name(name.to_owned());
        record.set_children(children);
        NodeType::Record(Box::new(record))
    };
    let estate = |id: u32, owner: u32| record(ESTATE_RECORD, vec![vec![u32_node(id), u32_node(owner), NodeType::Ascii("estate_a".to_owned())]]);
    let NodeType::Record(world) = record("WORLD", vec![vec![record("ESTATES", vec![vec![estate(1, 7)], vec![estate(2, 7)], vec![estate(3, 9)]])]]) else { panic!() };

    assert_eq!(estates_by_owner(&world), HashMap::from([(7, 2), (9, 1)]));
}
