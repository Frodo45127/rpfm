//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

use std::collections::{BTreeSet, HashMap};

use super::*;
use crate::startpos::esf_nav::{as_record, child_record};
use crate::startpos::startpos_test::{array_items, compare, field, field_str, test_data_tables, test_startpos, world, world_factions, Report};
use crate::startpos::world::CampaignIndex;
use crate::startpos::world::legacy::legacy_test::{linked_character, name_ids};
use crate::startpos::world::writer::forces::ForcePlan;
use crate::startpos::world::writer::government::PostPlan;
use crate::startpos::world::writer::sight::SightPlan;
use crate::startpos::world::writer::writer_test::{game_plan, world_test_data};
use crate::startpos::world::writer::WorldInputs;

/// Share of the compared links and flags that must match the game's. The rest are the rows the game drops (one
/// family with conflicting rows) and the adulthood of one of its members.
const MIN_FAMILY_MATCH_RATIO: f64 = 0.99;

/// Share of the dead characters' details fields that must match the game's. The rest are their art sets (the game
/// draws them at random), family ids (numbered differently), loyalty and two values of unknown origin.
const MIN_DEAD_DETAILS_MATCH_RATIO: f64 = 0.92;

/// A family member's links and flags, read from a written `FAMILY_MEMBER`.
#[derive(Debug, Default)]
struct ReadMember {
    dead: bool,
    mother: u32,
    father: u32,
    spouse: u32,
    married: bool,
    lists: [BTreeSet<u32>; 3],
    flags: [bool; 4],
}

/// Reads a `FAMILY_MEMBER` record.
fn read_member(member: &RecordNode) -> (u32, ReadMember) {
    let leaves = member.children()[0].iter().filter(|node| !matches!(node, NodeType::Ascii(_)) && !matches!(node, NodeType::Record(record) if record.name() == "CHARACTER_DETAILS")).collect::<Vec<_>>();
    let u32_at = |index: usize| match leaves[index] { NodeType::U32(value) => *value.value(), node => panic!("not a u32: {node:?}") };
    let bool_at = |index: usize| match leaves[index] { NodeType::Bool(value) => *value.value(), node => panic!("not a bool: {node:?}") };
    let date_at = |index: usize| match leaves[index] { NodeType::Record(record) => !matches!(field(record, 0), NodeType::U32(value) if *value.value() == 0), node => panic!("not a date: {node:?}") };

    let mut read = ReadMember { dead: bool_at(1), mother: u32_at(3), father: u32_at(4), spouse: u32_at(5), married: date_at(6), ..Default::default() };
    let mut index = 7;
    for list in &mut read.lists {
        let count = u32_at(index) as usize;
        *list = (index + 1..index + 1 + count).map(u32_at).collect();
        index += 1 + count;
    }
    read.flags = [bool_at(index + 3), bool_at(index + 5), bool_at(index + 10), bool_at(index + 11)];
    (u32_at(0), read)
}

/// Reads every member of a family tree.
fn read_tree(tree: &RecordNode) -> HashMap<u32, ReadMember> {
    tree.children()[0].iter().filter_map(as_record).map(read_member).collect()
}

/// Returns the game startpos's family id of each table character: the living ones linked by name to the world's
/// characters, the dead ones by name to the tree's members holding details.
fn game_family_ids(index: &CampaignIndex) -> HashMap<u32, String> {
    let names = test_data_tables().table("start_pos_characters").unwrap().rows()
        .map(|row| (row.str("ID").unwrap().into_owned(), ["Name", "Surname", "clan_name", "other_name"].iter().map(|column| row.str(column).unwrap().into_owned()).filter(|id| !id.is_empty()).collect::<BTreeSet<_>>()))
        .collect::<HashMap<_, _>>();
    let game_world = world(test_startpos());
    let mut ids = HashMap::new();
    for faction in world_factions(game_world) {
        let key = field_str(faction, 1);
        for character in child_record(faction, "CHARACTER_ARRAY").map(|array| array_items(array, "CHARACTER")).unwrap_or_default() {
            let details = child_record(character, "CHARACTER_DETAILS").unwrap();
            if let (NodeType::U32(family), Some(id)) = (field(details, 29), linked_character(index, key, &names, &name_ids(character))) {
                ids.insert(*family.value(), id);
            }
        }
    }
    for member in child_record(game_world, "FAMILY_TREE").unwrap().children()[0].iter().filter_map(as_record) {
        let (NodeType::U32(family), NodeType::Bool(dead)) = (field(member, 0), field(member, 1)) else { continue };
        if !*dead.value() {
            continue;
        }
        let names_of_member = name_ids(member);
        let found = index.characters_of(field_str(member, 2)).filter(|entry| !entry.death_type().is_empty() && names_of_member.is_subset(&names[entry.id()])).collect::<Vec<_>>();
        if let [entry] = found.as_slice() {
            ids.insert(*family.value(), entry.id().to_owned());
        }
    }
    ids
}

