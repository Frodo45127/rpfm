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
use crate::startpos::startpos_test::test_startpos;

/// Builds a record from its name, flags and groups.
fn record(name: &str, flags: RecordNodeFlags, groups: Vec<Vec<NodeType>>) -> NodeType {
    let mut record = RecordNode::default();
    record.set_name(name.to_owned());
    record.set_version(1);
    record.set_record_flags(flags | RecordNodeFlags::IS_RECORD_NODE);
    record.set_children(groups);
    NodeType::Record(Box::new(record))
}

/// An `ITEM` record: a leaf, a record `A`, a leaf, an optional `OPT` record, a second `A` and a last leaf.
fn item(first: u16, with_optional: bool, last: u8) -> NodeType {
    let mut group = vec![NodeType::U16(first), record("A", RecordNodeFlags::empty(), vec![vec![NodeType::U8(9)]]), NodeType::U8(2)];
    if with_optional {
        group.push(record("OPT", RecordNodeFlags::empty(), vec![vec![]]));
    }
    group.push(record("A", RecordNodeFlags::empty(), vec![vec![NodeType::U8(9)]]));
    group.push(NodeType::U8(last));
    record("ITEM", RecordNodeFlags::empty(), vec![group])
}

/// A small startpos: a root with two leaves and an array of three items.
fn sample() -> ESF {
    let items = [(1, false, 3), (1, true, 3), (5, false, 4)].iter()
        .map(|&(first, optional, last)| vec![NodeType::Ascii(format!("key_{first}")), item(first, optional, last)])
        .collect();
    let root = record("ROOT", RecordNodeFlags::empty(), vec![vec![NodeType::U16(7), NodeType::Ascii("a".to_owned()), record("ITEMS", RecordNodeFlags::HAS_NESTED_BLOCKS, items)]]);
    let mut esf = ESF::default();
    esf.set_root_node(root);
    esf
}

/// Returns the slots of a path as (name or key, optional) pairs.
fn slot_names(layout: &StartposLayout, path: &str) -> Vec<(String, bool)> {
    layout.record(path).unwrap().slots().iter().map(|slot| match slot {
        Slot::Leaf { name, optional, .. } => (name.to_owned(), *optional),
        Slot::Record { key, optional, .. } => (key.to_owned(), *optional),
    }).collect()
}

#[test]
fn test_extraction_aligns_optional_records() {
    let layout = StartposLayout::from_esf(&sample()).unwrap();

    assert!(layout.record("ROOT/ITEMS").unwrap().array());
    assert_eq!(slot_names(&layout, "ROOT/ITEMS/ITEM"), vec![
        ("^+1".to_owned(), false),
        ("A".to_owned(), false),
        ("A+1".to_owned(), false),
        ("OPT".to_owned(), true),
        ("A@2".to_owned(), false),
        ("A@2+1".to_owned(), false),
    ]);
}

#[test]
fn test_defaults_are_the_most_common_values() {
    let layout = StartposLayout::from_esf(&sample()).unwrap();

    let defaults = layout.record("ROOT/ITEMS/ITEM").unwrap().slots().iter()
        .filter_map(|slot| match slot { Slot::Leaf { default, .. } => Some(default.clone()), Slot::Record { .. } => None })
        .collect::<Vec<_>>();

    assert_eq!(defaults, vec![NodeType::U16(1), NodeType::U8(2), NodeType::U8(3)]);
}

#[test]
fn test_builder_sets_fields_and_optional_records() {
    let layout = StartposLayout::from_esf(&sample()).unwrap();

    let mut builder = layout.builder("ROOT/ITEMS/ITEM").unwrap();
    builder.group().unwrap()
        .set_u32("^+1", 5).unwrap()
        .set_u32("A@2+1", 4).unwrap();
    let built = builder.build().unwrap();

    assert_eq!(NodeType::Record(Box::new(built)), item(5, false, 4));
    assert!(layout.builder("ROOT/ITEMS/ITEM").unwrap().group().unwrap().set_str("^+1", "text").is_err());
    assert!(layout.builder("ROOT/ITEMS/ITEM").unwrap().group().unwrap().set_record("A", None).is_err());
}

#[test]
fn test_reader_names_the_children_of_each_group() {
    let layout = StartposLayout::from_esf(&sample()).unwrap();
    let NodeType::Record(item) = item(5, true, 4) else { unreachable!() };

    let groups = layout.read("ROOT/ITEMS/ITEM", &item).unwrap();

    assert_eq!(groups.len(), 1);
    assert_eq!((groups[0].u32("^+1"), groups[0].u32("A@2+1")), (Some(5), Some(4)));
    assert!(groups[0].record("OPT").is_some());
    assert!(groups[0].u32("A").is_none());
    assert!(layout.read("ROOT/MISSING", &item).is_err());
}

#[test]
fn test_arrays_start_empty() {
    let layout = StartposLayout::from_esf(&sample()).unwrap();

    let mut items = layout.builder("ROOT/ITEMS").unwrap();
    assert!(items.group().is_err());
    items.push_group().unwrap().set_str("^+1", "key_x").unwrap();

    assert_eq!(items.build().unwrap().children().len(), 1);
}

