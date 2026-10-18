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
use crate::startpos::writer::{bool_node, u32_node};

/// Returns the children of a living family member, as the family tree writer lays them out.
fn member_nodes(id: u32, spouse: u32, children: &[u32], adopted: &[u32], bastard: bool, progenitor: bool) -> Vec<NodeType> {
    let list = |ids: &[u32]| std::iter::once(u32_node(ids.len() as u32)).chain(ids.iter().map(|id| u32_node(*id))).collect::<Vec<_>>();
    let mut nodes = vec![u32_node(id), bool_node(false), u32_node(0), u32_node(0), u32_node(0), u32_node(spouse), NodeType::Ascii("date".to_owned())];
    nodes.extend(list(&[]));
    nodes.extend(list(children));
    nodes.extend(list(adopted));
    nodes.extend([bool_node(false), NodeType::Ascii("date".to_owned()), u32_node(0), bool_node(true), bool_node(false), bool_node(bastard), bool_node(false)]);
    nodes.extend([u32_node(0), u32_node(u32::MAX), bool_node(false), bool_node(progenitor), bool_node(false), NodeType::Ascii(String::new())]);
    nodes
}

#[test]
fn test_family_members_are_read_in_writer_order() {
    let nodes = member_nodes(4, 9, &[5, 6], &[7], true, true);

    let (id, member) = read_member(&nodes.iter().collect::<Vec<_>>()).unwrap();

    assert_eq!(id, 4);
    assert_eq!(member.spouse, 9);
    assert!(member.former_spouses.is_empty());
    assert_eq!(member.children, vec![5, 6]);
    assert_eq!(member.adopted_children, vec![7]);
    assert!(member.bastard);
    assert!(member.progenitor);
    assert!(member.faction.is_none());
}

#[test]
fn test_cut_family_members_are_skipped() {
    let nodes = member_nodes(4, 9, &[5, 6], &[], false, false);

    assert!(read_member(&nodes[..10].iter().collect::<Vec<_>>()).is_none());
}

#[test]
fn test_soldiers_are_the_lowest_percentage_giving_the_same_men() {
    assert_eq!(soldiers(29, 40), 73);
    assert_eq!(soldiers(40, 40), 100);
    assert_eq!(soldiers(0, 40), 1);
    assert_eq!(soldiers(10, 0), 100);
}