#[test]
#[ignore = "needs the game data, see startpos_test.rs"]
fn test_family_tree_matches_the_game_startpos() {
    let (index, layout, settlements, region_map, legacy_overrides) = world_test_data();
    let inputs = WorldInputs { tables: test_data_tables(), index: &index, layout: &layout, settlements: &settlements, region_map: &region_map, legacy_overrides: &legacy_overrides };
    let plan = game_plan(&index);
    let forces = ForcePlan::new(inputs.tables, &index, &plan).unwrap();
    let posts = PostPlan::new(&inputs, &plan).unwrap();
    let sight = SightPlan::new(&inputs, &plan);
    let writer = CharacterWriter::new(&inputs, &plan, &forces, &posts, &sight).unwrap();

    // The tree's own leaves (the member count and the trailing list) are laid out like the game's.
    let (our_tree, their_tree) = (writer.write_family_tree().unwrap(), child_record(world(test_startpos()), "FAMILY_TREE").unwrap().clone());
    let tree_leaves = |tree: &RecordNode| tree.children()[0].iter().filter(|node| !matches!(node, NodeType::Record(_))).cloned().collect::<Vec<_>>();
    assert_eq!(tree_leaves(&our_tree), tree_leaves(&their_tree));
    assert_eq!(our_tree.version(), their_tree.version());

    let ours = read_tree(&our_tree);
    let theirs = read_tree(&their_tree);
    assert_eq!(ours.len(), theirs.len());

    // Compare members by table character, with their links translated to table characters too.
    let our_ids = writer.entry_family_ids.iter().map(|(entry, id)| (*id, entry.to_owned())).collect::<HashMap<_, _>>();
    let game_ids = game_family_ids(&index);
    let link = |ids: &HashMap<u32, String>, id: u32| if id == 0 { "-".to_owned() } else { ids.get(&id).cloned().unwrap_or_else(|| "generated".to_owned()) };
    let (mut matches, mut total) = (0, 0);
    for (game_id, entry) in &game_ids {
        let Some(our_id) = writer.entry_family_ids.get(entry) else { continue };
        let (ours, theirs) = (&ours[our_id], &theirs[game_id]);
        let mut compare = |a: String, b: String| { matches += usize::from(a == b); total += 1; };
        compare(format!("{}", ours.dead), format!("{}", theirs.dead));
        compare(link(&our_ids, ours.mother), link(&game_ids, theirs.mother));
        compare(link(&our_ids, ours.father), link(&game_ids, theirs.father));
        compare(link(&our_ids, ours.spouse), link(&game_ids, theirs.spouse));
        compare(format!("{}", ours.married), format!("{}", theirs.married));
        for (our_list, their_list) in ours.lists.iter().zip(&theirs.lists) {
            let our_set = our_list.iter().map(|id| link(&our_ids, *id)).collect::<BTreeSet<_>>();
            let their_set = their_list.iter().map(|id| link(&game_ids, *id)).collect::<BTreeSet<_>>();
            compare(format!("{our_set:?}"), format!("{their_set:?}"));
        }
        compare(format!("{:?}", ours.flags), format!("{:?}", theirs.flags));
    }
    let ratio = matches as f64 / total as f64;
    println!("{matches}/{total} family links and flags match");
    assert!(ratio >= MIN_FAMILY_MATCH_RATIO, "{matches}/{total} ({ratio:.4})");

    // The dead characters' details.
    let details = |tree: &RecordNode| tree.children()[0].iter().filter_map(as_record)
        .filter_map(|member| child_record(member, "CHARACTER_DETAILS").map(|details| (field(member, 0).clone(), details.clone())))
        .filter_map(|(id, details)| match id { NodeType::U32(id) => Some((*id.value(), details)), _ => None })
        .collect::<HashMap<_, _>>();
    let our_tree = writer.write_family_tree().unwrap();
    let (our_details, their_details) = (details(&our_tree), details(child_record(world(test_startpos()), "FAMILY_TREE").unwrap()));
    let mut report = Report::default();
    for (game_id, their_record) in &their_details {
        let our_record = &our_details[&writer.entry_family_ids[&game_ids[game_id]]];
        compare(&layout, our_record, their_record, &child_path(&child_path(FAMILY_TREE_PATH, "FAMILY_MEMBER"), "CHARACTER_DETAILS"), &mut report);
    }
    let (matches, total) = report.totals();
    println!("dead details: {matches}/{total} fields match. Worst:\n{}", report.worst(10));
    assert_eq!(our_details.len(), their_details.len());
    assert!(matches as f64 / total as f64 >= MIN_DEAD_DETAILS_MATCH_RATIO);
}