#[test]
fn test_optional_leaves() {
    let short_item = record("ITEM", RecordNodeFlags::empty(), vec![vec![NodeType::U16(1), NodeType::U8(2)]]);
    let long_item = record("ITEM", RecordNodeFlags::empty(), vec![vec![NodeType::U16(1), NodeType::Ascii("extra".to_owned()), NodeType::U8(2)]]);
    let items = vec![vec![short_item.clone()], vec![short_item.clone()], vec![long_item]];
    let mut esf = ESF::default();
    esf.set_root_node(record("ROOT", RecordNodeFlags::empty(), vec![vec![record("ITEMS", RecordNodeFlags::HAS_NESTED_BLOCKS, items)]]));
    let layout = StartposLayout::from_esf(&esf).unwrap();

    assert_eq!(slot_names(&layout, "ROOT/ITEMS/ITEM"), vec![("^+1".to_owned(), false), ("^+2".to_owned(), true), ("^+3".to_owned(), false)]);
    assert_eq!(NodeType::Record(Box::new(layout.default_record("ROOT/ITEMS/ITEM").unwrap())), short_item);

    let mut builder = layout.builder("ROOT/ITEMS/ITEM").unwrap();
    builder.group().unwrap().set_str("^+2", "set").unwrap().unset("^+2").unwrap();
    assert!(builder.group().unwrap().unset("^+1").is_err());
}

/// Rebuilds a record from the layout, setting every leaf by name and every child record recursively.
fn rebuild(layout: &StartposLayout, record: &RecordNode, path: &str) -> RecordNode {
    let record_layout = layout.record(path).unwrap_or_else(|| panic!("no layout for {path}"));
    let mut builder = layout.builder(path).unwrap();

    for (index, group) in record.children().iter().enumerate() {
        let target = if record_layout.array() { builder.push_group().unwrap() } else { assert_eq!(index, 0); builder.group().unwrap() };

        // Clear the optional slots first: only the ones in this instance come back.
        for slot in record_layout.slots() {
            match slot {
                Slot::Record { key, optional: true, .. } => { target.set_record(key, None).unwrap(); },
                Slot::Leaf { name, optional: true, .. } => { target.unset(name).unwrap(); },
                _ => {},
            }
        }

        if record_layout.free_form() {
            for (key, node) in group_tokens(group) {
                match (key, node) {
                    (Some(key), NodeType::Record(child)) => { target.push_child(NodeType::Record(Box::new(rebuild(layout, child, &child_path(path, &key))))).unwrap(); },
                    (_, leaf) => { target.push_child(leaf.clone()).unwrap(); },
                }
            }
            continue;
        }

        let slots = record_layout.match_children(group).unwrap_or_else(|| panic!("{path}: a group doesn't fit the layout"));
        for (node, slot) in group.iter().zip(slots) {
            match (node, slot) {
                (NodeType::Record(child), Slot::Record { key, .. }) => { target.set_record(key, Some(rebuild(layout, child, &child_path(path, key)))).unwrap(); },
                (leaf, Slot::Leaf { name, .. }) => { target.set(name, leaf.clone()).unwrap(); },
                _ => panic!("{path}: a child matched a slot of another kind"),
            }
        }
    }

    builder.build().unwrap()
}

#[test]
#[ignore = "needs the game data, see startpos_test.rs"]
fn test_layout_rebuilds_the_game_startpos() {
    let startpos = test_startpos();
    let layout = StartposLayout::from_esf(startpos).unwrap();

    let NodeType::Record(root) = startpos.root_node() else { panic!("the root isn't a record") };
    let rebuilt = rebuild(&layout, root, root.name());
    assert!(rebuilt == **root, "the rebuilt startpos differs from the original");

    let path = std::env::temp_dir().join("rpfm_startpos_layout_test.ron");
    layout.save(&path).unwrap();
    let loaded = StartposLayout::load(&path).unwrap();
    std::fs::remove_file(&path).unwrap();
    assert_eq!(loaded, layout);
}

#[test]
fn test_long_records_are_free_form() {
    let long = record("LIST", RecordNodeFlags::empty(), vec![(0..=MAX_LAYOUT_SLOTS as u16).map(NodeType::U16).collect()]);
    let mut esf = ESF::default();
    esf.set_root_node(record("ROOT", RecordNodeFlags::empty(), vec![vec![long.clone()]]));
    let layout = StartposLayout::from_esf(&esf).unwrap();

    assert!(layout.record("ROOT/LIST").unwrap().free_form());
    assert!(layout.record("ROOT/LIST").unwrap().slots().is_empty());

    let mut builder = layout.builder("ROOT/LIST").unwrap();
    for value in 0..=MAX_LAYOUT_SLOTS as u16 {
        builder.group().unwrap().push_child(NodeType::U16(value)).unwrap();
    }
    assert_eq!(NodeType::Record(Box::new(builder.build().unwrap())), long);
    assert!(layout.builder("ROOT").unwrap().group().unwrap().push_child(NodeType::U8(1)).is_err());
}

#[test]
fn test_repeated_records_keep_their_contents_by_slot() {
    let numbered = |value: u8| record("STANCE", RecordNodeFlags::empty(), vec![vec![NodeType::U8(value)]]);
    let manager = record("MANAGER", RecordNodeFlags::empty(), vec![vec![numbered(0), numbered(1), numbered(2)]]);
    let mut esf = ESF::default();
    esf.set_root_node(record("ROOT", RecordNodeFlags::empty(), vec![vec![manager.clone(), manager.clone()]]));
    let layout = StartposLayout::from_esf(&esf).unwrap();

    assert_eq!(NodeType::Record(Box::new(layout.default_record("ROOT/MANAGER").unwrap())), manager);

    // The first occurrence matches the path's default, so it doesn't need its own.
    let defaults = layout.record("ROOT/MANAGER").unwrap().slots().iter()
        .map(|slot| matches!(slot, Slot::Record { default: Some(_), .. }))
        .collect::<Vec<_>>();
    assert_eq!(defaults, vec![false, true, true]);
}
